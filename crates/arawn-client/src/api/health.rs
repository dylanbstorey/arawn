//! Health API.

use crate::client::ArawnClient;
use crate::error::Result;
use crate::types::HealthResponse;

/// Health API client.
///
/// Note: Health endpoints typically don't require authentication.
pub struct HealthApi {
    client: ArawnClient,
}

impl HealthApi {
    pub(crate) fn new(client: ArawnClient) -> Self {
        Self { client }
    }

    /// Check basic health.
    pub async fn check(&self) -> Result<HealthResponse> {
        // Health endpoint is at root, not under /api/v1
        let inner = self.client.inner();
        let url = inner
            .base_url
            .join("health")
            .map_err(crate::error::Error::from)?;

        let response: reqwest::Response = inner.http.get(url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(crate::error::Error::Api {
                status: response.status().as_u16(),
                code: "health_check_failed".to_string(),
                message: "Health check failed".to_string(),
            })
        }
    }

    /// Simple connectivity check - returns true if server is reachable.
    pub async fn is_healthy(&self) -> bool {
        self.check().await.is_ok()
    }
}
