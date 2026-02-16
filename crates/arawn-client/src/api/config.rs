//! Config API.

use crate::client::ArawnClient;
use crate::error::Result;
use crate::types::ConfigResponse;

/// Config API client.
pub struct ConfigApi {
    client: ArawnClient,
}

impl ConfigApi {
    pub(crate) fn new(client: ArawnClient) -> Self {
        Self { client }
    }

    /// Get server configuration.
    pub async fn get(&self) -> Result<ConfigResponse> {
        self.client.get("config").await
    }
}
