import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  Grid,
  Card,
  CardContent,
  Typography,
  Box,
  LinearProgress,
  Chip,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Divider,
  Alert,
  Button,
  CircularProgress,
  TextField,
  FormControlLabel,
  Switch,
  Tabs,
  Tab,
} from '@mui/material'
import {
  NetworkCheck,
  Router,
  Speed,
  SignalWifi4Bar,
  SignalWifiOff,
  Hub,
  Security,
  Refresh,
  PlayArrow,
  Stop,
  Clear,
  BugReport,
} from '@mui/icons-material'

interface PeerConnection {
  id: string
  address: string
  latency: number
  status: 'Connected' | 'Connecting' | 'Disconnected' | 'Failed'
  nat_type: 'Direct' | 'STUN' | 'TURN' | 'Relay'
  connection_quality: number
  bandwidth: { up: number; down: number }
  last_seen: string
}

interface NetworkMetrics {
  bandwidth_up: number
  bandwidth_down: number
  packet_loss: number
  jitter: number
  nat_type: 'Open' | 'Moderate' | 'Strict' | 'Unknown'
  upnp_available: boolean
  ipv6_support: boolean
  total_connections: number
  active_connections: number
}

interface TabPanelProps {
  children?: React.ReactNode
  index: number
  value: number
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props
  return (
    <div role="tabpanel" hidden={value !== index} {...other}>
      {value === index && <Box>{children}</Box>}
    </div>
  )
}

const NetworkDiagnosticsTab: React.FC = () => {
  const [tabValue, setTabValue] = useState(0)
  const [metrics, setMetrics] = useState<NetworkMetrics>({
    bandwidth_up: 0,
    bandwidth_down: 0,
    packet_loss: 0,
    jitter: 0,
    nat_type: 'Unknown',
    upnp_available: false,
    ipv6_support: false,
    total_connections: 0,
    active_connections: 0,
  })

  const [peers, setPeers] = useState<PeerConnection[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  
  // Diagnostics state
  const [packetCapture, setPacketCapture] = useState(false)
  const [filter, setFilter] = useState('')
  
  const packets = [
    {
      id: '1',
      timestamp: new Date().toLocaleTimeString(),
      protocol: 'QUIC',
      source: '192.168.1.100:9001',
      destination: '159.89.81.21:9001',
      size: 1200,
      info: 'Initial packet',
    },
    {
      id: '2',
      timestamp: new Date().toLocaleTimeString(),
      protocol: 'QUIC',
      source: '159.89.81.21:9001',
      destination: '192.168.1.100:9001',
      size: 800,
      info: 'Version negotiation',
    },
    {
      id: '3',
      timestamp: new Date().toLocaleTimeString(),
      protocol: 'DHT',
      source: '192.168.1.100:9001',
      destination: '159.89.81.21:9100',
      size: 456,
      info: 'FIND_NODE query',
    },
  ]

  const filteredPackets = packets.filter(packet =>
    !filter || packet.protocol.toLowerCase().includes(filter.toLowerCase()) ||
    packet.info.toLowerCase().includes(filter.toLowerCase())
  )

  const fetchNetworkData = async () => {
    try {
      const metricsData = await invoke<NetworkMetrics>('get_network_metrics')
      setMetrics(metricsData)

      const peersData = await invoke<PeerConnection[]>('get_peer_connections')
      setPeers(peersData)

      setError(null)
    } catch (err) {
      console.error('Failed to fetch network data:', err)
      setError('Failed to fetch network data')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchNetworkData()
  }, [])

  useEffect(() => {
    const interval = setInterval(() => {
      fetchNetworkData()
    }, 5000)

    return () => clearInterval(interval)
  }, [])

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Connected': return 'success'
      case 'Connecting': return 'warning'
      case 'Disconnected': return 'default'
      case 'Failed': return 'error'
      default: return 'default'
    }
  }

  const getNatTypeColor = (natType: string) => {
    switch (natType) {
      case 'Direct': return 'success'
      case 'STUN': return 'info'
      case 'TURN': return 'warning'
      case 'Relay': return 'error'
      default: return 'default'
    }
  }

  const getNatDifficulty = (natType: string): { text: string; severity: 'success' | 'info' | 'warning' | 'error' } => {
    switch (natType) {
      case 'Open': return { text: 'Excellent connectivity - Direct peer connections possible', severity: 'success' }
      case 'Moderate': return { text: 'Good connectivity - Some NAT traversal required', severity: 'info' }
      case 'Strict': return { text: 'Limited connectivity - Relay servers may be needed', severity: 'warning' }
      case 'Unknown': return { text: 'Connectivity unknown - Testing in progress', severity: 'error' }
      default: return { text: 'Unknown NAT configuration', severity: 'error' }
    }
  }

  const getQualityIcon = (quality: number) => {
    if (quality >= 80) return <SignalWifi4Bar color="success" />
    if (quality >= 60) return <SignalWifi4Bar color="warning" />
    if (quality >= 40) return <SignalWifi4Bar color="error" />
    return <SignalWifiOff color="error" />
  }

  const natDiagnostic = getNatDifficulty(metrics.nat_type)

  if (loading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '50vh' }}>
        <CircularProgress />
      </Box>
    )
  }

  return (
    <Box>
      <Typography variant="h4" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
        <NetworkCheck />
        Network Diagnostics
        <Box sx={{ ml: 'auto' }}>
          <Button 
            variant="outlined" 
            size="small" 
            startIcon={<Refresh />}
            onClick={fetchNetworkData}
          >
            Refresh
          </Button>
        </Box>
      </Typography>
      
      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      {/* Tabs for Network and Advanced Diagnostics */}
      <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 3 }}>
        <Tabs value={tabValue} onChange={(_e, newValue) => setTabValue(newValue)}>
          <Tab label="Network Overview" icon={<NetworkCheck />} iconPosition="start" />
          <Tab label="Advanced Diagnostics" icon={<BugReport />} iconPosition="start" />
        </Tabs>
      </Box>

      {/* Network Overview Tab */}
      <TabPanel value={tabValue} index={0}>
        {/* NAT & Connectivity Status */}
        <Card sx={{ mb: 3 }}>
          <CardContent>
            <Typography variant="h6" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <Security />
              NAT & Connectivity Analysis
            </Typography>
            <Alert severity={natDiagnostic.severity} sx={{ mb: 2 }}>
              <strong>NAT Type: {metrics.nat_type}</strong> - {natDiagnostic.text}
            </Alert>
            <Grid container spacing={2}>
              <Grid item xs={12} md={4}>
                <Box sx={{ textAlign: 'center', p: 2 }}>
                  <Typography variant="body2" color="textSecondary">UPnP Support</Typography>
                  <Chip 
                    label={metrics.upnp_available ? 'Available' : 'Not Available'} 
                    color={metrics.upnp_available ? 'success' : 'error'}
                    sx={{ mt: 1 }}
                  />
                </Box>
              </Grid>
              <Grid item xs={12} md={4}>
                <Box sx={{ textAlign: 'center', p: 2 }}>
                  <Typography variant="body2" color="textSecondary">IPv6 Support</Typography>
                  <Chip 
                    label={metrics.ipv6_support ? 'Enabled' : 'Disabled'} 
                    color={metrics.ipv6_support ? 'success' : 'warning'}
                    sx={{ mt: 1 }}
                  />
                </Box>
              </Grid>
              <Grid item xs={12} md={4}>
                <Box sx={{ textAlign: 'center', p: 2 }}>
                  <Typography variant="body2" color="textSecondary">Active Connections</Typography>
                  <Typography variant="h6" sx={{ mt: 1 }}>
                    {metrics.active_connections}/{metrics.total_connections}
                  </Typography>
                </Box>
              </Grid>
            </Grid>
          </CardContent>
        </Card>

        <Grid container spacing={3}>
          {/* Network Performance */}
          <Grid item xs={12} md={6}>
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <Speed />
                  Network Performance
                </Typography>
                <Box sx={{ mb: 2 }}>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <Typography variant="body2">Upload</Typography>
                    <Typography variant="body2" fontWeight="bold">{Math.round(metrics.bandwidth_up)} kbps</Typography>
                  </Box>
                  <LinearProgress
                    variant="determinate"
                    value={Math.min(100, (metrics.bandwidth_up / 2000) * 100)}
                    sx={{ mt: 1, height: 8, borderRadius: 4 }}
                  />
                </Box>
                <Box sx={{ mb: 2 }}>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <Typography variant="body2">Download</Typography>
                    <Typography variant="body2" fontWeight="bold">{Math.round(metrics.bandwidth_down)} kbps</Typography>
                  </Box>
                  <LinearProgress
                    variant="determinate"
                    value={Math.min(100, (metrics.bandwidth_down / 2000) * 100)}
                    sx={{ mt: 1, height: 8, borderRadius: 4 }}
                  />
                </Box>
                <Divider sx={{ my: 2 }} />
                <Grid container spacing={2}>
                  <Grid item xs={6}>
                    <Typography variant="body2" color="textSecondary">Packet Loss</Typography>
                    <Typography variant="h6" color={metrics.packet_loss > 1 ? 'error' : 'success'}>
                      {metrics.packet_loss}%
                    </Typography>
                  </Grid>
                  <Grid item xs={6}>
                    <Typography variant="body2" color="textSecondary">Jitter</Typography>
                    <Typography variant="h6" color={metrics.jitter > 10 ? 'error' : 'success'}>
                      {metrics.jitter} ms
                    </Typography>
                  </Grid>
                </Grid>
              </CardContent>
            </Card>
          </Grid>
          
          {/* Peer Connection Graph */}
          <Grid item xs={12} md={6}>
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <Hub />
                  Peer Connection Topology
                </Typography>
                <Box sx={{ textAlign: 'center', p: 2, position: 'relative', minHeight: 200 }}>
                  {peers.length === 0 ? (
                    <Box sx={{ 
                      display: 'flex', 
                      flexDirection: 'column', 
                      alignItems: 'center',
                      justifyContent: 'center',
                      height: 200,
                      color: 'text.secondary'
                    }}>
                      <SignalWifiOff sx={{ fontSize: 48, mb: 2 }} />
                      <Typography variant="body1">No active peer connections</Typography>
                      <Typography variant="caption">Waiting for P2P network connection...</Typography>
                    </Box>
                  ) : (
                    <Box sx={{ 
                      display: 'flex', 
                      flexDirection: 'column', 
                      alignItems: 'center',
                      gap: 2 
                    }}>
                      {/* Central node (you) */}
                      <Box sx={{ 
                        width: 60, 
                        height: 60, 
                        borderRadius: '50%', 
                        bgcolor: 'primary.main', 
                        display: 'flex', 
                        alignItems: 'center', 
                        justifyContent: 'center',
                        color: 'white',
                        fontWeight: 'bold'
                      }}>
                        YOU
                      </Box>
                      
                      {/* Connection lines and peer nodes */}
                      <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 1, justifyContent: 'center' }}>
                        {peers.filter(p => p.status === 'Connected').map((peer) => (
                          <Box key={peer.id} sx={{ 
                            display: 'flex', 
                            flexDirection: 'column', 
                            alignItems: 'center',
                            minWidth: 80
                          }}>
                            <Box sx={{ 
                              width: 40, 
                              height: 40, 
                              borderRadius: '50%', 
                              bgcolor: peer.connection_quality > 70 ? 'success.main' : peer.connection_quality > 40 ? 'warning.main' : 'error.main',
                              display: 'flex', 
                              alignItems: 'center', 
                              justifyContent: 'center',
                              color: 'white',
                              fontSize: '0.75rem'
                            }}>
                              {peer.latency}ms
                            </Box>
                            <Typography variant="caption" sx={{ mt: 0.5, textAlign: 'center' }}>
                              {peer.address.split(':')[0].substring(0, 12)}...
                            </Typography>
                          </Box>
                        ))}
                      </Box>
                    </Box>
                  )}
                </Box>
              </CardContent>
            </Card>
          </Grid>
          
          {/* Detailed Peer List */}
          <Grid item xs={12}>
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <Router />
                  Detailed Peer Connections
                  <Typography variant="body2" color="textSecondary" sx={{ ml: 2 }}>
                    ({peers.length} {peers.length === 1 ? 'peer' : 'peers'})
                  </Typography>
                </Typography>
                {peers.length === 0 ? (
                  <Box sx={{ 
                    textAlign: 'center', 
                    py: 4, 
                    color: 'text.secondary' 
                  }}>
                    <Typography>No peer connections established yet</Typography>
                    <Typography variant="caption">
                      The P2P node will connect to bootstrap nodes when initialized
                    </Typography>
                  </Box>
                ) : (
                  <TableContainer component={Paper} variant="outlined">
                    <Table>
                      <TableHead>
                        <TableRow>
                          <TableCell>Quality</TableCell>
                          <TableCell>Address</TableCell>
                          <TableCell>Status</TableCell>
                          <TableCell>NAT Type</TableCell>
                          <TableCell>Latency</TableCell>
                          <TableCell>Bandwidth</TableCell>
                          <TableCell>Last Seen</TableCell>
                        </TableRow>
                      </TableHead>
                      <TableBody>
                        {peers.map((peer) => (
                          <TableRow key={peer.id} sx={{ 
                            backgroundColor: peer.status === 'Failed' ? 'error.light' : 'inherit',
                            opacity: peer.status === 'Failed' ? 0.6 : 1
                          }}>
                            <TableCell>
                              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                                {getQualityIcon(peer.connection_quality)}
                                <Typography variant="body2">{peer.connection_quality}%</Typography>
                              </Box>
                            </TableCell>
                            <TableCell sx={{ fontFamily: 'monospace' }}>{peer.address}</TableCell>
                            <TableCell>
                              <Chip
                                label={peer.status}
                                color={getStatusColor(peer.status)}
                                size="small"
                              />
                            </TableCell>
                            <TableCell>
                              <Chip
                                label={peer.nat_type}
                                color={getNatTypeColor(peer.nat_type)}
                                size="small"
                                variant="outlined"
                              />
                            </TableCell>
                            <TableCell>{peer.latency}ms</TableCell>
                            <TableCell>
                              <Typography variant="body2">
                                ↑{peer.bandwidth.up} ↓{peer.bandwidth.down} kbps
                              </Typography>
                            </TableCell>
                            <TableCell>
                              <Typography variant="body2" color="textSecondary">
                                {new Date(peer.last_seen).toLocaleTimeString()}
                              </Typography>
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </TableContainer>
                )}
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      </TabPanel>

      {/* Advanced Diagnostics Tab */}
      <TabPanel value={tabValue} index={1}>
        <Card sx={{ mb: 3 }}>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              Packet Capture
            </Typography>
            
            <Box sx={{ display: 'flex', gap: 2, alignItems: 'center', mb: 2 }}>
              <FormControlLabel
                control={
                  <Switch
                    checked={packetCapture}
                    onChange={(e) => setPacketCapture(e.target.checked)}
                  />
                }
                label="Enable Packet Capture"
              />
              
              <Button
                variant="contained"
                startIcon={packetCapture ? <Stop /> : <PlayArrow />}
                onClick={() => setPacketCapture(!packetCapture)}
                color={packetCapture ? 'error' : 'primary'}
              >
                {packetCapture ? 'Stop' : 'Start'} Capture
              </Button>
              
              <Button
                variant="outlined"
                startIcon={<Clear />}
                disabled={packetCapture}
              >
                Clear
              </Button>
            </Box>
            
            <TextField
              fullWidth
              label="Filter packets"
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              placeholder="Filter by protocol or info..."
              sx={{ mb: 2 }}
            />
            
            <Typography variant="body2" color="textSecondary">
              Status: {packetCapture ? 
                <Chip label="Capturing" color="success" size="small" /> : 
                <Chip label="Stopped" color="default" size="small" />
              }
            </Typography>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              Captured Packets ({filteredPackets.length})
            </Typography>
            
            <TableContainer component={Paper} sx={{ maxHeight: 400 }}>
              <Table stickyHeader size="small">
                <TableHead>
                  <TableRow>
                    <TableCell>Time</TableCell>
                    <TableCell>Protocol</TableCell>
                    <TableCell>Source</TableCell>
                    <TableCell>Destination</TableCell>
                    <TableCell>Size</TableCell>
                    <TableCell>Info</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {filteredPackets.map((packet) => (
                    <TableRow key={packet.id} hover>
                      <TableCell>{packet.timestamp}</TableCell>
                      <TableCell>
                        <Chip label={packet.protocol} size="small" />
                      </TableCell>
                      <TableCell>{packet.source}</TableCell>
                      <TableCell>{packet.destination}</TableCell>
                      <TableCell>{packet.size}B</TableCell>
                      <TableCell>{packet.info}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
          </CardContent>
        </Card>
      </TabPanel>
    </Box>
  )
}

export default NetworkDiagnosticsTab