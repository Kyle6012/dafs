# DAFS Quick Start Guide

Get DAFS (Decentralized Authenticated File System) running in 5 minutes!

## 🚀 Prerequisites

- **Rust** (1.70+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Node.js** (18+): `curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash - && sudo apt install nodejs`
- **Git**: `sudo apt install git` (Ubuntu/Debian)

## ⚡ Quick Setup

### 1. Clone and Build Backend

```bash
git clone https://github.com/Kyle6012/dafs.git
cd dafs
cargo build --release
```

### 2. Setup Web Dashboard

```bash
cd web
chmod +x setup.sh
./setup.sh
npm run build
cd ..
```

### 3. Start DAFS with Web Dashboard

```bash
./target/release/dafs --web
```

You should see:
```
🚀 Starting DAFS services...
✅ HTTP API server started on port 6543
✅ gRPC server started on port 50051
✅ Web dashboard server started on port 3093
✅ P2P network started on port 2093
   Use Ctrl+C to stop
```

### 4. Access Dashboard

Open your browser to: **http://localhost:3093**

## 🎯 First Steps

### 1. Create Account

1. Click "Register" in the web dashboard
2. Enter username and password
3. Click "Register"

### 2. Upload a File

1. Click "Upload File" 
2. Select a file
3. Add tags (optional)
4. Click "Upload"

### 3. Train AI Model

1. Go to "AI Operations" tab
2. Click "Train Model"
3. Wait for training to complete

### 4. Get Recommendations

1. In "AI Operations" tab
2. Click "Get Recommendations"
3. View AI-suggested files

## 🔧 Alternative: Using the CLI

### Start Interactive CLI

```bash
./target/release/dafs --cli
```

You'll see:
```
🚀 DAFS - Decentralized Authenticated File System
Welcome to DAFS Interactive Shell!
Type 'help' for available commands, 'exit' to quit.

dafs(guest)> 
```

### Basic CLI Commands

```bash
# Register a new user
dafs(guest)> register alice

# Login
dafs(guest)> login alice

# Start web dashboard
dafs(guest)> startweb

# Upload a file
dafs(guest)> upload document.pdf --tags work important

# List files
dafs(guest)> files

# List peers
dafs(guest)> peers

# Exit CLI
dafs(guest)> exit
```

### Direct Commands (without interactive shell)

```bash
# Register user
./target/release/dafs register alice

# Login
./target/release/dafs login alice

# Upload file
./target/release/dafs upload document.pdf --tags work important

# List files
./target/release/dafs files

# Start web dashboard
./target/release/dafs startweb
```

## 🔧 Basic Configuration

### Add Bootstrap Node

For P2P connectivity, add a bootstrap node:

```bash
# Using CLI
./target/release/dafs addbootstrap QmBootstrap1 /ip4/1.2.3.4/tcp/2093

# Or using API
curl -X POST http://localhost:6543/p2p/bootstrap/add \
  -H "Content-Type: application/json" \
  -d '{
    "peer_id": "QmBootstrap1",
    "address": "/ip4/1.2.3.4/tcp/2093"
  }'
```

### Check System Status

```bash
# List files
curl -X GET "http://localhost:6543/files?username=YOUR_USERNAME&password=YOUR_PASSWORD"

# List peers
curl -X GET http://localhost:6543/p2p/peers

# Check AI model status
curl -X POST http://localhost:6543/ai/train
```

## 🐛 Troubleshooting

### Backend Won't Start

```bash
# Check if ports are available
netstat -tulpn | grep :6543
netstat -tulpn | grep :50051

# Kill processes using ports
sudo kill -9 $(lsof -t -i:6543)
sudo kill -9 $(lsof -t -i:50051)

# Remove database lock (if needed)
rm -rf dafs_db
```

### Web Dashboard Issues

```bash
# Clear npm cache
npm cache clean --force

# Reinstall dependencies
rm -rf node_modules package-lock.json
npm install
```

### Permission Issues

```bash
# Fix file permissions
chmod +x setup.sh
chmod +x target/release/dafs

# Create required directories
mkdir -p files userkeys
```

### Database Lock Issues

If you see "could not acquire lock on dafs_db/db":

```bash
# Remove the database directory
rm -rf dafs_db

# Restart DAFS
./target/release/dafs --web
```

## 📚 Next Steps

- Read the [full README](README.md) for detailed documentation
- Check [CLI Usage Guide](CLI_USAGE.md) for command-line interface
- Explore [API documentation](docs/API.md) for advanced usage
- Set up [bootstrap nodes](BOOTSTRAP_NODE_MANAGEMENT.md) for P2P networking

## 🆘 Need Help?

- **Issues**: [GitHub Issues](https://github.com/Kyle6012/dafs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Kyle6012/dafs/discussions)
- **Documentation**: See the links at the top of [README.md](README.md)

---

**That's it!** You now have a fully functional DAFS node with AI-powered file recommendations and P2P networking capabilities. 