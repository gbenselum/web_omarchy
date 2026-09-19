#!/bin/bash
# web-omarchy installation script
# Run with: sudo ./install.sh

set -euo pipefail

echo "==> Installing web-omarchy daemon..."

# Install backend binary
echo "==> Installing backend binary..."
cp /home/gabriel/Projects/web_omarchy/backend/target/release/web_omarchy_daemon /usr/bin/web-omarchy-daemon
chmod 755 /usr/bin/web-omarchy-daemon

# Install frontend assets
echo "==> Installing frontend assets..."
mkdir -p /usr/share/web-omarchy/www
cp -r /home/gabriel/Projects/web_omarchy/frontend/dist/* /usr/share/web-omarchy/www/

# Install systemd service
echo "==> Installing systemd service..."
cp /home/gabriel/Projects/web_omarchy/packaging/web-omarchy-daemon.service /usr/lib/systemd/system/

# Create system user and directories
echo "==> Creating system user and directories..."
useradd -r -s /usr/bin/nologin -d /var/empty -c "web-omarchy user" web-omarchy 2>/dev/null || true
mkdir -p /var/lib/web-omarchy/sessions
chown -R web-omarchy:web-omarchy /var/lib/web-omarchy

# Reload and start
echo "==> Starting service..."
systemctl daemon-reload
systemctl enable --now web-omarchy-daemon.service

# Check status
echo "==> Service status:"
systemctl status web-omarchy-daemon.service --no-pager

echo ""
echo "==> Installation complete!"
echo "==> Access the web interface at: http://localhost:8080"
echo "==> Login with your system username/password"