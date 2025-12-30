#!/bin/bash
# Hairpin NAT Entrypoint
# Copyright (c) 2025 Saorsa Labs Limited
#
# Hairpin NAT (NAT Loopback/Reflection):
# - Allows internal hosts to reach their own external IP from inside
# - Useful for testing self-connectivity scenarios
#
# Many home routers do NOT support hairpin NAT, causing issues when
# internal hosts try to reach services via the public IP.

set -e

INTERNAL_IFACE="${INTERNAL_IFACE:-eth0}"
EXTERNAL_IFACE="${EXTERNAL_IFACE:-eth1}"
EXTERNAL_IP="${EXTERNAL_IP:-}"

echo "[Hairpin NAT] Starting configuration..."
echo "[Hairpin NAT] Internal: $INTERNAL_IFACE, External: $EXTERNAL_IFACE"

# Get external IP if not provided
if [ -z "$EXTERNAL_IP" ]; then
    EXTERNAL_IP=$(ip addr show "$EXTERNAL_IFACE" | grep "inet " | awk '{print $2}' | cut -d/ -f1)
fi

echo "[Hairpin NAT] External IP: $EXTERNAL_IP"

# Flush existing rules
iptables -F
iptables -t nat -F
iptables -t mangle -F

# Enable forwarding
echo 1 > /proc/sys/net/ipv4/ip_forward

# NAT: Standard masquerade for outbound
iptables -t nat -A POSTROUTING -o "$EXTERNAL_IFACE" -j MASQUERADE

# Hairpin: Also masquerade traffic going back to internal network
# This is the key for hairpin - rewrite source when looping back
iptables -t nat -A POSTROUTING -o "$INTERNAL_IFACE" -s 10.0.0.0/8 -d 10.0.0.0/8 -j MASQUERADE

# Forward: Allow all directions for hairpin
iptables -A FORWARD -i "$INTERNAL_IFACE" -o "$EXTERNAL_IFACE" -j ACCEPT
iptables -A FORWARD -i "$EXTERNAL_IFACE" -o "$INTERNAL_IFACE" -j ACCEPT
iptables -A FORWARD -i "$INTERNAL_IFACE" -o "$INTERNAL_IFACE" -j ACCEPT

echo "[Hairpin NAT] Configuration complete"
echo "[Hairpin NAT] Hairpin enabled for traffic to $EXTERNAL_IP"
echo "[Hairpin NAT] Rules:"
iptables -L -n -v
echo ""
iptables -t nat -L -n -v

exec "$@"
