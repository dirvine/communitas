import React, { useState, useEffect, useCallback, useMemo } from 'react'
import {
  Box,
  Paper,
  Tabs,
  Tab,
  Divider,
  Alert,
  Snackbar,
  LinearProgress,
  Typography,
  IconButton,
  Tooltip,
  Menu,
  MenuItem,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  TextField,
  Switch,
  FormControlLabel,
  Chip
} from '@mui/material'
import {
  Edit,
  Preview,
  CloudUpload,
  Settings,
  Info,
  Group,
  Storage,
  Language,
  Public,
  Security
} from '@mui/icons-material'

import { CollaborativeMarkdownEditor } from '../editor/CollaborativeMarkdownEditor'
import { MarkdownBrowser } from '../browser/MarkdownBrowser'
import { CompleteStorageSystem } from '../../services/storage/CompleteStorageSystem'
import { DHTWebRouter } from '../../services/dht/DHTWebRouter'
import { NetworkIdentity, PersonalUser, Organization, Project } from '../../types/collaboration'
import { AuthGuard, useAuth, usePermissions } from '../security/AuthGuard'

interface TabPanelProps {
  children?: React.ReactNode
  index: number
  value: number
}

function TabPanel({ children, value, index, ...other }: TabPanelProps) {
  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`workspace-tabpanel-${index}`}
      aria-labelledby={`workspace-tab-${index}`}
      {...other}
    >
      {value === index && children}
    </div>
  )
}

interface WebStorageWorkspaceProps {
  storageSystem: CompleteStorageSystem
  dhtRouter: DHTWebRouter
  currentUser: NetworkIdentity
  initialEntity?: PersonalUser | Organization | Project
  initialFile?: string
  onEntityChange?: (entity: PersonalUser | Organization | Project) => void
  className?: string
}

export const WebStorageWorkspace: React.FC<WebStorageWorkspaceProps> = ({
  storageSystem,
  dhtRouter,
  currentUser,
  initialEntity,
  initialFile = 'home.md',
  onEntityChange,
  className
}) => {
  const { isAuthenticated } = useAuth()
  const { canWrite, canCollaborate } = usePermissions()
  const [activeTab, setActiveTab] = useState(0)
  const [currentEntity, setCurrentEntity] = useState(initialEntity)
  const [currentFile, setCurrentFile] = useState(initialFile)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  
  // Editor state
  const [editorContent, setEditorContent] = useState('')
  const [isDirty, setIsDirty] = useState(false)
  const [lastSaved, setLastSaved] = useState<Date | null>(null)
  
  // Settings
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [previewMode, setPreviewMode] = useState(false)
  const [collaborationEnabled, setCollaborationEnabled] = useState(true)
  const [autoSave, setAutoSave] = useState(true)
  const [theme, setTheme] = useState<'light' | 'dark' | 'auto'>('auto')
  
  // Info dialog
  const [infoOpen, setInfoOpen] = useState(false)
  const [systemStats, setSystemStats] = useState<any>(null)

  // Get entity storage
  const getEntityStorage = useCallback(async () => {
    if (!currentEntity) return null
    
    switch (currentEntity.type) {
      case 'personal_user':
        return await storageSystem.getPersonalStorage(currentEntity.id)
      case 'organization':
        return await storageSystem.getOrganizationStorage(currentEntity.id)
      case 'project':
        return await storageSystem.getProjectStorage(currentEntity.id)
      default:
        return null
    }
  }, [currentEntity, storageSystem])

  // Get web publisher
  const getWebPublisher = useCallback(async () => {
    if (!currentEntity) return null
    return await storageSystem.getWebPublisher(currentEntity.id)
  }, [currentEntity, storageSystem])

  // Load file content
  const loadFile = useCallback(async (filePath: string) => {
    if (!currentEntity) return

    setLoading(true)
    setError(null)
    
    try {
      const storage = await getEntityStorage()
      if (storage) {
        const content = await storage.readFile(`/web/${filePath}`)
        setEditorContent(content)
        setCurrentFile(filePath)
        setIsDirty(false)
      }
    } catch (error) {
      console.error('Failed to load file:', error)
      setError(`Failed to load ${filePath}`)
      
      // If file doesn't exist and it's home.md, create it
      if (filePath === 'home.md') {
        const defaultContent = `# Welcome to ${currentEntity.name}

This is your home page on the Communitas P2P network. Your four-word address is: **${currentEntity.networkIdentity.fourWords}**

## Getting Started

- Edit this page by switching to the Editor tab
- Create new pages by adding links like [About](about.md)
- Collaborate in real-time with team members
- Publish changes to make them visible on the DHT network

## Features

- **Real-time collaboration** with Yjs CRDT
- **Distributed storage** with Reed-Solomon encoding
- **End-to-end encryption** for all content
- **Four-word addressing** for human-friendly navigation
- **Version history** and rollback capabilities

Happy collaborating! 🚀
`
        setEditorContent(defaultContent)
        setIsDirty(true)
      }
    } finally {
      setLoading(false)
    }
  }, [currentEntity, getEntityStorage])

  // Save file content
  const saveFile = useCallback(async (content?: string) => {
    if (!currentEntity) return

    // Check permissions before saving
    if (!canWrite('documents', { entityId: currentEntity.id, filePath: currentFile })) {
      setError('You do not have permission to save this document')
      return
    }

    setLoading(true)
    setError(null)
    
    try {
      const storage = await getEntityStorage()
      if (storage) {
        const contentToSave = content || editorContent
        await storage.createFile(`/web/${currentFile}`, contentToSave)
        setLastSaved(new Date())
        setIsDirty(false)
        setSuccess(`Saved ${currentFile}`)
      }
    } catch (error) {
      console.error('Save failed:', error)
      setError('Save failed')
    } finally {
      setLoading(false)
    }
  }, [currentEntity, editorContent, currentFile, getEntityStorage, canWrite])

  // Publish to DHT
  const publishToDHT = useCallback(async () => {
    if (!currentEntity) return

    // Check permissions before publishing
    if (!canWrite('dht_publish', { entityId: currentEntity.id })) {
      setError('You do not have permission to publish this content')
      return
    }

    setLoading(true)
    setError(null)
    
    try {
      const publisher = await getWebPublisher()
      if (publisher) {
        // First ensure the file is saved
        if (isDirty) {
          await saveFile()
        }
        
        // Add current file to publisher
        await publisher.addFile(`/web/${currentFile}`, editorContent)
        
        // Publish to DHT
        const result = await publisher.publish({
          entryPoint: 'home.md',
          theme: theme
        })
        
        // Register with router
        await dhtRouter.registerForwardIdentity(
          currentEntity.networkIdentity,
          result.manifest.version
        )
        
        setSuccess(`Published to DHT: ${currentEntity.networkIdentity.fourWords}`)
        
        // Update browser if on browser tab
        if (activeTab === 1) {
          // Refresh browser content
        }
      }
    } catch (error) {
      console.error('Publish failed:', error)
      setError('Publish to DHT failed')
    } finally {
      setLoading(false)
    }
  }, [currentEntity, isDirty, saveFile, currentFile, editorContent, theme, getWebPublisher, dhtRouter, activeTab, canWrite])

  // Handle content changes
  const handleContentChange = useCallback((content: string) => {
    setEditorContent(content)
    setIsDirty(true)
  }, [])

  // Tab labels and icons
  const tabs = useMemo(() => [
    { label: 'Editor', icon: <Edit fontSize="small" /> },
    { label: 'Preview', icon: <Preview fontSize="small" /> },
    { label: 'Browser', icon: <Language fontSize="small" /> }
  ], [])

  // Load initial file
  useEffect(() => {
    if (currentEntity && currentFile) {
      loadFile(currentFile)
    }
  }, [currentEntity, currentFile, loadFile])

  // Get system stats for info dialog
  useEffect(() => {
    if (infoOpen) {
      const stats = {
        storage: storageSystem.getStats?.() || {},
        router: dhtRouter.getCacheStats(),
        entity: currentEntity
      }
      setSystemStats(stats)
    }
  }, [infoOpen, storageSystem, dhtRouter, currentEntity])

  // Entity display name
  const entityDisplayName = useMemo(() => {
    if (!currentEntity) return 'No Entity'
    return `${currentEntity.name} (${currentEntity.networkIdentity.fourWords})`
  }, [currentEntity])

  return (
    <AuthGuard 
      requireAuth={true}
      requiredPermission={{ action: 'read', resource: 'documents' }}
    >
      <Paper className={className} sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <Box sx={{ borderBottom: 1, borderColor: 'divider', px: 2, py: 1 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <Typography variant="h6" sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <Storage color="primary" />
            {entityDisplayName}
            {isDirty && <Chip label="Unsaved" size="small" color="warning" />}
            {lastSaved && (
              <Chip 
                label={`Saved ${lastSaved.toLocaleTimeString()}`} 
                size="small" 
                color="success" 
                variant="outlined" 
              />
            )}
          </Typography>
          
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            {('members' in (currentEntity as any)) && (currentEntity as any).members && (
              <Tooltip title={`${(currentEntity as any).members.length} collaborators`}>
                <Chip
                  icon={<Group />}
                  label={(currentEntity as any).members.length}
                  size="small"
                  variant="outlined"
                />
              </Tooltip>
            )}
            
            <Tooltip title="Publish to DHT">
              <IconButton
                size="small"
                onClick={publishToDHT}
                disabled={loading || !canWrite('dht_publish', { entityId: currentEntity?.id })}
                color="primary"
              >
                <CloudUpload />
              </IconButton>
            </Tooltip>
            
            <Tooltip title="Settings">
              <IconButton
                size="small"
                onClick={() => setSettingsOpen(true)}
              >
                <Settings />
              </IconButton>
            </Tooltip>
            
            <Tooltip title="System Info">
              <IconButton
                size="small"
                onClick={() => setInfoOpen(true)}
              >
                <Info />
              </IconButton>
            </Tooltip>
          </Box>
        </Box>
      </Box>

      {/* Loading bar */}
      {loading && <LinearProgress />}

      {/* Tabs */}
      <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
        <Tabs value={activeTab} onChange={(_, value) => setActiveTab(value)} variant="fullWidth">
          {tabs.map((tab, index) => (
            <Tab
              key={index}
              label={tab.label}
              icon={tab.icon}
              iconPosition="start"
              id={`workspace-tab-${index}`}
              aria-controls={`workspace-tabpanel-${index}`}
            />
          ))}
        </Tabs>
      </Box>

      {/* Tab Panels */}
      <Box sx={{ flex: 1, overflow: 'hidden' }}>
        {/* Editor Tab */}
        <TabPanel value={activeTab} index={0}>
          {currentEntity && (
            <CollaborativeMarkdownEditor
              filePath={currentFile}
              entityId={currentEntity.id}
              currentUser={currentUser}
              initialContent={editorContent}
              onSave={saveFile}
              onPublish={publishToDHT}
              theme={theme}
              showPreview={previewMode}
              readOnly={!canWrite('documents', { entityId: currentEntity.id, filePath: currentFile })}
              showCollaborators={collaborationEnabled && canCollaborate('documents', { entityId: currentEntity.id })}
              enableVersionHistory={true}
            />
          )}
        </TabPanel>

        {/* Preview Tab */}
        <TabPanel value={activeTab} index={1}>
          {currentEntity && (
            <CollaborativeMarkdownEditor
              filePath={currentFile}
              entityId={currentEntity.id}
              currentUser={currentUser}
              initialContent={editorContent}
              theme={theme}
              showPreview={true}
              readOnly={true}
              showCollaborators={false}
              enableVersionHistory={false}
            />
          )}
        </TabPanel>

        {/* Browser Tab */}
        <TabPanel value={activeTab} index={2}>
          <MarkdownBrowser
            storageSystem={storageSystem}
            currentUser={currentUser}
            initialUrl={currentEntity ? `${currentEntity.networkIdentity.fourWords}/${currentFile}` : ''}
            theme={theme}
            showNavigationHistory={true}
            enableSearch={true}
            enableBookmarks={true}
            showTableOfContents={true}
          />
        </TabPanel>
      </Box>

      {/* Settings Dialog */}
      <Dialog open={settingsOpen} onClose={() => setSettingsOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Workspace Settings</DialogTitle>
        <DialogContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, pt: 1 }}>
            <FormControlLabel
              control={
                <Switch
                  checked={previewMode}
                  onChange={(e) => setPreviewMode(e.target.checked)}
                />
              }
              label="Show live preview in editor"
            />
            
            <FormControlLabel
              control={
                <Switch
                  checked={collaborationEnabled}
                  onChange={(e) => setCollaborationEnabled(e.target.checked)}
                />
              }
              label="Enable real-time collaboration"
            />
            
            <FormControlLabel
              control={
                <Switch
                  checked={autoSave}
                  onChange={(e) => setAutoSave(e.target.checked)}
                />
              }
              label="Auto-save changes"
            />
            
            <TextField
              select
              label="Theme"
              value={theme}
              onChange={(e) => setTheme(e.target.value as 'light' | 'dark' | 'auto')}
              fullWidth
            >
              <MenuItem value="auto">Auto (System)</MenuItem>
              <MenuItem value="light">Light</MenuItem>
              <MenuItem value="dark">Dark</MenuItem>
            </TextField>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setSettingsOpen(false)}>Close</Button>
        </DialogActions>
      </Dialog>

      {/* System Info Dialog */}
      <Dialog open={infoOpen} onClose={() => setInfoOpen(false)} maxWidth="md" fullWidth>
        <DialogTitle>System Information</DialogTitle>
        <DialogContent>
          {systemStats && (
            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, pt: 1 }}>
              <Typography variant="h6">Current Entity</Typography>
              <Box sx={{ pl: 2 }}>
                <Typography><strong>Name:</strong> {currentEntity?.name}</Typography>
                <Typography><strong>Type:</strong> {currentEntity?.type}</Typography>
                <Typography><strong>Four-word Address:</strong> {currentEntity?.networkIdentity.fourWords}</Typography>
                <Typography><strong>DHT Address:</strong> {currentEntity?.networkIdentity.dhtAddress}</Typography>
              </Box>
              
              <Divider />
              
              <Typography variant="h6">Storage System</Typography>
              <Box sx={{ pl: 2 }}>
                <Typography><strong>Active Entities:</strong> {systemStats.storage.activeEntities || 0}</Typography>
                <Typography><strong>Storage Mode:</strong> Reed-Solomon 10+6</Typography>
                <Typography><strong>Encryption:</strong> AES-256-GCM</Typography>
              </Box>
              
              <Divider />
              
              <Typography variant="h6">DHT Router</Typography>
              <Box sx={{ pl: 2 }}>
                <Typography><strong>Content Cache:</strong> {systemStats.router.contentCacheSize} entries</Typography>
                <Typography><strong>Identity Cache:</strong> {systemStats.router.identityCacheSize} identities</Typography>
                <Typography><strong>Publisher Cache:</strong> {systemStats.router.publisherCacheSize} publishers</Typography>
              </Box>
              
              <Divider />
              
              <Typography variant="h6">Network Features</Typography>
              <Box sx={{ pl: 2 }}>
                <Typography>✅ Real-time collaboration with Yjs CRDT</Typography>
                <Typography>✅ Distributed storage with 60% redundancy</Typography>
                <Typography>✅ End-to-end encryption</Typography>
                <Typography>✅ Four-word human-readable addresses</Typography>
                <Typography>✅ Home.md automatic routing</Typography>
                <Typography>✅ Version history and rollback</Typography>
              </Box>
            </Box>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setInfoOpen(false)}>Close</Button>
        </DialogActions>
      </Dialog>

      {/* Notifications */}
      <Snackbar
        open={Boolean(error)}
        autoHideDuration={6000}
        onClose={() => setError(null)}
      >
        <Alert severity="error" onClose={() => setError(null)}>
          {error}
        </Alert>
      </Snackbar>
      
      <Snackbar
        open={Boolean(success)}
        autoHideDuration={3000}
        onClose={() => setSuccess(null)}
      >
        <Alert severity="success" onClose={() => setSuccess(null)}>
          {success}
        </Alert>
      </Snackbar>
    </Paper>
    </AuthGuard>
  )
}