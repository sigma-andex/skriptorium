module Environment (Env, readEnv) where

import Prelude

import Data.Maybe (Maybe)
import Data.Nullable (Nullable, toMaybe)
import Effect (Effect)
import Heterogeneous.Extrablatt.Rec (hmapKRec)

type Env f =
  { "NLPCLOUD_TOKEN" :: f String
  }

foreign import readEnvImpl :: Effect (Env Nullable)

readEnv :: Effect (Env Maybe)
readEnv = readEnvImpl <#> hmapKRec toMaybe
