module Api.Api (mkHandlers) where

import Prelude

import AI.NLPCloud as NLPCloud
import AI.OpenAI as OpenAI
import Api.Types (Classification, Handlers, Selection)
import Control.Bind (bindFlipped)
import Data.Array (filter, foldl, head)
import Data.Either (Either(..), either, note)
import Data.Maybe (Maybe(..), maybe)
import Data.String (Pattern(..), split, trim)
import Data.String.Base64 as B64
import Data.String.Common (joinWith)
import Data.String.Utils (includes)
import Data.Traversable (traverse)
import Effect (Effect)
import Effect.Aff (Aff, Error, error, throwError)
import Effect.Class.Console (log, logShow)
import Environment (Engine(..))
import Node.Encoding (Encoding(..))
import Node.FS.Aff (readTextFile)
import Node.Path (FilePath)
import Types (Token)

nlpCloudQATemplate :: FilePath -> String -> Aff String
nlpCloudQATemplate fp snippet = readTextFile UTF8 fp <#> \template -> trim template <> "\nQ: \n" <> trim snippet <> "\nA: "

openAIQATemplate :: FilePath -> String -> Aff String
openAIQATemplate fp snippet = readTextFile UTF8 fp <#> \template -> trim template <> "\nQ: \n" <> trim snippet

nlpCloudClassification :: NLPCloud.Client -> Classification
nlpCloudClassification client req@{ language, files } = do
  pure $ Right { name: "", tldr: "", usage: "", version: Nothing, license: Nothing }

extractFirstChoice :: OpenAI.CompletionResponseProps Maybe -> Either Error String
extractFirstChoice { choices } = head choices <#> (_.text >>> trim) # note (error "No choice available")

extractSingleAnswer :: String -> Maybe String
extractSingleAnswer s = split (Pattern ("Q:")) s # head <#> trim

openAIClassification :: Token -> Classification
openAIClassification token req@{ language, files } = do
  log $ "Got request" <> show req
  b64Decoded <- either (const $ throwError $ error "#Not valid base64") pure $ traverse (\{ name, content } -> B64.atob content <#> \decoded -> { name, decoded }) files
  let
    langToName lang = case language of
      Just "rs" -> "Rust "
      _ -> ""

    contentPrefix lang = "The following code snippets are from a " <> langToName lang <> "project."

    qaPrefix lang = "The following code snippets are from a " <> langToName lang <> "project. Write the answer of question Q: into A: "

    header (Just name) = "--- " <> name <> " ---\n"
    header Nothing = "--- ---\n"
    concatenate acc { name, decoded } = acc <> (header name) <> decoded <> "\n\n"

    concatenated = foldl concatenate "" b64Decoded

    nameQuestion = "\nQ: What is the name of this project?\nA:"
    tldrQuestion = "\nWhat is this project about?\n"
    usageQuestion = "\nHow can I use this project?\n"
    versionQuestion = "\nQ: What is the version of this project?\nA:"
    licenseQuestion = "\nQ: What is the license of this project?\nA:"

    mkContentQuery question = contentPrefix language <> concatenated <> separator <> question
    mkQaQuery question = qaPrefix language <> concatenated <> separator <> question

    nameQuery = mkQaQuery nameQuestion
    tldrQuery = mkContentQuery tldrQuestion
    usageQuery = mkContentQuery usageQuestion
    versionQuery = mkQaQuery versionQuestion
    licenseQuery = mkQaQuery licenseQuestion

    contentRequest query = OpenAI.fillCompletionRequest
      { prompt: query
      , max_tokens: 70
      , stop: [ separator ] :: Array String
      , temperature: 0.0
      , top_p: 1.0
      , n: 1
      , frequency_penalty: 0.0
      , presence_penalty: 0.6
      }

    qaRequest query = OpenAI.fillCompletionRequest
      { prompt: query
      , max_tokens: 10
      , stop: [ separator ] :: Array String
      , temperature: 0.0
      , top_p: 1.0
      , n: 1
      , frequency_penalty: 0.0
      , presence_penalty: 0.4
      }

  log $ "Sending query:\n" <> tldrQuery
  eitherTldr <- OpenAI.completion token (contentRequest tldrQuery) <#> bindFlipped extractFirstChoice
  eitherUsage <- OpenAI.completion token (contentRequest usageQuery) <#> bindFlipped extractFirstChoice
  eitherName <- OpenAI.completion token (qaRequest nameQuery) <#> bindFlipped (extractFirstChoice >>> map (\a -> extractSingleAnswer a # maybe a identity))
  eitherVersion <- OpenAI.completion token (qaRequest versionQuery) <#> bindFlipped (extractFirstChoice >>> map (\a -> extractSingleAnswer a))
  eitherLicense <- OpenAI.completion token (qaRequest licenseQuery) <#> bindFlipped (extractFirstChoice >>> map (\a -> extractSingleAnswer a))
  log $ either show (\r -> "Received name:\n" <> show r) eitherName
  log $ either show (\r -> "Received tldr:\n" <> show r) eitherTldr
  log $ either show (\r -> "Received usage:\n" <> show r) eitherUsage
  log $ either show (\r -> "Received version:\n" <> show r) eitherVersion
  log $ either show (\r -> "Received license:\n" <> show r) eitherLicense
  let
    result = { name: _, tldr: _, usage: _, version: _, license: _ } <$> eitherName <*> eitherTldr <*> eitherUsage <*> eitherVersion <*> eitherLicense
  log $ "Sending result:\n" <> show result
  pure $ result

separator :: String
separator = "\n-----\n"

openAISelection :: Token -> Selection
openAISelection token req@{ files } = do
  log $ "Got request" <> show req
  let
    tldrQuery = joinWith "\n" files <> separator <> "# What are the three most important files? List them with their full path.\n"

    completionRequest = OpenAI.fillCompletionRequest
      { prompt: tldrQuery
      , max_tokens: 50
      , stop: [ separator ] :: Array String
      , temperature: 0.0
      , top_p: 1.0
      , frequency_penalty: 0.0
      , presence_penalty: 0.0
      }
  log $ "Sending query:\n" <> tldrQuery
  eitherCompletion <- OpenAI.completion token completionRequest
  log $ either show (\r -> "Received result:\n" <> show r) eitherCompletion
  let
    clean = trim
    dataError = error "Failed to extract data"
    extract potentialPaths =
      let
        selectedFiles = filter (\file -> includes file potentialPaths) files
      in
        Right { files: selectedFiles }
    toSelection { choices } = case head choices of
      Just { text } -> text # clean # extract
      Nothing -> Left dataError
  pure $ eitherCompletion >>= toSelection

mkHandlers :: Engine -> Effect Handlers
mkHandlers (NLPCloud token) = NLPCloud.makeClient token <#> \client -> { classification: nlpCloudClassification client, selectFiles: const (pure $ Right { files: [] }) }
mkHandlers (OpenAI token) = pure $ { classification: openAIClassification token, selectFiles: openAISelection token }
mkHandlers (Mock) = pure { classification, selectFiles }
  where
  classification request = do
    logShow request
    pure $ Right { name: "mocktography", tldr: "tldr", usage: "", version: Just "v0.3.0", license: Just "MIT" }

  selectFiles request = do
    logShow request
    pure $ Right { files: request.files }
