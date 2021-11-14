module Api.Api (Handlers, mkHandlers) where

import Prelude

import AI.NLPCloud as NLPCloud
import Api.Templates.Classification (classificationTemplate)
import Api.Types (ClassificationRequest, ClassificationResponse)
import Data.Either (either)
import Data.Newtype (un)
import Data.String (trim)
import Data.String.Base64 as B64
import Effect (Effect)
import Effect.Aff (Aff, error, throwError)
import Effect.Class.Console (log)
import Environment (AppEnvironment)

classification :: NLPCloud.Client -> ClassificationRequest -> Aff ClassificationResponse
classification client { snippet } = do
  b64Decoded <- either (const $ throwError $ error "Not valid base64") pure $ B64.atob snippet
  let
    templatedQuery = NLPCloud.Query $ classificationTemplate b64Decoded
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
  log $ "Sending query:\n" <> un NLPCloud.Query templatedQuery
  { "data": { generated_text } } <- NLPCloud.generation client settings templatedQuery
  pure { classification: trim generated_text }

type Handlers =
  { classification :: ClassificationRequest -> Aff ClassificationResponse
  }

mkHandlers :: AppEnvironment -> Effect Handlers
mkHandlers { token } = do
  client <- NLPCloud.makeClient token
  pure
    { classification: classification client
    }
