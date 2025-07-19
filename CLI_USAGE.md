# DAFS CLI Usage Guide

Complete guide to using the DAFS command-line interface with all enhanced features.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Interactive Shell](#interactive-shell)
3. [User Management](#user-management)
4. [Peer Discovery](#peer-discovery)
5. [Messaging](#messaging)
6. [File Operations](#file-operations)
7. [AI Features](#ai-features)
8. [System Management](#system-management)

## Getting Started

### Basic Commands
```bash
# Show help
dafs --help

# Show version
dafs --version

# Start interactive CLI shell
dafs --cli

# Start all services with web dashboard
dafs --web
```

## Interactive Shell

The DAFS interactive shell provides a user-friendly command-line interface with command completion, history, and comprehensive help.

### Starting the Interactive Shell
```bash
# Start the interactive shell
dafs --cli
```

You'll see:
```
🚀 DAFS - Decentralized Authenticated File System
Welcome to DAFS Interactive Shell!
Type 'help' for available commands, 'exit' to quit.

dafs(guest)> 
```

### Interactive Shell Commands
```bash
# Show comprehensive help
help

# Clear screen
clear

# Exit shell
exit
# or
quit
```

### Command Categories in Interactive Shell

#### Service Management
```bash
# Start web dashboard server
startweb [--port <port>]

# Stop web dashboard server
stopweb

# Start HTTP API server
startapi [--port <port>]

# Stop HTTP API server
stopapi

# Start gRPC server
startgrpc [--port <port>]

# Stop gRPC server
stopgrpc
```

#### Authentication
```bash
# Register new user account
register <username>

# Login with username
login <username>

# Logout from current session
logout
```

#### File Operations
```bash
# Upload file with tags
upload <file> --tags <tag1> <tag2>...

# Download file by ID
download <file_id>

# Share file with user
share <file_id> <username>

# List all files
files

# List P2P files
p2pfiles

# Download from P2P peer
p2pdownload <file_id> <peer_id>
```

#### Peer Management
```bash
# List known peers
peers

# Connect to peer
connectpeer <peer_id> [addr]

# Discover peers on network
discoverpeers

# Ping peer for connectivity
pingpeer <peer_id>

# List known peers
listknownpeers

# Remove peer from list
removepeer <peer_id>

# Scan local network for peers
scanlocalpeers

# Show peer connection history
peerhistory
```

#### Bootstrap Node Management
```bash
# Add bootstrap node
addbootstrap <peer> <addr>

# Remove bootstrap node
removebootstrap <peer>

# List all bootstrap nodes
listbootstrap
```

#### AI Operations
```bash
# Train AI recommendation model
aitrain

# Get file recommendations
airecommend <user_id>

# Aggregate remote model
aiaggregate <model_path>

# Export local AI model
aiexport <output_path>
```

#### Messaging
```bash
# Send encrypted message to peer
sendmessage <peer_id> <message>

# Create new chat room
createroom <name> <participants>...

# Join chat room
joinroom <room_id>

# Send message to chat room
sendroommessage <room_id> <message>

# List all chat rooms
listrooms

# List messages in chat room
listmessages <room_id>

# Update user status
setstatus <status>

# List online users
listusers

# Start interactive messaging shell
messagingshell
```

#### User Management
```bash
# Register new user
registeruser <username> <display_name> [email]

# Login user
loginuser <username>

# Logout from device
logoutdevice

# List all registered users
listallusers

# Search for users
searchusers <query>

# Change username
changeusername <new_username>

# List user's devices
listdevices

# Remove device
removedevice <device_id>

# Show current user info
whoami
```

#### Peer Access Control
```bash
# Allow peer access
allowpeer <peer_id>

# Disallow peer access
disallowpeer <peer_id>

# List allowed peers
listallowedpeers
```

#### Remote Management
```bash
# Connect to remote DAFS service
remoteconnect <host> <port> <username> <password>

# Execute command on remote service
remoteexec <command>

# Get remote service status
remotestatus

# Manage remote bootstrap node
remotebootstrap <action> [peer_id] [addr]

# View remote logs
remotelogs [lines]

# Restart remote service
remoterestart

# Stop remote service
remotestop

# Start remote service
remotestart

# Update remote configuration
remoteconfig <key> <value>

# Get remote configuration
remoteconfigget [key]

# Backup remote data
remotebackup <path>

# Restore remote data
remoterestore <path>
```

## Direct Command Usage

You can also use DAFS commands directly without the interactive shell:

### Service Management
```bash
# Start web dashboard
dafs startweb --port 3093

# Stop web dashboard
dafs stopweb

# Start HTTP API server
dafs startapi --port 6543

# Stop HTTP API server
dafs stopapi

# Start gRPC server
dafs startgrpc --port 50051

# Stop gRPC server
dafs stopgrpc
```

### Authentication
```bash
# Register a new user
dafs register alice

# Login with username
dafs login alice

# Logout from current session
dafs logout
```

### File Operations
```bash
# Upload file with tags
dafs upload document.pdf --tags work important

# Download file by ID
dafs download file_1234567890

# Share file with user
dafs share file_1234567890 bob

# List all files
dafs files
```

### Peer Management
```bash
# List known peers
dafs peers

# Add bootstrap node
dafs addbootstrap QmBootstrapPeer /ip4/1.2.3.4/tcp/2093

# Remove bootstrap node
dafs removebootstrap QmBootstrapPeer

# List bootstrap nodes
dafs listbootstrap
```

### AI Operations
```bash
# Train AI model
dafs aitrain

# Get recommendations
dafs airecommend user_123

# Aggregate remote model
dafs aiaggregate model.bin

# Export model
dafs aiexport output.bin
```

## Messaging Features

### Interactive Messaging Shell
```bash
# Start interactive messaging shell
dafs messagingshell
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
dafs sendmessage username "Hello, how are you?"

# Create chat room
dafs createroom "Room Name" user1 user2 user3

# Join chat room
dafs joinroom room_1234567890

# Send message to room
dafs sendroommessage room_1234567890 "Hello everyone!"

# List chat rooms
dafs listrooms

# List messages in room
dafs listmessages room_1234567890

# Set user status
dafs setstatus "Working on project"

# List online users
dafs listusers
```

## File Operations

### Local File Management
```bash
# Upload file with tags
dafs upload document.pdf --tags work important

# Download file
dafs download file_1234567890

# List local files
dafs files

# Share file with user
dafs share file_1234567890 username
```

### P2P File Operations
```bash
# List files from peers
dafs p2pfiles

# Download file from peer
dafs p2pdownload file_1234567890 QmPeerId123...
```

## AI Features

### Model Training and Recommendations
```bash
# Train AI model with local data
dafs aitrain

# Get file recommendations for user
dafs airecommend user_id

# Aggregate remote model
dafs aiaggregate model_path.bin

# Export local model
dafs aiexport output_path.bin
```

## System Management

### Service Control
```bash
# Start web dashboard
dafs startweb --port 3093

# Stop web dashboard
dafs stopweb

# Start HTTP API server
dafs startapi --port 6543

# Stop HTTP API server
dafs stopapi

# Start gRPC server
dafs startgrpc --port 50051

# Stop gRPC server
dafs stopgrpc
```

### Peer Management
```bash
# Allow peer
dafs allowpeer QmPeerId123...

# Disallow peer
dafs disallowpeer QmPeerId123...

# List allowed peers
dafs listallowedpeers
```

## Configuration and Environment

### Environment Variables
```bash
# Set custom ports
export DAFS_API_PORT=8080
export DAFS_GRPC_PORT=50052
export DAFS_WEB_PORT=3000
export DAFS_P2P_PORT=2094

# Start DAFS with custom settings
dafs --web
```

### Configuration Files
DAFS stores configuration in several files:
- `bootstrap_nodes.json`: List of trusted bootstrap nodes
- `discovered_peers.json`: Peers you've discovered
- `users/`: User account data
- `device_memory/`: Device-specific peer memory
- `files/`: Local file storage

## Troubleshooting

### Common Issues

#### DAFS Won't Start
```bash
# Check if ports are in use
netstat -tulpn | grep :6543
netstat -tulpn | grep :50051

# Kill processes using the ports
sudo kill -9 $(lsof -t -i:6543)
sudo kill -9 $(lsof -t -i:50051)

# Check permissions
chmod +x target/release/dafs
```

#### Database Lock Issues
```bash
# Remove database lock
rm -rf dafs_db

# Restart DAFS
dafs --web
```

#### Can't Find Other Users
```bash
# Try manual discovery
dafs discoverpeers

# Scan local network
dafs scanlocalpeers

# Add bootstrap nodes
dafs addbootstrap QmBootstrapPeer /ip4/1.2.3.4/tcp/2093
```

#### Web Interface Won't Load
```bash
# Make sure web dashboard is running
dafs startweb

# Check the URL: http://localhost:3093
# Try a different browser
# Clear browser cache
```

## Advanced Usage

### Scripting with DAFS
```bash
#!/bin/bash
# Start DAFS and perform operations

# Start DAFS
dafs --web &
DAFS_PID=$!

# Wait for startup
sleep 5

# Register user
dafs register alice

# Upload file
dafs upload document.pdf --tags work

# List files
dafs files

# Stop DAFS
kill $DAFS_PID
```

### Integration with Other Tools
```bash
# Use with curl for API access
curl -X POST http://localhost:6543/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"password"}'

# Use with jq for JSON processing
dafs listbootstrap | jq '.nodes[] | .peer_id'
```

## Command Reference

### Complete Command List
For a complete list of all available commands, run:
```bash
dafs --cli
help
```

This will show all available commands categorized by functionality.

### Command Options
Most commands support additional options. Use `--help` with any command to see available options:
```bash
dafs upload --help
dafs startweb --help
```

---

**DAFS CLI** - Complete command-line interface for the Decentralized Authenticated File System  
**Repository**: https://github.com/Kyle6012/dafs 