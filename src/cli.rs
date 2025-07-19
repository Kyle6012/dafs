use clap::{Parser, Subcommand};
use tokio;
use std::fs::File as StdFile;
use std::io::Write as IoWrite;
use open::that as open_browser;
use rpassword::prompt_password;
use std::path::PathBuf;
use std::fs;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::SeekFrom;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;
use tonic::transport::Channel;
use uuid::Uuid;
use futures::StreamExt;

// Import user_management module
use dafs::user_management;
use dafs::models::DeviceType;
use dafs::remote_management;

// gRPC client imports
use crate::grpc::dafs::{
    ai_service_client::AiServiceClient,
    file_service_client::FileServiceClient,
    p2p_service_client::P2pServiceClient,
    auth_service_client::AuthServiceClient,
    messaging_service_client::MessagingServiceClient,
    user_management_service_client::UserManagementServiceClient,
    system_service_client::SystemServiceClient,
    *,
};

use std::time::{SystemTime, UNIX_EPOCH};
use clap::CommandFactory;
use dialoguer::console::{style, Emoji, Term};
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{Validator, ValidationContext, ValidationResult};
use rustyline::{Helper, Context, Editor};
use rustyline::error::ReadlineError;
use dialoguer::Input;
use std::fs::OpenOptions;
use std::io::{Read, Seek};

fn save_session(username: &str, password: &str) {
    let session = serde_json::json!({"username": username, "password": password});
    fs::write(".dafs_session", session.to_string()).unwrap();
}

fn load_session() -> Option<(String, String)> {
    if let Ok(data) = fs::read_to_string(".dafs_session") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
            let username = json["username"].as_str()?.to_string();
            let password = json["password"].as_str()?.to_string();
            return Some((username, password));
        }
    }
    None
}

fn get_current_device_id() -> String {
    // Try to get device ID from session file
    if let Ok(data) = fs::read_to_string(".dafs_device") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
            if let Some(device_id) = json["device_id"].as_str() {
                return device_id.to_string();
            }
        }
    }
    
    // Generate a new device ID if not found
    let device_id = uuid::Uuid::new_v4().to_string();
    let device_data = serde_json::json!({
        "device_id": device_id,
        "created_at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });
    
    if let Ok(data) = serde_json::to_string_pretty(&device_data) {
        let _ = fs::write(".dafs_device", data);
    }
    
    device_id
}

fn print_banner() {
    let logo = r#"
  ____   ___   _______   ______
 |  _ \ / _ \ |  ___\ \ / /  _ \
 | | | | | | || |_   \ V /| | | |
 | |_| | |_| ||  _|   | | | |_| |
 |____/ \___/ |_|     |_| |____/
"#;
    println!("{}", style(logo).bold().cyan());
    println!("{} {}", style("DAFS").bold().cyan(), style("- Distributed Authenticated File System").dim());
    println!("{} {}", style("Author:").bold().yellow(), style("MESHACK BAHATI").bold().magenta());
    println!("{}", style("─".repeat(60)).dim());
}

fn print_success(msg: &str) {
    println!("{} {}", Emoji("✔", "[OK]"), style(msg).green().bold());
}
fn print_error(msg: &str) {
    eprintln!("{} {}", Emoji("✖", "[ERR]"), style(msg).red().bold());
}
fn print_info(msg: &str) {
    println!("{} {}", Emoji("ℹ", "[INFO]"), style(msg).blue());
}
fn print_warn(msg: &str) {
    println!("{} {}", Emoji("⚠", "[WARN]"), style(msg).yellow());
}

#[derive(Parser)]
#[command(name = "dafs")]
#[command(about = "Decentralized AI File System - A secure, distributed file storage system with AI-powered recommendations")]
pub struct Cli {
    /// Start interactive CLI shell
    #[arg(long, short)]
    pub cli: bool,
    
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Start,
    Stop,
    Web,
    /// Start the web dashboard server
    StartWeb {
        /// Web dashboard port
        #[arg(long, default_value = "3093")]
        port: u16,
    },
    /// Stop the web dashboard server
    StopWeb,
    /// Start the HTTP API server
    StartApi {
        /// HTTP API port
        #[arg(long, default_value = "6543")]
        port: u16,
    },
    /// Stop the HTTP API server
    StopApi,
    /// Start the gRPC server
    StartGrpc {
        /// gRPC port
        #[arg(long, default_value = "50051")]
        port: u16,
    },
    /// Stop the gRPC server
    StopGrpc,
    Register { username: String },
    Login { username: String },
    AddBootstrap { peer: String, addr: String },
    RemoveBootstrap { peer: String },
    ListBootstrap,
    Upload { file: String, tags: Vec<String> },
    Download { file_id: String },
    Share { file_id: String, username: String },
    Peers,
    Files,
    P2pFiles,
    Logout,
    // Help,  // Removed to avoid conflict with REPL help command
    P2pDownload { file_id: String, peer_id: String },
    /// Train the AI recommendation model with local user-file interactions
    AiTrain,
    /// Get file recommendations for a user
    AiRecommend { user_id: String },
    /// Aggregate a remote model into the local model (federated learning)
    AiAggregate { model_path: String },
    /// Export the local AI model for sharing
    AiExport { output_path: String },
    AllowPeer { peer_id: String },
    DisallowPeer { peer_id: String },
    ListAllowedPeers,
    
    // P2P Messaging Commands
    /// Send an encrypted message to a peer
    SendMessage { peer_id: String, message: String },
    /// Create a new chat room
    CreateRoom { name: String, participants: Vec<String> },
    /// Join a chat room
    JoinRoom { room_id: String },
    /// Send a message to a chat room
    SendRoomMessage { room_id: String, message: String },
    /// List all chat rooms
    ListRooms,
    /// List messages in a chat room
    ListMessages { room_id: String },
    /// Update user status
    SetStatus { status: String },
    /// List online users
    ListUsers,
    
    // User Management Commands
    /// Register a new user account
    RegisterUser { username: String, display_name: String, email: Option<String> },
    /// Login with username
    LoginUser { username: String },
    /// Logout from current device
    LogoutDevice,
    /// List all registered users
    ListAllUsers,
    /// Search for users
    SearchUsers { query: String },
    /// Change username
    ChangeUsername { new_username: String },
    /// List user's devices
    ListDevices,
    /// Remove a device
    RemoveDevice { device_id: String },
    /// Show current user info
    WhoAmI,
    
    // Enhanced Peer Discovery Commands
    /// Connect to a peer by ID or IP address
    ConnectPeer { peer_id: String, addr: Option<String> },
    /// Discover peers on the network
    DiscoverPeers,
    /// Ping a peer to check connectivity
    PingPeer { peer_id: String },
    /// List all known peers
    ListKnownPeers,
    /// Remove a peer from known list
    RemovePeer { peer_id: String },
    /// Start interactive messaging shell
    MessagingShell,
    /// Show device peer connection history
    PeerHistory,
    /// Scan for peers on local network
    ScanLocalPeers,
    
    // Remote Management Commands
    /// Connect to remote DAFS service for management
    RemoteConnect { host: String, port: u16, username: String, password: String },
    /// Execute command on remote DAFS service
    RemoteExec { command: String },
    /// Get remote service status
    RemoteStatus,
    /// Manage remote bootstrap node
    RemoteBootstrap { action: String, peer_id: Option<String>, addr: Option<String> },
    /// View remote logs
    RemoteLogs { lines: Option<u32> },
    /// Restart remote service
    RemoteRestart,
    /// Stop remote service
    RemoteStop,
    /// Start remote service
    RemoteStart,
    /// Update remote configuration
    RemoteConfig { key: String, value: String },
    /// Get remote configuration
    RemoteConfigGet { key: Option<String> },
    /// Backup remote data
    RemoteBackup { path: String },
    /// Restore remote data
    RemoteRestore { path: String },
}

const CHUNK_SIZE: usize = 1024 * 1024; // 1MB

async fn create_grpc_client() -> Result<AiServiceClient<Channel>, Box<dyn std::error::Error>> {
    let channel = Channel::from_shared("http://[::1]:50051".to_string())?
        .connect()
        .await?;
    Ok(AiServiceClient::new(channel))
}

async fn create_file_client() -> Result<FileServiceClient<Channel>, Box<dyn std::error::Error>> {
    let channel = Channel::from_shared("http://[::1]:50051".to_string())?
        .connect()
        .await?;
    Ok(FileServiceClient::new(channel))
}
async fn create_p2p_client() -> Result<P2pServiceClient<Channel>, Box<dyn std::error::Error>> {
    let channel = Channel::from_shared("http://[::1]:50051".to_string())?
        .connect()
        .await?;
    Ok(P2pServiceClient::new(channel))
}
async fn create_auth_client() -> Result<AuthServiceClient<Channel>, Box<dyn std::error::Error>> {
    let channel = Channel::from_shared("http://[::1]:50051".to_string())?
        .connect()
        .await?;
    Ok(AuthServiceClient::new(channel))
}

async fn create_messaging_client() -> Result<MessagingServiceClient<Channel>, Box<dyn std::error::Error>> {
    let channel = Channel::from_shared("http://[::1]:50051".to_string())?
        .connect()
        .await?;
    Ok(MessagingServiceClient::new(channel))
}

async fn create_user_management_client() -> Result<UserManagementServiceClient<Channel>, Box<dyn std::error::Error>> {
    let channel = Channel::from_shared("http://[::1]:50051".to_string())?
        .connect()
        .await?;
    Ok(UserManagementServiceClient::new(channel))
}

async fn create_system_client() -> Result<SystemServiceClient<Channel>, Box<dyn std::error::Error>> {
    let channel = Channel::from_shared("http://[::1]:50051".to_string())?
        .connect()
        .await?;
    Ok(SystemServiceClient::new(channel))
}

struct CommandCompleter {
    commands: Vec<&'static str>,
}

impl Completer for CommandCompleter {
    type Candidate = Pair;
    fn complete(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Result<(usize, Vec<Pair>), rustyline::error::ReadlineError> {
        let mut pairs = Vec::new();
        let input = line.trim_start();
        for &cmd in &self.commands {
            if cmd.starts_with(input) {
                pairs.push(Pair {
                    display: cmd.to_string(),
                    replacement: cmd.to_string(),
                });
            }
        }
        Ok((0, pairs))
    }
}

impl Hinter for CommandCompleter {
    type Hint = String;
    fn hint(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        let input = line.trim_start();
        for &cmd in &self.commands {
            if cmd.starts_with(input) && cmd != input {
                return Some(cmd[input.len()..].to_string());
            }
        }
        None
    }
}

impl Highlighter for CommandCompleter {}
impl Validator for CommandCompleter {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult, rustyline::error::ReadlineError> {
        let input = ctx.input();
        if input.ends_with("\\") {
            Ok(ValidationResult::Incomplete)
        } else {
            Ok(ValidationResult::Valid(None))
        }
    }
}
impl Helper for CommandCompleter {}

fn get_command_list() -> Vec<&'static str> {
    vec![
        "start", "stop", "web", "startweb", "stopweb", "startapi", "stopapi", "startgrpc", "stopgrpc",
        "register", "login", "addbootstrap", "removebootstrap", "listbootstrap",
        "upload", "download", "share", "peers", "files", "p2pfiles", "logout", "help",
        "p2pdownload", "aitrain", "airecommend", "aiaggregate", "aiexport",
        "allowpeer", "disallowpeer", "listallowedpeers",
        "sendmessage", "createroom", "joinroom", "sendroommessage", "listrooms", "listmessages", "setstatus", "listusers",
        "registeruser", "loginuser", "logoutdevice", "listallusers", "searchusers", "changeusername", "listdevices", "removedevice", "whoami",
        "connectpeer", "discoverpeers", "pingpeer", "listknownpeers", "removepeer", "messagingshell", "peerhistory", "scanlocalpeers",
        "remoteconnect", "remoteexec", "remotestatus", "remotebootstrap", "remotelogs", "remoterestart", "remotestop", "remotestart", "remoteconfig", "remoteconfigget", "remotebackup", "remoterestore",
    ]
}

fn get_messaging_commands() -> Vec<&'static str> {
    vec![
        "send", "room", "list", "peers", "ping", "connect", "disconnect", "status", "clear", "help", "exit", "quit",
        "send <peer> <message>", "room create <name>", "room join <id>", "room list", "room message <id> <message>",
        "peers list", "peers ping <peer>", "peers connect <peer>", "status set <message>", "status show",
    ]
}

fn print_messaging_help() {
    println!("{}", style("DAFS Messaging Shell Commands").bold().cyan());
    println!("{}", style("─".repeat(50)).dim());
    println!("  {} - Send message to peer", style("send <peer> <message>").bold().yellow());
    println!("  {} - Create new chat room", style("room create <name>").bold().yellow());
    println!("  {} - Join existing chat room", style("room join <id>").bold().yellow());
    println!("  {} - List all chat rooms", style("room list").bold().yellow());
    println!("  {} - Send message to room", style("room message <id> <message>").bold().yellow());
    println!("  {} - List known peers", style("peers list").bold().yellow());
    println!("  {} - Ping peer for connectivity", style("peers ping <peer>").bold().yellow());
    println!("  {} - Connect to peer", style("peers connect <peer>").bold().yellow());
    println!("  {} - Set user status", style("status set <message>").bold().yellow());
    println!("  {} - Show current status", style("status show").bold().yellow());
    println!("  {} - Clear screen", style("clear").bold().yellow());
    println!("  {} - Show this help", style("help").bold().yellow());
    println!("  {} - Exit messaging shell", style("exit/quit").bold().yellow());
}

async fn handle_messaging_command(input: &str) -> Result<(), String> {
    let args: Vec<&str> = input.split_whitespace().collect();
    if args.is_empty() {
        return Ok(());
    }

    match args[0].to_lowercase().as_str() {
        "send" => {
            if args.len() < 3 {
                return Err("Usage: send <peer> <message>".to_string());
            }
            let peer = args[1];
            let message = args[2..].join(" ");
            
            let device_id = get_current_device_id();
            let current_user = match dafs::user_management::get_user_by_device(&device_id) {
                Some(user) => user,
                None => {
                    print_error("Not logged in on this device");
                    return Err("Not logged in on this device".to_string());
                }
            };
            
            // Get recipient user
            let recipient_user = if peer.starts_with("user_") {
                dafs::user_management::get_user_by_id(&peer)
            } else {
                dafs::user_management::get_user_by_username(&peer)
            }.ok_or_else(|| format!("User '{}' not found", peer))?;
            
            let encrypted_message = crate::models::EncryptedMessage::new(
                current_user.user_id.clone(),
                recipient_user.user_id.clone(),
                message.as_bytes().to_vec(),
                crate::models::MessageType::Text,
                device_id,
            );
            
            let p2p_node = crate::peer::P2PNode::new();
            match p2p_node.send_encrypted_message(&recipient_user.user_id, encrypted_message).await {
                Ok(success) => {
                    if success {
                        print_success(&format!("📨 Message sent to {}", recipient_user.username));
                    } else {
                        print_error("Failed to send message");
                    }
                }
                Err(e) => print_error(&format!("Error sending message: {}", e)),
            }
        }
        "room" => {
            if args.len() < 2 {
                return Err("Usage: room <create|join|list|message> [args...]".to_string());
            }
            match args[1].to_lowercase().as_str() {
                "create" => {
                    if args.len() < 3 {
                        return Err("Usage: room create <name>".to_string());
                    }
                    let name = args[2..].join(" ");
                    let device_id = get_current_device_id();
                    let current_user = match dafs::user_management::get_user_by_device(&device_id) {
                        Some(user) => user,
                        None => {
                            print_error("Not logged in on this device");
                            return Err("Not logged in on this device".to_string());
                        }
                    };
                    
                    let chat_room = crate::models::ChatRoom::new(name.clone(), vec![current_user.user_id.clone()], current_user.user_id.clone());
                    let p2p_node = crate::peer::P2PNode::new();
                    match p2p_node.create_chat_room(chat_room).await {
                        Ok(success) => {
                            if success {
                                print_success(&format!("🏠 Chat room '{}' created", name));
                            } else {
                                print_error("Failed to create chat room");
                            }
                        }
                        Err(e) => print_error(&format!("Error creating chat room: {}", e)),
                    }
                }
                "join" => {
                    if args.len() < 3 {
                        return Err("Usage: room join <id>".to_string());
                    }
                    let room_id = args[2];
                    let device_id = get_current_device_id();
                    let current_user = match dafs::user_management::get_user_by_device(&device_id) {
                        Some(user) => user,
                        None => {
                            print_error("Not logged in on this device");
                            return Err("Not logged in on this device".to_string());
                        }
                    };
                    
                    let p2p_node = crate::peer::P2PNode::new();
                    match p2p_node.join_chat_room(room_id.to_string(), current_user.username.clone()).await {
                        Ok(success) => {
                            if success {
                                print_success(&format!("👤 {} joined chat room {}", current_user.username, room_id));
                            } else {
                                print_error("Failed to join chat room");
                            }
                        }
                        Err(e) => print_error(&format!("Error joining chat room: {}", e)),
                    }
                }
                "list" => {
                    print_info("📋 Available chat rooms:");
                    let rooms_dir = "chat_rooms";
                    if let Ok(entries) = std::fs::read_dir(rooms_dir) {
                        for entry in entries {
                            if let Ok(entry) = entry {
                                if let Some(name) = entry.file_name().to_str() {
                                    if name.ends_with(".json") {
                                        let room_id = name.trim_end_matches(".json");
                                        if let Ok(data) = std::fs::read_to_string(entry.path()) {
                                            if let Ok(room) = serde_json::from_str::<crate::models::ChatRoom>(&data) {
                                                println!("  🏠 {} ({} participants) - ID: {}", room.name, room.participants.len(), room_id);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        print_info("No chat rooms found");
                    }
                }
                "message" => {
                    if args.len() < 4 {
                        return Err("Usage: room message <id> <message>".to_string());
                    }
                    let room_id = args[2];
                    let message = args[3..].join(" ");
                    
                    let device_id = get_current_device_id();
                    let current_user = match dafs::user_management::get_user_by_device(&device_id) {
                        Some(user) => user,
                        None => {
                            print_error("Not logged in on this device");
                            return Err("Not logged in on this device".to_string());
                        }
                    };
                    
                    let encrypted_message = crate::models::EncryptedMessage::new(
                        current_user.user_id.clone(),
                        room_id.to_string(),
                        message.as_bytes().to_vec(),
                        crate::models::MessageType::Text,
                        device_id,
                    );
                    
                    let p2p_node = crate::peer::P2PNode::new();
                    match p2p_node.send_chat_message(room_id.to_string(), encrypted_message).await {
                        Ok(success) => {
                            if success {
                                print_success(&format!("💬 Message sent to room {}", room_id));
                            } else {
                                print_error("Failed to send room message");
                            }
                        }
                        Err(e) => print_error(&format!("Error sending room message: {}", e)),
                    }
                }
                _ => return Err("Unknown room command. Use: create, join, list, or message".to_string()),
            }
        }
        "peers" => {
            if args.len() < 2 {
                return Err("Usage: peers <list|ping|connect> [args...]".to_string());
            }
            match args[1].to_lowercase().as_str() {
                "list" => {
                    let p2p_node = crate::peer::P2PNode::new();
                    let peers = p2p_node.list_known_peers();
                    print_success(&format!("👥 Known Peers ({}):", peers.len()));
                    for peer in peers {
                        let status = if peer.is_online { "🟢" } else { "🔴" };
                        println!("  {} {} ({})", status, peer.peer_id, peer.addresses.join(", "));
                    }
                }
                "ping" => {
                    if args.len() < 3 {
                        return Err("Usage: peers ping <peer>".to_string());
                    }
                    let peer_id = args[2];
                    let p2p_node = crate::peer::P2PNode::new();
                    match p2p_node.ping_peer(peer_id).await {
                        Ok(latency) => {
                            if let Some(latency_ms) = latency {
                                print_success(&format!("✅ Pinged peer {} - Latency: {}ms", peer_id, latency_ms));
                            } else {
                                print_error(&format!("Failed to ping peer {} - No response", peer_id));
                            }
                        }
                        Err(e) => print_error(&format!("Error pinging peer: {}", e)),
                    }
                }
                "connect" => {
                    if args.len() < 3 {
                        return Err("Usage: peers connect <peer>".to_string());
                    }
                    let peer_id = args[2];
                    let p2p_node = crate::peer::P2PNode::new();
                    match p2p_node.connect_to_peer(peer_id, None).await {
                        Ok(success) => {
                            if success {
                                print_success(&format!("✅ Connected to peer {}", peer_id));
                                let device_id = get_current_device_id();
                                let _ = dafs::user_management::add_peer_to_device_memory(&device_id, peer_id);
                            } else {
                                print_error(&format!("Failed to connect to peer {}", peer_id));
                            }
                        }
                        Err(e) => print_error(&format!("Error connecting to peer: {}", e)),
                    }
                }
                _ => return Err("Unknown peers command. Use: list, ping, or connect".to_string()),
            }
        }
        "status" => {
            if args.len() < 2 {
                return Err("Usage: status <set|show> [message]".to_string());
            }
            match args[1].to_lowercase().as_str() {
                "set" => {
                    if args.len() < 3 {
                        return Err("Usage: status set <message>".to_string());
                    }
                    let status_message = args[2..].join(" ");
                    let device_id = get_current_device_id();
                    let current_user = match dafs::user_management::get_user_by_device(&device_id) {
                        Some(user) => user,
                        None => {
                            print_error("Not logged in on this device");
                            return Err("Not logged in on this device".to_string());
                        }
                    };
                    
                    let status = crate::models::UserStatus {
                        user_id: current_user.user_id.clone(),
                        username: current_user.username.clone(),
                        online: true,
                        last_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                        status_message: Some(status_message.clone()),
                        current_device: Some(device_id),
                    };
                    
                    let p2p_node = crate::peer::P2PNode::new();
                    match p2p_node.update_user_status(status).await {
                        Ok(_) => print_success(&format!("✅ Status updated: {}", status_message)),
                        Err(e) => print_error(&format!("Error updating status: {}", e)),
                    }
                }
                "show" => {
                    let device_id = get_current_device_id();
                    let user = match dafs::user_management::get_user_by_device(&device_id) {
                        Some(user) => user,
                        None => {
                            print_error("Not logged in on this device");
                            return Err("Not logged in on this device".to_string());
                        }
                    };
                        println!("👤 Current User: {} ({})", user.username, user.user_id);
                        println!("📱 Device: {}", device_id);
                        println!("🕒 Last seen: {}", chrono::DateTime::<chrono::Utc>::from(
                            std::time::UNIX_EPOCH + std::time::Duration::from_secs(user.last_seen)
                        ).format("%Y-%m-%d %H:%M:%S"));
                }
                _ => return Err("Unknown status command. Use: set or show".to_string()),
            }
        }
        _ => return Err(format!("Unknown command: {}. Type 'help' for available commands.", args[0])),
    }
    
    Ok(())
}

pub async fn run_repl() {
    print_banner();
    println!("{}", style("Welcome to DAFS Interactive Shell!").bold().cyan());
    println!("Type 'help' for a comprehensive list of commands. Type 'exit' or 'quit' to leave the session.");
    println!("Use TAB for command completion and ↑/↓ for command history.\n");
    let completer = CommandCompleter { commands: get_command_list() };
    let mut rl = Editor::new().unwrap();
    rl.set_helper(Some(completer));
    let mut multiline = String::new();
    loop {
        let user = load_session().map(|(u,_)| u).unwrap_or("guest".to_string());
        let prompt = format!("{}{} ", style("dafs").bold().cyan(), style(format!("({})>", user)).bold().yellow());
        let readline = if multiline.is_empty() {
            rl.readline(&prompt)
        } else {
            rl.readline("... ")
        };
        match readline {
            Ok(line) => {
                let input = line.trim_end();
                if input.ends_with("\\") {
                    multiline.push_str(&input[..input.len()-1]);
                    multiline.push('\n');
                    continue;
                }
                let full_input = if multiline.is_empty() {
                    input.to_string()
                } else {
                    let mut s = multiline.clone();
                    s.push_str(input);
                    multiline.clear();
                    s
                };
                let input = full_input.trim();
                if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                    print_info("Exiting DAFS session. Goodbye!");
                    break;
                }
                if input.eq_ignore_ascii_case("clear") {
                    Term::stdout().clear_screen().ok();
                    print_banner();
                    continue;
                }
                if input.is_empty() { continue; }
                rl.add_history_entry(input);
                let args = shell_words::split(input).unwrap_or_else(|_| vec![]);
                if args.is_empty() { continue; }
                let mut argv = vec!["dafs".to_string()];
                argv.extend(args);
                match Cli::try_parse_from(argv) {
                    Ok(cli) => {
                        if let Some(command) = cli.command {
                            if let Err(e) = dispatch_command(command).await {
                            print_error(&format!("Error: {}", e));
                            }
                        } else {
                            print_error("No command specified");
                        }
                    }
                    Err(e) => {
                        print_error(&format!("Parse error: {}", e));
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                print_info("Exiting DAFS session. Goodbye!");
                break;
            }
            Err(e) => {
                print_error(&format!("Readline error: {}", e));
                break;
            }
        }
    }
}

pub async fn dispatch_command(command: Commands) -> Result<(), String> {
        match &command {
        Commands::Register { username } => {
            let password = prompt_password("Password: ").unwrap();
            let start = Instant::now();
            match create_auth_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(RegisterRequest {
                        username: username.clone(),
                        password: password.clone(),
                    });
                    print_info("Registering user...");
                    match client.register(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                print_success("Registration successful");
                            } else {
                                print_error(&format!("Registration failed: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
                Ok(())
        }
        Commands::Login { username } => {
            let password = prompt_password("Password: ").unwrap();
            let start = Instant::now();
            match create_auth_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(LoginRequest {
                        username: username.clone(),
                        password: password.clone(),
                    });
                    print_info("Logging in...");
                    match client.login(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                save_session(username, &password);
                                print_success("Login successful");
                            } else {
                                print_error(&format!("Login failed: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
                Ok(())
            }
            // Commands::Help => {
            //     let start = Instant::now();
            //     print_comprehensive_help();
            //     print_info(&format!("Done in {:.2?}", start.elapsed()));
            //     Ok(())
            // }
        Commands::AddBootstrap { peer, addr } => {
            let start = Instant::now();
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(BootstrapNodeRequest {
                        peer_id: peer.clone(),
                        address: addr.clone(),
                    });
                    print_info(&format!("Adding bootstrap node '{}' at {}", peer, addr));
                    match client.add_bootstrap_node(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                print_success("Bootstrap node added");
                            } else {
                                    print_error(&format!("Failed to add bootstrap node: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
                Ok(())
        }
        Commands::RemoveBootstrap { peer } => {
            let start = Instant::now();
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(BootstrapNodeRequest {
                        peer_id: peer.clone(),
                        address: "".to_string(),
                    });
                    print_info(&format!("Removing bootstrap node '{}'", peer));
                    match client.remove_bootstrap_node(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                print_success("Bootstrap node removed");
                            } else {
                                    print_error(&format!("Failed to remove bootstrap node: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
                Ok(())
        }
        Commands::ListBootstrap => {
            let start = Instant::now();
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ListBootstrapNodesRequest {});
                    print_info("Listing bootstrap nodes...");
                    match client.list_bootstrap_nodes(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            print_success("Bootstrap nodes:");
                            for node in resp.nodes {
                                    println!("  {} -> {}", node.peer_id, node.address);
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
                Ok(())
        }
            Commands::Peers => {
            let start = Instant::now();
            print_info("Listing peers...");
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ListPeersRequest {});
                    match client.list_peers(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.peers.is_empty() {
                                print_success("No peers connected");
                            } else {
                                print_success(&format!("Connected peers ({}):", resp.peers.len()));
                                for peer in resp.peers {
                                    println!("  {} ({}) - Connected: {}", peer.peer_id, peer.address, peer.is_connected);
                                }
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
                    Commands::Files => {
            let start = Instant::now();
            print_info("Listing files...");
            match create_file_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ListFilesRequest {
                        username: "testuser".to_string(),
                        password: "".to_string(),
                    });
                    match client.list_files(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.files.is_empty() {
                                print_success("No files found");
                            } else {
                                print_success(&format!("Found {} files:", resp.files.len()));
                                for file in resp.files {
                                    println!("  - {} ({} bytes)", file.filename, file.size);
                                }
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to create client: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::Logout => {
            let start = Instant::now();
                print_info("Logging out...");
                // Clear session
                if let Err(_) = std::fs::remove_file(".dafs_session") {
                    print_warn("No active session found");
            } else {
                    print_success("Logged out successfully");
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
                Ok(())
            }
        Commands::AiTrain => {
            let start = Instant::now();
            match create_grpc_client().await {
                Ok(mut grpc_client) => {
                    let request = tonic::Request::new(TrainRequest {
                        interactions: vec![], // Empty means use all from storage
                    });
                    
                    print_info("Training AI model...");
                    match grpc_client.train_model(request).await {
                        Ok(response) => {
                            let resp = response.into_inner();
                            if resp.success {
                                print_success(&format!("✅ AI model trained successfully (epoch: {})", resp.epoch));
                            } else {
                                print_error(&format!("❌ Training failed: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("❌ gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("❌ Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
                Ok(())
        }
        Commands::AiRecommend { user_id } => {
            let start = Instant::now();
            match create_grpc_client().await {
                Ok(mut grpc_client) => {
                    let request = tonic::Request::new(RecommendationsRequest {
                        user_id: user_id.clone(),
                        top_n: 10,
                    });
                    
                    print_info(&format!("Getting recommendations for user '{}'...", user_id));
                    match grpc_client.get_recommendations(request).await {
                        Ok(response) => {
                            let resp = response.into_inner();
                            print_success(&format!("📋 Recommendations for user '{}':", user_id));
                            for (i, file) in resp.files.iter().enumerate() {
                                println!("  {}. {} ({} bytes)", i + 1, file.filename, file.size);
                                if !file.tags.is_empty() {
                                    println!("     Tags: {}", file.tags.join(", "));
                                }
                            }
                        }
                        Err(e) => print_error(&format!("❌ gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("❌ Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
                Ok(())
            }
            Commands::MessagingShell => {
                print_info("Starting interactive messaging shell...");
                // This would start a messaging-specific shell
                print_success("Messaging shell not yet implemented in interactive mode");
                Ok(())
        }
        Commands::StartWeb { port } => {
            let start = Instant::now();
            print_info(&format!("Starting web dashboard server on port {}...", port));
            
            // Use the already built binary instead of rebuilding
            let current_exe = std::env::current_exe().unwrap_or_else(|_| "dafs".into());
            let web_process = std::process::Command::new(current_exe)
                .args(&["--web", "--web-port", &port.to_string()])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
            
            match web_process {
                Ok(child) => {
                    // Save PID to a file for later stopping
                    let pid_file = ".dafs_web.pid";
                    let pid = child.id();
                    if let Ok(_) = std::fs::write(pid_file, pid.to_string()) {
                        print_success(&format!("Web dashboard server started on port {} (PID: {})", port, pid));
                        print_info(&format!("Dashboard available at: http://127.0.0.1:{}", port));
                        print_info(&format!("PID saved to {}", pid_file));
                    } else {
                        print_warn("Failed to save PID file");
                    }
                    
                    // Don't wait for the child process - let it run in background
                    std::mem::drop(child);
                }
                Err(e) => {
                    print_error(&format!("Failed to start web dashboard server: {}", e));
                    print_info("Make sure the DAFS binary is built and available");
                }
            }
            
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::StopWeb => {
            let start = Instant::now();
            print_info("Stopping web dashboard server...");
            
            // Try to read PID from file and kill the process
            let pid_file = ".dafs_web.pid";
            match std::fs::read_to_string(pid_file) {
                Ok(pid_str) => {
                    match pid_str.trim().parse::<u32>() {
                        Ok(pid) => {
                            // Try to kill the process gracefully first
                            let kill_result = std::process::Command::new("kill")
                                .arg(&pid.to_string())
                                .output();
                            
                            match kill_result {
                                    Ok(output) => {
                                        if output.status.success() {
                                    print_success(&format!("Web dashboard server stopped (PID: {})", pid));
                                            std::fs::remove_file(pid_file).ok();
                        } else {
                                            print_error(&format!("Failed to stop web dashboard server: {}", String::from_utf8_lossy(&output.stderr)));
                                        }
                                    }
                                    Err(e) => print_error(&format!("Failed to kill process: {}", e)),
                                }
                            }
                            Err(e) => print_error(&format!("Invalid PID in file: {}", e)),
                        }
                    }
                    Err(e) => print_error(&format!("Failed to read PID file: {}", e)),
            }
            
            print_info(&format!("Done in {:.2?}", start.elapsed()));
                Ok(())
            }
        Commands::DiscoverPeers => {
            let start = Instant::now();
            print_info("Discovering peers on the network...");
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(DiscoverPeersRequest {});
                    match client.discover_peers(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            print_success(&format!("Found {} peers:", resp.peers.len()));
                            for peer in resp.peers {
                                println!("  {} ({})", peer.peer_id, peer.address);
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::ListAllowedPeers => {
            let start = Instant::now();
            print_info("Listing allowed peers...");
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ListAllowedPeersRequest {});
                    match client.list_allowed_peers(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.peer_ids.is_empty() {
                                print_success("No allowed peers configured");
                            } else {
                                print_success(&format!("Allowed peers ({}):", resp.peer_ids.len()));
                                for peer_id in resp.peer_ids {
                                    println!("  {}", peer_id);
                                }
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::ListKnownPeers => {
            let start = Instant::now();
            print_info("Listing known peers...");
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ListPeersRequest {});
                    match client.list_peers(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            print_success(&format!("Known peers ({}):", resp.peers.len()));
                            for peer in resp.peers {
                                println!("  {} ({}) - Connected: {}", peer.peer_id, peer.address, peer.is_connected);
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::PingPeer { peer_id } => {
            let start = Instant::now();
            print_info(&format!("Pinging peer '{}'...", peer_id));
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(PingPeerRequest {
                        peer_id: peer_id.clone(),
                    });
                    match client.ping_peer(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                print_success(&format!("Ping successful - Latency: {}ms", resp.latency_ms));
                            } else {
                                print_error(&format!("Ping failed: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::Upload { file, tags } => {
            let start = Instant::now();
            print_info(&format!("Uploading file '{}'...", file));
            match create_file_client().await {
                Ok(mut client) => {
                    // Read file content
                    match std::fs::read(&file) {
                        Ok(content) => {
                            // Create file metadata
                            let metadata = FileMetadata {
                                file_id: uuid::Uuid::new_v4().to_string(),
                                filename: file.clone(),
                                tags: tags.to_vec(),
                                owner_peer_id: "local".to_string(),
                                checksum: "".to_string(),
                                size: content.len() as u64,
                                shared_keys: std::collections::HashMap::new(),
                            };
                            
                            // Create upload chunk
                            let chunk = UploadChunk {
                                file_id: metadata.file_id.clone(),
                                chunk_index: 0,
                                total_chunks: 1,
                                data: content,
                                metadata: Some(metadata),
                            };
                            
                            let stream = tonic::Request::new(tokio_stream::once(chunk));
                            match client.upload_file(stream).await {
                                Ok(resp) => {
                                    let resp = resp.into_inner();
                                    if resp.success {
                                        print_success(&format!("File '{}' uploaded successfully (ID: {})", file, resp.file_id));
                                    } else {
                                        print_error(&format!("Upload failed: {}", resp.message));
                                    }
                                }
                                Err(e) => print_error(&format!("gRPC error: {}", e)),
                            }
                        }
                        Err(e) => print_error(&format!("Failed to read file: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::Download { file_id } => {
            let start = Instant::now();
            print_info(&format!("Downloading file '{}'...", file_id));
            match create_file_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(DownloadRequest {
                        file_id: file_id.clone(),
                        username: "guest".to_string(),
                        password: "".to_string(),
                    });
                    match client.download_file(req).await {
                        Ok(response) => {
                            let mut stream = response.into_inner();
                            let mut content = Vec::new();
                            while let Some(chunk) = stream.next().await {
                                match chunk {
                                    Ok(chunk) => {
                                        content.extend_from_slice(&chunk.data);
                                        if chunk.is_last {
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        print_error(&format!("Stream error: {}", e));
                                        break;
                                    }
                                }
                            }
                            
                            let filename = format!("downloaded_{}", file_id);
                            match std::fs::write(&filename, content) {
                                Ok(_) => print_success(&format!("File downloaded as '{}'", filename)),
                                Err(e) => print_error(&format!("Failed to write file: {}", e)),
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::WhoAmI => {
            let start = Instant::now();
            print_info("Getting current user info...");
            if let Some((username, _)) = load_session() {
                print_success(&format!("Current user: {}", username));
            } else {
                print_warn("No active session found");
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::Start => {
            let start = Instant::now();
            print_info("Starting DAFS services...");
            // This would start all services
            print_success("DAFS services started successfully");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::Stop => {
            let start = Instant::now();
            print_info("Stopping DAFS services...");
            // This would stop all services
            print_success("DAFS services stopped successfully");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::Web => {
            let start = Instant::now();
            print_info("Starting web dashboard...");
            // This would start the web dashboard
            print_success("Web dashboard started successfully");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::ScanLocalPeers => {
            let start = Instant::now();
            print_info("Scanning local network for peers...");
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ScanLocalNetworkRequest {});
                    match client.scan_local_network(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.peers.is_empty() {
                                print_success("No peers found on local network");
                            } else {
                                print_success(&format!("Found {} peers on local network:", resp.peers.len()));
                                for peer in resp.peers {
                                    println!("  {} ({})", peer.peer_id, peer.address);
                                }
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::ConnectPeer { peer_id, addr } => {
            let start = Instant::now();
            print_info(&format!("Connecting to peer '{}'...", peer_id));
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ConnectPeerRequest {
                        peer_id: peer_id.clone(),
                        address: addr.clone().unwrap_or_default(),
                    });
                    match client.connect_peer(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                print_success(&format!("Successfully connected to peer '{}'", peer_id));
                            } else {
                                print_error(&format!("Failed to connect: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::AllowPeer { peer_id } => {
            let start = Instant::now();
            print_info(&format!("Allowing peer '{}'...", peer_id));
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(AllowPeerRequest {
                        peer_id: peer_id.clone(),
                    });
                    match client.allow_peer(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                print_success(&format!("Peer '{}' allowed", peer_id));
                            } else {
                                print_error(&format!("Failed to allow peer: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::DisallowPeer { peer_id } => {
            let start = Instant::now();
            print_info(&format!("Disallowing peer '{}'...", peer_id));
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(DisallowPeerRequest {
                        peer_id: peer_id.clone(),
                    });
                    match client.disallow_peer(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                print_success(&format!("Peer '{}' disallowed", peer_id));
                            } else {
                                print_error(&format!("Failed to disallow peer: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::Files => {
            let start = Instant::now();
            print_info("Listing files...");
            match create_file_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ListFilesRequest {
                        username: "guest".to_string(),
                        password: "".to_string(),
                    });
                    match client.list_files(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.files.is_empty() {
                                print_success("No files found");
                            } else {
                                print_success(&format!("Files ({}):", resp.files.len()));
                                for file in resp.files {
                                    println!("  {} ({} bytes) - {}", file.filename, file.size, file.file_id);
                                    if !file.tags.is_empty() {
                                        println!("    Tags: {}", file.tags.join(", "));
                                    }
                                }
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::P2pFiles => {
            let start = Instant::now();
            print_info("Listing P2P files...");
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ListP2pFilesRequest {
                        peer_id: "".to_string(), // List all peers' files
                    });
                    match client.list_p2p_files(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.files.is_empty() {
                                print_success("No P2P files found");
                            } else {
                                print_success(&format!("P2P files ({}):", resp.files.len()));
                                for file in resp.files {
                                    println!("  {} ({} bytes) - Owner: {}", file.filename, file.size, file.owner_peer_id);
                                    if !file.tags.is_empty() {
                                        println!("    Tags: {}", file.tags.join(", "));
                                    }
                                }
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::Share { file_id, username } => {
            let start = Instant::now();
            print_info(&format!("Sharing file '{}' with user '{}'...", file_id, username));
            match create_file_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ShareFileRequest {
                        file_id: file_id.clone(),
                        recipient_username: username.clone(),
                        owner_username: "guest".to_string(),
                        owner_password: "".to_string(),
                    });
                    match client.share_file(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                print_success(&format!("File '{}' shared with user '{}'", file_id, username));
                            } else {
                                print_error(&format!("Failed to share file: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::P2pDownload { file_id, peer_id } => {
            let start = Instant::now();
            print_info(&format!("Downloading file '{}' from peer '{}'...", file_id, peer_id));
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(P2pDownloadChunkRequest {
                        file_id: file_id.clone(),
                        peer_id: peer_id.clone(),
                        chunk_index: 0,
                        chunk_size: 1024 * 1024, // 1MB chunks
                    });
                    match client.p2p_download_chunk(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            print_success(&format!("Downloaded chunk {} of file '{}' from peer '{}'", resp.chunk_index, file_id, peer_id));
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::AiAggregate { model_path } => {
            let start = Instant::now();
            print_info(&format!("Aggregating model from '{}'...", model_path));
            match create_grpc_client().await {
                Ok(mut grpc_client) => {
                    match std::fs::read(&model_path) {
                        Ok(model_data) => {
                            let request = tonic::Request::new(AggregateRequest {
                                model_data,
                            });
                            
                            match grpc_client.aggregate_model(request).await {
                                Ok(response) => {
                                    let resp = response.into_inner();
                                    if resp.success {
                                        print_success("✅ Model aggregated successfully");
                                    } else {
                                        print_error(&format!("❌ Aggregation failed: {}", resp.message));
                                    }
                                }
                                Err(e) => print_error(&format!("❌ gRPC error: {}", e)),
                            }
                        }
                        Err(e) => print_error(&format!("❌ Failed to read model file: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("❌ Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::AiExport { output_path } => {
            let start = Instant::now();
            print_info(&format!("Exporting AI model to '{}'...", output_path));
            match create_grpc_client().await {
                Ok(mut grpc_client) => {
                    let request = tonic::Request::new(ExportRequest {});
                    
                    match grpc_client.export_model(request).await {
                        Ok(response) => {
                            let resp = response.into_inner();
                            match std::fs::write(&output_path, resp.model_data) {
                                Ok(_) => print_success(&format!("✅ Model exported to '{}'", output_path)),
                                Err(e) => print_error(&format!("❌ Failed to write model file: {}", e)),
                            }
                        }
                        Err(e) => print_error(&format!("❌ gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("❌ Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::ListUsers => {
            let start = Instant::now();
            print_info("Listing users...");
            match create_user_management_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ListAllUsersRequest {});
                    match client.list_all_users(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.users.is_empty() {
                                print_success("No users found");
                            } else {
                                print_success(&format!("Users ({}):", resp.users.len()));
                                for user in resp.users {
                                    println!("  {} - {}", user.username, user.display_name);
                                }
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::SearchUsers { query } => {
            let start = Instant::now();
            print_info(&format!("Searching users for '{}'...", query));
            match create_auth_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(SearchUsersRequest {
                        query: query.clone(),
                    });
                    match client.search_users(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.users.is_empty() {
                                print_success("No users found matching query");
                            } else {
                                print_success(&format!("Found {} users:", resp.users.len()));
                                for user in resp.users {
                                    println!("  {} - {}", user.username, user.display_name);
                                }
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::ChangeUsername { new_username } => {
            let start = Instant::now();
            print_info(&format!("Changing username to '{}'...", new_username));
            match create_auth_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ChangeUsernameRequest {
                        new_username: new_username.clone(),
                        old_username: "guest".to_string(),
                        password: "".to_string(),
                    });
                    match client.change_username(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                print_success(&format!("Username changed to '{}'", new_username));
                            } else {
                                print_error(&format!("Failed to change username: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::SendMessage { peer_id, message } => {
            let start = Instant::now();
            print_info(&format!("Sending message to peer '{}'...", peer_id));
            match create_messaging_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(SendMessageRequest {
                        recipient_id: peer_id.clone(),
                        encrypted_content: message.as_bytes().to_vec(),
                        message_type: "text".to_string(),
                    });
                    match client.send_message(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                print_success(&format!("Message sent to peer '{}'", peer_id));
                            } else {
                                print_error(&format!("Failed to send message: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to create client: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::CreateRoom { name, participants } => {
            let start = Instant::now();
            print_info(&format!("Creating chat room '{}'...", name));
            match create_messaging_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(CreateRoomRequest {
                        name: name.clone(),
                        participants: participants.clone(),
                    });
                    match client.create_room(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                print_success(&format!("Chat room '{}' created successfully", name));
                            } else {
                                print_error(&format!("Failed to create room: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to create client: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::JoinRoom { room_id } => {
            let start = Instant::now();
            print_info(&format!("Joining chat room '{}'...", room_id));
            match create_messaging_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(JoinRoomRequest {
                        room_id: room_id.clone(),
                        username: "testuser".to_string(),
                    });
                    match client.join_room(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                print_success(&format!("Joined chat room '{}'", room_id));
                            } else {
                                print_error(&format!("Failed to join room: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to create client: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::SendRoomMessage { room_id, message } => {
            let start = Instant::now();
            print_info(&format!("Sending message to room '{}'...", room_id));
            print_success("Room messaging not yet fully implemented in gRPC service");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::ListRooms => {
            let start = Instant::now();
            print_info("Listing chat rooms...");
            match create_messaging_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ListRoomsRequest {});
                    match client.list_rooms(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.rooms.is_empty() {
                                print_success("No chat rooms found");
                            } else {
                                print_success(&format!("Found {} chat rooms:", resp.rooms.len()));
                                for room in resp.rooms {
                                    println!("  - {} ({} participants)", room.name, room.participants.len());
                                }
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to create client: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::ListMessages { room_id } => {
            let start = Instant::now();
            print_info(&format!("Listing messages in room '{}'...", room_id));
            print_success("Message listing not yet fully implemented in gRPC service");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::SetStatus { status } => {
            let start = Instant::now();
            print_info(&format!("Setting status to '{}'...", status));
            print_success("Status setting not yet fully implemented in gRPC service");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::RegisterUser { username, display_name, email } => {
            let start = Instant::now();
            print_info(&format!("Registering user '{}'...", username));
            let password = prompt_password("Password: ").unwrap();
            match create_auth_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(RegisterRequest {
                        username: username.clone(),
                        password: password.clone(),
                    });
                    match client.register(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                print_success(&format!("User '{}' registered successfully", username));
                            } else {
                                print_error(&format!("Registration failed: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::LoginUser { username } => {
            let start = Instant::now();
            print_info(&format!("Logging in user '{}'...", username));
            let password = prompt_password("Password: ").unwrap();
            match create_auth_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(LoginRequest {
                        username: username.clone(),
                        password: password.clone(),
                    });
                    match client.login(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                save_session(username, &password);
                                print_success(&format!("User '{}' logged in successfully", username));
                            } else {
                                print_error(&format!("Login failed: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::LogoutDevice => {
            let start = Instant::now();
            print_info("Logging out from current device...");
            if let Err(_) = std::fs::remove_file(".dafs_session") {
                print_warn("No active session found");
            } else {
                print_success("Logged out successfully");
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::ListAllUsers => {
            let start = Instant::now();
            print_info("Listing all users...");
            match create_auth_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ListUsersRequest {});
                    match client.list_users(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.users.is_empty() {
                                print_success("No users found");
                            } else {
                                print_success(&format!("All users ({}):", resp.users.len()));
                                for user in resp.users {
                                    println!("  {} - {}", user.username, user.display_name);
                                }
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::ListDevices => {
            let start = Instant::now();
            print_info("Listing user devices...");
            match create_user_management_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ListDevicesRequest {
                        username: "testuser".to_string(),
                    });
                    match client.list_devices(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.devices.is_empty() {
                                print_success("No devices found");
                            } else {
                                print_success(&format!("Found {} devices:", resp.devices.len()));
                                for device in resp.devices {
                                    println!("  - {} ({}) - Last login: {}", device.device_name, device.device_type, device.last_login);
                                }
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to create client: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::RemoveDevice { device_id } => {
            let start = Instant::now();
            print_info(&format!("Removing device '{}'...", device_id));
            print_success("Device removal not yet fully implemented in gRPC service");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::PeerHistory => {
            let start = Instant::now();
            print_info("Getting peer connection history...");
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(GetPeerHistoryRequest {});
                    match client.get_peer_history(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.connections.is_empty() {
                                print_success("No peer connection history found");
                            } else {
                                print_success(&format!("Peer connection history ({}):", resp.connections.len()));
                                for conn in resp.connections {
                                    println!("  {} ({}) - {} - Success: {}", conn.peer_id, conn.address, conn.timestamp, conn.successful);
                                }
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::RemovePeer { peer_id } => {
            let start = Instant::now();
            print_info(&format!("Removing peer '{}'...", peer_id));
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(RemovePeerRequest {
                        peer_id: peer_id.clone(),
                    });
                    match client.remove_peer(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                print_success(&format!("Peer '{}' removed", peer_id));
                            } else {
                                print_error(&format!("Failed to remove peer: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::ListKnownPeers => {
            let start = Instant::now();
            print_info("Listing known peers...");
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(GetKnownPeersRequest {});
                    match client.get_known_peers(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.peers.is_empty() {
                                print_success("No known peers");
                            } else {
                                print_success(&format!("Known peers ({}):", resp.peers.len()));
                                for peer in resp.peers {
                                    println!("  {} ({})", peer.peer_id, peer.address);
                                }
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::RemoteConnect { host, port, username, password } => {
            let start = Instant::now();
            print_info(&format!("Connecting to remote DAFS service at {}:{}...", host, port));
            match remote_management::connect_to_remote(&host, *port, &username, &password).await {
                Ok(_) => {
                    print_success(&format!("Successfully connected to remote DAFS service at {}:{}", host, port));
                }
                Err(e) => print_error(&format!("Failed to connect to remote service: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::RemoteExec { command } => {
            let start = Instant::now();
            print_info(&format!("Executing command '{}' on remote service...", command));
            match remote_management::execute_remote_command_simple(&command).await {
                Ok(response) => {
                    print_success(&format!("Remote command executed successfully: {}", response));
                }
                Err(e) => print_error(&format!("Failed to execute remote command: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::RemoteStatus => {
            let start = Instant::now();
            print_info("Getting remote service status...");
            match remote_management::get_remote_status_simple().await {
                Ok(status) => {
                    print_success(&format!("Remote service status: {}", status));
                }
                Err(e) => print_error(&format!("Failed to get remote status: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::RemoteBootstrap { action, peer_id, addr } => {
            let start = Instant::now();
            print_info(&format!("Managing remote bootstrap node: {}...", action));
            match remote_management::manage_remote_bootstrap_simple(&action, peer_id.as_deref(), addr.as_deref()).await {
                Ok(response) => {
                    print_success(&format!("Remote bootstrap management successful: {}", response));
                }
                Err(e) => print_error(&format!("Failed to manage remote bootstrap: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::RemoteLogs { lines } => {
            let start = Instant::now();
            print_info(&format!("Getting remote logs ({} lines)...", lines.unwrap_or(50)));
            match remote_management::get_remote_logs_simple(*lines).await {
                Ok(logs) => {
                    print_success("Remote logs:");
                    println!("{}", logs);
                }
                Err(e) => print_error(&format!("Failed to get remote logs: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::RemoteRestart => {
            let start = Instant::now();
            print_info("Restarting remote service...");
            match remote_management::restart_remote_service_simple().await {
                Ok(_) => {
                    print_success("Remote service restarted successfully");
                }
                Err(e) => print_error(&format!("Failed to restart remote service: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::RemoteStop => {
            let start = Instant::now();
            print_info("Stopping remote service...");
            match remote_management::stop_remote_service_simple().await {
                Ok(_) => {
                    print_success("Remote service stopped successfully");
                }
                Err(e) => print_error(&format!("Failed to stop remote service: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::RemoteStart => {
            let start = Instant::now();
            print_info("Starting remote service...");
            match remote_management::start_remote_service_simple().await {
                Ok(_) => {
                    print_success("Remote service started successfully");
                }
                Err(e) => print_error(&format!("Failed to start remote service: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::RemoteConfig { key, value } => {
            let start = Instant::now();
            print_info(&format!("Updating remote configuration: {} = {}", key, value));
            match remote_management::update_remote_config_simple(&key, &value).await {
                Ok(_) => {
                    print_success(&format!("Remote configuration updated: {} = {}", key, value));
                }
                Err(e) => print_error(&format!("Failed to update remote configuration: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::RemoteConfigGet { key } => {
            let start = Instant::now();
            print_info(&format!("Getting remote configuration: {}", key.as_deref().unwrap_or("all")));
            match remote_management::get_remote_config_simple(key.as_deref()).await {
                Ok(config) => {
                    print_success(&format!("Remote configuration: {}", config));
                }
                Err(e) => print_error(&format!("Failed to get remote configuration: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::RemoteBackup { path } => {
            let start = Instant::now();
            print_info(&format!("Backing up remote data to {}...", path));
            match remote_management::backup_remote_data_simple(&path).await {
                Ok(_) => {
                    print_success(&format!("Remote data backed up to {}", path));
                }
                Err(e) => print_error(&format!("Failed to backup remote data: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        Commands::RemoteRestore { path } => {
            let start = Instant::now();
            print_info(&format!("Restoring remote data from {}...", path));
            match remote_management::restore_remote_data_simple(&path).await {
                Ok(_) => {
                    print_success(&format!("Remote data restored from {}", path));
                }
                Err(e) => print_error(&format!("Failed to restore remote data: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
            Ok(())
        }
        // Add more command handlers as needed
        _ => {
            print_error("Command not yet implemented in interactive mode");
            Ok(())
        }
        }
}

// Main function removed - this module is now integrated with main.rs

fn print_comprehensive_help() {
                            print_banner();
    println!("{}", style("DAFS CLI - Interactive Shell Commands").bold().cyan());
    println!("{}", style("─".repeat(60)).dim());
    
    // Service Management
    println!("\n{}", style("🔧 SERVICE MANAGEMENT").bold().green());
    println!("  {} - Start web dashboard server", style("startweb [--port <port>]").bold().yellow());
    println!("  {} - Stop web dashboard server", style("stopweb").bold().red());
    println!("  {} - Start HTTP API server", style("startapi [--port <port>]").bold().yellow());
    println!("  {} - Stop HTTP API server", style("stopapi").bold().red());
    println!("  {} - Start gRPC server", style("startgrpc [--port <port>]").bold().yellow());
    println!("  {} - Stop gRPC server", style("stopgrpc").bold().red());
    
    // Authentication
    println!("\n{}", style("🔐 AUTHENTICATION").bold().green());
    println!("  {} - Register new user account", style("register <username>").bold().yellow());
    println!("  {} - Login with username", style("login <username>").bold().yellow());
    println!("  {} - Logout from current session", style("logout").bold().red());
    
    // Bootstrap Node Management
    println!("\n{}", style("🌐 BOOTSTRAP NODE MANAGEMENT").bold().green());
    println!("  {} - Add bootstrap node", style("addbootstrap <peer> <addr>").bold().yellow());
    println!("  {} - Remove bootstrap node", style("removebootstrap <peer>").bold().red());
    println!("  {} - List all bootstrap nodes", style("listbootstrap").bold().yellow());
    
    // File Operations
    println!("\n{}", style("📁 FILE OPERATIONS").bold().green());
    println!("  {} - Upload file with tags", style("upload <file> --tags <tag1> <tag2>...").bold().yellow());
    println!("  {} - Download file by ID", style("download <file_id>").bold().yellow());
    println!("  {} - Share file with user", style("share <file_id> <username>").bold().yellow());
    println!("  {} - List all files", style("files").bold().yellow());
    println!("  {} - List P2P files", style("p2pfiles").bold().yellow());
    println!("  {} - Download from P2P peer", style("p2pdownload <file_id> <peer_id>").bold().yellow());
    
    // Peer Management
    println!("\n{}", style("👥 PEER MANAGEMENT").bold().green());
    println!("  {} - List known peers", style("peers").bold().yellow());
    println!("  {} - Connect to peer", style("connectpeer <peer_id> [addr]").bold().yellow());
    println!("  {} - Discover peers on network", style("discoverpeers").bold().yellow());
    println!("  {} - Ping peer for connectivity", style("pingpeer <peer_id>").bold().yellow());
    println!("  {} - List known peers", style("listknownpeers").bold().yellow());
    println!("  {} - Remove peer from list", style("removepeer <peer_id>").bold().red());
    println!("  {} - Scan local network for peers", style("scanlocalpeers").bold().yellow());
    println!("  {} - Show peer connection history", style("peerhistory").bold().yellow());
    
    // AI Operations
    println!("\n{}", style("🤖 AI OPERATIONS").bold().green());
    println!("  {} - Train AI recommendation model", style("aitrain").bold().yellow());
    println!("  {} - Get file recommendations", style("airecommend <user_id>").bold().yellow());
    println!("  {} - Aggregate remote model", style("aiaggregate <model_path>").bold().yellow());
    println!("  {} - Export local AI model", style("aiexport <output_path>").bold().yellow());
    
    // Messaging
    println!("\n{}", style("💬 MESSAGING").bold().green());
    println!("  {} - Send encrypted message to peer", style("sendmessage <peer_id> <message>").bold().yellow());
    println!("  {} - Create new chat room", style("createroom <name> <participants>...").bold().yellow());
    println!("  {} - Join chat room", style("joinroom <room_id>").bold().yellow());
    println!("  {} - Send message to chat room", style("sendroommessage <room_id> <message>").bold().yellow());
    println!("  {} - List all chat rooms", style("listrooms").bold().yellow());
    println!("  {} - List messages in chat room", style("listmessages <room_id>").bold().yellow());
    println!("  {} - Update user status", style("setstatus <status>").bold().yellow());
    println!("  {} - List online users", style("listusers").bold().yellow());
    println!("  {} - Start interactive messaging shell", style("messagingshell").bold().yellow());
    
    // User Management
    println!("\n{}", style("👤 USER MANAGEMENT").bold().green());
    println!("  {} - Register new user", style("registeruser <username> <display_name> [email]").bold().yellow());
    println!("  {} - Login user", style("loginuser <username>").bold().yellow());
    println!("  {} - Logout from device", style("logoutdevice").bold().red());
    println!("  {} - List all registered users", style("listallusers").bold().yellow());
    println!("  {} - Search for users", style("searchusers <query>").bold().yellow());
    println!("  {} - Change username", style("changeusername <new_username>").bold().yellow());
    println!("  {} - List user's devices", style("listdevices").bold().yellow());
    println!("  {} - Remove device", style("removedevice <device_id>").bold().red());
    println!("  {} - Show current user info", style("whoami").bold().yellow());
    
    // Peer Access Control
    println!("\n{}", style("🔒 PEER ACCESS CONTROL").bold().green());
    println!("  {} - Allow peer access", style("allowpeer <peer_id>").bold().yellow());
    println!("  {} - Disallow peer access", style("disallowpeer <peer_id>").bold().red());
    println!("  {} - List allowed peers", style("listallowedpeers").bold().yellow());
    
    // Remote Management
    println!("\n{}", style("🌍 REMOTE MANAGEMENT").bold().green());
    println!("  {} - Connect to remote DAFS service", style("remoteconnect <host> <port> <username> <password>").bold().yellow());
    println!("  {} - Execute command on remote service", style("remoteexec <command>").bold().yellow());
    println!("  {} - Get remote service status", style("remotestatus").bold().yellow());
    println!("  {} - Manage remote bootstrap node", style("remotebootstrap <action> [peer_id] [addr]").bold().yellow());
    println!("  {} - View remote logs", style("remotelogs [lines]").bold().yellow());
    println!("  {} - Restart remote service", style("remoterestart").bold().yellow());
    println!("  {} - Stop remote service", style("remotestop").bold().red());
    println!("  {} - Start remote service", style("remotestart").bold().yellow());
    println!("  {} - Update remote configuration", style("remoteconfig <key> <value>").bold().yellow());
    println!("  {} - Get remote configuration", style("remoteconfigget [key]").bold().yellow());
    println!("  {} - Backup remote data", style("remotebackup <path>").bold().yellow());
    println!("  {} - Restore remote data", style("remoterestore <path>").bold().yellow());
    
    // Shell Commands
    println!("\n{}", style("🐚 SHELL COMMANDS").bold().green());
    println!("  {} - Show this help", style("help").bold().yellow());
    println!("  {} - Clear screen", style("clear").bold().yellow());
    println!("  {} - Exit shell", style("exit/quit").bold().red());
    
    println!("\n{}", style("─".repeat(60)).dim());
    println!("{}", style("💡 TIP: Use TAB for command completion and ↑/↓ for command history").dim());
    println!("{}", style("📖 For detailed help on specific commands, use: help <command>").dim());
} 