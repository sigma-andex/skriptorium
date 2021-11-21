{ name = "skriptorium-api"
, dependencies =
  [ "aff"
  , "aff-promise"
  , "argonaut"
  , "arrays"
  , "b64"
  , "bifunctors"
  , "console"
  , "effect"
  , "either"
  , "exceptions"
  , "heterogeneous-extrablatt"
  , "httpure"
  , "justifill"
  , "maybe"
  , "milkis"
  , "newtype"
  , "node-buffer"
  , "node-fs-aff"
  , "node-path"
  , "node-process"
  , "nullable"
  , "posix-types"
  , "prelude"
  , "psci-support"
  , "strings"
  , "transformers"
  , "tuples"
  ]
, packages = ./packages.dhall
, sources = [ "src/**/*.purs", "test/**/*.purs" ]
}
