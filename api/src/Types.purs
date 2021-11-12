module Types (Token(..)) where

import Data.Newtype (class Newtype)

newtype Token = Token String

instance Newtype Token String
