use azure_client_builder_method_builder::{Secret, SecretClient, SecretProperties};
use azure_core::{ClientBuilder, Context, ExponentialRetryOptions, RetryOptions};
use azure_identity::DefaultAzureCredential;
use std::{env, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = env::var("AZURE_KEYVAULT_URL")?;

    let credential = Arc::new(DefaultAzureCredential::default());
    let client = SecretClient::builder(endpoint, credential)?
        .with_retry(RetryOptions::exponential(ExponentialRetryOptions::default()))
        .build();

    let mut ctx = Context::default();
    ctx.insert("example".to_string());

    let response = client
        .set_secret("secret-name", "secret-value")
        .with_context(ctx)
        .with_properties(SecretProperties { enabled: false })
        .send()
        .await?;

    let secret: Secret = response.json().await?;
    println!("set {} version {}", secret.name, secret.version);

    Ok(())
}
