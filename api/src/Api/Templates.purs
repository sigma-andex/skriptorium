module Api.Templates where

import Prelude

import Data.String (trim)
import Effect.Aff (Aff)
import Node.Encoding (Encoding(..))
import Node.FS.Aff (readTextFile)
import Node.Path (FilePath)

qaTemplate :: FilePath -> String -> Aff String
qaTemplate fp snippet = readTextFile UTF8 fp <#> \template -> trim template <> "\nQ: \n" <> trim snippet <> "\nA: "
