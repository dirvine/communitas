#!/bin/bash
# Setup TLS certificates for Communitas MCP Server
# Uses Let's Encrypt with certbot

set -e

# Configuration
DOMAIN="${1:-mcp.saorsalabs.com}"
EMAIL="${2:-admin@saorsalabs.com}"
CERT_DIR="/etc/letsencrypt/live/${DOMAIN}"

echo "=== Communitas MCP TLS Setup ==="
echo "Domain: ${DOMAIN}"
echo "Email: ${EMAIL}"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Error: Please run as root"
    exit 1
fi

# Install certbot if not present
if ! command -v certbot &> /dev/null; then
    echo "Installing certbot..."
    if command -v apt &> /dev/null; then
        apt update && apt install -y certbot
    elif command -v dnf &> /dev/null; then
        dnf install -y certbot
    else
        echo "Error: Unable to install certbot. Please install manually."
        exit 1
    fi
fi

# Request certificate
echo "Requesting certificate from Let's Encrypt..."
certbot certonly \
    --standalone \
    --non-interactive \
    --agree-tos \
    --email "${EMAIL}" \
    --domain "${DOMAIN}" \
    --preferred-challenges http

# Verify certificate exists
if [ ! -f "${CERT_DIR}/fullchain.pem" ]; then
    echo "Error: Certificate not found at ${CERT_DIR}/fullchain.pem"
    exit 1
fi

# Set permissions for communitas user
echo "Setting certificate permissions..."
chmod 755 /etc/letsencrypt/live
chmod 755 /etc/letsencrypt/archive
chmod 644 "${CERT_DIR}/fullchain.pem"
chmod 640 "${CERT_DIR}/privkey.pem"
chgrp communitas "${CERT_DIR}/privkey.pem"

# Create symbolic links in MCP directory
echo "Creating certificate links..."
MCP_DIR="/opt/communitas-mcp"
mkdir -p "${MCP_DIR}/certs"
ln -sf "${CERT_DIR}/fullchain.pem" "${MCP_DIR}/certs/cert.pem"
ln -sf "${CERT_DIR}/privkey.pem" "${MCP_DIR}/certs/key.pem"

# Setup auto-renewal
echo "Configuring auto-renewal..."
cat > /etc/systemd/system/certbot-renewal.service << 'EOF'
[Unit]
Description=Certbot Renewal
After=network-online.target

[Service]
Type=oneshot
ExecStart=/usr/bin/certbot renew --quiet
ExecStartPost=/bin/systemctl reload communitas-mcp
EOF

cat > /etc/systemd/system/certbot-renewal.timer << 'EOF'
[Unit]
Description=Run certbot renewal twice daily

[Timer]
OnCalendar=*-*-* 00,12:00:00
RandomizedDelaySec=1h
Persistent=true

[Install]
WantedBy=timers.target
EOF

systemctl daemon-reload
systemctl enable certbot-renewal.timer
systemctl start certbot-renewal.timer

echo ""
echo "=== TLS Setup Complete ==="
echo "Certificate: ${CERT_DIR}/fullchain.pem"
echo "Private key: ${CERT_DIR}/privkey.pem"
echo "MCP cert link: ${MCP_DIR}/certs/cert.pem"
echo "MCP key link: ${MCP_DIR}/certs/key.pem"
echo ""
echo "Auto-renewal configured via certbot-renewal.timer"
echo ""
echo "Update communitas-mcp.service to use these certs if needed."
