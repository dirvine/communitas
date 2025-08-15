import React, { useEffect, useState } from 'react'
import {
  Box,
  Paper,
  Typography,
  LinearProgress,
  Collapse,
  IconButton,
  Alert,
  Snackbar
} from '@mui/material'
import {
  Close as CloseIcon,
  ExpandLess as ExpandLessIcon,
  ExpandMore as ExpandMoreIcon
} from '@mui/icons-material'
import { useDHTSync, DHTSyncEvent } from '../../hooks/useDHTSync'
import SyncStatusIndicator from './SyncStatusIndicator'

interface GlobalSyncBarProps {
  userId: string
  position?: 'top' | 'bottom'
  autoHide?: boolean
  autoHideDelay?: number
}

export const GlobalSyncBar: React.FC<GlobalSyncBarProps> = ({
  userId,
  position = 'top',
  autoHide = true,
  autoHideDelay = 5000
}) => {
  const [expanded, setExpanded] = useState(false)
  const [visible, setVisible] = useState(true)
  const [recentEvents, setRecentEvents] = useState<DHTSyncEvent[]>([])
  const [showEventAlert, setShowEventAlert] = useState(false)
  const [lastEventMessage, setLastEventMessage] = useState('')

  const handleDHTEvent = (event: DHTSyncEvent) => {
    setRecentEvents(prev => [event, ...prev].slice(0, 10)) // Keep last 10 events
    
    // Create user-friendly message for the event
    const message = getEventMessage(event)
    if (message) {
      setLastEventMessage(message)
      setShowEventAlert(true)
    }
  }

  const {
    connected,
    syncing,
    peerCount,
    lastSync,
    pendingEvents
  } = useDHTSync({
    userId,
    onEvent: handleDHTEvent,
    autoReconnect: true
  })

  // Auto-hide logic
  useEffect(() => {
    if (autoHide && connected && !syncing) {
      const timer = setTimeout(() => {
        setVisible(false)
      }, autoHideDelay)
      return () => clearTimeout(timer)
    } else {
      setVisible(true)
    }
  }, [connected, syncing, autoHide, autoHideDelay])

  // Show bar when disconnected or syncing
  useEffect(() => {
    if (!connected || syncing) {
      setVisible(true)
    }
  }, [connected, syncing])

  const getEventMessage = (event: DHTSyncEvent): string => {
    switch (event.type) {
      case 'OrganizationCreated':
        return 'New organization created'
      case 'OrganizationUpdated':
        return 'Organization updated'
      case 'GroupCreated':
        return 'New group created'
      case 'GroupUpdated':
        return 'Group updated'
      case 'ProjectCreated':
        return 'New project created'
      case 'ProjectUpdated':
        return 'Project updated'
      case 'MemberJoined':
        return 'New member joined'
      case 'MemberLeft':
        return 'Member left'
      case 'FileUploaded':
        return 'File uploaded'
      case 'PeerConnected':
        return `Peer connected: ${event.address}`
      case 'PeerDisconnected':
        return 'Peer disconnected'
      default:
        return ''
    }
  }

  const getBarColor = () => {
    if (!connected) return 'error'
    if (syncing) return 'primary'
    return 'success'
  }

  return (
    <>
      <Collapse in={visible}>
        <Paper
          elevation={2}
          sx={{
            position: 'fixed',
            [position]: 0,
            left: 0,
            right: 0,
            zIndex: 1200,
            borderRadius: 0,
            transition: 'all 0.3s ease'
          }}
        >
          {/* Main Bar */}
          <Box
            sx={{
              px: 2,
              py: 1,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              bgcolor: connected ? 'background.paper' : 'error.light',
              color: connected ? 'text.primary' : 'error.contrastText'
            }}
          >
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, flex: 1 }}>
              <SyncStatusIndicator
                connected={connected}
                syncing={syncing}
                peerCount={peerCount}
                lastSync={lastSync}
                variant="compact"
              />
              
              {syncing && (
                <Box sx={{ flex: 1, maxWidth: 200 }}>
                  <Typography variant="caption">
                    Synchronizing changes...
                  </Typography>
                  <LinearProgress 
                    color={getBarColor() as any}
                    sx={{ mt: 0.5 }}
                  />
                </Box>
              )}
              
              {!connected && (
                <Typography variant="body2">
                  Working offline - changes will sync when reconnected
                </Typography>
              )}
              
              {pendingEvents.length > 0 && (
                <Typography variant="caption" sx={{ ml: 2 }}>
                  {pendingEvents.length} pending update{pendingEvents.length !== 1 ? 's' : ''}
                </Typography>
              )}
            </Box>
            
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              {recentEvents.length > 0 && (
                <IconButton
                  size="small"
                  onClick={() => setExpanded(!expanded)}
                  sx={{ color: 'inherit' }}
                >
                  {expanded ? <ExpandLessIcon /> : <ExpandMoreIcon />}
                </IconButton>
              )}
              
              <IconButton
                size="small"
                onClick={() => setVisible(false)}
                sx={{ color: 'inherit' }}
              >
                <CloseIcon />
              </IconButton>
            </Box>
          </Box>
          
          {/* Expanded Event List */}
          <Collapse in={expanded}>
            <Box
              sx={{
                px: 2,
                py: 1,
                bgcolor: 'background.default',
                borderTop: 1,
                borderColor: 'divider',
                maxHeight: 200,
                overflowY: 'auto'
              }}
            >
              <Typography variant="caption" color="text.secondary" gutterBottom>
                Recent Events
              </Typography>
              {recentEvents.map((event, index) => (
                <Box key={index} sx={{ py: 0.5 }}>
                  <Typography variant="caption">
                    {getEventMessage(event)}
                  </Typography>
                </Box>
              ))}
            </Box>
          </Collapse>
        </Paper>
      </Collapse>
      
      {/* Event Alert Snackbar */}
      <Snackbar
        open={showEventAlert}
        autoHideDuration={3000}
        onClose={() => setShowEventAlert(false)}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
      >
        <Alert 
          onClose={() => setShowEventAlert(false)} 
          severity="info"
          variant="filled"
        >
          {lastEventMessage}
        </Alert>
      </Snackbar>
    </>
  )
}

export default GlobalSyncBar