// API Types for DAFS Web Dashboard

export interface User {
  id: string;
  username: string;
  display_name?: string;
  email?: string;
  role: 'admin' | 'user';
  created_at: string;
  devices: Device[];
  status: UserStatus;
}

export interface Device {
  id: string;
  name: string;
  last_seen: string;
  is_active: boolean;
  peer_history: PeerConnection[];
}

export interface UserStatus {
  status: 'online' | 'offline' | 'away' | 'busy';
  message?: string;
  last_updated: string;
}

export interface PeerConnection {
  peer_id: string;
  connected_at: string;
  disconnected_at?: string;
  duration?: number;
}

export interface Peer {
  id: string;
  address: string;
  port: number;
  public_key: string;
  is_online: boolean;
  last_seen: string;
  files_count: number;
  status: UserStatus;
  is_allowed: boolean;
}

export interface FileInfo {
  id: string;
  name: string;
  size: number;
  hash: string;
  owner: string;
  created_at: string;
  updated_at: string;
  is_public: boolean;
  acl: FileACL[];
  shared_with: string[];
  tags: string[];
  checksum: string;
  encrypted_file_key: string;
  allowed_peers: string[];
}

export interface FileACL {
  user_id: string;
  username: string;
  permissions: 'read' | 'write' | 'admin';
}

export interface AIRecommendation {
  id: string;
  file_id: string;
  file_name: string;
  recommendation: string;
  confidence: number;
  created_at: string;
}

export interface AITrainingResult {
  id: string;
  status: 'completed' | 'failed' | 'in_progress';
  accuracy: number;
  training_time: number;
  created_at: string;
  error_message?: string;
}

export interface AIAggregationResult {
  id: string;
  result: string;
  confidence: number;
  created_at: string;
}

export interface BootstrapNode {
  address: string;
  port: number;
  is_active: boolean;
  peer_id: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export interface RegisterRequest {
  username: string;
  display_name: string;
  email?: string;
  password: string;
}

export interface FileUploadRequest {
  file: File;
  is_public?: boolean;
  acl?: FileACL[];
  tags?: string[];
  allowed_peers?: string[];
}

export interface FileShareRequest {
  file_id: string;
  user_ids: string[];
  permissions: 'read' | 'write' | 'admin';
}

export interface PeerAddRequest {
  address: string;
  port: number;
  public_key: string;
}

export interface AIRequest {
  file_ids?: string[];
  parameters?: Record<string, any>;
}

// Messaging Types
export interface Message {
  id: string;
  sender_id: string;
  sender_username: string;
  recipient_id?: string;
  room_id?: string;
  content: string;
  encrypted_content: string;
  timestamp: string;
  is_read: boolean;
  acknowledgment_status: 'pending' | 'delivered' | 'read';
}

export interface ChatRoom {
  id: string;
  name: string;
  participants: string[];
  created_by: string;
  created_at: string;
  last_message?: Message;
  unread_count: number;
}

export interface SendMessageRequest {
  recipient_id?: string;
  room_id?: string;
  message: string;
}

// User Management Types
export interface UserSearchResult {
  users: User[];
  total: number;
}

export interface ChangeUsernameRequest {
  new_username: string;
}

export interface DeviceInfo {
  id: string;
  name: string;
  last_seen: string;
  is_active: boolean;
}

// Peer Discovery Types
export interface PeerDiscoveryResult {
  peers: Peer[];
  total_discovered: number;
  scan_duration: number;
}

export interface ConnectPeerRequest {
  peer_id: string;
  address?: string;
}

export interface PingResult {
  peer_id: string;
  is_online: boolean;
  latency?: number;
  error?: string;
}

// Remote Management Types
export interface RemoteConnection {
  host: string;
  port: number;
  username: string;
  is_connected: boolean;
  last_connected?: string;
}

export interface RemoteCommandRequest {
  command: string;
  parameters?: Record<string, any>;
}

export interface RemoteCommandResult {
  success: boolean;
  output: string;
  error?: string;
  execution_time: number;
}

export interface RemoteServiceStatus {
  is_running: boolean;
  uptime: number;
  version: string;
  memory_usage: number;
  cpu_usage: number;
  active_connections: number;
}

export interface RemoteLogEntry {
  timestamp: string;
  level: 'debug' | 'info' | 'warn' | 'error';
  message: string;
  source: string;
}

export interface RemoteConfig {
  key: string;
  value: string;
  description?: string;
  is_editable: boolean;
}

export interface BackupInfo {
  path: string;
  size: number;
  created_at: string;
  status: 'completed' | 'failed' | 'in_progress';
}

// System Management Types
export interface SystemStatus {
  version: string;
  uptime: number;
  memory_usage: number;
  cpu_usage: number;
  disk_usage: number;
  active_connections: number;
  services: ServiceStatus[];
}

export interface ServiceStatus {
  name: string;
  is_running: boolean;
  port: number;
  last_started?: string;
  error_count: number;
}

export interface AllowlistEntry {
  peer_id: string;
  added_at: string;
  added_by: string;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

// File Transfer Types
export interface UploadProgress {
  file_id: string;
  filename: string;
  bytes_uploaded: number;
  total_bytes: number;
  chunks_uploaded: number;
  total_chunks: number;
  status: 'uploading' | 'completed' | 'failed' | 'paused';
  error?: string;
}

export interface DownloadProgress {
  file_id: string;
  filename: string;
  bytes_downloaded: number;
  total_bytes: number;
  chunks_downloaded: number;
  total_chunks: number;
  status: 'downloading' | 'completed' | 'failed' | 'paused';
  error?: string;
} 