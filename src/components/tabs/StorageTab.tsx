import React, { useState } from 'react'
import {
  Grid,
  Card,
  CardContent,
  Typography,
  Box,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Chip,
  LinearProgress,
  CircularProgress,
  Divider,
  Alert,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
} from '@mui/material'
import {
  Storage,
  Hub,
  TrendingUp,
  Security,
  CheckCircle,
  Warning,
  Error,
  Schedule,
  CloudSync,
  DataUsage,
  Analytics,
} from '@mui/icons-material'

interface DhtOperation {
  id: string
  operation: 'PUT' | 'GET' | 'DELETE'
  key: string
  status: 'Success' | 'Pending' | 'Failed' | 'Timeout'
  replicas: number
  target_replicas: number
  latency_ms: number
  timestamp: Date
  node_id?: string
}

interface StorageMetrics {
  total_messages: number
  storage_used_mb: number
  dht_operations: number
  replication_factor: number
  success_rate: number
  average_latency: number
  nodes_online: number
  total_keys: number
  storage_growth_mb_per_day: number
}

interface KeyDistribution {
  key_range: string
  key_count: number
  storage_mb: number
  responsible_nodes: number
  health_score: number
}

interface ReplicationStatus {
  key: string
  current_replicas: number
  target_replicas: number
  health: 'Healthy' | 'Degraded' | 'Critical'
  last_verified: Date
}

const StorageTab: React.FC = () => {
  const [metrics] = useState<StorageMetrics>({
    total_messages: 1247,
    storage_used_mb: 15.6,
    dht_operations: 3421,
    replication_factor: 8,
    success_rate: 94.2,
    average_latency: 45,
    nodes_online: 12,
    total_keys: 1247,
    storage_growth_mb_per_day: 2.3,
  })

  const [recentOperations] = useState<DhtOperation[]>([
    {
      id: '1',
      operation: 'PUT',
      key: 'msg_abc123_2024',
      status: 'Success',
      replicas: 8,
      target_replicas: 8,
      latency_ms: 23,
      timestamp: new Date(),
    },
    {
      id: '2',
      operation: 'GET',
      key: 'msg_def456_2024',
      status: 'Success',
      replicas: 6,
      target_replicas: 8,
      latency_ms: 67,
      timestamp: new Date(Date.now() - 30000),
    },
    {
      id: '3',
      operation: 'PUT',
      key: 'msg_ghi789_2024',
      status: 'Pending',
      replicas: 3,
      target_replicas: 8,
      latency_ms: 0,
      timestamp: new Date(Date.now() - 15000),
    },
    {
      id: '4',
      operation: 'GET',
      key: 'msg_jkl012_2024',
      status: 'Failed',
      replicas: 0,
      target_replicas: 8,
      latency_ms: 2500,
      timestamp: new Date(Date.now() - 45000),
    },
    {
      id: '5',
      operation: 'DELETE',
      key: 'msg_old_expired',
      status: 'Success',
      replicas: 8,
      target_replicas: 8,
      latency_ms: 156,
      timestamp: new Date(Date.now() - 60000),
    },
  ])

  const [keyDistribution] = useState<KeyDistribution[]>([
    { key_range: '0000-1FFF', key_count: 156, storage_mb: 2.1, responsible_nodes: 3, health_score: 95 },
    { key_range: '2000-3FFF', key_count: 243, storage_mb: 3.4, responsible_nodes: 3, health_score: 87 },
    { key_range: '4000-5FFF', key_count: 189, storage_mb: 2.8, responsible_nodes: 2, health_score: 72 },
    { key_range: '6000-7FFF', key_count: 201, storage_mb: 2.9, responsible_nodes: 3, health_score: 91 },
    { key_range: '8000-9FFF', key_count: 178, storage_mb: 2.2, responsible_nodes: 3, health_score: 88 },
    { key_range: 'A000-BFFF', key_count: 134, storage_mb: 1.7, responsible_nodes: 2, health_score: 68 },
    { key_range: 'C000-DFFF', key_count: 167, storage_mb: 2.3, responsible_nodes: 3, health_score: 93 },
    { key_range: 'E000-FFFF', key_count: 179, storage_mb: 2.4, responsible_nodes: 3, health_score: 89 },
  ])

  const [replicationStatuses] = useState<ReplicationStatus[]>([
    { key: 'msg_important_data', current_replicas: 8, target_replicas: 8, health: 'Healthy', last_verified: new Date() },
    { key: 'msg_user_profile_456', current_replicas: 6, target_replicas: 8, health: 'Degraded', last_verified: new Date(Date.now() - 300000) },
    { key: 'msg_chat_history_789', current_replicas: 8, target_replicas: 8, health: 'Healthy', last_verified: new Date(Date.now() - 60000) },
    { key: 'msg_file_metadata_123', current_replicas: 3, target_replicas: 8, health: 'Critical', last_verified: new Date(Date.now() - 900000) },
    { key: 'msg_group_settings_abc', current_replicas: 7, target_replicas: 8, health: 'Degraded', last_verified: new Date(Date.now() - 180000) },
  ])

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Success': return 'success'
      case 'Pending': return 'warning'
      case 'Failed': return 'error'
      case 'Timeout': return 'error'
      default: return 'default'
    }
  }

  const getHealthColor = (health: string) => {
    switch (health) {
      case 'Healthy': return 'success'
      case 'Degraded': return 'warning'
      case 'Critical': return 'error'
      default: return 'default'
    }
  }

  const getHealthIcon = (health: string) => {
    switch (health) {
      case 'Healthy': return <CheckCircle color="success" />
      case 'Degraded': return <Warning color="warning" />
      case 'Critical': return <Error color="error" />
      default: return <Schedule />
    }
  }

  const getDistributionHealthColor = (score: number) => {
    if (score >= 90) return 'success.main'
    if (score >= 75) return 'warning.main'
    return 'error.main'
  }

  const totalStorageCapacity = 100
  const storageUsagePercent = (metrics.storage_used_mb / totalStorageCapacity) * 100

  return (
    <Box>
      <Typography variant="h4" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
        <Storage />
        Storage & DHT Diagnostics
      </Typography>
      
      <Grid container spacing={3} sx={{ mb: 3 }}>
        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Typography variant="h6" color="textSecondary" sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <DataUsage />
                Total Messages
              </Typography>
              <Typography variant="h4">{metrics.total_messages}</Typography>
              <Typography variant="caption" color="textSecondary">
                Growing +{metrics.storage_growth_mb_per_day}MB/day
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        
        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Typography variant="h6" color="textSecondary" sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <TrendingUp />
                Storage Used
              </Typography>
              <Typography variant="h4">{metrics.storage_used_mb} MB</Typography>
              <LinearProgress
                variant="determinate"
                value={storageUsagePercent}
                sx={{ mt: 1, height: 6, borderRadius: 3 }}
                color={storageUsagePercent > 80 ? 'error' : storageUsagePercent > 60 ? 'warning' : 'primary'}
              />
              <Typography variant="caption" color="textSecondary">
                {storageUsagePercent.toFixed(1)}% of {totalStorageCapacity}MB
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        
        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Typography variant="h6" color="textSecondary" sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <Analytics />
                Success Rate
              </Typography>
              <Typography variant="h4" color={metrics.success_rate > 95 ? 'success.main' : metrics.success_rate > 90 ? 'warning.main' : 'error.main'}>
                {metrics.success_rate.toFixed(1)}%
              </Typography>
              <Typography variant="caption" color="textSecondary">
                Avg latency: {metrics.average_latency}ms
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        
        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Typography variant="h6" color="textSecondary" sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <CloudSync />
                Replication
              </Typography>
              <Typography variant="h4">K={metrics.replication_factor}</Typography>
              <Typography variant="caption" color="textSecondary">
                {metrics.nodes_online} nodes online
              </Typography>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      <Grid container spacing={3}>
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <Hub />
                DHT Key Distribution Map
              </Typography>
              <Typography variant="body2" color="textSecondary" sx={{ mb: 2 }}>
                Shows how keys are distributed across the DHT ring
              </Typography>
              
              <Box sx={{ position: 'relative', height: 200, mb: 2 }}>
                <Box sx={{ 
                  width: 160, 
                  height: 160, 
                  borderRadius: '50%', 
                  border: '3px solid',
                  borderColor: 'primary.main',
                  position: 'absolute',
                  top: '50%',
                  left: '50%',
                  transform: 'translate(-50%, -50%)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center'
                }}>
                  <Typography variant="h6" color="textSecondary">DHT Ring</Typography>
                </Box>
                
                {keyDistribution.map((range, index) => {
                  const angle = (index * 45) - 90
                  const radius = 100
                  const x = Math.cos(angle * Math.PI / 180) * radius
                  const y = Math.sin(angle * Math.PI / 180) * radius
                  
                  return (
                    <Box
                      key={range.key_range}
                      sx={{
                        position: 'absolute',
                        top: 'calc(50% + ' + y + 'px)',
                        left: 'calc(50% + ' + x + 'px)',
                        transform: 'translate(-50%, -50%)',
                        width: 32,
                        height: 32,
                        borderRadius: '50%',
                        bgcolor: getDistributionHealthColor(range.health_score),
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        color: 'white',
                        fontSize: '0.7rem',
                        fontWeight: 'bold'
                      }}
                      title={range.key_range + ': ' + range.key_count + ' keys, ' + range.storage_mb + 'MB'}
                    >
                      {range.key_count}
                    </Box>
                  )
                })}
              </Box>

              <TableContainer component={Paper} variant="outlined" sx={{ maxHeight: 200 }}>
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell>Range</TableCell>
                      <TableCell>Keys</TableCell>
                      <TableCell>Size</TableCell>
                      <TableCell>Health</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {keyDistribution.map((range) => (
                      <TableRow key={range.key_range}>
                        <TableCell sx={{ fontFamily: 'monospace', fontSize: '0.8rem' }}>{range.key_range}</TableCell>
                        <TableCell>{range.key_count}</TableCell>
                        <TableCell>{range.storage_mb}MB</TableCell>
                        <TableCell>
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                            <Box sx={{ 
                              width: 12, 
                              height: 12, 
                              borderRadius: '50%', 
                              bgcolor: getDistributionHealthColor(range.health_score) 
                            }} />
                            <Typography variant="body2">{range.health_score}%</Typography>
                          </Box>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <Security />
                Replication Health Dashboard
              </Typography>
              
              <Grid container spacing={2} sx={{ mb: 2 }}>
                <Grid item xs={4}>
                  <Box sx={{ textAlign: 'center' }}>
                    <Typography variant="h6" color="success.main">
                      {replicationStatuses.filter(r => r.health === 'Healthy').length}
                    </Typography>
                    <Typography variant="caption">Healthy</Typography>
                  </Box>
                </Grid>
                <Grid item xs={4}>
                  <Box sx={{ textAlign: 'center' }}>
                    <Typography variant="h6" color="warning.main">
                      {replicationStatuses.filter(r => r.health === 'Degraded').length}
                    </Typography>
                    <Typography variant="caption">Degraded</Typography>
                  </Box>
                </Grid>
                <Grid item xs={4}>
                  <Box sx={{ textAlign: 'center' }}>
                    <Typography variant="h6" color="error.main">
                      {replicationStatuses.filter(r => r.health === 'Critical').length}
                    </Typography>
                    <Typography variant="caption">Critical</Typography>
                  </Box>
                </Grid>
              </Grid>

              <Divider sx={{ my: 2 }} />

              {replicationStatuses.some(r => r.health === 'Critical') && (
                <Alert severity="error" sx={{ mb: 2 }}>
                  {replicationStatuses.filter(r => r.health === 'Critical').length} items have critical replication issues
                </Alert>
              )}

              <List dense>
                {replicationStatuses.map((item) => (
                  <ListItem key={item.key}>
                    <ListItemIcon>
                      {getHealthIcon(item.health)}
                    </ListItemIcon>
                    <ListItemText
                      primary={
                        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                          <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
                            {item.key.substring(0, 20)}...
                          </Typography>
                          <Chip
                            label={item.current_replicas + '/' + item.target_replicas}
                            color={getHealthColor(item.health)}
                            size="small"
                          />
                        </Box>
                      }
                      secondary={'Last checked: ' + item.last_verified.toLocaleTimeString()}
                    />
                  </ListItem>
                ))}
              </List>
            </CardContent>
          </Card>
        </Grid>
        
        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <Schedule />
                Recent DHT Operations
              </Typography>
              <TableContainer component={Paper} variant="outlined">
                <Table>
                  <TableHead>
                    <TableRow>
                      <TableCell>Timestamp</TableCell>
                      <TableCell>Operation</TableCell>
                      <TableCell>Key</TableCell>
                      <TableCell>Status</TableCell>
                      <TableCell>Replication</TableCell>
                      <TableCell>Latency</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {recentOperations.map((op) => (
                      <TableRow key={op.id}>
                        <TableCell>
                          <Typography variant="body2" color="textSecondary">
                            {op.timestamp.toLocaleTimeString()}
                          </Typography>
                        </TableCell>
                        <TableCell>
                          <Chip
                            label={op.operation}
                            variant="outlined"
                            size="small"
                            color={op.operation === 'DELETE' ? 'error' : op.operation === 'PUT' ? 'primary' : 'info'}
                          />
                        </TableCell>
                        <TableCell sx={{ fontFamily: 'monospace', maxWidth: 150 }}>
                          <Typography variant="body2" noWrap>
                            {op.key}
                          </Typography>
                        </TableCell>
                        <TableCell>
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                            <Chip
                              label={op.status}
                              color={getStatusColor(op.status)}
                              size="small"
                            />
                            {op.status === 'Pending' && <CircularProgress size={16} />}
                          </Box>
                        </TableCell>
                        <TableCell>
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                            <Typography variant="body2">
                              {op.replicas}/{op.target_replicas}
                            </Typography>
                            <LinearProgress
                              variant="determinate"
                              value={(op.replicas / op.target_replicas) * 100}
                              sx={{ width: 40, height: 4 }}
                              color={op.replicas === op.target_replicas ? 'success' : op.replicas === 0 ? 'error' : 'warning'}
                            />
                          </Box>
                        </TableCell>
                        <TableCell>
                          <Typography variant="body2" color={op.latency_ms > 1000 ? 'error' : op.latency_ms > 200 ? 'warning' : 'textPrimary'}>
                            {op.latency_ms > 0 ? op.latency_ms + 'ms' : '-'}
                          </Typography>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
            </CardContent>
          </Card>
        </Grid>
      </Grid>
    </Box>
  )
}

export default StorageTab
