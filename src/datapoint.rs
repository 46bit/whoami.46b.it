use serde::{Deserialize, Serialize};

use crate::utils::DatapointEndpointConfig;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Datapoint {
    pub index_id: String,
    #[serde(default = "one")]
    pub sample_interval: i64,
    pub timestamp: String,
    #[serde(default)]
    pub strings: Vec<String>,
    #[serde(default)]
    pub doubles: Vec<f64>,
}

impl Datapoint {
    pub async fn write(&self, config: &DatapointEndpointConfig) -> std::result::Result<(), String> {
        let client = reqwest::Client::new();
        let resp = client
            .post(config.url.clone())
            .body(serde_json::to_string(self).unwrap())
            .query(&[("submit", config.submit_token.clone())])
            .header("Authorization", format!("Bearer {}", config.bearer_token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let response_text = resp.text().await.map_err(|e| e.to_string())?;
        if response_text == "submitted ok" {
            return Ok(());
        }
        Err("unexpected response filing datapoint".to_string())
    }
}

fn one() -> i64 {
    1
}
