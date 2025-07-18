import axios from 'axios';
import type { AxiosInstance } from 'axios';
import { config } from '../config';
import type {
  User,
  Peer,
  FileInfo,
  AIRecommendation,
  AITrainingResult,
  AIAggregationResult,
  BootstrapNode,
  AuthResponse,
  LoginRequest,
  RegisterRequest,
  FileShareRequest,
  PeerAddRequest,
  ApiResponse,
  PaginatedResponse,
  Message,
  ChatRoom,
  SendMessageRequest,
  UserSearchResult,
  DeviceInfo,
  PeerDiscoveryResult,
  ConnectPeerRequest,
  PingResult,
  RemoteConnection,
  RemoteCommandResult,
  RemoteServiceStatus,
  RemoteLogEntry,
  RemoteConfig,
  BackupInfo,
  SystemStatus,
  AllowlistEntry,
  UploadProgress,
  DownloadProgress
} from '../types/api';

class ApiClient {
  private client: AxiosInstance;

  constructor() {
    this.client = axios.create({
      baseURL: config.apiUrl,
      timeout: config.apiTimeout,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    // Request interceptor to add auth token
    this.client.interceptors.request.use(
      (config) => {
        const token = localStorage.getItem('auth_token');
        if (token) {
          config.headers.Authorization = `Bearer ${token}`;
        }
        return config;
      },
      (error) => Promise.reject(error)
    );

    // Response interceptor to handle auth errors
    this.client.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response?.status === 401) {
          localStorage.removeItem('auth_token');
          localStorage.removeItem('user');
          window.location.href = '/login';
        }
        return Promise.reject(error);
      }
    );
  }

  // Auth endpoints
  async login(credentials: LoginRequest): Promise<AuthResponse> {
    const response = await this.client.post<ApiResponse<AuthResponse>>('/auth/login', credentials);
    return response.data.data!;
  }

  async register(userData: RegisterRequest): Promise<AuthResponse> {
    const response = await this.client.post<ApiResponse<AuthResponse>>('/auth/register', userData);
    return response.data.data!;
  }

  async logout(): Promise<void> {
    await this.client.post('/auth/logout');
    localStorage.removeItem('auth_token');
    localStorage.removeItem('user');
  }

  async getCurrentUser(): Promise<User> {
    const response = await this.client.get<ApiResponse<User>>('/auth/me');
    return response.data.data!;
  }

  // Peer management
  async getPeers(): Promise<Peer[]> {
    const response = await this.client.get<ApiResponse<Peer[]>>('/p2p/peers');
    return response.data.data!;
  }

  async addPeer(peerData: PeerAddRequest): Promise<Peer> {
    const response = await this.client.post<ApiResponse<Peer>>('/p2p/peers', peerData);
    return response.data.data!;
  }

  async removePeer(peerId: string): Promise<void> {
    await this.client.delete(`/p2p/peers/${peerId}`);
  }

  async getBootstrapNodes(): Promise<BootstrapNode[]> {
    const response = await this.client.get<ApiResponse<BootstrapNode[]>>('/p2p/bootstrap');
    return response.data.data!;
  }

  async addBootstrapNode(node: Omit<BootstrapNode, 'is_active'>): Promise<BootstrapNode> {
    const response = await this.client.post<ApiResponse<BootstrapNode>>('/p2p/bootstrap', node);
    return response.data.data!;
  }

  async removeBootstrapNode(peerId: string): Promise<void> {
    await this.client.delete(`/p2p/bootstrap/${peerId}`);
  }

  // Enhanced Peer Discovery
  async discoverPeers(): Promise<PeerDiscoveryResult> {
    const response = await this.client.post<ApiResponse<PeerDiscoveryResult>>('/p2p/discover');
    return response.data.data!;
  }

  async connectPeer(peerData: ConnectPeerRequest): Promise<Peer> {
    const response = await this.client.post<ApiResponse<Peer>>('/p2p/connect', peerData);
    return response.data.data!;
  }

  async pingPeer(peerId: string): Promise<PingResult> {
    const response = await this.client.post<ApiResponse<PingResult>>(`/p2p/ping/${peerId}`);
    return response.data.data!;
  }

  async scanLocalPeers(): Promise<PeerDiscoveryResult> {
    const response = await this.client.post<ApiResponse<PeerDiscoveryResult>>('/p2p/scan-local');
    return response.data.data!;
  }

  async getPeerHistory(): Promise<Peer[]> {
    const response = await this.client.get<ApiResponse<Peer[]>>('/p2p/history');
    return response.data.data!;
  }

  // File management
  async getFiles(page = 1, perPage = 20): Promise<PaginatedResponse<FileInfo>> {
    const response = await this.client.get<ApiResponse<PaginatedResponse<FileInfo>>>(
      `/files?page=${page}&per_page=${perPage}`
    );
    return response.data.data!;
  }

  async uploadFile(file: File, isPublic = false, tags?: string[], allowedPeers?: string[]): Promise<FileInfo> {
    const formData = new FormData();
    formData.append('file', file);
    formData.append('is_public', isPublic.toString());
    if (tags) {
      formData.append('tags', JSON.stringify(tags));
    }
    if (allowedPeers) {
      formData.append('allowed_peers', JSON.stringify(allowedPeers));
    }

    const response = await this.client.post<ApiResponse<FileInfo>>('/files/upload', formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    });
    return response.data.data!;
  }

  async downloadFile(fileId: string): Promise<Blob> {
    const response = await this.client.get(`/files/${fileId}/download`, {
      responseType: 'blob',
    });
    return response.data;
  }

  async p2pDownloadFile(fileId: string, peerId: string): Promise<Blob> {
    const response = await this.client.post(`/files/${fileId}/p2p-download`, { peer_id: peerId }, {
      responseType: 'blob',
    });
    return response.data;
  }

  async deleteFile(fileId: string): Promise<void> {
    await this.client.delete(`/files/${fileId}`);
  }

  async shareFile(shareData: FileShareRequest): Promise<void> {
    await this.client.post('/files/share', shareData);
  }

  async getFileACL(fileId: string): Promise<any[]> {
    const response = await this.client.get<ApiResponse<any[]>>(`/files/${fileId}/acl`);
    return response.data.data!;
  }

  async updateFileACL(fileId: string, acl: any[]): Promise<void> {
    await this.client.put(`/files/${fileId}/acl`, { acl });
  }

  async getP2PFiles(peerId: string): Promise<FileInfo[]> {
    const response = await this.client.get<ApiResponse<FileInfo[]>>(`/p2p/files/${peerId}`);
    return response.data.data!;
  }

  // Messaging
  async sendMessage(messageData: SendMessageRequest): Promise<Message> {
    const response = await this.client.post<ApiResponse<Message>>('/messaging/send', messageData);
    return response.data.data!;
  }

  async getMessages(recipientId?: string, roomId?: string, limit = 50): Promise<Message[]> {
    const params = new URLSearchParams();
    if (recipientId) params.append('recipient_id', recipientId);
    if (roomId) params.append('room_id', roomId);
    params.append('limit', limit.toString());
    
    const response = await this.client.get<ApiResponse<Message[]>>(`/messaging/messages?${params}`);
    return response.data.data!;
  }

  async createRoom(name: string, participants: string[]): Promise<ChatRoom> {
    const response = await this.client.post<ApiResponse<ChatRoom>>('/messaging/rooms', {
      name,
      participants
    });
    return response.data.data!;
  }

  async joinRoom(roomId: string): Promise<ChatRoom> {
    const response = await this.client.post<ApiResponse<ChatRoom>>(`/messaging/rooms/${roomId}/join`);
    return response.data.data!;
  }

  async getRooms(): Promise<ChatRoom[]> {
    const response = await this.client.get<ApiResponse<ChatRoom[]>>('/messaging/rooms');
    return response.data.data!;
  }

  async sendRoomMessage(roomId: string, message: string): Promise<Message> {
    const response = await this.client.post<ApiResponse<Message>>(`/messaging/rooms/${roomId}/messages`, {
      message
    });
    return response.data.data!;
  }

  async setStatus(status: string, message?: string): Promise<void> {
    await this.client.post('/messaging/status', { status, message });
  }

  async getOnlineUsers(): Promise<User[]> {
    const response = await this.client.get<ApiResponse<User[]>>('/messaging/users/online');
    return response.data.data!;
  }

  // User Management
  async registerUser(username: string, displayName: string, email?: string): Promise<User> {
    const response = await this.client.post<ApiResponse<User>>('/users/register', {
      username,
      display_name: displayName,
      email
    });
    return response.data.data!;
  }

  async loginUser(username: string): Promise<AuthResponse> {
    const response = await this.client.post<ApiResponse<AuthResponse>>('/users/login', { username });
    return response.data.data!;
  }

  async logoutDevice(): Promise<void> {
    await this.client.post('/users/logout-device');
  }

  async getAllUsers(): Promise<User[]> {
    const response = await this.client.get<ApiResponse<User[]>>('/users/all');
    return response.data.data!;
  }

  async searchUsers(query: string): Promise<UserSearchResult> {
    const response = await this.client.get<ApiResponse<UserSearchResult>>(`/users/search?q=${query}`);
    return response.data.data!;
  }

  async changeUsername(newUsername: string): Promise<User> {
    const response = await this.client.put<ApiResponse<User>>('/users/username', { new_username: newUsername });
    return response.data.data!;
  }

  async getDevices(): Promise<DeviceInfo[]> {
    const response = await this.client.get<ApiResponse<DeviceInfo[]>>('/users/devices');
    return response.data.data!;
  }

  async removeDevice(deviceId: string): Promise<void> {
    await this.client.delete(`/users/devices/${deviceId}`);
  }

  async whoAmI(): Promise<User> {
    const response = await this.client.get<ApiResponse<User>>('/users/me');
    return response.data.data!;
  }

  // Allowlist Management
  async getAllowlist(): Promise<AllowlistEntry[]> {
    const response = await this.client.get<ApiResponse<AllowlistEntry[]>>('/system/allowlist');
    return response.data.data!;
  }

  async addToAllowlist(peerId: string): Promise<void> {
    await this.client.post('/system/allowlist', { peer_id: peerId });
  }

  async removeFromAllowlist(peerId: string): Promise<void> {
    await this.client.delete(`/system/allowlist/${peerId}`);
  }

  // AI operations
  async getAIRecommendations(fileIds?: string[]): Promise<AIRecommendation[]> {
    const params = fileIds ? { file_ids: fileIds.join(',') } : {};
    const response = await this.client.get<ApiResponse<AIRecommendation[]>>('/ai/recommendations', { params });
    return response.data.data!;
  }

  async trainAI(parameters?: Record<string, any>): Promise<AITrainingResult> {
    const response = await this.client.post<ApiResponse<AITrainingResult>>('/ai/train', { parameters });
    return response.data.data!;
  }

  async getTrainingStatus(trainingId: string): Promise<AITrainingResult> {
    const response = await this.client.get<ApiResponse<AITrainingResult>>(`/ai/train/${trainingId}`);
    return response.data.data!;
  }

  async aggregateAI(fileIds: string[], parameters?: Record<string, any>): Promise<AIAggregationResult> {
    const response = await this.client.post<ApiResponse<AIAggregationResult>>('/ai/aggregate', {
      file_ids: fileIds,
      parameters,
    });
    return response.data.data!;
  }

  async exportAIModel(outputPath: string): Promise<Blob> {
    const response = await this.client.post('/ai/export', { output_path: outputPath }, {
      responseType: 'blob',
    });
    return response.data;
  }

  // Remote Management
  async connectRemote(host: string, port: number, username: string, password: string): Promise<RemoteConnection> {
    const response = await this.client.post<ApiResponse<RemoteConnection>>('/remote/connect', {
      host,
      port,
      username,
      password
    });
    return response.data.data!;
  }

  async executeRemoteCommand(command: string, parameters?: Record<string, any>): Promise<RemoteCommandResult> {
    const response = await this.client.post<ApiResponse<RemoteCommandResult>>('/remote/execute', {
      command,
      parameters
    });
    return response.data.data!;
  }

  async getRemoteStatus(): Promise<RemoteServiceStatus> {
    const response = await this.client.get<ApiResponse<RemoteServiceStatus>>('/remote/status');
    return response.data.data!;
  }

  async manageRemoteBootstrap(action: string, peerId?: string, addr?: string): Promise<void> {
    await this.client.post('/remote/bootstrap', { action, peer_id: peerId, addr });
  }

  async getRemoteLogs(lines = 100): Promise<RemoteLogEntry[]> {
    const response = await this.client.get<ApiResponse<RemoteLogEntry[]>>(`/remote/logs?lines=${lines}`);
    return response.data.data!;
  }

  async restartRemoteService(): Promise<void> {
    await this.client.post('/remote/restart');
  }

  async stopRemoteService(): Promise<void> {
    await this.client.post('/remote/stop');
  }

  async startRemoteService(): Promise<void> {
    await this.client.post('/remote/start');
  }

  async updateRemoteConfig(key: string, value: string): Promise<void> {
    await this.client.put('/remote/config', { key, value });
  }

  async getRemoteConfig(key?: string): Promise<RemoteConfig[]> {
    const params = key ? { key } : {};
    const response = await this.client.get<ApiResponse<RemoteConfig[]>>('/remote/config', { params });
    return response.data.data!;
  }

  async backupRemoteData(path: string): Promise<BackupInfo> {
    const response = await this.client.post<ApiResponse<BackupInfo>>('/remote/backup', { path });
    return response.data.data!;
  }

  async restoreRemoteData(path: string): Promise<void> {
    await this.client.post('/remote/restore', { path });
  }

  // System status
  async getSystemStatus(): Promise<SystemStatus> {
    const response = await this.client.get<ApiResponse<SystemStatus>>('/system/status');
    return response.data.data!;
  }

  async getLogs(level = 'info', limit = 100): Promise<string[]> {
    const response = await this.client.get<ApiResponse<string[]>>(`/system/logs?level=${level}&limit=${limit}`);
    return response.data.data!;
  }

  // File Transfer Progress
  async getUploadProgress(fileId: string): Promise<UploadProgress> {
    const response = await this.client.get<ApiResponse<UploadProgress>>(`/files/${fileId}/upload-progress`);
    return response.data.data!;
  }

  async getDownloadProgress(fileId: string): Promise<DownloadProgress> {
    const response = await this.client.get<ApiResponse<DownloadProgress>>(`/files/${fileId}/download-progress`);
    return response.data.data!;
  }

  async pauseUpload(fileId: string): Promise<void> {
    await this.client.post(`/files/${fileId}/upload-pause`);
  }

  async resumeUpload(fileId: string): Promise<void> {
    await this.client.post(`/files/${fileId}/upload-resume`);
  }

  async pauseDownload(fileId: string): Promise<void> {
    await this.client.post(`/files/${fileId}/download-pause`);
  }

  async resumeDownload(fileId: string): Promise<void> {
    await this.client.post(`/files/${fileId}/download-resume`);
  }
}

export default new ApiClient(); 