/**
 * DHT Status Component
 * 
 * Displays real-time DHT network status including:
 * - Connection status and peer count
 * - Network health metrics
 * - Performance indicators
 * - Node information
 */

import React, { useState, useEffect } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography, 
  Chip,
  Grid,
  LinearProgress,
  IconButton,
  Tooltip,
  Alert,
  CircularProgress
} from '@mui/material';
import {
  CloudQueue as NetworkIcon,
  Speed as PerformanceIcon,
  Security as SecurityIcon,
  Refresh as RefreshIcon,
  CheckCircle as ConnectedIcon,
  Error as DisconnectedIcon,
  Warning as WarningIcon
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';

interface DhtStatus {
  node_id: string;
  peer_count: number;
  stored_items: number;
  network_health: number;
  uptime: number;
  performance: {
    avg_lookup_latency: number;
    avg_store_latency: number;
    operation_success_rate: number;
    throughput: number;
    bandwidth_utilization: number;
    memory_usage: number;
  };
}

interface DhtStatusProps {
  className?: string;
}

export const DhtStatus: React.FC<DhtStatusProps> = ({ className }) => {
  const [status, setStatus] = useState<DhtStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());

  const fetchStatus = async () => {
    try {
      setError(null);
      const dhtStatus = await invoke<DhtStatus | null>('get_dht_status');
      setStatus(dhtStatus);
      setLastUpdate(new Date());
    } catch (err) {
      setError(`Failed to fetch DHT status: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStatus();
    
    // Auto-refresh every 10 seconds
    const interval = setInterval(fetchStatus, 10000);
    return () => clearInterval(interval);
  }, []);

  const formatUptime = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);
    
    if (hours > 0) {
      return `${hours}h ${minutes}m ${secs}s`;
    } else if (minutes > 0) {
      return `${minutes}m ${secs}s`;
    } else {
      return `${secs}s`;
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const getHealthColor = (health: number): 'success' | 'warning' | 'error' => {
    if (health >= 0.8) return 'success';
    if (health >= 0.5) return 'warning';
    return 'error';
  };

  const getHealthIcon = (health: number) => {
    if (health >= 0.8) return <ConnectedIcon color="success" />;
    if (health >= 0.5) return <WarningIcon color="warning" />;
    return <DisconnectedIcon color="error" />;
  };

  if (loading) {
    return (
      <Box className={className} display="flex" justifyContent="center" alignItems="center" p={4}>
        <CircularProgress />
        <Typography variant="body2" ml={2}>Loading DHT status...</Typography>
      </Box>
    );
  }

  if (error) {
    return (
      <Box className={className}>
        <Alert severity="error" action={
          <IconButton onClick={fetchStatus} size="small">
            <RefreshIcon />
          </IconButton>
        }>
          {error}
        </Alert>
      </Box>
    );
  }

  if (!status) {
    return (
      <Box className={className}>
        <Alert severity="info">
          DHT is not initialized. Click "Initialize DHT" to connect to the network.
        </Alert>
      </Box>
    );
  }

  return (
    <Box className={className}>
      <Grid container spacing={3}>
        {/* Network Status Card */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Box display="flex" alignItems="center" justifyContent="space-between" mb={2}>
                <Box display="flex" alignItems="center">
                  <NetworkIcon color="primary" sx={{ mr: 1 }} />
                  <Typography variant="h6">Network Status</Typography>
                </Box>
                <Tooltip title="Refresh status">
                  <IconButton onClick={fetchStatus} size="small">
                    <RefreshIcon />
                  </IconButton>
                </Tooltip>
              </Box>
              
              <Box mb={2}>
                <Box display="flex" alignItems="center" mb={1}>
                  {getHealthIcon(status.network_health)}
                  <Typography variant="body2" ml={1}>
                    Network Health: {Math.round(status.network_health * 100)}%
                  </Typography>
                </Box>
                <LinearProgress 
                  variant="determinate" 
                  value={status.network_health * 100}
                  color={getHealthColor(status.network_health)}
                />
              </Box>

              <Grid container spacing={2}>
                <Grid item xs={6}>
                  <Typography variant="body2" color="textSecondary">Peers</Typography>
                  <Typography variant="h6">{status.peer_count}</Typography>
                </Grid>
                <Grid item xs={6}>
                  <Typography variant="body2" color="textSecondary">Uptime</Typography>
                  <Typography variant="h6">{formatUptime(status.uptime)}</Typography>
                </Grid>
              </Grid>

              <Box mt={2}>
                <Typography variant="body2" color="textSecondary">Node ID</Typography>
                <Typography variant="body2" sx={{ 
                  fontFamily: 'monospace', 
                  fontSize: '0.75rem',
                  wordBreak: 'break-all'
                }}>
                  {status.node_id.substring(0, 16)}...
                </Typography>
              </Box>
            </CardContent>
          </Card>
        </Grid>

        {/* Performance Card */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Box display="flex" alignItems="center" mb={2}>
                <PerformanceIcon color="primary" sx={{ mr: 1 }} />
                <Typography variant="h6">Performance</Typography>
              </Box>

              <Grid container spacing={2}>
                <Grid item xs={6}>
                  <Typography variant="body2" color="textSecondary">Lookup Latency</Typography>
                  <Typography variant="h6">{status.performance.avg_lookup_latency}ms</Typography>
                </Grid>
                <Grid item xs={6}>
                  <Typography variant="body2" color="textSecondary">Store Latency</Typography>
                  <Typography variant="h6">{status.performance.avg_store_latency}ms</Typography>
                </Grid>
                <Grid item xs={6}>
                  <Typography variant="body2" color="textSecondary">Success Rate</Typography>
                  <Typography variant="h6">
                    {Math.round(status.performance.operation_success_rate * 100)}%
                  </Typography>
                </Grid>
                <Grid item xs={6}>
                  <Typography variant="body2" color="textSecondary">Throughput</Typography>
                  <Typography variant="h6">{status.performance.throughput.toFixed(1)} ops/s</Typography>
                </Grid>
              </Grid>

              <Box mt={2}>
                <Typography variant="body2" color="textSecondary" mb={1}>
                  Bandwidth Utilization
                </Typography>
                <LinearProgress 
                  variant="determinate" 
                  value={status.performance.bandwidth_utilization * 100}
                  color="primary"
                />
                <Typography variant="caption" color="textSecondary">
                  {Math.round(status.performance.bandwidth_utilization * 100)}%
                </Typography>
              </Box>
            </CardContent>
          </Card>
        </Grid>

        {/* Storage Card */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Box display="flex" alignItems="center" mb={2}>
                <SecurityIcon color="primary" sx={{ mr: 1 }} />
                <Typography variant="h6">Storage</Typography>
              </Box>

              <Grid container spacing={2}>
                <Grid item xs={6}>
                  <Typography variant="body2" color="textSecondary">Stored Items</Typography>
                  <Typography variant="h6">{status.stored_items}</Typography>
                </Grid>
                <Grid item xs={6}>
                  <Typography variant="body2" color="textSecondary">Memory Usage</Typography>
                  <Typography variant="h6">{formatBytes(status.performance.memory_usage)}</Typography>
                </Grid>
              </Grid>

              <Box mt={2}>
                <Typography variant="body2" color="textSecondary" mb={1}>
                  Replication Status
                </Typography>
                <Box display="flex" gap={1}>
                  <Chip label="K=8" size="small" color="primary" />
                  <Chip label="Healthy" size="small" color="success" />
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>

        {/* Quick Stats Card */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" mb={2}>Quick Stats</Typography>
              
              <Box display="flex" flexDirection="column" gap={1}>
                <Box display="flex" justifyContent="space-between">
                  <Typography variant="body2">Status:</Typography>
                  <Chip 
                    label={status.network_health >= 0.8 ? "Connected" : "Degraded"}
                    size="small"
                    color={getHealthColor(status.network_health)}
                  />
                </Box>
                
                <Box display="flex" justifyContent="space-between">
                  <Typography variant="body2">Last Update:</Typography>
                  <Typography variant="body2" color="textSecondary">
                    {lastUpdate.toLocaleTimeString()}
                  </Typography>
                </Box>
                
                <Box display="flex" justifyContent="space-between">
                  <Typography variant="body2">Protocol:</Typography>
                  <Chip label="Kademlia" size="small" variant="outlined" />
                </Box>
                
                <Box display="flex" justifyContent="space-between">
                  <Typography variant="body2">Transport:</Typography>
                  <Chip label="QUIC" size="small" variant="outlined" />
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
      </Grid>
    </Box>
  );
};

export default DhtStatus;
