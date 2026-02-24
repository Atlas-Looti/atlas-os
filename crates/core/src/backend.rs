//! Atlas Backend API client — used by CLI to access backend services
//! (CoinGecko, Alchemy, etc.) through the Atlas API gateway.

use anyhow::{Context, Result};

/// Lightweight client for calling the Atlas backend API.
pub struct BackendClient {
    http: reqwest::Client,
    base_url: String,
}

impl BackendClient {
    /// Create a new backend client from config.
    pub fn new(api_url: &str) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http,
            base_url: api_url.trim_end_matches('/').to_string(),
        }
    }

    /// Create from the active config's api_url.
    pub fn from_config() -> Result<Self> {
        let config = crate::workspace::load_config()?;
        Ok(Self::new(&config.system.api_url))
    }

    /// GET a JSON endpoint from the backend.
    pub async fn get(&self, path: &str, query: &[(&str, &str)]) -> Result<serde_json::Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.http
            .get(&url)
            .query(query)
            .send()
            .await
            .with_context(|| format!("Failed to reach Atlas backend at {url}. Is atlas-server running?"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Backend error {status}: {text}");
        }

        resp.json::<serde_json::Value>()
            .await
            .context("Failed to parse backend response")
    }

    /// Check if the backend is reachable.
    pub async fn health(&self) -> Result<bool> {
        let url = format!("{}/api/health", self.base_url);
        match self.http.get(&url).send().await {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
