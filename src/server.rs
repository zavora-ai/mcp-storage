use aws_sdk_s3::Client;
use aws_sdk_s3::presigning::PresigningConfig;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EmptyInput {}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BucketInput {
    /// Bucket name
    pub bucket: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListObjectsInput {
    /// Bucket name
    pub bucket: String,
    /// Prefix filter (folder path, e.g. "images/2024/")
    pub prefix: Option<String>,
    /// Max results (default 50)
    pub limit: Option<i32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ObjectInput {
    /// Bucket name
    pub bucket: String,
    /// Object key (path)
    pub key: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UploadInput {
    /// Bucket name
    pub bucket: String,
    /// Object key (path)
    pub key: String,
    /// Content to upload (text)
    pub content: String,
    /// Content type (default: text/plain)
    pub content_type: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CopyInput {
    /// Source bucket
    pub source_bucket: String,
    /// Source key
    pub source_key: String,
    /// Destination bucket
    pub dest_bucket: String,
    /// Destination key
    pub dest_key: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PresignInput {
    /// Bucket name
    pub bucket: String,
    /// Object key
    pub key: String,
    /// Expiry in seconds (default 3600)
    pub expires_in: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateBucketInput {
    /// Bucket name to create
    pub bucket: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchInput {
    /// Bucket name
    pub bucket: String,
    /// Search pattern (prefix match)
    pub pattern: String,
    /// Max results (default 20)
    pub limit: Option<i32>,
}

#[derive(Clone)]
pub struct StorageServer {
    pub client: Client,
}

#[tool_router(server_handler)]
impl StorageServer {
    #[tool(description = "List all buckets/containers in the storage account")]
    async fn list_buckets(&self, Parameters(_input): Parameters<EmptyInput>) -> String {
        match self.client.list_buckets().send().await {
            Ok(resp) => {
                let buckets: Vec<Value> = resp.buckets().iter().map(|b| {
                    json!({
                        "name": b.name().unwrap_or_default(),
                        "created": b.creation_date().map(|d| d.to_string())
                    })
                }).collect();
                serde_json::to_string_pretty(&buckets).unwrap_or_default()
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "List objects in a bucket with optional prefix filter")]
    async fn list_objects(&self, Parameters(input): Parameters<ListObjectsInput>) -> String {
        let limit = input.limit.unwrap_or(50);
        let mut req = self.client.list_objects_v2().bucket(&input.bucket).max_keys(limit);
        if let Some(prefix) = &input.prefix {
            req = req.prefix(prefix);
        }
        match req.send().await {
            Ok(resp) => {
                let objects: Vec<Value> = resp.contents().iter().map(|o| {
                    json!({
                        "key": o.key().unwrap_or_default(),
                        "size": o.size().unwrap_or_default(),
                        "last_modified": o.last_modified().map(|d| d.to_string()),
                        "storage_class": format!("{:?}", o.storage_class())
                    })
                }).collect();
                json!({"count": objects.len(), "objects": objects}).to_string()
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get object metadata (size, content type, last modified, etag)")]
    async fn get_object_info(&self, Parameters(input): Parameters<ObjectInput>) -> String {
        match self.client.head_object().bucket(&input.bucket).key(&input.key).send().await {
            Ok(resp) => {
                json!({
                    "key": input.key,
                    "size": resp.content_length().unwrap_or_default(),
                    "content_type": resp.content_type().unwrap_or_default(),
                    "last_modified": resp.last_modified().map(|d| d.to_string()),
                    "etag": resp.e_tag().unwrap_or_default(),
                    "metadata": resp.metadata()
                }).to_string()
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Download object content. Returns text inline or base64 for binary")]
    async fn download_object(&self, Parameters(input): Parameters<ObjectInput>) -> String {
        match self.client.get_object().bucket(&input.bucket).key(&input.key).send().await {
            Ok(resp) => {
                let content_type = resp.content_type().unwrap_or("application/octet-stream").to_string();
                let body = resp.body.collect().await.map(|b| b.into_bytes()).unwrap_or_default();
                if content_type.starts_with("text/") || content_type.contains("json") || content_type.contains("xml") {
                    String::from_utf8(body.to_vec()).unwrap_or_else(|_| "Binary content, use generate_presigned_url instead".into())
                } else {
                    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &body);
                    json!({"encoding": "base64", "content_type": content_type, "size": body.len(), "data": b64}).to_string()
                }
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Generate a presigned URL for temporary access (download or upload)")]
    async fn generate_presigned_url(&self, Parameters(input): Parameters<PresignInput>) -> String {
        let expires = Duration::from_secs(input.expires_in.unwrap_or(3600));
        let presign_config = match PresigningConfig::expires_in(expires) {
            Ok(c) => c,
            Err(e) => return format!("Error: {e}"),
        };
        match self.client.get_object().bucket(&input.bucket).key(&input.key).presigned(presign_config).await {
            Ok(presigned) => json!({
                "url": presigned.uri(),
                "expires_in_seconds": input.expires_in.unwrap_or(3600),
                "method": "GET"
            }).to_string(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Upload text content to an object path")]
    async fn upload_object(&self, Parameters(input): Parameters<UploadInput>) -> String {
        let content_type = input.content_type.as_deref().unwrap_or("text/plain");
        let body = aws_sdk_s3::primitives::ByteStream::from(input.content.into_bytes());
        match self.client.put_object()
            .bucket(&input.bucket)
            .key(&input.key)
            .content_type(content_type)
            .body(body)
            .send().await {
            Ok(resp) => json!({
                "status": "uploaded",
                "bucket": input.bucket,
                "key": input.key,
                "etag": resp.e_tag().unwrap_or_default()
            }).to_string(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Copy an object within or between buckets")]
    async fn copy_object(&self, Parameters(input): Parameters<CopyInput>) -> String {
        let source = format!("{}/{}", input.source_bucket, input.source_key);
        match self.client.copy_object()
            .bucket(&input.dest_bucket)
            .key(&input.dest_key)
            .copy_source(&source)
            .send().await {
            Ok(_) => json!({
                "status": "copied",
                "from": format!("s3://{}/{}", input.source_bucket, input.source_key),
                "to": format!("s3://{}/{}", input.dest_bucket, input.dest_key)
            }).to_string(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Move an object (copy then delete source)")]
    async fn move_object(&self, Parameters(input): Parameters<CopyInput>) -> String {
        let source = format!("{}/{}", input.source_bucket, input.source_key);
        match self.client.copy_object()
            .bucket(&input.dest_bucket)
            .key(&input.dest_key)
            .copy_source(&source)
            .send().await {
            Ok(_) => {
                let _ = self.client.delete_object()
                    .bucket(&input.source_bucket)
                    .key(&input.source_key)
                    .send().await;
                json!({
                    "status": "moved",
                    "from": format!("s3://{}/{}", input.source_bucket, input.source_key),
                    "to": format!("s3://{}/{}", input.dest_bucket, input.dest_key)
                }).to_string()
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Delete a single object")]
    async fn delete_object(&self, Parameters(input): Parameters<ObjectInput>) -> String {
        match self.client.delete_object().bucket(&input.bucket).key(&input.key).send().await {
            Ok(_) => json!({"status": "deleted", "bucket": input.bucket, "key": input.key}).to_string(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Create a new bucket")]
    async fn create_bucket(&self, Parameters(input): Parameters<CreateBucketInput>) -> String {
        match self.client.create_bucket().bucket(&input.bucket).send().await {
            Ok(_) => json!({"status": "created", "bucket": input.bucket}).to_string(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Search objects by prefix pattern in a bucket")]
    async fn search_objects(&self, Parameters(input): Parameters<SearchInput>) -> String {
        let limit = input.limit.unwrap_or(20);
        match self.client.list_objects_v2()
            .bucket(&input.bucket)
            .prefix(&input.pattern)
            .max_keys(limit)
            .send().await {
            Ok(resp) => {
                let objects: Vec<Value> = resp.contents().iter().map(|o| {
                    json!({
                        "key": o.key().unwrap_or_default(),
                        "size": o.size().unwrap_or_default(),
                        "last_modified": o.last_modified().map(|d| d.to_string())
                    })
                }).collect();
                serde_json::to_string_pretty(&objects).unwrap_or_default()
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get storage usage summary for a bucket (total size, object count)")]
    async fn get_storage_usage(&self, Parameters(input): Parameters<BucketInput>) -> String {
        let mut total_size: i64 = 0;
        let mut count: i64 = 0;
        let mut token: Option<String> = None;
        loop {
            let mut req = self.client.list_objects_v2().bucket(&input.bucket).max_keys(1000);
            if let Some(t) = &token { req = req.continuation_token(t); }
            match req.send().await {
                Ok(resp) => {
                    for obj in resp.contents() {
                        total_size += obj.size().unwrap_or_default();
                        count += 1;
                    }
                    if resp.is_truncated().unwrap_or(false) {
                        token = resp.next_continuation_token().map(String::from);
                    } else {
                        break;
                    }
                }
                Err(e) => return format!("Error: {e}"),
            }
            if count > 100000 { break; } // Safety limit
        }
        json!({
            "bucket": input.bucket,
            "total_objects": count,
            "total_size_bytes": total_size,
            "total_size_mb": total_size as f64 / 1_048_576.0
        }).to_string()
    }
}
