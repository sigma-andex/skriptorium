{ name = "skriptorium-api"
, dependencies =
  [ "aff"
  , "aff-promise"
  , "arrays"
  , "console"
  , "effect"
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
  ]
, packages = ./packages.dhall
, sources = [ "src/**/*.purs", "test/**/*.purs" ]
}
