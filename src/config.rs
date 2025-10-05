//! Handles configuration parsing and validation for the AWS provider.

use aws_credential_types::Credentials;
use aws_sdk_secretsmanager::config::BehaviorVersion;
use aws_types::{
    region::Region,
    sdk_config::{SdkConfig, SharedCredentialsProvider},
};
use lockset_vault_provider::ProviderError;
use serde::Deserialize;
use zeroize::{Zeroize, Zeroizing};

/// Defines the supported AWS authentication methods.
#[derive(Deserialize, Zeroize)]
#[serde(tag = "type")]
pub enum AwsAuth {
    AccessKey {
        access_key_id: Zeroizing<String>,
        secret_access_key: Zeroizing<String>,
    },
}

/// Defines the structure for the AWS provider configuration.
#[derive(Deserialize, Zeroize)]
pub struct AwsConfig {
    pub region: String,
    pub auth: AwsAuth,
}

impl AwsConfig {
    /// Parses the configuration from a string.
    pub fn parse(s: &Zeroizing<String>) -> Result<Self, ProviderError> {
        serde_json::from_str(s.as_str())
            .map_err(|e| ProviderError::InvalidConfiguration(e.to_string()))
    }
}

impl Into<SdkConfig> for AwsConfig {
    /// Creates an SdkConfig exclusively from the provided configuration.
    ///
    /// This function ensures that only the credentials provided in the configuration are used,
    /// preventing the SDK from falling back to ambient credentials (e.g., from environment
    /// variables or IAM roles). This is crucial for maintaining isolation and security.
    fn into(mut self) -> SdkConfig {
        let credentials = match self.auth {
            AwsAuth::AccessKey {
                ref access_key_id,
                ref secret_access_key,
            } => Credentials::new(
                access_key_id.to_string(),
                secret_access_key.to_string(),
                None, // session token
                None, // expiration
                "StaticConfig",
            ),
        };

        self.auth.zeroize();

        SdkConfig::builder()
            .credentials_provider(SharedCredentialsProvider::new(credentials))
            .region(Region::new(self.region))
            .behavior_version(BehaviorVersion::v2025_08_07())
            .build()
    }
}
