module Environment (readAppEnvironment, AppEnvironment) where

import Prelude

import Control.Monad.Error.Class (throwError)
import Data.Maybe (Maybe, maybe)
import Data.Nullable (Nullable, toMaybe)
import Data.String (toLower)
import Effect (Effect)
import Effect.Exception (error)
import Heterogeneous.Extrablatt.Rec (hmapKRec)
import Types (Token(..))

type AppEnvironment =
  { token :: Token
  , mock :: Boolean
  }

type Env f =
  { "NLPCLOUD_TOKEN" :: f String
  , "MOCK" :: f String
  }

foreign import readEnvImpl :: Effect (Env Nullable)

readEnv :: Effect (Env Maybe)
readEnv = readEnvImpl <#> hmapKRec toMaybe

readAppEnvironment :: Effect AppEnvironment
readAppEnvironment = do
  env <- readEnv
  token <- maybe (throwError (error "Missing NLPCloud token")) pure env."NLPCLOUD_TOKEN"
  let mock = maybe false identity $ env."MOCK" <#> \m -> toLower m == "true"
  pure { token: Token token, mock }
