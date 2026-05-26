mod server;

use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Region;
use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manifest = adk_mcp_sdk::ServerManifest::from_file(std::path::Path::new("mcp-server.toml"))?;
    let errors = manifest.validate();
    if !errors.is_empty() {
        for e in &errors { eprintln!("  - {e}"); }
    }

    let endpoint = std::env::var("STORAGE_ENDPOINT").ok();
    let access_key = std::env::var("STORAGE_ACCESS_KEY").ok().or(std::env::var("AWS_ACCESS_KEY_ID").ok());
    let secret_key = std::env::var("STORAGE_SECRET_KEY").ok().or(std::env::var("AWS_SECRET_ACCESS_KEY").ok());
    let region = std::env::var("STORAGE_REGION").or(std::env::var("AWS_REGION")).unwrap_or_else(|_| "us-east-1".into());

    let client = if let (Some(ak), Some(sk)) = (access_key, secret_key) {
        // Explicit credentials (for R2, MinIO, etc.)
        let creds = Credentials::new(&ak, &sk, None, None, "mcp-storage");
        let mut builder = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(region))
            .credentials_provider(creds)
            .force_path_style(endpoint.is_some());
        if let Some(ep) = endpoint {
            builder = builder.endpoint_url(ep);
        }
        aws_sdk_s3::Client::from_conf(builder.build())
    } else {
        // Use AWS default credential chain (~/.aws/credentials, env, IAM role)
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(region))
            .load().await;
        aws_sdk_s3::Client::new(&config)
    };

    let service = server::StorageServer { client }.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
