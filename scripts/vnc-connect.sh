#!/bin/bash
# Copyright (c) 2025 Saorsa Labs Limited
# VNC Connection Helper for Communitas Testing
#
# Usage:
#   ./scripts/vnc-connect.sh <node>           # Connect to VNC on node
#   ./scripts/vnc-connect.sh list             # List available VNC nodes
#   ./scripts/vnc-connect.sh install <node>   # Install VNC server on node
#   ./scripts/vnc-connect.sh status           # Show VNC status on all nodes

set -eo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
VNC_PORT=5901
VNC_LOCAL_PORT=5901
SSH_OPTS="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=5"

# VPS nodes with VNC capability (test nodes only, not bootstrap) - bash 3.2 compatible
VNC_NODES="saorsa-4 saorsa-5 saorsa-6 saorsa-7 saorsa-8 saorsa-9"

# Get node info (bash 3.2 compatible - no associative arrays)
get_node_info() {
    local node=$1
    case $node in
        saorsa-4) echo "206.189.7.117:DigitalOcean:AMS" ;;
        saorsa-5) echo "144.126.230.161:DigitalOcean:LON" ;;
        saorsa-6) echo "65.21.157.229:Hetzner:Helsinki" ;;
        saorsa-7) echo "116.203.101.172:Hetzner:Nuremberg" ;;
        saorsa-8) echo "149.28.156.231:Vultr:Singapore" ;;
        saorsa-9) echo "45.77.176.184:Vultr:Tokyo" ;;
        *) echo "" ;;
    esac
}

# Check if node exists
node_exists() {
    local node=$1
    [[ -n "$(get_node_info $node)" ]]
}

get_ip() { get_node_info $1 | cut -d: -f1; }
get_provider() { get_node_info $1 | cut -d: -f2; }
get_location() { get_node_info $1 | cut -d: -f3; }

print_status() {
    local status=$1
    local message=$2
    case $status in
        "ok")     echo -e "${GREEN}✓${NC} $message" ;;
        "fail")   echo -e "${RED}✗${NC} $message" ;;
        "warn")   echo -e "${YELLOW}!${NC} $message" ;;
        "info")   echo -e "${BLUE}→${NC} $message" ;;
    esac
}

check_ssh() {
    local node=$1
    local ip=$(get_ip $node)
    timeout 5 ssh $SSH_OPTS root@$ip "echo ok" &>/dev/null
}

check_vnc() {
    local node=$1
    local ip=$(get_ip $node)
    ssh $SSH_OPTS root@$ip "systemctl is-active vncserver@:1 2>/dev/null || pgrep -f Xvnc" &>/dev/null
}

# List available VNC nodes
cmd_list() {
    echo -e "${BLUE}Available VNC Nodes${NC}"
    echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    printf "%-12s %-16s %-12s %-12s %s\n" "NODE" "IP" "PROVIDER" "LOCATION" "VNC"
    echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    for node in $VNC_NODES; do
        local ip=$(get_ip $node)
        local provider=$(get_provider $node)
        local location=$(get_location $node)

        local vnc_status="${RED}not installed${NC}"
        if check_ssh $node; then
            if check_vnc $node; then
                vnc_status="${GREEN}running${NC}"
            else
                vnc_status="${YELLOW}stopped${NC}"
            fi
        else
            vnc_status="${RED}unreachable${NC}"
        fi

        printf "%-12s %-16s %-12s %-12s %b\n" "$node" "$ip" "$provider" "$location" "$vnc_status"
    done | sort
}

# Install VNC server on a node
cmd_install() {
    local node=${1:-}

    if [[ -z "$node" ]]; then
        echo "Usage: $0 install <node>"
        echo "Available: $VNC_NODES"
        exit 1
    fi

    if ! node_exists $node; then
        echo "Unknown node: $node"
        exit 1
    fi

    local ip=$(get_ip $node)
    print_status "info" "Installing VNC server on $node ($ip)..."

    # Detect package manager and install
    ssh $SSH_OPTS root@$ip << 'INSTALL_SCRIPT'
set -e

# Detect OS
if command -v apt-get &>/dev/null; then
    # Debian/Ubuntu
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y tigervnc-standalone-server tigervnc-common dbus-x11 xfce4 xfce4-goodies

    # Create VNC systemd service
    cat > /etc/systemd/system/vncserver@.service << 'EOF'
[Unit]
Description=Remote desktop service (VNC)
After=syslog.target network.target

[Service]
Type=simple
User=root
PAMName=login
PIDFile=/root/.vnc/%H%i.pid
ExecStartPre=/bin/sh -c '/usr/bin/vncserver -kill :%i > /dev/null 2>&1 || :'
ExecStart=/usr/bin/vncserver :%i -geometry 1280x800 -depth 24 -localhost no
ExecStop=/usr/bin/vncserver -kill :%i

[Install]
WantedBy=multi-user.target
EOF

elif command -v dnf &>/dev/null; then
    # Fedora/RHEL
    dnf install -y tigervnc-server xfce4-session xfwm4 xfce4-panel

elif command -v yum &>/dev/null; then
    # CentOS/older RHEL
    yum install -y tigervnc-server xfce4-session
fi

# Create VNC config directory
mkdir -p /root/.vnc

# Set default VNC password (change in production!)
echo "communitas" | vncpasswd -f > /root/.vnc/passwd
chmod 600 /root/.vnc/passwd

# Create xstartup
cat > /root/.vnc/xstartup << 'EOF'
#!/bin/sh
unset SESSION_MANAGER
unset DBUS_SESSION_BUS_ADDRESS
exec startxfce4
EOF
chmod +x /root/.vnc/xstartup

# Enable and start
systemctl daemon-reload
systemctl enable vncserver@:1
systemctl start vncserver@:1

echo "VNC server installed and started on display :1"
INSTALL_SCRIPT

    if [[ $? -eq 0 ]]; then
        print_status "ok" "VNC server installed on $node"
        print_status "info" "Default password: communitas (change for production!)"
        print_status "info" "Connect with: $0 $node"
    else
        print_status "fail" "Failed to install VNC on $node"
    fi
}

# Connect to VNC via SSH tunnel
cmd_connect() {
    local node=${1:-}

    if [[ -z "$node" ]]; then
        echo "Usage: $0 <node>"
        echo "Available: $VNC_NODES"
        exit 1
    fi

    if ! node_exists $node; then
        echo "Unknown node: $node"
        exit 1
    fi

    local ip=$(get_ip $node)
    local location=$(get_location $node)

    print_status "info" "Connecting to $node ($location)..."

    # Check if VNC is running
    if ! check_vnc $node; then
        print_status "warn" "VNC not running on $node, starting..."
        ssh $SSH_OPTS root@$ip "systemctl start vncserver@:1 || vncserver :1"
    fi

    # Find available local port
    local local_port=$VNC_LOCAL_PORT
    while nc -z localhost $local_port 2>/dev/null; do
        ((local_port++))
    done

    print_status "info" "Creating SSH tunnel localhost:$local_port -> $ip:$VNC_PORT"
    print_status "info" "Opening VNC viewer..."

    # Start SSH tunnel in background
    ssh -f -N -L $local_port:localhost:$VNC_PORT $SSH_OPTS root@$ip

    # Open VNC viewer (macOS)
    if [[ "$(uname)" == "Darwin" ]]; then
        # Try built-in Screen Sharing
        open "vnc://localhost:$local_port" 2>/dev/null || \
        # Try RealVNC
        open -a "VNC Viewer" "localhost:$local_port" 2>/dev/null || \
        # Try TigerVNC
        vncviewer localhost:$local_port 2>/dev/null || \
        print_status "info" "Connect your VNC client to: localhost:$local_port"
    else
        # Linux
        vncviewer localhost:$local_port 2>/dev/null || \
        xdg-open "vnc://localhost:$local_port" 2>/dev/null || \
        print_status "info" "Connect your VNC client to: localhost:$local_port"
    fi

    echo ""
    print_status "info" "SSH tunnel running. Kill with: pkill -f 'ssh.*$local_port.*$ip'"
}

# Show VNC status
cmd_status() {
    echo -e "${BLUE}VNC Server Status${NC}"
    echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    for node in $VNC_NODES; do
        local ip=$(get_ip $node)

        if ! check_ssh $node; then
            print_status "fail" "$node - SSH unreachable"
            continue
        fi

        if check_vnc $node; then
            local display=$(ssh $SSH_OPTS root@$ip "pgrep -af Xvnc | grep -o ':[0-9]*' | head -1" 2>/dev/null || echo ":1")
            print_status "ok" "$node - VNC running on display $display"
        else
            local installed=$(ssh $SSH_OPTS root@$ip "which vncserver" 2>/dev/null)
            if [[ -n "$installed" ]]; then
                print_status "warn" "$node - VNC installed but not running"
            else
                print_status "info" "$node - VNC not installed"
            fi
        fi
    done | sort
}

# Stop VNC on a node
cmd_stop() {
    local node=${1:-}

    if [[ -z "$node" ]]; then
        echo "Usage: $0 stop <node|all>"
        exit 1
    fi

    local nodes_to_stop=""
    if [[ "$node" == "all" ]]; then
        nodes_to_stop="$VNC_NODES"
    else
        nodes_to_stop="$node"
    fi

    for n in $nodes_to_stop; do
        if ! node_exists $n; then
            print_status "warn" "Unknown node: $n"
            continue
        fi

        local ip=$(get_ip $n)
        print_status "info" "Stopping VNC on $n..."
        ssh $SSH_OPTS root@$ip "systemctl stop vncserver@:1 2>/dev/null || vncserver -kill :1 2>/dev/null || true"
        print_status "ok" "$n VNC stopped"
    done
}

# Start VNC on a node
cmd_start() {
    local node=${1:-}

    if [[ -z "$node" ]]; then
        echo "Usage: $0 start <node|all>"
        exit 1
    fi

    local nodes_to_start=""
    if [[ "$node" == "all" ]]; then
        nodes_to_start="$VNC_NODES"
    else
        nodes_to_start="$node"
    fi

    for n in $nodes_to_start; do
        if ! node_exists $n; then
            print_status "warn" "Unknown node: $n"
            continue
        fi

        local ip=$(get_ip $n)
        print_status "info" "Starting VNC on $n..."
        ssh $SSH_OPTS root@$ip "systemctl start vncserver@:1 2>/dev/null || vncserver :1 2>/dev/null || true"
        print_status "ok" "$n VNC started"
    done
}

# Main
case ${1:-help} in
    list)    cmd_list ;;
    install) shift; cmd_install "$@" ;;
    status)  cmd_status ;;
    start)   shift; cmd_start "$@" ;;
    stop)    shift; cmd_stop "$@" ;;
    help)
        echo "Communitas VNC Connection Helper"
        echo ""
        echo "Usage: $0 <command> [options]"
        echo ""
        echo "Commands:"
        echo "  <node>           Connect to VNC on specified node"
        echo "  list             List available VNC nodes"
        echo "  status           Show VNC status on all nodes"
        echo "  install <node>   Install VNC server on node"
        echo "  start <node|all> Start VNC server"
        echo "  stop <node|all>  Stop VNC server"
        echo ""
        echo "Available nodes: $VNC_NODES"
        echo ""
        echo "Examples:"
        echo "  $0 saorsa-4              # Connect to saorsa-4"
        echo "  $0 install saorsa-4      # Install VNC on saorsa-4"
        echo "  $0 status                # Check VNC status"
        ;;
    *)
        # Default: try to connect to node
        cmd_connect "$1"
        ;;
esac
