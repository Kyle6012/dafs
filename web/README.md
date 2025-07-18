# DAFS Web Dashboard

A comprehensive web interface for the Decentralized Authenticated File System (DAFS) that provides all the functionality available in the CLI through an intuitive graphical user interface.

## Features

### 🗂️ File Management
- **Upload Files**: Drag-and-drop file upload with progress tracking
- **Download Files**: Direct download or P2P download from peers
- **File Sharing**: Share files with specific users and set permissions
- **Access Control**: Set allowed peers for each file
- **File Tagging**: Organize files with custom tags
- **Progress Tracking**: Real-time upload/download progress with pause/resume
- **File Metadata**: View file size, owner, creation date, and status

### 💬 Messaging System
- **Direct Messages**: Send encrypted messages to individual users
- **Chat Rooms**: Create and join group chat rooms
- **User Status**: Set and view user online/offline status
- **Message History**: View conversation history
- **Real-time Chat**: Live messaging with message acknowledgments

### 👥 User Management
- **User Registration**: Register new user accounts
- **User Login/Logout**: Secure authentication
- **User Search**: Find users by username or display name
- **Device Management**: View and manage connected devices
- **Username Changes**: Update usernames
- **User Profiles**: View user information and status

### 🌐 Peer Discovery
- **Network Discovery**: Discover peers using bootstrap nodes
- **Local Network Scan**: Scan for peers on local network
- **Peer Connection**: Connect to peers by ID or IP address
- **Peer Pinging**: Test connectivity to peers
- **Connection History**: View peer connection history
- **Bootstrap Management**: Add/remove bootstrap nodes

### 🔧 Remote Management
- **Remote Connections**: Connect to remote DAFS services
- **Service Control**: Start, stop, and restart remote services
- **Command Execution**: Execute commands on remote services
- **Configuration Management**: View and update remote configurations
- **Log Monitoring**: View remote service logs
- **Backup/Restore**: Create and restore remote backups
- **Service Status**: Monitor remote service health and metrics

### 🤖 AI Operations
- **AI Training**: Train recommendation models with local data
- **File Recommendations**: Get AI-powered file suggestions
- **Model Aggregation**: Aggregate remote AI models (federated learning)
- **Model Export**: Export trained models for sharing
- **Training Status**: Monitor AI training progress

### 🔒 Security & Access Control
- **File Encryption**: End-to-end encrypted file storage
- **Access Control Lists**: Granular file access permissions
- **Peer Allowlisting**: Control which peers can access files
- **User Authentication**: Secure login with session management
- **Device Tracking**: Monitor and manage connected devices

## Getting Started

### Prerequisites
- Node.js 16+ and npm
- DAFS backend running with API endpoints

### Installation

1. Navigate to the web directory:
```bash
cd web
```

2. Install dependencies:
```bash
npm install
```

3. Configure the API endpoint in `src/config.ts`:
```typescript
export const config = {
  apiUrl: 'http://localhost:6543', // DAFS API endpoint
  apiTimeout: 30000,
  // ... other settings
};
```

4. Start the development server:
```bash
npm run dev
```

5. Open your browser to `http://localhost:5173`

### Building for Production

```bash
npm run build
```

The built files will be in the `dist` directory.

## Usage Guide

### File Management

#### Uploading Files
1. Navigate to the **Files** page
2. Click **Upload File**
3. Select a file from your device
4. Choose whether to make it public
5. Add tags for organization
6. Select allowed peers (optional)
7. Click **Upload**

#### Downloading Files
- **Direct Download**: Click the download icon next to any file
- **P2P Download**: Click the cloud download icon to download from a specific peer

#### Sharing Files
1. Click the share icon next to a file
2. Select recipients from the list
3. Choose permissions (read/write/admin)
4. Click **Share**

### Messaging

#### Sending Messages
1. Go to the **Messaging** page
2. Select a user or room from the left panel
3. Type your message in the input field
4. Press Enter or click Send

#### Creating Chat Rooms
1. Click **Create Room**
2. Enter a room name
3. Select participants
4. Click **Create**

#### Setting Status
1. Click **Set Status**
2. Choose your status (online/away/busy/offline)
3. Add an optional status message
4. Click **Set Status**

### User Management

#### Registering Users
1. Go to **User Management**
2. Click **Register User**
3. Enter username, display name, and optional email
4. Click **Register**

#### Managing Devices
1. Click **My Devices**
2. View all connected devices
3. Remove inactive devices as needed

### Peer Discovery

#### Discovering Peers
1. Go to **Peer Discovery**
2. Click **Discover Peers** to find peers on the network
3. Click **Scan Local Network** to find local peers

#### Connecting to Peers
1. Click **Connect Peer**
2. Enter peer ID and optional address
3. Click **Connect**

#### Managing Bootstrap Nodes
1. Click **Add Bootstrap**
2. Enter peer ID, address, and port
3. Click **Add Bootstrap Node**

### Remote Management

#### Connecting to Remote Services
1. Go to **Remote Management**
2. Click **Connect to Remote Service**
3. Enter host, port, username, and password
4. Click **Connect**

#### Service Control
- **Start Service**: Start the remote DAFS service
- **Stop Service**: Stop the service
- **Restart Service**: Restart the service

#### Executing Commands
1. Click **Execute Command**
2. Enter the command and optional parameters (JSON)
3. Click **Execute**

#### Configuration Management
1. Click **Configuration**
2. View current configuration
3. Update configuration values as needed

#### Backup and Restore
1. **Create Backup**: Click **Backup** and specify backup path
2. **Restore Backup**: Click **Restore** and specify backup path

## API Integration

The web dashboard communicates with the DAFS backend through REST API endpoints. All API calls are authenticated using JWT tokens stored in localStorage.

### Key API Endpoints

- **Authentication**: `/auth/login`, `/auth/register`, `/auth/logout`
- **Files**: `/files`, `/files/upload`, `/files/{id}/download`
- **Messaging**: `/messaging/send`, `/messaging/rooms`, `/messaging/messages`
- **Users**: `/users/register`, `/users/login`, `/users/search`
- **Peers**: `/p2p/peers`, `/p2p/discover`, `/p2p/connect`
- **Remote**: `/remote/connect`, `/remote/execute`, `/remote/status`
- **AI**: `/ai/train`, `/ai/recommendations`, `/ai/aggregate`

## Security Features

- **JWT Authentication**: Secure token-based authentication
- **HTTPS Support**: Encrypted communication with backend
- **Input Validation**: Client-side and server-side validation
- **XSS Protection**: Sanitized user inputs
- **CSRF Protection**: Token-based CSRF protection

## Error Handling

The web interface includes comprehensive error handling:
- Network errors with retry mechanisms
- User-friendly error messages
- Loading states for better UX
- Graceful degradation for offline scenarios

## Browser Support

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## Development

### Project Structure
```
web/
├── src/
│   ├── components/     # Reusable UI components
│   ├── pages/         # Page components
│   ├── contexts/      # React contexts
│   ├── api/           # API client and types
│   ├── types/         # TypeScript type definitions
│   └── utils/         # Utility functions
├── public/            # Static assets
└── dist/              # Built files
```

### Adding New Features
1. Create new page component in `src/pages/`
2. Add API methods in `src/api/client.ts`
3. Update types in `src/types/api.ts`
4. Add route in `src/App.tsx`
5. Update navigation in `src/components/Layout/AppLayout.tsx`

## Troubleshooting

### Common Issues

1. **API Connection Failed**
   - Check if DAFS backend is running
   - Verify API URL in config
   - Check network connectivity

2. **Authentication Issues**
   - Clear browser localStorage
   - Re-login to refresh tokens
   - Check backend authentication settings

3. **File Upload Fails**
   - Check file size limits
   - Verify file permissions
   - Check available disk space

4. **Peer Discovery Issues**
   - Ensure bootstrap nodes are configured
   - Check firewall settings
   - Verify network connectivity

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
