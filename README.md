# Lockset Vault Provider for AWS Secrets Manager

This crate provides a `VaultProvider` implementation for AWS Secrets Manager, allowing the Lockset Vault system to retrieve secrets from AWS. It is built upon the `lockset-vault-provider` traits.

## Core Concepts

This provider integrates AWS Secrets Manager with the Lockset Vault by implementing two key traits from the `lockset-vault-provider` crate:

-   `VaultProvider`: The `AwsSecretsManagerProvider` struct implements this trait. Its `get_secret` method fetches a secret from AWS Secrets Manager by its name or ARN.

-   `VaultProviderFactory`: The `AwsSecretsManagerFactory` struct implements this trait. It is responsible for validating the provider's configuration and creating instances of `AwsSecretsManagerProvider`.

## Configuration

The provider is configured using a JSON string. The configuration specifies the AWS region and the authentication credentials.

Here is an example of the configuration format:

```json
{
    "region": "us-east-1",
    "auth": {
        "type": "AccessKey",
        "access_key_id": "YOUR_AWS_ACCESS_KEY_ID",
        "secret_access_key": "YOUR_AWS_SECRET_ACCESS_KEY"
    }
}
```

### Supported Authentication Methods

Currently, the only supported authentication method is:

-   **`AccessKey`**: Uses an AWS access key ID and secret access key.

### Configuration Validation

The `validate` method of the `AwsSecretsManagerFactory` performs a check to ensure the provided credentials and region are valid. It does this by making a `sts:GetCallerIdentity` call to AWS. This is a low-cost, non-intrusive way to confirm that the credentials are correct and that the provider can connect to AWS.

## Security

Security is a primary consideration in the design of this provider:

-   **Secure Memory Handling**: The configuration string and the retrieved secret values are wrapped in `zeroize::Zeroizing`. This ensures that sensitive data is securely erased from memory as soon as it is no longer needed.

-   **Isolated Credentials**: The provider is designed to use *only* the credentials provided in the configuration. It explicitly prevents the AWS SDK from falling back to ambient credentials (such as those from environment variables or IAM roles). This guarantees that the provider's access is strictly limited to the permissions of the configured credentials.

## Usage

To use this provider, you would typically register the `AwsSecretsManagerFactory` with your Lockset Vault instance. The vault would then use the factory to create a provider for retrieving secrets from AWS Secrets Manager.

Here is a conceptual example of how the provider and factory might be used:

```rust
use lockset_vault_provider::{VaultProvider, VaultProviderFactory};
use lockset_vault_provider_aws::AwsSecretsManagerFactory;
use zeroize::Zeroizing;

async fn use_aws_provider() {
    let factory = AwsSecretsManagerFactory;

    // Your configuration would come from a secure source.
    let config = Zeroizing::new(
        r#"{
            "region": "us-east-1",
            "auth": {
                "type": "AccessKey",
                "access_key_id": "...",
                "secret_access_key": "..."
            }
        }"#
        .to_string(),
    );

    // 1. Validate the configuration
    factory.validate(&config).await.expect("Invalid configuration");

    // 2. Create a provider instance
    let provider = factory.create(config).await.expect("Failed to create provider");

    // 3. Retrieve a secret
    let secret = provider
        .get_secret("my-app/production/api-key")
        .await
        .expect("Failed to get secret");

    println!("Retrieved secret version: {:?}", secret.version);
    // The secret value is securely handled by Zeroizing.
}
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
