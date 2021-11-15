pub mod api {
    extern crate base64;

    use base64::{decode, encode};
    use serde::{Deserialize, Serialize};
    use log::{info, trace, warn};

    #[derive(Serialize, Deserialize, Debug)]
    struct ClassificationRequest {
        snippet: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Classification {
        pub classification: String,
    }

    pub async fn classify(
        snippet: String,
    ) -> Result<Classification, Box<dyn std::error::Error + Send + Sync>> {
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
}
