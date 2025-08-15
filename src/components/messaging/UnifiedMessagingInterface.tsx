import React, { useState, useEffect, useCallback, useRef } from 'react'
import {
  Box,
  Paper,
  Typography,
  List,
  ListItemButton,
  ListItemAvatar,
  ListItemText,
  Avatar,
  Badge,
  Divider,
  IconButton,
  TextField,
  InputAdornment,
  Chip,
  Button,
  Drawer,
  AppBar,
  Toolbar,
  Tabs,
  Tab,
  Menu,
  MenuItem,
  Tooltip,
  CircularProgress,
  Alert,
  Snackbar,
  Fab,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Switch,
  FormControlLabel,
} from '@mui/material'
import {
  Search,
  Add,
  Settings,
  Notifications,
  NotificationsOff,
  Groups,
  Person,
  Public,
  Lock,
  VideoCall,
  Call,
  MoreVert,
  ArrowBack,
  Edit,
  CheckCircle,
  Error as ErrorIcon,
  CloudSync,
  CloudOff,
  Refresh,
  KeyboardArrowDown,
} from '@mui/icons-material'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import RichMessageComposer from './RichMessageComposer'
import RichMessageDisplay from './RichMessageDisplay'

interface Channel {
  id: string
  name: string
  type: 'direct' | 'group' | 'public'
  avatar?: string
  lastMessage?: {
    content: string
    timestamp: string
    sender: string
  }
  unreadCount: number
  members?: string[]
  isOnline?: boolean
  isMuted?: boolean
  isPinned?: boolean
  fourWordAddress?: string
}

interface MessagingState {
  currentChannel?: Channel
  messages: any[]
  channels: Channel[]
  searchQuery: string
  isLoading: boolean
  isSyncing: boolean
  isOnline: boolean
  typingUsers: Map<string, { user: string; timestamp: number }>
  presence: Map<string, 'online' | 'away' | 'busy' | 'offline'>
  notifications: boolean
  ephemeralMode: boolean
}

interface UnifiedMessagingInterfaceProps {
  userId: string
  fourWordAddress: string
  onVideoCall?: (channelId: string) => void
  onVoiceCall?: (channelId: string) => void
}

export const UnifiedMessagingInterface: React.FC<UnifiedMessagingInterfaceProps> = ({
  userId,
  fourWordAddress,
  onVideoCall,
  onVoiceCall,
}) => {
  const [state, setState] = useState<MessagingState>({
    messages: [],
    channels: [],
    searchQuery: '',
    isLoading: true,
    isSyncing: false,
    isOnline: true,
    typingUsers: new Map(),
    presence: new Map(),
    notifications: true,
    ephemeralMode: false,
  })
  
  const [drawerOpen, setDrawerOpen] = useState(true)
  const [tabValue, setTabValue] = useState(0)
  const [channelMenuAnchor, setChannelMenuAnchor] = useState<HTMLElement | null>(null)
  const [selectedChannel, setSelectedChannel] = useState<Channel | null>(null)
  const [newChannelDialog, setNewChannelDialog] = useState(false)
  const [newChannelType, setNewChannelType] = useState<'direct' | 'group'>('direct')
  const [newChannelName, setNewChannelName] = useState('')
  const [newChannelMembers, setNewChannelMembers] = useState<string[]>([])
  const [snackbar, setSnackbar] = useState<{ open: boolean; message: string; severity: 'success' | 'error' | 'info' }>({
    open: false,
    message: '',
    severity: 'info',
  })
  const [replyToMessage, setReplyToMessage] = useState<any>(null)
  const [threadMessage, setThreadMessage] = useState<any>(null)
  
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const messageListRef = useRef<HTMLDivElement>(null)
  
  // Initialize messaging system
  useEffect(() => {
    const initialize = async () => {
      try {
        setState(prev => ({ ...prev, isLoading: true }))
        
        // Initialize P2P messaging
        await invoke('initialize_unified_messaging', {
          userId,
          fourWordAddress,
        })
        
        // Load channels
        const channels = await loadChannels()
        
        setState(prev => ({
          ...prev,
          channels,
          isLoading: false,
        }))
        
        // Set up event listeners
        setupEventListeners()
        
      } catch (error) {
        console.error('Failed to initialize messaging:', error)
        showSnackbar('Failed to initialize messaging', 'error')
        setState(prev => ({ ...prev, isLoading: false }))
      }
    }
    
    initialize()
    
    return () => {
      // Cleanup event listeners
    }
  }, [userId, fourWordAddress])
  
  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [state.messages])
  
  // Set up event listeners for real-time updates
  const setupEventListeners = async () => {
    // Listen for new messages
    await listen('new-message', (event: any) => {
      const message = event.payload
      
      setState(prev => {
        // Add message if it's for current channel
        if (prev.currentChannel?.id === message.channelId) {
          return {
            ...prev,
            messages: [...prev.messages, message],
          }
        }
        
        // Update channel last message and unread count
        const channels = prev.channels.map(ch => {
          if (ch.id === message.channelId) {
            return {
              ...ch,
              lastMessage: {
                content: message.content.text || 'New message',
                timestamp: message.timestamp,
                sender: message.sender.name,
              },
              unreadCount: ch.id !== prev.currentChannel?.id ? ch.unreadCount + 1 : ch.unreadCount,
            }
          }
          return ch
        })
        
        return { ...prev, channels }
      })
      
      // Show notification if enabled
      if (state.notifications && message.channelId !== state.currentChannel?.id) {
        showNotification(message)
      }
    })
    
    // Listen for typing indicators
    await listen('user-typing', (event: any) => {
      const { channelId, userId, isTyping } = event.payload
      
      setState(prev => {
        const typingUsers = new Map(prev.typingUsers)
        
        if (isTyping) {
          typingUsers.set(`${channelId}-${userId}`, {
            user: userId,
            timestamp: Date.now(),
          })
        } else {
          typingUsers.delete(`${channelId}-${userId}`)
        }
        
        return { ...prev, typingUsers }
      })
    })
    
    // Listen for presence updates
    await listen('presence-update', (event: any) => {
      const { userId, status } = event.payload
      
      setState(prev => {
        const presence = new Map(prev.presence)
        presence.set(userId, status)
        return { ...prev, presence }
      })
    })
    
    // Listen for sync status
    await listen('sync-status', (event: any) => {
      const { isSyncing, isOnline } = event.payload
      setState(prev => ({ ...prev, isSyncing, isOnline }))
    })
  }
  
  // Load channels
  const loadChannels = async (): Promise<Channel[]> => {
    try {
      const channels = await invoke<Channel[]>('get_channels', { userId })
      return channels
    } catch (error) {
      console.error('Failed to load channels:', error)
      return []
    }
  }
  
  // Load messages for channel
  const loadMessages = async (channelId: string) => {
    try {
      setState(prev => ({ ...prev, isLoading: true }))
      
      const messages = await invoke<any[]>('get_channel_messages', {
        channelId,
        limit: 50,
      })
      
      setState(prev => ({
        ...prev,
        messages: Array.isArray(messages) ? messages : [],
        isLoading: false,
      }))
      
      // Mark messages as read
      await invoke('mark_channel_as_read', { channelId })
      
      // Update unread count
      setState(prev => ({
        ...prev,
        channels: prev.channels.map(ch =>
          ch.id === channelId ? { ...ch, unreadCount: 0 } : ch
        ),
      }))
      
    } catch (error) {
      console.error('Failed to load messages:', error)
      setState(prev => ({ ...prev, isLoading: false }))
    }
  }
  
  // Select channel
  const selectChannel = async (channel: Channel) => {
    setState(prev => ({ ...prev, currentChannel: channel }))
    setSelectedChannel(channel)
    await loadMessages(channel.id)
  }
  
  // Send message
  const handleSendMessage = async (message: any) => {
    try {
      // Message is sent through the composer
      // Just update UI optimistically
      setState(prev => ({
        ...prev,
        messages: [...prev.messages, {
          ...message,
          id: `temp-${Date.now()}`,
          sender: {
            id: userId,
            name: 'You',
            fourWordAddress,
          },
          timestamp: new Date().toISOString(),
          status: 'sending',
        }],
      }))
    } catch (error) {
      console.error('Failed to send message:', error)
      showSnackbar('Failed to send message', 'error')
    }
  }
  
  // Handle typing indicator
  const handleTyping = useCallback(async (isTyping: boolean) => {
    if (!state.currentChannel) return
    
    try {
      await invoke('send_typing_indicator', {
        channelId: state.currentChannel.id,
        isTyping,
      })
    } catch (error) {
      console.error('Failed to send typing indicator:', error)
    }
  }, [state.currentChannel])
  
  // Create new channel
  const createChannel = async () => {
    try {
      const channel = await invoke<Channel>('create_channel', {
        name: newChannelName,
        type: newChannelType,
        members: newChannelMembers,
      })
      
      setState(prev => ({
        ...prev,
        channels: [...prev.channels, channel],
      }))
      
      setNewChannelDialog(false)
      setNewChannelName('')
      setNewChannelMembers([])
      
      showSnackbar('Channel created successfully', 'success')
      selectChannel(channel)
      
    } catch (error) {
      console.error('Failed to create channel:', error)
      showSnackbar('Failed to create channel', 'error')
    }
  }
  
  // Show notification
  const showNotification = (message: any) => {
    if ('Notification' in window && Notification.permission === 'granted') {
      new Notification(`New message from ${message.sender.name}`, {
        body: message.content.text || 'New message',
        icon: '/icon.png',
      })
    }
  }
  
  // Show snackbar
  const showSnackbar = (message: string, severity: 'success' | 'error' | 'info') => {
    setSnackbar({ open: true, message, severity })
  }
  
  // Filter channels based on tab and search
  const getFilteredChannels = () => {
    let filtered = state.channels
    
    // Filter by tab
    switch (tabValue) {
      case 0: // All
        break
      case 1: // Direct
        filtered = filtered.filter(ch => ch.type === 'direct')
        break
      case 2: // Groups
        filtered = filtered.filter(ch => ch.type === 'group')
        break
      case 3: // Pinned
        filtered = filtered.filter(ch => ch.isPinned)
        break
    }
    
    // Filter by search
    if (state.searchQuery) {
      filtered = filtered.filter(ch =>
        ch.name.toLowerCase().includes(state.searchQuery.toLowerCase())
      )
    }
    
    // Sort by last message
    return filtered.sort((a, b) => {
      if (a.isPinned && !b.isPinned) return -1
      if (!a.isPinned && b.isPinned) return 1
      
      const aTime = a.lastMessage?.timestamp || '0'
      const bTime = b.lastMessage?.timestamp || '0'
      return bTime.localeCompare(aTime)
    })
  }
  
  // Get typing users for current channel
  const getTypingUsers = () => {
    if (!state.currentChannel) return []
    
    const channelTyping = Array.from(state.typingUsers.entries())
      .filter(([key]) => key.startsWith(state.currentChannel.id))
      .map(([, value]) => value.user)
      .filter(user => user !== userId)
    
    return channelTyping
  }
  
  return (
    <Box sx={{ display: 'flex', height: '100vh' }}>
      {/* Sidebar */}
      <Drawer
        variant="persistent"
        open={drawerOpen}
        sx={{
          width: drawerOpen ? 320 : 0,
          flexShrink: 0,
          '& .MuiDrawer-paper': {
            width: 320,
            boxSizing: 'border-box',
          },
        }}
      >
        <Box sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
          {/* Header */}
          <AppBar position="static" color="default" elevation={0}>
            <Toolbar>
              <Typography variant="h6" sx={{ flex: 1 }}>
                Messages
              </Typography>
              <IconButton onClick={() => setNewChannelDialog(true)}>
                <Add />
              </IconButton>
              <IconButton>
                <Settings />
              </IconButton>
            </Toolbar>
          </AppBar>
          
          {/* Search */}
          <Box sx={{ p: 2 }}>
            <TextField
              fullWidth
              size="small"
              placeholder="Search conversations..."
              value={state.searchQuery}
              onChange={(e) => setState(prev => ({ ...prev, searchQuery: e.target.value }))}
              InputProps={{
                startAdornment: (
                  <InputAdornment position="start">
                    <Search />
                  </InputAdornment>
                ),
              }}
            />
          </Box>
          
          {/* Tabs */}
          <Tabs
            value={tabValue}
            onChange={(_, value) => setTabValue(value)}
            variant="fullWidth"
          >
            <Tab label="All" />
            <Tab label="Direct" />
            <Tab label="Groups" />
            <Tab label="Pinned" />
          </Tabs>
          
          {/* Channel list */}
          <List sx={{ flex: 1, overflow: 'auto' }}>
            {getFilteredChannels().map(channel => (
              <ListItemButton
                key={channel.id}
                selected={state.currentChannel?.id === channel.id}
                onClick={() => selectChannel(channel)}
              >
                <ListItemAvatar>
                  <Badge
                    color="success"
                    variant="dot"
                    invisible={!channel.isOnline}
                    overlap="circular"
                  >
                    <Avatar>
                      {channel.avatar || channel.name[0]}
                    </Avatar>
                  </Badge>
                </ListItemAvatar>
                
                <ListItemText
                  primary={
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
                      <Typography variant="body1">
                        {channel.name}
                      </Typography>
                      {channel.type === 'group' && <Groups fontSize="small" />}
                      {channel.type === 'public' ? <Public fontSize="small" /> : <Lock fontSize="small" />}
                      {channel.isMuted && <NotificationsOff fontSize="small" />}
                    </Box>
                  }
                  secondary={
                    <Typography variant="caption" noWrap>
                      {channel.lastMessage
                        ? `${channel.lastMessage.sender}: ${channel.lastMessage.content}`
                        : channel.fourWordAddress || 'No messages yet'
                      }
                    </Typography>
                  }
                />
                
                {channel.unreadCount > 0 && (
                  <Chip
                    size="small"
                    label={channel.unreadCount}
                    color="primary"
                  />
                )}
              </ListItemButton>
            ))}
          </List>
          
          {/* Sync status */}
          <Box sx={{ p: 2, display: 'flex', alignItems: 'center', gap: 1 }}>
            {state.isSyncing ? (
              <>
                <CircularProgress size={16} />
                <Typography variant="caption">Syncing...</Typography>
              </>
            ) : state.isOnline ? (
              <>
                <CloudSync color="success" fontSize="small" />
                <Typography variant="caption">Connected</Typography>
              </>
            ) : (
              <>
                <CloudOff color="error" fontSize="small" />
                <Typography variant="caption">Offline</Typography>
              </>
            )}
            
            <Box sx={{ flex: 1 }} />
            
            <IconButton size="small" onClick={() => loadChannels()}>
              <Refresh fontSize="small" />
            </IconButton>
          </Box>
        </Box>
      </Drawer>
      
      {/* Main content */}
      <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
        {state.currentChannel ? (
          <>
            {/* Channel header */}
            <AppBar position="static" color="default" elevation={1}>
              <Toolbar>
                <IconButton
                  edge="start"
                  onClick={() => setDrawerOpen(!drawerOpen)}
                  sx={{ mr: 2 }}
                >
                  <ArrowBack />
                </IconButton>
                
                <Avatar sx={{ mr: 2 }}>
                  {state.currentChannel.avatar || state.currentChannel.name[0]}
                </Avatar>
                
                <Box sx={{ flex: 1 }}>
                  <Typography variant="h6">
                    {state.currentChannel.name}
                  </Typography>
                  
                  {/* Typing indicator */}
                  {getTypingUsers().length > 0 && (
                    <Typography variant="caption" color="text.secondary">
                      {getTypingUsers().join(', ')} typing...
                    </Typography>
                  )}
                </Box>
                
                <IconButton onClick={() => onVoiceCall?.(state.currentChannel!.id)}>
                  <Call />
                </IconButton>
                
                <IconButton onClick={() => onVideoCall?.(state.currentChannel!.id)}>
                  <VideoCall />
                </IconButton>
                
                <IconButton onClick={(e) => setChannelMenuAnchor(e.currentTarget)}>
                  <MoreVert />
                </IconButton>
              </Toolbar>
            </AppBar>
            
            {/* Messages */}
            <Box
              ref={messageListRef}
              sx={{
                flex: 1,
                overflow: 'auto',
                p: 2,
                backgroundColor: 'background.default',
              }}
            >
              {state.isLoading ? (
                <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}>
                  <CircularProgress />
                </Box>
              ) : state.messages.length === 0 ? (
                <Box sx={{ textAlign: 'center', p: 4 }}>
                  <Typography variant="h6" color="text.secondary">
                    No messages yet
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Start the conversation!
                  </Typography>
                </Box>
              ) : (
                <>
                  {state.messages.map(message => (
                    <RichMessageDisplay
                      key={message.id}
                      message={message}
                      currentUserId={userId}
                      onReply={setReplyToMessage}
                      onThread={setThreadMessage}
                      onReact={(messageId, emoji) => {
                        invoke('add_reaction', { messageId, emoji })
                      }}
                      onDelete={(messageId) => {
                        invoke('delete_message', { messageId })
                      }}
                    />
                  ))}
                  <div ref={messagesEndRef} />
                </>
              )}
            </Box>
            
            {/* Message composer */}
            <Box sx={{ p: 2, borderTop: 1, borderColor: 'divider' }}>
              <RichMessageComposer
                channelId={state.currentChannel.id}
                threadId={threadMessage?.id}
                replyTo={replyToMessage?.id}
                onSend={handleSendMessage}
                onTyping={handleTyping}
                ephemeral={state.ephemeralMode}
                placeholder={`Message ${state.currentChannel.name}...`}
              />
              
              {/* Ephemeral mode toggle */}
              <FormControlLabel
                control={
                  <Switch
                    checked={state.ephemeralMode}
                    onChange={(e) => setState(prev => ({ ...prev, ephemeralMode: e.target.checked }))}
                  />
                }
                label="Ephemeral mode"
                sx={{ mt: 1 }}
              />
            </Box>
          </>
        ) : (
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
            <Typography variant="h6" color="text.secondary">
              Select a conversation to start messaging
            </Typography>
          </Box>
        )}
      </Box>
      
      {/* Channel menu */}
      <Menu
        anchorEl={channelMenuAnchor}
        open={Boolean(channelMenuAnchor)}
        onClose={() => setChannelMenuAnchor(null)}
      >
        <MenuItem onClick={() => setChannelMenuAnchor(null)}>
          <Person sx={{ mr: 1 }} />
          View Profile
        </MenuItem>
        
        <MenuItem onClick={() => setChannelMenuAnchor(null)}>
          {selectedChannel?.isMuted ? (
            <>
              <Notifications sx={{ mr: 1 }} />
              Unmute
            </>
          ) : (
            <>
              <NotificationsOff sx={{ mr: 1 }} />
              Mute
            </>
          )}
        </MenuItem>
        
        <MenuItem onClick={() => setChannelMenuAnchor(null)}>
          <Search sx={{ mr: 1 }} />
          Search in conversation
        </MenuItem>
        
        <Divider />
        
        <MenuItem onClick={() => setChannelMenuAnchor(null)} sx={{ color: 'error.main' }}>
          Delete conversation
        </MenuItem>
      </Menu>
      
      {/* New channel dialog */}
      <Dialog open={newChannelDialog} onClose={() => setNewChannelDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>New Conversation</DialogTitle>
        <DialogContent>
          <Tabs
            value={newChannelType === 'direct' ? 0 : 1}
            onChange={(_, value) => setNewChannelType(value === 0 ? 'direct' : 'group')}
            sx={{ mb: 2 }}
          >
            <Tab label="Direct Message" />
            <Tab label="Group" />
          </Tabs>
          
          {newChannelType === 'direct' ? (
            <TextField
              fullWidth
              label="Four-word address or username"
              value={newChannelName}
              onChange={(e) => setNewChannelName(e.target.value)}
              placeholder="ocean-forest-moon-star"
            />
          ) : (
            <>
              <TextField
                fullWidth
                label="Group name"
                value={newChannelName}
                onChange={(e) => setNewChannelName(e.target.value)}
                sx={{ mb: 2 }}
              />
              
              <TextField
                fullWidth
                label="Add members (four-word addresses)"
                placeholder="ocean-forest-moon-star, river-mountain-sun-cloud"
                onChange={(e) => setNewChannelMembers(e.target.value.split(',').map(s => s.trim()))}
              />
            </>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setNewChannelDialog(false)}>Cancel</Button>
          <Button onClick={createChannel} variant="contained" disabled={!newChannelName}>
            Create
          </Button>
        </DialogActions>
      </Dialog>
      
      {/* Snackbar */}
      <Snackbar
        open={snackbar.open}
        autoHideDuration={6000}
        onClose={() => setSnackbar(prev => ({ ...prev, open: false }))}
      >
        <Alert
          onClose={() => setSnackbar(prev => ({ ...prev, open: false }))}
          severity={snackbar.severity}
        >
          {snackbar.message}
        </Alert>
      </Snackbar>
    </Box>
  )
}

export default UnifiedMessagingInterface