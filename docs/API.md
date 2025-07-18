# DAFS API Documentation

Complete API reference for the DAFS (Decentralized AI File System) backend services.

## Table of Contents

- [Overview](#overview)
- [Authentication](#authentication)
- [HTTP REST API](#http-rest-api)
- [gRPC API](#grpc-api)
- [Error Handling](#error-handling)
- [Rate Limiting](#rate-limiting)
- [Examples](#examples)

## Overview

DAFS provides two API interfaces:

1. **HTTP REST API** (Port 6543) - For web dashboard and general HTTP clients
2. **gRPC API** (Port 50051) - For high-performance CLI operations and AI model management

### Base URLs

- **HTTP API**: `http://localhost:6543`
- **gRPC API**: `grpc://localhost:50051`

### Content Types

- **HTTP**: `application/json` for JSON requests, `multipart/form-data` for file uploads
- **gRPC**: Protocol Buffers (binary)

## Authentication

DAFS uses password-based authentication with encrypted key storage.

### Registration

**HTTP POST** `/register`

Creates a new user account and generates X25519 keypair.

```bash
curl -X POST http://localhost:6543/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "password": "secure_password_123"
  }'
```

**Response:**
```json
{
  "success": true,
  "message": "User registered successfully"
}
```

**gRPC:**
```protobuf
service AuthService {
  rpc Register(RegisterRequest) returns (RegisterResponse);
}

message RegisterRequest {
  string username = 1;
  string password = 2;
}

message RegisterResponse {
  bool success = 1;
  string message = 2;
}
```

### Login

**HTTP POST** `/login`

Authenticates user credentials.

```bash
curl -X POST http://localhost:6543/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "password": "secure_password_123"
  }'
```

**Response:**
```json
{
  "success": true,
  "message": "Login successful"
}
```

## HTTP REST API

### File Management

#### Upload File

**POST** `/upload`

Uploads a file with metadata. Uses multipart form data.

```bash
curl -X POST http://localhost:6543/upload \
  -F "file=@document.pdf" \
  -F "metadata={\"filename\":\"document.pdf\",\"tags\":[\"work\",\"important\"],\"username\":\"alice\",\"password\":\"secure_password_123\"}"
```

**Form Fields:**
- `file`: The file to upload
- `metadata`: JSON string containing file metadata

**Response:**
```json
{
  "success": true,
  "file_id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "File uploaded successfully"
}
```

#### Download File

**GET** `/download`

Downloads a file by ID.

```bash
curl -X GET "http://localhost:6543/download?file_id=550e8400-e29b-41d4-a716-446655440000&username=alice&password=secure_password_123" \
  -o downloaded_file.pdf
```

**Query Parameters:**
- `file_id`: UUID of the file to download
- `username`: Username for authentication
- `password`: Password for authentication

**Response:** Binary file data

#### List Files

**GET** `/files`

Lists all files accessible to the user.

```bash
curl -X GET "http://localhost:6543/files?username=alice&password=secure_password_123"
```

**Response:**
```json
[
  {
    "file_id": "550e8400-e29b-41d4-a716-446655440000",
    "filename": "document.pdf",
    "tags": ["work", "important"],
    "owner_peer_id": "QmAlice123",
    "checksum": "sha256:abc123...",
    "size": 1048576,
    "shared_keys": {
      "bob": [/* encrypted key bytes */]
    },
    "allowed_peers": ["QmBob456"]
  }
]
```

#### Share File

**POST** `/share`

Shares a file with another user.

```bash
curl -X POST http://localhost:6543/share \
  -H "Content-Type: application/json" \
  -d '{
    "file_id": "550e8400-e29b-41d4-a716-446655440000",
    "owner_username": "alice",
    "owner_password": "secure_password_123",
    "recipient_username": "bob"
  }'
```

**Response:**
```json
{
  "success": true,
  "message": "File shared successfully"
}
```

### AI Operations

#### Train Model

**POST** `/ai/train`

Trains the local AI model with user interactions.

```bash
curl -X POST http://localhost:6543/ai/train
```

**Response:**
```json
{
  "success": true,
  "message": "Model trained successfully",
  "epoch": 42
}
```

#### Get Recommendations

**GET** `/ai/recommend`

Gets AI-powered file recommendations for a user.

```bash
curl -X GET "http://localhost:6543/ai/recommend?user_id=alice"
```

**Response:**
```json
[
  {
    "file_id": "550e8400-e29b-41d4-a716-446655440000",
    "filename": "recommended_document.pdf",
    "tags": ["work", "ai_recommended"],
    "owner_peer_id": "QmAlice123",
    "checksum": "sha256:def456...",
    "size": 2097152,
    "shared_keys": {},
    "allowed_peers": []
  }
]
```

#### Aggregate Model

**POST** `/ai/aggregate`

Aggregates a remote model into the local model (federated learning).

```bash
curl -X POST http://localhost:6543/ai/aggregate \
  -H "Content-Type: application/octet-stream" \
  --data-binary @remote_model.bin
```

**Response:**
```json
{
  "success": true,
  "message": "Model aggregated successfully"
}
```

### P2P Operations

#### List Peers

**GET** `/p2p/peers`

Lists connected P2P peers.

```bash
curl -X GET http://localhost:6543/p2p/peers
```

**Response:**
```json
[
  {
    "peer_id": "QmBob456",
    "addresses": ["/ip4/192.168.1.100/tcp/2093"],
    "protocols": ["file-exchange/1.0.0", "kad/1.0.0"]
  }
]
```

#### Add Bootstrap Node

**POST** `/p2p/bootstrap/add`

Adds a bootstrap node for P2P network connectivity.

```bash
curl -X POST http://localhost:6543/p2p/bootstrap/add \
  -H "Content-Type: application/json" \
  -d '{
    "peer_id": "QmBootstrap1",
    "address": "/ip4/1.2.3.4/tcp/2093"
  }'
```

**Response:**
```json
{
  "success": true,
  "message": "Bootstrap node added successfully"
}
```

#### Remove Bootstrap Node

**POST** `/p2p/bootstrap/remove`

Removes a bootstrap node.

```bash
curl -X POST http://localhost:6543/p2p/bootstrap/remove \
  -H "Content-Type: application/json" \
  -d '{
    "peer_id": "QmBootstrap1",
    "address": "/ip4/1.2.3.4/tcp/2093"
  }'
```

**Response:**
```json
{
  "success": true,
  "message": "Bootstrap node removed successfully"
}
```

#### List Bootstrap Nodes

**GET** `/p2p/bootstrap/list`

Lists all configured bootstrap nodes.

```bash
curl -X GET http://localhost:6543/p2p/bootstrap/list
```

**Response:**
```json
[
  {
    "peer_id": "QmBootstrap1",
    "address": "/ip4/1.2.3.4/tcp/2093"
  },
  {
    "peer_id": "QmBootstrap2", 
    "address": "/ip4/5.6.7.8/tcp/2093"
  }
]
```

#### List P2P Files

**GET** `/p2p/files`

Lists files available from a specific peer.

```bash
curl -X GET "http://localhost:6543/p2p/files?peer_id=QmBob456"
```

**Response:**
```json
[
  {
    "file_id": "550e8400-e29b-41d4-a716-446655440000",
    "filename": "shared_document.pdf",
    "tags": ["shared", "work"],
    "owner_peer_id": "QmBob456",
    "checksum": "sha256:ghi789...",
    "size": 3145728,
    "shared_keys": {},
    "allowed_peers": []
  }
]
```

#### P2P Download Chunk

**POST** `/p2p/download-chunk`

Downloads a file chunk from a peer.

```bash
curl -X POST http://localhost:6543/p2p/download-chunk \
  -H "Content-Type: application/json" \
  -d '{
    "peer_id": "QmBob456",
    "file_id": "550e8400-e29b-41d4-a716-446655440000",
    "chunk_index": 0,
    "chunk_size": 1048576
  }'
```

**Response:** Binary chunk data

### File Chunking

#### Upload Chunk

**POST** `/upload-chunk`

Uploads a file chunk (for large files).

```bash
curl -X POST "http://localhost:6543/upload-chunk?file_id=550e8400-e29b-41d4-a716-446655440000&chunk_index=0&total_chunks=3" \
  -H "Content-Type: application/octet-stream" \
  --data-binary @chunk_0.bin
```

**Query Parameters:**
- `file_id`: UUID of the file
- `chunk_index`: Index of the chunk (0-based)
- `total_chunks`: Total number of chunks

**Response:**
```json
{
  "success": true,
  "message": "Chunk uploaded successfully"
}
```

#### Download Chunk

**GET** `/download-chunk`

Downloads a file chunk.

```bash
curl -X GET "http://localhost:6543/download-chunk?file_id=550e8400-e29b-41d4-a716-446655440000&chunk_index=0&chunk_size=1048576"
```

**Response:** Binary chunk data

## gRPC API

### Service Definitions

#### AI Service

```protobuf
service AIService {
  // Train the local AI model
  rpc TrainModel(TrainRequest) returns (TrainResponse);
  
  // Get recommendations for a user
  rpc GetRecommendations(RecommendationsRequest) returns (RecommendationsResponse);
  
  // Aggregate a remote model into local model
  rpc AggregateModel(AggregateRequest) returns (AggregateResponse);
  
  // Export local model for sharing
  rpc ExportModel(ExportRequest) returns (ExportResponse);
}
```

#### File Service

```protobuf
service FileService {
  // Upload file (streaming)
  rpc UploadFile(stream UploadChunk) returns (UploadResponse);
  
  // Download file (streaming)
  rpc DownloadFile(DownloadRequest) returns (stream DownloadChunk);
  
  // List files
  rpc ListFiles(ListFilesRequest) returns (ListFilesResponse);
  
  // Share file with another user
  rpc ShareFile(ShareFileRequest) returns (ShareFileResponse);
  
  // Get file metadata
  rpc GetFileMetadata(FileMetadataRequest) returns (FileMetadataResponse);
}
```

#### Auth Service

```protobuf
service AuthService {
  rpc Register(RegisterRequest) returns (RegisterResponse);
  rpc Login(LoginRequest) returns (LoginResponse);
}
```

#### P2P Service

```protobuf
service P2PService {
  // List connected peers
  rpc ListPeers(ListPeersRequest) returns (ListPeersResponse);
  
  // Add bootstrap node
  rpc AddBootstrapNode(BootstrapNodeRequest) returns (BootstrapNodeResponse);
  
  // Remove bootstrap node
  rpc RemoveBootstrapNode(BootstrapNodeRequest) returns (BootstrapNodeResponse);
  
  // List bootstrap nodes
  rpc ListBootstrapNodes(ListBootstrapNodesRequest) returns (ListBootstrapNodesResponse);
  
  // P2P file listing and download
  rpc ListP2pFiles(ListP2pFilesRequest) returns (ListP2pFilesResponse);
  rpc P2pDownloadChunk(P2pDownloadChunkRequest) returns (P2pDownloadChunkResponse);
}
```

### Using gRPC with grpcurl

#### List Services

```bash
grpcurl -plaintext localhost:50051 list
```

#### List Methods

```bash
grpcurl -plaintext localhost:50051 list dafs.AIService
```

#### Call Methods

```bash
# Train AI model
grpcurl -plaintext -d '{}' localhost:50051 dafs.AIService/TrainModel

# Get recommendations
grpcurl -plaintext -d '{"user_id": "alice"}' localhost:50051 dafs.AIService/GetRecommendations

# Register user
grpcurl -plaintext -d '{"username": "alice", "password": "secure_password_123"}' \
  localhost:50051 dafs.AuthService/Register
```

### Using gRPC with Rust Client

```rust
use tonic::transport::Channel;
use dafs::ai_service_client::AiServiceClient;
use dafs::TrainRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_shared("http://localhost:50051".to_string())?
        .connect()
        .await?;
    
    let mut client = AiServiceClient::new(channel);
    
    let request = tonic::Request::new(TrainRequest {});
    let response = client.train_model(request).await?;
    
    println!("Training response: {:?}", response);
    Ok(())
}
```

## Error Handling

### HTTP Error Responses

```json
{
  "error": "Error message",
  "code": "ERROR_CODE",
  "details": "Additional error details"
}
```

### Common Error Codes

- `400 Bad Request`: Invalid request format or parameters
- `401 Unauthorized`: Authentication failed
- `403 Forbidden`: Access denied to resource
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource conflict (e.g., user already exists)
- `413 Payload Too Large`: File too large
- `500 Internal Server Error`: Server error
- `503 Service Unavailable`: Service temporarily unavailable

### gRPC Error Status Codes

- `OK (0)`: Success
- `INVALID_ARGUMENT (3)`: Invalid arguments
- `UNAUTHENTICATED (16)`: Authentication failed
- `PERMISSION_DENIED (7)`: Access denied
- `NOT_FOUND (5)`: Resource not found
- `ALREADY_EXISTS (6)`: Resource already exists
- `RESOURCE_EXHAUSTED (8)`: Resource exhausted
- `INTERNAL (13)`: Internal server error
- `UNAVAILABLE (14)`: Service unavailable

## Rate Limiting

DAFS implements rate limiting to prevent abuse:

- **File uploads**: 10 requests per minute per user
- **AI operations**: 5 requests per minute per user
- **P2P operations**: 20 requests per minute per user
- **Authentication**: 5 attempts per minute per IP

Rate limit headers:
```
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 7
X-RateLimit-Reset: 1640995200
```

## Examples

### Complete File Upload Workflow

```bash
# 1. Register user
curl -X POST http://localhost:6543/register \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "password": "secure_password_123"}'

# 2. Upload file
curl -X POST http://localhost:6543/upload \
  -F "file=@document.pdf" \
  -F "metadata={\"filename\":\"document.pdf\",\"tags\":[\"work\"],\"username\":\"alice\",\"password\":\"secure_password_123\"}"

# 3. Share file
curl -X POST http://localhost:6543/share \
  -H "Content-Type: application/json" \
  -d '{"file_id":"550e8400-e29b-41d4-a716-446655440000","owner_username":"alice","owner_password":"secure_password_123","recipient_username":"bob"}'

# 4. Train AI model
curl -X POST http://localhost:6543/ai/train

# 5. Get recommendations
curl -X GET "http://localhost:6543/ai/recommend?user_id=alice"
```

### P2P Network Setup

```bash
# 1. Add bootstrap nodes
curl -X POST http://localhost:6543/p2p/bootstrap/add \
  -H "Content-Type: application/json" \
  -d '{"peer_id": "QmBootstrap1", "address": "/ip4/1.2.3.4/tcp/2093"}'

# 2. List connected peers
curl -X GET http://localhost:6543/p2p/peers

# 3. List files from peer
curl -X GET "http://localhost:6543/p2p/files?peer_id=QmBob456"

# 4. Download file chunk from peer
curl -X POST http://localhost:6543/p2p/download-chunk \
  -H "Content-Type: application/json" \
  -d '{"peer_id":"QmBob456","file_id":"550e8400-e29b-41d4-a716-446655440000","chunk_index":0,"chunk_size":1048576}'
```

### AI Model Management

```bash
# 1. Train local model
curl -X POST http://localhost:6543/ai/train

# 2. Export model for sharing
curl -X POST http://localhost:6543/ai/export -o local_model.bin

# 3. Aggregate remote model
curl -X POST http://localhost:6543/ai/aggregate \
  -H "Content-Type: application/octet-stream" \
  --data-binary @remote_model.bin

# 4. Get recommendations
curl -X GET "http://localhost:6543/ai/recommend?user_id=alice"
```

### Large File Handling

```bash
# Split large file into chunks
split -b 1M large_file.bin chunk_

# Upload chunks
for i in chunk_*; do
  chunk_index=$(echo $i | sed 's/chunk_//')
  curl -X POST "http://localhost:6543/upload-chunk?file_id=550e8400-e29b-41d4-a716-446655440000&chunk_index=$chunk_index&total_chunks=10" \
    -H "Content-Type: application/octet-stream" \
    --data-binary @$i
done

# Download chunks
for i in {0..9}; do
  curl -X GET "http://localhost:6543/download-chunk?file_id=550e8400-e29b-41d4-a716-446655440000&chunk_index=$i&chunk_size=1048576" \
    -o downloaded_chunk_$i.bin
done

# Reassemble file
cat downloaded_chunk_*.bin > reassembled_file.bin
```

## SDK Examples

### Rust Client

```rust
use dafs_client::{Client, Config, FileMetadata};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new("http://localhost:6543");
    let client = Client::new(config);
    
    // Register user
    client.register("alice", "secure_password_123").await?;
    
    // Upload file
    let file_data = std::fs::read("document.pdf")?;
    let metadata = FileMetadata {
        filename: "document.pdf".to_string(),
        tags: vec!["work".to_string(), "important".to_string()],
        owner_peer_id: "QmAlice123".to_string(),
        checksum: "sha256:abc123...".to_string(),
        size: file_data.len() as u64,
        encrypted_file_key: vec![],
        shared_keys: std::collections::HashMap::new(),
        allowed_peers: vec![],
    };
    
    let file_id = client.upload_file(&file_data, metadata).await?;
    println!("Uploaded file: {}", file_id);
    
    // Get recommendations
    let recommendations = client.get_recommendations("alice").await?;
    println!("Recommendations: {:?}", recommendations);
    
    Ok(())
}
```

### JavaScript Client

```javascript
import { DafsClient } from '@dafs/client';

async function main() {
    const client = new DafsClient('http://localhost:6543');
    
    // Register user
    await client.register('alice', 'secure_password_123');
    
    // Upload file
    const file = new File(['Hello, World!'], 'hello.txt', { type: 'text/plain' });
    const metadata = {
        filename: 'hello.txt',
        tags: ['test', 'example'],
        username: 'alice',
        password: 'secure_password_123'
    };
    
    const fileId = await client.uploadFile(file, metadata);
    console.log('Uploaded file:', fileId);
    
    // Get recommendations
    const recommendations = await client.getRecommendations('alice');
    console.log('Recommendations:', recommendations);
}

main().catch(console.error);
```

### Python Client

```python
import asyncio
from dafs_client import DafsClient

async def main():
    client = DafsClient("http://localhost:6543")
    
    # Register user
    await client.register("alice", "secure_password_123")
    
    # Upload file
    with open("document.pdf", "rb") as f:
        file_data = f.read()
    
    metadata = {
        "filename": "document.pdf",
        "tags": ["work", "important"],
        "username": "alice",
        "password": "secure_password_123"
    }
    
    file_id = await client.upload_file(file_data, metadata)
    print(f"Uploaded file: {file_id}")
    
    # Get recommendations
    recommendations = await client.get_recommendations("alice")
    print(f"Recommendations: {recommendations}")

asyncio.run(main())
```

## Testing

### Health Check

```bash
curl -X GET http://localhost:6543/health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime": 3600,
  "services": {
    "http_api": "running",
    "grpc_api": "running",
    "p2p_network": "running",
    "ai_model": "ready"
  }
}
```

### Load Testing

```bash
# Test file upload performance
ab -n 100 -c 10 -p upload_data.json -T application/json http://localhost:6543/upload

# Test AI recommendation performance
ab -n 1000 -c 50 "http://localhost:6543/ai/recommend?user_id=test_user"
```

---

For more information, see the [main README](../README.md) or visit the [project documentation](https://github.com/your-repo/dafs/wiki). 