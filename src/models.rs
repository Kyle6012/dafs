pub use crate::storage::FileMetadata; 
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub public_key: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentity {
    pub user_id: String,           // Unique UUID for the user
    pub username: String,          // Display username (can be changed)
    pub display_name: String,      // Full name or display name
    pub email: Option<String>,     // Optional email for recovery
    pub created_at: u64,           // Account creation timestamp
    pub last_seen: u64,            // Last activity timestamp
    pub devices: Vec<UserDevice>,  // List of user's devices
    pub public_key: [u8; 32],      // User's public key
    pub is_active: bool,           // Account status
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDevice {
    pub device_id: String,         // Unique device identifier
    pub device_name: String,       // Human-readable device name
    pub device_type: DeviceType,   // Type of device
    pub last_login: u64,           // Last login timestamp
    pub is_current: bool,          // Is this the current device
    pub ip_address: Option<String>, // Last known IP
    pub user_agent: Option<String>, // Browser/client info
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    Desktop,
    Laptop,
    Mobile,
    Tablet,
    Server,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub id: String,
    pub sender_id: String,         // Use user_id instead of username
    pub recipient_id: String,      // Use user_id instead of username
    pub encrypted_content: Vec<u8>,
    pub timestamp: u64,
    pub message_type: MessageType,
    pub signature: Option<Vec<u8>>,
    pub device_id: String,         // Which device sent the message
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    File,
    Image,
    Voice,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRoom {
    pub id: String,
    pub name: String,
    pub participants: Vec<String>, // Use user_ids instead of usernames
    pub created_at: u64,
    pub last_message_at: u64,
    pub created_by: String,        // user_id of creator
    pub is_private: bool,          // Private vs public room
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatus {
    pub user_id: String,           // Use user_id instead of username
    pub username: String,          // Keep for display purposes
    pub online: bool,
    pub last_seen: u64,
    pub status_message: Option<String>,
    pub current_device: Option<String>, // Current device ID
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAck {
    pub message_id: String,
    pub delivered: bool,
    pub timestamp: u64,
    pub recipient_device_id: String, // Which device received it
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub session_id: String,
    pub user_id: String,
    pub device_id: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub is_active: bool,
}

impl UserIdentity {
    pub fn new(username: String, display_name: String, email: Option<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            user_id: Uuid::new_v4().to_string(),
            username,
            display_name,
            email,
            created_at: now,
            last_seen: now,
            devices: Vec::new(),
            public_key: [0u8; 32], // Will be set during registration
            is_active: true,
        }
    }

    pub fn add_device(&mut self, device: UserDevice) {
        // Mark all other devices as not current
        for dev in &mut self.devices {
            dev.is_current = false;
        }
        self.devices.push(device);
        self.last_seen = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    pub fn remove_device(&mut self, device_id: &str) {
        self.devices.retain(|d| d.device_id != device_id);
    }

    pub fn get_current_device(&self) -> Option<&UserDevice> {
        self.devices.iter().find(|d| d.is_current)
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

impl UserDevice {
    pub fn new(device_name: String, device_type: DeviceType) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            device_id: Uuid::new_v4().to_string(),
            device_name,
            device_type,
            last_login: now,
            is_current: true,
            ip_address: None,
            user_agent: None,
        }
    }

    pub fn update_login(&mut self) {
        self.last_login = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

impl EncryptedMessage {
    pub fn new(sender_id: String, recipient_id: String, content: Vec<u8>, message_type: MessageType, device_id: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            id: format!("msg_{}", timestamp),
            sender_id,
            recipient_id,
            encrypted_content: content,
            timestamp,
            message_type,
            signature: None,
            device_id,
        }
    }
}

impl ChatRoom {
    pub fn new(name: String, participants: Vec<String>, created_by: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            id: format!("room_{}", timestamp),
            name,
            participants,
            created_at: timestamp,
            last_message_at: timestamp,
            created_by,
            is_private: true,
        }
    }
} 