module Main where

import Prelude

import Data.Posix.Signal (Signal(..))
import Effect (Effect)
import Effect.Console as Console
import HTTPure as HTTPure
import Node.Process (onSignal)
import Routes.Routes as Routes

main :: Effect Unit
main = do
  closingHandler <- HTTPure.serve 8080 Routes.routes do
    Console.log $ "Skriptorium 🖋  up and running on 8080"

  let
    thankYou = "Thank you and good bye 👋"
  onSignal SIGINT $ closingHandler $ Console.log thankYou
  onSignal SIGTERM $ closingHandler $ Console.log thankYou
