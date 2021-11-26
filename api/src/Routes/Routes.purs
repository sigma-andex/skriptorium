module Routes.Routes (routes) where

import Prelude

import Api.Types as ApiTypes
import Data.Argonaut (class DecodeJson, class EncodeJson, JsonDecodeError, decodeJson, encodeJson, parseJson, stringify)
import Data.Array (drop)
import Data.Either (Either(..))
import Data.Maybe (Maybe(..))
import Data.Tuple (Tuple(..))
import Debug (spy)
import Effect.Aff (Aff, Error)
import Effect.Class.Console (log)
import HTTPure ((!!))
import HTTPure as HTTPure

parseAndDecode :: forall elem. DecodeJson elem => String -> Either JsonDecodeError elem
parseAndDecode = parseJson >=> decodeJson

encodeAndStringify :: forall elem. EncodeJson elem => elem -> String
encodeAndStringify = encodeJson >>> stringify

type ErrorOr r = Either JsonDecodeError r

apiV1 :: (HTTPure.Request -> HTTPure.ResponseM) -> HTTPure.Request -> HTTPure.ResponseM
apiV1 router request = case request.path !! 1, request.path !! 2 of
  Just _, Just _ -> router request { path = drop 2 request.path }
  _, _ -> HTTPure.notFound

defaultHandleRequest
  :: forall apiRequest apiResponse
   . DecodeJson apiRequest
  => EncodeJson apiResponse
  => (apiRequest -> Aff (Either Error apiResponse))
  -> HTTPure.Request
  -> HTTPure.ResponseM
defaultHandleRequest handle request = case parseAndDecode request.body :: ErrorOr apiRequest of
  Right apiRequest ->
    do
      eitherApiResponse <- handle apiRequest
      case eitherApiResponse of
        Left err -> do
          log $ "An internal error occured: " <> show err
          HTTPure.internalServerError' jsonHeaders $ encodeAndStringify { error: "An internal server error occured. Please try again later" }
        Right apiResponse -> HTTPure.ok' jsonHeaders $ spy "sending json" $ encodeAndStringify apiResponse
  Left err -> do
    log $ "Got invalid request" <> show err
    HTTPure.badRequest' jsonHeaders $ encodeAndStringify { error: "Unable to parse request format." }
  where
  jsonHeaders = HTTPure.headers
    [ Tuple "Content-Type" "application/json"
    ]

skriptioriumRoutes :: ApiTypes.Handlers -> HTTPure.Request -> HTTPure.ResponseM
skriptioriumRoutes { classification } request@{ path: [ "classification" ], method: HTTPure.Post } = defaultHandleRequest classification request
skriptioriumRoutes { selectFiles } request@{ path: [ "select-files" ], method: HTTPure.Post } = defaultHandleRequest selectFiles request
skriptioriumRoutes _ _ = HTTPure.notFound

routes :: ApiTypes.Handlers -> HTTPure.Request -> HTTPure.ResponseM
routes handlers = apiV1 (skriptioriumRoutes handlers)
