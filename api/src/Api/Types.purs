module Api.Types where

import Data.Either (Either)
import Data.Maybe (Maybe)
import Effect.Aff (Aff, Error)

type ClassificationRequest =
  { language :: Maybe String
  , snippet :: String
  }

type ClassificationResponse =
  { classification :: Maybe String
  }

type Classification = ClassificationRequest -> Aff (Either Error ClassificationResponse)

type Handlers =
  { classification :: Classification
  }

