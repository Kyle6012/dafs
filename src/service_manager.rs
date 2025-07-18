use crate::models::{UserIdentity, UserSession};
use crate::remote_management::{RemoteCommand, RemoteCommandResponse, RemoteServiceStatus, ServiceStatus};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use anyhow::Result;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json;
use std::thread;
use std::time::Duration;

// Global service manager
static SERVICE_MANAGER: Lazy<Mutex<ServiceManager>> = Lazy::new(|| {
    Mutex::new(ServiceManager::new())
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub service_name: String,
    pub service_port: u16,
    pub admin_port: u16,
    pub admin_username: String,
    pub admin_password: String,
    pub auto_start: bool,
    pub log_level: String,
    pub data_dir: String,
    pub backup_dir: String,
    pub max_connections: usize,
    pub allowed_ips: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub service_id: String,
    pub version: String,
    pub start_time: u64,
    pub config: ServiceConfig,
    pub status: ServiceStatus,
    pub metrics: ServiceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub uptime: u64,
    pub peer_count: usize,
    pub file_count: usize,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub disk_usage: u64,
    pub active_connections: usize,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
}

pub struct ServiceManager {
    service_info: ServiceInfo,
    connections: std::collections::HashMap<String, Arc<Mutex<TcpStream>>>,
    command_history: Vec<RemoteCommand>,
    is_running: bool,
}

impl ServiceManager {
    pub fn new() -> Self {
        let config = ServiceConfig {
            service_name: "dafs-bootstrap".to_string(),
            service_port: 2093,
            admin_port: 2094,
            admin_username: "admin".to_string(),
            admin_password: "admin123".to_string(),
            auto_start: true,
            log_level: "info".to_string(),
            data_dir: "./data".to_string(),
            backup_dir: "./backups".to_string(),
            max_connections: 100,
            allowed_ips: vec!["127.0.0.1".to_string(), "::1".to_string()],
        };

        Self {
            service_info: ServiceInfo {
                service_id: Uuid::new_v4().to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                start_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                config,
                status: ServiceStatus::Stopped,
                metrics: ServiceMetrics {
                    uptime: 0,
                    peer_count: 0,
                    file_count: 0,
                    memory_usage: 0,
                    cpu_usage: 0.0,
                    disk_usage: 0,
                    active_connections: 0,
                    total_requests: 0,
                    successful_requests: 0,
                    failed_requests: 0,
                },
            },
            connections: HashMap::new(),
            command_history: Vec::new(),
            is_running: false,
        }
    }

    pub async fn start_service(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.is_running {
            return Ok(());
        }
        self.service_info.status = ServiceStatus::Starting;
        tokio::fs::create_dir_all(&self.service_info.config.data_dir).await?;
        tokio::fs::create_dir_all(&self.service_info.config.backup_dir).await?;
        let admin_port = self.service_info.config.admin_port;
        let admin_addr = format!("0.0.0.0:{}", admin_port);
        let listener = TcpListener::bind(&admin_addr).await?;
        println!("DAFS Bootstrap Service started on admin port {}", admin_port);
        self.service_info.status = ServiceStatus::Running;
        self.service_info.start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.is_running = true;
        self.start_metrics_collection();
        let connections = Arc::new(Mutex::new(std::collections::HashMap::new()));
        while self.is_running {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    if self.is_ip_allowed(&addr.ip().to_string()) {
                        let connection_id = Uuid::new_v4().to_string();
                        let socket = Arc::new(Mutex::new(socket));
                        {
                            let mut conns = connections.lock().await;
                            conns.insert(connection_id.clone(), socket.clone());
                        }
                        // Extract only Send-compatible data for the async block
                        let service_info = self.service_info.clone();
                        let socket_owned = socket.clone();
                        tokio::spawn(async move {
                            let _ = ServiceManager::handle_connection(socket_owned, service_info).await;
                        });
                    } else {
                        println!("Connection rejected from unauthorized IP: {}", addr.ip());
                    }
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
        Ok(())
    }

    pub async fn stop_service(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_running {
            return Ok(());
        }
        self.service_info.status = ServiceStatus::Stopping;
        self.is_running = false;
        // Close all connections
        // Use Arc<Mutex<HashMap<...>>> for connections
        // (Assume connections is Arc<Mutex<HashMap<...>>> in aggressive rewrite)
        // If not, clear as before
        // self.connections.clear();
        Ok(())
    }

    pub async fn restart_service(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self.stop_service().await {
            Ok(val) => val,
            Err(e) => return Err(e),
        };
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        match self.start_service().await {
            Ok(val) => val,
            Err(e) => return Err(e),
        };
        Ok(())
    }

    pub fn get_status(&self) -> RemoteServiceStatus {
        RemoteServiceStatus {
            service_id: self.service_info.service_id.clone(),
            status: self.service_info.status.clone(),
            uptime: self.service_info.metrics.uptime,
            version: self.service_info.version.clone(),
            peer_count: self.service_info.metrics.peer_count,
            file_count: self.service_info.metrics.file_count,
            memory_usage: self.service_info.metrics.memory_usage,
            cpu_usage: self.service_info.metrics.cpu_usage,
            disk_usage: self.service_info.metrics.disk_usage,
            last_backup: None, // TODO: Implement backup tracking
        }
    }

    pub async fn execute_command(&mut self, command: &str, user_id: &str) -> Result<RemoteCommandResponse, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = SystemTime::now();
        let result: Result<String, Box<dyn std::error::Error + Send + Sync>> = match command.split_whitespace().next().unwrap_or("") {
            "status" => self.handle_status_command().await.map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e.to_string())),
            "restart" => self.handle_restart_command().await,
            "stop" => self.handle_stop_command().await,
            "start" => self.handle_start_command().await,
            _ => Err(Box::<dyn std::error::Error + Send + Sync>::from(format!("Unknown command: {}", command))),
        };
        let execution_time = start_time.elapsed().unwrap_or_default().as_millis() as u64;
        match result {
            Ok(output) => Ok(RemoteCommandResponse {
                success: true,
                output,
                error: None,
                execution_time,
            }),
            Err(e) => Ok(RemoteCommandResponse {
                success: false,
                output: String::new(),
                error: Some(e.to_string()),
                execution_time,
            }),
        }
    }

    async fn handle_connection(socket: Arc<Mutex<TcpStream>>, service_info: ServiceInfo) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut buffer = [0; 4096];
        // Lock the socket, read, then drop the guard before any .await
        let n = {
            let mut guard = socket.lock().await;
            guard.read(&mut buffer).await?
        };
        if n == 0 {
            return Ok(());
        }
        let request = String::from_utf8_lossy(&buffer[..n]);
        if let Ok(json_request) = serde_json::from_str::<serde_json::Value>(&request) {
            let response = match json_request["type"].as_str() {
                Some("auth") => Self::handle_auth_request(&json_request, &service_info).await,
                Some("command") => {
                    // Extract command string
                    let command = json_request["command"].as_str().unwrap_or("");
                    // Execute command synchronously on SERVICE_MANAGER before spawning
                    let response_result = {
                        let mut manager = SERVICE_MANAGER.blocking_lock();
                        // Use block_in_place to allow blocking in async context
                        tokio::task::block_in_place(|| futures::executor::block_on(manager.execute_command(command, "admin")))
                    };
                    let response = response_result.unwrap_or_else(|e| RemoteCommandResponse {
                        success: false,
                        output: String::new(),
                        error: Some(e.to_string()),
                        execution_time: 0,
                    });
                    serde_json::json!({
                        "success": response.success,
                        "output": response.output,
                        "error": response.error,
                        "execution_time": response.execution_time
                    })
                },
                _ => serde_json::json!({
                    "success": false,
                    "error": "Unknown request type"
                }),
            };
            let response_data = serde_json::to_string(&response)?;
            // Lock the socket again to write, then drop the guard before any .await
            {
                let mut guard = socket.lock().await;
                guard.write_all(response_data.as_bytes()).await?;
            }
        }
        Ok(())
    }

    async fn handle_auth_request(request: &serde_json::Value, service_info: &ServiceInfo) -> serde_json::Value {
        let username = request["username"].as_str().unwrap_or("");
        let password = request["password"].as_str().unwrap_or("");
        
        if username == service_info.config.admin_username && password == service_info.config.admin_password {
            let token = Uuid::new_v4().to_string();
            serde_json::json!({
                "success": true,
                "token": token,
                "message": "Authentication successful"
            })
        } else {
            serde_json::json!({
                "success": false,
                "error": "Invalid credentials"
            })
        }
    }

    async fn handle_command_request(request: &serde_json::Value, service_info: &ServiceInfo) -> serde_json::Value {
        let command = request["command"].as_str().unwrap_or("");
        let _auth_token = request["auth_token"].as_str();
        // TODO: Validate auth token
        let response_result = {
            let mut manager = SERVICE_MANAGER.lock().await;
            manager.execute_command(command, "admin").await
        };
        let response = response_result.unwrap_or_else(|e| RemoteCommandResponse {
            success: false,
            output: String::new(),
            error: Some(e.to_string()),
            execution_time: 0,
        });
        serde_json::json!({
            "success": response.success,
            "output": response.output,
            "error": response.error,
            "execution_time": response.execution_time
        })
    }

    async fn handle_status_command(&self) -> Result<String> {
        let status = self.get_status();
        Ok(serde_json::to_string_pretty(&status)?)
    }

    async fn handle_restart_command(&mut self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match self.restart_service().await {
            Ok(val) => val,
            Err(e) => return Err(e),
        };
        Ok("Service restart initiated".to_string())
    }

    async fn handle_stop_command(&mut self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match self.stop_service().await {
            Ok(val) => val,
            Err(e) => return Err(e),
        };
        Ok("Service stop initiated".to_string())
    }

    async fn handle_start_command(&mut self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match self.start_service().await {
            Ok(val) => val,
            Err(e) => return Err(e),
        };
        Ok("Service start initiated".to_string())
    }

    async fn handle_config_command(&self, command: &str) -> Result<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        match parts.get(1) {
            Some(&"get") => {
                if let Some(key) = parts.get(2) {
                    Ok(format!("Config {} = {}", key, "value")) // TODO: Implement actual config retrieval
                } else {
                    Ok("Current configuration: ...".to_string()) // TODO: Implement full config listing
                }
            }
            Some(&"set") => {
                if parts.len() >= 4 {
                    let key = parts[2];
                    let value = parts[3];
                    Ok(format!("Config {} set to {}", key, value)) // TODO: Implement actual config setting
                } else {
                    Err(anyhow::anyhow!("Usage: config set <key> <value>"))
                }
            }
            _ => Err(anyhow::anyhow!("Usage: config <get|set> [key] [value]")),
        }
    }

    async fn handle_backup_command(&self, command: &str) -> Result<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(path) = parts.get(1) {
            // TODO: Implement actual backup
            Ok(format!("Backup created at {}", path))
        } else {
            Err(anyhow::anyhow!("Usage: backup <path>"))
        }
    }

    async fn handle_restore_command(&self, command: &str) -> Result<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(path) = parts.get(1) {
            // TODO: Implement actual restore
            Ok(format!("Restore completed from {}", path))
        } else {
            Err(anyhow::anyhow!("Usage: restore <path>"))
        }
    }

    async fn handle_logs_command(&self, command: &str) -> Result<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let _lines = parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(50);
        
        // TODO: Implement actual log retrieval
        let logs = vec![
            serde_json::json!({
                "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                "level": "INFO",
                "message": "Service running normally",
                "service": "dafs-bootstrap"
            })
        ];
        
        Ok(serde_json::to_string_pretty(&logs)?)
    }

    async fn handle_add_bootstrap_command(&self, command: &str) -> Result<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() >= 3 {
            let peer_id = parts[1];
            let addr = parts[2];
            
            // TODO: Implement actual bootstrap node addition
            crate::peer::add_bootstrap_node(peer_id, addr)?;
            Ok(format!("Bootstrap node {} added at {}", peer_id, addr))
        } else {
            Err(anyhow::anyhow!("Usage: add-bootstrap <peer_id> <address>"))
        }
    }

    async fn handle_remove_bootstrap_command(&self, command: &str) -> Result<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(peer_id) = parts.get(1) {
            // TODO: Implement actual bootstrap node removal
            crate::peer::remove_bootstrap_node(peer_id)?;
            Ok(format!("Bootstrap node {} removed", peer_id))
        } else {
            Err(anyhow::anyhow!("Usage: remove-bootstrap <peer_id>"))
        }
    }

    async fn handle_list_bootstrap_command(&self) -> Result<String> {
        let nodes = crate::peer::list_bootstrap_nodes();
        Ok(serde_json::to_string_pretty(&nodes)?)
    }

    fn is_ip_allowed(&self, ip: &str) -> bool {
        self.service_info.config.allowed_ips.contains(&ip.to_string())
    }

    fn start_metrics_collection(&self) {
        // TODO: Implement actual metrics collection
        // This would collect system metrics like CPU, memory, disk usage
        // and update the service_info.metrics field
    }
}

// Public API functions
pub fn init_service_manager() -> Result<()> {
    let manager = tokio::task::block_in_place(|| futures::executor::block_on(SERVICE_MANAGER.lock()));
    Ok(())
}

pub async fn start_bootstrap_service() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut manager = SERVICE_MANAGER.lock().await;
    manager.start_service().await
}

pub async fn stop_bootstrap_service() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut manager = SERVICE_MANAGER.lock().await;
    manager.stop_service().await
}

pub async fn restart_bootstrap_service() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut manager = SERVICE_MANAGER.lock().await;
    manager.restart_service().await
}

pub fn get_bootstrap_service_status() -> RemoteServiceStatus {
    let manager = tokio::task::block_in_place(|| futures::executor::block_on(SERVICE_MANAGER.lock()));
    manager.get_status()
}

pub async fn execute_bootstrap_command(command: &str, user_id: &str) -> Result<RemoteCommandResponse, Box<dyn std::error::Error + Send + Sync>> {
    let mut manager = SERVICE_MANAGER.lock().await;
    manager.execute_command(command, user_id).await
} 