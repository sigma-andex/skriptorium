module Api.Types where

type ClassificationRequest =
  { "snippet" :: String
  }

type ClassificationResponse =
  { "classification" :: String
  }
