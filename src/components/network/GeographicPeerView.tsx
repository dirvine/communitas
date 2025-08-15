import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  Box,
  Card,
  CardContent,
  Typography,
  Grid,
  Chip,
  LinearProgress,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Tooltip,
  IconButton,
  Alert,
} from '@mui/material'
import {
  Public,
  LocationOn,
  Speed,
  SignalCellularAlt,
  Refresh,
  Map as MapIcon,
} from '@mui/icons-material'

interface GeographicPeerInfo {
  peer_id: string
  address: string
  region: string
  country?: string
  city?: string
  latency_ms?: number
  reliability_score: number
  last_seen: string
  connection_quality: string
  is_bootstrap: boolean
  cross_region: boolean
}

interface GeographicStatus {
  enabled: boolean
  local_region: string
  peer_count: number
  regions_covered: string[]
  cross_region_ratio: number
  avg_latency_ms: number
  routing_efficiency: number
}

interface RegionalPeerStats {
  region: string
  peer_count: number
  avg_latency_ms?: number
  avg_reliability: number
  is_local_region: boolean
}

const REGION_COLORS: Record<string, string> = {
  NorthAmerica: '#4CAF50',
  Europe: '#2196F3',
  AsiaPacific: '#FF9800',
  SouthAmerica: '#9C27B0',
  Africa: '#F44336',
  Oceania: '#00BCD4',
  Unknown: '#9E9E9E',
}

export const GeographicPeerView: React.FC = () => {
  const [status, setStatus] = useState<GeographicStatus | null>(null)
  const [peers, setPeers] = useState<GeographicPeerInfo[]>([])
  const [regionalStats, setRegionalStats] = useState<RegionalPeerStats[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchGeographicData = async () => {
    try {
      // Fetch geographic status
      const statusData = await invoke<GeographicStatus>('get_geographic_status')
      setStatus(statusData)

      // Only fetch peers if geographic routing is enabled
      if (statusData.enabled) {
        const peersData = await invoke<GeographicPeerInfo[]>('get_geographic_peers')
        setPeers(peersData)

        const statsData = await invoke<RegionalPeerStats[]>('get_regional_stats')
        setRegionalStats(statsData)
      }

      setError(null)
    } catch (err) {
      console.error('Failed to fetch geographic data:', err)
      setError('Failed to fetch geographic routing data')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchGeographicData()
  }, [])

  // Auto-refresh every 10 seconds
  useEffect(() => {
    const interval = setInterval(() => {
      fetchGeographicData()
    }, 10000)

    return () => clearInterval(interval)
  }, [])

  const getRegionColor = (region: string) => {
    return REGION_COLORS[region] || REGION_COLORS.Unknown
  }

  const getQualityColor = (quality: string) => {
    switch (quality) {
      case 'Excellent': return 'success'
      case 'Good': return 'info'
      case 'Fair': return 'warning'
      case 'Poor': return 'error'
      default: return 'default'
    }
  }

  const getReliabilityColor = (score: number) => {
    if (score >= 0.8) return 'success'
    if (score >= 0.6) return 'warning'
    return 'error'
  }

  if (loading) {
    return (
      <Box sx={{ p: 2 }}>
        <LinearProgress />
        <Typography variant="body2" sx={{ mt: 1 }}>Loading geographic routing data...</Typography>
      </Box>
    )
  }

  if (!status || !status.enabled) {
    return (
      <Alert severity="info">
        Geographic routing is not enabled. Enable it in settings to see peer geographic distribution.
      </Alert>
    )
  }

  return (
    <Box>
      {/* Geographic Status Overview */}
      <Card sx={{ mb: 3 }}>
        <CardContent>
          <Typography variant="h6" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <Public color="primary" />
            Geographic Routing Status
            <IconButton size="small" onClick={fetchGeographicData} sx={{ ml: 'auto' }}>
              <Refresh />
            </IconButton>
          </Typography>
          
          <Grid container spacing={2}>
            <Grid item xs={12} md={3}>
              <Box sx={{ textAlign: 'center' }}>
                <Typography variant="body2" color="textSecondary">Local Region</Typography>
                <Chip 
                  label={status.local_region}
                  sx={{ 
                    mt: 1,
                    backgroundColor: getRegionColor(status.local_region),
                    color: 'white'
                  }}
                />
              </Box>
            </Grid>
            <Grid item xs={12} md={3}>
              <Box sx={{ textAlign: 'center' }}>
                <Typography variant="body2" color="textSecondary">Geographic Peers</Typography>
                <Typography variant="h6">{status.peer_count}</Typography>
              </Box>
            </Grid>
            <Grid item xs={12} md={3}>
              <Box sx={{ textAlign: 'center' }}>
                <Typography variant="body2" color="textSecondary">Cross-Region</Typography>
                <Typography variant="h6">{(status.cross_region_ratio * 100).toFixed(1)}%</Typography>
              </Box>
            </Grid>
            <Grid item xs={12} md={3}>
              <Box sx={{ textAlign: 'center' }}>
                <Typography variant="body2" color="textSecondary">Avg Latency</Typography>
                <Typography variant="h6">{status.avg_latency_ms.toFixed(0)}ms</Typography>
              </Box>
            </Grid>
          </Grid>

          {/* Region Coverage */}
          <Box sx={{ mt: 2 }}>
            <Typography variant="body2" color="textSecondary" gutterBottom>
              Regional Coverage
            </Typography>
            <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 1 }}>
              {status.regions_covered.map((region) => (
                <Chip
                  key={region}
                  label={region}
                  size="small"
                  sx={{
                    backgroundColor: getRegionColor(region),
                    color: 'white'
                  }}
                />
              ))}
            </Box>
          </Box>
        </CardContent>
      </Card>

      {/* Regional Statistics */}
      <Card sx={{ mb: 3 }}>
        <CardContent>
          <Typography variant="h6" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <MapIcon />
            Regional Distribution
          </Typography>
          
          <Grid container spacing={2}>
            {regionalStats.map((stat) => (
              <Grid item xs={12} sm={6} md={4} key={stat.region}>
                <Card variant="outlined" sx={{ 
                  borderColor: stat.is_local_region ? 'primary.main' : 'divider',
                  borderWidth: stat.is_local_region ? 2 : 1
                }}>
                  <CardContent>
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1 }}>
                      <LocationOn sx={{ color: getRegionColor(stat.region) }} />
                      <Typography variant="subtitle2">
                        {stat.region}
                        {stat.is_local_region && (
                          <Chip label="LOCAL" size="small" color="primary" sx={{ ml: 1 }} />
                        )}
                      </Typography>
                    </Box>
                    <Grid container spacing={1}>
                      <Grid item xs={6}>
                        <Typography variant="caption" color="textSecondary">Peers</Typography>
                        <Typography variant="body2" fontWeight="bold">{stat.peer_count}</Typography>
                      </Grid>
                      <Grid item xs={6}>
                        <Typography variant="caption" color="textSecondary">Latency</Typography>
                        <Typography variant="body2" fontWeight="bold">
                          {stat.avg_latency_ms ? `${stat.avg_latency_ms.toFixed(0)}ms` : 'N/A'}
                        </Typography>
                      </Grid>
                      <Grid item xs={12}>
                        <Typography variant="caption" color="textSecondary">Reliability</Typography>
                        <LinearProgress
                          variant="determinate"
                          value={stat.avg_reliability * 100}
                          color={getReliabilityColor(stat.avg_reliability)}
                          sx={{ height: 6, borderRadius: 3 }}
                        />
                      </Grid>
                    </Grid>
                  </CardContent>
                </Card>
              </Grid>
            ))}
          </Grid>
        </CardContent>
      </Card>

      {/* Detailed Peer List */}
      <Card>
        <CardContent>
          <Typography variant="h6" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <SignalCellularAlt />
            Geographic Peers ({peers.length})
          </Typography>

          {peers.length === 0 ? (
            <Typography variant="body2" color="textSecondary">
              No geographic peers connected
            </Typography>
          ) : (
            <TableContainer component={Paper} variant="outlined">
              <Table size="small">
                <TableHead>
                  <TableRow>
                    <TableCell>Region</TableCell>
                    <TableCell>Location</TableCell>
                    <TableCell>Address</TableCell>
                    <TableCell>Quality</TableCell>
                    <TableCell>Latency</TableCell>
                    <TableCell>Reliability</TableCell>
                    <TableCell>Type</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {peers.map((peer) => (
                    <TableRow key={peer.peer_id}>
                      <TableCell>
                        <Chip
                          label={peer.region}
                          size="small"
                          sx={{
                            backgroundColor: getRegionColor(peer.region),
                            color: 'white'
                          }}
                        />
                      </TableCell>
                      <TableCell>
                        {peer.city && peer.country ? (
                          <Tooltip title={`${peer.city}, ${peer.country}`}>
                            <Typography variant="body2">
                              {peer.city}
                            </Typography>
                          </Tooltip>
                        ) : (
                          <Typography variant="body2" color="textSecondary">
                            Unknown
                          </Typography>
                        )}
                      </TableCell>
                      <TableCell sx={{ fontFamily: 'monospace', fontSize: '0.85rem' }}>
                        {peer.address}
                      </TableCell>
                      <TableCell>
                        <Chip
                          label={peer.connection_quality}
                          size="small"
                          color={getQualityColor(peer.connection_quality)}
                        />
                      </TableCell>
                      <TableCell>
                        {peer.latency_ms ? (
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
                            <Speed fontSize="small" />
                            {peer.latency_ms.toFixed(0)}ms
                          </Box>
                        ) : (
                          'N/A'
                        )}
                      </TableCell>
                      <TableCell>
                        <LinearProgress
                          variant="determinate"
                          value={peer.reliability_score * 100}
                          sx={{ 
                            height: 6, 
                            borderRadius: 3,
                            minWidth: 60
                          }}
                          color={getReliabilityColor(peer.reliability_score)}
                        />
                        <Typography variant="caption" color="textSecondary">
                          {(peer.reliability_score * 100).toFixed(0)}%
                        </Typography>
                      </TableCell>
                      <TableCell>
                        <Box sx={{ display: 'flex', gap: 0.5 }}>
                          {peer.is_bootstrap && (
                            <Chip label="Bootstrap" size="small" variant="outlined" />
                          )}
                          {peer.cross_region && (
                            <Chip label="Cross-Region" size="small" variant="outlined" color="warning" />
                          )}
                        </Box>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
          )}
        </CardContent>
      </Card>
    </Box>
  )
}

export default GeographicPeerView