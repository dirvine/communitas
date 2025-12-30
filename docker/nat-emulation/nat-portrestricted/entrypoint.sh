#!/bin/bash
# Port-Restricted Cone NAT Entrypoint
# Copyright (c) 2025 Saorsa Labs Limited
#
# Port-Restricted Cone NAT (RFC 3489):
# - Endpoint Independent Mapping (EIM): same external port for all destinations
# - Address and Port Dependent Filtering (APDF): only accepts from exact IP:port
#
# Hole-punching requires BOTH sides to send to exact IP:port pairs simultaneously.
# This is the default behavior of most modern home routers.

set -e

INTERNAL_IFACE="${INTERNAL_IFACE:-eth0}"
EXTERNAL_IFACE="${EXTERNAL_IFACE:-eth1}"

echo "[Port-Restricted NAT] Starting configuration..."
echo "[Port-Restricted NAT] Internal: $INTERNAL_IFACE, External: $EXTERNAL_IFACE"

# Flush existing rules
iptables -F
iptables -t nat -F
iptables -t mangle -F

# Enable forwarding
echo 1 > /proc/sys/net/ipv4/ip_forward

# NAT: Simple masquerade
iptables -t nat -A POSTROUTING -o "$EXTERNAL_IFACE" -j MASQUERADE

# Forward: Allow outgoing
iptables -A FORWARD -i "$INTERNAL_IFACE" -o "$EXTERNAL_IFACE" -j ACCEPT

# Forward: Only allow incoming from established (IP:port specific)
# Linux conntrack naturally does port-restricted filtering for UDP
iptables -A FORWARD -i "$EXTERNAL_IFACE" -o "$INTERNAL_IFACE" \
    -m state --state ESTABLISHED,RELATED -j ACCEPT

# Drop everything else
iptables -A FORWARD -i "$EXTERNAL_IFACE" -o "$INTERNAL_IFACE" -j DROP

echo "[Port-Restricted NAT] Configuration complete"
echo "[Port-Restricted NAT] Rules:"
iptables -L -n -v
echo ""
iptables -t nat -L -n -v

exec "$@"
