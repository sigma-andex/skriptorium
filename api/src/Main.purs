module Main where

import Prelude

import Control.Monad.Error.Class (throwError)
import Data.Maybe (maybe)
import Data.Posix.Signal (Signal(..))
import Effect (Effect)
import Effect.Console as Console
import Effect.Exception (error)
import Environment (readEnv)
import HTTPure as HTTPure
import Node.Process (onSignal)
import Routes.Routes as Routes
import Types (Token(..))

type AppConfig =
  { token :: Token
  }

readAppConfig :: Effect AppConfig
readAppConfig = do
  env <- readEnv
  token <- maybe (throwError (error "Missing NLPCloud token")) pure env."NLPCLOUD_TOKEN"
  pure { token: Token token }

main :: Effect Unit
main = do
  app <- readAppConfig

  let
    port = 8080
    thankYou = "Thank you and good bye 👋"

  closingHandler <- HTTPure.serve port Routes.routes do
    Console.log $ "Skriptorium 🖋  up and running on " <> (show port)

  onSignal SIGINT $ closingHandler $ Console.log thankYou
  onSignal SIGTERM $ closingHandler $ Console.log thankYou
