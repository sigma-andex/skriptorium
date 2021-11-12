module Main where

import Prelude

import Data.Array (drop)
import Data.Maybe (Maybe(..))
import Data.Posix.Signal (Signal(..))
import Effect (Effect)
import Effect.Console as Console
import HTTPure ((!!))
import HTTPure as HTTPure
import Node.Process (onSignal)

apiV1 :: (HTTPure.Request -> HTTPure.ResponseM) -> HTTPure.Request -> HTTPure.ResponseM
apiV1 router request = case request.path !! 1, request.path !! 2 of
  Just _, Just _ -> router request { path = drop 2 request.path }
  _, _ -> HTTPure.notFound

routes :: HTTPure.Request -> HTTPure.ResponseM
routes req@{ path: [ "tldr" ] } = HTTPure.ok "tldr"
routes _ = HTTPure.notFound

main :: Effect Unit
main = do
  closingHandler <- HTTPure.serve 8080 (apiV1 routes) do
    Console.log $ "Skriptorium 🖋 up and running on 8080"

  onSignal SIGINT $ closingHandler $ Console.log "Received SIGINT, stopping service now."
  onSignal SIGTERM $ closingHandler $ Console.log "Received SIGTERM, stopping service now."
