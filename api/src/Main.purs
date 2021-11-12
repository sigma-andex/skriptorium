module Main where

import Prelude

import Data.Posix.Signal (Signal(..))
import Effect (Effect)
import Effect.Console as Console
import HTTPure as HTTPure
import Node.Process (onSignal)
import Routes.Routes as Routes
import Types (Token)

type Environment =
  { token :: Token
  }

main :: Effect Unit
main = do
  let
    port = 8080
    thankYou = "Thank you and good bye 👋"

  closingHandler <- HTTPure.serve port Routes.routes do
    Console.log $ "Skriptorium 🖋  up and running on " <> (show port)

  onSignal SIGINT $ closingHandler $ Console.log thankYou
  onSignal SIGTERM $ closingHandler $ Console.log thankYou
