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

// Import user_management module
use dafs::user_management;
use dafs::models::DeviceType;

// gRPC client imports
use crate::grpc::dafs::{
    ai_service_client::AiServiceClient,
    file_service_client::FileServiceClient,
    p2p_service_client::P2pServiceClient,
    auth_service_client::AuthServiceClient,
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
struct Cli {
    /// Start interactive CLI shell
    #[arg(long, short)]
    cli: bool,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
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

async fn dispatch_command(command: Commands) -> Result<(), String> {
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
                // This would typically call a gRPC service
                print_success("Peer discovery not yet implemented in interactive mode");
                print_info(&format!("Done in {:.2?}", start.elapsed()));
                Ok(())
            }
            Commands::Files => {
                let start = Instant::now();
                print_info("Listing files...");
                // This would typically call a gRPC service
                print_success("File listing not yet implemented in interactive mode");
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
                
                // Start web dashboard as a separate process
                let web_process = std::process::Command::new("cargo")
                    .args(&["run", "--", "--web", "--web-port", &port.to_string()])
                    .spawn();
                
                match web_process {
                    Ok(mut child) => {
                        // Save PID to a file for later stopping
                        let pid_file = ".dafs_web.pid";
                        let pid = child.id();
                        if let Ok(_) = std::fs::write(pid_file, pid.to_string()) {
                            print_success(&format!("Web dashboard server started on port {} (PID: {})", port, pid));
                            print_info(&format!("PID saved to {}", pid_file));
                        } else {
                            print_warn("Failed to save PID file");
                        }
                        
                        // Don't wait for the child process - let it run in background
                        std::mem::drop(child);
                    }
                    Err(e) => {
                        print_error(&format!("Failed to start web dashboard server: {}", e));
                        print_info("Make sure you're in the DAFS project directory");
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
            // Add more command handlers as needed
            _ => {
                print_error("Command not yet implemented in interactive mode");
                Ok(())
            }
        }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Create a runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;
    
    // If --cli flag is provided or no arguments given, start interactive shell
    if cli.cli || std::env::args().len() == 1 {
        rt.block_on(run_repl());
        return Ok(());
    }
    
    // If no command provided, show help
    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            print_banner();
            print_comprehensive_help();
            return Ok(());
        }
    };
    
    print_banner();
    let term = Term::stdout();
    
    // Use dispatch_command to handle all commands
    let result = rt.block_on(dispatch_command(command));
    
    // Convert String error to Box<dyn std::error::Error>
    result.map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Box<dyn std::error::Error>)
}

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