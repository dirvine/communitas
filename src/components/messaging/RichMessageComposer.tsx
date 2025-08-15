import React, { useState, useRef, useCallback, useEffect } from 'react'
import {
  Box,
  TextField,
  IconButton,
  Button,
  Paper,
  Menu,
  MenuItem,
  Tooltip,
  Chip,
  Avatar,
  List,
  ListItem,
  ListItemAvatar,
  ListItemText,
  Popover,
  Grid,
  Typography,
  Divider,
  Badge,
  LinearProgress,
} from '@mui/material'
import {
  Send,
  AttachFile,
  EmojiEmotions,
  FormatBold,
  FormatItalic,
  Code,
  FormatQuote,
  Link,
  Image,
  VideoCall,
  Mic,
  Close,
  Schedule,
  Poll,
  LocationOn,
  Gif,
} from '@mui/icons-material'
import { invoke } from '@tauri-apps/api/core'

interface RichMessageComposerProps {
  channelId: string
  threadId?: string
  replyTo?: string
  onSend: (message: any) => void
  onTyping?: (isTyping: boolean) => void
  onAttachment?: (files: File[]) => void
  disabled?: boolean
  placeholder?: string
  autoFocus?: boolean
  ephemeral?: boolean
}

interface DraftMessage {
  text: string
  formattedText?: string
  mentions: string[]
  attachments: File[]
  ephemeral: boolean
}

interface EmojiCategory {
  name: string
  emojis: string[]
}

const EMOJI_CATEGORIES: EmojiCategory[] = [
  {
    name: 'Recent',
    emojis: ['😊', '❤️', '👍', '😂', '🎉', '🔥', '✨', '💯']
  },
  {
    name: 'Smileys',
    emojis: ['😀', '😃', '😄', '😁', '😅', '😂', '🤣', '😊', '😇', '🙂', '🙃', '😉', '😌', '😍', '🥰', '😘']
  },
  {
    name: 'Reactions',
    emojis: ['👍', '👎', '👌', '✌️', '🤞', '🤟', '🤘', '🤙', '👏', '🙌', '👐', '🤲', '🙏', '✊', '👊', '🤛']
  },
  {
    name: 'Objects',
    emojis: ['💝', '💖', '💗', '💓', '💞', '💕', '💟', '❣️', '💔', '❤️', '🧡', '💛', '💚', '💙', '💜', '🤎']
  }
]

const FORMATTING_SHORTCUTS = [
  { key: 'bold', icon: FormatBold, markdown: '**', label: 'Bold (Ctrl+B)' },
  { key: 'italic', icon: FormatItalic, markdown: '*', label: 'Italic (Ctrl+I)' },
  { key: 'code', icon: Code, markdown: '`', label: 'Code (Ctrl+E)' },
  { key: 'quote', icon: FormatQuote, markdown: '> ', label: 'Quote' },
]

export const RichMessageComposer: React.FC<RichMessageComposerProps> = ({
  channelId,
  threadId,
  replyTo,
  onSend,
  onTyping,
  onAttachment,
  disabled = false,
  placeholder = 'Type a message...',
  autoFocus = false,
  ephemeral: defaultEphemeral = false,
}) => {
  const [draft, setDraft] = useState<DraftMessage>({
    text: '',
    formattedText: undefined,
    mentions: [],
    attachments: [],
    ephemeral: defaultEphemeral,
  })
  
  const [emojiAnchor, setEmojiAnchor] = useState<HTMLElement | null>(null)
  const [mentionAnchor, setMentionAnchor] = useState<HTMLElement | null>(null)
  const [mentionSearch, setMentionSearch] = useState('')
  const [availableUsers, setAvailableUsers] = useState<Array<{ id: string; name: string; avatar?: string }>>([])
  const [selectedEmoji, setSelectedEmoji] = useState<string[]>([])
  const [isUploading, setIsUploading] = useState(false)
  const [typingTimeout, setTypingTimeout] = useState<NodeJS.Timeout | null>(null)
  
  const textFieldRef = useRef<HTMLInputElement>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)

  // Load available users for mentions
  useEffect(() => {
    const loadUsers = async () => {
      try {
        // This would load from the actual P2P network
        const mockUsers = [
          { id: 'ocean-forest-moon-star', name: 'Alice', avatar: 'A' },
          { id: 'river-mountain-sun-cloud', name: 'Bob', avatar: 'B' },
          { id: 'desert-valley-wind-rain', name: 'Charlie', avatar: 'C' },
        ]
        setAvailableUsers(mockUsers)
      } catch (error) {
        console.error('Failed to load users:', error)
      }
    }
    loadUsers()
  }, [])

  // Handle typing indicator
  const handleTyping = useCallback(() => {
    if (onTyping) {
      onTyping(true)
      
      if (typingTimeout) {
        clearTimeout(typingTimeout)
      }
      
      const timeout = setTimeout(() => {
        onTyping(false)
      }, 3000)
      
      setTypingTimeout(timeout)
    }
  }, [onTyping, typingTimeout])

  // Handle text change
  const handleTextChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const text = event.target.value
    setDraft(prev => ({ ...prev, text }))
    
    // Check for @ mentions
    const lastWord = text.split(' ').pop() || ''
    if (lastWord.startsWith('@') && lastWord.length > 1) {
      setMentionSearch(lastWord.substring(1))
      setMentionAnchor(textFieldRef.current)
    } else {
      setMentionAnchor(null)
    }
    
    handleTyping()
  }

  // Apply formatting
  const applyFormatting = (markdown: string) => {
    const field = textFieldRef.current
    if (!field) return
    
    const start = field.selectionStart || 0
    const end = field.selectionEnd || 0
    const selectedText = draft.text.substring(start, end)
    
    let newText: string
    if (markdown === '> ') {
      // Quote formatting - add to beginning of line
      const lines = draft.text.substring(0, start).split('\n')
      const currentLineStart = draft.text.substring(0, start).lastIndexOf('\n') + 1
      newText = draft.text.substring(0, currentLineStart) + '> ' + draft.text.substring(currentLineStart)
    } else {
      // Wrap selected text
      newText = draft.text.substring(0, start) + markdown + selectedText + markdown + draft.text.substring(end)
    }
    
    setDraft(prev => ({ ...prev, text: newText }))
    
    // Restore focus and selection
    setTimeout(() => {
      field.focus()
      field.setSelectionRange(start + markdown.length, end + markdown.length)
    }, 0)
  }

  // Handle emoji selection
  const handleEmojiSelect = (emoji: string) => {
    const newText = draft.text + emoji
    setDraft(prev => ({ ...prev, text: newText }))
    setEmojiAnchor(null)
    
    // Track recently used emojis
    setSelectedEmoji(prev => [emoji, ...prev.filter(e => e !== emoji)].slice(0, 8))
    
    // Focus back on text field
    textFieldRef.current?.focus()
  }

  // Handle mention selection
  const handleMentionSelect = (user: { id: string; name: string }) => {
    const lastAtIndex = draft.text.lastIndexOf('@')
    const newText = draft.text.substring(0, lastAtIndex) + `@${user.name} `
    
    setDraft(prev => ({
      ...prev,
      text: newText,
      mentions: [...prev.mentions, user.id]
    }))
    
    setMentionAnchor(null)
    textFieldRef.current?.focus()
  }

  // Handle file selection
  const handleFileSelect = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(event.target.files || [])
    if (files.length === 0) return
    
    setIsUploading(true)
    
    try {
      // Process files
      setDraft(prev => ({ ...prev, attachments: [...prev.attachments, ...files] }))
      
      if (onAttachment) {
        onAttachment(files)
      }
    } catch (error) {
      console.error('Failed to process files:', error)
    } finally {
      setIsUploading(false)
    }
  }

  // Remove attachment
  const removeAttachment = (index: number) => {
    setDraft(prev => ({
      ...prev,
      attachments: prev.attachments.filter((_, i) => i !== index)
    }))
  }

  // Send message
  const handleSend = async () => {
    if (!draft.text.trim() && draft.attachments.length === 0) return
    
    try {
      const message = {
        channelId,
        content: draft.text.trim(),
        attachments: draft.attachments,
        threadId,
        replyTo,
        mentions: draft.mentions,
        ephemeral: draft.ephemeral,
      }
      
      await invoke('send_rich_message', { message })
      
      // Clear draft
      setDraft({
        text: '',
        formattedText: undefined,
        mentions: [],
        attachments: [],
        ephemeral: defaultEphemeral,
      })
      
      // Clear typing indicator
      if (onTyping) {
        onTyping(false)
      }
      
      if (onSend) {
        onSend(message)
      }
    } catch (error) {
      console.error('Failed to send message:', error)
    }
  }

  // Handle keyboard shortcuts
  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault()
      handleSend()
    } else if (event.ctrlKey || event.metaKey) {
      switch (event.key) {
        case 'b':
          event.preventDefault()
          applyFormatting('**')
          break
        case 'i':
          event.preventDefault()
          applyFormatting('*')
          break
        case 'e':
          event.preventDefault()
          applyFormatting('`')
          break
      }
    }
  }

  return (
    <Paper sx={{ p: 2, borderRadius: 2 }}>
      {/* Reply indicator */}
      {replyTo && (
        <Box sx={{ mb: 1, display: 'flex', alignItems: 'center', gap: 1 }}>
          <Typography variant="caption" color="text.secondary">
            Replying to message
          </Typography>
          <IconButton size="small" onClick={() => {}}>
            <Close fontSize="small" />
          </IconButton>
        </Box>
      )}
      
      {/* Thread indicator */}
      {threadId && (
        <Chip
          size="small"
          label="In thread"
          onDelete={() => {}}
          sx={{ mb: 1 }}
        />
      )}
      
      {/* Attachments preview */}
      {draft.attachments.length > 0 && (
        <Box sx={{ mb: 2, display: 'flex', gap: 1, flexWrap: 'wrap' }}>
          {draft.attachments.map((file, index) => (
            <Chip
              key={index}
              label={file.name}
              onDelete={() => removeAttachment(index)}
              icon={file.type.startsWith('image/') ? <Image /> : <AttachFile />}
              variant="outlined"
            />
          ))}
        </Box>
      )}
      
      {/* Upload progress */}
      {isUploading && <LinearProgress sx={{ mb: 1 }} />}
      
      {/* Main input area */}
      <Box sx={{ display: 'flex', gap: 1, alignItems: 'flex-end' }}>
        {/* Formatting toolbar */}
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 0.5 }}>
          {FORMATTING_SHORTCUTS.map(({ key, icon: Icon, markdown, label }) => (
            <Tooltip key={key} title={label} placement="left">
              <IconButton
                size="small"
                onClick={() => applyFormatting(markdown)}
                disabled={disabled}
              >
                <Icon fontSize="small" />
              </IconButton>
            </Tooltip>
          ))}
        </Box>
        
        {/* Text input */}
        <TextField
          ref={textFieldRef}
          fullWidth
          multiline
          maxRows={6}
          value={draft.text}
          onChange={handleTextChange}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          disabled={disabled}
          autoFocus={autoFocus}
          variant="outlined"
          sx={{ flex: 1 }}
        />
        
        {/* Action buttons */}
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 0.5 }}>
          <Tooltip title="Add emoji">
            <IconButton
              size="small"
              onClick={(e) => setEmojiAnchor(e.currentTarget)}
              disabled={disabled}
            >
              <EmojiEmotions />
            </IconButton>
          </Tooltip>
          
          <Tooltip title="Attach file">
            <IconButton
              size="small"
              onClick={() => fileInputRef.current?.click()}
              disabled={disabled}
            >
              <AttachFile />
            </IconButton>
          </Tooltip>
          
          <Tooltip title="Video call">
            <IconButton size="small" disabled>
              <VideoCall />
            </IconButton>
          </Tooltip>
          
          <Tooltip title="Voice message">
            <IconButton size="small" disabled>
              <Mic />
            </IconButton>
          </Tooltip>
          
          <Divider />
          
          <Button
            variant="contained"
            onClick={handleSend}
            disabled={disabled || (!draft.text.trim() && draft.attachments.length === 0)}
            startIcon={<Send />}
            size="small"
          >
            Send
          </Button>
        </Box>
      </Box>
      
      {/* Hidden file input */}
      <input
        ref={fileInputRef}
        type="file"
        multiple
        hidden
        onChange={handleFileSelect}
      />
      
      {/* Emoji picker */}
      <Popover
        open={Boolean(emojiAnchor)}
        anchorEl={emojiAnchor}
        onClose={() => setEmojiAnchor(null)}
        anchorOrigin={{
          vertical: 'top',
          horizontal: 'center',
        }}
        transformOrigin={{
          vertical: 'bottom',
          horizontal: 'center',
        }}
      >
        <Box sx={{ p: 2, maxWidth: 320 }}>
          {/* Recent emojis */}
          {selectedEmoji.length > 0 && (
            <>
              <Typography variant="caption" color="text.secondary">
                Recent
              </Typography>
              <Box sx={{ display: 'flex', gap: 0.5, mb: 2, flexWrap: 'wrap' }}>
                {selectedEmoji.map(emoji => (
                  <IconButton
                    key={emoji}
                    size="small"
                    onClick={() => handleEmojiSelect(emoji)}
                  >
                    <Typography>{emoji}</Typography>
                  </IconButton>
                ))}
              </Box>
            </>
          )}
          
          {/* Emoji categories */}
          {EMOJI_CATEGORIES.map(category => (
            <Box key={category.name} sx={{ mb: 2 }}>
              <Typography variant="caption" color="text.secondary">
                {category.name}
              </Typography>
              <Box sx={{ display: 'flex', gap: 0.5, flexWrap: 'wrap' }}>
                {category.emojis.map(emoji => (
                  <IconButton
                    key={emoji}
                    size="small"
                    onClick={() => handleEmojiSelect(emoji)}
                  >
                    <Typography>{emoji}</Typography>
                  </IconButton>
                ))}
              </Box>
            </Box>
          ))}
        </Box>
      </Popover>
      
      {/* Mention suggestions */}
      <Popover
        open={Boolean(mentionAnchor)}
        anchorEl={mentionAnchor}
        onClose={() => setMentionAnchor(null)}
        anchorOrigin={{
          vertical: 'top',
          horizontal: 'left',
        }}
        transformOrigin={{
          vertical: 'bottom',
          horizontal: 'left',
        }}
      >
        <List sx={{ width: 250 }}>
          {availableUsers
            .filter(user => 
              user.name.toLowerCase().includes(mentionSearch.toLowerCase())
            )
            .slice(0, 5)
            .map(user => (
              <ListItem
                key={user.id}
                button
                onClick={() => handleMentionSelect(user)}
              >
                <ListItemAvatar>
                  <Avatar sx={{ width: 32, height: 32 }}>
                    {user.avatar || user.name[0]}
                  </Avatar>
                </ListItemAvatar>
                <ListItemText
                  primary={user.name}
                  secondary={user.id}
                />
              </ListItem>
            ))}
        </List>
      </Popover>
      
      {/* Ephemeral message indicator */}
      {draft.ephemeral && (
        <Box sx={{ mt: 1, display: 'flex', alignItems: 'center', gap: 1 }}>
          <Schedule fontSize="small" color="warning" />
          <Typography variant="caption" color="warning.main">
            This message will disappear after being read
          </Typography>
        </Box>
      )}
    </Paper>
  )
}

export default RichMessageComposer