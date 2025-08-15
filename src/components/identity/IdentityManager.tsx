import { invoke } from '@tauri-apps/api/core'
import React, { useState, useEffect } from 'react'
import {
  Box,
  Typography,
  Button,
  Alert,
  CircularProgress,
  Snackbar,
  Card,
  CardContent,
  Grid,
  Chip,
  Stack,
} from '@mui/material'
import {
  Add as AddIcon,
  Refresh as RefreshIcon,
  Storage as StorageIcon,
} from '@mui/icons-material'
import { IdentityInfo, StorageBackendInfo } from '../../types'
import IdentityCard from './IdentityCard'
import IdentitySetup from './IdentitySetup'

const IdentityManager: React.FC = () => {
  const [currentIdentity, setCurrentIdentity] = useState<IdentityInfo | null>(null)
  const [identities, setIdentities] = useState<IdentityInfo[]>([])
  const [storageInfo, setStorageInfo] = useState<StorageBackendInfo | null>(null)
  const [loading, setLoading] = useState(true)
  const [showSetup, setShowSetup] = useState(false)
  const [snackbarMessage, setSnackbarMessage] = useState('')
  const [error, setError] = useState<string | null>(null)

  // Load identity data on component mount
  useEffect(() => {
    loadIdentityData()
  }, [])

  const loadIdentityData = async () => {
    setLoading(true)
    setError(null)

    try {
      // Load current identity
      const current = await invoke<IdentityInfo>('get_identity')
      setCurrentIdentity(current)

      // Load all identities
      const allIdentities = await invoke<IdentityInfo[]>('list_identities')
      setIdentities(allIdentities)

      // Load storage info
      const storage = await invoke<StorageBackendInfo>('get_storage_info')
      setStorageInfo(storage)

    } catch (err) {
      setError(`Failed to load identity data: ${err}`)
    } finally {
      setLoading(false)
    }
  }

  const handleCreateIdentity = () => {
    setShowSetup(true)
  }

  const handleIdentityCreated = (identity: IdentityInfo) => {
    setCurrentIdentity(identity)
    setIdentities([identity, ...identities])
    setSnackbarMessage('Identity created successfully!')
    setShowSetup(false)
  }

  const handleCopyAddress = () => {
    setSnackbarMessage('Address copied to clipboard!')
  }

  const handleRefresh = () => {
    loadIdentityData()
  }

  if (loading) {
    return (
      <Box display="flex" justify-content="center" align-items="center" min-height="400px">
        <CircularProgress />
      </Box>
    )
  }

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Identity Management
      </Typography>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      {/* No identity state */}
      {!currentIdentity && !loading && (
        <Box textAlign="center" py={4}>
          <Typography variant="h6" gutterBottom>
            No Identity Found
          </Typography>
          <Typography variant="body1" color="text.secondary" sx={{ mb: 3 }}>
            You need to create an identity to use Communitas. This will generate
            a unique 4-word address that others can use to connect to you.
          </Typography>
          <Button
            variant="contained"
            size="large"
            startIcon={<AddIcon />}
            onClick={handleCreateIdentity}
          >
            Create New Identity
          </Button>
        </Box>
      )}

      {/* Current identity display */}
      {currentIdentity && (
        <Grid container spacing={3}>
          <Grid xs={12} md={8}>
            <Typography variant="h6" gutterBottom>
              Current Identity
            </Typography>
            <IdentityCard
              identity={currentIdentity}
              onCopyAddress={handleCopyAddress}
            />
          </Grid>

          <Grid xs={12} md={4}>
            <Stack spacing={2}>
              {/* Storage info */}
              {storageInfo && (
                <Card>
                  <CardContent>
                    <Box display="flex" alignItems="center" gap={1} mb={1}>
                      <StorageIcon color="primary" />
                      <Typography variant="h6">Storage Backend</Typography>
                    </Box>
                    <Typography variant="body2" color="text.secondary" gutterBottom>
                      {storageInfo.backend_type}
                    </Typography>
                    <Box display="flex" justify-content="space-between" align-items="center">
                      <Typography variant="body2">
                        {storageInfo.key_count} keys stored
                      </Typography>
                      <Chip
                        size="small"
                        label={storageInfo.is_available ? 'Available' : 'Unavailable'}
                        color={storageInfo.is_available ? 'success' : 'error'}
                      />
                    </Box>
                  </CardContent>
                </Card>
              )}

              {/* Actions */}
              <Card>
                <CardContent>
                  <Typography variant="h6" gutterBottom>
                    Actions
                  </Typography>
                  <Stack spacing={1}>
                    <Button
                      variant="outlined"
                      startIcon={<AddIcon />}
                      onClick={handleCreateIdentity}
                      fullWidth
                    >
                      Create New Identity
                    </Button>
                    <Button
                      variant="outlined"
                      startIcon={<RefreshIcon />}
                      onClick={handleRefresh}
                      fullWidth
                    >
                      Refresh
                    </Button>
                  </Stack>
                </CardContent>
              </Card>
            </Stack>
          </Grid>
        </Grid>
      )}

      {/* All identities list */}
      {identities.length > 1 && (
        <Box sx={{ mt: 4 }}>
          <Typography variant="h6" gutterBottom>
            All Identities ({identities.length})
          </Typography>
          <Grid container spacing={2}>
            {identities.map((identity, index) => (
              <Grid xs={12} md={6} key={index}>
                <IdentityCard
                  identity={identity}
                  onCopyAddress={handleCopyAddress}
                />
              </Grid>
            ))}
          </Grid>
        </Box>
      )}

      {/* Identity Setup Dialog */}
      <IdentitySetup
        open={showSetup}
        onClose={() => setShowSetup(false)}
        onIdentityCreated={handleIdentityCreated}
      />

      {/* Success Snackbar */}
      <Snackbar
        open={!!snackbarMessage}
        autoHideDuration={3000}
        onClose={() => setSnackbarMessage('')}
        message={snackbarMessage}
      />
    </Box>
  )
}

export default IdentityManager
