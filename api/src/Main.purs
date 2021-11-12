module Main where

import Prelude
import Data.Posix.Signal (Signal(..))
import Effect (Effect)
import Effect.Console as Console
import HTTPure as HTTPure
import Node.Process (onSignal)

main :: Effect Unit
main = do 
  closingHandler <- HTTPure.serve 8080 (const $ HTTPure.ok "hello world!") do
    Console.log $ "Skriptorium 🖋 up and running on 8080"

  onSignal SIGINT $ closingHandler $ Console.log "Received SIGINT, stopping service now."  
  onSignal SIGTERM $ closingHandler $ Console.log "Received SIGTERM, stopping service now."
