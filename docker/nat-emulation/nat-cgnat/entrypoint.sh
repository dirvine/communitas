#!/bin/bash
# CGNAT (Carrier-Grade NAT) Entrypoint
# Copyright (c) 2025 Saorsa Labs Limited
#
# CGNAT (RFC 6598):
# - Shared public IP address among many subscribers
# - Limited port range per subscriber (simulating ISP port allocation)
# - Uses 100.64.0.0/10 address range between ISP and customer
#
# CGNAT is increasingly common with IPv4 exhaustion.
# Port exhaustion is a real concern - each subscriber gets limited ports.

set -e

INTERNAL_IFACE="${INTERNAL_IFACE:-eth0}"
EXTERNAL_IFACE="${EXTERNAL_IFACE:-eth1}"
PORT_RANGE_START="${PORT_RANGE_START:-32768}"
PORT_RANGE_END="${PORT_RANGE_END:-33023}"
SHARED_IP="${SHARED_IP:-}"

echo "[CGNAT] Starting configuration..."
echo "[CGNAT] Internal: $INTERNAL_IFACE, External: $EXTERNAL_IFACE"
echo "[CGNAT] Port range: $PORT_RANGE_START-$PORT_RANGE_END"

# Flush existing rules
iptables -F
iptables -t nat -F
iptables -t mangle -F

# Enable forwarding
echo 1 > /proc/sys/net/ipv4/ip_forward

# Limit local port range to simulate per-subscriber allocation
echo "$PORT_RANGE_START $PORT_RANGE_END" > /proc/sys/net/ipv4/ip_local_port_range

# Get external IP if not provided
if [ -z "$SHARED_IP" ]; then
    SHARED_IP=$(ip addr show "$EXTERNAL_IFACE" | grep "inet " | awk '{print $2}' | cut -d/ -f1)
fi

echo "[CGNAT] External IP: $SHARED_IP"

# NAT: SNAT with limited port range (simulates per-subscriber allocation)
iptables -t nat -A POSTROUTING -o "$EXTERNAL_IFACE" \
    -j SNAT --to-source "$SHARED_IP:$PORT_RANGE_START-$PORT_RANGE_END"

# Forward: Allow outgoing
iptables -A FORWARD -i "$INTERNAL_IFACE" -o "$EXTERNAL_IFACE" -j ACCEPT

# Forward: Only allow established
iptables -A FORWARD -i "$EXTERNAL_IFACE" -o "$INTERNAL_IFACE" \
    -m state --state ESTABLISHED,RELATED -j ACCEPT

# Drop everything else
iptables -A FORWARD -i "$EXTERNAL_IFACE" -o "$INTERNAL_IFACE" -j DROP

# Limit conntrack entries (simulates ISP limits)
echo 1024 > /proc/sys/net/netfilter/nf_conntrack_max 2>/dev/null || true

echo "[CGNAT] Configuration complete"
echo "[CGNAT] Port range: $PORT_RANGE_START-$PORT_RANGE_END ($(( PORT_RANGE_END - PORT_RANGE_START + 1 )) ports)"
echo "[CGNAT] Rules:"
iptables -L -n -v
echo ""
iptables -t nat -L -n -v

exec "$@"
