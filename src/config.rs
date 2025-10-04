//! Handles configuration parsing and validation for the AWS provider.

use aws_credential_types::Credentials;
use aws_types::{
    region::Region,
    sdk_config::{SdkConfig, SharedCredentialsProvider},
};
use lockset_vault_provider::ProviderError;
use serde::Deserialize;
use std::str::FromStr;

/// Defines the supported AWS authentication methods.
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum AwsAuth {
    AccessKey {
        access_key_id: String,
        secret_access_key: String,
        session_token: Option<String>,
    },
}

/// Defines the structure for the AWS provider configuration.
#[derive(Deserialize)]
pub struct AwsConfig {
    pub region: String,
    pub auth: AwsAuth,
}

impl FromStr for AwsConfig {
    type Err = ProviderError;

    /// Parses the configuration from a string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(|e| ProviderError::InvalidConfiguration(e.to_string()))
    }
}

impl Into<SdkConfig> for AwsConfig {
    /// Creates an SdkConfig exclusively from the provided configuration.
    ///
    /// This function ensures that only the credentials provided in the configuration are used,
    /// preventing the SDK from falling back to ambient credentials (e.g., from environment
    /// variables or IAM roles). This is crucial for maintaining isolation and security.
    fn into(self) -> SdkConfig {
        let credentials = match self.auth {
            AwsAuth::AccessKey {
                access_key_id,
                secret_access_key,
                session_token,
            } => Credentials::new(
                access_key_id,
                secret_access_key,
                session_token,
                None, // expiration
                "StaticConfig",
            ),
        };

        SdkConfig::builder()
            .credentials_provider(SharedCredentialsProvider::new(credentials))
            .region(Region::new(self.region))
            .build()
    }
}
