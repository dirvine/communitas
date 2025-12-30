#!/bin/bash
# Address-Restricted Cone NAT Entrypoint
# Copyright (c) 2025 Saorsa Labs Limited
#
# Address-Restricted Cone NAT (RFC 3489):
# - Endpoint Independent Mapping (EIM): same external port for all destinations
# - Address Dependent Filtering (ADF): only accepts packets from IPs we've sent to
#
# Hole-punching works if both sides send packets to each other first.
# Common in older home routers and some enterprise firewalls.

set -e

INTERNAL_IFACE="${INTERNAL_IFACE:-eth0}"
EXTERNAL_IFACE="${EXTERNAL_IFACE:-eth1}"

echo "[Address-Restricted NAT] Starting configuration..."
echo "[Address-Restricted NAT] Internal: $INTERNAL_IFACE, External: $EXTERNAL_IFACE"

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

# Forward: Only allow incoming from ESTABLISHED connections (IP-based)
# This implements address-dependent filtering
iptables -A FORWARD -i "$EXTERNAL_IFACE" -o "$INTERNAL_IFACE" \
    -m state --state ESTABLISHED,RELATED -j ACCEPT

# Drop everything else incoming
iptables -A FORWARD -i "$EXTERNAL_IFACE" -o "$INTERNAL_IFACE" -j DROP

echo "[Address-Restricted NAT] Configuration complete"
echo "[Address-Restricted NAT] Rules:"
iptables -L -n -v
echo ""
iptables -t nat -L -n -v

exec "$@"
