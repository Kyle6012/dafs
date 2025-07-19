use libp2p::{
    identity,
    swarm::{Swarm, SwarmEvent},
    PeerId,
    noise,
    core::upgrade,
    yamux,
    Transport,
    Multiaddr,
};
use libp2p::swarm::NetworkBehaviour;
use libp2p::futures::AsyncReadExt;
use libp2p::futures::AsyncWriteExt;
use anyhow::Result;
use futures::StreamExt;
use std::fs;
use tokio::sync::{mpsc, oneshot};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::storage::Storage;
use std::sync::Arc;
use once_cell::sync::Lazy;
use libp2p::kad::{self as kad, store::MemoryStore};
use libp2p::relay::Behaviour as RelayBehaviour;
use std::str::FromStr;
use serde_json;
use crate::ai::NCFModel as CFModel;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Mutex;
use libp2p::request_response::{Behaviour as RequestResponseBehaviour, Event as RequestResponseEvent, ProtocolSupport};
use async_trait::async_trait;
use chrono::Utc;
use libp2p::request_response::Codec;
use futures::io::{AsyncRead, AsyncWrite};

// Global storage for discovered peers
static DISCOVERED_PEERS: Lazy<Mutex<HashMap<String, DiscoveredPeer>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

// Global bootstrap nodes storage
static BOOTSTRAP_NODES: Lazy<Mutex<Vec<(PeerId, Multiaddr)>>> = Lazy::new(|| Mutex::new(Vec::new()));

// Global allowed peers storage
static ALLOWED_PEERS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2PMessage {
    FileKeyExchange { file_id: String, encrypted_key: Vec<u8>, from: String, to: String },
    FileChunkRequest { file_id: String, chunk_index: usize, chunk_size: usize, signature: Option<Vec<u8>> },
    FileChunkResponse { file_id: String, chunk_index: usize, data: Vec<u8> },
    FileListRequest { signature: Option<Vec<u8>> },
    FileListResponse { files: Vec<crate::storage::FileMetadata> },
    ModelUpdate { weights: Vec<u8>, epoch: u32 },
    
    // P2P Messaging
    EncryptedMessage { message: crate::models::EncryptedMessage },
    MessageAck { ack: crate::models::MessageAck },
    UserStatus { status: crate::models::UserStatus },
    ChatRoomCreate { room: crate::models::ChatRoom },
    ChatRoomJoin { room_id: String, username: String },
    ChatRoomLeave { room_id: String, username: String },
    ChatRoomMessage { room_id: String, message: crate::models::EncryptedMessage },
    TypingIndicator { room_id: String, username: String, is_typing: bool },
    
    // Enhanced Peer Discovery
    PeerDiscovery { peer_id: String, addresses: Vec<String>, user_info: Option<crate::models::UserIdentity> },
    PeerPing { timestamp: u64, peer_id: String },
    PeerPong { timestamp: u64, peer_id: String },
}

#[derive(Debug, Clone)]
pub struct FileExchangeProtocol();

impl AsRef<str> for FileExchangeProtocol {
    fn as_ref(&self) -> &str {
        "/dafs/file-exchange/1.0.0"
    }
}

#[derive(Debug, Clone)]
pub struct MessagingProtocol();

impl AsRef<str> for MessagingProtocol {
    fn as_ref(&self) -> &str {
        "/dafs/messaging/1.0.0"
    }
}

#[derive(Debug, Clone)]
pub struct PeerDiscoveryProtocol();

impl AsRef<str> for PeerDiscoveryProtocol {
    fn as_ref(&self) -> &str {
        "/dafs/peer-discovery/1.0.0"
    }
}

#[derive(Clone, Default)]
pub struct FileExchangeCodec;

#[async_trait]
impl Codec for FileExchangeCodec {
    type Protocol = FileExchangeProtocol;
    type Request = Vec<u8>;
    type Response = Vec<u8>;

    async fn read_request<T>(&mut self, _: &<Self as Codec>::Protocol, io: &mut T) -> std::io::Result<<Self as Codec>::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        Ok(buf)
    }

    async fn read_response<T>(&mut self, _: &<Self as Codec>::Protocol, io: &mut T) -> std::io::Result<<Self as Codec>::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        Ok(buf)
    }

    async fn write_request<T>(&mut self, _: &<Self as Codec>::Protocol, io: &mut T, req: <Self as Codec>::Request) -> std::io::Result<()> 
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&req).await?;
        io.close().await
    }

    async fn write_response<T>(&mut self, _: &<Self as Codec>::Protocol, io: &mut T, res: <Self as Codec>::Response) -> std::io::Result<()> 
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&res).await?;
        io.close().await
    }
}

#[derive(Clone, Default)]
pub struct MessagingCodec;

#[async_trait]
impl Codec for MessagingCodec {
    type Protocol = MessagingProtocol;
    type Request = Vec<u8>;
    type Response = Vec<u8>;

    async fn read_request<T>(&mut self, _: &<Self as Codec>::Protocol, io: &mut T) -> std::io::Result<<Self as Codec>::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        Ok(buf)
    }

    async fn read_response<T>(&mut self, _: &<Self as Codec>::Protocol, io: &mut T) -> std::io::Result<<Self as Codec>::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        Ok(buf)
    }

    async fn write_request<T>(&mut self, _: &<Self as Codec>::Protocol, io: &mut T, req: <Self as Codec>::Request) -> std::io::Result<()> 
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&req).await?;
        io.close().await
    }

    async fn write_response<T>(&mut self, _: &<Self as Codec>::Protocol, io: &mut T, res: <Self as Codec>::Response) -> std::io::Result<()> 
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&res).await?;
        io.close().await
    }
}

#[derive(Clone, Default)]
pub struct PeerDiscoveryCodec;

#[async_trait]
impl Codec for PeerDiscoveryCodec {
    type Protocol = PeerDiscoveryProtocol;
    type Request = Vec<u8>;
    type Response = Vec<u8>;

    async fn read_request<T>(&mut self, _: &<Self as Codec>::Protocol, io: &mut T) -> std::io::Result<<Self as Codec>::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        Ok(buf)
    }

    async fn read_response<T>(&mut self, _: &<Self as Codec>::Protocol, io: &mut T) -> std::io::Result<<Self as Codec>::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        Ok(buf)
    }

    async fn write_request<T>(&mut self, _: &<Self as Codec>::Protocol, io: &mut T, req: <Self as Codec>::Request) -> std::io::Result<()> 
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&req).await?;
        io.close().await
    }

    async fn write_response<T>(&mut self, _: &<Self as Codec>::Protocol, io: &mut T, res: <Self as Codec>::Response) -> std::io::Result<()> 
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&res).await?;
        io.close().await
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "MyBehaviourEvent")]
pub struct MyBehaviour {
    pub file_exchange: RequestResponseBehaviour<FileExchangeCodec>,
    pub messaging: RequestResponseBehaviour<MessagingCodec>,
    pub peer_discovery: RequestResponseBehaviour<PeerDiscoveryCodec>,
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub relay: RelayBehaviour,
}

#[derive(Debug)]
pub enum MyBehaviourEvent {
    FileExchange(RequestResponseEvent<Vec<u8>, Vec<u8>>),
    Messaging(RequestResponseEvent<Vec<u8>, Vec<u8>>),
    PeerDiscovery(RequestResponseEvent<Vec<u8>, Vec<u8>>),
    Kademlia(kad::Event),
    Relay(()),
}

impl From<RequestResponseEvent<Vec<u8>, Vec<u8>>> for MyBehaviourEvent {
    fn from(event: RequestResponseEvent<Vec<u8>, Vec<u8>>) -> Self {
        MyBehaviourEvent::FileExchange(event)
    }
}
// Repeat for Messaging and PeerDiscovery if needed
impl From<kad::Event> for MyBehaviourEvent {
    fn from(event: kad::Event) -> Self {
        MyBehaviourEvent::Kademlia(event)
    }
}
impl From<()> for MyBehaviourEvent {
    fn from(_: ()) -> Self {
        MyBehaviourEvent::Relay(())
    }
}
impl From<libp2p::relay::Event> for MyBehaviourEvent {
    fn from(_: libp2p::relay::Event) -> Self {
        MyBehaviourEvent::Relay(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    pub peer_id: String,
    pub addresses: Vec<String>,
    pub last_seen: u64,
    pub user_info: Option<crate::models::UserIdentity>,
    pub is_online: bool,
    pub latency_ms: Option<u64>,
}

pub enum P2PCommand {
    ListFiles { peer: PeerId, respond_to: oneshot::Sender<Vec<String>> },
    GetFile { peer: PeerId, file_id: String, respond_to: oneshot::Sender<Vec<u8>> },
    SendMessage { peer: PeerId, message: P2PMessage },
    RequestChunk { peer: PeerId, file_id: String, chunk_index: usize, chunk_size: usize, respond_to: oneshot::Sender<Vec<u8>> },
    
    // Messaging commands
    SendEncryptedMessage { peer: PeerId, message: crate::models::EncryptedMessage, respond_to: oneshot::Sender<bool> },
    CreateChatRoom { room: crate::models::ChatRoom, respond_to: oneshot::Sender<bool> },
    JoinChatRoom { room_id: String, username: String, respond_to: oneshot::Sender<bool> },
    SendChatMessage { room_id: String, message: crate::models::EncryptedMessage, respond_to: oneshot::Sender<bool> },
    UpdateUserStatus { status: crate::models::UserStatus },
    
    // Enhanced peer discovery commands
    ConnectToPeer { peer_id: String, addr: Option<String>, respond_to: oneshot::Sender<bool> },
    DiscoverPeers { respond_to: oneshot::Sender<Vec<DiscoveredPeer>> },
    PingPeer { peer_id: String, respond_to: oneshot::Sender<Option<u64>> },
    GetKnownPeers { respond_to: oneshot::Sender<Vec<DiscoveredPeer>> },
    RemovePeer { peer_id: String, respond_to: oneshot::Sender<bool> },
}

pub struct P2PNode {
    cmd_tx: mpsc::Sender<P2PCommand>,
}

impl P2PNode {
    pub fn new() -> Self {
        let (cmd_tx, mut cmd_rx) = mpsc::channel(32);
        
        // Load discovered peers on startup
        let _ = load_discovered_peers();
        
        // Spawn background task for event loop
        tokio::spawn(async move {
            let id_keys = identity::Keypair::generate_ed25519();
            let peer_id = PeerId::from(id_keys.public());
            println!("Local peer id: {:?}", peer_id);

            // Build transport manually
            let transport = libp2p::tcp::tokio::Transport::new(
                libp2p::tcp::Config::default().nodelay(true),
            )
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&id_keys).unwrap())
            .multiplex(yamux::Config::default())
            .boxed();

            let file_proto = FileExchangeProtocol();
            let file_codec = FileExchangeCodec;
            let mut file_cfg = libp2p::request_response::Config::default();
            file_cfg.set_request_timeout(std::time::Duration::from_secs(30));
            let file_protocols = std::iter::once((file_proto, ProtocolSupport::Full));
            let file_exchange = RequestResponseBehaviour::new(file_protocols, file_cfg);
            
            let msg_proto = MessagingProtocol();
            let msg_codec = MessagingCodec;
            let mut msg_cfg = libp2p::request_response::Config::default();
            msg_cfg.set_request_timeout(std::time::Duration::from_secs(10));
            let msg_protocols = std::iter::once((msg_proto, ProtocolSupport::Full));
            let messaging = RequestResponseBehaviour::new(msg_protocols, msg_cfg);
            
            let discovery_proto = PeerDiscoveryProtocol();
            let discovery_codec = PeerDiscoveryCodec;
            let mut discovery_cfg = libp2p::request_response::Config::default();
            discovery_cfg.set_request_timeout(std::time::Duration::from_secs(15));
            let discovery_protocols = std::iter::once((discovery_proto, ProtocolSupport::Full));
            let peer_discovery = RequestResponseBehaviour::new(discovery_protocols, discovery_cfg);
            
            let local_peer_id = peer_id;
            let store = MemoryStore::new(local_peer_id);
            let kademlia = kad::Behaviour::new(local_peer_id, store);
            let relay = RelayBehaviour::new(local_peer_id, libp2p::relay::Config::default());
            let mut behaviour = MyBehaviour {
                file_exchange,
                messaging,
                peer_discovery,
                kademlia,
                relay,
            };
            
            let mut swarm = Swarm::new(
                transport,
                behaviour,
                peer_id,
                libp2p::swarm::Config::with_executor(Box::new(|fut| { tokio::spawn(fut); })),
            );
            
            // Listen on multiple addresses for better connectivity
            swarm.listen_on("/ip4/0.0.0.0/tcp/2093".parse().unwrap()).unwrap();
            swarm.listen_on("/ip6/::/tcp/2093".parse().unwrap());
            
            // Add bootstrap nodes for Kademlia
            let _ = load_bootstrap_nodes();
            for (peer, addr) in BOOTSTRAP_NODES.lock().unwrap().iter() {
                swarm.behaviour_mut().kademlia.add_address(peer, addr.clone());
            }

            let mut pending_responses: HashMap<libp2p::request_response::RequestId, oneshot::Sender<Vec<u8>>> = HashMap::new();
            let mut pending_list: HashMap<libp2p::request_response::RequestId, oneshot::Sender<Vec<String>>> = HashMap::new();
            let mut pending_chunk_responses: HashMap<libp2p::request_response::RequestId, oneshot::Sender<Vec<u8>>> = HashMap::new();
            let mut pending_messaging_responses: HashMap<libp2p::request_response::RequestId, oneshot::Sender<bool>> = HashMap::new();
            let mut pending_discovery_responses: HashMap<libp2p::request_response::RequestId, oneshot::Sender<bool>> = HashMap::new();
            let mut pending_peer_responses: HashMap<libp2p::request_response::RequestId, oneshot::Sender<Vec<DiscoveredPeer>>> = HashMap::new();
            let mut pending_ping_responses: HashMap<libp2p::request_response::RequestId, oneshot::Sender<Option<u64>>> = HashMap::new();
            let mut known_peers: Vec<PeerId> = Vec::new();

            loop {
                tokio::select! {
                    Some(cmd) = cmd_rx.recv() => {
                        match cmd {
                            P2PCommand::ListFiles { peer, respond_to } => {
                                let req = b"LIST".to_vec();
                                let req_id = swarm.behaviour_mut().file_exchange.send_request(&peer, req);
                                pending_list.insert(req_id, respond_to);
                            }
                            P2PCommand::GetFile { peer, file_id, respond_to } => {
                                let req = file_id.into_bytes();
                                let req_id = swarm.behaviour_mut().file_exchange.send_request(&peer, req);
                                pending_responses.insert(req_id, respond_to);
                            }
                            P2PCommand::SendMessage { peer, message } => {
                                let data = bincode::serialize(&message).unwrap();
                                swarm.behaviour_mut().file_exchange.send_request(&peer, data);
                            }
                            P2PCommand::RequestChunk { peer, file_id, chunk_index, chunk_size, respond_to } => {
                                let msg = P2PMessage::FileChunkRequest { file_id: file_id.clone(), chunk_index, chunk_size, signature: None };
                                let data = bincode::serialize(&msg).unwrap();
                                let req_id = swarm.behaviour_mut().file_exchange.send_request(&peer, data);
                                pending_chunk_responses.insert(req_id, respond_to);
                            }
                            P2PCommand::SendEncryptedMessage { peer, message, respond_to } => {
                                let msg = P2PMessage::EncryptedMessage { message };
                                let data = bincode::serialize(&msg).unwrap();
                                let req_id = swarm.behaviour_mut().messaging.send_request(&peer, data);
                                pending_messaging_responses.insert(req_id, respond_to);
                            }
                            P2PCommand::CreateChatRoom { room, respond_to } => {
                                let msg = P2PMessage::ChatRoomCreate { room };
                                let data = bincode::serialize(&msg).unwrap();
                                // Broadcast to all known peers
                                for peer in &known_peers {
                                    swarm.behaviour_mut().messaging.send_request(peer, data.clone());
                                }
                                let _ = respond_to.send(true);
                            }
                            P2PCommand::JoinChatRoom { room_id, username, respond_to } => {
                                let msg = P2PMessage::ChatRoomJoin { room_id, username };
                                let data = bincode::serialize(&msg).unwrap();
                                // Broadcast to all known peers
                                for peer in &known_peers {
                                    swarm.behaviour_mut().messaging.send_request(peer, data.clone());
                                }
                                let _ = respond_to.send(true);
                            }
                            P2PCommand::SendChatMessage { room_id, message, respond_to } => {
                                let msg = P2PMessage::ChatRoomMessage { room_id, message };
                                let data = bincode::serialize(&msg).unwrap();
                                // Broadcast to all known peers
                                for peer in &known_peers {
                                    swarm.behaviour_mut().messaging.send_request(peer, data.clone());
                                }
                                let _ = respond_to.send(true);
                            }
                            P2PCommand::UpdateUserStatus { status } => {
                                let msg = P2PMessage::UserStatus { status };
                                let data = bincode::serialize(&msg).unwrap();
                                // Broadcast to all known peers
                                for peer in &known_peers {
                                    swarm.behaviour_mut().messaging.send_request(peer, data.clone());
                                }
                            }
                            P2PCommand::ConnectToPeer { peer_id, addr, respond_to } => {
                                match PeerId::from_str(&peer_id) {
                                    Ok(peer) => {
                                        if let Some(addr_str) = addr {
                                            if let Ok(multiaddr) = Multiaddr::from_str(&addr_str) {
                                                swarm.behaviour_mut().kademlia.add_address(&peer, multiaddr);
                                                known_peers.push(peer);
                                                let _ = respond_to.send(true);
                                            } else {
                                                let _ = respond_to.send(false);
                                            }
                                        } else {
                                            // Try to discover peer via Kademlia
                                            swarm.behaviour_mut().kademlia.start_providing(kad::record::Key::from(peer.to_bytes()));
                                            known_peers.push(peer);
                                            let _ = respond_to.send(true);
                                        }
                                    }
                                    Err(_) => {
                                        let _ = respond_to.send(false);
                                    }
                                }
                            }
                            P2PCommand::DiscoverPeers { respond_to } => {
                                // Start Kademlia discovery
                                swarm.behaviour_mut().kademlia.start_providing(kad::record::Key::from(peer_id.to_bytes()));
                                
                                // Get known peers from storage
                                let peers = DISCOVERED_PEERS.lock().unwrap().values().cloned().collect();
                                let _ = respond_to.send(peers);
                            }
                            P2PCommand::PingPeer { peer_id, respond_to } => {
                                match PeerId::from_str(&peer_id) {
                                    Ok(peer) => {
                                        let ping_msg = P2PMessage::PeerPing {
                                            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                                            peer_id: peer_id.clone(),
                                        };
                                        let data = bincode::serialize(&ping_msg).unwrap();
                                        let req_id = swarm.behaviour_mut().peer_discovery.send_request(&peer, data);
                                        pending_ping_responses.insert(req_id, respond_to);
                                    }
                                    Err(_) => {
                                        let _ = respond_to.send(None);
                                    }
                                }
                            }
                            P2PCommand::GetKnownPeers { respond_to } => {
                                let peers = DISCOVERED_PEERS.lock().unwrap().values().cloned().collect();
                                let _ = respond_to.send(peers);
                            }
                            P2PCommand::RemovePeer { peer_id, respond_to } => {
                                let mut peers = DISCOVERED_PEERS.lock().unwrap();
                                let removed = peers.remove(&peer_id).is_some();
                                if removed {
                                    save_discovered_peers().ok();
                                }
                                let _ = respond_to.send(removed);
                            }
                        }
                    }
                    event = swarm.next() => {
                        match event {
                            Some(SwarmEvent::NewListenAddr { address, .. }) => {
                                println!("Listening on {}", address);
                            }
                            Some(SwarmEvent::Behaviour(MyBehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed { result, .. }))) => {
                                match result {
                                    libp2p::kad::QueryResult::Bootstrap(_) => {
                                        println!("Kademlia bootstrap completed");
                                    }
                                    libp2p::kad::QueryResult::GetProviders(Ok(ok)) => {
                                        // TODO: Fix providers field for GetProvidersOk in current libp2p version
                                        // for peer in ok.providers {
                                        //     println!("Kademlia discovered provider: {}", peer);
                                        //     known_peers.push(peer);
                                        // }
                                        save_discovered_peers().ok();
                                    }
                                    libp2p::kad::QueryResult::GetProviders(Err(_)) => {
                                        println!("Kademlia get providers failed");
                                    }
                                    libp2p::kad::QueryResult::GetRecord(Ok(ok)) => {
                                        // TODO: Fix record field for GetRecordOk in current libp2p version
                                        // if let Some(record) = ok.record.iter().next() {
                                        //     println!("Kademlia get record: {:?}", record);
                                        // }
                                    }
                                    libp2p::kad::QueryResult::GetRecord(Err(_)) => {
                                        println!("Kademlia get record failed");
                                    }
                                    libp2p::kad::QueryResult::PutRecord(Ok(_)) => {
                                        println!("Kademlia put record completed");
                                    }
                                    libp2p::kad::QueryResult::PutRecord(Err(_)) => {
                                        println!("Kademlia put record failed");
                                    }
                                    libp2p::kad::QueryResult::StartProviding(Ok(_)) => {
                                        println!("Kademlia start providing completed");
                                    }
                                    libp2p::kad::QueryResult::StartProviding(Err(_)) => {
                                        println!("Kademlia start providing failed");
                                    }
                                    _ => {}
                                }
                            }
                            Some(SwarmEvent::Behaviour(MyBehaviourEvent::FileExchange(libp2p::request_response::Event::Message { peer, message }))) => {
                                match message {
                                    libp2p::request_response::Message::Request { request_id, request, channel } => {
                                        let response = handle_file_request(&request);
                                        swarm.behaviour_mut().file_exchange.send_response(channel, response).unwrap();
                                    }
                                    libp2p::request_response::Message::Response { request_id, response } => {
                                        if let Some(respond_to) = pending_responses.remove(&request_id) {
                                            let _ = respond_to.send(response);
                                        } else if let Some(respond_to) = pending_list.remove(&request_id) {
                                            let files = String::from_utf8_lossy(&response);
                                            let file_list: Vec<String> = files.lines().map(|s| s.to_string()).collect();
                                            let _ = respond_to.send(file_list);
                                        } else if let Some(respond_to) = pending_chunk_responses.remove(&request_id) {
                                            let _ = respond_to.send(response);
                                        }
                                    }
                                }
                            }
                            Some(SwarmEvent::Behaviour(MyBehaviourEvent::Messaging(libp2p::request_response::Event::Message { peer, message }))) => {
                                match message {
                                    libp2p::request_response::Message::Request { request_id, request, channel } => {
                                        let response = handle_messaging_request(&request);
                                        swarm.behaviour_mut().messaging.send_response(channel, response).unwrap();
                                    }
                                    libp2p::request_response::Message::Response { request_id, response } => {
                                        if let Some(respond_to) = pending_messaging_responses.remove(&request_id) {
                                            let success = response.len() > 0;
                                            let _ = respond_to.send(success);
                                        }
                                    }
                                }
                            }
                            Some(SwarmEvent::Behaviour(MyBehaviourEvent::PeerDiscovery(libp2p::request_response::Event::Message { peer, message }))) => {
                                match message {
                                    libp2p::request_response::Message::Request { request_id, request, channel } => {
                                        let response = handle_discovery_request(&request);
                                        swarm.behaviour_mut().peer_discovery.send_response(channel, response).unwrap();
                                    }
                                    libp2p::request_response::Message::Response { request_id, response } => {
                                        if let Some(respond_to) = pending_discovery_responses.remove(&request_id) {
                                            let success = response.len() > 0;
                                            let _ = respond_to.send(success);
                                        } else if let Some(respond_to) = pending_ping_responses.remove(&request_id) {
                                            if let Ok(pong_msg) = bincode::deserialize::<P2PMessage>(&response) {
                                                if let P2PMessage::PeerPong { timestamp, .. } = pong_msg {
                                                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                                                    let latency = if now > timestamp { now - timestamp } else { 0 };
                                                    let _ = respond_to.send(Some(latency));
                                                } else {
                                                    let _ = respond_to.send(None);
                                                }
                                            } else {
                                                let _ = respond_to.send(None);
                                            }
                                        }
                                    }
                                }
                            }
                            Some(SwarmEvent::Behaviour(MyBehaviourEvent::Relay(event))) => {
                                // Handle relay events
                            }
                            Some(SwarmEvent::ConnectionEstablished { peer_id, .. }) => {
                                println!("Connected to peer: {}", peer_id);
                                known_peers.push(peer_id);
                                
                                // Update peer status
                                if let Some(peer) = DISCOVERED_PEERS.lock().unwrap().get_mut(&peer_id.to_string()) {
                                    peer.is_online = true;
                                    peer.last_seen = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                                    save_discovered_peers().ok();
                                }
                            }
                            Some(SwarmEvent::ConnectionClosed { peer_id, .. }) => {
                                println!("Disconnected from peer: {}", peer_id);
                                known_peers.retain(|p| p != &peer_id);
                                
                                // Update peer status
                                if let Some(peer) = DISCOVERED_PEERS.lock().unwrap().get_mut(&peer_id.to_string()) {
                                    peer.is_online = false;
                                    save_discovered_peers().ok();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        Self { cmd_tx }
    }
    pub async fn list_files(&self, peer: PeerId) -> Vec<String> {
        let (tx, rx) = oneshot::channel::<Vec<String>>();
        let _ = self.cmd_tx.send(P2PCommand::ListFiles { peer, respond_to: tx }).await;
        rx.await.unwrap_or_default()
    }
    pub async fn get_file(&self, peer: PeerId, file_id: String) -> Vec<u8> {
        let (tx, rx) = oneshot::channel::<Vec<u8>>();
        let _ = self.cmd_tx.send(P2PCommand::GetFile { peer, file_id, respond_to: tx }).await;
        rx.await.unwrap_or_default()
    }
    pub async fn send_message(&self, peer: PeerId, message: P2PMessage) {
        let _ = self.cmd_tx.send(P2PCommand::SendMessage { peer, message }).await;
    }
    pub async fn request_chunk(&self, peer_id: &str, file_id: &str, chunk_index: usize, chunk_size: usize) -> anyhow::Result<Vec<u8>> {
        let peer = PeerId::from_str(peer_id)?;
        let (tx, rx) = oneshot::channel::<Vec<u8>>();
        let _ = self.cmd_tx.send(P2PCommand::RequestChunk {
            peer,
            file_id: file_id.to_string(),
            chunk_index,
            chunk_size,
            respond_to: tx,
        }).await;
        Ok(rx.await.unwrap_or_default())
    }
    pub async fn send_model_update(&self, peer_id: &str, model: &CFModel) -> anyhow::Result<()> {
        let peer = PeerId::from_str(peer_id)?;
        let weights = bincode::serialize(model)?;
        let msg = P2PMessage::ModelUpdate { weights, epoch: model.epoch };
        self.send_message(peer, msg).await;
        Ok(())
    }
    pub async fn query_peer_files(&self, peer_id: &str) -> anyhow::Result<Vec<crate::storage::FileMetadata>> {
        use libp2p::PeerId;
        let peer = PeerId::from_str(peer_id)?;
        let (tx, rx) = oneshot::channel::<Vec<u8>>();
        let _ = self.cmd_tx.send(P2PCommand::SendMessage {
            peer: peer.clone(),
            message: P2PMessage::FileListRequest { signature: None },
        }).await;
        // Wait for response (simulate for now)
        // In a real implementation, you would have a response channel or event handler
        // For now, return local files as a stub
        let files = crate::storage::Storage::new("dafs_db")?.list_metadata()?;
        Ok(files)
    }

    // P2P Messaging Methods
    pub async fn send_encrypted_message(&self, peer_id: &str, message: crate::models::EncryptedMessage) -> anyhow::Result<bool> {
        use libp2p::PeerId;
        let peer = PeerId::from_str(peer_id)?;
        let (tx, rx) = oneshot::channel::<bool>();
        let _ = self.cmd_tx.send(P2PCommand::SendEncryptedMessage { peer, message, respond_to: tx }).await;
        Ok(rx.await.unwrap_or(false))
    }

    pub async fn create_chat_room(&self, room: crate::models::ChatRoom) -> anyhow::Result<bool> {
        let (tx, rx) = oneshot::channel::<bool>();
        let _ = self.cmd_tx.send(P2PCommand::CreateChatRoom { room, respond_to: tx }).await;
        Ok(rx.await.unwrap_or(false))
    }

    pub async fn join_chat_room(&self, room_id: String, username: String) -> anyhow::Result<bool> {
        let (tx, rx) = oneshot::channel::<bool>();
        let _ = self.cmd_tx.send(P2PCommand::JoinChatRoom { room_id, username, respond_to: tx }).await;
        Ok(rx.await.unwrap_or(false))
    }

    pub async fn send_chat_message(&self, room_id: String, message: crate::models::EncryptedMessage) -> anyhow::Result<bool> {
        let (tx, rx) = oneshot::channel::<bool>();
        let _ = self.cmd_tx.send(P2PCommand::SendChatMessage { room_id, message, respond_to: tx }).await;
        Ok(rx.await.unwrap_or(false))
    }

    pub async fn update_user_status(&self, status: crate::models::UserStatus) -> anyhow::Result<()> {
        let _ = self.cmd_tx.send(P2PCommand::UpdateUserStatus { status }).await;
        Ok(())
    }

    // Enhanced peer discovery methods
    pub async fn connect_to_peer(&self, peer_id: &str, addr: Option<String>) -> anyhow::Result<bool> {
        // Simulate connection success for now
        // In a real implementation, this would attempt to establish a connection
        let success = true; // Mock successful connection
        Ok(success)
    }

    pub async fn discover_peers(&self) -> anyhow::Result<Vec<DiscoveredPeer>> {
        // For now, return some mock discovered peers
        // In a real implementation, this would use mDNS, DHT, or other discovery mechanisms
        let mock_peers = vec![
            DiscoveredPeer {
                peer_id: "12D3KooWExample1".to_string(),
                addresses: vec!["/ip4/192.168.1.100/tcp/2093".to_string()],
                last_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                user_info: None,
                is_online: true,
                latency_ms: Some(15),
            },
            DiscoveredPeer {
                peer_id: "12D3KooWExample2".to_string(),
                addresses: vec!["/ip4/192.168.1.101/tcp/2093".to_string()],
                last_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                user_info: None,
                is_online: true,
                latency_ms: Some(25),
            },
        ];
        
        Ok(mock_peers)
    }

    pub async fn ping_peer(&self, peer_id: &str) -> anyhow::Result<Option<u64>> {
        let (tx, rx) = oneshot::channel::<Option<u64>>();
        let _ = self.cmd_tx.send(P2PCommand::PingPeer {
            peer_id: peer_id.to_string(),
            respond_to: tx,
        }).await;
        rx.await.map_err(|e| anyhow::anyhow!("Failed to ping peer: {}", e))
    }

    pub async fn get_known_peers(&self) -> anyhow::Result<Vec<DiscoveredPeer>> {
        // Return discovered peers from global storage
        let peers = DISCOVERED_PEERS.lock().unwrap().values().cloned().collect();
        Ok(peers)
    }

    pub async fn remove_peer(&self, peer_id: &str) -> anyhow::Result<bool> {
        // Remove from discovered peers
        let removed = DISCOVERED_PEERS.lock().unwrap().remove(peer_id).is_some();
        Ok(removed)
    }

    // Additional convenience methods
    pub fn list_known_peers(&self) -> Vec<DiscoveredPeer> {
        // This is a blocking call for convenience
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            self.get_known_peers().await.unwrap_or_default()
        })
    }

    pub fn get_peer_connection_history(&self) -> Vec<(String, PeerConnectionInfo)> {
        // Return mock connection history
        // In a real implementation, this would be persisted and retrieved from storage
        vec![
            ("12D3KooWExample1".to_string(), PeerConnectionInfo {
                connected: true,
                last_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 3600, // 1 hour ago
            }),
            ("12D3KooWExample2".to_string(), PeerConnectionInfo {
                connected: false,
                last_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 7200, // 2 hours ago
            }),
        ]
    }

    pub async fn scan_local_network(&self) -> anyhow::Result<Vec<DiscoveredPeer>> {
        // Simulate local network scan
        // In a real implementation, this would use mDNS or similar local discovery
        let local_peers = vec![
            DiscoveredPeer {
                peer_id: "12D3KooWLocal1".to_string(),
                addresses: vec!["/ip4/192.168.1.50/tcp/2093".to_string()],
                last_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                user_info: None,
                is_online: true,
                latency_ms: Some(5),
            },
            DiscoveredPeer {
                peer_id: "12D3KooWLocal2".to_string(),
                addresses: vec!["/ip4/192.168.1.51/tcp/2093".to_string()],
                last_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                user_info: None,
                is_online: true,
                latency_ms: Some(8),
            },
        ];
        
        // Add to discovered peers
        for peer in &local_peers {
            DISCOVERED_PEERS.lock().unwrap().insert(peer.peer_id.clone(), peer.clone());
        }
        
        Ok(local_peers)
    }

    pub async fn connect_peer(&self, peer_id: String, addr: Option<String>) -> anyhow::Result<bool> {
        self.connect_to_peer(&peer_id, addr).await
    }
}

#[derive(Debug, Clone)]
pub struct PeerConnectionInfo {
    pub connected: bool,
    pub last_seen: u64,
}

pub static P2P_STORAGE: Lazy<Arc<Storage>> = Lazy::new(|| Arc::new(Storage::new("dafs_db").unwrap()));

pub fn encrypt_file_key_for_peer(file_key: &[u8; 32], recipient_pub: &x25519_dalek::PublicKey) -> Vec<u8> {
    let ephemeral = x25519_dalek::EphemeralSecret::random_from_rng(rand::thread_rng());
    let shared = ephemeral.diffie_hellman(recipient_pub);
    let mut encrypted = file_key.clone();
    for (b, k) in encrypted.iter_mut().zip(shared.as_bytes()) {
        *b ^= k;
    }
    encrypted.to_vec()
}

// Stub: send encrypted file key to recipient peer
pub async fn send_encrypted_file_key_to_peer(_peer: &PeerId, _encrypted_key: Vec<u8>) {
    // TODO: Implement P2P message sending logic
}

pub async fn send_file_key_exchange(peer: &PeerId, file_id: &str, encrypted_key: Vec<u8>, from: &str, to: &str, swarm: &mut Swarm<MyBehaviour>) {
    let msg = P2PMessage::FileKeyExchange {
        file_id: file_id.to_string(),
        encrypted_key,
        from: from.to_string(),
        to: to.to_string(),
    };
    let data = bincode::serialize(&msg).unwrap();
    swarm.behaviour_mut().file_exchange.send_request(peer, data);
}

fn handle_file_request(request: &[u8]) -> Vec<u8> {
    if let Ok(msg) = bincode::deserialize::<P2PMessage>(request) {
        match msg {
            P2PMessage::FileKeyExchange { file_id, encrypted_key, from, to } => {
                println!("Received file key for file {} from {} to {}", file_id, from, to);
                // Integrate with storage to update shared_keys for the recipient
                use uuid::Uuid;
                let file_uuid = Uuid::parse_str(&file_id).ok();
                if let Some(file_uuid) = file_uuid {
                    if let Ok(Some(mut meta)) = P2P_STORAGE.get_metadata(&file_uuid) {
                        meta.shared_keys.insert(to, encrypted_key);
                        let _ = P2P_STORAGE.insert_metadata(&meta);
                    }
                }
            }
            P2PMessage::FileChunkRequest { file_id, chunk_index, chunk_size, signature } => {
                use std::fs::OpenOptions;
                use std::io::{Seek, SeekFrom, Read};
                let file_path = format!("files/{}.bin", file_id);
                let mut file = match OpenOptions::new().read(true).open(&file_path) {
                    Ok(f) => f,
                    Err(_) => {
                        // Send empty response
                        let resp = P2PMessage::FileChunkResponse { file_id, chunk_index, data: vec![] };
                        return bincode::serialize(&resp).unwrap();
                    }
                };
                let offset = chunk_index * chunk_size;
                if file.seek(SeekFrom::Start(offset as u64)).is_err() {
                    // Send empty response
                    let resp = P2PMessage::FileChunkResponse { file_id, chunk_index, data: vec![] };
                    return bincode::serialize(&resp).unwrap();
                }
                let mut buf = vec![0u8; chunk_size];
                let n = file.read(&mut buf).unwrap_or(0);
                buf.truncate(n);
                let resp = P2PMessage::FileChunkResponse { file_id, chunk_index, data: buf };
                return bincode::serialize(&resp).unwrap();
            }
            P2PMessage::ModelUpdate { weights, epoch } => {
                if let Ok(remote_model) = bincode::deserialize::<CFModel>(&weights) {
                    // Aggregate with local model (assume ai::LOCAL_MODEL is a static Mutex<CFModel>)
                    let mut local = crate::ai::LOCAL_MODEL.lock().unwrap();
                    local.aggregate(&remote_model);
                    println!("Aggregated model update from peer (epoch {})", epoch);
                }
            }
            // ... handle other message types
            _ => {}
        }
    }
    vec![]
}

// In the event loop, handle incoming FileChunkResponse messages
// ... inside RequestResponseMessage::Response ...

pub fn add_bootstrap_node(peer_id: &str, addr: &str) -> anyhow::Result<()> {
    let peer = PeerId::from_str(peer_id)?;
    let addr = Multiaddr::from_str(addr)?;
    BOOTSTRAP_NODES.lock().unwrap().push((peer.clone(), addr.clone()));
    // Optionally persist to config
    save_bootstrap_nodes()?;
    Ok(())
}

pub fn remove_bootstrap_node(peer_id: &str) -> anyhow::Result<()> {
    let peer = PeerId::from_str(peer_id)?;
    BOOTSTRAP_NODES.lock().unwrap().retain(|(p, _)| p != &peer);
    save_bootstrap_nodes()?;
    Ok(())
}

pub fn list_bootstrap_nodes() -> Vec<(String, String)> {
    BOOTSTRAP_NODES.lock().unwrap().iter().map(|(p, a)| (p.to_string(), a.to_string())).collect()
}

pub fn save_bootstrap_nodes() -> anyhow::Result<()> {
    let nodes: Vec<(String, String)> = list_bootstrap_nodes();
    let json = serde_json::to_string_pretty(&nodes)?;
    fs::write("bootstrap_nodes.json", json)?;
    Ok(())
}

pub fn load_bootstrap_nodes() -> anyhow::Result<()> {
    if let Ok(data) = fs::read_to_string("bootstrap_nodes.json") {
        let nodes: Vec<(String, String)> = serde_json::from_str(&data)?;
        let mut lock = BOOTSTRAP_NODES.lock().unwrap();
        lock.clear();
        for (peer, addr) in nodes {
            lock.push((PeerId::from_str(&peer)?, Multiaddr::from_str(&addr)?));
        }
    }
    Ok(())
}

pub fn allow_peer(peer_id: &str) {
    ALLOWED_PEERS.lock().unwrap().push(peer_id.to_string());
    save_allowed_peers().ok();
}

pub fn disallow_peer(peer_id: &str) {
    ALLOWED_PEERS.lock().unwrap().retain(|p| p != peer_id);
    save_allowed_peers().ok();
}

pub fn list_allowed_peers() -> Vec<String> {
    ALLOWED_PEERS.lock().unwrap().iter().cloned().collect()
}

pub fn save_allowed_peers() -> anyhow::Result<()> {
    let peers: Vec<String> = list_allowed_peers();
    let json = serde_json::to_string_pretty(&peers)?;
    std::fs::write("allowed_peers.json", json)?;
    Ok(())
}

pub fn load_allowed_peers() -> anyhow::Result<()> {
    if let Ok(data) = std::fs::read_to_string("allowed_peers.json") {
        let peers: Vec<String> = serde_json::from_str(&data)?;
        let mut lock = ALLOWED_PEERS.lock().unwrap();
        lock.clear();
        for peer in peers {
            lock.push(peer);
        }
    }
    Ok(())
}

fn verify_signature(_peer_id: &str, _data: &[u8], _signature: &[u8]) -> bool {
    // TODO: Implement real signature verification using crypto module
    true // Accept all for now
}

// Messaging helper functions
fn save_message(message: &crate::models::EncryptedMessage) {
    let messages_dir = "messages";
    if let Err(_) = std::fs::create_dir_all(messages_dir) {
        eprintln!("Failed to create messages directory");
        return;
    }
    
    let filename = format!("{}/{}.json", messages_dir, message.id);
    if let Ok(data) = serde_json::to_string_pretty(message) {
        let _ = std::fs::write(filename, data);
    }
}

fn save_chat_room(room: &crate::models::ChatRoom) {
    let rooms_dir = "chat_rooms";
    if let Err(_) = std::fs::create_dir_all(rooms_dir) {
        eprintln!("Failed to create chat rooms directory");
        return;
    }
    
    let filename = format!("{}/{}.json", rooms_dir, room.id);
    if let Ok(data) = serde_json::to_string_pretty(room) {
        let _ = std::fs::write(filename, data);
    }
}

fn save_chat_message(room_id: &str, message: &crate::models::EncryptedMessage) {
    let messages_dir = format!("chat_rooms/{}/messages", room_id);
    if let Err(_) = std::fs::create_dir_all(&messages_dir) {
        eprintln!("Failed to create chat messages directory");
        return;
    }
    
    let filename = format!("{}/{}.json", messages_dir, message.id);
    if let Ok(data) = serde_json::to_string_pretty(message) {
        let _ = std::fs::write(filename, data);
    }
}

fn save_user_status(status: &crate::models::UserStatus) {
    let status_dir = "user_status";
    if let Err(_) = std::fs::create_dir_all(status_dir) {
        eprintln!("Failed to create user status directory");
        return;
    }
    
    let filename = format!("{}/{}.json", status_dir, status.username);
    if let Ok(data) = serde_json::to_string_pretty(status) {
        let _ = std::fs::write(filename, data);
    }
}

fn handle_messaging_request(request: &[u8]) -> Vec<u8> {
    match bincode::deserialize::<P2PMessage>(request) {
        Ok(msg) => {
            match msg {
                P2PMessage::EncryptedMessage { message } => {
                    println!("📨 Received encrypted message from {}: {:?}", message.sender_id, message.message_type);
                    // Store message locally
                    save_message(&message);
                    // Send acknowledgment
                    let ack = crate::models::MessageAck {
                        message_id: String::new(),
                        recipient_device_id: String::new(),
                        timestamp: 0,
                        delivered: true,
                    };
                    let ack_msg = P2PMessage::MessageAck { ack };
                    return bincode::serialize(&ack_msg).unwrap();
                }
                P2PMessage::ChatRoomCreate { room } => {
                    println!("🏠 Chat room created: {}", room.name);
                    save_chat_room(&room);
                    return b"OK".to_vec();
                }
                P2PMessage::ChatRoomJoin { room_id, username } => {
                    println!("👤 {} joined chat room: {}", username, room_id);
                    return b"OK".to_vec();
                }
                P2PMessage::ChatRoomMessage { room_id, message } => {
                    println!("💬 Chat message in room {} from {}: {:?}", room_id, message.sender_id, message.message_type);
                    save_chat_message(&room_id, &message);
                    return b"OK".to_vec();
                }
                P2PMessage::UserStatus { status } => {
                    println!("👤 User status update: {} is {}", status.username, if status.online { "online" } else { "offline" });
                    save_user_status(&status);
                    return b"OK".to_vec();
                }
                _ => {
                    println!("📨 Received other message type: {:?}", msg);
                    return b"OK".to_vec();
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to deserialize message: {}", e);
            return b"ERROR".to_vec();
        }
    }
}

fn handle_discovery_request(request: &[u8]) -> Vec<u8> {
    if let Ok(msg) = bincode::deserialize::<P2PMessage>(request) {
        match msg {
            P2PMessage::PeerPing { timestamp, peer_id } => {
                // Respond with pong
                let pong_msg = P2PMessage::PeerPong {
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    peer_id,
                };
                bincode::serialize(&pong_msg).unwrap_or_default()
            }
            P2PMessage::PeerDiscovery { peer_id, addresses, user_info } => {
                // Store discovered peer information
                let discovered_peer = DiscoveredPeer {
                    peer_id,
                    addresses,
                    last_seen: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    user_info,
                    is_online: true,
                    latency_ms: None,
                };
                
                DISCOVERED_PEERS.lock().unwrap().insert(discovered_peer.peer_id.clone(), discovered_peer);
                save_discovered_peers().ok();
                
                // Respond with our own peer info
                let response = P2PMessage::PeerDiscovery {
                    peer_id: "local".to_string(),
                    addresses: vec!["/ip4/0.0.0.0/tcp/2093".to_string()],
                    user_info: None,
                };
                bincode::serialize(&response).unwrap_or_default()
            }
            _ => Vec::new(),
        }
    } else {
        Vec::new()
    }
}

pub fn load_discovered_peers() -> anyhow::Result<()> {
    if let Ok(data) = fs::read_to_string("discovered_peers.json") {
        if let Ok(peers) = serde_json::from_str::<HashMap<String, DiscoveredPeer>>(&data) {
            let mut lock = DISCOVERED_PEERS.lock().unwrap();
            *lock = peers;
        }
    }
    Ok(())
}

pub fn save_discovered_peers() -> anyhow::Result<()> {
    let peers = DISCOVERED_PEERS.lock().unwrap();
    let json = serde_json::to_string_pretty(&*peers)?;
    fs::write("discovered_peers.json", json)?;
    Ok(())
}
