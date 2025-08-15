import React, { useState, useEffect, useCallback, useRef } from 'react'
import {
  Box,
  TextField,
  Button,
  List,
  ListItem,
  // ListItemText,
  ListItemAvatar,
  Typography,
  Paper,
  Divider,
  Chip,
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Menu,
  MenuItem,
  Avatar,
  Badge,
  Tooltip,
  InputAdornment,
  Card,
  CardContent,
  Fade,
  Alert,
} from '@mui/material'
import {
  Send,
  Add,
  MoreVert,
  Person,
  PersonAdd,
  ExitToApp,
  Settings,
  EmojiEmotions,
  AttachFile,
  // Search,
  Notifications,
  NotificationsOff,
} from '@mui/icons-material'
import { invoke } from '@tauri-apps/api/core'

// Enhanced message type with P2P integration
interface P2PMessage {
  id: string
  sender: string
  recipient?: string
  group_id?: string
  content: string
  timestamp: string
  encrypted: boolean
  message_type: 'direct' | 'group'
  status: 'sending' | 'sent' | 'delivered' | 'read' | 'failed'
  metadata?: {
    reply_to?: string
    edited?: boolean
    reactions?: Array<{ emoji: string; users: string[] }>
  }
}

// Group information
interface GroupInfo {
  id: string
  name: string
  description?: string
  created_by: string
  created_at: string
  member_count: number
  members: string[]
  is_admin: boolean
  notifications_enabled: boolean
}

// User presence status
interface UserPresence {
  user_id: string
  status: 'online' | 'away' | 'busy' | 'offline'
  last_seen: string
  activity?: string
}

// Message request for sending
interface MessageRequest {
  recipient: string
  content: string
  message_type: 'direct' | 'group'
  group_id?: string
}

interface GroupChatInterfaceProps {
  groupId?: string
  onGroupChange?: (groupId: string) => void
}

const GroupChatInterface: React.FC<GroupChatInterfaceProps> = ({ 
  groupId, 
  onGroupChange 
}) => {
  // State management
  const [messages, setMessages] = useState<P2PMessage[]>([])
  const [currentMessage, setCurrentMessage] = useState('')
  const [groups, setGroups] = useState<GroupInfo[]>([])
  const [currentGroup, setCurrentGroup] = useState<GroupInfo | null>(null)
  const [userPresence] = useState<Record<string, UserPresence>>({})
  // const [setUserPresence] = useState<Record<string, UserPresence>>({})
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  
  // UI state
  const [createGroupOpen, setCreateGroupOpen] = useState(false)
  const [groupMenuAnchor, setGroupMenuAnchor] = useState<null | HTMLElement>(null)
  const [newGroupName, setNewGroupName] = useState('')
  const [newGroupDescription, setNewGroupDescription] = useState('')
  
  // Refs
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const messageInputRef = useRef<HTMLInputElement>(null)

  // Auto-scroll to bottom when new messages arrive
  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [])

  // Initialize messaging system and load initial data
  useEffect(() => {
    const initializeMessaging = async () => {
      try {
        setLoading(true)
        
        // Initialize messaging system
        await invoke('initialize_messaging')
        
        // Load existing groups and messages
        await loadGroups()
        
        if (groupId) {
          await loadMessages(groupId)
          setCurrentGroup(groups.find(g => g.id === groupId) || null)
        }
      } catch (err) {
        console.error('Failed to initialize messaging:', err)
        setError('Failed to initialize messaging: ' + err)
      } finally {
        setLoading(false)
      }
    }

    initializeMessaging()
  }, [groupId])

  // Auto-scroll when messages change
  useEffect(() => {
    scrollToBottom()
  }, [messages, scrollToBottom])

  // Load groups from backend
  const loadGroups = async () => {
    try {
      // For now, we'll create mock groups since the backend doesn't have a get_groups command yet
      const mockGroups: GroupInfo[] = [
        {
          id: 'general',
          name: 'General',
          description: 'General discussion for everyone',
          created_by: 'system',
          created_at: new Date().toISOString(),
          member_count: 12,
          members: ['alice', 'bob', 'charlie', 'diana'],
          is_admin: false,
          notifications_enabled: true,
        },
        {
          id: 'tech-talk',
          name: 'Tech Talk',
          description: 'Technical discussions and P2P development',
          created_by: 'alice',
          created_at: new Date().toISOString(),
          member_count: 8,
          members: ['alice', 'bob', 'eve'],
          is_admin: true,
          notifications_enabled: true,
        }
      ]
      setGroups(mockGroups)
    } catch (err) {
      console.error('Failed to load groups:', err)
      setError('Failed to load groups: ' + err)
    }
  }

  // Load messages for specific group
  const loadMessages = async (group_id: string) => {
    try {
      setLoading(true)
      
      const messages: P2PMessage[] = await invoke('get_messages', {
        userId: null,
        groupId: group_id,
        limit: 50
      })
      
      // Add status and metadata to messages
      const enhancedMessages = messages.map(msg => ({
        ...msg,
        status: 'delivered' as const,
        message_type: (msg.group_id ? 'group' : 'direct') as 'group' | 'direct',
        metadata: {
          reactions: []
        }
      }))
      
      setMessages(enhancedMessages)
    } catch (err) {
      console.error('Failed to load messages:', err)
      setError('Failed to load messages: ' + err)
    } finally {
      setLoading(false)
    }
  }

  // Send message to current group
  const sendMessage = async () => {
    if (!currentMessage.trim() || !currentGroup) return

    const tempMessage: P2PMessage = {
      id: 'temp-' + Date.now(),
      sender: 'You',
      group_id: currentGroup.id,
      content: currentMessage.trim(),
      timestamp: new Date().toISOString(),
      encrypted: true,
      message_type: 'group',
      status: 'sending',
      metadata: { reactions: [] }
    }

    // Add message optimistically
    setMessages(prev => [...prev, tempMessage])
    setCurrentMessage('')

    try {
      const messageRequest: MessageRequest = {
        recipient: '', // Not needed for group messages
        content: currentMessage.trim(),
        message_type: 'group',
        group_id: currentGroup.id
      }

      const messageId = await invoke<string>('send_group_message', { request: messageRequest })
      
      // Update message status
      setMessages(prev => prev.map(msg => 
        msg.id === tempMessage.id 
          ? { ...msg, id: messageId, status: 'sent' }
          : msg
      ))
      
    } catch (err) {
      console.error('Failed to send message:', err)
      setError('Failed to send message: ' + err)
      
      // Update message status to failed
      setMessages(prev => prev.map(msg => 
        msg.id === tempMessage.id 
          ? { ...msg, status: 'failed' }
          : msg
      ))
    }
  }

  // Create new group
  const createGroup = async () => {
    if (!newGroupName.trim()) return

    try {
      const groupId = await invoke<string>('create_group', {
        name: newGroupName.trim(),
        description: newGroupDescription.trim() || null
      })

      const newGroup: GroupInfo = {
        id: groupId,
        name: newGroupName.trim(),
        description: newGroupDescription.trim(),
        created_by: 'You',
        created_at: new Date().toISOString(),
        member_count: 1,
        members: ['You'],
        is_admin: true,
        notifications_enabled: true,
      }

      setGroups(prev => [...prev, newGroup])
      setCurrentGroup(newGroup)
      setCreateGroupOpen(false)
      setNewGroupName('')
      setNewGroupDescription('')
      
      if (onGroupChange) {
        onGroupChange(groupId)
      }
      
    } catch (err) {
      console.error('Failed to create group:', err)
      setError('Failed to create group: ' + err)
    }
  }

  // Handle keyboard shortcuts
  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      sendMessage()
    }
  }

  // Format timestamp for display
  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    
    if (diff < 60000) return 'Just now'
    if (diff < 3600000) return Math.floor(diff / 60000) + 'm ago'
    if (diff < 86400000) return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    return date.toLocaleDateString()
  }

  // Get status color for message
  const getStatusColor = (status: P2PMessage['status']) => {
    switch (status) {
      case 'sending': return 'orange'
      case 'sent': return 'blue'
      case 'delivered': return 'green'
      case 'read': return 'purple'
      case 'failed': return 'red'
      default: return 'grey'
    }
  }

  // Switch to different group
  const switchGroup = (group: GroupInfo) => {
    setCurrentGroup(group)
    loadMessages(group.id)
    if (onGroupChange) {
      onGroupChange(group.id)
    }
  }

  return (
    <Box sx={{ display: 'flex', height: '100%', flexDirection: 'column' }}>
      {/* Error Alert */}
      {error && (
        <Alert severity="error" onClose={() => setError(null)} sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      {/* Header with group info and controls */}
      <Box sx={{ 
        display: 'flex', 
        alignItems: 'center', 
        justifyContent: 'space-between',
        p: 2,
        borderBottom: 1,
        borderColor: 'divider'
      }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
          <Typography variant="h5">
            {currentGroup?.name || 'Select a Group'}
          </Typography>
          {currentGroup && (
            <Chip 
              size="small" 
              label={currentGroup.member_count + ' members'}
              icon={<Person />}
            />
          )}
        </Box>
        
        <Box sx={{ display: 'flex', gap: 1 }}>
          <Tooltip title="Create Group">
            <IconButton onClick={() => setCreateGroupOpen(true)}>
              <Add />
            </IconButton>
          </Tooltip>
          
          {currentGroup && (
            <Tooltip title="Group Options">
              <IconButton onClick={(e) => setGroupMenuAnchor(e.currentTarget)}>
                <MoreVert />
              </IconButton>
            </Tooltip>
          )}
        </Box>
      </Box>

      {/* Group Selection Bar */}
      <Box sx={{ 
        display: 'flex', 
        gap: 1, 
        p: 1, 
        overflowX: 'auto',
        borderBottom: 1,
        borderColor: 'divider'
      }}>
        {groups.map(group => (
          <Chip
            key={group.id}
            label={group.name}
            color={currentGroup?.id === group.id ? 'primary' : 'default'}
            onClick={() => switchGroup(group)}
            variant={currentGroup?.id === group.id ? 'filled' : 'outlined'}
          />
        ))}
      </Box>

      {/* Messages Container */}
      <Paper sx={{ 
        flex: 1, 
        overflow: 'auto', 
        p: 1,
        backgroundColor: 'background.default'
      }}>
        {loading ? (
          <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}>
            <Typography>Loading messages...</Typography>
          </Box>
        ) : messages.length === 0 ? (
          <Box sx={{ 
            display: 'flex', 
            flexDirection: 'column',
            alignItems: 'center', 
            justifyContent: 'center',
            height: '100%',
            gap: 2
          }}>
            <Typography variant="h6" color="textSecondary">
              {currentGroup ? 'No messages yet' : 'Select a group to start chatting'}
            </Typography>
            {currentGroup && (
              <Typography variant="body2" color="textSecondary">
                Be the first to send a message in {currentGroup.name}!
              </Typography>
            )}
          </Box>
        ) : (
          <List sx={{ pb: 0 }}>
            {messages.map((message) => (
              <Fade in key={message.id}>
                <ListItem 
                  sx={{ 
                    alignItems: 'flex-start',
                    flexDirection: message.sender === 'You' ? 'row-reverse' : 'row',
                    mb: 1
                  }}
                >
                  <ListItemAvatar sx={{ minWidth: 40 }}>
                    <Badge
                      color="success"
                      variant="dot"
                      invisible={!userPresence[message.sender] || userPresence[message.sender].status !== 'online'}
                    >
                      <Avatar sx={{ width: 32, height: 32, fontSize: '0.8rem' }}>
                        {message.sender[0].toUpperCase()}
                      </Avatar>
                    </Badge>
                  </ListItemAvatar>
                  
                  <Card sx={{ 
                    maxWidth: '70%',
                    ml: message.sender === 'You' ? 0 : 1,
                    mr: message.sender === 'You' ? 1 : 0,
                    backgroundColor: message.sender === 'You' ? 'primary.light' : 'background.paper'
                  }}>
                    <CardContent sx={{ p: 1.5, '&:last-child': { pb: 1.5 } }}>
                      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 0.5 }}>
                        <Typography variant="caption" fontWeight="bold">
                          {message.sender}
                        </Typography>
                        <Typography variant="caption" color="textSecondary">
                          {formatTimestamp(message.timestamp)}
                        </Typography>
                        {message.encrypted && (
                          <Chip size="small" label="🔒" sx={{ height: 16, fontSize: '0.6rem' }} />
                        )}
                        <Chip 
                          size="small" 
                          label={message.status}
                          color={getStatusColor(message.status) as any}
                          variant="outlined"
                          sx={{ height: 16, fontSize: '0.6rem' }}
                        />
                      </Box>
                      
                      <Typography variant="body2">
                        {message.content}
                      </Typography>
                      
                      {/* Message reactions (placeholder for future) */}
                      {message.metadata?.reactions && message.metadata.reactions.length > 0 && (
                        <Box sx={{ mt: 1, display: 'flex', gap: 0.5 }}>
                          {message.metadata.reactions.map((reaction, idx) => (
                            <Chip
                              key={idx}
                              size="small"
                              label={reaction.emoji + ' ' + reaction.users.length}
                              sx={{ height: 20, fontSize: '0.7rem' }}
                            />
                          ))}
                        </Box>
                      )}
                    </CardContent>
                  </Card>
                </ListItem>
              </Fade>
            ))}
            <div ref={messagesEndRef} />
          </List>
        )}
      </Paper>

      {/* Message Input */}
      {currentGroup && (
        <Box sx={{ 
          p: 2, 
          borderTop: 1, 
          borderColor: 'divider',
          backgroundColor: 'background.paper'
        }}>
          <TextField
            ref={messageInputRef}
            fullWidth
            multiline
            maxRows={4}
            value={currentMessage}
            onChange={(e) => setCurrentMessage(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder={'Message ' + currentGroup.name + '...'}
            variant="outlined"
            InputProps={{
              endAdornment: (
                <InputAdornment position="end">
                  <Box sx={{ display: 'flex', gap: 1 }}>
                    <IconButton size="small" disabled>
                      <EmojiEmotions />
                    </IconButton>
                    <IconButton size="small" disabled>
                      <AttachFile />
                    </IconButton>
                    <Button
                      variant="contained"
                      onClick={sendMessage}
                      disabled={!currentMessage.trim() || loading}
                      startIcon={<Send />}
                      size="small"
                    >
                      Send
                    </Button>
                  </Box>
                </InputAdornment>
              )
            }}
          />
        </Box>
      )}

      {/* Create Group Dialog */}
      <Dialog open={createGroupOpen} onClose={() => setCreateGroupOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create New Group</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Group Name"
            fullWidth
            variant="outlined"
            value={newGroupName}
            onChange={(e) => setNewGroupName(e.target.value)}
            sx={{ mb: 2 }}
          />
          <TextField
            margin="dense"
            label="Description (optional)"
            fullWidth
            multiline
            rows={3}
            variant="outlined"
            value={newGroupDescription}
            onChange={(e) => setNewGroupDescription(e.target.value)}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateGroupOpen(false)}>Cancel</Button>
          <Button 
            onClick={createGroup} 
            variant="contained"
            disabled={!newGroupName.trim()}
          >
            Create Group
          </Button>
        </DialogActions>
      </Dialog>

      {/* Group Options Menu */}
      <Menu
        anchorEl={groupMenuAnchor}
        open={Boolean(groupMenuAnchor)}
        onClose={() => setGroupMenuAnchor(null)}
      >
        <MenuItem onClick={() => setGroupMenuAnchor(null)}>
          <PersonAdd sx={{ mr: 1 }} />
          Invite Members
        </MenuItem>
        <MenuItem onClick={() => setGroupMenuAnchor(null)}>
          <Settings sx={{ mr: 1 }} />
          Group Settings
        </MenuItem>
        <MenuItem onClick={() => setGroupMenuAnchor(null)}>
          {currentGroup?.notifications_enabled ? <NotificationsOff sx={{ mr: 1 }} /> : <Notifications sx={{ mr: 1 }} />}
          {currentGroup?.notifications_enabled ? 'Mute' : 'Unmute'}
        </MenuItem>
        <Divider />
        <MenuItem onClick={() => setGroupMenuAnchor(null)} sx={{ color: 'error.main' }}>
          <ExitToApp sx={{ mr: 1 }} />
          Leave Group
        </MenuItem>
      </Menu>
    </Box>
  )
}

export default GroupChatInterface
