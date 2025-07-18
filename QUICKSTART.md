# DAFS Quick Start Guide

Get DAFS (Decentralized AI File System) running in 5 minutes!

## 🚀 Prerequisites

- **Rust** (1.70+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Node.js** (18+): `curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash - && sudo apt install nodejs`
- **Git**: `sudo apt install git` (Ubuntu/Debian)

## ⚡ Quick Setup

### 1. Clone and Build Backend

```bash
git clone <repository-url>
cd dafs
cargo build --release
```

### 2. Start DAFS

```bash
./target/release/dafs
```

You should see:
```
🚀 Starting DAFS node in integrated mode...
✅ DAFS node started in integrated mode!
   HTTP API: http://127.0.0.1:6543
   gRPC: grpc://[::1]:50051
   Web Dashboard: Use 'dafs cli startweb' to start
   Use Ctrl+C to stop
```

### 3. Setup Web Dashboard

```bash
cd web
chmod +x setup.sh
./setup.sh
npm run build
cd ..
```

### 4. Start and Access Dashboard

```bash
# Start web dashboard
./target/release/dafs cli startweb

# Open your browser to: http://localhost:3093
```

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

## 🔧 Basic Configuration

### Add Bootstrap Node

For P2P connectivity, add a bootstrap node:

```bash
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

## 📚 Next Steps

- Read the [full README](README.md) for detailed documentation
- Check [API documentation](docs/API.md) for advanced usage
- Explore [AI features](README.md#-ai-system-overview) for federated learning
- Set up [P2P networking](README.md#-p2p-networking) for peer discovery

## 🆘 Need Help?

- **Issues**: [GitHub Issues](https://github.com/your-repo/dafs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-repo/dafs/discussions)
- **Documentation**: [Wiki](https://github.com/your-repo/dafs/wiki)

---

**That's it!** You now have a fully functional DAFS node with AI-powered file recommendations and P2P networking capabilities. 