import React, { useState, useEffect } from 'react'
import {
  Grid,
  Card,
  CardContent,
  Typography,
  Box,
  Chip,
  Button,
  CircularProgress,
  Alert,
} from '@mui/material'
import { NetworkHealth } from '../../types'
import { invoke } from '@tauri-apps/api/core'

interface OverviewTabProps {
  networkHealth: NetworkHealth
}

interface DHTStatus {
  node_id: string
  peer_count: number
  stored_items: number
  network_health: number
  uptime: number
  performance: {
    avg_lookup_latency: number
    avg_store_latency: number
    operation_success_rate: number
    throughput: number
    bandwidth_utilization: number
    memory_usage: number
  }
}

const OverviewTab: React.FC<OverviewTabProps> = ({ networkHealth }) => {
  const [isInitializing, setIsInitializing] = useState(false)
  const [isConnected, setIsConnected] = useState(false)
  const [nodeId, setNodeId] = useState<string | null>(null)
  const [dhtStatus, setDhtStatus] = useState<DHTStatus | null>(null)
  const [error, setError] = useState<string | null>(null)

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Connected': return 'success'
      case 'Connecting': return 'warning'
      case 'Disconnected': return 'error'
      default: return 'default'
    }
  }

  const initializeP2P = async () => {
    setIsInitializing(true)
    setError(null)
    try {
      const peerId = await invoke<string>('initialize_p2p_node')
      setNodeId(peerId)
      setIsConnected(true)
      console.log('P2P node initialized with ID:', peerId)
    } catch (err) {
      setError(`Failed to initialize P2P: ${err}`)
      console.error('Failed to initialize P2P:', err)
    } finally {
      setIsInitializing(false)
    }
  }

  useEffect(() => {
    if (isConnected) {
      const fetchDHTStatus = async () => {
        try {
          const status = await invoke<DHTStatus>('get_dht_status')
          if (status) {
            setDhtStatus(status)
            console.log('DHT Status:', status)
          }
        } catch (err) {
          console.error('Failed to get DHT status:', err)
        }
      }

      fetchDHTStatus()
      const interval = setInterval(fetchDHTStatus, 3000)
      return () => clearInterval(interval)
    }
  }, [isConnected])

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Network Overview
      </Typography>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      {!isConnected && (
        <Card sx={{ mb: 3 }}>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              P2P Network Connection
            </Typography>
            <Typography variant="body2" color="textSecondary" sx={{ mb: 2 }}>
              Connect to the P2P network to join 100+ nodes on DigitalOcean
            </Typography>
            <Button
              variant="contained"
              color="primary"
              onClick={initializeP2P}
              disabled={isInitializing}
              startIcon={isInitializing ? <CircularProgress size={20} /> : null}
            >
              {isInitializing ? 'Connecting...' : 'Connect to P2P Network'}
            </Button>
          </CardContent>
        </Card>
      )}

      {nodeId && (
        <Alert severity="success" sx={{ mb: 2 }}>
          Connected to P2P network! Node ID: {nodeId}
        </Alert>
      )}
      
      <Grid container spacing={3}>
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Connection Status
              </Typography>
              <Chip 
                label={isConnected ? 'Connected' : networkHealth.status}
                color={getStatusColor(isConnected ? 'Connected' : networkHealth.status)}
                size="medium"
              />
            </CardContent>
          </Card>
        </Grid>
        
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Connected Peers
              </Typography>
              <Typography variant="h3" color={(dhtStatus?.peer_count ?? 0) > 0 ? 'success.main' : 'text.primary'}>
                {dhtStatus?.peer_count ?? networkHealth.peer_count}
              </Typography>
              {(dhtStatus?.peer_count ?? 0) > 50 && (
                <Typography variant="caption" color="success.main">
                  Large network detected!
                </Typography>
              )}
            </CardContent>
          </Card>
        </Grid>
        
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                NAT Type
              </Typography>
              <Typography variant="h5">
                {networkHealth.nat_type}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Network Performance
              </Typography>
              <Typography variant="body1">
                Bandwidth: {networkHealth.bandwidth_kbps} kbps
              </Typography>
              <Typography variant="body1">
                Latency: {networkHealth.avg_latency_ms} ms
              </Typography>
            </CardContent>
          </Card>
        </Grid>

        {dhtStatus && (
          <>
            <Grid item xs={12} md={6}>
              <Card>
                <CardContent>
                  <Typography variant="h6" gutterBottom>
                    DHT Items Stored
                  </Typography>
                  <Typography variant="h3">
                    {dhtStatus.stored_items}
                  </Typography>
                </CardContent>
              </Card>
            </Grid>

            <Grid item xs={12}>
              <Card>
                <CardContent>
                  <Typography variant="h6" gutterBottom>
                    DHT Performance Metrics
                  </Typography>
                  <Grid container spacing={2}>
                    <Grid item xs={6} md={3}>
                      <Typography variant="body2" color="textSecondary">
                        Network Health
                      </Typography>
                      <Typography variant="h6">
                        {(dhtStatus.network_health * 100).toFixed(0)}%
                      </Typography>
                    </Grid>
                    <Grid item xs={6} md={3}>
                      <Typography variant="body2" color="textSecondary">
                        Success Rate
                      </Typography>
                      <Typography variant="h6">
                        {(dhtStatus.performance.operation_success_rate * 100).toFixed(0)}%
                      </Typography>
                    </Grid>
                    <Grid item xs={6} md={3}>
                      <Typography variant="body2" color="textSecondary">
                        Throughput
                      </Typography>
                      <Typography variant="h6">
                        {dhtStatus.performance.throughput.toFixed(2)} ops/s
                      </Typography>
                    </Grid>
                    <Grid item xs={6} md={3}>
                      <Typography variant="body2" color="textSecondary">
                        Bandwidth Usage
                      </Typography>
                      <Typography variant="h6">
                        {(dhtStatus.performance.bandwidth_utilization * 100).toFixed(0)}%
                      </Typography>
                    </Grid>
                  </Grid>
                </CardContent>
              </Card>
            </Grid>
          </>
        )}
      </Grid>
    </Box>
  )
}

export default OverviewTab
