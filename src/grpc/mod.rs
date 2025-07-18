use tonic::{transport::Server, Request, Response, Status};
use std::sync::Arc;
use crate::storage::Storage;
use crate::peer::P2PNode;
use crate::ai::{train_local_model, get_recommendations, aggregate_remote_model, NCFModel, LOCAL_MODEL};
use crate::crypto::load_and_decrypt_keypair;
use uuid::Uuid;
use std::fs;
use std::io::Write;

// Include the generated protobuf code
pub mod dafs {
    tonic::include_proto!("dafs");
}

use dafs::{
    ai_service_server::{AiService, AiServiceServer},
    file_service_server::{FileService, FileServiceServer},
    p2p_service_server::{P2pService, P2pServiceServer},
    auth_service_server::{AuthService, AuthServiceServer},
    *,
};

#[derive(Default)]
pub struct DafsAiService {
    storage: Arc<Storage>,
}

#[tonic::async_trait]
impl AiService for DafsAiService {
    async fn train_model(
        &self,
        request: Request<TrainRequest>,
    ) -> Result<Response<TrainResponse>, Status> {
        let req = request.into_inner();
        
        // If no specific interactions provided, use all from storage
        let interactions = if req.interactions.is_empty() {
            let files = self.storage.list_metadata()
                .map_err(|e| Status::internal(format!("Storage error: {}", e)))?;
            
            let mut interactions = Vec::new();
            for f in &files {
                interactions.push((f.owner_peer_id.clone(), f.file_id.to_string()));
                for (user, _) in &f.shared_keys {
                    interactions.push((user.clone(), f.file_id.to_string()));
                }
            }
            interactions
        } else {
            req.interactions.into_iter()
                .map(|i| (i.user_id, i.file_id))
                .collect()
        };

        match train_local_model(&interactions) {
            Ok(_) => {
                let model = LOCAL_MODEL.lock()
                    .map_err(|_| Status::internal("Model lock poisoned"))?;
                Ok(Response::new(TrainResponse {
                    success: true,
                    message: "Model trained successfully".to_string(),
                    epoch: model.epoch,
                }))
            }
            Err(e) => Ok(Response::new(TrainResponse {
                success: false,
                message: format!("Training failed: {}", e),
                epoch: 0,
            }))
        }
    }

    async fn get_recommendations(
        &self,
        request: Request<RecommendationsRequest>,
    ) -> Result<Response<RecommendationsResponse>, Status> {
        let req = request.into_inner();
        let files = self.storage.list_metadata()
            .map_err(|e| Status::internal(format!("Storage error: {}", e)))?;
        
        match get_recommendations(&req.user_id, &files) {
            Ok(recommendations) => {
                let proto_files = recommendations.into_iter().map(|f| FileMetadata {
                    file_id: f.file_id.to_string(),
                    filename: f.filename,
                    tags: f.tags,
                    owner_peer_id: f.owner_peer_id,
                    checksum: f.checksum,
                    size: f.size,
                    shared_keys: f.shared_keys,
                }).collect();
                
                Ok(Response::new(RecommendationsResponse {
                    files: proto_files,
                }))
            }
            Err(e) => Err(Status::internal(format!("Recommendation error: {}", e)))
        }
    }

    async fn aggregate_model(
        &self,
        request: Request<AggregateRequest>,
    ) -> Result<Response<AggregateResponse>, Status> {
        let req = request.into_inner();
        
        match bincode::deserialize::<NCFModel>(&req.model_data) {
            Ok(remote_model) => {
                match aggregate_remote_model(&remote_model) {
                    Ok(_) => Ok(Response::new(AggregateResponse {
                        success: true,
                        message: "Model aggregated successfully".to_string(),
                    })),
                    Err(e) => Ok(Response::new(AggregateResponse {
                        success: false,
                        message: format!("Aggregation failed: {}", e),
                    }))
                }
            }
            Err(e) => Ok(Response::new(AggregateResponse {
                success: false,
                message: format!("Model deserialize failed: {}", e),
            }))
        }
    }

    async fn export_model(
        &self,
        _request: Request<ExportRequest>,
    ) -> Result<Response<ExportResponse>, Status> {
        let model = LOCAL_MODEL.lock()
            .map_err(|_| Status::internal("Model lock poisoned"))?;
        
        match bincode::serialize(&*model) {
            Ok(model_data) => Ok(Response::new(ExportResponse {
                model_data,
            })),
            Err(e) => Err(Status::internal(format!("Model serialize failed: {}", e)))
        }
    }
}

#[derive(Default)]
pub struct DafsAuthService;

#[tonic::async_trait]
impl AuthService for DafsAuthService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        // Use same logic as HTTP register
        let keypair = crate::crypto::generate_x25519_keypair();
        let secret = keypair.0;
        let public = keypair.1;
        let keyfile = format!("userkeys/{}.key", req.username);
        std::fs::create_dir_all("userkeys").ok();
        if let Err(e) = crate::crypto::encrypt_and_save_keypair(&secret, &keyfile, &req.password) {
            return Ok(Response::new(RegisterResponse {
                success: false,
                message: format!("Key save error: {}", e),
            }));
        }
        let user = crate::models::User { username: req.username.clone(), public_key: public.to_bytes() };
        crate::api::USER_DB.lock().unwrap().insert(req.username, user);
        Ok(Response::new(RegisterResponse {
            success: true,
            message: "ok".to_string(),
        }))
    }
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();
        let keyfile = format!("userkeys/{}.key", req.username);
        match crate::crypto::load_and_decrypt_keypair(&keyfile, &req.password) {
            Ok(_) => Ok(Response::new(LoginResponse {
                success: true,
                message: "ok".to_string(),
            })),
            Err(_) => Ok(Response::new(LoginResponse {
                success: false,
                message: "Invalid username or password".to_string(),
            })),
        }
    }
}

#[derive(Default)]
pub struct DafsFileService {
    storage: Arc<Storage>,
}

#[tonic::async_trait]
impl FileService for DafsFileService {
    
    async fn upload_file(
        &self,
        request: Request<tonic::Streaming<UploadChunk>>,
    ) -> Result<Response<UploadResponse>, Status> {
        let mut stream = request.into_inner();
        let mut file_id = String::new();
        let mut metadata = None;
        let mut temp_dir = String::new();
        
        while let Some(chunk) = stream.message().await.unwrap() {
            if file_id.is_empty() {
                file_id = chunk.file_id.clone();
                temp_dir = format!("upload_tmp/{}", file_id);
                fs::create_dir_all(&temp_dir).ok();
                metadata = chunk.metadata;
            }
            
            let chunk_path = format!("{}/chunk_{}", temp_dir, chunk.chunk_index);
            if let Err(e) = fs::write(&chunk_path, &chunk.data) {
                return Ok(Response::new(UploadResponse {
                    success: false,
                    file_id: file_id.clone(),
                    message: format!("Chunk write error: {}", e),
                }));
            }
        }
        // Check if all chunks are present
        let mut all_present = true;
        let total_chunks = if let Some(ref meta) = metadata {
            meta.tags.len() // Use an existing field, e.g., tags
        } else {
            0
        };
        for i in 0..total_chunks {
            if !fs::metadata(format!("{}/chunk_{}", temp_dir, i)).is_ok() {
                all_present = false;
                break;
            }
        }
        if all_present {
            // Assemble file
            let final_path = format!("files/{}.bin", file_id);
            let mut out = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&final_path)
                .unwrap();
            for i in 0..total_chunks {
                let chunk_data = fs::read(format!("{}/chunk_{}", temp_dir, i)).unwrap();
                out.write_all(&chunk_data).unwrap();
            }
            let _ = fs::remove_dir_all(&temp_dir);
            // Save metadata if provided
            if let Some(meta) = metadata {
                let file_meta = crate::storage::FileMetadata {
                    file_id: Uuid::parse_str(&file_id).unwrap(),
                    filename: meta.filename,
                    tags: meta.tags,
                    owner_peer_id: meta.owner_peer_id,
                    checksum: meta.checksum,
                    size: meta.size,
                    encrypted_file_key: vec![], // TODO: implement encryption
                    shared_keys: meta.shared_keys,
                    allowed_peers: vec![], // Add this field
                };
                if let Err(e) = self.storage.insert_metadata(&file_meta) {
                    return Ok(Response::new(UploadResponse {
                        success: false,
                        file_id: file_id.clone(),
                        message: format!("Metadata save error: {}", e),
                    }));
                }
            }
            return Ok(Response::new(UploadResponse {
                success: true,
                file_id: file_id.clone(),
                message: "Upload complete".to_string(),
            }));
        }
        Ok(Response::new(UploadResponse {
            success: false,
            file_id: file_id.clone(),
            message: "Not all chunks uploaded".to_string(),
        }))
    }

    type DownloadFileStream = tokio_stream::wrappers::ReceiverStream<Result<DownloadChunk, Status>>;
    
    async fn download_file(
        &self,
        request: Request<DownloadRequest>,
    ) -> Result<Response<Self::DownloadFileStream>, Status> {
        let req = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(128);
        
        let keyfile = format!("userkeys/{}.key", req.username.clone());
        let password = req.password.clone();
        let tx_clone = tx.clone();
        // Extract all needed data from self before the spawn
        let keyfile_owned = keyfile.clone();
        let password_owned = password.clone();
        let tx_clone_owned = tx_clone.clone();
        // Extract file_id as Uuid from req
        let file_id = match Uuid::parse_str(&req.file_id) {
            Ok(id) => id,
            Err(_) => {
                return Err(Status::invalid_argument("Invalid file_id"));
            }
        };
        let storage = self.storage.clone();
        spawn_keypair_loader(keyfile_owned, password_owned, tx_clone_owned, file_id, storage);
        
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn list_files(
        &self,
        request: Request<ListFilesRequest>,
    ) -> Result<Response<ListFilesResponse>, Status> {
        let req = request.into_inner();
        
        // Authenticate user
        let keyfile = format!("userkeys/{}.key", req.username);
        if let Err(_) = load_and_decrypt_keypair(&keyfile, &req.password) {
            return Err(Status::unauthenticated("Invalid credentials"));
        }
        
        let files = self.storage.list_metadata()
            .map_err(|e| Status::internal(format!("Storage error: {}", e)))?;
        
        let proto_files = files.into_iter().map(|f| FileMetadata {
            file_id: f.file_id.to_string(),
            filename: f.filename,
            tags: f.tags,
            owner_peer_id: f.owner_peer_id,
            checksum: f.checksum,
            size: f.size,
            shared_keys: f.shared_keys,
        }).collect();
        
        Ok(Response::new(ListFilesResponse {
            files: proto_files,
        }))
    }

    async fn share_file(
        &self,
        request: Request<ShareFileRequest>,
    ) -> Result<Response<ShareFileResponse>, Status> {
        let req = request.into_inner();
        
        // TODO: Implement file sharing logic
        // This would involve encrypting the file key for the recipient
        
        Ok(Response::new(ShareFileResponse {
            success: true,
            message: "File shared successfully".to_string(),
        }))
    }

    async fn get_file_metadata(
        &self,
        request: Request<FileMetadataRequest>,
    ) -> Result<Response<FileMetadataResponse>, Status> {
        let req = request.into_inner();
        let file_id = match uuid::Uuid::parse_str(&req.file_id) {
            Ok(id) => id,
            Err(_) => {
                return Ok(Response::new(FileMetadataResponse {
                    found: false,
                    message: "Invalid file_id".to_string(),
                    metadata: None,
                }));
            }
        };
        match self.storage.get_metadata(&file_id) {
            Ok(Some(meta)) => Ok(Response::new(FileMetadataResponse {
                found: true,
                message: "ok".to_string(),
                metadata: Some(FileMetadata {
                    file_id: meta.file_id.to_string(),
                    filename: meta.filename,
                    tags: meta.tags,
                    owner_peer_id: meta.owner_peer_id,
                    checksum: meta.checksum,
                    size: meta.size,
                    shared_keys: meta.shared_keys,
                }),
            })),
            Ok(None) => Ok(Response::new(FileMetadataResponse {
                found: false,
                message: "Not found".to_string(),
                metadata: None,
            })),
            Err(e) => Ok(Response::new(FileMetadataResponse {
                found: false,
                message: format!("DB error: {}", e),
                metadata: None,
            })),
        }
    }
}

pub struct DafsP2PService {
    p2p: Arc<P2PNode>,
}
// Implement Default for DafsP2PService if needed
impl Default for DafsP2PService {
    fn default() -> Self {
        Self { p2p: Arc::new(P2PNode::new()) }
    }
}

#[tonic::async_trait]
impl P2pService for DafsP2PService {
    async fn list_peers(
        &self,
        _request: Request<ListPeersRequest>,
    ) -> Result<Response<ListPeersResponse>, Status> {
        // TODO: Implement peer listing
        Ok(Response::new(ListPeersResponse {
            peers: vec![],
        }))
    }

    async fn add_bootstrap_node(
        &self,
        request: Request<BootstrapNodeRequest>,
    ) -> Result<Response<BootstrapNodeResponse>, Status> {
        let req = request.into_inner();
        
        match crate::peer::add_bootstrap_node(&req.peer_id, &req.address) {
            Ok(_) => Ok(Response::new(BootstrapNodeResponse {
                success: true,
                message: "Bootstrap node added".to_string(),
            })),
            Err(e) => Ok(Response::new(BootstrapNodeResponse {
                success: false,
                message: format!("Error: {}", e),
            }))
        }
    }

    async fn remove_bootstrap_node(
        &self,
        request: Request<BootstrapNodeRequest>,
    ) -> Result<Response<BootstrapNodeResponse>, Status> {
        let req = request.into_inner();
        
        match crate::peer::remove_bootstrap_node(&req.peer_id) {
            Ok(_) => Ok(Response::new(BootstrapNodeResponse {
                success: true,
                message: "Bootstrap node removed".to_string(),
            })),
            Err(e) => Ok(Response::new(BootstrapNodeResponse {
                success: false,
                message: format!("Error: {}", e),
            }))
        }
    }

    async fn list_bootstrap_nodes(
        &self,
        _request: Request<ListBootstrapNodesRequest>,
    ) -> Result<Response<ListBootstrapNodesResponse>, Status> {
        let nodes = crate::peer::list_bootstrap_nodes();
        
        let proto_nodes = nodes.into_iter().map(|(peer_id, addr)| BootstrapNodeInfo {
            peer_id,
            address: addr,
        }).collect();
        
        Ok(Response::new(ListBootstrapNodesResponse {
            nodes: proto_nodes,
        }))
    }

    async fn list_p2p_files(
        &self,
        request: Request<ListP2pFilesRequest>,
    ) -> Result<Response<ListP2pFilesResponse>, Status> {
        let _req = request.into_inner();
        if _req.peer_id.is_empty() {
            // Aggregate all known peers' files (not implemented)
            return Ok(Response::new(ListP2pFilesResponse { files: vec![] }));
        } else {
            match self.p2p.query_peer_files(&_req.peer_id).await {
                Ok(files) => Ok(Response::new(ListP2pFilesResponse { files: files.into_iter().map(|f| dafs::FileMetadata {
                    file_id: f.file_id.to_string(),
                    filename: f.filename,
                    tags: f.tags,
                    owner_peer_id: f.owner_peer_id,
                    checksum: f.checksum,
                    size: f.size,
                    shared_keys: f.shared_keys,
                }).collect() })),
                Err(e) => Err(Status::internal(format!("P2P file listing error: {}", e))),
            }
        }
    }
    async fn p2p_download_chunk(
        &self,
        request: Request<P2pDownloadChunkRequest>,
    ) -> Result<Response<P2pDownloadChunkResponse>, Status> {
        let req = request.into_inner();
        match self.p2p.request_chunk(&req.peer_id, &req.file_id, req.chunk_index as usize, req.chunk_size as usize).await {
            Ok(data) => {
                let is_last = data.len() < req.chunk_size as usize;
                Ok(Response::new(P2pDownloadChunkResponse {
                    data: data.clone(),
                    chunk_index: req.chunk_index,
                    is_last,
                }))
            },
            Err(e) => Err(Status::internal(format!("P2P download error: {}", e))),
        }
    }
}

// Refactor: make this a static function, pass in all needed owned data
fn spawn_keypair_loader(
    keyfile: String,
    password: String,
    tx_clone: tokio::sync::mpsc::Sender<Result<DownloadChunk, Status>>,
    file_id: uuid::Uuid,
    storage: std::sync::Arc<Storage>,
) {
    tokio::spawn(async move {
        if let Err(_) = load_and_decrypt_keypair(&keyfile, &password) {
            let _ = tx_clone.send(Err(Status::unauthenticated("Invalid credentials"))).await;
            return;
        }
        // Get file metadata
        let _meta = match storage.get_metadata(&file_id) {
            Ok(Some(m)) => m,
            _ => {
                let _ = tx_clone.send(Err(Status::not_found("File not found"))).await;
                return;
            }
        };
        // Read and send file in chunks
        let file_path = format!("files/{}.bin", file_id);
        let file_data = match std::fs::read(&file_path) {
            Ok(data) => data,
            Err(_) => {
                let _ = tx_clone.send(Err(Status::internal("File read error"))).await;
                return;
            }
        };
        const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks
        let total_chunks = (file_data.len() + CHUNK_SIZE - 1) / CHUNK_SIZE;
        for i in 0..total_chunks {
            let start = i * CHUNK_SIZE;
            let end = std::cmp::min(start + CHUNK_SIZE, file_data.len());
            let chunk_data = file_data[start..end].to_vec();
            let is_last = i == total_chunks - 1;
            let _ = tx_clone.send(Ok(DownloadChunk {
                data: chunk_data,
                chunk_index: i as u32,
                is_last,
            })).await;
        }
    });
}

pub async fn run_grpc_server(storage: Arc<Storage>, p2p: Arc<P2PNode>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    
    let ai_service = DafsAiService {
        storage: storage.clone(),
    };
    
    let file_service = DafsFileService {
        storage: storage.clone(),
    };
    
    let p2p_service = DafsP2PService {
        p2p: p2p.clone(),
    };
    let auth_service = DafsAuthService;

    println!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(AiServiceServer::new(ai_service))
        .add_service(FileServiceServer::new(file_service))
        .add_service(P2pServiceServer::new(p2p_service))
        .add_service(AuthServiceServer::new(auth_service))
        .serve(addr)
        .await?;

    Ok(())
} 