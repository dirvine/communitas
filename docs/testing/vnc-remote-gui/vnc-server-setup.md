# VNC Remote GUI Testing Setup

Access remote GUI on VPS test nodes for WebRTC and visual testing.

## Overview

VNC servers on test nodes allow:
- Visual verification of GUI applications
- WebRTC call testing across geographic locations
- Screen recording of test scenarios
- Manual testing through remote desktop

## Quick Start

```bash
# Check VNC status on all nodes
./scripts/vnc-connect.sh status

# Install VNC on a node (first time only)
./scripts/vnc-connect.sh install saorsa-4

# Connect to node
./scripts/vnc-connect.sh saorsa-4
```

## Node Configuration

VNC is installed on test nodes only (not bootstrap):

| Node | Location | VNC Status |
|------|----------|------------|
| saorsa-4 | AMS | Available |
| saorsa-5 | LON | Available |
| saorsa-6 | Helsinki | Available |
| saorsa-7 | Nuremberg | Available |
| saorsa-8 | Singapore | Available |
| saorsa-9 | Tokyo | Available |

## Installation

### Automatic (Recommended)
```bash
./scripts/vnc-connect.sh install saorsa-4
```

### Manual Installation

```bash
# SSH to node
ssh root@206.189.7.117

# Install packages (Debian/Ubuntu)
apt-get update
apt-get install -y tigervnc-standalone-server tigervnc-common dbus-x11 xfce4

# Set VNC password
vncpasswd

# Create xstartup
cat > ~/.vnc/xstartup << 'EOF'
#!/bin/sh
unset SESSION_MANAGER
unset DBUS_SESSION_BUS_ADDRESS
exec startxfce4
EOF
chmod +x ~/.vnc/xstartup

# Start VNC server
vncserver :1 -geometry 1280x800 -depth 24
```

## Connecting

### Via SSH Tunnel (Recommended)
```bash
# Automatic (creates tunnel and opens viewer)
./scripts/vnc-connect.sh saorsa-4

# Manual tunnel
ssh -L 5901:localhost:5901 root@206.189.7.117

# Then connect VNC viewer to localhost:5901
```

### macOS Screen Sharing
```bash
# Open Screen Sharing
open "vnc://localhost:5901"
```

### TigerVNC/RealVNC
```bash
vncviewer localhost:5901
```

## Testing Scenarios

### WebRTC Call Test

1. Connect to two different VNC nodes (e.g., saorsa-4 and saorsa-8)
2. Start Communitas Iced app on each
3. Initiate call from one to the other
4. Verify video/audio quality

### Cross-Region Latency Test

1. Connect to geographically distant nodes (e.g., NYC and Tokyo)
2. Start real-time collaboration session
3. Measure typing latency, cursor sync
4. Record results

### NAT Traversal Visual Test

1. Connect to node behind NAT emulation
2. Start Communitas app
3. Observe connection status indicators
4. Verify peer discovery and connection

## VNC Server Management

### Start VNC
```bash
./scripts/vnc-connect.sh start saorsa-4
# Or all nodes
./scripts/vnc-connect.sh start all
```

### Stop VNC
```bash
./scripts/vnc-connect.sh stop saorsa-4
```

### Check Status
```bash
./scripts/vnc-connect.sh status
```

## Display Configuration

### Change Resolution
```bash
# Stop and restart with new geometry
vncserver -kill :1
vncserver :1 -geometry 1920x1080 -depth 24
```

### Multiple Displays
```bash
# Display :1
vncserver :1 -geometry 1280x800

# Display :2
vncserver :2 -geometry 1024x768
```

## Troubleshooting

### "Connection refused"
```bash
# Check if VNC is running
./scripts/vnc-connect.sh status

# Start VNC
./scripts/vnc-connect.sh start saorsa-4
```

### "Authentication failed"
```bash
# Reset VNC password
ssh root@206.189.7.117 "vncpasswd"
# Default password: communitas
```

### "Black screen"
```bash
# Check xstartup
ssh root@206.189.7.117 "cat ~/.vnc/xstartup"

# Reinstall desktop environment
ssh root@206.189.7.117 "apt-get install -y xfce4"
```

### "Tunnel not working"
```bash
# Kill existing tunnels
pkill -f 'ssh.*5901'

# Create new tunnel
./scripts/vnc-connect.sh saorsa-4
```

## Security Notes

1. VNC traffic is encrypted via SSH tunnel
2. Default password should be changed for production
3. VNC ports (5900-5999) are NOT exposed publicly
4. Access requires SSH key authentication

## Screen Recording

### Record VNC Session
```bash
# On local machine, record vnc window
# macOS: Use QuickTime Player
# Linux: Use SimpleScreenRecorder or OBS
```

### Automated Screenshot
```bash
# Take screenshot via SSH
ssh root@206.189.7.117 "DISPLAY=:1 xwd -root | convert xwd:- screenshot.png"
scp root@206.189.7.117:screenshot.png ./
```
