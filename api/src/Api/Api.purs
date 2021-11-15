module Api.Api (Handlers, mkHandlers, mkMockHandlers) where

import Prelude

import AI.NLPCloud as NLPCloud
import Api.Templates as Templates
import Api.Types (ClassificationRequest, ClassificationResponse)
import Data.Either (either)
import Data.String (trim)
import Data.String.Base64 as B64
import Effect (Effect)
import Effect.Aff (Aff, error, throwError)
import Environment (AppEnvironment)

classification :: NLPCloud.Client -> ClassificationRequest -> Aff ClassificationResponse
classification client { snippet } = do
  b64Decoded <- either (const $ throwError $ error "Not valid base64") pure $ B64.atob snippet
  templatedQuery <- Templates.qaTemplate "templates/classification.txt" b64Decoded <#> NLPCloud.Query
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

mkMockHandlers :: AppEnvironment -> Effect Handlers
mkMockHandlers _ = pure { classification: const (pure { classification: "mocktography" }) }
