module NLPCloud (Client, makeClient, generation) where

import Prelude

import Control.Promise (Promise)
import Control.Promise as Promise
import Data.Maybe (Maybe)
import Data.Nullable (Nullable, toNullable)
import Effect (Effect)
import Effect.Aff (Aff)
import Heterogeneous.Extrablatt.Rec (hmapKRec)
import Justifill (justifill)
import Justifill.Fillable (class Fillable, class FillableFields)
import Justifill.Justifiable (class Justifiable, class JustifiableFields)
import Prim.Row (class Union)
import Prim.RowList (class RowToList)

foreign import data Client :: Type

foreign import makeClient :: String -> Effect Client

type Response a
  = { data :: a }

type Generation
  = { generated_text :: String
    }

type InternalGenerationProps
  = { minLength :: Nullable Int
    , maxLength :: Nullable Int
    , endSequence :: Nullable String
    }

type GenerationProps
  = ( minLength :: Maybe Int
    , maxLength :: Maybe Int
    , endSequence :: Maybe String
    )

foreign import generationImpl :: Client -> InternalGenerationProps -> String -> (Effect (Promise (Response Generation)))

generation ::
  forall from fromRL via missing missingList.
  RowToList missing missingList =>
  FillableFields missingList () missing =>
  Union via missing GenerationProps =>
  RowToList from fromRL =>
  JustifiableFields fromRL from () via =>
  Client -> { | from } -> String -> Aff (Response Generation)
generation client props input = generationImpl client (toInternalGenerationProps props) input # Promise.toAffE
  where
  toGenerationProps :: { | from } -> { | GenerationProps }
  toGenerationProps = justifill

  toInternalGenerationProps :: { | from } -> InternalGenerationProps
  toInternalGenerationProps = toGenerationProps >>> hmapKRec toNullable
