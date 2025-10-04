use async_trait::async_trait;
use std::str::FromStr;
use aws_sdk_secretsmanager::{
    Client as SecretsManagerClient, error::SdkError as SecretsManagerSdkError,
};
use aws_sdk_sts::Client as StsClient;
use lockset_vault_provider::{ProviderError, ProviderSecret, VaultProvider, VaultProviderFactory};

mod config;

pub struct AwsSecretsManagerProvider {
    client: SecretsManagerClient,
}
pub struct AwsSecretsManagerFactory;

#[async_trait]
impl VaultProvider for AwsSecretsManagerProvider {
    async fn get_secret(&self, name: &str) -> Result<ProviderSecret, ProviderError> {
        let resp = self
            .client
            .get_secret_value()
            .secret_id(name)
            .send()
            .await
            .map_err(|e| {
                if let SecretsManagerSdkError::ServiceError(service_err) = &e {
                    if service_err.err().is_resource_not_found_exception() {
                        return ProviderError::SecretNotFound(name.to_string());
                    }
                }
                ProviderError::ClientError(Box::new(e))
            })?;

        let value = resp.secret_string().ok_or_else(|| {
            ProviderError::InvalidConfiguration("Secret value is not a string".to_string())
        })?;

        Ok(ProviderSecret {
            value: value.to_string(),
            version: resp.version_id().map(String::from),
        })
    }
}

#[async_trait]
impl VaultProviderFactory for AwsSecretsManagerFactory {
    async fn validate(&self, config_str: &str) -> Result<(), ProviderError> {
        let config = config::AwsConfig::from_str(config_str)?;
        let client = StsClient::new(&config.into());

        // A simple, low-cost operation to validate credentials and region.
        client
            .get_caller_identity()
            .send()
            .await
            .map_err(|e| ProviderError::InvalidConfiguration(e.to_string()))?;
        Ok(())
    }

    async fn create(&self, config_str: &str) -> Result<Box<dyn VaultProvider>, ProviderError> {
        let config = config::AwsConfig::from_str(config_str)?;
        let client = SecretsManagerClient::new(&config.into());
        Ok(Box::new(AwsSecretsManagerProvider { client }))
    }
}
