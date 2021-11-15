module Main where

import Prelude

import Api.Api as Api
import Data.Posix.Signal (Signal(..))
import Effect (Effect)
import Effect.Console as Console
import Environment (readAppEnvironment)
import HTTPure as HTTPure
import Node.Process (onSignal)
import Routes.Routes as Routes

main :: Effect Unit
main = do
  app <- readAppEnvironment
  handlers <- if app.mock then Api.mkMockHandlers app else Api.mkHandlers app
  let
    port = 8080
    thankYou = "Thank you and good bye 👋"

  closingHandler <- HTTPure.serve port (Routes.routes handlers) do
    Console.log $ "Skriptorium 🖋  up and running on " <> (show port)

  onSignal SIGINT $ closingHandler $ Console.log thankYou
  onSignal SIGTERM $ closingHandler $ Console.log thankYou
