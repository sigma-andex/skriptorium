module Api.Types
  ( Classification
  , ClassificationFile
  , ClassificationRequest
  , ClassificationResponse
  , Handlers
  , Selection
  , SelectionRequest
  , SelectionResponse
  ) where

import Data.Either (Either)
import Data.Maybe (Maybe)
import Effect.Aff (Aff, Error)

type ClassificationFile =
  { name :: Maybe String
  , content :: String
  }

type ClassificationRequest =
  { language :: Maybe String
  , files :: Array ClassificationFile
  }

type ClassificationResponse =
  { name :: String
  , tldr :: String
  , usage :: String
  , version :: Maybe String
  , license :: Maybe String
  }

type Classification = ClassificationRequest -> Aff (Either Error ClassificationResponse)

type SelectionRequest =
  { files :: Array String
  , language :: Maybe String
  }

type SelectionResponse =
  { files :: Array String
  }

type Selection = SelectionRequest -> Aff (Either Error SelectionResponse)

type Handlers =
  { classification :: Classification
  , selectFiles :: Selection
  }

