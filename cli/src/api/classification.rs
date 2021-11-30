extern crate base64;

use crate::types::Result;
use base64::encode;
use log::info;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path;

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
struct ClassificationFile {
    name: Option<String>,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ClassificationRequest {
    language: Option<String>,
    files: Vec<ClassificationFile>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Classification {
    pub name: String,
    pub tldr: String,
    pub usage: String,
    pub version: Option<String>,
    pub license: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SelectionRequest {
    language: Option<String>,
    files: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Selection {
    pub files: Vec<String>,
}

pub async fn classify(
    maybe_language: Option<String>,
    files: Vec<(Option<String>, String)>,
) -> Result<Classification> {
    let client = reqwest::Client::new();
    let encoded_files: Vec<ClassificationFile> = files
        .iter()
        .map(|(name, content)| ClassificationFile {
            name: name.clone(),
            content: encode(content.trim()),
        })
        .collect();
    let request = ClassificationRequest {
        language: maybe_language,
        files: encoded_files,
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

pub async fn select(maybe_language: Option<String>, files: Vec<String>) -> Result<Selection> {
    let client = reqwest::Client::new();
    let request = SelectionRequest {
        language: maybe_language,
        files: files,
    };
    let request = serde_json::to_string(&request)?;
    info!("Sending request {}", request);
    let response = client
        .post("http://localhost:8080/api/v1/select-files")
        .body(request)
        .send()
        .await?
        .json::<Selection>()
        .await?;
    Ok(response)
}
