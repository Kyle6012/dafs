# DAFS Complete Features Guide

## 🎯 What is DAFS?

DAFS (Decentralized Authenticated File System) is a revolutionary system that combines file storage, messaging, AI, and social networking into one decentralized platform. Think of it as a combination of:

- **Dropbox** (file storage and sharing)
- **WhatsApp** (encrypted messaging)
- **LinkedIn** (user profiles and networking)
- **Spotify** (AI recommendations)
- **BitTorrent** (peer-to-peer file sharing)

All running on your own devices without needing big tech companies!

## 🏗️ Core Architecture

### How DAFS Works (Simple Explanation)

Imagine you have a group of friends who all want to share files and chat securely:

1. **Your Device**: You install DAFS on your computer
2. **Network**: DAFS automatically finds other DAFS users on your network and the internet
3. **Storage**: When you upload a file, it gets split into pieces and stored across multiple devices
4. **Security**: Everything is encrypted so only you and people you choose can see your files
5. **Messaging**: You can chat with other users through the same system
6. **AI**: The system learns from your file usage and suggests relevant files

### Key Components

- **P2P Node**: Handles finding other users and transferring data
- **Storage Engine**: Manages your files and metadata
- **Messaging System**: Handles encrypted chat and group conversations
- **User Management**: Manages accounts, devices, and permissions
- **AI Engine**: Provides smart file recommendations
- **Web Interface**: Beautiful browser-based interface
- **API Server**: Allows other programs to connect to DAFS

## 📁 File Management System

### What Makes DAFS File Storage Special?

**Traditional Cloud Storage Problems:**
- Files stored on someone else's servers
- Monthly fees for storage
- Privacy depends on trusting big companies
- Internet required to access files
- Single point of failure

**DAFS File Storage Benefits:**
- Files distributed across trusted peers
- Free to use (you contribute storage to help others)
- Your data never touches central servers
- Works with limited internet connectivity
- No single point of failure

### File Upload Process

When you upload a file to DAFS:

1. **File Analysis**: DAFS examines your file and creates metadata
2. **Chunking**: Large files are split into smaller pieces (chunks)
3. **Encryption**: Each chunk is encrypted with your private key
4. **Distribution**: Chunks are distributed across multiple peer devices
5. **Indexing**: A file index is created to track where chunks are stored
6. **Tagging**: You can add tags to organize your files

### File Download Process

When you download a file:

1. **Request**: DAFS requests the file from the network
2. **Discovery**: The system finds which peers have the file chunks
3. **Retrieval**: Chunks are downloaded from multiple peers simultaneously
4. **Verification**: Each chunk is verified for integrity
5. **Decryption**: Chunks are decrypted using your private key
6. **Assembly**: All chunks are reassembled into the original file

### File Organization Features

#### Tags and Categories
```bash
# Upload with tags
./target/release/dafs upload work_report.pdf --tags work important reports

# Or using interactive CLI
./target/release/dafs --cli
dafs(guest)> upload work_report.pdf --tags work important reports
```

#### File Metadata
- **File Name**: Original filename
- **File Size**: Size in bytes
- **File Type**: Determined by extension and content
- **Creation Date**: When the file was uploaded
- **Last Modified**: When the file was last changed
- **Owner**: Who uploaded the file
- **Tags**: User-defined categories
- **Hash**: Cryptographic fingerprint for verification

#### Access Control
- **Private**: Only you can access
- **Shared**: Specific users you choose can access
- **Public**: Anyone on the network can access (not recommended for personal files)

### File Sharing

#### Sharing with Specific Users
```bash
# Share a file with a user
./target/release/dafs share file_1234567890 bob

# Or using interactive CLI
dafs(guest)> share file_1234567890 bob
```

#### Permission Levels
- **Read**: User can download and view the file
- **Write**: User can modify the file
- **Admin**: User can change sharing settings

#### Sharing via Web Interface
1. Go to the **Files** page
2. Click the **share icon** next to a file
3. Select **recipients** from the list
4. Choose **permission level**
5. Click **Share**

### P2P File Operations

#### Discovering Files from Peers
```bash
# List files available from peers
./target/release/dafs p2pfiles

# Or using interactive CLI
dafs(guest)> p2pfiles
```

#### Downloading from Peers
```bash
# Download a file from a specific peer
./target/release/dafs p2pdownload file_1234567890 QmPeerId123...

# Or using interactive CLI
dafs(guest)> p2pdownload file_1234567890 QmPeerId123...
```

#### File Synchronization
- **Automatic Sync**: Files are automatically synchronized across your devices
- **Conflict Resolution**: When the same file is modified on multiple devices, DAFS helps resolve conflicts
- **Version History**: Keep track of file versions and changes

## 💬 Messaging System

### What Makes DAFS Messaging Special?

**Traditional Messaging Problems:**
- Messages stored on central servers
- Privacy depends on trusting messaging companies
- Messages can be intercepted or monitored
- No control over your data

**DAFS Messaging Benefits:**
- End-to-end encrypted messages
- Messages never touch central servers
- You control your own data
- Works even with limited internet connectivity

### Direct Messaging

#### Sending Messages
```bash
# Send a direct message
./target/release/dafs sendmessage bob "Hello! How are you?"

# Or using interactive CLI
dafs(guest)> sendmessage bob "Hello! How are you?"
```

#### Message Features
- **Real-time Delivery**: Messages are delivered instantly when the recipient is online
- **Offline Storage**: Messages are stored and delivered when the recipient comes online
- **Read Receipts**: See when your messages have been read
- **Message History**: All conversations are stored locally
- **Search**: Search through your message history

### Chat Rooms (Group Conversations)

#### Creating Chat Rooms
```bash
# Create a chat room
./target/release/dafs createroom "Project Team" alice bob charlie

# Or using interactive CLI
dafs(guest)> createroom "Project Team" alice bob charlie
```

#### Managing Chat Rooms
```bash
# Join an existing room
./target/release/dafs joinroom room_1234567890

# Send message to room
./target/release/dafs sendroommessage room_1234567890 "Hello everyone!"

# Or using interactive CLI
dafs(guest)> joinroom room_1234567890
dafs(guest)> sendroommessage room_1234567890 "Hello everyone!"
```

#### Room Features
- **Member Management**: Add and remove members
- **Admin Controls**: Room creators can manage settings
- **File Sharing**: Share files directly in chat rooms
- **Message History**: All room messages are preserved
- **Notifications**: Get notified of new messages

### Interactive Messaging Shell

The messaging shell provides a real-time chat experience in your terminal:

```bash
# Start the messaging shell
./target/release/dafs cli messaging-shell
```

#### Shell Commands
- `send <user> <message>` - Send message to user
- `room create <name> <user1> <user2>` - Create chat room
- `room join <id>` - Join existing room
- `room list` - List all rooms
- `room message <id> <message>` - Send message to room
- `peers list` - List known users
- `status set <message>` - Set your status
- `clear` - Clear the screen
- `help` - Show all commands
- `exit` - Leave the shell

### User Status and Presence

#### Setting Your Status
```bash
# Set status message
./target/release/dafs cli set-status "Working on project"

# Set status with emoji
./target/release/dafs cli set-status "In a meeting 📅"
```

#### Status Types
- **Online**: Available for messaging
- **Away**: Temporarily unavailable
- **Busy**: Do not disturb
- **Offline**: Not available

#### Viewing Other Users
```bash
# List online users
./target/release/dafs cli list-users

# View specific user info
./target/release/dafs cli user-info "bob"
```

## 👥 User Management System

### What Makes DAFS User Management Special?

**Traditional User Systems Problems:**
- Usernames must be globally unique
- No multi-device support
- Account recovery is difficult
- Privacy concerns with central databases

**DAFS User Management Benefits:**
- Usernames can be changed and are locally unique
- Seamless multi-device support
- Secure account recovery
- Your data stays on your devices

### User Registration and Authentication

#### Creating an Account
```bash
# Register a new user
./target/release/dafs cli register-user "alice" "Alice Johnson" "alice@example.com"

# Register with minimal info
./target/release/dafs cli register-user "bob" "Bob Smith"
```

#### Account Information
- **Username**: Unique identifier (can be changed)
- **Display Name**: Your real name or preferred name
- **Email**: Optional contact information
- **User ID**: Cryptographically generated unique ID
- **Registration Date**: When the account was created

#### Logging In
```bash
# Login with username
./target/release/dafs cli login-user "alice"

# Check current user
./target/release/dafs cli who-am-i

# Logout from current device
./target/release/dafs cli logout-device
```

### Multi-Device Support

#### How Multi-Device Works
1. **Register** on your first device
2. **Login** on additional devices using the same username
3. **Automatic Sync**: All your data syncs across devices
4. **Device Management**: See and control all your devices

#### Managing Devices
```bash
# List all your devices
./target/release/dafs cli list-devices

# Remove a device (if you lost your phone, etc.)
./target/release/dafs cli remove-device "device_1234567890"

# View device details
./target/release/dafs cli device-info "device_1234567890"
```

#### Device Information
- **Device ID**: Unique identifier for each device
- **Device Name**: Automatically detected (e.g., "Alice's MacBook")
- **Last Seen**: When the device was last active
- **IP Address**: Current network address
- **Platform**: Operating system information

### User Operations

#### Searching for Users
```bash
# Search by username
./target/release/dafs cli search-users "alice"

# Search by display name
./target/release/dafs cli search-users "Alice Johnson"

# List all users
./target/release/dafs cli list-all-users
```

#### Managing Your Profile
```bash
# Change username
./target/release/dafs cli change-username "alice_new"

# Update display name
./target/release/dafs cli update-display-name "Alice Johnson-Smith"

# Update email
./target/release/dafs cli update-email "alice.new@example.com"
```

#### User Privacy
- **Profile Visibility**: Control who can see your profile
- **Contact Information**: Choose what to share
- **Online Status**: Control who can see when you're online
- **File Sharing**: Granular control over file access

### Session Management

#### How Sessions Work
1. **Login**: Creates a new session on the device
2. **Session Token**: Cryptographically secure token for authentication
3. **Auto-Refresh**: Sessions automatically refresh while active
4. **Logout**: Invalidates the session token

#### Session Security
- **Token Expiration**: Sessions expire after inactivity
- **Device Binding**: Sessions are tied to specific devices
- **Revocation**: You can revoke sessions from any device
- **Audit Trail**: Track all login/logout events

## 🌐 Peer Discovery System

### What Makes DAFS Peer Discovery Special?

**Traditional Discovery Problems:**
- Requires central servers to find users
- Limited to specific networks
- No privacy protection
- Single point of failure

**DAFS Discovery Benefits:**
- Multiple discovery methods
- Works across the internet
- Privacy-preserving discovery
- No central dependencies

### Discovery Methods

#### 1. Local Network Discovery (mDNS)
**How it works**: Automatically finds other DAFS users on your WiFi network

```bash
# Scan local network
./target/release/dafs cli scan-local-peers

# What you'll see:
# Found peer: QmAlice123... (Alice's MacBook) - Online
# Found peer: QmBob456... (Bob's iPhone) - Online
```

**Benefits**:
- Works without internet
- Very fast discovery
- Automatic peer finding
- No configuration needed

#### 2. Distributed Hash Table (Kademlia DHT)
**How it works**: Uses a peer-to-peer network to find users across the internet

```bash
# Discover peers on the internet
./target/release/dafs cli discover-peers

# What you'll see:
# Discovering peers via DHT...
# Found peer: QmCharlie789... (Charlie's Server) - Online
# Found peer: QmDavid012... (David's Laptop) - Away
```

**Benefits**:
- Works across the internet
- No central servers needed
- Scalable to millions of users
- Privacy-preserving

#### 3. Bootstrap Nodes
**How it works**: Connect to trusted servers that help you find other users

```bash
# Add a bootstrap node
./target/release/dafs cli add-bootstrap "QmBootstrap1" "/ip4/1.2.3.4/tcp/2093"

# List bootstrap nodes
./target/release/dafs cli list-bootstrap

# Remove bootstrap node
./target/release/dafs cli remove-bootstrap "QmBootstrap1"
```

**Benefits**:
- Reliable entry points to the network
- Can be run by trusted organizations
- Helps new users find the network
- Provides network stability

#### 4. Manual Peer Connection
**How it works**: Connect directly to users you know

```bash
# Connect by peer ID
./target/release/dafs cli connect-peer "QmFriend123..."

# Connect by IP address
./target/release/dafs cli connect-peer "QmFriend123..." --addr "/ip4/192.168.1.100/tcp/2093"
```

**Benefits**:
- Direct connection to known users
- Works even if discovery fails
- Useful for private networks
- Immediate connection

### Peer Management

#### Viewing Known Peers
```bash
# List all known peers
./target/release/dafs cli list-known-peers

# What you'll see:
# Peer ID: QmAlice123...
#   Name: Alice's MacBook
#   Status: Online
#   Last Seen: 2 minutes ago
#   Connection: Established
#   Latency: 15ms
```

#### Peer Information
- **Peer ID**: Unique identifier for each peer
- **Display Name**: Human-readable name
- **Status**: Online, Away, Busy, Offline
- **Last Seen**: When the peer was last active
- **Connection Status**: Connected, Disconnected, Connecting
- **Latency**: Network response time
- **IP Address**: Network address (if known)

#### Testing Connectivity
```bash
# Ping a peer to test connectivity
./target/release/dafs cli ping-peer "QmAlice123..."

# What you'll see:
# Pinging QmAlice123... (Alice's MacBook)
# Response time: 15ms
# Status: Online
```

#### Managing Peer List
```bash
# Remove a peer from your list
./target/release/dafs cli remove-peer "QmSpamUser123..."

# Clear all peers (start fresh)
./target/release/dafs cli clear-peers
```

### Device Peer Memory

#### How Device Memory Works
Each device remembers its own peer connections:

```bash
# View device peer history
./target/release/dafs cli peer-history

# What you'll see:
# Device: Alice's MacBook
# Known Peers:
#   QmBob456... (Bob's iPhone) - Last seen: 1 hour ago
#   QmCharlie789... (Charlie's Server) - Last seen: 2 days ago
```

#### Benefits of Device Memory
- **Faster Reconnection**: Devices remember how to connect to peers
- **Offline Discovery**: Can connect to peers even when discovery is down
- **Personalized Experience**: Each device has its own peer network
- **Privacy**: Peer lists are device-specific

## 🤖 AI-Powered Features

### What Makes DAFS AI Special?

**Traditional AI Problems:**
- AI models trained on your data without permission
- Privacy concerns with cloud-based AI
- No control over AI recommendations
- Centralized AI services

**DAFS AI Benefits:**
- AI runs on your own devices
- Your data never leaves your control
- Federated learning across trusted peers
- Personalized recommendations

### AI Training

#### How AI Training Works
1. **Data Collection**: DAFS learns from your file usage patterns
2. **Local Training**: AI models are trained on your device
3. **Federated Learning**: Models are shared with trusted peers
4. **Continuous Learning**: Models improve over time

#### Training Your AI Model
```bash
# Train AI model with your files
./target/release/dafs cli train-ai

# What you'll see:
# Training AI model...
# Analyzing 150 files...
# Learning usage patterns...
# Model training complete!
```

#### Training Data
The AI learns from:
- **File Access Patterns**: Which files you open most often
- **File Relationships**: Files that are used together
- **Time Patterns**: When you access different types of files
- **User Behavior**: How you organize and tag files

### File Recommendations

#### Getting Recommendations
```bash
# Get AI recommendations
./target/release/dafs cli get-recommendations

# What you'll see:
# AI Recommendations:
# 1. work_report_final.pdf (95% relevance)
#    - Similar to: work_report_draft.pdf
#    - Reason: Frequently accessed work documents
# 2. family_photos_2024.zip (87% relevance)
#    - Similar to: vacation_photos_2023.zip
#    - Reason: Photo collections accessed together
```

#### Recommendation Types
- **Similar Files**: Files that are related to what you're working on
- **Frequently Used**: Files you access often
- **Recently Relevant**: Files that might be useful now
- **Collaborative**: Files shared by your team members

#### Improving Recommendations
```bash
# Provide feedback on recommendations
./target/release/dafs cli rate-recommendation "file_1234567890" 5

# Clear recommendation history
./target/release/dafs cli clear-recommendations
```

### Federated Learning

#### How Federated Learning Works
1. **Local Training**: Each user trains their own AI model
2. **Model Sharing**: Models are shared with trusted peers
3. **Model Aggregation**: Models are combined to create better models
4. **Privacy Preservation**: Raw data never leaves your device

#### Participating in Federated Learning
```bash
# Share your AI model with peers
./target/release/dafs cli share-ai-model

# Import AI models from peers
./target/release/dafs cli import-ai-models

# View federated learning status
./target/release/dafs cli federated-status
```

#### Benefits of Federated Learning
- **Better Recommendations**: Models learn from more data
- **Privacy Preserved**: Your data stays on your device
- **Collaborative Intelligence**: Everyone benefits from shared learning
- **No Central Control**: No single entity controls the AI

### AI Model Management

#### Exporting Models
```bash
# Export your trained model
./target/release/dafs cli export-model "my_model_v1"

# What you'll see:
# Exporting AI model...
# Model saved to: models/my_model_v1.json
# Size: 2.3MB
# Accuracy: 87%
```

#### Importing Models
```bash
# Import a model from file
./target/release/dafs cli import-model "models/friend_model.json"

# Import from peer
./target/release/dafs cli import-peer-model "QmAlice123..."
```

#### Model Information
- **Model Version**: Version number and date
- **Training Data**: What data was used to train
- **Accuracy**: How well the model performs
- **Size**: File size of the model
- **Features**: What the model can predict

## 🔧 Remote Management System

### What Makes DAFS Remote Management Special?

**Traditional Remote Management Problems:**
- Requires complex VPN setups
- Security concerns with remote access
- Limited control over remote systems
- No integrated management interface

**DAFS Remote Management Benefits:**
- Built-in secure remote access
- Integrated with the DAFS system
- Full control over remote services
- No additional software needed

### Remote Service Management

#### Setting Up Remote Services
1. **Install DAFS** on the remote computer
2. **Start as Service**: Run DAFS as a system service
3. **Configure Access**: Set up authentication for remote access
4. **Connect**: Use DAFS CLI to connect and manage

#### Connecting to Remote Services
```bash
# Connect to remote DAFS service
./target/release/dafs cli remote-connect "192.168.1.100" 6543 "admin" "password"

# What you'll see:
# Connecting to remote service...
# Authentication successful!
# Remote service: DAFS v1.0.0
# Status: Running
```

#### Remote Commands
```bash
# Execute command on remote service
./target/release/dafs cli remote-exec "list-files"

# Start remote service
./target/release/dafs cli remote-start

# Stop remote service
./target/release/dafs cli remote-stop

# Restart remote service
./target/release/dafs cli remote-restart
```

### Service Monitoring

#### Viewing Service Status
```bash
# Check remote service status
./target/release/dafs cli remote-status

# What you'll see:
# Service: DAFS
# Status: Running
# Uptime: 5 days, 3 hours
# CPU Usage: 2.3%
# Memory Usage: 45MB
# Active Connections: 12
```

#### Monitoring Metrics
- **Service Status**: Running, Stopped, Error
- **Uptime**: How long the service has been running
- **Resource Usage**: CPU, memory, disk usage
- **Network Activity**: Active connections, data transfer
- **Error Logs**: Recent errors and warnings

#### Log Management
```bash
# View remote service logs
./target/release/dafs cli remote-logs

# View recent errors
./target/release/dafs cli remote-logs --level error

# Download log files
./target/release/dafs cli remote-download-logs
```

### Configuration Management

#### Viewing Remote Configuration
```bash
# View remote configuration
./target/release/dafs cli remote-config

# What you'll see:
# API Port: 6543
# gRPC Port: 50051
# Web Port: 3093
# Storage Path: /home/user/dafs/files
# Log Level: INFO
```

#### Updating Remote Configuration
```bash
# Update remote configuration
./target/release/dafs cli remote-update-config --api-port 8080

# Update multiple settings
./target/release/dafs cli remote-update-config --api-port 8080 --log-level DEBUG
```

### Backup and Restore

#### Creating Backups
```bash
# Create remote backup
./target/release/dafs cli remote-backup

# What you'll see:
# Creating backup...
# Files: 1,234 files backed up
# Users: 45 users backed up
# Configuration: Backed up
# Backup saved to: backup_2024_01_15.tar.gz
```

#### Restoring from Backup
```bash
# Restore from backup
./target/release/dafs cli remote-restore "backup_2024_01_15.tar.gz"

# What you'll see:
# Restoring from backup...
# Files: 1,234 files restored
# Users: 45 users restored
# Configuration: Restored
# Service restarted successfully
```

## 🔒 Security Features

### What Makes DAFS Secure?

**Traditional Security Problems:**
- Data stored on untrusted servers
- Encryption keys controlled by companies
- Centralized authentication systems
- Single points of failure

**DAFS Security Benefits:**
- End-to-end encryption
- Your keys, your data
- Decentralized authentication
- No single point of failure

### Encryption

#### How Encryption Works
1. **Key Generation**: Each user generates their own encryption keys
2. **File Encryption**: Files are encrypted before leaving your device
3. **Message Encryption**: Messages are encrypted end-to-end
4. **Key Management**: You control all your encryption keys

#### Encryption Types
- **AES-256**: For file encryption
- **RSA-2048**: For key exchange
- **ChaCha20-Poly1305**: For message encryption
- **SHA-256**: For data integrity

### Authentication

#### User Authentication
```bash
# Login with username
./target/release/dafs cli login-user "alice"

# What happens:
# 1. Username is verified locally
# 2. Session token is generated
# 3. Token is encrypted and stored
# 4. Session is established
```

#### Device Authentication
- **Device Registration**: Each device gets a unique certificate
- **Certificate Verification**: Devices verify each other's identity
- **Session Management**: Secure session tokens for device communication

### Access Control

#### File Access Control
```bash
# Set file permissions
./target/release/dafs cli set-permissions "file_1234567890" "bob" "read"

# Permission levels:
# - read: Can download and view
# - write: Can modify the file
# - admin: Can change permissions
```

#### User Access Control
- **Profile Privacy**: Control who can see your profile
- **Online Status**: Control who can see when you're online
- **Message Privacy**: Control who can send you messages

### Network Security

#### Peer Verification
```bash
# Verify peer identity
./target/release/dafs cli verify-peer "QmAlice123..."

# What happens:
# 1. Peer certificate is checked
# 2. Certificate chain is verified
# 3. Peer identity is confirmed
# 4. Secure connection is established
```

#### Network Protection
- **DHT Security**: Distributed hash table with built-in security
- **Bootstrap Node Verification**: Trusted bootstrap nodes only
- **Connection Encryption**: All network connections are encrypted

## 🌐 Web Interface

### What Makes DAFS Web Interface Special?

**Traditional Web Apps Problems:**
- Require constant internet connection
- Data stored on remote servers
- Limited offline functionality
- Privacy concerns

**DAFS Web Interface Benefits:**
- Works with limited connectivity
- All data stays on your device
- Full offline functionality
- Complete privacy control

### Web Dashboard Features

#### File Management Interface
- **Drag-and-Drop Upload**: Simply drag files to upload
- **Progress Tracking**: Real-time upload/download progress
- **File Preview**: Preview files before downloading
- **Bulk Operations**: Select multiple files for operations
- **Search and Filter**: Find files quickly

#### Messaging Interface
- **Real-time Chat**: Live messaging with typing indicators
- **File Sharing**: Share files directly in chat
- **Emoji Support**: Full emoji and reaction support
- **Message History**: Scrollable conversation history
- **Room Management**: Create and manage chat rooms

#### User Management Interface
- **Profile Editor**: Update your profile information
- **Device Manager**: View and manage your devices
- **User Search**: Find and connect with other users
- **Privacy Settings**: Control your privacy preferences

#### Peer Discovery Interface
- **Network Map**: Visual representation of your network
- **Peer Status**: Real-time status of all peers
- **Connection Manager**: Manage peer connections
- **Bootstrap Configuration**: Configure bootstrap nodes

### Web Interface Setup

#### Starting the Web Interface
```bash
# Start web dashboard
./target/release/dafs cli startweb

# What you'll see:
# 🌐 Starting web dashboard...
# ✅ Web dashboard started on http://localhost:3093
# 📱 Mobile-friendly interface available
# 🔒 Secure HTTPS available
```

#### Accessing the Interface
- **Local Access**: http://localhost:3093
- **Network Access**: http://your-ip:3093
- **Mobile Access**: Works on phones and tablets
- **Offline Access**: Works without internet

### Web Interface Customization

#### Themes and Appearance
- **Light Theme**: Clean, modern light interface
- **Dark Theme**: Easy on the eyes dark interface
- **Custom Colors**: Customize interface colors
- **Responsive Design**: Works on all screen sizes

#### Layout Customization
- **Sidebar Position**: Left or right sidebar
- **Panel Sizes**: Adjustable panel sizes
- **Widget Arrangement**: Customize dashboard layout
- **Shortcuts**: Custom keyboard shortcuts

## 📊 Monitoring and Analytics

### System Monitoring

#### Performance Metrics
```bash
# View system performance
./target/release/dafs cli system-stats

# What you'll see:
# CPU Usage: 2.3%
# Memory Usage: 45MB
# Disk Usage: 1.2GB
# Network: 15KB/s upload, 45KB/s download
# Active Connections: 12
```

#### Resource Monitoring
- **CPU Usage**: How much processing power is being used
- **Memory Usage**: How much RAM is being used
- **Disk Usage**: How much storage space is used
- **Network Activity**: Data transfer rates
- **Connection Count**: Number of active peer connections

### Usage Analytics

#### File Usage Statistics
```bash
# View file usage stats
./target/release/dafs cli file-stats

# What you'll see:
# Total Files: 1,234
# Total Size: 2.3GB
# Most Used: work_report.pdf (accessed 45 times)
# Recently Added: 23 files in last 7 days
```

#### Network Statistics
```bash
# View network stats
./target/release/dafs cli network-stats

# What you'll see:
# Active Peers: 15
# Total Peers: 45
# Messages Sent: 234
# Files Shared: 67
# Network Uptime: 99.8%
```

### Health Monitoring

#### System Health Checks
```bash
# Run health check
./target/release/dafs cli health-check

# What you'll see:
# ✅ Storage: Healthy
# ✅ Network: Healthy
# ✅ Encryption: Healthy
# ✅ AI Models: Healthy
# ⚠️  Peer Discovery: 2 peers offline
```

#### Automated Monitoring
- **Health Checks**: Regular system health verification
- **Performance Alerts**: Notifications when performance degrades
- **Error Reporting**: Automatic error detection and reporting
- **Recovery Actions**: Automatic recovery from common issues

## 🔧 Advanced Configuration

### Configuration Files

#### Main Configuration
```json
{
  "api": {
    "port": 6543,
    "host": "127.0.0.1"
  },
  "grpc": {
    "port": 50051,
    "host": "[::1]"
  },
  "web": {
    "port": 3093,
    "host": "127.0.0.1"
  },
  "storage": {
    "path": "./files",
    "max_size": "10GB"
  },
  "network": {
    "p2p_port": 2093,
    "discovery_timeout": 30
  }
}
```

#### User Configuration
```json
{
  "username": "alice",
  "display_name": "Alice Johnson",
  "email": "alice@example.com",
  "privacy": {
    "profile_visible": true,
    "online_status_visible": true,
    "file_sharing_enabled": true
  },
  "notifications": {
    "messages": true,
    "file_shares": true,
    "peer_discovery": false
  }
}
```

### Environment Variables

#### Setting Environment Variables
```bash
# Set custom ports
export DAFS_API_PORT=8080
export DAFS_GRPC_PORT=50052
export DAFS_WEB_PORT=3000

# Set storage path
export DAFS_STORAGE_PATH="/home/user/dafs/files"

# Set log level
export DAFS_LOG_LEVEL=DEBUG

# Start DAFS with custom settings
./target/release/dafs
```

#### Available Environment Variables
- `DAFS_API_PORT`: HTTP API port (default: 6543)
- `DAFS_GRPC_PORT`: gRPC port (default: 50051)
- `DAFS_WEB_PORT`: Web dashboard port (default: 3093)
- `DAFS_P2P_PORT`: P2P communication port (default: 2093)
- `DAFS_STORAGE_PATH`: File storage path
- `DAFS_LOG_LEVEL`: Logging level (DEBUG, INFO, WARN, ERROR)
- `DAFS_BOOTSTRAP_NODES`: Comma-separated bootstrap node list

### Performance Tuning

#### Network Optimization
```bash
# Optimize for high-latency networks
./target/release/dafs cli config --network-timeout 60

# Optimize for low-bandwidth networks
./target/release/dafs cli config --chunk-size 64KB

# Optimize for high-bandwidth networks
./target/release/dafs cli config --chunk-size 1MB
```

#### Storage Optimization
```bash
# Set maximum storage size
./target/release/dafs cli config --max-storage 10GB

# Enable compression
./target/release/dafs cli config --enable-compression

# Set cache size
./target/release/dafs cli config --cache-size 1GB
```

## 🚀 Getting Started Checklist

### Initial Setup
- [ ] Install prerequisites (Rust, Node.js, Git)
- [ ] Download and build DAFS
- [ ] Start DAFS for the first time
- [ ] Create your first account
- [ ] Upload your first file

### Basic Usage
- [ ] Explore the web interface
- [ ] Send your first message
- [ ] Discover and connect to peers
- [ ] Create a chat room
- [ ] Share a file with someone

### Advanced Features
- [ ] Set up bootstrap nodes
- [ ] Train your AI model
- [ ] Configure remote management
- [ ] Set up automated backups
- [ ] Customize your configuration

### Security Setup
- [ ] Review privacy settings
- [ ] Set up file permissions
- [ ] Configure device management
- [ ] Test backup and restore
- [ ] Verify encryption is working

## 🆘 Need Help?

### Getting Support
- **Documentation**: Read the complete guides
- **Community**: Join the DAFS community
- **Issues**: Report bugs on GitHub
- **Discussions**: Ask questions and share ideas

### Common Issues
- **Can't find other users**: Check network settings and bootstrap nodes
- **Files won't upload**: Check disk space and permissions
- **Messages not sending**: Check peer connectivity
- **Web interface won't load**: Check if web dashboard is running

### Performance Tips
- **Use wired connections** for better performance
- **Add more bootstrap nodes** for better peer discovery
- **Regular backups** to protect your data
- **Monitor system resources** to avoid bottlenecks

---

**Welcome to the decentralized future!** 🚀

You now have a complete understanding of all DAFS features. Start exploring and discover the power of decentralized file sharing, messaging, and AI! 