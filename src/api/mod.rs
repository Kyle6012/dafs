// Placeholder for REST API integration
// In production, use axum or warp

use axum::{Router, routing::get, routing::post, response::IntoResponse, http::StatusCode, extract::Extension, Json, body::Bytes};
use rand::RngCore;
use axum::extract::{Multipart, Query};
use uuid::Uuid;
use std::sync::Arc;
use crate::storage::Storage;
use crate::crypto::encrypt_file;
use std::fs::File;
use std::io::{Write, Seek, SeekFrom, Read};
use crate::crypto::decrypt_file;
use crate::ai::get_recommendations;
use crate::peer::P2PNode;
use crate::crypto::{generate_x25519_keypair, encrypt_and_save_keypair, load_and_decrypt_keypair};
use crate::models::User;
use std::collections::HashMap;
use std::sync::Mutex;
use axum::http::HeaderMap;
use std::fs::{self, OpenOptions};
use crate::ai::{train_local_model, aggregate_remote_model, NCFModel};
use tower_http::cors::{CorsLayer, Any};

pub static USER_DB: once_cell::sync::Lazy<Mutex<HashMap<String, User>>> = once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(serde::Deserialize)]
pub struct UploadMetadata {
    pub filename: String,
    pub tags: Vec<String>,
    pub owner_peer_id: String,
}

#[derive(serde::Deserialize)]
pub struct DownloadQuery {
    pub file_id: String,
}

#[derive(serde::Deserialize)]
pub struct RecommendationsQuery {
    pub user_id: String,
}

#[derive(serde::Deserialize)]
pub struct P2PListQuery {
    pub peer_id: String,
}

#[derive(serde::Deserialize)]
pub struct P2PGetFileQuery {
    pub peer_id: String,
    pub file_id: String,
}

#[derive(serde::Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct AuthDownloadQuery {
    pub file_id: String,
    pub username: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct AuthUploadMetadata {
    pub filename: String,
    pub tags: Vec<String>,
    pub username: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct ShareFileRequest {
    pub file_id: String,
    pub owner_username: String,
    pub owner_password: String,
    pub recipient_username: String,
}

#[derive(serde::Deserialize)]
pub struct RequestFileKey {
    pub file_id: String,
    pub from_peer_id: String,
    pub to_peer_id: Option<String>,
    pub username: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct AcceptSharedFileKey {
    pub file_id: String,
    pub username: String,
    pub encrypted_key: Vec<u8>,
}

#[derive(serde::Deserialize)]
pub struct BootstrapNodeReq {
    pub peer_id: String,
    pub address: String,
}

#[derive(serde::Deserialize)]
pub struct UploadChunkQuery {
    pub file_id: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
}

#[derive(serde::Deserialize)]
pub struct DownloadChunkQuery {
    pub file_id: String,
    pub chunk_index: usize,
    pub chunk_size: usize,
}

#[derive(serde::Deserialize)]
pub struct P2PChunkRequest {
    pub peer_id: String,
    pub file_id: String,
    pub chunk_index: usize,
    pub chunk_size: usize,
}

pub async fn list_files(Extension(storage): Extension<Arc<Storage>>) -> impl IntoResponse {
    match storage.list_metadata() {
        Ok(files) => Json(files).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response(),
    }
}

pub async fn upload_file(
    Extension(storage): Extension<Arc<Storage>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut file_bytes = None;
    let mut metadata = None;
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("");
        if name == "file" {
            file_bytes = Some(field.bytes().await.unwrap());
        } else if name == "metadata" {
            let meta_json = field.text().await.unwrap();
            metadata = serde_json::from_str::<AuthUploadMetadata>(&meta_json).ok();
        }
    }
    let file_bytes = match file_bytes {
        Some(b) => b,
        None => return (StatusCode::BAD_REQUEST, "Missing file").into_response(),
    };
    let metadata = match metadata {
        Some(m) => m,
        None => return (StatusCode::BAD_REQUEST, "Missing metadata").into_response(),
    };
    // Authenticate user and get public key
    let keyfile = format!("userkeys/{}.key", metadata.username);
    let _secret = match load_and_decrypt_keypair(&keyfile, &metadata.password) {
        Ok(s) => s,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid username or password").into_response(),
    };
    let user_db = USER_DB.lock().unwrap();
    let user = match user_db.get(&metadata.username) {
        Some(u) => u,
        None => return (StatusCode::BAD_REQUEST, "Unknown user").into_response(),
    };
    let user_pub = x25519_dalek::PublicKey::from(user.public_key);
    // Generate file ID and per-file encryption key
    let file_id = Uuid::new_v4();
    let mut file_key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut file_key);
    // Encrypt file with file_key
    let encrypted = match encrypt_file(&file_bytes, &file_key) {
        Ok(e) => e,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Encryption error: {}", e)).into_response(),
    };
    // Encrypt file_key with user's public key
    let ephemeral = x25519_dalek::EphemeralSecret::random_from_rng(rand::thread_rng());
    let shared = ephemeral.diffie_hellman(&user_pub);
    let mut encrypted_file_key = file_key.clone();
    for (b, k) in encrypted_file_key.iter_mut().zip(shared.as_bytes()) {
        *b ^= k;
    }
    // Save encrypted file
    let file_path = format!("files/{}.bin", file_id);
    let mut f = match File::create(&file_path) {
        Ok(f) => f,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("File save error: {}", e)).into_response(),
    };
    if let Err(e) = f.write_all(&encrypted) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("File write error: {}", e)).into_response();
    }
    // Save metadata
    let meta = crate::models::FileMetadata {
        file_id,
        filename: metadata.filename,
        tags: metadata.tags,
        owner_peer_id: metadata.username,
        checksum: "TODO".to_string(), // TODO: Compute checksum
        size: file_bytes.len() as u64,
        encrypted_file_key: encrypted_file_key.to_vec(),
        shared_keys: HashMap::new(), // Initialize shared_keys
        allowed_peers: vec![],
    };
    if let Err(e) = storage.insert_metadata(&meta) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Metadata save error: {}", e)).into_response();
    }
    Json(serde_json::json!({"status": "ok", "file_id": file_id})).into_response()
}

pub async fn upload_chunk(
    Query(params): Query<UploadChunkQuery>,
    _headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let temp_dir = format!("upload_tmp/{}", params.file_id);
    fs::create_dir_all(&temp_dir).ok();
    let chunk_path = format!("{}/chunk_{}", temp_dir, params.chunk_index);
    if let Err(e) = fs::write(&chunk_path, &body) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Chunk write error: {}", e)).into_response();
    }
    // Check if all chunks are present
    let mut all_present = true;
    for i in 0..params.total_chunks {
        if !fs::metadata(format!("{}/chunk_{}", temp_dir, i)).is_ok() {
            all_present = false;
            break;
        }
    }
    if all_present {
        // Assemble file
        let final_path = format!("files/{}.bin", params.file_id);
        let mut out = OpenOptions::new().create(true).write(true).truncate(true).open(&final_path).unwrap();
        for i in 0..params.total_chunks {
            let chunk = fs::read(format!("{}/chunk_{}", temp_dir, i)).unwrap();
            out.write_all(&chunk).unwrap();
        }
        let _ = fs::remove_dir_all(&temp_dir);
        return Json(serde_json::json!({"status": "upload complete"})).into_response();
    }
    Json(serde_json::json!({"status": "chunk uploaded"})).into_response()
}

pub async fn download_file(
    Extension(storage): Extension<Arc<Storage>>,
    Query(params): Query<AuthDownloadQuery>,
) -> impl IntoResponse {
    let file_id = match Uuid::parse_str(&params.file_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid file_id").into_response(),
    };
    // Authenticate user and load private key
    let keyfile = format!("userkeys/{}.key", params.username);
    let secret = match load_and_decrypt_keypair(&keyfile, &params.password) {
        Ok(s) => s,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid username or password").into_response(),
    };
    // Fetch metadata
    let meta = match storage.get_metadata(&file_id) {
        Ok(Some(m)) => m,
        Ok(None) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)).into_response(),
    };
    // Access control: only allow owner to download
    if meta.owner_peer_id != params.username {
        // Check if the user is in shared_keys
        let user_db = USER_DB.lock().unwrap();
        let user = user_db.get(&params.username);
        if user.is_none() {
            return (StatusCode::UNAUTHORIZED, "You do not have access to this file").into_response();
        }
        let user_pub = x25519_dalek::PublicKey::from(user.unwrap().public_key);
        let shared = secret.diffie_hellman(&user_pub);
        let mut file_key = meta.encrypted_file_key.clone();
        for (b, k) in file_key.iter_mut().zip(shared.as_bytes()) {
            *b ^= k;
        }
        // Decrypt file
        let decrypted = match decrypt_file(&meta.encrypted_file_key, &file_key.try_into().unwrap_or([0u8; 32])) {
            Ok(d) => d,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Decryption error: {}", e)).into_response(),
        };
        return (
            [("Content-Type", "application/octet-stream"), ("Content-Disposition", &format!("attachment; filename=\"{}\"", meta.filename))],
            decrypted
        ).into_response();
    }
    // Decrypt file key
    let user_pub = x25519_dalek::PublicKey::from(meta.owner_peer_id.as_bytes().try_into().unwrap_or([0u8; 32]));
    let shared = secret.diffie_hellman(&user_pub);
    let mut file_key = meta.encrypted_file_key.clone();
    for (b, k) in file_key.iter_mut().zip(shared.as_bytes()) {
        *b ^= k;
    }
    // Read encrypted file
    let file_path = format!("files/{}.bin", file_id);
    let encrypted = match std::fs::read(&file_path) {
        Ok(b) => b,
        Err(_) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
    };
    // Decrypt file
    let decrypted = match decrypt_file(&encrypted, &file_key.try_into().unwrap_or([0u8; 32])) {
        Ok(d) => d,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Decryption error: {}", e)).into_response(),
    };
    (
        [("Content-Type", "application/octet-stream"), ("Content-Disposition", &format!("attachment; filename=\"{}\"", meta.filename))],
        decrypted
    ).into_response()
}

pub async fn download_chunk(
    Query(params): Query<DownloadChunkQuery>,
) -> impl IntoResponse {
    let file_path = format!("files/{}.bin", params.file_id);
    let mut file = match OpenOptions::new().read(true).open(&file_path) {
        Ok(f) => f,
        Err(_) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
    };
    let offset = params.chunk_index * params.chunk_size;
    if file.seek(SeekFrom::Start(offset as u64)).is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Seek error").into_response();
    }
    let mut buf = vec![0u8; params.chunk_size];
    let n = file.read(&mut buf).unwrap_or(0);
    buf.truncate(n);
    Bytes::from(buf).into_response()
}

pub async fn recommendations(Query(params): Query<RecommendationsQuery>) -> impl IntoResponse {
    let storage = Storage::new("dafs_db").unwrap();
    let files = storage.list_metadata().unwrap_or_default();
    let recs = get_recommendations(&params.user_id, &files);
    Json(recs).into_response()
}

pub async fn p2p_list_files(
    Extension(p2p): Extension<Arc<P2PNode>>,
    Query(params): Query<P2PListQuery>,
) -> impl IntoResponse {
    let peer_id = match params.peer_id.parse() {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid peer_id").into_response(),
    };
    let files = p2p.list_files(peer_id).await;
    Json(files).into_response()
}

pub async fn p2p_get_file(
    Extension(p2p): Extension<Arc<P2PNode>>,
    Query(params): Query<P2PGetFileQuery>,
) -> impl IntoResponse {
    let peer_id = match params.peer_id.parse() {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid peer_id").into_response(),
    };
    let data = p2p.get_file(peer_id, params.file_id).await;
    (
        [("Content-Type", "application/octet-stream")],
        data
    ).into_response()
}

pub async fn p2p_request_chunk(
    Extension(p2p): Extension<Arc<P2PNode>>,
    Json(req): Json<P2PChunkRequest>,
) -> impl IntoResponse {
    match p2p.request_chunk(&req.peer_id, &req.file_id, req.chunk_index, req.chunk_size).await {
        Ok(data) => Bytes::from(data).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("P2P chunk error: {}", e)).into_response(),
    }
}

pub async fn register(Json(req): Json<RegisterRequest>) -> impl IntoResponse {
    let keypair = generate_x25519_keypair();
    let secret = keypair.0;
    let public = keypair.1;
    let keyfile = format!("userkeys/{}.key", req.username);
    std::fs::create_dir_all("userkeys").ok();
    if let Err(e) = encrypt_and_save_keypair(&secret, &keyfile, &req.password) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Key save error: {}", e)).into_response();
    }
    let user = crate::models::User { username: req.username.clone(), public_key: public.to_bytes() };
    USER_DB.lock().unwrap().insert(req.username, user);
    Json(serde_json::json!({"status": "ok"})).into_response()
}

pub async fn login(Json(req): Json<LoginRequest>) -> impl IntoResponse {
    let keyfile = format!("userkeys/{}.key", req.username);
    match load_and_decrypt_keypair(&keyfile, &req.password) {
        Ok(_secret) => Json(serde_json::json!({"status": "ok"})).into_response(),
        Err(_) => (StatusCode::UNAUTHORIZED, "Invalid username or password").into_response(),
    }
}

pub async fn share_file(
    Extension(storage): Extension<Arc<Storage>>,
    Json(req): Json<ShareFileRequest>,
) -> impl IntoResponse {
    let file_id = match Uuid::parse_str(&req.file_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid file_id").into_response(),
    };
    // Authenticate owner
    let keyfile = format!("userkeys/{}.key", req.owner_username);
    let owner_secret = match load_and_decrypt_keypair(&keyfile, &req.owner_password) {
        Ok(s) => s,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid owner credentials").into_response(),
    };
    // Fetch metadata
    let mut meta = match storage.get_metadata(&file_id) {
        Ok(Some(m)) => m,
        Ok(None) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)).into_response(),
    };
    // Only owner can share
    if meta.owner_peer_id != req.owner_username {
        return (StatusCode::FORBIDDEN, "Only the owner can share this file").into_response();
    }
    // Get file key
    let owner_pub = x25519_dalek::PublicKey::from(meta.owner_peer_id.as_bytes().try_into().unwrap_or([0u8; 32]));
    let shared = owner_secret.diffie_hellman(&owner_pub);
    let mut file_key = meta.encrypted_file_key.clone();
    for (b, k) in file_key.iter_mut().zip(shared.as_bytes()) {
        *b ^= k;
    }
    // Get recipient public key
    let user_db = USER_DB.lock().unwrap();
    let recipient = match user_db.get(&req.recipient_username) {
        Some(u) => u,
        None => return (StatusCode::BAD_REQUEST, "Unknown recipient").into_response(),
    };
    let recipient_pub = x25519_dalek::PublicKey::from(recipient.public_key);
    let encrypted_for_recipient = crate::peer::encrypt_file_key_for_peer(&file_key.try_into().unwrap_or([0u8; 32]), &recipient_pub);
    // Store in shared_keys
    meta.shared_keys.insert(req.recipient_username.clone(), encrypted_for_recipient);
    if let Err(e) = storage.insert_metadata(&meta) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Metadata update error: {}", e)).into_response();
    }
    Json(serde_json::json!({"status": "ok"})).into_response()
}

pub async fn request_file_key(
    Extension(storage): Extension<Arc<Storage>>,
    Json(req): Json<RequestFileKey>,
) -> impl IntoResponse {
    // Authenticate user (must be owner)
    let keyfile = format!("userkeys/{}.key", req.username);
    let secret = match load_and_decrypt_keypair(&keyfile, &req.password) {
        Ok(s) => s,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid username or password").into_response(),
    };
    // Look up file metadata
    let file_id = match Uuid::parse_str(&req.file_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid file_id").into_response(),
    };
    let storage = Storage::new("dafs_db").unwrap();
    let meta = match storage.get_metadata(&file_id) {
        Ok(Some(m)) => m,
        _ => return (StatusCode::NOT_FOUND, "File not found").into_response(),
    };
    if meta.owner_peer_id != req.username {
        return (StatusCode::FORBIDDEN, "Only the owner can share this file").into_response();
    }
    // Decrypt file key
    let owner_pub = x25519_dalek::PublicKey::from(meta.owner_peer_id.as_bytes().try_into().unwrap_or([0u8; 32]));
    let shared = secret.diffie_hellman(&owner_pub);
    let mut file_key = meta.encrypted_file_key.clone();
    for (b, k) in file_key.iter_mut().zip(shared.as_bytes()) {
        *b ^= k;
    }
    // Encrypt file key for recipient
    let user_db = USER_DB.lock().unwrap();
    // Fix: Use to_peer_id as String directly, do not call .parse(), use &to_peer_id for user_db.get
    let to_peer_id = match req.to_peer_id.clone() {
        Some(val) => val,
        None => return (StatusCode::BAD_REQUEST, "No peer id provided").into_response(),
    };
    let recipient = match user_db.get(&to_peer_id) {
        Some(u) => u,
        None => return (StatusCode::BAD_REQUEST, "Unknown recipient").into_response(),
    };
    let recipient_pub = x25519_dalek::PublicKey::from(recipient.public_key);
    let encrypted_for_recipient = crate::peer::encrypt_file_key_for_peer(&file_key.try_into().unwrap_or([0u8; 32]), &recipient_pub);
    let msg = crate::peer::P2PMessage::FileKeyExchange {
        file_id: req.file_id.clone(),
        encrypted_key: encrypted_for_recipient,
        from: req.from_peer_id.clone(),
        to: to_peer_id,
    };
    // This part needs to be adapted to use a P2PNode extension or a newtype extractor
    // For now, we'll just return a placeholder response
    // In a real scenario, you'd call p2p.send_message(peer_id, msg).await;
    Json(serde_json::json!({"status": "request sent"})).into_response()
}

pub async fn accept_shared_file_key(
    Extension(storage): Extension<Arc<Storage>>,
    Json(req): Json<AcceptSharedFileKey>,
) -> impl IntoResponse {
    let file_id = match Uuid::parse_str(&req.file_id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid file_id").into_response(),
    };
    let mut meta = match storage.get_metadata(&file_id) {
        Ok(Some(m)) => m,
        Ok(None) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)).into_response(),
    };
    meta.shared_keys.insert(req.username.clone(), req.encrypted_key);
    if let Err(e) = storage.insert_metadata(&meta) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Metadata update error: {}", e)).into_response();
    }
    Json(serde_json::json!({"status": "ok"})).into_response()
}

pub async fn add_bootstrap_node(Json(req): Json<BootstrapNodeReq>) -> impl IntoResponse {
    match crate::peer::add_bootstrap_node(&req.peer_id, &req.address) {
        Ok(_) => Json(serde_json::json!({"status": "ok"})).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, format!("Error: {}", e)).into_response(),
    }
}

pub async fn remove_bootstrap_node(Json(req): Json<BootstrapNodeReq>) -> impl IntoResponse {
    match crate::peer::remove_bootstrap_node(&req.peer_id) {
        Ok(_) => Json(serde_json::json!({"status": "ok"})).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, format!("Error: {}", e)).into_response(),
    }
}

pub async fn list_bootstrap_nodes() -> impl IntoResponse {
    let nodes = crate::peer::list_bootstrap_nodes();
    Json(nodes).into_response()
}

/// POST /ai/train: Triggers local model training (stub: uses all user-file pairs in storage)
pub async fn ai_train(Extension(storage): Extension<Arc<Storage>>) -> impl IntoResponse {
    // For demo: collect all user-file pairs
    let files = match storage.list_metadata() {
        Ok(f) => f,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Storage error: {}", e)).into_response(),
    };
    let mut interactions = Vec::new();
    for f in &files {
        interactions.push((f.owner_peer_id.clone(), f.file_id.to_string()));
        for (user, _) in &f.shared_keys {
            interactions.push((user.clone(), f.file_id.to_string()));
        }
    }
    match train_local_model(&interactions) {
        Ok(_) => Json(serde_json::json!({"status": "ok"})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("AI train error: {}", e)).into_response(),
    }
}

/// GET /ai/recommend: Returns recommendations for a user (alias for /recommendations)
pub async fn ai_recommend(Query(params): Query<RecommendationsQuery>, Extension(storage): Extension<Arc<Storage>>) -> impl IntoResponse {
    let files = match storage.list_metadata() {
        Ok(f) => f,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Storage error: {}", e)).into_response(),
    };
    match get_recommendations(&params.user_id, &files) {
        Ok(recs) => Json(recs).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("AI recommend error: {}", e)).into_response(),
    }
}

/// POST /ai/aggregate: Accepts a model file and aggregates it into the local model
pub async fn ai_aggregate(body: Bytes) -> impl IntoResponse {
    match bincode::deserialize::<NCFModel>(&body) {
        Ok(remote_model) => match aggregate_remote_model(&remote_model) {
            Ok(_) => Json(serde_json::json!({"status": "ok"})).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("AI aggregate error: {}", e)).into_response(),
        },
        Err(e) => (StatusCode::BAD_REQUEST, format!("Model deserialize error: {}", e)).into_response(),
    }
}

pub async fn run_with_storage_and_p2p(storage: Arc<Storage>, p2p: Arc<P2PNode>) {
    let app = Router::new()
        .route("/files", get(list_files))
        .route("/files/upload", post(upload_file))
        .route("/files/upload_chunk", post(upload_chunk))
        .route("/files/download", get(download_file))
        .route("/files/download_chunk", get(download_chunk))
        .route("/recommendations", get(recommendations))
        .route("/p2p/list_files", get(p2p_list_files))
        .route("/p2p/get_file", get(p2p_get_file))
        .route("/p2p/request_chunk", post(p2p_request_chunk))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/share_file", post(share_file))
        .route("/request_file_key", post(request_file_key))
        .route("/accept_shared_file_key", post(accept_shared_file_key))
        .route("/add_bootstrap_node", post(add_bootstrap_node))
        .route("/remove_bootstrap_node", post(remove_bootstrap_node))
        .route("/list_bootstrap_nodes", get(list_bootstrap_nodes))
        .route("/ai/train", post(ai_train))
        .route("/ai/recommend", get(ai_recommend))
        .route("/ai/aggregate", post(ai_aggregate))
        ;
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    let app = app
        .layer(cors)
        .layer(Extension(storage))
        .layer(Extension(p2p));

    let addr: std::net::SocketAddr = "0.0.0.0:6543".parse().unwrap();
    println!("API server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::Server::from_tcp(listener.into_std().unwrap()).unwrap()
        .serve(app.into_make_service())
        .await
        .unwrap();
} 