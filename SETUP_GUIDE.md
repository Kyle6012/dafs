# DAFS Complete Setup Guide

## 🎯 What You'll Learn

This guide will walk you through every step of setting up DAFS (Decentralized Authenticated File System) on your computer. By the end, you'll have a fully working DAFS system that can:

- Store and share files securely
- Send encrypted messages to other users
- Discover and connect to other DAFS users
- Use AI-powered file recommendations
- Access everything through a web interface

## 📋 Prerequisites Checklist

Before we start, make sure you have:

- [ ] A computer running Windows, macOS, or Linux
- [ ] At least 2GB of free disk space
- [ ] Internet connection for downloading software
- [ ] Basic familiarity with using a terminal/command prompt
- [ ] Administrator/sudo access on your computer

## 🛠️ Step 1: Installing Required Software

### What We're Installing

1. **Rust** - The programming language DAFS is written in
2. **Node.js** - For the web interface
3. **Git** - To download DAFS code
4. **Build Tools** - To compile DAFS

### Windows Installation

#### 1.1 Install Rust
1. **Go to** https://rustup.rs/
2. **Download** the installer (rustup-init.exe)
3. **Run** the installer
4. **Choose option 1** (default installation)
5. **Wait** for installation to complete
6. **Restart** your computer

#### 1.2 Install Node.js
1. **Go to** https://nodejs.org/
2. **Download** the LTS version (recommended)
3. **Run** the installer
4. **Follow** the installation wizard
5. **Restart** your computer

#### 1.3 Install Git
1. **Go to** https://git-scm.com/
2. **Download** the Windows version
3. **Run** the installer
4. **Use default settings** throughout
5. **Restart** your computer

#### 1.4 Verify Installation
1. **Open Command Prompt** (search for "cmd" in Start menu)
2. **Type these commands**:
   ```cmd
   rustc --version
   cargo --version
   node --version
   npm --version
   git --version
   ```
3. **You should see version numbers** for each command

### macOS Installation

#### 1.1 Install Homebrew (Package Manager)
1. **Open Terminal** (Applications > Utilities > Terminal)
2. **Copy and paste** this command:
   ```bash
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```
3. **Press Enter** and wait for installation
4. **Follow** any instructions that appear

#### 1.2 Install Rust
1. **In Terminal**, run:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **Press 1** for default installation
3. **Wait** for installation to complete
4. **Restart Terminal** or run:
   ```bash
   source ~/.cargo/env
   ```

#### 1.3 Install Node.js
1. **In Terminal**, run:
   ```bash
   brew install node
   ```
2. **Wait** for installation to complete

#### 1.4 Install Git
1. **In Terminal**, run:
   ```bash
   brew install git
   ```
2. **Wait** for installation to complete

#### 1.5 Verify Installation
1. **In Terminal**, run:
   ```bash
   rustc --version
   cargo --version
   node --version
   npm --version
   git --version
   ```
2. **You should see version numbers** for each command

### Linux Installation (Ubuntu/Debian)

#### 1.1 Update Your System
1. **Open Terminal** (Ctrl+Alt+T)
2. **Run these commands**:
   ```bash
   sudo apt update
   sudo apt upgrade -y
   ```

#### 1.2 Install Rust
1. **In Terminal**, run:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **Press 1** for default installation
3. **Wait** for installation to complete
4. **Reload your shell**:
   ```bash
   source ~/.cargo/env
   ```

#### 1.3 Install Node.js
1. **In Terminal**, run:
   ```bash
   curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
   sudo apt install -y nodejs
   ```

#### 1.4 Install Git and Build Tools
1. **In Terminal**, run:
   ```bash
   sudo apt install git build-essential
   ```

#### 1.5 Verify Installation
1. **In Terminal**, run:
   ```bash
   rustc --version
   cargo --version
   node --version
   npm --version
   git --version
   ```
2. **You should see version numbers** for each command

### Linux Installation (Arch Linux)

#### 1.1 Update Your System
1. **Open Terminal**
2. **Run**:
   ```bash
   sudo pacman -Syu
   ```

#### 1.2 Install All Dependencies
1. **In Terminal**, run:
   ```bash
   sudo pacman -S rust nodejs npm git base-devel
   ```

#### 1.3 Verify Installation
1. **In Terminal**, run:
   ```bash
   rustc --version
   cargo --version
   node --version
   npm --version
   git --version
   ```
2. **You should see version numbers** for each command

## 📥 Step 2: Downloading DAFS

### 2.1 Choose Installation Location
1. **Decide where** you want to install DAFS
   - **Windows**: `C:\Users\YourName\Documents\dafs`
   - **macOS/Linux**: `~/Documents/dafs` or `~/dafs`

### 2.2 Download DAFS Code
1. **Open Terminal/Command Prompt**
2. **Navigate to your chosen location**:
   ```bash
   # Windows
   cd C:\Users\YourName\Documents
   
   # macOS/Linux
   cd ~/Documents
   ```

3. **Download DAFS**:
   ```bash
   git clone https://github.com/your-username/dafs.git
   cd dafs
   ```

### 2.3 Verify Download
1. **Check that files were downloaded**:
   ```bash
   ls  # On macOS/Linux
   dir # On Windows
   ```

2. **You should see files like**:
   - `Cargo.toml`
   - `README.md`
   - `src/` directory
   - `web/` directory

## 🔨 Step 3: Building DAFS

### 3.1 Build the Backend (Rust)
1. **Make sure you're in the dafs directory**:
   ```bash
   pwd  # Should show path ending with /dafs
   ```

2. **Build DAFS** (this will take 5-15 minutes):
   ```bash
   cargo build --release
   ```

3. **What's happening**: This compiles all the Rust code into an executable program

4. **Wait for completion** - you'll see progress messages like:
   ```
   Compiling dafs v0.1.0
   Finished release [optimized] target(s) in 2m 30s
   ```

### 3.2 Build the Web Interface
1. **Navigate to the web directory**:
   ```bash
   cd web
   ```

2. **Install web dependencies**:
   ```bash
   npm install
   ```

3. **Build the web interface**:
   ```bash
   npm run build
   ```

4. **Return to the main directory**:
   ```bash
   cd ..
   ```

### 3.3 Verify Build
1. **Check that the executable was created**:
   ```bash
   ls target/release/dafs  # On macOS/Linux
   dir target\release\dafs.exe  # On Windows
   ```

2. **Test the executable**:
   ```bash
   ./target/release/dafs --version  # On macOS/Linux
   target\release\dafs.exe --version  # On Windows
   ```

## 🚀 Step 4: Starting DAFS for the First Time

### 4.1 Start the Backend
1. **In Terminal/Command Prompt**, run:
   ```bash
   ./target/release/dafs  # On macOS/Linux
   target\release\dafs.exe  # On Windows
   ```

2. **You should see output like**:
   ```
   🚀 Starting DAFS node in integrated mode...
   ✅ DAFS node started in integrated mode!
      HTTP API: http://127.0.0.1:6543
      gRPC: grpc://[::1]:50051
      Web Dashboard: Use 'dafs cli startweb' to start
      Use Ctrl+C to stop
   ```

3. **Keep this terminal window open** - DAFS is now running!

### 4.2 Start the Web Dashboard
1. **Open a new Terminal/Command Prompt window**
2. **Navigate to your DAFS directory**:
   ```bash
   cd /path/to/your/dafs
   ```

3. **Start the web dashboard**:
   ```bash
   ./target/release/dafs cli startweb  # On macOS/Linux
   target\release\dafs.exe cli startweb  # On Windows
   ```

4. **You should see output like**:
   ```
   🌐 Starting web dashboard...
   ✅ Web dashboard started on http://localhost:3093
   ```

### 4.3 Access the Web Interface
1. **Open your web browser** (Chrome, Firefox, Safari, Edge)
2. **Go to**: `http://localhost:3093`
3. **You should see the DAFS web interface**!

## 👤 Step 5: Creating Your First Account

### 5.1 Using the Web Interface (Recommended)
1. **In your browser**, click **"Register"** in the top right
2. **Fill in your details**:
   - **Username**: Choose a unique username (e.g., "alice")
   - **Display Name**: Your real name (e.g., "Alice Johnson")
   - **Email**: Your email address (optional)
3. **Click "Register"**
4. **You're now logged in!**

### 5.2 Using the Command Line
1. **Open a new Terminal/Command Prompt**
2. **Navigate to your DAFS directory**
3. **Register a new user**:
   ```bash
   ./target/release/dafs cli register-user "alice" "Alice Johnson" "alice@example.com"
   ```
4. **Login**:
   ```bash
   ./target/release/dafs cli login-user "alice"
   ```

## 📁 Step 6: Your First File Upload

### 6.1 Using the Web Interface
1. **In your browser**, click on the **"Files"** tab
2. **Click "Upload File"**
3. **Select a file** from your computer (any file will work)
4. **Add some tags** (optional):
   - For a work document: `work`, `important`
   - For a photo: `photos`, `family`
   - For a video: `videos`, `fun`
5. **Choose privacy settings**:
   - **Private**: Only you can see it
   - **Shared**: You can share it with specific people
   - **Public**: Anyone on the network can see it (not recommended for personal files)
6. **Click "Upload"**
7. **Wait for the upload to complete** (you'll see a progress bar)

### 6.2 Using the Command Line
1. **In Terminal/Command Prompt**, run:
   ```bash
   # Upload a file with tags
   ./target/release/dafs cli upload "my_document.pdf" "work" "important"
   
   # List your files
   ./target/release/dafs cli files
   
   # Download a file
   ./target/release/dafs cli download "file_1234567890"
   ```

## 🌐 Step 7: Connecting with Other Users

### 7.1 Discovering Peers
1. **In your browser**, click on **"Peer Discovery"**
2. **Click "Discover Peers"** to find users on the network
3. **Click "Scan Local Network"** to find users on your WiFi
4. **You'll see a list of discovered users**

### 7.2 Using Command Line
```bash
# Discover peers on the network
./target/release/dafs cli discover-peers

# Scan your local network
./target/release/dafs cli scan-local-peers

# List all known peers
./target/release/dafs cli list-known-peers
```

## 💬 Step 8: Sending Your First Message

### 8.1 Using the Web Interface
1. **Click on "Messaging"** in the navigation
2. **You'll see a list of users** on the left side
3. **Click on a user** to start chatting
4. **Type your message** in the text box at the bottom
5. **Press Enter** to send

### 8.2 Using the Command Line
```bash
# Send a direct message
./target/release/dafs cli send-message "bob" "Hello! How are you?"

# Start the interactive messaging shell
./target/release/dafs cli messaging-shell
```

## 🔧 Step 9: Configuration and Customization

### 9.1 Understanding DAFS Files
DAFS creates several files and directories:

```
dafs/
├── bootstrap_nodes.json    # Trusted peer addresses
├── discovered_peers.json   # Peers you've found
├── users/                  # User account data
├── device_memory/          # Device-specific data
├── files/                  # Local file storage
└── logs/                   # System logs
```

### 9.2 Customizing Ports
If you need to change the default ports:

```bash
# Set custom ports (before starting DAFS)
export DAFS_API_PORT=8080
export DAFS_GRPC_PORT=50052
export DAFS_WEB_PORT=3000

# Start DAFS
./target/release/dafs
```

### 9.3 Adding Bootstrap Nodes
Bootstrap nodes help you find other users:

```bash
# Add a bootstrap node (get these from other DAFS users)
./target/release/dafs cli add-bootstrap "QmBootstrapPeer" "/ip4/1.2.3.4/tcp/2093"

# List your bootstrap nodes
./target/release/dafs cli list-bootstrap
```

## 🚨 Troubleshooting Common Issues

### Problem: "Command not found" errors
**Solution**: Make sure you've installed all prerequisites and restarted your terminal

### Problem: Port already in use
**Solution**: 
```bash
# Find what's using the port
netstat -tulpn | grep :6543  # Linux/macOS
netstat -an | findstr :6543  # Windows

# Kill the process
sudo kill -9 $(lsof -t -i:6543)  # Linux/macOS
```

### Problem: Build fails
**Solution**:
1. Make sure you have enough disk space
2. Try updating Rust: `rustup update`
3. Clear build cache: `cargo clean`

### Problem: Web interface won't load
**Solution**:
1. Make sure the web dashboard is running
2. Check the URL: `http://localhost:3093`
3. Try a different browser
4. Clear browser cache

### Problem: Can't find other users
**Solution**:
1. Make sure you're on the same network (for local discovery)
2. Try manual discovery commands
3. Add bootstrap nodes
4. Check your firewall settings

## 📚 Next Steps

### What You Can Do Now
1. **Upload more files** and organize them with tags
2. **Create chat rooms** for group conversations
3. **Share files** with other users
4. **Explore AI features** for file recommendations
5. **Set up bootstrap nodes** to find more users

### Advanced Features to Explore
1. **Remote Management**: Manage DAFS on other computers
2. **AI Training**: Train custom recommendation models
3. **File Sharing**: Set up complex sharing permissions
4. **Network Configuration**: Optimize for your network

### Getting Help
- **Read the full README**: More detailed information
- **Check CLI Usage Guide**: Complete command reference
- **Join the Community**: Ask questions and share experiences
- **Report Issues**: Help improve DAFS

## 🎉 Congratulations!

You've successfully set up DAFS and are now part of a decentralized file sharing network! You can:

- ✅ Store and share files securely
- ✅ Send encrypted messages
- ✅ Discover and connect to other users
- ✅ Use AI-powered features
- ✅ Access everything through a web interface

**Welcome to the future of decentralized file sharing!** 🚀

---

## 📋 Quick Reference

### Starting DAFS
```bash
# Start backend
./target/release/dafs

# Start web dashboard (in new terminal)
./target/release/dafs cli startweb

# Access web interface
# Open browser to: http://localhost:3093
```

### Common Commands
```bash
# User management
./target/release/dafs cli register-user "username" "Name" "email"
./target/release/dafs cli login-user "username"

# File operations
./target/release/dafs cli upload "file.txt" "tag1" "tag2"
./target/release/dafs cli files

# Peer discovery
./target/release/dafs cli discover-peers
./target/release/dafs cli list-known-peers

# Messaging
./target/release/dafs cli messaging-shell
./target/release/dafs cli send-message "user" "message"
```

### Important URLs
- **Web Dashboard**: http://localhost:3093
- **API Server**: http://localhost:6543
- **gRPC Server**: localhost:50051

### Important Ports
- **2093**: P2P communication
- **6543**: Web API
- **50051**: gRPC
- **3093**: Web dashboard 