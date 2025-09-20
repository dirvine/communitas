import React, { useState, useEffect } from 'react'
import {
  Box,
  Typography,
  Card,
  CardContent,
  Alert,
  TextField,
  Button,
  Stack,
  Chip,
  Grid,
  LinearProgress,
} from '@mui/material'
import {
  NetworkCheck as NetworkIcon,
  Security as SecurityIcon,
  Storage as StorageIcon,
  People as PeopleIcon,
} from '@mui/icons-material'
import IdentityManager from '../identity/IdentityManager'
import { invoke } from '@tauri-apps/api/core'
import { useAuth } from '../../contexts/AuthContext'

const IdentityTab: React.FC = () => {
  const { authState, getNetworkStatus } = useAuth()
  const [verifyInput, setVerifyInput] = useState('')
  const [verifyLoading, setVerifyLoading] = useState(false)
  const [networkStatus, setNetworkStatus] = useState<{ connected: boolean; peers: number } | null>(null)
  const [verifyResult, setVerifyResult] = useState<{
    status: 'idle' | 'verified' | 'not_found' | 'error'
    message?: string
    packet?: any
    dhtId?: string
  }>({ status: 'idle' })

  // Load network status on component mount
  useEffect(() => {
    const loadNetworkStatus = async () => {
      try {
        const status = await getNetworkStatus()
        setNetworkStatus(status)
      } catch (error) {
        console.error('Failed to get network status:', error)
      }
    }
    loadNetworkStatus()
  }, [getNetworkStatus])

  const handleVerifyFetch = async () => {
    if (!verifyInput.trim()) return
    setVerifyLoading(true)
    setVerifyResult({ status: 'idle' })
    try {
      let dhtId = verifyInput.trim()
      const maybeWords = verifyInput.trim()
      const looksLikeFourWords = /[a-z]+(-[a-z]+){3}/i.test(maybeWords) || maybeWords.split(/\s+/).length === 4
      if (looksLikeFourWords) {
        // Calculate DHT id from four words
        dhtId = await invoke<string>('calculate_dht_id', { fourWords: maybeWords })
      }

      const packet = await invoke<any | null>('get_published_identity', { dhtId, dht_id: dhtId })
      if (packet) {
        // Basic consistency check if input was four words
        if (looksLikeFourWords) {
          const computed = await invoke<string>('calculate_dht_id', { fourWords: packet.four_words })
          if (computed !== packet.dht_id) {
            setVerifyResult({ status: 'error', message: 'Identity data mismatch detected' })
            setVerifyLoading(false)
            return
          }
        }
        setVerifyResult({ status: 'verified', packet, dhtId })
      } else {
        setVerifyResult({ status: 'not_found', message: 'No published identity found' })
      }
    } catch (e: any) {
      setVerifyResult({ status: 'error', message: e?.message || String(e) })
    } finally {
      setVerifyLoading(false)
    }
  }

  return (
    <Box>
      <Typography variant="h5" gutterBottom>
        Identity Management
      </Typography>
      
      <Alert severity="info" sx={{ mb: 3 }}>
        Manage your P2P identities, 4-word addresses, and secure key storage.
      </Alert>

      <Card>
        <CardContent>
          <IdentityManager />
        </CardContent>
      </Card>

      {/* Network Status */}
      <Card sx={{ mt: 3 }}>
        <CardContent>
          <Typography variant="h6" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <NetworkIcon />
            Testnet Status
          </Typography>

          {networkStatus ? (
            <Grid container spacing={2}>
              <Grid item xs={12} sm={6}>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <NetworkIcon color={networkStatus.connected ? 'success' : 'error'} />
                  <Box>
                    <Typography variant="body2" fontWeight={500}>
                      Connection Status
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      {networkStatus.connected ? 'Connected' : 'Disconnected'}
                    </Typography>
                  </Box>
                </Box>
              </Grid>
              <Grid item xs={12} sm={6}>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <PeopleIcon color="primary" />
                  <Box>
                    <Typography variant="body2" fontWeight={500}>
                      Active Peers
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      {networkStatus.peers} connected
                    </Typography>
                  </Box>
                </Box>
              </Grid>
              <Grid item xs={12}>
                <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                  Testnet nodes: localhost:9002-9006
                </Typography>
              </Grid>
            </Grid>
          ) : (
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <LinearProgress sx={{ flex: 1 }} />
              <Typography variant="body2" color="text.secondary">
                Checking network status...
              </Typography>
            </Box>
          )}
        </CardContent>
      </Card>

      {/* Current User Status */}
      {authState.isAuthenticated && authState.user && (
        <Card sx={{ mt: 3 }}>
          <CardContent>
            <Typography variant="h6" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <SecurityIcon />
              Current Identity
            </Typography>
            <Grid container spacing={2}>
              <Grid item xs={12} sm={6}>
                <Typography variant="body2" fontWeight={500}>
                  Name
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {authState.user.name}
                </Typography>
              </Grid>
              <Grid item xs={12} sm={6}>
                <Typography variant="body2" fontWeight={500}>
                  Four-Word Address
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ fontFamily: 'monospace' }}>
                  {authState.user.fourWordAddress}
                </Typography>
              </Grid>
              <Grid item xs={12} sm={6}>
                <Typography variant="body2" fontWeight={500}>
                  Status
                </Typography>
                <Chip
                  size="small"
                  label="Active"
                  color="success"
                  variant="outlined"
                />
              </Grid>
              <Grid item xs={12} sm={6}>
                <Typography variant="body2" fontWeight={500}>
                  Created
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {new Date(authState.user.createdAt).toLocaleDateString()}
                </Typography>
              </Grid>
            </Grid>
          </CardContent>
        </Card>
      )}

      {/* Verify & Fetch Identity */}
      <Card sx={{ mt: 3 }}>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Verify & Fetch Identity
          </Typography>
          <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2} alignItems="flex-start">
            <TextField
              fullWidth
              label="Four-word address or DHT ID"
              placeholder="e.g., ocean-forest-mountain-star or dht id"
              value={verifyInput}
              onChange={(e) => setVerifyInput(e.target.value)}
            />
            <Button variant="contained" onClick={handleVerifyFetch} disabled={verifyLoading}>
              {verifyLoading ? 'Verifying...' : 'Verify'}
            </Button>
          </Stack>

          {verifyResult.status === 'verified' && (
            <Alert severity="success" sx={{ mt: 2 }}>
              Verified identity for DHT ID {verifyResult.dhtId}. Four words: {verifyResult.packet?.four_words}
            </Alert>
          )}
          {verifyResult.status === 'not_found' && (
            <Alert severity="warning" sx={{ mt: 2 }}>
              {verifyResult.message}
            </Alert>
          )}
          {verifyResult.status === 'error' && (
            <Alert severity="error" sx={{ mt: 2 }}>
              {verifyResult.message}
            </Alert>
          )}
        </CardContent>
      </Card>
    </Box>
  )
}

export default IdentityTab
