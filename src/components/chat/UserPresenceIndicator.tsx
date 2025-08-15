import React, { useState, useEffect, useCallback } from 'react'
import {
  Box,
  Avatar,
  Badge,
  Tooltip,
  Typography,
  List,
  ListItem,
  ListItemAvatar,
  ListItemText,
  Paper,
  Chip,
  Divider,
} from '@mui/material'
import { 
  Circle as CircleIcon,
  Schedule as ScheduleIcon,
  DoNotDisturb as DoNotDisturbIcon,
  OfflinePin as OfflinePinIcon 
} from '@mui/icons-material'

// User presence types
interface UserPresence {
  user_id: string
  display_name: string
  status: 'online' | 'away' | 'busy' | 'offline'
  last_seen: string
  activity?: string
  avatar_url?: string
}

interface UserPresenceIndicatorProps {
  user: UserPresence
  size?: 'small' | 'medium' | 'large'
  showDetails?: boolean
}

// Individual presence indicator component
const UserPresenceIndicator: React.FC<UserPresenceIndicatorProps> = ({ 
  user, 
  size = 'medium',
  showDetails = false 
}) => {
  const getStatusColor = (status: UserPresence['status']) => {
    switch (status) {
      case 'online': return 'success'
      case 'away': return 'warning'
      case 'busy': return 'error'
      case 'offline': return 'default'
      default: return 'default'
    }
  }

  const getStatusIcon = (status: UserPresence['status']) => {
    switch (status) {
      case 'online': return <CircleIcon sx={{ fontSize: 12, color: 'success.main' }} />
      case 'away': return <ScheduleIcon sx={{ fontSize: 12, color: 'warning.main' }} />
      case 'busy': return <DoNotDisturbIcon sx={{ fontSize: 12, color: 'error.main' }} />
      case 'offline': return <OfflinePinIcon sx={{ fontSize: 12, color: 'grey.500' }} />
    }
  }

  const formatLastSeen = (lastSeen: string) => {
    const date = new Date(lastSeen)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    
    if (diff < 60000) return 'Just now'
    if (diff < 3600000) return Math.floor(diff / 60000) + 'm ago'
    if (diff < 86400000) return Math.floor(diff / 3600000) + 'h ago'
    if (diff < 604800000) return Math.floor(diff / 86400000) + 'd ago'
    return date.toLocaleDateString()
  }

  const getAvatarSize = () => {
    switch (size) {
      case 'small': return { width: 24, height: 24 }
      case 'medium': return { width: 40, height: 40 }
      case 'large': return { width: 56, height: 56 }
    }
  }

  const tooltipContent = (
    <Box>
      <Typography variant="body2" fontWeight="bold">
        {user.display_name}
      </Typography>
      <Typography variant="caption" sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
        {getStatusIcon(user.status)}
        {user.status.charAt(0).toUpperCase() + user.status.slice(1)}
      </Typography>
      {user.activity && (
        <Typography variant="caption" color="textSecondary">
          {user.activity}
        </Typography>
      )}
      {user.status !== 'online' && (
        <Typography variant="caption" color="textSecondary">
          Last seen {formatLastSeen(user.last_seen)}
        </Typography>
      )}
    </Box>
  )

  const avatar = (
    <Badge
      color={getStatusColor(user.status)}
      variant="dot"
      anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
      invisible={user.status === 'offline'}
    >
      <Avatar 
        sx={getAvatarSize()}
        src={user.avatar_url}
      >
        {user.display_name[0]?.toUpperCase()}
      </Avatar>
    </Badge>
  )

  if (showDetails) {
    return (
      <ListItem>
        <ListItemAvatar>
          {avatar}
        </ListItemAvatar>
        <ListItemText
          primary={
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <Typography variant="body2" fontWeight="bold">
                {user.display_name}
              </Typography>
              <Chip 
                size="small" 
                label={user.status}
                color={getStatusColor(user.status)}
                icon={getStatusIcon(user.status)}
                sx={{ height: 20, fontSize: '0.7rem' }}
              />
            </Box>
          }
          secondary={
            <Box>
              {user.activity && (
                <Typography variant="caption" color="textSecondary">
                  {user.activity}
                </Typography>
              )}
              {user.status !== 'online' && (
                <Typography variant="caption" color="textSecondary" display="block">
                  Last seen {formatLastSeen(user.last_seen)}
                </Typography>
              )}
            </Box>
          }
        />
      </ListItem>
    )
  }

  return (
    <Tooltip title={tooltipContent} placement="top">
      {avatar}
    </Tooltip>
  )
}

// Group presence panel component
interface GroupPresencePanelProps {
  users: UserPresence[]
  groupId: string
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
// @ts-ignore - groupId parameter required by interface but not used
const GroupPresencePanel: React.FC<GroupPresencePanelProps> = ({ users, groupId }) => {
  const [sortedUsers, setSortedUsers] = useState<UserPresence[]>([])

  useEffect(() => {
    // Sort users by status priority and then by name
    const statusPriority = { online: 0, away: 1, busy: 2, offline: 3 }
    const sorted = [...users].sort((a, b) => {
      const statusDiff = statusPriority[a.status] - statusPriority[b.status]
      if (statusDiff !== 0) return statusDiff
      return a.display_name.localeCompare(b.display_name)
    })
    setSortedUsers(sorted)
  }, [users])

  const getStatusCounts = () => {
    return users.reduce((counts, user) => {
      counts[user.status] = (counts[user.status] || 0) + 1
      return counts
    }, {} as Record<string, number>)
  }

  const statusCounts = getStatusCounts()

  return (
    <Paper sx={{ p: 2, maxHeight: '400px', overflow: 'auto' }}>
      <Typography variant="h6" gutterBottom>
        Members ({users.length})
      </Typography>
      
      {/* Status Summary */}
      <Box sx={{ display: 'flex', gap: 1, mb: 2, flexWrap: 'wrap' }}>
        {statusCounts.online && (
          <Chip 
            size="small" 
            label={statusCounts.online + ' online'}
            color="success"
            icon={<CircleIcon />}
          />
        )}
        {statusCounts.away && (
          <Chip 
            size="small" 
            label={statusCounts.away + ' away'}
            color="warning"
            icon={<ScheduleIcon />}
          />
        )}
        {statusCounts.busy && (
          <Chip 
            size="small" 
            label={statusCounts.busy + ' busy'}
            color="error"
            icon={<DoNotDisturbIcon />}
          />
        )}
        {statusCounts.offline && (
          <Chip 
            size="small" 
            label={statusCounts.offline + ' offline'}
            color="default"
            icon={<OfflinePinIcon />}
          />
        )}
      </Box>

      <Divider sx={{ mb: 1 }} />

      {/* User List */}
      <List sx={{ p: 0 }}>
        {sortedUsers.map((user, index) => (
          <React.Fragment key={user.user_id}>
            <UserPresenceIndicator 
              user={user} 
              size="medium" 
              showDetails={true} 
            />
            {index < sortedUsers.length - 1 && <Divider variant="inset" component="li" />}
          </React.Fragment>
        ))}
      </List>
    </Paper>
  )
}

// Hook for managing user presence
export const useUserPresence = (groupId?: string) => {
  const [users, setUsers] = useState<UserPresence[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!groupId) return

    const loadUserPresence = async () => {
      try {
        setLoading(true)
        
        // Mock user presence data for now
        // In a real implementation, this would come from the P2P network
        const mockUsers: UserPresence[] = [
          {
            user_id: 'alice',
            display_name: 'Alice Cooper',
            status: 'online',
            last_seen: new Date().toISOString(),
            activity: 'Developing P2P features',
          },
          {
            user_id: 'bob',
            display_name: 'Bob Builder',
            status: 'away',
            last_seen: new Date(Date.now() - 300000).toISOString(), // 5 minutes ago
            activity: 'In a meeting',
          },
          {
            user_id: 'charlie',
            display_name: 'Charlie Brown',
            status: 'busy',
            last_seen: new Date(Date.now() - 600000).toISOString(), // 10 minutes ago
            activity: 'Do not disturb',
          },
          {
            user_id: 'diana',
            display_name: 'Diana Prince',
            status: 'offline',
            last_seen: new Date(Date.now() - 3600000).toISOString(), // 1 hour ago
          },
        ]
        
        setUsers(mockUsers)
      } catch (err) {
        console.error('Failed to load user presence:', err)
        setError('Failed to load user presence: ' + err)
      } finally {
        setLoading(false)
      }
    }

    loadUserPresence()

    // Set up periodic updates (every 30 seconds)
    const interval = setInterval(loadUserPresence, 30000)
    
    return () => clearInterval(interval)
  }, [groupId])

  const updateUserStatus = useCallback(async (userId: string, status: UserPresence['status'], activity?: string) => {
    try {
      // In a real implementation, this would call a Tauri command
      // await invoke('update_user_presence', { userId, status, activity })
      
      setUsers(prev => prev.map(user => 
        user.user_id === userId 
          ? { 
              ...user, 
              status, 
              activity, 
              last_seen: new Date().toISOString() 
            }
          : user
      ))
    } catch (err) {
      console.error('Failed to update user status:', err)
      setError('Failed to update user status: ' + err)
    }
  }, [])

  return {
    users,
    loading,
    error,
    updateUserStatus,
  }
}

export { UserPresenceIndicator, GroupPresencePanel }
export type { UserPresence }
