#!/bin/bash
# Symmetric NAT Entrypoint
# Copyright (c) 2025 Saorsa Labs Limited
#
# Symmetric NAT (RFC 3489):
# - Address and Port Dependent Mapping (APDM): different port per destination
# - Address and Port Dependent Filtering (APDF): only accept from exact IP:port
#
# The --random flag causes iptables to use different ports for each connection,
# making it behave like symmetric NAT.
#
# This is the HARDEST NAT type to traverse. Hole-punching between two symmetric
# NATs typically requires relay or port prediction (unreliable).

set -e

INTERNAL_IFACE="${INTERNAL_IFACE:-eth0}"
EXTERNAL_IFACE="${EXTERNAL_IFACE:-eth1}"

echo "[Symmetric NAT] Starting configuration..."
echo "[Symmetric NAT] Internal: $INTERNAL_IFACE, External: $EXTERNAL_IFACE"

# Flush existing rules
iptables -F
iptables -t nat -F
iptables -t mangle -F

# Enable forwarding
echo 1 > /proc/sys/net/ipv4/ip_forward

# NAT: Masquerade with --random to randomize source ports
# This is the key for symmetric NAT behavior
# --random-fully provides even more randomization (kernel 5.0+)
if iptables -t nat -A POSTROUTING -o "$EXTERNAL_IFACE" -j MASQUERADE --random-fully 2>/dev/null; then
    echo "[Symmetric NAT] Using --random-fully"
else
    iptables -t nat -A POSTROUTING -o "$EXTERNAL_IFACE" -j MASQUERADE --random
    echo "[Symmetric NAT] Using --random"
fi

# Forward: Allow outgoing
iptables -A FORWARD -i "$INTERNAL_IFACE" -o "$EXTERNAL_IFACE" -j ACCEPT

# Forward: Only allow incoming from established connections
iptables -A FORWARD -i "$EXTERNAL_IFACE" -o "$INTERNAL_IFACE" \
    -m state --state ESTABLISHED,RELATED -j ACCEPT

# Drop everything else
iptables -A FORWARD -i "$EXTERNAL_IFACE" -o "$INTERNAL_IFACE" -j DROP

echo "[Symmetric NAT] Configuration complete"
echo "[Symmetric NAT] Rules:"
iptables -L -n -v
echo ""
iptables -t nat -L -n -v

exec "$@"
