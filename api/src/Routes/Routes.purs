module Routes.Routes (routes) where

import Data.Array (drop)
import Data.Maybe (Maybe(..))
import HTTPure ((!!))
import HTTPure as HTTPure

apiV1 :: (HTTPure.Request -> HTTPure.ResponseM) -> HTTPure.Request -> HTTPure.ResponseM
apiV1 router request = case request.path !! 1, request.path !! 2 of
  Just _, Just _ -> router request { path = drop 2 request.path }
  _, _ -> HTTPure.notFound

skriptioriumRoutes :: HTTPure.Request -> HTTPure.ResponseM
skriptioriumRoutes req@{ path: [ "tldr" ] } = HTTPure.ok "tldr"
skriptioriumRoutes _ = HTTPure.notFound

routes :: HTTPure.Request -> HTTPure.ResponseM
routes = apiV1 skriptioriumRoutes
