use crate::models::{UserIdentity, UserDevice, DeviceType, UserSession};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use anyhow::Result;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

// Global user registry
static USER_REGISTRY: Lazy<Mutex<UserRegistry>> = Lazy::new(|| {
    Mutex::new(UserRegistry::new())
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePeerMemory {
    pub device_id: String,
    pub known_peers: Vec<String>, // List of peer IDs this device has connected to
    pub last_peer_scan: u64,
    pub peer_connection_history: Vec<PeerConnection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnection {
    pub peer_id: String,
    pub connected_at: u64,
    pub disconnected_at: Option<u64>,
    pub connection_duration: Option<u64>,
    pub success: bool,
}

pub struct UserRegistry {
    users: HashMap<String, UserIdentity>, // user_id -> UserIdentity
    username_to_user_id: HashMap<String, String>, // username -> user_id
    sessions: HashMap<String, UserSession>, // session_id -> UserSession
    device_sessions: HashMap<String, String>, // device_id -> session_id
    device_peer_memory: HashMap<String, DevicePeerMemory>, // device_id -> DevicePeerMemory
}

impl UserRegistry {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            username_to_user_id: HashMap::new(),
            sessions: HashMap::new(),
            device_sessions: HashMap::new(),
            device_peer_memory: HashMap::new(),
        }
    }

    pub fn load_from_storage(&mut self) -> Result<()> {
        let users_dir = "users";
        if !Path::new(users_dir).exists() {
            fs::create_dir_all(users_dir)?;
            return Ok(());
        }

        for entry in fs::read_dir(users_dir)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    let user_id = name.trim_end_matches(".json");
                    if let Ok(data) = fs::read_to_string(entry.path()) {
                        if let Ok(user) = serde_json::from_str::<UserIdentity>(&data) {
                            self.users.insert(user_id.to_string(), user.clone());
                            self.username_to_user_id.insert(user.username.clone(), user_id.to_string());
                        }
                    }
                }
            }
        }

        // Load sessions
        let sessions_dir = "sessions";
        if Path::new(sessions_dir).exists() {
            for entry in fs::read_dir(sessions_dir)? {
                let entry = entry?;
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".json") {
                        let session_id = name.trim_end_matches(".json");
                        if let Ok(data) = fs::read_to_string(entry.path()) {
                            if let Ok(session) = serde_json::from_str::<UserSession>(&data) {
                                self.sessions.insert(session_id.to_string(), session.clone());
                                self.device_sessions.insert(session.device_id.clone(), session_id.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Load device peer memory
        let memory_dir = "device_memory";
        if Path::new(memory_dir).exists() {
            for entry in fs::read_dir(memory_dir)? {
                let entry = entry?;
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".json") {
                        let device_id = name.trim_end_matches(".json");
                        if let Ok(data) = fs::read_to_string(entry.path()) {
                            if let Ok(memory) = serde_json::from_str::<DevicePeerMemory>(&data) {
                                self.device_peer_memory.insert(device_id.to_string(), memory);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn save_user(&self, user: &UserIdentity) -> Result<()> {
        let users_dir = "users";
        fs::create_dir_all(users_dir)?;
        
        let filename = format!("{}/{}.json", users_dir, user.user_id);
        let data = serde_json::to_string_pretty(user)?;
        fs::write(filename, data)?;
        Ok(())
    }

    pub fn save_session(&self, session: &UserSession) -> Result<()> {
        let sessions_dir = "sessions";
        fs::create_dir_all(sessions_dir)?;
        
        let filename = format!("{}/{}.json", sessions_dir, session.session_id);
        let data = serde_json::to_string_pretty(session)?;
        fs::write(filename, data)?;
        Ok(())
    }

    pub fn save_device_memory(&self, memory: &DevicePeerMemory) -> Result<()> {
        let memory_dir = "device_memory";
        fs::create_dir_all(memory_dir)?;
        
        let filename = format!("{}/{}.json", memory_dir, memory.device_id);
        let data = serde_json::to_string_pretty(memory)?;
        fs::write(filename, data)?;
        Ok(())
    }

    pub fn register_user(&mut self, username: String, display_name: String, email: Option<String>) -> Result<UserIdentity> {
        // Check if username already exists
        if self.username_to_user_id.contains_key(&username) {
            return Err(anyhow::anyhow!("Username '{}' already exists", username));
        }

        let mut user = UserIdentity::new(username.clone(), display_name, email);
        
        // Generate a unique username if needed (handle conflicts)
        let mut counter = 1;
        let mut final_username = username.clone();
        while self.username_to_user_id.contains_key(&final_username) {
            final_username = format!("{}{}", username, counter);
            counter += 1;
        }
        user.username = final_username.clone();

        // Save to storage
        self.save_user(&user)?;
        
        // Add to registry
        self.users.insert(user.user_id.clone(), user.clone());
        self.username_to_user_id.insert(final_username, user.user_id.clone());
        
        Ok(user)
    }

    pub fn login_user(&mut self, username: &str, device_name: String, device_type: DeviceType) -> Result<(UserIdentity, UserSession)> {
        let user_id = self.username_to_user_id.get(username)
            .ok_or_else(|| anyhow::anyhow!("User '{}' not found", username))?;
        let user_clone = if let Some(user) = self.users.get_mut(user_id) {
            // Create or update device
            let device = UserDevice::new(device_name, device_type);
            user.add_device(device.clone());
            Some(user.clone())
        } else {
            None
        };
        let session = if let Some(user) = user_clone.as_ref() {
            UserSession {
                session_id: Uuid::new_v4().to_string(),
                user_id: user.user_id.clone(),
                device_id: user.devices.last().unwrap().device_id.clone(),
                created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                expires_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + (30 * 24 * 60 * 60),
                is_active: true,
            }
        } else {
            return Err(anyhow::anyhow!("User identity not found"));
        };
        let session_clone = session.clone();
        if let Some(ref user) = user_clone {
            self.save_user(user)?;
        }
        self.save_session(&session_clone)?;
        Ok((user_clone.unwrap(), session_clone))
    }

    pub fn get_user_by_id(&self, user_id: &str) -> Option<&UserIdentity> {
        self.users.get(user_id)
    }

    pub fn get_user_by_username(&self, username: &str) -> Option<&UserIdentity> {
        let user_id = self.username_to_user_id.get(username)?;
        self.users.get(user_id)
    }

    pub fn get_user_by_session(&self, session_id: &str) -> Option<&UserIdentity> {
        let session = self.sessions.get(session_id)?;
        self.users.get(&session.user_id)
    }

    pub fn get_user_by_device(&self, device_id: &str) -> Option<&UserIdentity> {
        let session_id = self.device_sessions.get(device_id)?;
        self.get_user_by_session(session_id)
    }

    pub fn update_user_status(&mut self, user_id: &str) -> Result<()> {
        let user_clone = if let Some(user) = self.users.get_mut(user_id) {
            user.update_last_seen();
            Some(user.clone())
        } else {
            None
        };
        if let Some(user) = user_clone {
            self.save_user(&user)?;
        }
        Ok(())
    }

    pub fn logout_device(&mut self, device_id: &str) -> Result<()> {
        let session_id_opt = self.device_sessions.remove(device_id);
        let session_clone = if let Some(session_id) = session_id_opt {
            if let Some(session) = self.sessions.get_mut(&session_id) {
                session.is_active = false;
                Some(session.clone())
            } else {
                None
            }
        } else {
            None
        };
        if let Some(session) = session_clone {
            self.save_session(&session)?;
        }
        Ok(())
    }

    pub fn list_users(&self) -> Vec<&UserIdentity> {
        self.users.values().collect()
    }

    pub fn search_users(&self, query: &str) -> Vec<&UserIdentity> {
        self.users.values()
            .filter(|user| {
                user.username.to_lowercase().contains(&query.to_lowercase()) ||
                user.display_name.to_lowercase().contains(&query.to_lowercase()) ||
                user.email.as_ref().map_or(false, |email| email.to_lowercase().contains(&query.to_lowercase()))
            })
            .collect()
    }

    pub fn change_username(&mut self, user_id: &str, new_username: String) -> Result<()> {
        let user_clone = if let Some(user) = self.users.get_mut(user_id) {
            user.username = new_username.clone();
            Some(user.clone())
        } else {
            None
        };
        if let Some(user) = user_clone {
            self.save_user(&user)?;
        }
        Ok(())
    }

    pub fn remove_device(&mut self, user_id: &str, device_id: &str) -> Result<()> {
        if let Some(user) = self.users.get_mut(user_id) {
            user.remove_device(device_id);
            let user_clone = user.clone();
            let _ = user;
            self.save_user(&user_clone)?;
        }
        Ok(())
    }

    pub fn add_peer_to_device_memory(&mut self, device_id: &str, peer_id: &str) -> Result<()> {
        let memory = self.device_peer_memory.entry(device_id.to_string()).or_insert_with(|| {
            DevicePeerMemory {
                device_id: device_id.to_string(),
                known_peers: Vec::new(),
                last_peer_scan: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                peer_connection_history: Vec::new(),
            }
        });

        if !memory.known_peers.contains(&peer_id.to_string()) {
            memory.known_peers.push(peer_id.to_string());
        }

        // Add connection record
        let connection = PeerConnection {
            peer_id: peer_id.to_string(),
            connected_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            disconnected_at: None,
            connection_duration: None,
            success: true,
        };
        memory.peer_connection_history.push(connection);

        let memory_clone = memory.clone();
        let _ = memory;
        self.save_device_memory(&memory_clone)?;
        Ok(())
    }

    pub fn record_peer_disconnection(&mut self, device_id: &str, peer_id: &str) -> Result<()> {
        if let Some(memory) = self.device_peer_memory.get_mut(device_id) {
            if let Some(connection) = memory.peer_connection_history.iter_mut().rev().find(|c| c.peer_id == peer_id && c.disconnected_at.is_none()) {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                connection.disconnected_at = Some(now);
                connection.connection_duration = Some(now - connection.connected_at);
            }
            let memory_clone = memory.clone();
            let _ = memory;
            self.save_device_memory(&memory_clone)?;
        }
        Ok(())
    }

    pub fn get_device_known_peers(&self, device_id: &str) -> Vec<String> {
        self.device_peer_memory.get(device_id)
            .map(|memory| memory.known_peers.clone())
            .unwrap_or_default()
    }

    pub fn get_device_connection_history(&self, device_id: &str) -> Vec<PeerConnection> {
        self.device_peer_memory.get(device_id)
            .map(|memory| memory.peer_connection_history.clone())
            .unwrap_or_default()
    }

    pub fn update_device_peer_scan(&mut self, device_id: &str) -> Result<()> {
        if let Some(memory) = self.device_peer_memory.get_mut(device_id) {
            memory.last_peer_scan = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let memory_clone = memory.clone();
            let _ = memory;
            self.save_device_memory(&memory_clone)?;
        }
        Ok(())
    }
}

// Public API functions
pub fn init_user_registry() -> Result<()> {
    let mut registry = USER_REGISTRY.lock().unwrap();
    registry.load_from_storage()
}

pub fn register_user(username: String, display_name: String, email: Option<String>) -> Result<UserIdentity> {
    let mut registry = USER_REGISTRY.lock().unwrap();
    registry.register_user(username, display_name, email)
}

pub fn login_user(username: &str, device_name: String, device_type: DeviceType) -> Result<(UserIdentity, UserSession)> {
    let mut registry = USER_REGISTRY.lock().unwrap();
    registry.login_user(username, device_name, device_type)
}

pub fn get_user_by_id(user_id: &str) -> Option<UserIdentity> {
    let registry = USER_REGISTRY.lock().unwrap();
    registry.get_user_by_id(user_id).cloned()
}

pub fn get_user_by_username(username: &str) -> Option<UserIdentity> {
    let registry = USER_REGISTRY.lock().unwrap();
    registry.get_user_by_username(username).cloned()
}

pub fn get_user_by_session(session_id: &str) -> Option<UserIdentity> {
    let registry = USER_REGISTRY.lock().unwrap();
    registry.get_user_by_session(session_id).cloned()
}

pub fn get_user_by_device(device_id: &str) -> Option<UserIdentity> {
    let registry = USER_REGISTRY.lock().unwrap();
    registry.get_user_by_device(device_id).cloned()
}

pub fn update_user_status(user_id: &str) -> Result<()> {
    let mut registry = USER_REGISTRY.lock().unwrap();
    registry.update_user_status(user_id)
}

pub fn logout_device(device_id: &str) -> Result<()> {
    let mut registry = USER_REGISTRY.lock().unwrap();
    registry.logout_device(device_id)
}

pub fn list_users() -> Vec<UserIdentity> {
    let registry = USER_REGISTRY.lock().unwrap();
    registry.list_users().into_iter().cloned().collect()
}

pub fn search_users(query: &str) -> Vec<UserIdentity> {
    let registry = USER_REGISTRY.lock().unwrap();
    registry.search_users(query).into_iter().cloned().collect()
}

pub fn change_username(user_id: &str, new_username: String) -> Result<()> {
    let mut registry = USER_REGISTRY.lock().unwrap();
    registry.change_username(user_id, new_username)
}

pub fn remove_device(user_id: &str, device_id: &str) -> Result<()> {
    let mut registry = USER_REGISTRY.lock().unwrap();
    registry.remove_device(user_id, device_id)
}

// Enhanced peer memory functions
pub fn add_peer_to_device_memory(device_id: &str, peer_id: &str) -> Result<()> {
    let mut registry = USER_REGISTRY.lock().unwrap();
    registry.add_peer_to_device_memory(device_id, peer_id)
}

pub fn record_peer_disconnection(device_id: &str, peer_id: &str) -> Result<()> {
    let mut registry = USER_REGISTRY.lock().unwrap();
    registry.record_peer_disconnection(device_id, peer_id)
}

pub fn get_device_known_peers(device_id: &str) -> Vec<String> {
    let registry = USER_REGISTRY.lock().unwrap();
    registry.get_device_known_peers(device_id)
}

pub fn get_device_connection_history(device_id: &str) -> Vec<PeerConnection> {
    let registry = USER_REGISTRY.lock().unwrap();
    registry.get_device_connection_history(device_id)
}

pub fn update_device_peer_scan(device_id: &str) -> Result<()> {
    let mut registry = USER_REGISTRY.lock().unwrap();
    registry.update_device_peer_scan(device_id)
} 