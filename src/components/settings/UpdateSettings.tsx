import React, { useState, useEffect } from 'react'
import {
  Box,
  Typography,
  Switch,
  FormControlLabel,
  Button,
  LinearProgress,
  Alert,
  Chip,
  Card,
  CardContent,
  Divider,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Stack,
  IconButton,
  Tooltip,
} from '@mui/material'
import { invoke } from '@tauri-apps/api/tauri'
import {
  Refresh as RefreshIcon,
  Download as DownloadIcon,
  CheckCircle as CheckCircleIcon,
  Error as ErrorIcon,
} from '@mui/icons-material'

interface UpdateStatus {
  available: boolean
  current_version: string
  latest_version: string | null
  download_url: string | null
  release_notes: string | null
  checking: boolean
  error: string | null
}

interface UpdateSettings {
  auto_update_enabled: boolean
  check_frequency: number
  update_channel: 'stable' | 'beta'
}

export const UpdateSettings: React.FC = () => {
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus>({
    available: false,
    current_version: '0.1.17',
    latest_version: null,
    download_url: null,
    release_notes: null,
    checking: false,
    error: null,
  })

  const [settings, setSettings] = useState<UpdateSettings>({
    auto_update_enabled: true,
    check_frequency: 6, // hours
    update_channel: 'stable',
  })

  const [updateProgress, setUpdateProgress] = useState(0)
  const [isInstalling, setIsInstalling] = useState(false)

  useEffect(() => {
    loadStatus()
    checkForUpdates()
  }, [])

  const loadStatus = async () => {
    try {
      const status: UpdateStatus = await invoke('get_update_status')
      setUpdateStatus(status)
    } catch (error) {
      console.error('Failed to load update status:', error)
    }
  }

  const checkForUpdates = async () => {
    try {
      setUpdateStatus(prev => ({ ...prev, checking: true, error: null }))
      const status: UpdateStatus = await invoke('check_for_updates')
      setUpdateStatus({ ...status, checking: false })
    } catch (error) {
      setUpdateStatus(prev => ({
        ...prev,
        checking: false,
        error: error as string,
      }))
    }
  }

  const installUpdate = async () => {
    try {
      setIsInstalling(true)
      setUpdateProgress(0)

      // Simulate progress (real progress should come from backend)
      const progressInterval = setInterval(() => {
        setUpdateProgress(prev => Math.min(prev + 10, 90))
      }, 500)

      await invoke('install_update')
      
      clearInterval(progressInterval)
      setUpdateProgress(100)
      
      // Show restart notification
      setTimeout(() => {
        setIsInstalling(false)
        alert('Update installed! The application will restart.')
      }, 1000)
    } catch (error) {
      setIsInstalling(false)
      setUpdateStatus(prev => ({
        ...prev,
        error: error as string,
      }))
    }
  }

  const toggleAutoUpdate = async (enabled: boolean) => {
    try {
      await invoke('set_auto_update', { enabled })
      setSettings(prev => ({ ...prev, auto_update_enabled: enabled }))
    } catch (error) {
      console.error('Failed to toggle auto-update:', error)
    }
  }

  const setCheckFrequency = async (frequency: number) => {
    try {
      await invoke('set_check_frequency', { hours: frequency })
      setSettings(prev => ({ ...prev, check_frequency: frequency }))
    } catch (error) {
      console.error('Failed to set check frequency:', error)
    }
  }

  const setUpdateChannel = async (channel: 'stable' | 'beta') => {
    try {
      await invoke('set_update_channel', { channel })
      setSettings(prev => ({ ...prev, update_channel: channel }))
      // Check for updates in new channel
      await checkForUpdates()
    } catch (error) {
      console.error('Failed to set update channel:', error)
    }
  }

  return (
    <Box sx={{ maxWidth: 800, mx: 'auto', p: 2 }}>
      <Typography variant="h4" gutterBottom>
        Update Settings
      </Typography>

      {/* Update Status Card */}
      <Card sx={{ mb: 3 }}>
        <CardContent>
          <Stack spacing={2}>
            <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <Typography variant="h6">Current Status</Typography>
              <Tooltip title="Check for updates">
                <IconButton onClick={checkForUpdates} disabled={updateStatus.checking}>
                  <RefreshIcon />
                </IconButton>
              </Tooltip>
            </Box>

            <Box sx={{ display: 'flex', gap: 2, alignItems: 'center' }}>
              <Chip 
                label={`v${updateStatus.current_version}`}
                color="primary"
                size="small"
              />
              <Typography>→</Typography>
              {updateStatus.latest_version ? (
                <Chip 
                  label={`v${updateStatus.latest_version}`}
                  color={updateStatus.available ? "success" : "default"}
                  size="small"
                />
              ) : (
                <Typography variant="body2" color="text.secondary">
                  No newer version
                </Typography>
              )}
            </Box>

            {updateStatus.checking && (
              <Box>
                <Typography variant="body2" gutterBottom>
                  Checking for updates...
                </Typography>
                <LinearProgress />
              </Box>
            )}

            {updateStatus.error && (
              <Alert severity="error" sx={{ mb: 2 }}>
                Error checking for updates: {updateStatus.error}
              </Alert>
            )}

            {updateStatus.available && updateStatus.latest_version && (
              <Box>
                <Alert severity="info" sx={{ mb: 2 }}>
                  <Typography variant="body2" gutterBottom>
                    Update available: v{updateStatus.latest_version}
                  </Typography>
                  {updateStatus.release_notes && (
                    <Typography variant="body2" component="div">
                      {updateStatus.release_notes}
                    </Typography>
                  )}
                </Alert>

                <Button
                  variant="contained"
                  startIcon={<DownloadIcon />}
                  onClick={installUpdate}
                  disabled={isInstalling}
                  fullWidth
                >
                  {isInstalling ? 'Installing...' : 'Install Update'}
                </Button>

                {isInstalling && (
                  <Box sx={{ mt: 2 }}>
                    <Typography variant="body2" gutterBottom>
                      Installing update...
                    </Typography>
                    <LinearProgress 
                      variant="determinate" 
                      value={updateProgress}
                    />
                  </Box>
                )}
              </Box>
            )}
          </Stack>
        </CardContent>
      </Card>

      {/* Settings Card */}
      <Card sx={{ mb: 3 }}>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Automatic Updates
          </Typography>

          <Stack spacing={3}>
            <FormControlLabel
              control={
                <Switch
                  checked={settings.auto_update_enabled}
                  onChange={(e) => toggleAutoUpdate(e.target.checked)}
                />
              }
              label="Enable automatic updates"
            />

            <FormControl fullWidth>
              <InputLabel>Check Frequency</InputLabel>
              <Select
                value={settings.check_frequency}
                label="Check Frequency"
                onChange={(e) => setCheckFrequency(e.target.value as number)}
                disabled={!settings.auto_update_enabled}
              >
                <MenuItem value={1}>Every hour</MenuItem>
                <MenuItem value={6}>Every 6 hours</MenuItem>
                <MenuItem value={12}>Every 12 hours</MenuItem>
                <MenuItem value={24}>Daily</MenuItem>
                <MenuItem value={168}>Weekly</MenuItem>
              </Select>
            </FormControl>

            <FormControl fullWidth>
              <InputLabel>Update Channel</InputLabel>
              <Select
                value={settings.update_channel}
                label="Update Channel"
                onChange={(e) => setUpdateChannel(e.target.value as 'stable' | 'beta')}
              >
                <MenuItem value="stable">
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                    <Typography>Stable</Typography>
                    <Chip label="Recommended" size="small" color="success" />
                  </Box>
                </MenuItem>
                <MenuItem value="beta">
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                    <Typography>Beta</Typography>
                    <Chip label="Early Access" size="small" color="warning" />
                  </Box>
                </MenuItem>
              </Select>
            </FormControl>

            <Divider />

            <Alert severity="info">
              <Typography variant="body2">
                <strong>Post-Quantum Security:</strong> All updates are signed with ML-DSA 
                signatures and verified automatically.
              </Typography>
            </Alert>
          </Stack>
        </CardContent>
      </Card>

      {/* Security Card */}
      <Card>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Security & Verification
          </Typography>

          <Stack spacing={2}>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <CheckCircleIcon color="success" fontSize="small" />
              <Typography variant="body2">
                Updates signed with ML-DSA post-quantum signatures
              </Typography>
            </Box>

            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <CheckCircleIcon color="success" fontSize="small" />
              <Typography variant="body2">
                Automatic signature verification
              </Typography>
            </Box>

            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <CheckCircleIcon color="success" fontSize="small" />
              <Typography variant="body2">
                Rollback protection prevents downgrades
              </Typography>
            </Box>

            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <CheckCircleIcon color="success" fontSize="small" />
              <Typography variant="body2">
                Delta updates minimize download size
              </Typography>
            </Box>
          </Stack>
        </CardContent>
      </Card>
    </Box>
  )
}
