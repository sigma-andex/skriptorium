extern crate base64;

use crate::types::Result;
use base64::encode;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ClassificationRequest {
    snippet: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Classification {
    pub classification: String,
}

pub async fn classify(snippet: String) -> Result<Classification> {
    let client = reqwest::Client::new();
    let request = ClassificationRequest {
        snippet: encode(snippet),
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
