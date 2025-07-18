# DAFS CLI Usage Guide

Complete guide to using the DAFS command-line interface with all enhanced features.

## Table of Contents

1. [Getting Started](#getting-started)
2. [User Management](#user-management)
3. [Peer Discovery](#peer-discovery)
4. [Messaging](#messaging)
5. [File Operations](#file-operations)
6. [AI Features](#ai-features)
7. [System Management](#system-management)
8. [Interactive Shell](#interactive-shell)

## Getting Started

### Basic Commands
```bash
# Show help
dafs-cli --help

# Show version
dafs-cli --version

# Start all services
dafs-cli start

# Stop all services
dafs-cli stop
```

## User Management

### Registration and Authentication
```bash
# Register a new user
dafs-cli register-user "username" "Display Name" "email@example.com"

# Login with username
dafs-cli login-user "username"

# Logout from current device
dafs-cli logout-device

# Show current user info
dafs-cli who-am-i
```

### User Operations
```bash
# List all registered users
dafs-cli list-all-users

# Search for users
dafs-cli search-users "query"

# Change username
dafs-cli change-username "new_username"

# List user's devices
dafs-cli list-devices

# Remove a device
dafs-cli remove-device "device_id"
```

## Peer Discovery

### Automatic Discovery
```bash
# Discover peers on the network
dafs-cli discover-peers

# Scan local network for peers
dafs-cli scan-local-peers

# List all known peers
dafs-cli list-known-peers
```

### Manual Peer Connection
```bash
# Connect to peer by ID
dafs-cli connect-peer "QmPeerId123..."

# Connect to peer by IP address
dafs-cli connect-peer "QmPeerId123..." --addr "/ip4/192.168.1.100/tcp/2093"

# Ping peer to check connectivity
dafs-cli ping-peer "QmPeerId123..."

# Remove peer from known list
dafs-cli remove-peer "QmPeerId123..."
```

### Bootstrap Nodes
```bash
# Add bootstrap node
dafs-cli add-bootstrap "QmBootstrapPeer" "/ip4/1.2.3.4/tcp/2093"

# List bootstrap nodes
dafs-cli list-bootstrap

# Remove bootstrap node
dafs-cli remove-bootstrap "QmBootstrapPeer"
```

### Peer History
```bash
# Show device peer connection history
dafs-cli peer-history
```

## Messaging

### Interactive Messaging Shell
```bash
# Start interactive messaging shell
dafs-cli messaging-shell
```

The messaging shell provides real-time communication with the following commands:

#### Basic Commands
- `help` - Show available commands
- `clear` - Clear the screen
- `exit` or `quit` - Exit the shell

#### Sending Messages
- `send <peer> <message>` - Send message to peer
- `room message <room_id> <message>` - Send message to chat room

#### Chat Rooms
- `room create <name>` - Create new chat room
- `room join <room_id>` - Join existing chat room
- `room list` - List all chat rooms

#### Peer Management
- `peers list` - List known peers
- `peers ping <peer>` - Ping peer for connectivity
- `peers connect <peer>` - Connect to peer

#### User Status
- `status set <message>` - Set user status message
- `status show` - Show current user status

### Direct Messaging Commands
```bash
# Send message to peer
dafs-cli send-message "username" "Hello, how are you?"

# Create chat room
dafs-cli create-room "Room Name" "user1" "user2" "user3"

# Join chat room
dafs-cli join-room "room_1234567890"

# Send message to room
dafs-cli send-room-message "room_1234567890" "Hello everyone!"

# List chat rooms
dafs-cli list-rooms

# List messages in room
dafs-cli list-messages "room_1234567890"

# Set user status
dafs-cli set-status "Working on project"

# List online users
dafs-cli list-users
```

## File Operations

### Local File Management
```bash
# Upload file with tags
dafs-cli upload "document.pdf" "work" "important"

# Download file
dafs-cli download "file_1234567890"

# List local files
dafs-cli files

# Share file with user
dafs-cli share "file_1234567890" "username"
```

### P2P File Operations
```bash
# List files from peers
dafs-cli p2p-files

# Download file from peer
dafs-cli p2p-download "file_1234567890" "QmPeerId123..."
```

## AI Features

### Model Training and Recommendations
```bash
# Train AI model with local data
dafs-cli ai-train

# Get file recommendations for user
dafs-cli ai-recommend "user_id"

# Aggregate remote model
dafs-cli ai-aggregate "model_path.bin"

# Export local model
dafs-cli ai-export "output_path.bin"
```

## System Management

### Service Control
```bash
# Start web dashboard
dafs-cli start-web --port 3093

# Stop web dashboard
dafs-cli stop-web

# Start HTTP API server
dafs-cli start-api --port 6543

# Stop HTTP API server
dafs-cli stop-api

# Start gRPC server
dafs-cli start-grpc --port 50051

# Stop gRPC server
dafs-cli stop-grpc
```

### Peer Management
```bash
# Allow peer
dafs-cli allow-peer "QmPeerId123..."

# Disallow peer
dafs-cli disallow-peer "QmPeerId123..."

# List allowed peers
dafs-cli list-allowed-peers
```

## Interactive Shell

### Starting the Shell
```bash
# Start interactive shell
dafs-cli
```

The interactive shell provides command completion and history:

- **Tab Completion**: Press Tab to complete commands
- **Command History**: Use Up/Down arrows to navigate history
- **Multi-line Commands**: Use `\` at end of line for continuation
- **Clear Screen**: Type `clear` to clear the screen
- **Exit**: Type `exit` or `quit` to exit

### Shell Commands
All CLI commands are available in the interactive shell. The shell provides:
- Command completion
- Syntax highlighting
- Error handling
- Persistent history

## Examples

### Complete Workflow Example

```bash
# 1. Start the system
dafs-cli start

# 2. Register and login
dafs-cli register-user "alice" "Alice Johnson" "alice@example.com"
dafs-cli login-user "alice"

# 3. Discover and connect to peers
dafs-cli discover-peers
dafs-cli connect-peer "QmBobPeer" --addr "/ip4/192.168.1.100/tcp/2093"

# 4. Start messaging
dafs-cli messaging-shell

# In the messaging shell:
# > send bob Hello Bob!
# > room create "Project Team"
# > room join room_1234567890
# > room message room_1234567890 Hello team!
# > peers list
# > status set "Working on DAFS project"
# > exit

# 5. Upload and share files
dafs-cli upload "presentation.pdf" "work" "important"
dafs-cli share "file_1234567890" "bob"

# 6. Use AI features
dafs-cli ai-train
dafs-cli ai-recommend "alice"

# 7. Check system status
dafs-cli who-am-i
dafs-cli peer-history
dafs-cli list-known-peers
```

### Advanced Peer Discovery Example

```bash
# Add multiple bootstrap nodes
dafs-cli add-bootstrap "QmBootstrap1" "/ip4/1.2.3.4/tcp/2093"
dafs-cli add-bootstrap "QmBootstrap2" "/ip4/5.6.7.8/tcp/2093"

# Discover peers using multiple methods
dafs-cli discover-peers
dafs-cli scan-local-peers

# Connect to specific peers
dafs-cli connect-peer "QmAlice" --addr "/ip4/192.168.1.101/tcp/2093"
dafs-cli connect-peer "QmBob" --addr "/ip4/192.168.1.102/tcp/2093"

# Test connectivity
dafs-cli ping-peer "QmAlice"
dafs-cli ping-peer "QmBob"

# View peer information
dafs-cli list-known-peers
dafs-cli peer-history
```

### Messaging Shell Session Example

```bash
$ dafs-cli messaging-shell
Starting interactive messaging shell...
Type 'help' for available commands, 'exit' to quit
DAFS(alice)> help
DAFS Messaging Shell Commands
────────────────────────────────────────────────────────
  send <peer> <message> - Send message to peer
  room create <name> - Create new chat room
  room join <id> - Join existing chat room
  room list - List all chat rooms
  room message <id> <message> - Send message to room
  peers list - List known peers
  peers ping <peer> - Ping peer for connectivity
  peers connect <peer> - Connect to peer
  status set <message> - Set user status
  status show - Show current status
  clear - Clear screen
  help - Show help
  exit/quit - Exit messaging shell

DAFS(alice)> peers list
👥 Known Peers (3):
  🟢 QmBob (/ip4/192.168.1.102/tcp/2093)
  🟢 QmCharlie (/ip4/192.168.1.103/tcp/2093)
  🔴 QmDavid (/ip4/192.168.1.104/tcp/2093)

DAFS(alice)> send bob Hello Bob, how's the project going?
📨 Message sent to bob

DAFS(alice)> room create "DAFS Development"
🏠 Chat room 'DAFS Development' created

DAFS(alice)> room list
📋 Available chat rooms:
  🏠 DAFS Development (1 participants) - ID: room_1234567890

DAFS(alice)> room message room_1234567890 Hello team! Working on peer discovery features.
💬 Message sent to room room_1234567890

DAFS(alice)> status set "Implementing enhanced peer discovery"
✅ Status updated: Implementing enhanced peer discovery

DAFS(alice)> status show
👤 Current User: alice (user_1234567890)
📱 Device: device_9876543210
🕒 Last seen: 2024-01-15 14:30:25

DAFS(alice)> exit
Exiting messaging shell.
```

## Troubleshooting

### Common Issues

1. **Peer Connection Failed**
   ```bash
   # Check if peer is online
   dafs-cli ping-peer "QmPeerId"
   
   # Try connecting with explicit address
   dafs-cli connect-peer "QmPeerId" --addr "/ip4/192.168.1.100/tcp/2093"
   ```

2. **Message Not Delivered**
   ```bash
   # Check if recipient is online
   dafs-cli list-users
   
   # Verify peer connection
   dafs-cli list-known-peers
   ```

3. **User Not Found**
   ```bash
   # Search for users
   dafs-cli search-users "username"
   
   # List all users
   dafs-cli list-all-users
   ```

4. **Device Issues**
   ```bash
   # Check current device
   dafs-cli who-am-i
   
   # List all devices
   dafs-cli list-devices
   
   # Remove problematic device
   dafs-cli remove-device "device_id"
   ```

### Debug Commands

```bash
# Check system status
dafs-cli who-am-i
dafs-cli list-known-peers
dafs-cli peer-history

# Test connectivity
dafs-cli ping-peer "QmPeerId"
dafs-cli discover-peers

# Verify user authentication
dafs-cli list-all-users
dafs-cli search-users "username"
```

## Configuration Files

### Bootstrap Nodes
Stored in `bootstrap_nodes.json`:
```json
[
  ["QmBootstrap1", "/ip4/1.2.3.4/tcp/2093"],
  ["QmBootstrap2", "/ip4/5.6.7.8/tcp/2093"]
]
```

### Discovered Peers
Stored in `discovered_peers.json`:
```json
{
  "QmPeer1": {
    "peer_id": "QmPeer1",
    "addresses": ["/ip4/192.168.1.100/tcp/2093"],
    "last_seen": 1705321825,
    "is_online": true,
    "latency_ms": 15
  }
}
```

### User Data
Stored in `users/` directory:
- `users/user_id.json` - User identity information
- `sessions/session_id.json` - Session data
- `device_memory/device_id.json` - Device peer memory

## Best Practices

1. **Peer Discovery**
   - Use multiple bootstrap nodes for better connectivity
   - Regularly scan for new peers
   - Maintain a list of trusted peers

2. **Messaging**
   - Use the interactive shell for real-time communication
   - Create chat rooms for group discussions
   - Set status messages to inform others of your availability

3. **File Sharing**
   - Use descriptive tags when uploading files
   - Share files only with trusted users
   - Regularly backup important files

4. **User Management**
   - Use strong, unique usernames
   - Regularly review and clean up device list
   - Keep session information secure

5. **System Management**
   - Monitor peer connections regularly
   - Keep bootstrap nodes updated
   - Use appropriate ports for different services 