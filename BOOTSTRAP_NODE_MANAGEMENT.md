# DAFS Bootstrap Node Remote Management Guide

Complete guide for setting up, deploying, and remotely managing DAFS bootstrap nodes.

## Table of Contents

1. [Overview](#overview)
2. [Bootstrap Node Setup](#bootstrap-node-setup)
3. [Service Installation](#service-installation)
4. [Remote Management](#remote-management)
5. [Security Considerations](#security-considerations)
6. [Monitoring and Maintenance](#monitoring-and-maintenance)
7. [Troubleshooting](#troubleshooting)

## Overview

A DAFS bootstrap node is a dedicated server that:
- Runs DAFS as a system service
- Provides peer discovery for the network
- Accepts remote management connections
- Maintains persistent peer storage
- Serves as a reliable entry point for the P2P network

## Bootstrap Node Setup

### Prerequisites

```bash
# System requirements
- Linux/Unix system (Ubuntu 20.04+ recommended)
- 2+ CPU cores
- 4GB+ RAM
- 50GB+ storage
- Static IP address
- Open ports: 2093 (P2P), 2094 (Admin), 6543 (HTTP API), 50051 (gRPC)

# Install dependencies
sudo apt update
sudo apt install -y curl build-essential pkg-config libssl-dev
```

### Installation

```bash
# 1. Clone and build DAFS
git clone https://github.com/Kyle6012/dafs.git
cd dafs
cargo build --release

# 2. Create service user
sudo useradd -r -s /bin/false dafs
sudo mkdir -p /opt/dafs
sudo chown dafs:dafs /opt/dafs

# 3. Copy binary and create directories
sudo cp target/release/dafs /opt/dafs/
sudo mkdir -p /opt/dafs/{data,backups,logs,config}
sudo chown -R dafs:dafs /opt/dafs
```

## Service Installation

### Systemd Service

Create `/etc/systemd/system/dafs-bootstrap.service`:

```ini
[Unit]
Description=DAFS Bootstrap Node Service
After=network.target
Wants=network.target

[Service]
Type=simple
User=dafs
Group=dafs
WorkingDirectory=/opt/dafs
ExecStart=/opt/dafs/dafs --bootstrap --admin-port 2094 --config /opt/dafs/config/service.toml
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=dafs-bootstrap

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/dafs/data /opt/dafs/backups /opt/dafs/logs

[Install]
WantedBy=multi-user.target
```

### Configuration File

Create `/opt/dafs/config/service.toml`:

```toml
[service]
name = "dafs-bootstrap"
version = "1.0.0"
auto_start = true
log_level = "info"

[network]
p2p_port = 2093
admin_port = 2094
http_port = 6543
grpc_port = 50051

[admin]
username = "admin"
password = "secure_password_here"
allowed_ips = ["127.0.0.1", "::1", "YOUR_MANAGEMENT_IP"]

[storage]
data_dir = "/opt/dafs/data"
backup_dir = "/opt/dafs/backups"
log_dir = "/opt/dafs/logs"

[security]
max_connections = 100
session_timeout = 3600
rate_limit = 100

[bootstrap]
auto_discover = true
persistent_peers = true
backup_interval = 86400  # 24 hours
```

### Enable and Start Service

```bash
# Enable and start the service
sudo systemctl daemon-reload
sudo systemctl enable dafs-bootstrap
sudo systemctl start dafs-bootstrap

# Check status
sudo systemctl status dafs-bootstrap
sudo journalctl -u dafs-bootstrap -f
```

## Remote Management

### From User Device

Once the bootstrap node is running, you can manage it remotely from your user device:

#### 1. Connect to Remote Service

```bash
# Connect to bootstrap node
dafs remoteconnect 192.168.1.100 2094 admin secure_password_here

# Or using interactive CLI
dafs --cli
dafs(guest)> remoteconnect 192.168.1.100 2094 admin secure_password_here
```

#### 2. Check Service Status

```bash
# Get service status
dafs remotestatus

# Or using interactive CLI
dafs(guest)> remotestatus

# Output example:
# Service ID: dafs-bootstrap-12345
# Status: Running
# Uptime: 86400 seconds (24 hours)
# Version: 1.0.0
# Peer Count: 15
# File Count: 234
# Memory Usage: 512MB
# CPU Usage: 2.5%
# Disk Usage: 15GB
```

#### 3. Manage Bootstrap Nodes

```bash
# Add bootstrap node
dafs remotebootstrap add QmBootstrap1 /ip4/1.2.3.4/tcp/2093

# Remove bootstrap node
dafs remotebootstrap remove QmBootstrap1

# List bootstrap nodes
dafs remotebootstrap list

# Or using interactive CLI
dafs(guest)> remotebootstrap add QmBootstrap1 /ip4/1.2.3.4/tcp/2093
dafs(guest)> remotebootstrap list
```

#### 4. Service Control

```bash
# Restart service
dafs remoterestart

# Stop service
dafs remotestop

# Start service
dafs remotestart

# Or using interactive CLI
dafs(guest)> remoterestart
dafs(guest)> remotestop
dafs(guest)> remotestart
```

#### 5. Configuration Management

```bash
# Update configuration
dafs remoteconfig log_level debug
dafs remoteconfig max_connections 200

# Get configuration
dafs remoteconfigget log_level
dafs remoteconfigget  # List all config

# Or using interactive CLI
dafs(guest)> remoteconfig log_level debug
dafs(guest)> remoteconfigget log_level
```

#### 6. Logs and Monitoring

```bash
# View logs
dafs remotelogs 50  # Last 50 lines
dafs remotelogs 100 # Last 100 lines

# Execute custom commands
dafs remoteexec "peers list"
dafs remoteexec "files count"
dafs remoteexec "system info"

# Or using interactive CLI
dafs(guest)> remotelogs 50
dafs(guest)> remoteexec "peers list"
```

#### 7. Backup and Restore

```bash
# Create backup
dafs remotebackup /opt/dafs/backups/backup_$(date +%Y%m%d).tar.gz

# Restore from backup
dafs remoterestore /opt/dafs/backups/backup_20240115.tar.gz

# Or using interactive CLI
dafs(guest)> remotebackup /opt/dafs/backups/backup_$(date +%Y%m%d).tar.gz
dafs(guest)> remoterestore /opt/dafs/backups/backup_20240115.tar.gz
```

### Interactive Remote Shell

```bash
# Start interactive remote management shell
dafs remoteshell

# Or using interactive CLI
dafs --cli
dafs(guest)> remoteshell

# In the remote shell:
# > status
# > peers list
# > config get log_level
```

## Security Considerations

### 1. Network Security

```bash
# Firewall configuration (UFW)
sudo ufw allow 2093/tcp  # P2P port
sudo ufw allow 2094/tcp  # Admin port (restrict to management IPs)
sudo ufw allow 6543/tcp  # HTTP API (optional)
sudo ufw allow 50051/tcp # gRPC (optional)

# For production, restrict admin port to specific IPs
sudo ufw allow from YOUR_MANAGEMENT_IP to any port 2094
```

### 2. Authentication

```bash
# Use strong passwords
# Generate secure password
openssl rand -base64 32

# Update configuration
dafs remoteconfig admin_password generated_secure_password
```

### 3. SSL/TLS (Optional)

```bash
# For production, enable SSL/TLS
# Generate certificates
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

# Update configuration
dafs remoteconfig ssl_enabled true
dafs remoteconfig ssl_cert /opt/dafs/certs/cert.pem
dafs remoteconfig ssl_key /opt/dafs/certs/key.pem
```

### 4. Access Control

```bash
# Restrict allowed IPs
dafs remoteconfig allowed_ips 192.168.1.50,192.168.1.51

# Set rate limiting
dafs remoteconfig rate_limit 50

# Session timeout
dafs remoteconfig session_timeout 1800  # 30 minutes
```

## Monitoring and Maintenance

### 1. Automated Monitoring

Create `/opt/dafs/scripts/monitor.sh`:

```bash
#!/bin/bash
# Monitor script for DAFS bootstrap node

SERVICE_NAME="dafs-bootstrap"
LOG_FILE="/opt/dafs/logs/monitor.log"
ALERT_EMAIL="admin@example.com"

# Check if service is running
if ! systemctl is-active --quiet $SERVICE_NAME; then
    echo "$(date): Service $SERVICE_NAME is down, restarting..." >> $LOG_FILE
    systemctl restart $SERVICE_NAME
    
    # Send alert
    echo "DAFS Bootstrap service was down and has been restarted" | mail -s "DAFS Alert" $ALERT_EMAIL
fi

# Check disk usage
DISK_USAGE=$(df /opt/dafs | tail -1 | awk '{print $5}' | sed 's/%//')
if [ $DISK_USAGE -gt 80 ]; then
    echo "$(date): Disk usage is ${DISK_USAGE}%" >> $LOG_FILE
    echo "DAFS Bootstrap disk usage is ${DISK_USAGE}%" | mail -s "DAFS Disk Alert" $ALERT_EMAIL
fi

# Check memory usage
MEMORY_USAGE=$(free | grep Mem | awk '{printf("%.0f", $3/$2 * 100.0)}')
if [ $MEMORY_USAGE -gt 80 ]; then
    echo "$(date): Memory usage is ${MEMORY_USAGE}%" >> $LOG_FILE
    echo "DAFS Bootstrap memory usage is ${MEMORY_USAGE}%" | mail -s "DAFS Memory Alert" $ALERT_EMAIL
fi
```

### 2. Cron Job for Monitoring

```bash
# Add to crontab
sudo crontab -e

# Add this line for monitoring every 5 minutes
*/5 * * * * /opt/dafs/scripts/monitor.sh
```

### 3. Automated Backups

Create `/opt/dafs/scripts/backup.sh`:

```bash
#!/bin/bash
# Backup script for DAFS bootstrap node

BACKUP_DIR="/opt/dafs/backups"
DATA_DIR="/opt/dafs/data"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="dafs_backup_${DATE}.tar.gz"

# Create backup
tar -czf "${BACKUP_DIR}/${BACKUP_FILE}" -C /opt/dafs data config

# Keep only last 7 days of backups
find $BACKUP_DIR -name "dafs_backup_*.tar.gz" -mtime +7 -delete

# Log backup
echo "$(date): Backup created: ${BACKUP_FILE}" >> /opt/dafs/logs/backup.log
```

### 4. Log Rotation

Create `/etc/logrotate.d/dafs-bootstrap`:

```
/opt/dafs/logs/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 dafs dafs
    postrotate
        systemctl reload dafs-bootstrap
    endscript
}
```

## Troubleshooting

### Common Issues

#### 1. Service Won't Start

```bash
# Check service status
sudo systemctl status dafs-bootstrap

# Check logs
sudo journalctl -u dafs-bootstrap -n 50

# Check permissions
sudo chown -R dafs:dafs /opt/dafs

# Check configuration
dafs --config /opt/dafs/config/service.toml --validate
```

#### 2. Connection Refused

```bash
# Check if service is listening
sudo netstat -tlnp | grep 2094

# Check firewall
sudo ufw status

# Check allowed IPs in config
dafs remoteconfigget allowed_ips
```

#### 3. Authentication Failed

```bash
# Reset admin password
sudo systemctl stop dafs-bootstrap
# Edit config file manually
sudo nano /opt/dafs/config/service.toml
sudo systemctl start dafs-bootstrap

# Or use recovery mode
dafs --recovery-mode --reset-admin-password
```

#### 4. High Resource Usage

```bash
# Check resource usage
top -p $(pgrep dafs)

# Check peer connections
dafs remoteexec "peers list"

# Reduce max connections
dafs remoteconfig max_connections 50
```

### Debug Mode

```bash
# Enable debug logging
dafs remoteconfig log_level debug
dafs remoterestart

# View debug logs
dafs remotelogs 100
```

### Recovery Procedures

#### 1. Service Recovery

```bash
# If service is completely down
sudo systemctl stop dafs-bootstrap
sudo rm -f /opt/dafs/data/lockfile
sudo systemctl start dafs-bootstrap
```

#### 2. Data Recovery

```bash
# Restore from backup
dafs remoterestore "/opt/dafs/backups/dafs_backup_20240115_120000.tar.gz"

# Or manually restore
sudo systemctl stop dafs-bootstrap
sudo tar -xzf backup_file.tar.gz -C /opt/dafs
sudo chown -R dafs:dafs /opt/dafs
sudo systemctl start dafs-bootstrap
```

#### 3. Configuration Recovery

```bash
# Reset to default configuration
sudo cp /opt/dafs/config/service.toml.default /opt/dafs/config/service.toml
sudo systemctl restart dafs-bootstrap
```

## Best Practices

### 1. Deployment

- Use dedicated servers for bootstrap nodes
- Implement proper monitoring and alerting
- Regular backups and testing
- Document all configuration changes

### 2. Security

- Use strong authentication
- Restrict network access
- Regular security updates
- Monitor access logs

### 3. Performance

- Monitor resource usage
- Optimize configuration for your network
- Regular maintenance and cleanup
- Scale horizontally if needed

### 4. Reliability

- Use redundant bootstrap nodes
- Implement automatic failover
- Regular health checks
- Disaster recovery planning

## Example Deployment Script

Create `deploy_bootstrap.sh`:

```bash
#!/bin/bash
# Automated bootstrap node deployment script

set -e

BOOTSTRAP_IP="192.168.1.100"
ADMIN_USER="admin"
ADMIN_PASSWORD=$(openssl rand -base64 32)

echo "Deploying DAFS bootstrap node to $BOOTSTRAP_IP..."

# Build and copy binary
cargo build --release
scp target/release/dafs root@$BOOTSTRAP_IP:/opt/dafs/

# Copy configuration
scp config/service.toml root@$BOOTSTRAP_IP:/opt/dafs/config/
scp systemd/dafs-bootstrap.service root@$BOOTSTRAP_IP:/etc/systemd/system/

# Setup on remote server
ssh root@$BOOTSTRAP_IP << EOF
    # Create user and directories
    useradd -r -s /bin/false dafs || true
    mkdir -p /opt/dafs/{data,backups,logs,config}
    chown -R dafs:dafs /opt/dafs
    
    # Update configuration
    sed -i "s/admin_password = \".*\"/admin_password = \"$ADMIN_PASSWORD\"/" /opt/dafs/config/service.toml
    
    # Enable and start service
    systemctl daemon-reload
    systemctl enable dafs-bootstrap
    systemctl start dafs-bootstrap
    
    # Setup firewall
    ufw allow 2093/tcp
    ufw allow 2094/tcp
    ufw allow from $(curl -s ifconfig.me) to any port 2094
EOF

echo "Bootstrap node deployed successfully!"
echo "Admin password: $ADMIN_PASSWORD"
echo "Connect with: dafs remoteconnect $BOOTSTRAP_IP 2094 admin '$ADMIN_PASSWORD'"
```

This comprehensive guide provides everything needed to set up, deploy, and manage DAFS bootstrap nodes remotely, ensuring reliable operation of your decentralized file system network. 