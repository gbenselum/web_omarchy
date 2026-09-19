#!/bin/bash
# web-omarchy complete installation script
# Run with: sudo ./install-complete.sh
# Note: Builds run as current user, install runs as root

set -euo pipefail

REPO_DIR="/home/gabriel/Projects/web_omarchy"
SUDO_USER="${SUDO_USER:-$(whoami)}"
USER_HOME=$(getent passwd "$SUDO_USER" | cut -d: -f6)

echo "==> Building web-omarchy from source as user: $SUDO_USER"

# Build backend
echo "==> Building Rust backend..."
cd "$REPO_DIR/backend"
CARGO_BIN="${USER_HOME}/.cargo/bin/cargo"
if [ ! -f "$CARGO_BIN" ]; then
    CARGO_BIN="$(sudo -u "$SUDO_USER" which cargo 2>/dev/null || echo "/usr/bin/cargo")"
fi
sudo -u "$SUDO_USER" "$CARGO_BIN" build --release

# Build frontend
echo "==> Building frontend..."
cd "$REPO_DIR/frontend"
sudo -u "$SUDO_USER" npm run build

# Build PAM auth helper
echo "==> Building PAM auth helper..."
gcc -o "$REPO_DIR/pam-auth" "$REPO_DIR/packaging/polkit/pam-auth.c" -lpam -lpam_misc

# Install everything (as root)
echo "==> Installing binaries and assets..."
systemctl stop web-omarchy-daemon.service 2>/dev/null || true

cp "$REPO_DIR/backend/target/release/web_omarchy_daemon" /usr/bin/web-omarchy-daemon
chmod 755 /usr/bin/web-omarchy-daemon

mkdir -p /usr/lib/web-omarchy
cp "$REPO_DIR/pam-auth" /usr/lib/web-omarchy/pam-auth
cp "$REPO_DIR/packaging/polkit/auth-helper.sh" /usr/lib/web-omarchy/auth-helper
chmod 755 /usr/lib/web-omarchy/pam-auth /usr/lib/web-omarchy/auth-helper

cp "$REPO_DIR/packaging/polkit/org.omarchy.web.policy" /usr/share/polkit-1/actions/

mkdir -p /usr/share/web-omarchy/www
cp -r "$REPO_DIR/frontend/dist/"* /usr/share/web-omarchy/www/

# Install systemd service
cp "$REPO_DIR/packaging/web-omarchy-daemon.service" /usr/lib/systemd/system/
systemctl daemon-reload

# Create system user and directories
useradd -r -s /usr/bin/nologin -d /var/empty -c "web-omarchy user" web-omarchy 2>/dev/null || true
mkdir -p /var/lib/web-omarchy/sessions
chown -R web-omarchy:web-omarchy /var/lib/web-omarchy

# Start service
echo "==> Starting service..."
systemctl enable --now web-omarchy-daemon.service

sleep 2

echo "==> Service status:"
systemctl status web-omarchy-daemon.service --no-pager

echo ""
echo "==> Installation complete!"
echo "==> Access the web interface at: http://localhost:8080"
echo "==> Login with your system username/password"
echo ""
echo "==> For remote access, ensure firewall allows port 8080:"
echo "    sudo ufw allow 8080"