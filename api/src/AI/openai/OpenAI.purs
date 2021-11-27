module AI.OpenAI where

import Prelude

import Data.Argonaut (class DecodeJson, JsonDecodeError, decodeJson, encodeJson, parseJson, printJsonDecodeError, stringify)
import Data.Bifunctor (lmap)
import Data.Either (Either(..))
import Data.Maybe (Maybe)
import Effect.Aff (Aff, Error, attempt, error)
import Effect.Class.Console (log)
import Justifill (justifill)
import Justifill.Fillable (class FillableFields)
import Justifill.Justifiable (class JustifiableFields)
import Milkis as M
import Milkis.Impl.Node (nodeFetch)
import Prim.Row (class Union)
import Prim.RowList (class RowToList)
import Types (Token(..))

fetch :: M.Fetch
fetch = M.fetch nodeFetch

type CompletionRequestPropsR f =
  ( logprobs :: f Int
  , max_tokens :: f Int
  , n :: f Int
  , prompt :: String
  , stop :: Array String
  , stream :: f Boolean
  , temperature :: f Number
  , top_p :: f Number
  , frequency_penalty :: f Number
  , presence_penalty :: f Number
  )

type CompletionRequestProps f
  =
  { | CompletionRequestPropsR f }

type CompletionResponsePropsR f =
  ( choices ::
      Array
        { finish_reason :: String
        , index :: Int
        , logprobs :: f Int
        , text :: String
        }
  , created :: f Int
  , id :: f String
  , model :: f String
  , object :: f String
  )

type CompletionResponseProps f = { | CompletionResponsePropsR f }

parseAndDecode :: forall t. DecodeJson t => String -> Either JsonDecodeError t
parseAndDecode = parseJson >=> decodeJson

fillCompletionRequest
  :: forall from fromRL via missing missingList
   . RowToList missing missingList
  => FillableFields missingList () missing
  => Union via missing (CompletionRequestPropsR Maybe)
  => RowToList from fromRL
  => JustifiableFields fromRL from () via
  => { | from }
  -> CompletionRequestProps Maybe
fillCompletionRequest = justifill

completion :: Token -> String -> CompletionRequestProps Maybe -> Aff (Either Error (CompletionResponseProps Maybe))
completion (Token token) model request = do
  let
    opts =
      { method: M.postMethod
      , body: stringify $ encodeJson request
      , headers: M.makeHeaders { "Content-Type": "application/json", "Authorization": "Bearer " <> token }
      }
  eitherResponse <- attempt $ fetch (M.URL $ "https://api.openai.com/v1/engines/" <> model <> "/completions") opts
  case eitherResponse of
    Right response | M.statusCode response >= 200 && M.statusCode response < 400 -> do
      body <- M.text response
      log $ "Received response:\n" <> body
      pure $ lmap (printJsonDecodeError >>> error) $ parseAndDecode body
    Right response -> pure $ Left $ error $ "Server responded with status " <> (show $ M.statusCode response)
    Left e -> pure (Left e)
