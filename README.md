# mcp-storage

[![Crates.io](https://img.shields.io/crates/v/mcp-storage.svg)](https://crates.io/crates/mcp-storage)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Universal cloud storage MCP server — list, upload, download, copy, move, delete, and search objects across any S3-compatible provider. **12 tools** supporting AWS S3, Cloudflare R2, MinIO, DigitalOcean Spaces, Backblaze B2, Wasabi, and GCS (S3 interop).

## Quick Start

```bash
cargo install mcp-storage

# AWS S3
STORAGE_ACCESS_KEY=... STORAGE_SECRET_KEY=... mcp-storage

# Cloudflare R2
STORAGE_ENDPOINT=https://ACCOUNT.r2.cloudflarestorage.com STORAGE_ACCESS_KEY=... STORAGE_SECRET_KEY=... mcp-storage

# MinIO (self-hosted)
STORAGE_ENDPOINT=http://localhost:9000 STORAGE_ACCESS_KEY=minioadmin STORAGE_SECRET_KEY=minioadmin mcp-storage
```

## Tools (12)

### Read (6)
| Tool | Description |
|------|-------------|
| `list_buckets` | List all buckets/containers |
| `list_objects` | List objects with prefix filter |
| `get_object_info` | Metadata, size, content type, etag |
| `download_object` | Get content (text inline, binary as base64) |
| `generate_presigned_url` | Temporary download URL |
| `search_objects` | Find objects by prefix pattern |
| `get_storage_usage` | Total size and object count |

### Write (5, gated)
| Tool | Description |
|------|-------------|
| `upload_object` | Upload text content |
| `copy_object` | Copy within/between buckets |
| `move_object` | Move (copy + delete) — requires approval |
| `delete_object` | Delete single object — requires approval |
| `create_bucket` | Create new bucket — requires approval |

## Supported Providers

| Provider | Endpoint |
|----------|----------|
| AWS S3 | (default, no endpoint needed) |
| Cloudflare R2 | `https://ACCOUNT.r2.cloudflarestorage.com` |
| MinIO | `http://localhost:9000` |
| DigitalOcean Spaces | `https://REGION.digitaloceanspaces.com` |
| Backblaze B2 | `https://s3.REGION.backblazeb2.com` |
| Wasabi | `https://s3.REGION.wasabisys.com` |
| GCS (S3 interop) | `https://storage.googleapis.com` |

## Configuration

```json
{
  "mcpServers": {
    "storage": {
      "command": "mcp-storage",
      "env": {
        "STORAGE_ENDPOINT": "https://your-endpoint",
        "STORAGE_ACCESS_KEY": "your-key",
        "STORAGE_SECRET_KEY": "your-secret",
        "STORAGE_REGION": "us-east-1"
      }
    }
  }
}
```

## License

Apache-2.0
