{ name = "skriptorium-api"
, dependencies =
  [ "aff"
  , "aff-promise"
  , "console"
  , "effect"
  , "heterogeneous-extrablatt"
  , "httpure"
  , "justifill"
  , "maybe"
  , "node-process"
  , "nullable"
  , "posix-types"
  , "prelude"
  , "psci-support"
  ]
, packages = ./packages.dhall
, sources = [ "src/**/*.purs", "test/**/*.purs" ]
}
