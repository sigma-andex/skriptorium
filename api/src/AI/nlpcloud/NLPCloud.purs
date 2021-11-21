module AI.NLPCloud (Client, Query(..), makeClient, generation) where

import Prelude

import Control.Promise (Promise)
import Control.Promise as Promise
import Data.Maybe (Maybe)
import Data.Newtype (un, class Newtype)
import Data.Nullable (Nullable, toNullable)
import Effect (Effect)
import Effect.Aff (Aff)
import Heterogeneous.Extrablatt.Rec (hmapKRec)
import Justifill (justifill)
import Justifill.Fillable (class FillableFields)
import Justifill.Justifiable (class JustifiableFields)
import Prim.Row (class Union)
import Prim.RowList (class RowToList)
import Types (Token(..))

foreign import data Client :: Type

newtype Query = Query String

instance Newtype Query String
derive newtype instance Show Query
derive newtype instance Eq Query

foreign import makeClientImpl :: String -> Effect Client

makeClient :: Token -> Effect Client
makeClient = un Token >>> makeClientImpl

type Response a
  = { data :: a }

type Generation
  =
  { generated_text :: String
  }

type GenerationPropsR :: forall k. (Type -> k) -> Row k
type GenerationPropsR f
  =
  ( minLength :: f Int
  , maxLength :: f Int
  , lengthNoInput :: f Boolean
  , endSequence :: f String
  , removeInput :: f Boolean
  , doSample :: f Boolean
  , numBeams :: f Int
  , earlyStopping :: f Boolean
  , noRepeatNgramSize :: f Int
  , numReturnSequences :: f Int
  , topK :: f Number
  , topP :: f Number
  , temperature :: f Number
  , repetitionPenalty :: f Number
  , lengthPenalty :: f Number
  , badWords :: f String
  )

type GenerationProps f
  =
  { | GenerationPropsR f }

foreign import generationImpl :: Client -> GenerationProps Nullable -> String -> (Effect (Promise (Response Generation)))

generation
  :: forall from fromRL via missing missingList
   . RowToList missing missingList
  => FillableFields missingList () missing
  => Union via missing (GenerationPropsR Maybe)
  => RowToList from fromRL
  => JustifiableFields fromRL from () via
  => Client
  -> { | from }
  -> Query
  -> Aff (Response Generation)
generation client props (Query input) = generationImpl client (toInternalGenerationProps props) input # Promise.toAffE
  where
  toGenerationProps :: { | from } -> GenerationProps Maybe
  toGenerationProps = justifill

  toInternalGenerationProps :: { | from } -> GenerationProps Nullable
  toInternalGenerationProps = toGenerationProps >>> hmapKRec toNullable
