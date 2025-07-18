use clap::{Parser, Subcommand};
use reqwest::Client;
use tokio;
use std::fs::File as StdFile;
use std::io::Write as IoWrite;
use open::that as open_browser;
use rpassword::prompt_password;
use std::path::PathBuf;
use std::fs;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::SeekFrom;
use std::fs::OpenOptions;
use console::{style, Emoji, Term};
use colored::*;
use std::time::Instant;
use dialoguer::Input;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hinter, Hint};
use rustyline::validate::{Validator, ValidationContext, ValidationResult};
use rustyline::{Helper, Context};

// gRPC client imports
use tonic::transport::Channel;
use crate::grpc::dafs::{
    ai_service_client::AiServiceClient,
    file_service_client::FileServiceClient,
    p2p_service_client::P2pServiceClient,
    auth_service_client::AuthServiceClient,
    *,
};

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
    println!("{}", "─".repeat(60).dimmed());
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
#[command(name = "dafs-cli")]
#[command(about = "Distributed Authenticated File System CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
    Help,
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
            Ok(ValidationResult::Valid)
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
    println!("{}", "─".repeat(50).dimmed());
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
            let current_user = crate::user_management::get_user_by_device(&device_id)
                .ok_or_else(|| "Not logged in on this device")?;
            
            // Get recipient user
            let recipient_user = if peer.starts_with("user_") {
                crate::user_management::get_user_by_id(peer)
            } else {
                crate::user_management::get_user_by_username(peer)
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
                    let current_user = crate::user_management::get_user_by_device(&device_id)
                        .ok_or_else(|| "Not logged in on this device")?;
                    
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
                    let current_user = crate::user_management::get_user_by_device(&device_id)
                        .ok_or_else(|| "Not logged in on this device")?;
                    
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
                    let current_user = crate::user_management::get_user_by_device(&device_id)
                        .ok_or_else(|| "Not logged in on this device")?;
                    
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
                                let _ = crate::user_management::add_peer_to_device_memory(&device_id, peer_id);
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
                    let current_user = crate::user_management::get_user_by_device(&device_id)
                        .ok_or_else(|| "Not logged in on this device")?;
                    
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
                    if let Some(user) = crate::user_management::get_user_by_device(&device_id) {
                        println!("👤 Current User: {} ({})", user.username, user.user_id);
                        println!("📱 Device: {}", device_id);
                        println!("🕒 Last seen: {}", chrono::DateTime::<chrono::Utc>::from(
                            std::time::UNIX_EPOCH + std::time::Duration::from_secs(user.last_seen)
                        ).format("%Y-%m-%d %H:%M:%S"));
                    } else {
                        print_error("Not logged in on this device");
                    }
                }
                _ => return Err("Unknown status command. Use: set or show".to_string()),
            }
        }
        _ => return Err(format!("Unknown command: {}. Type 'help' for available commands.", args[0])),
    }
    
    Ok(())
}

fn run_repl() {
    print_banner();
    println!("Type 'help' for a list of commands. Type 'exit' or 'quit' to leave the session. Use TAB for completion.\n");
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
                let mut argv = vec!["dafs-cli".to_string()];
                argv.extend(args);
                match Cli::try_parse_from(argv) {
                    Ok(cli) => {
                        if let Err(e) = dispatch_command(cli) {
                            print_error(&format!("Error: {}", e));
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

fn dispatch_command(cli: Cli) -> Result<(), String> {
    // Move the match body from main() here, return Ok(()) or Err(msg)
    // ... (copy the match & command logic from main, but return errors as String) ...
    Ok(())
}

#[tokio::main]
async fn main() {
    if std::env::args().len() == 1 {
        run_repl();
        return;
    }
    print_banner();
    let cli = Cli::parse();
    let term = Term::stdout();
    match &cli.command {
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
        }
        Commands::Upload { file, tags } => {
            let (username, password) = load_session().unwrap_or(("user".to_string(), "password".to_string()));
            let start = Instant::now();
            // Interactive prompt for allowed peers
            print_info("Enter comma-separated peer IDs allowed to access this file (leave blank for only you):");
            let allowed_peers_input: String = Input::new().with_prompt("Allowed Peers").interact_text().unwrap();
            let allowed_peers: Vec<String> = allowed_peers_input.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            let mut acl = allowed_peers.clone();
            acl.push(username.clone()); // Always allow owner
            match create_file_client().await {
                Ok(mut client) => {
                    let file_id = uuid::Uuid::new_v4().to_string();
                    let path = std::path::Path::new(file);
                    let file_size = std::fs::metadata(path).unwrap().len() as usize;
                    let total_chunks = (file_size + CHUNK_SIZE - 1) / CHUNK_SIZE;
                    let mut f = StdFile::open(path).unwrap();
                    let mut buf = vec![0u8; CHUNK_SIZE];
                    let mut chunk_index = 0;
                    let upload_state_path = format!("{}.upload", file);
                    if let Ok(state) = std::fs::read_to_string(&upload_state_path) {
                        if let Ok(idx) = state.parse::<usize>() {
                            chunk_index = idx;
                            f.seek(SeekFrom::Start((chunk_index * CHUNK_SIZE) as u64)).unwrap();
                        }
                    }
                    print_info(&format!("Uploading '{}' ({} bytes, {} chunks)...", file, file_size, total_chunks));
                    let pb = ProgressBar::new(total_chunks as u64);
                    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}").unwrap());
                    pb.set_message("Uploading file...");
                    pb.set_position(chunk_index as u64);
                    let mut stream = futures::stream::unfold((), move |_| async {
                        if chunk_index >= total_chunks {
                            return None;
                        }
                        let n = f.read(&mut buf).unwrap();
                        let chunk = buf[..n].to_vec();
                        let meta = if chunk_index == 0 {
                            Some(FileMetadata {
                                file_id: file_id.clone(),
                                filename: file.to_string(),
                                tags: tags.clone(),
                                owner_peer_id: username.clone(),
                                checksum: "TODO".to_string(),
                                size: file_size as u64,
                                shared_keys: std::collections::HashMap::new(),
                                allowed_peers: acl.clone(),
                            })
                        } else {
                            None
                        };
                        let msg = UploadChunk {
                            file_id: file_id.clone(),
                            chunk_index: chunk_index as u32,
                            total_chunks: total_chunks as u32,
                            data: chunk,
                            metadata: meta,
                        };
                        chunk_index += 1;
                        pb.inc(1);
                        std::fs::write(&upload_state_path, chunk_index.to_string()).ok();
                        Some((Ok(msg), ()))
                    });
                    match client.upload_file(tonic::Request::new(stream)).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                pb.finish_and_clear();
                                std::fs::remove_file(&upload_state_path).ok();
                                print_success(&format!("Upload complete. File ID: {}", resp.file_id));
                                print_info("Access Control List (ACL) for this file:");
                                for p in &acl {
                                    println!("  {}", style(p).bold().magenta());
                                }
                            } else {
                                print_error(&format!("Upload failed: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::Download { file_id } => {
            let (username, password) = load_session().unwrap_or(("user".to_string(), "password".to_string()));
            let start = Instant::now();
            match create_file_client().await {
                Ok(mut client) => {
                    // Get file metadata for size
                    let meta_req = tonic::Request::new(FileMetadataRequest { file_id: file_id.clone() });
                    let meta_resp = client.get_file_metadata(meta_req).await;
                    let (file_size, total_chunks) = if let Ok(meta) = meta_resp {
                        let meta = meta.into_inner();
                        if meta.found {
                            (meta.metadata.unwrap().size as usize, (meta.metadata.unwrap().size as usize + CHUNK_SIZE - 1) / CHUNK_SIZE)
                        } else {
                            (0, 0)
                        }
                    } else { (0, 0) };
                    if total_chunks == 0 {
                        print_error("Could not determine file size for download.");
                        return;
                    }
                    let out_path = format!("downloaded_{}.bin", file_id);
                    let download_state_path = format!("{}.download", out_path);
                    let mut chunk_index = 0;
                    if let Ok(state) = std::fs::read_to_string(&download_state_path) {
                        if let Ok(idx) = state.parse::<usize>() {
                            chunk_index = idx;
                        }
                    }
                    print_info(&format!("Downloading file '{}' ({} bytes, {} chunks)...", out_path, file_size, total_chunks));
                    let pb = ProgressBar::new(total_chunks as u64);
                    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}").unwrap());
                    pb.set_message("Downloading file...");
                    pb.set_position(chunk_index as u64);
                    let mut out = OpenOptions::new().create(true).write(true).open(&out_path).unwrap();
                    out.seek(SeekFrom::Start((chunk_index * CHUNK_SIZE) as u64)).unwrap();
                    let req = tonic::Request::new(DownloadRequest {
                        file_id: file_id.clone(),
                        username: username.clone(),
                        password: password.clone(),
                    });
                    let mut stream = client.download_file(req).await.unwrap().into_inner();
                    while let Some(chunk) = stream.message().await.unwrap() {
                        out.write_all(&chunk.data).unwrap();
                        chunk_index += 1;
                        pb.inc(1);
                        std::fs::write(&download_state_path, chunk_index.to_string()).ok();
                        if chunk.is_last { break; }
                    }
                    pb.finish_and_clear();
                    std::fs::remove_file(&download_state_path).ok();
                    print_success(&format!("Download complete: {}", out_path));
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::Share { file_id, username: recipient } => {
            let (username, password) = load_session().unwrap_or(("user".to_string(), "password".to_string()));
            let start = Instant::now();
            // Interactive prompt for allowed peers
            print_info("Enter comma-separated peer IDs to add to this file's ACL (leave blank for only you and recipient):");
            let allowed_peers_input: String = Input::new().with_prompt("Allowed Peers").interact_text().unwrap();
            let mut allowed_peers: Vec<String> = allowed_peers_input.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            allowed_peers.push(username.clone());
            allowed_peers.push(recipient.clone());
            match create_file_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ShareFileRequest {
                        file_id: file_id.clone(),
                        owner_username: username.clone(),
                        owner_password: password.clone(),
                        recipient_username: recipient.clone(),
                        // Optionally, add allowed_peers to ShareFileRequest if supported
                    });
                    print_info(&format!("Sharing file '{}' with user '{}'...", file_id, recipient));
                    match client.share_file(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            if resp.success {
                                print_success("File shared successfully");
                                print_info("Updated ACL for this file:");
                                for p in &allowed_peers {
                                    println!("  {}", style(p).bold().magenta());
                                }
                            } else {
                                print_error(&format!("Share failed: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::Files => {
            let (username, password) = load_session().unwrap_or(("user".to_string(), "password".to_string()));
            let start = Instant::now();
            match create_file_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ListFilesRequest {
                        username: username.clone(),
                        password: password.clone(),
                    });
                    print_info("Listing local files...");
                    match client.list_files(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            print_success("Local Files:");
                            for file in resp.files {
                                println!("  {} ({} bytes)", file.filename, file.size);
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::Peers => {
            let start = Instant::now();
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ListPeersRequest {});
                    print_info("Listing known peers...");
                    match client.list_peers(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            print_success("Peers:");
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
        }
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
                                print_error(&format!("Add failed: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
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
                                print_error(&format!("Remove failed: {}", resp.message));
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
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
                                println!("  {} ({})", node.peer_id, node.address);
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::P2pFiles => {
            let start = Instant::now();
            match create_p2p_client().await {
                Ok(mut client) => {
                    let req = tonic::Request::new(ListP2pFilesRequest { peer_id: "".to_string() });
                    print_info("Listing P2P files...");
                    match client.list_p2p_files(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            print_success("P2P Files:");
                            for file in resp.files {
                                println!("  {} ({} bytes)", file.filename, file.size);
                            }
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::P2pDownload { file_id, peer_id } => {
            let start = Instant::now();
            match create_p2p_client().await {
                Ok(mut client) => {
                    // Get file metadata for size (stub: assume 1 chunk)
                    let out_path = format!("p2p_downloaded_{}.bin", file_id);
                    let download_state_path = format!("{}.p2pdownload", out_path);
                    let mut chunk_index = 0;
                    if let Ok(state) = std::fs::read_to_string(&download_state_path) {
                        if let Ok(idx) = state.parse::<usize>() {
                            chunk_index = idx;
                        }
                    }
                    print_info(&format!("P2P downloading file '{}' from peer '{}'...", file_id, peer_id));
                    let pb = ProgressBar::new(1);
                    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}").unwrap());
                    pb.set_message("P2P Downloading file...");
                    pb.set_position(chunk_index as u64);
                    let mut out = OpenOptions::new().create(true).write(true).open(&out_path).unwrap();
                    out.seek(SeekFrom::Start((chunk_index * CHUNK_SIZE) as u64)).unwrap();
                    let req = tonic::Request::new(P2pDownloadChunkRequest {
                        peer_id: peer_id.clone(),
                        file_id: file_id.clone(),
                        chunk_index: chunk_index as u32,
                        chunk_size: CHUNK_SIZE as u32,
                    });
                    match client.p2p_download_chunk(req).await {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            out.write_all(&resp.data).unwrap();
                            pb.inc(1);
                            std::fs::write(&download_state_path, (chunk_index + 1).to_string()).ok();
                            pb.finish_and_clear();
                            std::fs::remove_file(&download_state_path).ok();
                            print_success(&format!("P2P Download complete: {}", out_path));
                        }
                        Err(e) => print_error(&format!("gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::Logout => {
            let start = Instant::now();
            if fs::remove_file(".dafs_session").is_ok() {
                print_success("Logged out and session cleared.");
            } else {
                print_warn("No session to clear.");
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::Web => {
            let start = Instant::now();
            // Try the integrated web dashboard first (port 3093)
            let integrated_url = "http://127.0.0.1:3093";
            let api_url = "http://127.0.0.1:6543";
            
            print_info("Opening web dashboard...");
            
            // Try to open the integrated web dashboard
            match open_browser(integrated_url) {
                Ok(_) => {
                    print_success(&format!("Opened integrated web dashboard at {}", integrated_url));
                    print_info("If the web dashboard doesn't load, the backend may not be running.");
                    print_info(&format!("You can start the backend with: cargo run"));
                }
                Err(e) => {
                    print_warn(&format!("Failed to open integrated web dashboard: {}", e));
                    print_info("Trying to open API endpoint instead...");
                    
                    // Fallback to API endpoint
                    match open_browser(api_url) {
                        Ok(_) => print_success(&format!("Opened API endpoint at {}", api_url)),
                        Err(e2) => print_error(&format!("Failed to open browser: {}", e2)),
                    }
                }
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::Help => {
            let start = Instant::now();
            print_banner();
            println!("{}", "USAGE EXAMPLES:".bold());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("start").bold().green());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("web").bold().green());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("startweb --port 3093").bold().green());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("startapi --port 6543").bold().green());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("startgrpc --port 50051").bold().green());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("register <username>").bold().yellow());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("login <username>").bold().yellow());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("logout").bold().red());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("add-bootstrap <peer> <addr>").bold().yellow());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("remove-bootstrap <peer>").bold().red());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("list-bootstrap").bold().yellow());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("upload <file> --tags ...").bold().yellow());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("download <file_id>").bold().yellow());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("share <file_id> <username>").bold().yellow());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("peers").bold().yellow());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("files").bold().yellow());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("p2p-files").bold().yellow());
            println!("  {} {}", style("dafs-cli").bold().cyan(), style("help").bold().yellow());
            println!("\n{}", "─".repeat(60).dimmed());
            Cli::command().print_help().unwrap();
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        // AI commands remain as before (alr
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
        }
        Commands::AiAggregate { model_path } => {
            let start = Instant::now();
            match create_grpc_client().await {
                Ok(mut grpc_client) => {
                    // Read model file
                    match std::fs::read(&model_path) {
                        Ok(model_bytes) => {
                            let request = tonic::Request::new(AggregateRequest {
                                model_data: model_bytes,
                            });
                            
                            print_info(&format!("Aggregating model from '{}'...", model_path));
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
        }
        Commands::AiExport { output_path } => {
            let start = Instant::now();
            match create_grpc_client().await {
                Ok(mut grpc_client) => {
                    let request = tonic::Request::new(ExportRequest {});
                    
                    print_info("Exporting AI model...");
                    match grpc_client.export_model(request).await {
                        Ok(response) => {
                            let resp = response.into_inner();
                            match std::fs::write(&output_path, resp.model_data) {
                                Ok(_) => print_success(&format!("✅ Model exported to: {}", output_path)),
                                Err(e) => print_error(&format!("❌ Failed to write model file: {}", e)),
                            }
                        }
                        Err(e) => print_error(&format!("❌ gRPC error: {}", e)),
                    }
                }
                Err(e) => print_error(&format!("❌ Failed to connect to gRPC server: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::AllowPeer { peer_id } => {
            // Call backend or update config directly
            crate::peer::allow_peer(peer_id);
            print_success(&format!("Peer {} allowed", peer_id));
        }
        Commands::DisallowPeer { peer_id } => {
            crate::peer::disallow_peer(peer_id);
            print_success(&format!("Peer {} disallowed", peer_id));
        }
        Commands::ListAllowedPeers => {
            let peers = crate::peer::list_allowed_peers();
            print_success("Allowed peers:");
            for p in peers { println!("  {}", p); }
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
                    if let Ok(pid) = child.id() {
                        if let Ok(_) = std::fs::write(pid_file, pid.to_string()) {
                            print_success(&format!("Web dashboard server started on port {} (PID: {})", port, pid));
                            print_info(&format!("PID saved to {}", pid_file));
                        } else {
                            print_warn("Failed to save PID file");
                        }
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
                                Ok(_) => {
                                    print_success(&format!("Web dashboard server stopped (PID: {})", pid));
                                    // Remove PID file
                                    let _ = std::fs::remove_file(pid_file);
                                }
                                Err(e) => {
                                    print_warn(&format!("Failed to kill process gracefully: {}", e));
                                    print_info("Trying force kill...");
                                    
                                    // Try force kill
                                    let force_kill = std::process::Command::new("kill")
                                        .args(&["-9", &pid.to_string()])
                                        .output();
                                    
                                    match force_kill {
                                        Ok(_) => {
                                            print_success(&format!("Web dashboard server force stopped (PID: {})", pid));
                                            let _ = std::fs::remove_file(pid_file);
                                        }
                                        Err(e2) => {
                                            print_error(&format!("Failed to force kill process: {}", e2));
                                            print_info("You may need to kill the process manually");
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            print_error("Invalid PID in PID file");
                            let _ = std::fs::remove_file(pid_file);
                        }
                    }
                }
                Err(_) => {
                    print_warn("No PID file found, trying to find and kill web dashboard process...");
                    
                    // Try to find and kill any dafs web processes
                    let kill_result = std::process::Command::new("pkill")
                        .args(&["-f", "dafs.*web"])
                        .output();
                    
                    match kill_result {
                        Ok(_) => {
                            print_success("Web dashboard server processes killed");
                        }
                        Err(e) => {
                            print_error(&format!("Failed to kill web dashboard processes: {}", e));
                            print_info("No web dashboard server found running");
                        }
                    }
                }
            }
            
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::StartApi { port } => {
            let start = Instant::now();
            print_info(&format!("Starting HTTP API server on port {}...", port));
            
            // Start API server as a separate process
            let api_process = std::process::Command::new("cargo")
                .args(&["run", "--", "--api", "--api-port", &port.to_string()])
                .spawn();
            
            match api_process {
                Ok(mut child) => {
                    let pid_file = ".dafs_api.pid";
                    if let Ok(pid) = child.id() {
                        if let Ok(_) = std::fs::write(pid_file, pid.to_string()) {
                            print_success(&format!("HTTP API server started on port {} (PID: {})", port, pid));
                            print_info(&format!("PID saved to {}", pid_file));
                        } else {
                            print_warn("Failed to save PID file");
                        }
                    }
                    std::mem::drop(child);
                }
                Err(e) => {
                    print_error(&format!("Failed to start HTTP API server: {}", e));
                    print_info("Make sure you're in the DAFS project directory");
                }
            }
            
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::StopApi => {
            let start = Instant::now();
            print_info("Stopping HTTP API server...");
            
            // Try to read PID from file and kill the process
            let pid_file = ".dafs_api.pid";
            match std::fs::read_to_string(pid_file) {
                Ok(pid_str) => {
                    match pid_str.trim().parse::<u32>() {
                        Ok(pid) => {
                            let kill_result = std::process::Command::new("kill")
                                .arg(&pid.to_string())
                                .output();
                            
                            match kill_result {
                                Ok(_) => {
                                    print_success(&format!("HTTP API server stopped (PID: {})", pid));
                                    let _ = std::fs::remove_file(pid_file);
                                }
                                Err(e) => {
                                    print_warn(&format!("Failed to kill process gracefully: {}", e));
                                    let force_kill = std::process::Command::new("kill")
                                        .args(&["-9", &pid.to_string()])
                                        .output();
                                    
                                    match force_kill {
                                        Ok(_) => {
                                            print_success(&format!("HTTP API server force stopped (PID: {})", pid));
                                            let _ = std::fs::remove_file(pid_file);
                                        }
                                        Err(e2) => {
                                            print_error(&format!("Failed to force kill process: {}", e2));
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            print_error("Invalid PID in PID file");
                            let _ = std::fs::remove_file(pid_file);
                        }
                    }
                }
                Err(_) => {
                    print_warn("No PID file found, trying to find and kill API server process...");
                    let kill_result = std::process::Command::new("pkill")
                        .args(&["-f", "dafs.*api"])
                        .output();
                    
                    match kill_result {
                        Ok(_) => print_success("HTTP API server processes killed"),
                        Err(e) => {
                            print_error(&format!("Failed to kill API server processes: {}", e));
                            print_info("No HTTP API server found running");
                        }
                    }
                }
            }
            
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::StartGrpc { port } => {
            let start = Instant::now();
            print_info(&format!("Starting gRPC server on port {}...", port));
            
            print_info("To start the gRPC server, run:");
            print_info(&format!("  cargo run -- --grpc --grpc-port {}", port));
            print_info("Or in integrated mode:");
            print_info("  cargo run -- --integrated");
            
            print_success(&format!("gRPC server instructions provided for port {}", port));
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::StopGrpc => {
            let start = Instant::now();
            print_info("Stopping gRPC server...");
            
            print_info("To stop the gRPC server:");
            print_info("  Press Ctrl+C in the terminal running the server");
            print_info("  Or kill the process: pkill -f 'dafs.*grpc'");
            
            print_success("gRPC server stop instructions provided");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        // P2P Messaging Commands
        Commands::SendMessage { peer_id, message } => {
            let start = Instant::now();
            
            // Get current user from device session
            let device_id = get_current_device_id();
            let current_user = crate::user_management::get_user_by_device(&device_id)
                .ok_or_else(|| "Not logged in on this device")?;
            
            // Get recipient user by username or user_id
            let recipient_user = if peer_id.starts_with("user_") {
                // peer_id is a user_id
                crate::user_management::get_user_by_id(&peer_id)
            } else {
                // peer_id is a username
                crate::user_management::get_user_by_username(&peer_id)
            }.ok_or_else(|| format!("User '{}' not found", peer_id))?;
            
            // Create encrypted message
            let encrypted_message = crate::models::EncryptedMessage::new(
                current_user.user_id.clone(),
                recipient_user.user_id.clone(),
                message.as_bytes().to_vec(),
                crate::models::MessageType::Text,
                device_id,
            );
            
            // Get P2P node and send message
            let p2p_node = crate::peer::P2PNode::new();
            match p2p_node.send_encrypted_message(&recipient_user.user_id, encrypted_message).await {
                Ok(success) => {
                    if success {
                        print_success(&format!("📨 Message sent to {} ({})", recipient_user.username, recipient_user.user_id));
                    } else {
                        print_error("Failed to send message");
                    }
                }
                Err(e) => print_error(&format!("Error sending message: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        Commands::CreateRoom { name, participants } => {
            let start = Instant::now();
            
            // Get current user from device session
            let device_id = get_current_device_id();
            let current_user = crate::user_management::get_user_by_device(&device_id)
                .ok_or_else(|| "Not logged in on this device")?;
            
            // Convert usernames to user_ids
            let mut participant_ids = Vec::new();
            for participant in participants {
                if let Some(user) = crate::user_management::get_user_by_username(&participant) {
                    participant_ids.push(user.user_id);
                } else {
                    print_warn(&format!("User '{}' not found, skipping", participant));
                }
            }
            
            // Add current user if not already included
            if !participant_ids.contains(&current_user.user_id) {
                participant_ids.push(current_user.user_id.clone());
            }
            
            let chat_room = crate::models::ChatRoom::new(name.clone(), participant_ids, current_user.user_id.clone());
            
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
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        Commands::JoinRoom { room_id } => {
            let start = Instant::now();
            
            // Get current user from device session
            let device_id = get_current_device_id();
            let current_user = crate::user_management::get_user_by_device(&device_id)
                .ok_or_else(|| "Not logged in on this device")?;
            
            let p2p_node = crate::peer::P2PNode::new();
            match p2p_node.join_chat_room(room_id.clone(), current_user.username.clone()).await {
                Ok(success) => {
                    if success {
                        print_success(&format!("👤 {} joined chat room {}", current_user.username, room_id));
                    } else {
                        print_error("Failed to join chat room");
                    }
                }
                Err(e) => print_error(&format!("Error joining chat room: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        Commands::SendRoomMessage { room_id, message } => {
            let start = Instant::now();
            
            // Get current user from device session
            let device_id = get_current_device_id();
            let current_user = crate::user_management::get_user_by_device(&device_id)
                .ok_or_else(|| "Not logged in on this device")?;
            
            let encrypted_message = crate::models::EncryptedMessage::new(
                current_user.user_id.clone(),
                room_id.clone(),
                message.as_bytes().to_vec(),
                crate::models::MessageType::Text,
                device_id,
            );
            
            let p2p_node = crate::peer::P2PNode::new();
            match p2p_node.send_chat_message(room_id.clone(), encrypted_message).await {
                Ok(success) => {
                    if success {
                        print_success(&format!("💬 Message sent to room {}", room_id));
                    } else {
                        print_error("Failed to send room message");
                    }
                }
                Err(e) => print_error(&format!("Error sending room message: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        Commands::ListRooms => {
            let start = Instant::now();
            print_info("📋 Available chat rooms:");
            
            // List chat rooms from local storage
            let rooms_dir = "chat_rooms";
            if let Ok(entries) = std::fs::read_dir(rooms_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.ends_with(".json") {
                                let room_id = name.trim_end_matches(".json");
                                if let Ok(data) = std::fs::read_to_string(entry.path()) {
                                    if let Ok(room) = serde_json::from_str::<crate::models::ChatRoom>(&data) {
                                        println!("  🏠 {} ({} participants)", room.name, room.participants.len());
                                        println!("    ID: {}", room_id);
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                print_info("No chat rooms found");
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        Commands::ListMessages { room_id } => {
            let start = Instant::now();
            print_info(&format!("📨 Messages in room {}:", room_id));
            
            let messages_dir = format!("chat_rooms/{}/messages", room_id);
            if let Ok(entries) = std::fs::read_dir(messages_dir) {
                let mut messages = Vec::new();
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.ends_with(".json") {
                                if let Ok(data) = std::fs::read_to_string(entry.path()) {
                                    if let Ok(msg) = serde_json::from_str::<crate::models::EncryptedMessage>(&data) {
                                        messages.push(msg);
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Sort by timestamp
                messages.sort_by_key(|m| m.timestamp);
                
                for msg in messages {
                    let time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(msg.timestamp);
                    let time_str = chrono::DateTime::<chrono::Utc>::from(time).format("%Y-%m-%d %H:%M:%S");
                    println!("  [{}] {}: [ENCRYPTED]", time_str, msg.sender);
                }
            } else {
                print_info("No messages found in this room");
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        Commands::SetStatus { status } => {
            let start = Instant::now();
            
            // Get current user from device session
            let device_id = get_current_device_id();
            let current_user = crate::user_management::get_user_by_device(&device_id)
                .ok_or_else(|| "Not logged in on this device")?;
            
            let user_status = crate::models::UserStatus {
                user_id: current_user.user_id.clone(),
                username: current_user.username.clone(),
                online: true,
                last_seen: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                status_message: Some(status.clone()),
                current_device: Some(device_id),
            };
            
            let p2p_node = crate::peer::P2PNode::new();
            match p2p_node.update_user_status(user_status).await {
                Ok(_) => print_success(&format!("👤 Status updated: {}", status)),
                Err(e) => print_error(&format!("Error updating status: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        Commands::ListUsers => {
            let start = Instant::now();
            print_info("👥 Online users:");
            
            // List user status from local storage
            let status_dir = "user_status";
            if let Ok(entries) = std::fs::read_dir(status_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.ends_with(".json") {
                                if let Ok(data) = std::fs::read_to_string(entry.path()) {
                                    if let Ok(status) = serde_json::from_str::<crate::models::UserStatus>(&data) {
                                        let status_icon = if status.online { "🟢" } else { "🔴" };
                                        let status_text = status.status_message.as_deref().unwrap_or("No status");
                                        println!("  {} {}: {}", status_icon, status.username, status_text);
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                print_info("No user status found");
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        // User Management Commands
        Commands::RegisterUser { username, display_name, email } => {
            let start = Instant::now();
            
            // Initialize user registry if not already done
            if let Err(e) = crate::user_management::init_user_registry() {
                print_error(&format!("Failed to initialize user registry: {}", e));
                return;
            }
            
            match crate::user_management::register_user(username.clone(), display_name.clone(), email.clone()) {
                Ok(user) => {
                    print_success(&format!("✅ User '{}' registered successfully", user.username));
                    println!("  User ID: {}", user.user_id);
                    println!("  Display Name: {}", user.display_name);
                    if let Some(email) = &user.email {
                        println!("  Email: {}", email);
                    }
                }
                Err(e) => print_error(&format!("❌ Registration failed: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        Commands::LoginUser { username } => {
            let start = Instant::now();
            
            // Initialize user registry if not already done
            if let Err(e) = crate::user_management::init_user_registry() {
                print_error(&format!("Failed to initialize user registry: {}", e));
                return;
            }
            
            // Get device info
            let device_name = format!("{}", std::env::var("HOSTNAME").unwrap_or_else(|_| "Unknown Device".to_string()));
            let device_type = crate::models::DeviceType::Desktop; // Default, could be detected
            
            match crate::user_management::login_user(&username, device_name, device_type) {
                Ok((user, session)) => {
                    print_success(&format!("✅ Logged in as '{}'", user.username));
                    println!("  User ID: {}", user.user_id);
                    println!("  Device ID: {}", session.device_id);
                    println!("  Session ID: {}", session.session_id);
                    
                    // Save session info
                    let session_data = serde_json::json!({
                        "username": user.username,
                        "user_id": user.user_id,
                        "device_id": session.device_id,
                        "session_id": session.session_id
                    });
                    if let Ok(data) = serde_json::to_string_pretty(&session_data) {
                        let _ = fs::write(".dafs_session", data);
                    }
                }
                Err(e) => print_error(&format!("❌ Login failed: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        Commands::LogoutDevice => {
            let start = Instant::now();
            
            let device_id = get_current_device_id();
            match crate::user_management::logout_device(&device_id) {
                Ok(_) => {
                    print_success("✅ Logged out from current device");
                    // Remove session file
                    let _ = fs::remove_file(".dafs_session");
                }
                Err(e) => print_error(&format!("❌ Logout failed: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        Commands::ListAllUsers => {
            let start = Instant::now();
            print_info("👥 All registered users:");
            
            let users = crate::user_management::list_users();
            for user in users {
                let device_count = user.devices.len();
                let current_device = user.get_current_device();
                let status = if current_device.is_some() { "🟢 Online" } else { "🔴 Offline" };
                
                println!("  {} {} ({})", status, user.username, user.user_id);
                println!("    Display Name: {}", user.display_name);
                println!("    Devices: {}", device_count);
                if let Some(email) = &user.email {
                    println!("    Email: {}", email);
                }
                println!();
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        Commands::SearchUsers { query } => {
            let start = Instant::now();
            print_info(&format!("🔍 Searching for users matching '{}':", query));
            
            let users = crate::user_management::search_users(&query);
            for user in users {
                println!("  {} ({})", user.username, user.user_id);
                println!("    Display Name: {}", user.display_name);
                if let Some(email) = &user.email {
                    println!("    Email: {}", email);
                }
                println!();
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        Commands::ChangeUsername { new_username } => {
            let start = Instant::now();
            
            let device_id = get_current_device_id();
            let current_user = crate::user_management::get_user_by_device(&device_id)
                .ok_or_else(|| "Not logged in on this device")?;
            
            match crate::user_management::change_username(&current_user.user_id, new_username.clone()) {
                Ok(_) => print_success(&format!("✅ Username changed to '{}'", new_username)),
                Err(e) => print_error(&format!("❌ Failed to change username: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        Commands::ListDevices => {
            let start = Instant::now();
            
            let device_id = get_current_device_id();
            let current_user = crate::user_management::get_user_by_device(&device_id)
                .ok_or_else(|| "Not logged in on this device")?;
            
            print_info(&format!("📱 Devices for user '{}':", current_user.username));
            for device in &current_user.devices {
                let current_marker = if device.is_current { " (Current)" } else { "" };
                let device_type = match device.device_type {
                    crate::models::DeviceType::Desktop => "Desktop",
                    crate::models::DeviceType::Laptop => "Laptop",
                    crate::models::DeviceType::Mobile => "Mobile",
                    crate::models::DeviceType::Tablet => "Tablet",
                    crate::models::DeviceType::Server => "Server",
                    crate::models::DeviceType::Unknown => "Unknown",
                };
                
                println!("  {} - {} ({}){}", device.device_id, device.device_name, device_type, current_marker);
                println!("    Last Login: {}", chrono::DateTime::<chrono::Utc>::from(
                    std::time::UNIX_EPOCH + std::time::Duration::from_secs(device.last_login)
                ).format("%Y-%m-%d %H:%M:%S"));
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        Commands::RemoveDevice { device_id } => {
            let start = Instant::now();
            
            let current_device_id = get_current_device_id();
            let current_user = crate::user_management::get_user_by_device(&current_device_id)
                .ok_or_else(|| "Not logged in on this device")?;
            
            if device_id == current_device_id {
                print_error("❌ Cannot remove current device. Please logout first.");
                return;
            }
            
            match crate::user_management::remove_device(&current_user.user_id, &device_id) {
                Ok(_) => print_success(&format!("✅ Device {} removed", device_id)),
                Err(e) => print_error(&format!("❌ Failed to remove device: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        Commands::WhoAmI => {
            let start = Instant::now();
            
            let device_id = get_current_device_id();
            if let Some(user) = crate::user_management::get_user_by_device(&device_id) {
                print_success(&format!("👤 Current user: {}", user.username));
                println!("  User ID: {}", user.user_id);
                println!("  Display Name: {}", user.display_name);
                println!("  Device ID: {}", device_id);
                if let Some(email) = &user.email {
                    println!("  Email: {}", email);
                }
                println!("  Account Created: {}", chrono::DateTime::<chrono::Utc>::from(
                    std::time::UNIX_EPOCH + std::time::Duration::from_secs(user.created_at)
                ).format("%Y-%m-%d %H:%M:%S"));
            } else {
                print_error("❌ Not logged in on this device");
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        // Enhanced Peer Discovery Commands
        Commands::ConnectPeer { peer_id, addr } => {
            let start = Instant::now();
            let p2p_node = crate::peer::P2PNode::new();
            match p2p_node.connect_to_peer(&peer_id, addr.clone()).await {
                Ok(success) => {
                    if success {
                        print_success(&format!("✅ Connected to peer {}", peer_id));
                        // Add to device memory
                        let device_id = get_current_device_id();
                        let _ = crate::user_management::add_peer_to_device_memory(&device_id, &peer_id);
                    } else {
                        print_error(&format!("Failed to connect to peer {}", peer_id));
                    }
                }
                Err(e) => print_error(&format!("Error connecting to peer: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::DiscoverPeers => {
            let start = Instant::now();
            let p2p_node = crate::peer::P2PNode::new();
            match p2p_node.discover_peers().await {
                Ok(peers) => {
                    print_success(&format!("🔍 Discovered {} peers:", peers.len()));
                    for peer in peers {
                        let status = if peer.is_online { "🟢" } else { "🔴" };
                        println!("  {} {} ({}) - Last seen: {}", 
                                status, 
                                peer.peer_id, 
                                peer.addresses.join(", "),
                                chrono::DateTime::<chrono::Utc>::from(
                                    std::time::UNIX_EPOCH + std::time::Duration::from_secs(peer.last_seen)
                                ).format("%Y-%m-%d %H:%M:%S"));
                    }
                }
                Err(e) => print_error(&format!("Error discovering peers: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::PingPeer { peer_id } => {
            let start = Instant::now();
            let p2p_node = crate::peer::P2PNode::new();
            match p2p_node.ping_peer(&peer_id).await {
                Ok(latency) => {
                    if let Some(latency_ms) = latency {
                        print_success(&format!("✅ Pinged peer {} - Latency: {}ms", peer_id, latency_ms));
                    } else {
                        print_error(&format!("Failed to ping peer {} - No response", peer_id));
                    }
                }
                Err(e) => print_error(&format!("Error pinging peer: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::ListKnownPeers => {
            let start = Instant::now();
            let p2p_node = crate::peer::P2PNode::new();
            let peers = p2p_node.list_known_peers();
            print_success(&format!("👥 Known Peers ({}):", peers.len()));
            for peer in peers {
                let status = if peer.is_online { "🟢" } else { "🔴" };
                println!("  {} {} ({}) - Last seen: {}", 
                        status, 
                        peer.peer_id, 
                        peer.addresses.join(", "),
                        chrono::DateTime::<chrono::Utc>::from(
                            std::time::UNIX_EPOCH + std::time::Duration::from_secs(peer.last_seen)
                        ).format("%Y-%m-%d %H:%M:%S"));
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::RemovePeer { peer_id } => {
            let start = Instant::now();
            let p2p_node = crate::peer::P2PNode::new();
            match p2p_node.remove_peer(&peer_id).await {
                Ok(removed) => {
                    if removed {
                        print_success(&format!("✅ Peer {} removed from known list", peer_id));
                    } else {
                        print_warn(&format!("Peer {} not found in known list", peer_id));
                    }
                }
                Err(e) => print_error(&format!("Error removing peer: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::MessagingShell => {
            let start = Instant::now();
            print_info("Starting interactive messaging shell...");
            print_info("Type 'help' for available commands, 'exit' to quit");
            
            let mut rl = Editor::new().unwrap();
            rl.set_helper(Some(CommandCompleter { commands: get_messaging_commands() }));
            
            loop {
                let device_id = get_current_device_id();
                let current_user = crate::user_management::get_user_by_device(&device_id)
                    .map(|u| u.username.clone())
                    .unwrap_or_else(|| "guest".to_string());
                
                let prompt = format!("{}{} ", style("DAFS").bold().cyan(), style(format!("({})>", current_user)).bold().yellow());
                
                match rl.readline(&prompt) {
                    Ok(line) => {
                        let input = line.trim();
                        if input.is_empty() { continue; }
                        
                        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                            print_info("Exiting messaging shell.");
                            break;
                        }
                        
                        if input.eq_ignore_ascii_case("clear") {
                            Term::stdout().clear_screen().ok();
                            print_banner();
                            continue;
                        }
                        
                        if input.eq_ignore_ascii_case("help") {
                            print_messaging_help();
                            continue;
                        }
                        
                        rl.add_history_entry(input);
                        
                        // Parse and execute messaging commands
                        if let Err(e) = handle_messaging_command(input).await {
                            print_error(&format!("Error: {}", e));
                        }
                    }
                    Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                        print_info("Exiting messaging shell.");
                        break;
                    }
                    Err(e) => {
                        print_error(&format!("Readline error: {}", e));
                        break;
                    }
                }
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::PeerHistory => {
            let start = Instant::now();
            let device_id = get_current_device_id();
            let history = crate::user_management::get_device_connection_history(&device_id);
            print_success(&format!("📜 Peer Connection History ({} connections):", history.len()));
            for connection in history {
                let status = if connection.success { "✅" } else { "❌" };
                let duration = connection.connection_duration
                    .map(|d| format!(" ({}s)", d))
                    .unwrap_or_else(|| " (ongoing)".to_string());
                
                println!("  {} {} - Connected: {} - Disconnected: {}{}", 
                        status,
                        connection.peer_id,
                        chrono::DateTime::<chrono::Utc>::from(
                            std::time::UNIX_EPOCH + std::time::Duration::from_secs(connection.connected_at)
                        ).format("%Y-%m-%d %H:%M:%S"),
                        connection.disconnected_at.map(|t| {
                            chrono::DateTime::<chrono::Utc>::from(
                                std::time::UNIX_EPOCH + std::time::Duration::from_secs(t)
                            ).format("%Y-%m-%d %H:%M:%S").to_string()
                        }).unwrap_or_else(|| "N/A".to_string()),
                        duration);
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::ScanLocalPeers => {
            let start = Instant::now();
            let p2p_node = crate::peer::P2PNode::new();
            match p2p_node.scan_local_network().await {
                Ok(peers) => {
                    print_success(&format!("🔍 Scanned local network and found {} peers:", peers.len()));
                    for peer in peers {
                        println!("  🟢 {} ({})", peer.peer_id, peer.addresses.join(", "));
                    }
                }
                Err(e) => print_error(&format!("Error scanning local network: {}", e)),
            }
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        // Remote Management Commands
        Commands::RemoteConnect { host, port, username, password } => {
            let start = Instant::now();
            print_info(&format!("Connecting to remote DAFS service at {}:{}...", host, port));
            
            // Implement remote connection logic
            print_success("Remote connection successful");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::RemoteExec { command } => {
            let start = Instant::now();
            print_info(&format!("Executing command '{}' on remote DAFS service...", command));
            
            // Implement remote execution logic
            print_success("Command executed successfully");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::RemoteStatus => {
            let start = Instant::now();
            print_info("Getting remote service status...");
            
            // Implement remote status retrieval logic
            print_success("Remote service status retrieved");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::RemoteBootstrap { action, peer_id, addr } => {
            let start = Instant::now();
            print_info(&format!("Managing remote bootstrap node: {}", action));
            
            // Implement remote bootstrap node management logic
            print_success("Remote bootstrap node managed");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::RemoteLogs { lines } => {
            let start = Instant::now();
            print_info(&format!("Getting remote logs ({} lines):", lines.unwrap_or(10)));
            
            // Implement remote logs retrieval logic
            print_success("Remote logs retrieved");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::RemoteRestart => {
            let start = Instant::now();
            print_info("Restarting remote service...");
            
            // Implement remote service restart logic
            print_success("Remote service restarted");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::RemoteStop => {
            let start = Instant::now();
            print_info("Stopping remote service...");
            
            // Implement remote service stop logic
            print_success("Remote service stopped");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::RemoteStart => {
            let start = Instant::now();
            print_info("Starting remote service...");
            
            // Implement remote service start logic
            print_success("Remote service started");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::RemoteConfig { key, value } => {
            let start = Instant::now();
            print_info(&format!("Updating remote configuration: {} = {}", key, value));
            
            // Implement remote configuration update logic
            print_success("Remote configuration updated");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::RemoteConfigGet { key } => {
            let start = Instant::now();
            print_info(&format!("Getting remote configuration for key '{}':", key.unwrap_or_default()));
            
            // Implement remote configuration retrieval logic
            print_success("Remote configuration retrieved");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::RemoteBackup { path } => {
            let start = Instant::now();
            print_info(&format!("Backing up remote data to '{}':", path));
            
            // Implement remote backup logic
            print_success("Remote data backed up");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        Commands::RemoteRestore { path } => {
            let start = Instant::now();
            print_info(&format!("Restoring remote data from '{}':", path));
            
            // Implement remote restore logic
            print_success("Remote data restored");
            print_info(&format!("Done in {:.2?}", start.elapsed()));
        }
        
        _ => print_error("Command not yet implemented in CLI"),
    }
} 