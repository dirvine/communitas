import React, { useState } from 'react'
import { 
  Box,
  Typography,
  Card,
  CardContent,
  Alert,
  TextField,
  Button,
  Stack,
} from '@mui/material'
import IdentityManager from '../identity/IdentityManager'
import { invoke } from '@tauri-apps/api/core'

const IdentityTab: React.FC = () => {
  const [verifyInput, setVerifyInput] = useState('')
  const [verifyLoading, setVerifyLoading] = useState(false)
  const [verifyResult, setVerifyResult] = useState<{
    status: 'idle' | 'verified' | 'not_found' | 'error'
    message?: string
    packet?: any
    dhtId?: string
  }>({ status: 'idle' })

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
