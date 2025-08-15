import React from 'react'
import {
  Box,
  Typography,
  Badge,
  Tooltip,
  IconButton,
  Popover,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  Divider,
  CircularProgress
} from '@mui/material'
import {
  Sync as SyncIcon,
  SyncDisabled as SyncDisabledIcon,
  Cloud as CloudIcon,
  CloudOff as CloudOffIcon,
  People as PeopleIcon,
  Update as UpdateIcon,
  Check as CheckIcon,
  Warning as WarningIcon
} from '@mui/icons-material'
import { format } from 'date-fns'

interface SyncStatusIndicatorProps {
  connected: boolean
  syncing: boolean
  peerCount: number
  lastSync?: Date
  variant?: 'icon' | 'compact' | 'detailed'
  showDetails?: boolean
}

export const SyncStatusIndicator: React.FC<SyncStatusIndicatorProps> = ({
  connected,
  syncing,
  peerCount,
  lastSync,
  variant = 'compact',
  showDetails = false
}) => {
  const [anchorEl, setAnchorEl] = React.useState<HTMLButtonElement | null>(null)

  const handleClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    if (showDetails) {
      setAnchorEl(event.currentTarget)
    }
  }

  const handleClose = () => {
    setAnchorEl(null)
  }

  const open = Boolean(anchorEl)

  const getStatusColor = () => {
    if (!connected) return 'error'
    if (syncing) return 'primary'
    return 'success'
  }

  const getStatusText = () => {
    if (!connected) return 'Offline'
    if (syncing) return 'Syncing...'
    return 'Connected'
  }

  const getStatusIcon = () => {
    if (!connected) {
      return <SyncDisabledIcon color="error" />
    }
    
    if (syncing) {
      return (
        <SyncIcon 
          color="primary"
          sx={{ 
            animation: 'spin 2s linear infinite',
            '@keyframes spin': {
              '0%': { transform: 'rotate(0deg)' },
              '100%': { transform: 'rotate(360deg)' }
            }
          }} 
        />
      )
    }
    
    return <SyncIcon color="success" />
  }

  if (variant === 'icon') {
    return (
      <>
        <Tooltip title={`${getStatusText()} • ${peerCount} peers`}>
          <IconButton 
            size="small" 
            onClick={handleClick}
            sx={{ 
              color: `${getStatusColor()}.main`
            }}
          >
            <Badge badgeContent={peerCount} color={getStatusColor()}>
              {getStatusIcon()}
            </Badge>
          </IconButton>
        </Tooltip>
        
        <Popover
          open={open}
          anchorEl={anchorEl}
          onClose={handleClose}
          anchorOrigin={{
            vertical: 'bottom',
            horizontal: 'right',
          }}
          transformOrigin={{
            vertical: 'top',
            horizontal: 'right',
          }}
        >
          <SyncStatusDetails
            connected={connected}
            syncing={syncing}
            peerCount={peerCount}
            lastSync={lastSync}
          />
        </Popover>
      </>
    )
  }

  if (variant === 'detailed') {
    return (
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
        <Badge badgeContent={peerCount} color={getStatusColor()}>
          {getStatusIcon()}
        </Badge>
        <Box>
          <Typography variant="body2" fontWeight="medium">
            {getStatusText()}
          </Typography>
          <Typography variant="caption" color="text.secondary">
            {peerCount} peer{peerCount !== 1 ? 's' : ''} connected
            {lastSync && ` • Last sync: ${format(lastSync, 'HH:mm:ss')}`}
          </Typography>
        </Box>
      </Box>
    )
  }

  // Compact variant (default)
  return (
    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
      <Badge badgeContent={peerCount} color={getStatusColor()}>
        {getStatusIcon()}
      </Badge>
      <Typography variant="caption" color="text.secondary">
        {getStatusText()}
      </Typography>
    </Box>
  )
}

interface SyncStatusDetailsProps {
  connected: boolean
  syncing: boolean
  peerCount: number
  lastSync?: Date
}

const SyncStatusDetails: React.FC<SyncStatusDetailsProps> = ({
  connected,
  syncing,
  peerCount,
  lastSync
}) => {
  return (
    <Box sx={{ p: 2, minWidth: 280 }}>
      <Typography variant="subtitle2" gutterBottom>
        Network Status
      </Typography>
      
      <List dense>
        <ListItem>
          <ListItemIcon>
            {connected ? (
              <CloudIcon color="success" />
            ) : (
              <CloudOffIcon color="error" />
            )}
          </ListItemIcon>
          <ListItemText
            primary="Connection"
            secondary={connected ? 'Connected to P2P network' : 'Disconnected from network'}
          />
        </ListItem>
        
        <ListItem>
          <ListItemIcon>
            <PeopleIcon color={peerCount > 0 ? 'success' : 'disabled'} />
          </ListItemIcon>
          <ListItemText
            primary="Peers"
            secondary={`${peerCount} peer${peerCount !== 1 ? 's' : ''} connected`}
          />
        </ListItem>
        
        <ListItem>
          <ListItemIcon>
            {syncing ? (
              <CircularProgress size={20} />
            ) : (
              <CheckIcon color="success" />
            )}
          </ListItemIcon>
          <ListItemText
            primary="Sync Status"
            secondary={syncing ? 'Synchronizing data...' : 'All data synchronized'}
          />
        </ListItem>
        
        {lastSync && (
          <ListItem>
            <ListItemIcon>
              <UpdateIcon />
            </ListItemIcon>
            <ListItemText
              primary="Last Sync"
              secondary={format(lastSync, 'MMM dd, yyyy HH:mm:ss')}
            />
          </ListItem>
        )}
      </List>
      
      <Divider sx={{ my: 1 }} />
      
      <Typography variant="caption" color="text.secondary">
        {connected 
          ? 'Your data is being synchronized across the P2P network'
          : 'Working offline. Changes will sync when reconnected.'}
      </Typography>
    </Box>
  )
}

export default SyncStatusIndicator