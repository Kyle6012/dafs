# DAFS Web Integration Guide

This document explains how the web dashboard is now fully integrated into the DAFS system, making it a single unified application that can be packaged and distributed as one unit.

## 🏗️ Integration Architecture

### Before Integration
- **Separate components**: Rust backend + React web dashboard
- **Multiple processes**: Backend server + web dev server
- **Complex deployment**: Need to manage two separate services
- **Port conflicts**: Web dashboard on 5173, API on 6543

### After Integration
- **Unified system**: Single Rust binary serves everything
- **Single process**: One DAFS process handles all services
- **Simple deployment**: One binary, one service
- **Clean ports**: Web dashboard on 3093, API on 6543

## 🔧 How It Works

### Build Process

1. **Cargo Build Trigger**
   ```bash
   cargo build --release
   ```

2. **Build Script Execution** (`build.rs`)
   - Compiles protobuf files
   - Detects web directory
   - Runs `npm install` in web/
   - Runs `npm run build` in web/
   - Copies built assets to `target/web-assets/`

3. **Web Assets Integration**
   - Built React app copied to `target/web-assets/`
   - Assets served by Rust backend on port 3093
   - Single-page application (SPA) routing handled

### Runtime Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    DAFS Unified System                      │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   HTTP API  │  │   gRPC API  │  │ Web Server  │         │
│  │   Port 6543 │  │  Port 50051 │  │ Port 3093   │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Storage   │  │     AI      │  │    P2P      │         │
│  │   Service   │  │   Service   │  │  Network    │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              Web Assets (target/web-assets/)           │ │
│  │  - index.html                                          │ │
│  │  - assets/ (JS, CSS, images)                          │ │
│  │  - SPA routing handled by Rust                        │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## 📦 Package Manager Integration

### Arch Linux (PKGBUILD)

The `PKGBUILD` file demonstrates how to package DAFS for Arch Linux:

```bash
# Build package
makepkg -si

# Install dependencies
sudo pacman -S nodejs npm rust cargo

# Install DAFS
sudo pacman -U dafs-0.1.0-1-x86_64.pkg.tar.zst
```

### Ubuntu/Debian (deb)

Similar packaging can be done for Debian-based systems:

```bash
# Create deb package
cargo deb

# Install
sudo dpkg -i target/debian/dafs_0.1.0_amd64.deb
```

### Systemd Service

The integrated system runs as a single systemd service:

```bash
# Enable and start service
sudo systemctl enable dafs
sudo systemctl start dafs

# Check status
sudo systemctl status dafs

# View logs
sudo journalctl -u dafs -f
```

## 🚀 Development Workflow

### Local Development

1. **Setup**
   ```bash
   git clone <repo>
   cd dafs
   cd web && npm install && cd ..
   ```

2. **Build**
   ```bash
   cargo build
   # This automatically builds the web dashboard
   ```

3. **Run**
   ```bash
   cargo run
   # Serves everything on:
   # - Web Dashboard: http://localhost:3093
   # - HTTP API: http://localhost:6543
   # - gRPC: grpc://localhost:50051
   ```

### Production Deployment

1. **Build Release**
   ```bash
   cargo build --release
   ```

2. **Install**
   ```bash
   # Copy binary
   sudo cp target/release/dafs /usr/bin/
   
   # Copy web assets
   sudo mkdir -p /usr/share/dafs/web-assets
   sudo cp -r target/web-assets/* /usr/share/dafs/web-assets/
   
   # Create service user
   sudo useradd -r -s /bin/false dafs
   sudo mkdir -p /var/lib/dafs
   sudo chown dafs:dafs /var/lib/dafs
   ```

3. **Start Service**
   ```bash
   sudo systemctl enable dafs
   sudo systemctl start dafs
   ```

## 🔍 Web Server Implementation

### Static File Serving

The web server (`src/web.rs`) implements:

1. **Asset Serving**
   ```rust
   .nest_service("/assets", ServeDir::new("target/web-assets/assets"))
   ```

2. **SPA Routing**
   ```rust
   .fallback(handle_spa)
   ```

3. **Multiple Path Support**
   - Development: `target/web-assets/`
   - Installed: `/usr/share/dafs/web-assets/`
   - Relative: `web-assets/`

### Fallback Page

If web assets aren't found, a beautiful fallback page is served with:
- System status information
- Links to API endpoints
- Instructions for building web dashboard

## 🛠️ Configuration

### Environment Variables

```bash
# Backend configuration
RUST_LOG=info
DAFS_P2P_PORT=2093
DAFS_HTTP_PORT=6543
DAFS_GRPC_PORT=50051

# Web dashboard (served by backend)
VITE_API_URL=http://localhost:6543
VITE_WEB_URL=http://localhost:3093
```

### Web Dashboard Configuration

The web dashboard (`web/src/config.ts`) is configured to:
- Make API calls to the backend on port 6543
- Be served by the backend on port 3093
- Work in both development and production modes

## 🔧 Troubleshooting

### Web Assets Not Found

If the web dashboard shows the fallback page:

1. **Check build process**
   ```bash
   cd web
   npm run build
   cd ..
   cargo build
   ```

2. **Verify assets exist**
   ```bash
   ls -la target/web-assets/
   ```

3. **Check file permissions**
   ```bash
   sudo chown -R $USER:$USER target/web-assets/
   ```

### Port Conflicts

If ports are already in use:

```bash
# Check what's using the ports
sudo netstat -tulpn | grep :3093
sudo netstat -tulpn | grep :6543

# Kill conflicting processes
sudo kill -9 <PID>
```

### Build Issues

If the build script fails:

1. **Check Node.js installation**
   ```bash
   node --version
   npm --version
   ```

2. **Clear npm cache**
   ```bash
   npm cache clean --force
   ```

3. **Reinstall dependencies**
   ```bash
   cd web
   rm -rf node_modules package-lock.json
   npm install
   cd ..
   ```

## 📊 Benefits of Integration

### For Users
- **Single installation**: One package, one binary
- **Simple management**: One service to start/stop
- **No port conflicts**: Predictable port assignments
- **Unified experience**: Everything works together

### For Developers
- **Simplified deployment**: One build process
- **Easier testing**: Single application to test
- **Better packaging**: Works with all package managers
- **Reduced complexity**: No need to manage multiple services

### For System Administrators
- **Single service**: One systemd unit
- **Unified logging**: All logs in one place
- **Simplified monitoring**: One process to monitor
- **Easier updates**: Single package to update

## 🔮 Future Enhancements

### Planned Features
- **Hot reloading**: Web dashboard updates without restart
- **Configuration UI**: Web-based configuration interface
- **Health dashboard**: Real-time system monitoring
- **Plugin system**: Extensible web dashboard

### Package Manager Support
- **AUR package**: Community-maintained Arch package
- **Homebrew**: macOS package manager support
- **Snap/Flatpak**: Universal Linux packages
- **Docker**: Containerized deployment

---

This integration makes DAFS a truly unified system that can be easily packaged, distributed, and deployed across different platforms and package managers. 