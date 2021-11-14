{ name = "skriptorium-api"
, dependencies =
  [ "aff"
  , "aff-promise"
  , "argonaut"
  , "arrays"
  , "b64"
  , "console"
  , "effect"
  , "either"
  , "exceptions"
  , "heterogeneous-extrablatt"
  , "httpure"
  , "justifill"
  , "maybe"
  , "newtype"
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
