import React, { useState } from 'react'
import {
  Box,
  Paper,
  Typography,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  ListItemButton,
  Switch,
  FormControlLabel,
  TextField,
  Button,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Slider,
  Card,
  CardContent,
  Divider,




  Alert,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
} from '@mui/material'
import {
  Person,
  Palette,
  Security,
  Notifications,
  Hub,
  Storage,

  Videocam,


  Info,
  Warning,
  CheckCircle,
  ImportExport,
  Backup,
  Restore,
} from '@mui/icons-material'

interface SettingsData {
  profile: {
    displayName: string
    avatar: string
    status: string
    fourWordAddress: string
  }
  appearance: {
    theme: 'light' | 'dark' | 'auto'
    language: string
    fontSize: number
    compactMode: boolean
    animations: boolean
  }
  privacy: {
    showOnlineStatus: boolean
    allowDirectMessages: boolean
    shareTypingIndicators: boolean
    dataCollection: boolean
    crashReports: boolean
  }
  notifications: {
    desktopNotifications: boolean
    soundNotifications: boolean
    messageNotifications: boolean
    callNotifications: boolean
    emailDigest: boolean
  }
  network: {
    autoConnect: boolean
    preferredRegion: string
    maxPeers: number
    bandwidthLimit: number
    useRelay: boolean
  }
  storage: {
    cacheSize: number
    downloadPath: string
    autoCleanup: boolean
    backupEnabled: boolean
    syncSettings: boolean
  }
  media: {
    microphoneDevice: string
    cameraDevice: string
    speakerDevice: string
    microphoneGain: number
    cameraQuality: string
    echoCancellation: boolean
  }
}

const defaultSettings: SettingsData = {
  profile: {
    displayName: 'John Doe',
    avatar: '/avatar-default.jpg',
    status: 'Available',
    fourWordAddress: 'calm-river-mountain-dawn',
  },
  appearance: {
    theme: 'light',
    language: 'English',
    fontSize: 14,
    compactMode: false,
    animations: true,
  },
  privacy: {
    showOnlineStatus: true,
    allowDirectMessages: true,
    shareTypingIndicators: true,
    dataCollection: false,
    crashReports: true,
  },
  notifications: {
    desktopNotifications: true,
    soundNotifications: true,
    messageNotifications: true,
    callNotifications: true,
    emailDigest: false,
  },
  network: {
    autoConnect: true,
    preferredRegion: 'auto',
    maxPeers: 50,
    bandwidthLimit: 0,
    useRelay: true,
  },
  storage: {
    cacheSize: 500,
    downloadPath: '/Downloads',
    autoCleanup: true,
    backupEnabled: true,
    syncSettings: true,
  },
  media: {
    microphoneDevice: 'default',
    cameraDevice: 'default',
    speakerDevice: 'default',
    microphoneGain: 70,
    cameraQuality: 'HD',
    echoCancellation: true,
  },
}

const settingsCategories = [
  { id: 'profile', label: 'Profile', icon: <Person /> },
  { id: 'appearance', label: 'Appearance', icon: <Palette /> },
  { id: 'privacy', label: 'Privacy & Security', icon: <Security /> },
  { id: 'notifications', label: 'Notifications', icon: <Notifications /> },
  { id: 'network', label: 'Network', icon: <Hub /> },
  { id: 'storage', label: 'Storage', icon: <Storage /> },
  { id: 'media', label: 'Audio & Video', icon: <Videocam /> },
]

export default function SettingsInterface() {
  const [settings, setSettings] = useState<SettingsData>(defaultSettings)
  const [selectedCategory, setSelectedCategory] = useState('profile')
  const [importDialogOpen, setImportDialogOpen] = useState(false)
  const [exportDialogOpen, setExportDialogOpen] = useState(false)
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false)

  const updateSettings = <T extends keyof SettingsData>(
    category: T,
    key: keyof SettingsData[T],
    value: any
  ) => {
    setSettings(prev => ({
      ...prev,
      [category]: {
        ...prev[category],
        [key]: value,
      },
    }))
    setHasUnsavedChanges(true)
  }

  const handleSaveSettings = () => {
    console.log('Saving settings:', settings)
    setHasUnsavedChanges(false)
    // In real implementation, save to storage/network
  }

  const handleResetSettings = () => {
    setSettings(defaultSettings)
    setHasUnsavedChanges(true)
  }

  const handleExportSettings = () => {
    const dataStr = JSON.stringify(settings, null, 2)
    const dataBlob = new Blob([dataStr], { type: 'application/json' })
    const url = URL.createObjectURL(dataBlob)
    const link = document.createElement('a')
    link.href = url
    link.download = 'communitas-settings.json'
    link.click()
    setExportDialogOpen(false)
  }

  const handleImportSettings = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (file) {
      const reader = new FileReader()
      reader.onload = (e) => {
        try {
          const importedSettings = JSON.parse(e.target?.result as string)
          setSettings(importedSettings)
          setHasUnsavedChanges(true)
          setImportDialogOpen(false)
        } catch (error) {
          console.error('Error importing settings:', error)
        }
      }
      reader.readAsText(file)
    }
  }

  const renderProfileSettings = () => (
    <Card>
      <CardContent>
        <Typography variant="h6" gutterBottom>
          Profile Information
        </Typography>
        
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
          <TextField
            label="Display Name"
            value={settings.profile.displayName}
            onChange={(e) => updateSettings('profile', 'displayName', e.target.value)}
            fullWidth
          />
          
          <TextField
            label="Four-Word Address"
            value={settings.profile.fourWordAddress}
            disabled
            fullWidth
            helperText="This is your unique P2P address"
          />
          
          <FormControl fullWidth>
            <InputLabel>Status</InputLabel>
            <Select
              value={settings.profile.status}
              onChange={(e) => updateSettings('profile', 'status', e.target.value)}
            >
              <MenuItem value="Available">Available</MenuItem>
              <MenuItem value="Busy">Busy</MenuItem>
              <MenuItem value="Away">Away</MenuItem>
              <MenuItem value="Invisible">Invisible</MenuItem>
            </Select>
          </FormControl>
        </Box>
      </CardContent>
    </Card>
  )

  const renderAppearanceSettings = () => (
    <Card>
      <CardContent>
        <Typography variant="h6" gutterBottom>
          Appearance & Display
        </Typography>
        
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
          <FormControl>
            <InputLabel>Theme</InputLabel>
            <Select
              value={settings.appearance.theme}
              onChange={(e) => updateSettings('appearance', 'theme', e.target.value)}
            >
              <MenuItem value="light">Light</MenuItem>
              <MenuItem value="dark">Dark</MenuItem>
              <MenuItem value="auto">Auto</MenuItem>
            </Select>
          </FormControl>
          
          <Box>
            <Typography gutterBottom>Font Size</Typography>
            <Slider
              value={settings.appearance.fontSize}
              onChange={(_e, value) => updateSettings('appearance', 'fontSize', value)}
              min={10}
              max={24}
              step={1}
              valueLabelDisplay="auto"
              marks={[
                { value: 12, label: 'Small' },
                { value: 16, label: 'Medium' },
                { value: 20, label: 'Large' },
              ]}
            />
          </Box>
          
          <FormControlLabel
            control={
              <Switch
                checked={settings.appearance.compactMode}
                onChange={(e) => updateSettings('appearance', 'compactMode', e.target.checked)}
              />
            }
            label="Compact Mode"
          />
          
          <FormControlLabel
            control={
              <Switch
                checked={settings.appearance.animations}
                onChange={(e) => updateSettings('appearance', 'animations', e.target.checked)}
              />
            }
            label="Enable Animations"
          />
        </Box>
      </CardContent>
    </Card>
  )

  const renderPrivacySettings = () => (
    <Card>
      <CardContent>
        <Typography variant="h6" gutterBottom>
          Privacy & Security
        </Typography>
        
        <List>
          <ListItem>
            <ListItemIcon><Person /></ListItemIcon>
            <ListItemText
              primary="Show Online Status"
              secondary="Let others see when you're online"
            />
            <Switch
              checked={settings.privacy.showOnlineStatus}
              onChange={(e) => updateSettings('privacy', 'showOnlineStatus', e.target.checked)}
            />
          </ListItem>
          
          <ListItem>
            <ListItemIcon><Security /></ListItemIcon>
            <ListItemText
              primary="Allow Direct Messages"
              secondary="Receive messages from anyone"
            />
            <Switch
              checked={settings.privacy.allowDirectMessages}
              onChange={(e) => updateSettings('privacy', 'allowDirectMessages', e.target.checked)}
            />
          </ListItem>
          
          <ListItem>
            <ListItemIcon><Info /></ListItemIcon>
            <ListItemText
              primary="Share Typing Indicators"
              secondary="Show when you're typing"
            />
            <Switch
              checked={settings.privacy.shareTypingIndicators}
              onChange={(e) => updateSettings('privacy', 'shareTypingIndicators', e.target.checked)}
            />
          </ListItem>
          
          <Divider />
          
          <ListItem>
            <ListItemIcon><Warning /></ListItemIcon>
            <ListItemText
              primary="Data Collection"
              secondary="Allow anonymous usage analytics"
            />
            <Switch
              checked={settings.privacy.dataCollection}
              onChange={(e) => updateSettings('privacy', 'dataCollection', e.target.checked)}
            />
          </ListItem>
          
          <ListItem>
            <ListItemIcon><CheckCircle /></ListItemIcon>
            <ListItemText
              primary="Crash Reports"
              secondary="Send crash reports to help improve Communitas"
            />
            <Switch
              checked={settings.privacy.crashReports}
              onChange={(e) => updateSettings('privacy', 'crashReports', e.target.checked)}
            />
          </ListItem>
        </List>
      </CardContent>
    </Card>
  )

  const renderNotificationSettings = () => (
    <Card>
      <CardContent>
        <Typography variant="h6" gutterBottom>
          Notifications
        </Typography>
        
        <List>
          <ListItem>
            <ListItemText primary="Desktop Notifications" />
            <Switch
              checked={settings.notifications.desktopNotifications}
              onChange={(e) => updateSettings('notifications', 'desktopNotifications', e.target.checked)}
            />
          </ListItem>
          
          <ListItem>
            <ListItemText primary="Sound Notifications" />
            <Switch
              checked={settings.notifications.soundNotifications}
              onChange={(e) => updateSettings('notifications', 'soundNotifications', e.target.checked)}
            />
          </ListItem>
          
          <ListItem>
            <ListItemText primary="Message Notifications" />
            <Switch
              checked={settings.notifications.messageNotifications}
              onChange={(e) => updateSettings('notifications', 'messageNotifications', e.target.checked)}
            />
          </ListItem>
          
          <ListItem>
            <ListItemText primary="Call Notifications" />
            <Switch
              checked={settings.notifications.callNotifications}
              onChange={(e) => updateSettings('notifications', 'callNotifications', e.target.checked)}
            />
          </ListItem>
        </List>
      </CardContent>
    </Card>
  )

  const renderMediaSettings = () => (
    <Card>
      <CardContent>
        <Typography variant="h6" gutterBottom>
          Audio & Video Settings
        </Typography>
        
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
          <FormControl fullWidth>
            <InputLabel>Microphone</InputLabel>
            <Select
              value={settings.media.microphoneDevice}
              onChange={(e) => updateSettings('media', 'microphoneDevice', e.target.value)}
            >
              <MenuItem value="default">Default Microphone</MenuItem>
              <MenuItem value="built-in">Built-in Microphone</MenuItem>
            </Select>
          </FormControl>
          
          <Box>
            <Typography gutterBottom>Microphone Gain</Typography>
            <Slider
              value={settings.media.microphoneGain}
              onChange={(_e, value) => updateSettings('media', 'microphoneGain', value)}
              min={0}
              max={100}
              valueLabelDisplay="auto"
            />
          </Box>
          
          <FormControl fullWidth>
            <InputLabel>Camera</InputLabel>
            <Select
              value={settings.media.cameraDevice}
              onChange={(e) => updateSettings('media', 'cameraDevice', e.target.value)}
            >
              <MenuItem value="default">Default Camera</MenuItem>
              <MenuItem value="built-in">Built-in Camera</MenuItem>
            </Select>
          </FormControl>
          
          <FormControl fullWidth>
            <InputLabel>Camera Quality</InputLabel>
            <Select
              value={settings.media.cameraQuality}
              onChange={(e) => updateSettings('media', 'cameraQuality', e.target.value)}
            >
              <MenuItem value="HD">HD (720p)</MenuItem>
              <MenuItem value="FHD">Full HD (1080p)</MenuItem>
              <MenuItem value="4K">4K (2160p)</MenuItem>
            </Select>
          </FormControl>
          
          <FormControlLabel
            control={
              <Switch
                checked={settings.media.echoCancellation}
                onChange={(e) => updateSettings('media', 'echoCancellation', e.target.checked)}
              />
            }
            label="Echo Cancellation"
          />
        </Box>
      </CardContent>
    </Card>
  )

  const renderContent = () => {
    switch (selectedCategory) {
      case 'profile': return renderProfileSettings()
      case 'appearance': return renderAppearanceSettings()
      case 'privacy': return renderPrivacySettings()
      case 'notifications': return renderNotificationSettings()
      case 'media': return renderMediaSettings()
      default: return renderProfileSettings()
    }
  }

  return (
    <Box sx={{ display: 'flex', height: '100%' }}>
      {/* Settings categories sidebar */}
      <Paper sx={{ width: 280, mr: 2 }}>
        <Box sx={{ p: 2 }}>
          <Typography variant="h6" gutterBottom>
            Settings
          </Typography>
          
          <List>
            {settingsCategories.map((category) => (
              <ListItemButton
                key={category.id}
                selected={selectedCategory === category.id}
                onClick={() => setSelectedCategory(category.id)}
              >
                <ListItemIcon>{category.icon}</ListItemIcon>
                <ListItemText primary={category.label} />
              </ListItemButton>
            ))}
          </List>
          
          <Divider sx={{ my: 2 }} />
          
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
            <Button
              startIcon={<ImportExport />}
              onClick={() => setImportDialogOpen(true)}
              fullWidth
            >
              Import
            </Button>
            <Button
              startIcon={<Backup />}
              onClick={() => setExportDialogOpen(true)}
              fullWidth
            >
              Export
            </Button>
            <Button
              startIcon={<Restore />}
              onClick={handleResetSettings}
              color="warning"
              fullWidth
            >
              Reset
            </Button>
          </Box>
        </Box>
      </Paper>

      {/* Settings content */}
      <Box sx={{ flexGrow: 1 }}>
        <Box sx={{ mb: 2 }}>
          {hasUnsavedChanges && (
            <Alert severity="warning" sx={{ mb: 2 }}>
              You have unsaved changes. Don't forget to save your settings.
            </Alert>
          )}
        </Box>
        
        {renderContent()}
        
        <Box sx={{ mt: 3, display: 'flex', gap: 2 }}>
          <Button
            variant="contained"
            onClick={handleSaveSettings}
            disabled={!hasUnsavedChanges}
          >
            Save Changes
          </Button>
          <Button
            variant="outlined"
            onClick={() => setSettings(defaultSettings)}
          >
            Discard
          </Button>
        </Box>
      </Box>

      {/* Import dialog */}
      <Dialog open={importDialogOpen} onClose={() => setImportDialogOpen(false)}>
        <DialogTitle>Import Settings</DialogTitle>
        <DialogContent>
          <Typography variant="body2" sx={{ mb: 2 }}>
            Select a settings file to import your configuration.
          </Typography>
          <input
            type="file"
            accept=".json"
            onChange={handleImportSettings}
            style={{ width: '100%' }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setImportDialogOpen(false)}>Cancel</Button>
        </DialogActions>
      </Dialog>

      {/* Export dialog */}
      <Dialog open={exportDialogOpen} onClose={() => setExportDialogOpen(false)}>
        <DialogTitle>Export Settings</DialogTitle>
        <DialogContent>
          <Typography variant="body2">
            Export your current settings to a file for backup or transfer.
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setExportDialogOpen(false)}>Cancel</Button>
          <Button variant="contained" onClick={handleExportSettings}>
            Export
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  )
}