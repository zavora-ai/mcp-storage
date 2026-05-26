# mcp-storage

[![Crates.io](https://img.shields.io/crates/v/mcp-storage.svg)](https://crates.io/crates/mcp-storage)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Universal cloud storage MCP server — manage files across AWS S3, Google Cloud Storage, Cloudflare R2, MinIO, and any S3-compatible provider through a single interface. **12 tools** for listing, uploading, downloading, copying, moving, and searching objects.

## Installation

```bash
cargo install mcp-storage
```

## Provider Setup

### AWS S3

No extra config needed if you have `~/.aws/credentials` set up:

```bash
mcp-storage
```

Or with explicit credentials:

```bash
STORAGE_ACCESS_KEY=AKIA... STORAGE_SECRET_KEY=... STORAGE_REGION=us-east-1 mcp-storage
```

### Google Cloud Storage

GCS works via S3 interoperability. Create HMAC keys first:

```bash
# Create HMAC key (one-time setup)
gcloud storage hmac create YOUR-SERVICE-ACCOUNT@PROJECT.iam.gserviceaccount.com

# Run with GCS
STORAGE_ENDPOINT=https://storage.googleapis.com \
STORAGE_ACCESS_KEY=GOOG1E... \
STORAGE_SECRET_KEY=... \
STORAGE_REGION=us-east-1 \
mcp-storage
```

> **Note:** GCS S3 interop doesn't support `list_buckets`. Use `list_objects` with a specific bucket name.

### Cloudflare R2

```bash
STORAGE_ENDPOINT=https://ACCOUNT_ID.r2.cloudflarestorage.com \
STORAGE_ACCESS_KEY=... \
STORAGE_SECRET_KEY=... \
STORAGE_REGION=auto \
mcp-storage
```

Get R2 credentials from: Cloudflare Dashboard → R2 → Manage R2 API Tokens.

### MinIO (Self-hosted)

```bash
STORAGE_ENDPOINT=http://localhost:9000 \
STORAGE_ACCESS_KEY=minioadmin \
STORAGE_SECRET_KEY=minioadmin \
mcp-storage
```

### DigitalOcean Spaces

```bash
STORAGE_ENDPOINT=https://nyc3.digitaloceanspaces.com \
STORAGE_ACCESS_KEY=... \
STORAGE_SECRET_KEY=... \
STORAGE_REGION=nyc3 \
mcp-storage
```

### Backblaze B2

```bash
STORAGE_ENDPOINT=https://s3.us-west-004.backblazeb2.com \
STORAGE_ACCESS_KEY=... \
STORAGE_SECRET_KEY=... \
STORAGE_REGION=us-west-004 \
mcp-storage
```

## MCP Client Configuration

### Claude Desktop

```json
{
  "mcpServers": {
    "storage": {
      "command": "mcp-storage",
      "env": {
        "STORAGE_ENDPOINT": "https://storage.googleapis.com",
        "STORAGE_ACCESS_KEY": "GOOG1E...",
        "STORAGE_SECRET_KEY": "your-secret",
        "STORAGE_REGION": "us-east-1"
      }
    }
  }
}
```

### Cursor

Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "storage": {
      "command": "mcp-storage",
      "env": {}
    }
  }
}
```

If using AWS with default credentials (`~/.aws/credentials`), no env vars needed.

## Tools (12)

### Browse & Read

| Tool | Description | Example |
|------|-------------|---------|
| `list_buckets` | List all buckets | "Show me all my S3 buckets" |
| `list_objects` | List objects with prefix filter | "List files in my-bucket/images/" |
| `get_object_info` | Get size, type, last modified | "What's the size of report.pdf?" |
| `download_object` | Get file content (text inline, binary as base64) | "Show me config.json" |
| `generate_presigned_url` | Create temporary download link | "Give me a link to share this file" |
| `search_objects` | Find files by prefix pattern | "Find all .csv files in data/" |
| `get_storage_usage` | Total size and count for a bucket | "How much storage am I using?" |

### Write (Gated)

| Tool | Approval | Description | Example |
|------|:--------:|-------------|---------|
| `upload_object` | No | Upload text content | "Save this JSON to config/app.json" |
| `copy_object` | No | Copy within/between buckets | "Copy report.pdf to the archive bucket" |
| `move_object` | Yes | Move (copy + delete source) | "Move old logs to cold storage" |
| `delete_object` | Yes | Delete a single object | "Delete the test file" |
| `create_bucket` | Yes | Create a new bucket | "Create a bucket called backups" |

## Usage Examples

### List what's in a bucket

```
Agent: "What files are in my data-pipeline bucket?"
→ list_objects(bucket="data-pipeline", prefix="output/", limit=20)

Result:
  output/report-2026-05-01.csv (2.3 MB)
  output/report-2026-05-02.csv (2.1 MB)
  output/summary.json (4.2 KB)
```

### Upload a file

```
Agent: "Save this analysis to my reports bucket"
→ upload_object(bucket="reports", key="2026/q2-analysis.md", content="# Q2 Analysis\n...")

Result: uploaded, etag="abc123"
```

### Generate a shareable link

```
Agent: "I need a download link for the quarterly report, valid for 1 hour"
→ generate_presigned_url(bucket="reports", key="2026/q2-report.pdf", expires_in=3600)

Result: https://s3.amazonaws.com/reports/2026/q2-report.pdf?X-Amz-Signature=...
```

### Search for files

```
Agent: "Find all CSV files from May"
→ search_objects(bucket="data-pipeline", pattern="output/2026-05")

Result:
  output/2026-05-01.csv
  output/2026-05-02.csv
  ...
```

### Check storage usage

```
Agent: "How much space is my logs bucket using?"
→ get_storage_usage(bucket="application-logs")

Result: 142,847 objects, 23.4 GB total
```

## Environment Variables

| Variable | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `STORAGE_ENDPOINT` | No | AWS S3 | Custom endpoint URL |
| `STORAGE_ACCESS_KEY` | No* | — | Access key ID |
| `STORAGE_SECRET_KEY` | No* | — | Secret access key |
| `STORAGE_REGION` | No | us-east-1 | Region |

*If not set, uses AWS default credential chain (`~/.aws/credentials`, `AWS_ACCESS_KEY_ID`/`AWS_SECRET_ACCESS_KEY` env vars, IAM roles).

## Security

- **Write operations are gated** — `delete_object`, `move_object`, and `create_bucket` require approval in governed environments
- **Presigned URLs expire** — default 1 hour, configurable per request
- **No secrets in responses** — credentials are never echoed back
- **Read-only by default** — browse and download tools don't modify anything

## Tested Providers

| Provider | list_buckets | list_objects | upload | download | delete | presigned |
|----------|:---:|:---:|:---:|:---:|:---:|:---:|
| AWS S3 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Google Cloud Storage | ❌* | ✅ | ✅ | ✅ | ✅ | ✅ |
| Cloudflare R2 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| MinIO | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

*GCS S3 interop doesn't support ListBuckets — use `list_objects` with a known bucket name.

## License

Apache-2.0
