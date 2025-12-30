#!/bin/bash
# Full Cone NAT Entrypoint
# Copyright (c) 2025 Saorsa Labs Limited
#
# Full Cone NAT (RFC 3489):
# - Endpoint Independent Mapping (EIM): same external port for all destinations
# - Endpoint Independent Filtering (EIF): accepts packets from ANY external host
#
# This is the EASIEST NAT type to traverse. Once a port is mapped, any external
# host can reach the internal client through that mapping.
# Common in gaming routers and UPnP-enabled devices.

set -e

INTERNAL_IFACE="${INTERNAL_IFACE:-eth0}"
EXTERNAL_IFACE="${EXTERNAL_IFACE:-eth1}"

echo "[Full Cone NAT] Starting configuration..."
echo "[Full Cone NAT] Internal: $INTERNAL_IFACE, External: $EXTERNAL_IFACE"

# Flush existing rules
iptables -F
iptables -t nat -F
iptables -t mangle -F

# Enable forwarding
echo 1 > /proc/sys/net/ipv4/ip_forward

# NAT: Simple masquerade (keeps consistent port mapping)
iptables -t nat -A POSTROUTING -o "$EXTERNAL_IFACE" -j MASQUERADE

# Forward: Allow ALL traffic in both directions
# This is the key for Full Cone - allow ANY incoming once outbound exists
iptables -A FORWARD -i "$INTERNAL_IFACE" -o "$EXTERNAL_IFACE" -j ACCEPT
iptables -A FORWARD -i "$EXTERNAL_IFACE" -o "$INTERNAL_IFACE" -j ACCEPT

echo "[Full Cone NAT] Configuration complete"
echo "[Full Cone NAT] Rules:"
iptables -L -n -v
echo ""
iptables -t nat -L -n -v

exec "$@"
