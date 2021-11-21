module Environment (readAppEnvironment, AppEnvironment, Engine(..)) where

import Prelude

import Control.Monad.Error.Class (throwError)
import Data.Maybe (Maybe(..))
import Data.Nullable (Nullable, toMaybe)
import Effect (Effect)
import Effect.Exception (error)
import Heterogeneous.Extrablatt.Rec (hmapKRec)
import Types (Token(..))

data Engine = Mock | NLPCloud Token | OpenAI Token

type AppEnvironment =
  { engine :: Engine
  }

type Env f =
  { "ENGINE" :: f String
  , "NLPCLOUD_TOKEN" :: f String
  , "OPENAI_TOKEN" :: f String
  }

foreign import readEnvImpl :: Effect (Env Nullable)

readEnv :: Effect (Env Maybe)
readEnv = readEnvImpl <#> hmapKRec toMaybe

getOrThrow :: String -> Maybe Engine -> Effect Engine
getOrThrow _ (Just engine) = pure engine
getOrThrow msg Nothing = throwError (error msg)

readAppEnvironment :: Effect AppEnvironment
readAppEnvironment = do
  env <- readEnv
  engine <- case env."ENGINE" of
    Just "mock" -> pure $ Mock
    Just "nlpcloud" -> getOrThrow "Missing env var NLPCLOUD_TOKEN for engine nlpcloud." $
      env."NLPCLOUD_TOKEN" <#> (Token >>> NLPCloud)
    Just "openai" -> getOrThrow "Missing env var OPENAI_TOKEN for engine openai." $
      env."OPENAI_TOKEN" <#> (Token >>> OpenAI)
    Just _ -> throwError $ error "Env var ENGINE should be one of mock, nlpcloud or openai."
    Nothing -> throwError $ error "Missing env var ENGINE."
  pure { engine }
