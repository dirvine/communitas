import React, { useState, useCallback } from 'react'
import {
  Box,
  Card,
  CardContent,
  CardActions,
  Typography,
  Avatar,
  IconButton,
  Chip,
  Menu,
  MenuItem,
  Tooltip,
  Badge,
  Collapse,
  List,
  ListItem,
  ListItemAvatar,
  ListItemText,
  Divider,
  Button,
  TextField,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  ImageList,
  ImageListItem,
  Link,
  Paper,
  Popover,
} from '@mui/material'
import {
  Reply,
  MoreVert,
  Edit,
  Delete,
  Forward,
  Star,
  StarBorder,
  EmojiEmotions,
  ChatBubbleOutline,
  Done,
  DoneAll,
  Schedule,
  Lock,
  LockOpen,
  AttachFile,
  Image as ImageIcon,
  VideoLibrary,
  AudioFile,
  Description,
  Download,
  Share,
  ContentCopy,
  Flag,
  PushPin,
  Bookmark,
  BookmarkBorder,
  ThumbUp,
  Favorite,
  Celebration,
  SentimentVerySatisfied,
  SentimentDissatisfied,
} from '@mui/icons-material'
import { invoke } from '@tauri-apps/api/core'
import ReactMarkdown from 'react-markdown'
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism'

interface RichMessage {
  id: string
  sender: {
    id: string
    name: string
    avatar?: string
    fourWordAddress: string
  }
  channelId: string
  content: {
    text?: string
    markdown?: string
    mentions?: string[]
    links?: string[]
  }
  attachments?: Array<{
    id: string
    type: 'image' | 'video' | 'audio' | 'file'
    name: string
    size: number
    url: string
    thumbnail?: string
    mimeType: string
  }>
  threadId?: string
  replyTo?: {
    id: string
    sender: string
    preview: string
  }
  reactions?: Array<{
    emoji: string
    users: string[]
  }>
  timestamp: string
  editedAt?: string
  deletedAt?: string
  readBy?: string[]
  status: 'sending' | 'sent' | 'delivered' | 'read' | 'failed'
  encrypted: boolean
  ephemeral?: boolean
  pinned?: boolean
  starred?: boolean
  signature?: string
}

interface RichMessageDisplayProps {
  message: RichMessage
  currentUserId: string
  onReply?: (message: RichMessage) => void
  onEdit?: (message: RichMessage) => void
  onDelete?: (messageId: string) => void
  onReact?: (messageId: string, emoji: string) => void
  onThread?: (message: RichMessage) => void
  onDownload?: (attachment: any) => void
  onUserClick?: (userId: string) => void
  showThread?: boolean
  isCompact?: boolean
  isHighlighted?: boolean
}

const QUICK_REACTIONS = ['👍', '❤️', '😂', '😮', '😢', '🎉']

export const RichMessageDisplay: React.FC<RichMessageDisplayProps> = ({
  message,
  currentUserId,
  onReply,
  onEdit,
  onDelete,
  onReact,
  onThread,
  onDownload,
  onUserClick,
  showThread = true,
  isCompact = false,
  isHighlighted = false,
}) => {
  const [menuAnchor, setMenuAnchor] = useState<HTMLElement | null>(null)
  const [emojiAnchor, setEmojiAnchor] = useState<HTMLElement | null>(null)
  const [showThreadReplies, setShowThreadReplies] = useState(false)
  const [threadReplies, setThreadReplies] = useState<RichMessage[]>([])
  const [editMode, setEditMode] = useState(false)
  const [editedContent, setEditedContent] = useState(message.content.text || '')
  const [imagePreview, setImagePreview] = useState<string | null>(null)
  
  const isOwnMessage = message.sender.id === currentUserId
  const isDeleted = Boolean(message.deletedAt)
  
  // Format timestamp
  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    
    if (diff < 60000) return 'Just now'
    if (diff < 3600000) return Math.floor(diff / 60000) + 'm ago'
    if (diff < 86400000) return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    return date.toLocaleDateString()
  }
  
  // Get file icon based on type
  const getFileIcon = (type: string) => {
    switch (type) {
      case 'image': return <ImageIcon />
      case 'video': return <VideoLibrary />
      case 'audio': return <AudioFile />
      default: return <Description />
    }
  }
  
  // Handle reaction
  const handleReaction = async (emoji: string) => {
    if (onReact) {
      onReact(message.id, emoji)
    }
    
    try {
      await invoke('add_reaction', {
        messageId: message.id,
        emoji,
      })
    } catch (error) {
      console.error('Failed to add reaction:', error)
    }
    
    setEmojiAnchor(null)
  }
  
  // Handle edit submit
  const handleEditSubmit = async () => {
    if (onEdit) {
      onEdit({ ...message, content: { ...message.content, text: editedContent } })
    }
    
    try {
      await invoke('edit_message', {
        messageId: message.id,
        newContent: editedContent,
      })
      setEditMode(false)
    } catch (error) {
      console.error('Failed to edit message:', error)
    }
  }
  
  // Load thread replies
  const loadThreadReplies = useCallback(async () => {
    if (!message.threadId) return
    
    try {
      const replies = await invoke<RichMessage[]>('get_thread_messages', {
        threadId: message.threadId,
      })
      setThreadReplies(replies)
    } catch (error) {
      console.error('Failed to load thread replies:', error)
    }
  }, [message.threadId])
  
  // Custom markdown components
  const markdownComponents = {
    code({ node, inline, className, children, ...props }: any) {
      const match = /language-(\w+)/.exec(className || '')
      return !inline && match ? (
        <SyntaxHighlighter
          style={vscDarkPlus}
          language={match[1]}
          PreTag="div"
          {...props}
        >
          {String(children).replace(/\n$/, '')}
        </SyntaxHighlighter>
      ) : (
        <code className={className} {...props}>
          {children}
        </code>
      )
    },
    a({ href, children }: any) {
      return (
        <Link href={href} target="_blank" rel="noopener noreferrer">
          {children}
        </Link>
      )
    },
  }
  
  // Message status icon
  const getStatusIcon = () => {
    switch (message.status) {
      case 'sending':
        return <Schedule fontSize="small" color="action" />
      case 'sent':
        return <Done fontSize="small" color="action" />
      case 'delivered':
        return <DoneAll fontSize="small" color="action" />
      case 'read':
        return <DoneAll fontSize="small" color="primary" />
      case 'failed':
        return <Typography variant="caption" color="error">Failed</Typography>
      default:
        return null
    }
  }
  
  if (isDeleted) {
    return (
      <Card sx={{ mb: 1, opacity: 0.6 }}>
        <CardContent>
          <Typography variant="body2" color="text.secondary" fontStyle="italic">
            This message was deleted
          </Typography>
        </CardContent>
      </Card>
    )
  }
  
  return (
    <Card
      sx={{
        mb: 2,
        ml: isOwnMessage ? 'auto' : 0,
        mr: isOwnMessage ? 0 : 'auto',
        maxWidth: '70%',
        backgroundColor: isHighlighted ? 'action.selected' : isOwnMessage ? 'primary.light' : 'background.paper',
        position: 'relative',
      }}
    >
      {/* Reply context */}
      {message.replyTo && (
        <Box sx={{ p: 1, backgroundColor: 'action.hover', borderLeft: 3, borderColor: 'primary.main' }}>
          <Typography variant="caption" color="text.secondary">
            Replying to {message.replyTo.sender}
          </Typography>
          <Typography variant="caption" display="block" noWrap>
            {message.replyTo.preview}
          </Typography>
        </Box>
      )}
      
      <CardContent sx={{ pb: 1 }}>
        {/* Header */}
        <Box sx={{ display: 'flex', alignItems: 'center', mb: 1, gap: 1 }}>
          <Badge
            overlap="circular"
            anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
            variant="dot"
            color="success"
            invisible={!isCompact}
          >
            <Avatar
              sx={{ width: 32, height: 32, cursor: 'pointer' }}
              onClick={() => onUserClick?.(message.sender.id)}
            >
              {message.sender.avatar || message.sender.name[0]}
            </Avatar>
          </Badge>
          
          <Box sx={{ flex: 1 }}>
            <Typography variant="subtitle2" component="span">
              {message.sender.name}
            </Typography>
            <Typography variant="caption" color="text.secondary" sx={{ ml: 1 }}>
              {message.sender.fourWordAddress}
            </Typography>
          </Box>
          
          <Typography variant="caption" color="text.secondary">
            {formatTimestamp(message.timestamp)}
          </Typography>
          
          {message.editedAt && (
            <Typography variant="caption" color="text.secondary" fontStyle="italic">
              (edited)
            </Typography>
          )}
          
          {message.encrypted && (
            <Tooltip title="End-to-end encrypted">
              <Lock fontSize="small" color="success" />
            </Tooltip>
          )}
          
          {message.ephemeral && (
            <Tooltip title="Ephemeral message">
              <Schedule fontSize="small" color="warning" />
            </Tooltip>
          )}
          
          {message.pinned && (
            <Tooltip title="Pinned message">
              <PushPin fontSize="small" color="primary" />
            </Tooltip>
          )}
          
          <IconButton size="small" onClick={(e) => setMenuAnchor(e.currentTarget)}>
            <MoreVert fontSize="small" />
          </IconButton>
        </Box>
        
        {/* Content */}
        {editMode ? (
          <Box sx={{ mt: 1 }}>
            <TextField
              fullWidth
              multiline
              value={editedContent}
              onChange={(e) => setEditedContent(e.target.value)}
              variant="outlined"
              size="small"
            />
            <Box sx={{ mt: 1, display: 'flex', gap: 1 }}>
              <Button size="small" onClick={handleEditSubmit}>Save</Button>
              <Button size="small" onClick={() => setEditMode(false)}>Cancel</Button>
            </Box>
          </Box>
        ) : (
          <Box sx={{ mt: 1 }}>
            {message.content.markdown ? (
              <ReactMarkdown components={markdownComponents}>
                {message.content.markdown}
              </ReactMarkdown>
            ) : (
              <Typography variant="body2">
                {message.content.text}
              </Typography>
            )}
            
            {/* Mentions */}
            {message.content.mentions && message.content.mentions.length > 0 && (
              <Box sx={{ mt: 1, display: 'flex', gap: 0.5, flexWrap: 'wrap' }}>
                {message.content.mentions.map(mention => (
                  <Chip
                    key={mention}
                    size="small"
                    label={`@${mention}`}
                    onClick={() => onUserClick?.(mention)}
                  />
                ))}
              </Box>
            )}
          </Box>
        )}
        
        {/* Attachments */}
        {message.attachments && message.attachments.length > 0 && (
          <Box sx={{ mt: 2 }}>
            {message.attachments.filter(a => a.type === 'image').length > 0 && (
              <ImageList sx={{ width: '100%', height: 200 }} cols={3} rowHeight={164}>
                {message.attachments
                  .filter(a => a.type === 'image')
                  .map((attachment) => (
                    <ImageListItem
                      key={attachment.id}
                      sx={{ cursor: 'pointer' }}
                      onClick={() => setImagePreview(attachment.url)}
                    >
                      <img
                        src={attachment.thumbnail || attachment.url}
                        alt={attachment.name}
                        loading="lazy"
                      />
                    </ImageListItem>
                  ))}
              </ImageList>
            )}
            
            {message.attachments.filter(a => a.type !== 'image').map(attachment => (
              <Paper
                key={attachment.id}
                sx={{ p: 1, mb: 1, display: 'flex', alignItems: 'center', gap: 1 }}
              >
                {getFileIcon(attachment.type)}
                <Box sx={{ flex: 1 }}>
                  <Typography variant="body2">{attachment.name}</Typography>
                  <Typography variant="caption" color="text.secondary">
                    {(attachment.size / 1024).toFixed(1)} KB
                  </Typography>
                </Box>
                <IconButton size="small" onClick={() => onDownload?.(attachment)}>
                  <Download />
                </IconButton>
              </Paper>
            ))}
          </Box>
        )}
        
        {/* Reactions */}
        {message.reactions && message.reactions.length > 0 && (
          <Box sx={{ mt: 1, display: 'flex', gap: 0.5, flexWrap: 'wrap' }}>
            {message.reactions.map((reaction, idx) => (
              <Chip
                key={idx}
                size="small"
                label={`${reaction.emoji} ${reaction.users.length}`}
                onClick={() => handleReaction(reaction.emoji)}
                variant={reaction.users.includes(currentUserId) ? 'filled' : 'outlined'}
              />
            ))}
          </Box>
        )}
      </CardContent>
      
      <CardActions sx={{ px: 2, py: 0.5 }}>
        {/* Quick reactions */}
        <Box sx={{ display: 'flex', gap: 0.5 }}>
          {QUICK_REACTIONS.map(emoji => (
            <IconButton
              key={emoji}
              size="small"
              onClick={() => handleReaction(emoji)}
            >
              <Typography fontSize="small">{emoji}</Typography>
            </IconButton>
          ))}
          <IconButton size="small" onClick={(e) => setEmojiAnchor(e.currentTarget)}>
            <EmojiEmotions fontSize="small" />
          </IconButton>
        </Box>
        
        <Box sx={{ flex: 1 }} />
        
        {/* Actions */}
        <IconButton size="small" onClick={() => onReply?.(message)}>
          <Reply fontSize="small" />
        </IconButton>
        
        {showThread && (
          <IconButton
            size="small"
            onClick={() => {
              if (message.threadId) {
                setShowThreadReplies(!showThreadReplies)
                if (!showThreadReplies) {
                  loadThreadReplies()
                }
              } else {
                onThread?.(message)
              }
            }}
          >
            <Badge badgeContent={threadReplies.length} color="primary">
              <ChatBubbleOutline fontSize="small" />
            </Badge>
          </IconButton>
        )}
        
        {message.starred ? (
          <IconButton size="small">
            <Star fontSize="small" color="warning" />
          </IconButton>
        ) : (
          <IconButton size="small">
            <StarBorder fontSize="small" />
          </IconButton>
        )}
        
        {isOwnMessage && getStatusIcon()}
      </CardActions>
      
      {/* Thread replies */}
      {showThreadReplies && threadReplies.length > 0 && (
        <Collapse in={showThreadReplies}>
          <Divider />
          <Box sx={{ pl: 4, pr: 2, py: 1, backgroundColor: 'action.hover' }}>
            <Typography variant="caption" color="text.secondary" sx={{ mb: 1, display: 'block' }}>
              Thread - {threadReplies.length} replies
            </Typography>
            {threadReplies.map(reply => (
              <RichMessageDisplay
                key={reply.id}
                message={reply}
                currentUserId={currentUserId}
                onReply={onReply}
                onReact={onReact}
                onUserClick={onUserClick}
                showThread={false}
                isCompact
              />
            ))}
          </Box>
        </Collapse>
      )}
      
      {/* Message menu */}
      <Menu
        anchorEl={menuAnchor}
        open={Boolean(menuAnchor)}
        onClose={() => setMenuAnchor(null)}
      >
        {isOwnMessage && (
          <MenuItem onClick={() => { setEditMode(true); setMenuAnchor(null) }}>
            <Edit fontSize="small" sx={{ mr: 1 }} />
            Edit
          </MenuItem>
        )}
        
        <MenuItem onClick={() => onReply?.(message)}>
          <Reply fontSize="small" sx={{ mr: 1 }} />
          Reply
        </MenuItem>
        
        <MenuItem onClick={() => onThread?.(message)}>
          <ChatBubbleOutline fontSize="small" sx={{ mr: 1 }} />
          Start Thread
        </MenuItem>
        
        <MenuItem>
          <Forward fontSize="small" sx={{ mr: 1 }} />
          Forward
        </MenuItem>
        
        <MenuItem>
          <ContentCopy fontSize="small" sx={{ mr: 1 }} />
          Copy
        </MenuItem>
        
        <MenuItem>
          <Share fontSize="small" sx={{ mr: 1 }} />
          Share
        </MenuItem>
        
        <MenuItem>
          <PushPin fontSize="small" sx={{ mr: 1 }} />
          Pin
        </MenuItem>
        
        <MenuItem>
          {message.starred ? (
            <>
              <Star fontSize="small" sx={{ mr: 1 }} />
              Unstar
            </>
          ) : (
            <>
              <StarBorder fontSize="small" sx={{ mr: 1 }} />
              Star
            </>
          )}
        </MenuItem>
        
        <Divider />
        
        <MenuItem>
          <Flag fontSize="small" sx={{ mr: 1 }} />
          Report
        </MenuItem>
        
        {isOwnMessage && (
          <MenuItem
            onClick={() => { onDelete?.(message.id); setMenuAnchor(null) }}
            sx={{ color: 'error.main' }}
          >
            <Delete fontSize="small" sx={{ mr: 1 }} />
            Delete
          </MenuItem>
        )}
      </Menu>
      
      {/* Emoji picker popover */}
      <Popover
        open={Boolean(emojiAnchor)}
        anchorEl={emojiAnchor}
        onClose={() => setEmojiAnchor(null)}
        anchorOrigin={{
          vertical: 'bottom',
          horizontal: 'center',
        }}
        transformOrigin={{
          vertical: 'top',
          horizontal: 'center',
        }}
      >
        <Box sx={{ p: 1, display: 'flex', gap: 0.5 }}>
          {['👍', '👎', '❤️', '😂', '😮', '😢', '😡', '🎉', '🔥', '💯'].map(emoji => (
            <IconButton key={emoji} size="small" onClick={() => handleReaction(emoji)}>
              <Typography>{emoji}</Typography>
            </IconButton>
          ))}
        </Box>
      </Popover>
      
      {/* Image preview dialog */}
      <Dialog
        open={Boolean(imagePreview)}
        onClose={() => setImagePreview(null)}
        maxWidth="lg"
      >
        <DialogContent sx={{ p: 0 }}>
          {imagePreview && (
            <img
              src={imagePreview}
              alt="Preview"
              style={{ width: '100%', height: 'auto' }}
            />
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setImagePreview(null)}>Close</Button>
          <Button onClick={() => onDownload?.({ url: imagePreview })}>Download</Button>
        </DialogActions>
      </Dialog>
    </Card>
  )
}

export default RichMessageDisplay