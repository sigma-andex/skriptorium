extern crate base64;

use crate::types::Result;
use base64::encode;
use log::info;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug)]
pub enum ClassificationError {
    ClassificationFailed,
}

impl std::error::Error for ClassificationError {}

impl fmt::Display for ClassificationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClassificationError::ClassificationFailed => write!(f, "Classification failed."),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ClassificationRequest {
    snippet: String,
    language: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Classification {
    pub classification: String,
    pub tldr: String
}

fn mk_snippet(snippet: String) -> String {
    format!("```\n{}\n```", snippet.trim())
}

pub async fn classify(maybe_language: Option<String>, snippet: String) -> Result<Classification> {
    let client = reqwest::Client::new();
    let request = ClassificationRequest {
        language: maybe_language,
        snippet: encode(mk_snippet(snippet)),
    };
    let request = serde_json::to_string(&request)?;
    info!("Sending request {}", request);
    let response = client
        .post("http://localhost:8080/api/v1/classification")
        .body(request)
        .send()
        .await?
        .json::<Classification>()
        .await?;
    Ok(response)
}
