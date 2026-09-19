#!/bin/bash
# web-omarchy auth helper - runs via pkexec for PAM authentication
# Usage: auth-helper <username> <password>

set -euo pipefail

USERNAME="${1:-}"
PASSWORD="${2:-}"

if [[ -z "$USERNAME" || -z "$PASSWORD" ]]; then
    echo "Usage: $0 <username> <password>"
    exit 1
fi

# Use PAM to authenticate
exec /usr/lib/web-omarchy/pam-auth "$USERNAME" "$PASSWORD"