#!/usr/bin/env node

/**
 * MCP Peer Cache Monitor
 * Monitors the peer cache and network connectivity of running Communitas apps
 */

const net = require('net');
const fs = require('fs');
const path = require('path');

// Colors for output
const colors = {
    reset: '\x1b[0m',
    bright: '\x1b[1m',
    red: '\x1b[31m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    cyan: '\x1b[36m',
};

class MCPMonitor {
    constructor(socketPath, appName) {
        this.socketPath = socketPath;
        this.appName = appName;
        this.socket = null;
        this.requestId = 0;
    }

    async connect() {
        return new Promise((resolve, reject) => {
            this.socket = net.createConnection(this.socketPath, () => {
                console.log(`${colors.green}✓${colors.reset} Connected to ${this.appName} MCP at ${this.socketPath}`);
                resolve();
            });

            this.socket.on('error', (err) => {
                console.error(`${colors.red}✗${colors.reset} Failed to connect to ${this.appName}: ${err.message}`);
                reject(err);
            });

            this.socket.setTimeout(5000);
        });
    }

    async sendCommand(command, payload = {}) {
        return new Promise((resolve, reject) => {
            const request = {
                command: command,
                payload: { window_label: 'main', ...payload }
            };

            const requestStr = JSON.stringify(request) + '\n';
            this.socket.write(requestStr);

            const timeout = setTimeout(() => {
                reject(new Error('Command timeout'));
            }, 5000);

            this.socket.once('data', (data) => {
                clearTimeout(timeout);
                try {
                    const response = JSON.parse(data.toString());
                    resolve(response);
                } catch (e) {
                    reject(e);
                }
            });
        });
    }

    async monitorPeerCache() {
        const response = await this.sendCommand('execute_js', {
            code: `
                (function() {
                    // Access network service
                    const networkService = window.networkService || window.testNetwork?.service;
                    if (!networkService) {
                        return JSON.stringify({
                            error: 'Network service not available',
                            status: 'not-initialized'
                        });
                    }

                    // Get current state
                    const state = networkService.getState();

                    // Try to get peer information
                    let peers = [];
                    if (networkService.getPeers) {
                        peers = networkService.getPeers();
                    } else if (state.peers) {
                        peers = state.peers;
                    }

                    // Get bootstrap nodes
                    let bootstrapNodes = [];
                    if (state.bootstrapNodes) {
                        bootstrapNodes = state.bootstrapNodes;
                    } else if (networkService.getBootstrapNodes) {
                        bootstrapNodes = networkService.getBootstrapNodes();
                    }

                    return JSON.stringify({
                        status: state.status || 'unknown',
                        connectionCount: state.connectionCount || peers.length || 0,
                        peers: peers.slice(0, 10).map(p => ({
                            id: p.id || p.peerId || p.address,
                            address: p.address || p.multiaddr,
                            latency: p.latency || p.rtt || null,
                            connected: p.connected !== false,
                            lastSeen: p.lastSeen || Date.now()
                        })),
                        bootstrapNodes: bootstrapNodes.slice(0, 5),
                        localAddress: state.localAddress || null,
                        networkId: state.networkId || null,
                        startTime: state.startTime || null
                    });
                })()
            `
        });

        if (response.success && response.data) {
            try {
                return JSON.parse(response.data.value);
            } catch (e) {
                return { error: 'Failed to parse response', raw: response.data.value };
            }
        }

        return { error: response.error || 'Unknown error' };
    }

    async getNetworkStats() {
        const response = await this.sendCommand('execute_js', {
            code: `
                (function() {
                    // Get various network statistics
                    const stats = {
                        timestamp: Date.now(),
                        userAgent: navigator.userAgent
                    };

                    // Check for network status indicator
                    const indicator = document.querySelector('[data-testid="network-status"]');
                    if (indicator) {
                        stats.uiStatus = indicator.getAttribute('data-status');
                        stats.uiIndicatorFound = true;
                    } else {
                        stats.uiIndicatorFound = false;
                    }

                    // Check for peer count display
                    const peerCount = document.querySelector('[data-testid="peer-count"]');
                    if (peerCount) {
                        stats.displayedPeerCount = peerCount.textContent;
                    }

                    // Check local storage for cached peers
                    try {
                        const cachedPeers = localStorage.getItem('communitas-peer-cache');
                        if (cachedPeers) {
                            const parsed = JSON.parse(cachedPeers);
                            stats.cachedPeerCount = Array.isArray(parsed) ? parsed.length :
                                                   (parsed.peers ? parsed.peers.length : 0);
                            stats.cacheAge = parsed.timestamp ?
                                           Date.now() - parsed.timestamp : null;
                        }
                    } catch (e) {
                        stats.cacheError = e.message;
                    }

                    // Check IndexedDB for offline storage
                    if (window.indexedDB) {
                        stats.indexedDBAvailable = true;
                    }

                    return JSON.stringify(stats);
                })()
            `
        });

        if (response.success && response.data) {
            try {
                return JSON.parse(response.data.value);
            } catch (e) {
                return { error: 'Failed to parse stats' };
            }
        }

        return { error: response.error || 'Unknown error' };
    }

    async takeScreenshot() {
        const response = await this.sendCommand('take_screenshot', {
            format: 'png'
        });

        if (response.success && response.data) {
            const filename = `screenshot-${this.appName.toLowerCase()}-${Date.now()}.png`;
            const screenshotPath = path.join('/tmp', filename);
            fs.writeFileSync(screenshotPath, Buffer.from(response.data.value, 'base64'));
            return screenshotPath;
        }

        return null;
    }

    disconnect() {
        if (this.socket) {
            this.socket.end();
        }
    }
}

async function monitorAllApps() {
    console.log(`\n${colors.blue}═══════════════════════════════════════════${colors.reset}`);
    console.log(`${colors.blue} MCP Peer Cache & Network Monitor${colors.reset}`);
    console.log(`${colors.blue}═══════════════════════════════════════════${colors.reset}\n`);

    // Find all MCP sockets
    const tmpFiles = fs.readdirSync('/tmp');
    const mcpSockets = tmpFiles
        .filter(f => f.startsWith('tauri-mcp-') && f.endsWith('.sock'))
        .map(f => `/tmp/${f}`);

    if (mcpSockets.length === 0) {
        console.log(`${colors.yellow}⚠ No MCP sockets found. Make sure Tauri apps are running.${colors.reset}`);
        console.log('Looking for sockets matching: /tmp/tauri-mcp-*.sock\n');
        return;
    }

    console.log(`Found ${colors.cyan}${mcpSockets.length}${colors.reset} MCP socket(s)\n`);

    // Monitor each app
    for (let i = 0; i < mcpSockets.length; i++) {
        const socketPath = mcpSockets[i];
        const appName = `App${i + 1}`;

        console.log(`${colors.bright}${colors.blue}━━━ ${appName} ━━━${colors.reset}`);

        const monitor = new MCPMonitor(socketPath, appName);

        try {
            await monitor.connect();

            // Get peer cache info
            const peerInfo = await monitor.monitorPeerCache();

            if (peerInfo.error) {
                console.log(`${colors.red}Error: ${peerInfo.error}${colors.reset}`);
            } else {
                console.log(`\n${colors.cyan}Network Status:${colors.reset}`);
                console.log(`  Status: ${getStatusColor(peerInfo.status)}${peerInfo.status}${colors.reset}`);
                console.log(`  Connected Peers: ${colors.bright}${peerInfo.connectionCount}${colors.reset}`);

                if (peerInfo.localAddress) {
                    console.log(`  Local Address: ${peerInfo.localAddress}`);
                }

                if (peerInfo.networkId) {
                    console.log(`  Network ID: ${peerInfo.networkId}`);
                }

                if (peerInfo.bootstrapNodes && peerInfo.bootstrapNodes.length > 0) {
                    console.log(`\n${colors.cyan}Bootstrap Nodes:${colors.reset}`);
                    peerInfo.bootstrapNodes.forEach(node => {
                        console.log(`  • ${node}`);
                    });
                }

                if (peerInfo.peers && peerInfo.peers.length > 0) {
                    console.log(`\n${colors.cyan}Peer Cache (${peerInfo.peers.length} peers):${colors.reset}`);
                    peerInfo.peers.forEach(peer => {
                        const latencyStr = peer.latency ? `${peer.latency}ms` : 'N/A';
                        const statusIcon = peer.connected ? '✓' : '✗';
                        const statusColor = peer.connected ? colors.green : colors.red;
                        console.log(`  ${statusColor}${statusIcon}${colors.reset} ${peer.id || peer.address} (latency: ${latencyStr})`);
                    });
                }
            }

            // Get network stats
            const stats = await monitor.getNetworkStats();
            if (!stats.error) {
                console.log(`\n${colors.cyan}Additional Stats:${colors.reset}`);
                if (stats.uiIndicatorFound) {
                    console.log(`  UI Status: ${getStatusColor(stats.uiStatus)}${stats.uiStatus}${colors.reset}`);
                }
                if (stats.displayedPeerCount !== undefined) {
                    console.log(`  Displayed Peer Count: ${stats.displayedPeerCount}`);
                }
                if (stats.cachedPeerCount !== undefined) {
                    console.log(`  Cached Peers: ${stats.cachedPeerCount}`);
                    if (stats.cacheAge) {
                        const ageSeconds = Math.floor(stats.cacheAge / 1000);
                        console.log(`  Cache Age: ${ageSeconds} seconds`);
                    }
                }
                if (stats.indexedDBAvailable) {
                    console.log(`  IndexedDB: ${colors.green}Available${colors.reset}`);
                }
            }

            // Take screenshot
            const screenshotPath = await monitor.takeScreenshot();
            if (screenshotPath) {
                console.log(`\n${colors.green}✓${colors.reset} Screenshot saved: ${screenshotPath}`);
            }

            monitor.disconnect();

        } catch (error) {
            console.log(`${colors.red}Failed to monitor: ${error.message}${colors.reset}`);
        }

        console.log('');
    }

    console.log(`${colors.blue}═══════════════════════════════════════════${colors.reset}\n`);
}

function getStatusColor(status) {
    switch(status) {
        case 'connected':
        case 'online':
            return colors.green;
        case 'connecting':
        case 'local':
            return colors.yellow;
        case 'disconnected':
        case 'offline':
        case 'error':
            return colors.red;
        default:
            return colors.reset;
    }
}

// Continuous monitoring mode
async function continuousMonitor() {
    while (true) {
        console.clear();
        await monitorAllApps();

        console.log(`${colors.yellow}Refreshing in 5 seconds... (Ctrl+C to stop)${colors.reset}`);
        await new Promise(resolve => setTimeout(resolve, 5000));
    }
}

// Main execution
const args = process.argv.slice(2);
const continuous = args.includes('--continuous') || args.includes('-c');

if (continuous) {
    continuousMonitor().catch(console.error);
} else {
    monitorAllApps().catch(console.error);
}