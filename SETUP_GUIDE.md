# DAFS Complete Setup Guide

## 🎯 What You'll Learn

This guide will walk you through every step of setting up DAFS (Decentralized Authenticated File System) on your computer. By the end, you'll have a fully working DAFS system that can:

- Store and share files securely
- Send encrypted messages to other users
- Discover and connect to other DAFS users
- Use AI-powered file recommendations
- Access everything through a web interface or command line

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
   git clone https://github.com/Kyle6012/dafs.git
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

### 4.1 Start DAFS with Web Dashboard (Recommended)
1. **In Terminal/Command Prompt**, run:
   ```bash
   ./target/release/dafs --web  # On macOS/Linux
   target\release\dafs.exe --web  # On Windows
   ```

2. **You should see output like**:
   ```
   🚀 Starting DAFS services...
   ✅ HTTP API server started on port 6543
   ✅ gRPC server started on port 50051
   ✅ Web dashboard server started on port 3093
   ✅ P2P network started on port 2093
      Use Ctrl+C to stop
   ```

3. **Keep this terminal window open** - DAFS is now running!

### 4.2 Alternative: Start Interactive CLI
1. **In Terminal/Command Prompt**, run:
   ```bash
   ./target/release/dafs --cli  # On macOS/Linux
   target\release\dafs.exe --cli  # On Windows
   ```

2. **You should see output like**:
   ```
   🚀 DAFS - Decentralized Authenticated File System
   Welcome to DAFS Interactive Shell!
   Type 'help' for available commands, 'exit' to quit.
   
   dafs(guest)> 
   ```

3. **Start the web dashboard from the CLI**:
   ```
   dafs(guest)> startweb
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

### 5.2 Using the Interactive CLI
1. **Start the interactive CLI**:
   ```bash
   ./target/release/dafs --cli
   ```

2. **Register a new user**:
   ```
   dafs(guest)> register alice
   Password: [enter your password]
   ```

3. **Login**:
   ```
   dafs(guest)> login alice
   Password: [enter your password]
   ```

### 5.3 Using Direct Commands
1. **Open a new Terminal/Command Prompt**
2. **Navigate to your DAFS directory**
3. **Register a new user**:
   ```bash
   ./target/release/dafs register alice
   ```
4. **Login**:
   ```bash
   ./target/release/dafs login alice
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

### 6.2 Using the Interactive CLI
1. **Start the interactive CLI**:
   ```bash
   ./target/release/dafs --cli
   ```

2. **Upload a file with tags**:
   ```
   dafs(guest)> upload my_document.pdf --tags work important
   ```

3. **List your files**:
   ```
   dafs(guest)> files
   ```

4. **Download a file**:
   ```
   dafs(guest)> download file_1234567890
   ```

### 6.3 Using Direct Commands
1. **In Terminal/Command Prompt**, run:
   ```bash
   # Upload a file with tags
   ./target/release/dafs upload my_document.pdf --tags work important
   
   # List your files
   ./target/release/dafs files
   
   # Download a file
   ./target/release/dafs download file_1234567890
   ```

## 🌐 Step 7: Connecting with Other Users

### 7.1 Discovering Peers
1. **In your browser**, click on **"Peer Discovery"**
2. **Click "Discover Peers"** to find users on the network
3. **Click "Scan Local Network"** to find users on your WiFi
4. **You'll see a list of discovered users**

### 7.2 Using Interactive CLI
```bash
# Start the CLI
./target/release/dafs --cli

# Discover peers on the network
dafs(guest)> discoverpeers

# Scan your local network
dafs(guest)> scanlocalpeers

# List all known peers
dafs(guest)> peers
```

### 7.3 Using Direct Commands
```bash
# Discover peers on the network
./target/release/dafs discoverpeers

# Scan your local network
./target/release/dafs scanlocalpeers

# List all known peers
./target/release/dafs peers
```

## 💬 Step 8: Sending Your First Message

### 8.1 Using the Web Interface
1. **Click on "Messaging"** in the navigation
2. **You'll see a list of users** on the left side
3. **Click on a user** to start chatting
4. **Type your message** in the text box at the bottom
5. **Press Enter** to send

### 8.2 Using Interactive CLI
```bash
# Start the CLI
./target/release/dafs --cli

# Send a direct message
dafs(guest)> sendmessage bob "Hello! How are you?"

# Start the interactive messaging shell
dafs(guest)> messagingshell
```

### 8.3 Using Direct Commands
```bash
# Send a direct message
./target/release/dafs sendmessage bob "Hello! How are you?"

# Start the interactive messaging shell
./target/release/dafs messagingshell
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
./target/release/dafs --web
```

### 9.3 Adding Bootstrap Nodes
Bootstrap nodes help you find other users:

```bash
# Using CLI
./target/release/dafs addbootstrap QmBootstrapPeer /ip4/1.2.3.4/tcp/2093

# Or using interactive CLI
./target/release/dafs --cli
dafs(guest)> addbootstrap QmBootstrapPeer /ip4/1.2.3.4/tcp/2093
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

### Problem: Database lock issues
**Solution**: If you see "could not acquire lock on dafs_db/db":
```bash
# Remove the database directory
rm -rf dafs_db

# Restart DAFS
./target/release/dafs --web
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
- ✅ Access everything through a web interface or command line

**Welcome to the future of decentralized file sharing!** 🚀

---

## 📋 Quick Reference

### Starting DAFS
```bash
# Start with web dashboard (recommended)
./target/release/dafs --web

# Start interactive CLI
./target/release/dafs --cli

# Access web interface
# Open browser to: http://localhost:3093
```

### Common Commands
```bash
# User management
./target/release/dafs register alice
./target/release/dafs login alice

# File operations
./target/release/dafs upload document.pdf --tags work important
./target/release/dafs files

# Peer discovery
./target/release/dafs discoverpeers
./target/release/dafs peers

# Messaging
./target/release/dafs messagingshell
./target/release/dafs sendmessage bob "Hello!"
```

### Interactive CLI Commands
```bash
# Start CLI
./target/release/dafs --cli

# Register user
dafs(guest)> register alice

# Login
dafs(guest)> login alice

# Upload file
dafs(guest)> upload document.pdf --tags work important

# List files
dafs(guest)> files

# Start web dashboard
dafs(guest)> startweb

# Exit CLI
dafs(guest)> exit
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

### Repository Information
- **Repository**: https://github.com/Kyle6012/dafs
- **Documentation**: See the links at the top of README.md
- **Issues**: https://github.com/Kyle6012/dafs/issues
- **Discussions**: https://github.com/Kyle6012/dafs/discussions 