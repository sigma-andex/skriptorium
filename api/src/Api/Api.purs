module Api.Api (mkHandlers) where

import Prelude

import AI.NLPCloud as NLPCloud
import AI.OpenAI as OpenAI
import Api.Types (Classification, Handlers)
import Data.Array (head)
import Data.Either (Either(..), either, note)
import Data.Maybe (Maybe(..), maybe)
import Data.String (Pattern(..), split, stripPrefix, trim)
import Data.String.Base64 as B64
import Effect (Effect)
import Effect.Aff (Aff, error, throwError)
import Effect.Class.Console (log)
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
nlpCloudClassification client req@{ language, snippet } = do
  log $ "Got request" <> show req
  b64Decoded <- either (const $ throwError $ error "Not valid base64") pure $ B64.atob snippet
  templatedQuery <- nlpCloudQATemplate "templates/nlpcloud/classification.txt" b64Decoded
  let
    settings =
      { minLength: 10
      , maxLength: 50
      , endSequence: "\n###"
      , removeInput: true
      , numBeams: 1
      , noRepeatNgramSize: 0
      , numReturnSequences: 1
      , topK: 0.0
      , topP: 0.7
      , temperature: 1.0
      , repetitionPenalty: 1.0
      , lengthPenalty: 1.0
      }
  log $ "Sending query:\n" <> templatedQuery
  { "data": { generated_text } } <- NLPCloud.generation client settings $ NLPCloud.Query templatedQuery
  log $ "Received result:\n" <> generated_text
  pure $ Right { classification: trim generated_text, tldr: "" }

openAIClassification :: Token -> Classification
openAIClassification token req@{ language, snippet } = do
  log $ "Got request" <> show req
  b64Decoded <- either (const $ throwError $ error "Not valid base64") pure $ B64.atob snippet
  templatedQuery <- openAIQATemplate "templates/openai/classification.txt" b64Decoded
  let
    completionRequest = OpenAI.fillCompletionRequest
      { prompt: templatedQuery
      , max_tokens: 50
      , stop: ["###"] :: Array String
      , temperature: 0.0
      , top_p: 1.0
      , frequency_penalty: 0.0
      , presence_penalty: 0.0
      }
  log $ "Sending query:\n" <> templatedQuery
  eitherCompletion <- OpenAI.completion token completionRequest
  log $ either show (\r -> "Received result:\n" <> show r) eitherCompletion
  let
    clean = stripPrefix (Pattern "\nA: ") >>> map trim 
    dataError = error "Failed to extract data"
    extract s = case split (Pattern "---") s of 
      [classification, tldr] -> Right { classification: trim classification, tldr: trim tldr }
      _ -> Left dataError
    toClassification { choices } = case head choices of 
      Just { text } -> text # clean # note dataError >>= extract
      Nothing -> Left dataError
  pure $ eitherCompletion >>= toClassification

mkHandlers :: Engine -> Effect Handlers
mkHandlers (NLPCloud token) = NLPCloud.makeClient token <#> \client -> { classification: nlpCloudClassification client }
mkHandlers (OpenAI token) = pure $ { classification: openAIClassification token }
mkHandlers (Mock) = pure { classification: const (pure $ Right { classification: "mocktography", tldr: "tldr" }) }
