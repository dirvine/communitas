import React, { useState, useRef, useEffect } from 'react';
import {
  Box,
  Paper,
  Typography,
  TextField,
  IconButton,
  Avatar,
  Stack,
  Chip,
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  Divider,
  Card,
  CardContent,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  List,
  ListItem,
  ListItemButton,
  ListItemAvatar,
  Badge,
  Tooltip,
  InputAdornment,
  Collapse,
  Alert,
  LinearProgress,
  CircularProgress,
} from '@mui/material';
import {
  Send as SendIcon,
  AttachFile as AttachFileIcon,
  EmojiEmotions as EmojiIcon,
  Reply as ReplyIcon,
  Edit as EditIcon,
  Delete as DeleteIcon,
  MoreVert as MoreVertIcon,
  Phone as PhoneIcon,
  Videocam as VideocamIcon,
  Star as StarIcon,
  StarBorder as StarBorderIcon,
  Search as SearchIcon,
  FilterList as FilterIcon,
  Image as ImageIcon,
  VideoFile as VideoIcon,
  AudioFile as AudioIcon,
  InsertDriveFile as FileIcon,
  Link as LinkIcon,
  Schedule as ScheduleIcon,
  Check as CheckIcon,
  CheckCircle as CheckCircleIcon,
  Error as ErrorIcon,
  Refresh as RefreshIcon,
  VolumeUp as VolumeUpIcon,
  VolumeOff as VolumeOffIcon,
  Download as DownloadIcon,
  Share as ShareIcon,
  Forward as ForwardIcon,
  ContentCopy as CopyIcon,
  Bookmark as BookmarkIcon,
  Flag as FlagIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { Member } from '../../types/organization';

interface ChatMessage {
  id: string;
  sender: Member;
  content: string;
  timestamp: string;
  type: 'text' | 'file' | 'image' | 'video' | 'audio' | 'system' | 'emoji';
  status: 'sending' | 'sent' | 'delivered' | 'read' | 'failed';
  reply_to?: ChatMessage;
  reactions: MessageReaction[];
  attachments?: MessageAttachment[];
  edited?: boolean;
  edited_at?: string;
  is_starred?: boolean;
  is_pinned?: boolean;
  thread_count?: number;
}

interface MessageReaction {
  emoji: string;
  users: Member[];
  count: number;
}

interface MessageAttachment {
  id: string;
  name: string;
  size: number;
  type: 'image' | 'video' | 'audio' | 'document';
  url: string;
  thumbnail?: string;
  duration?: number; // for audio/video files
}

interface ChatInterfaceProps {
  participants: Member[];
  currentUser: Member;
  entityName: string;
  entityType: 'organization' | 'project' | 'group';
  onStartCall: (type: 'voice' | 'video') => void;
  onSendMessage: (content: string, type: string, attachments?: File[]) => void;
  isTypingUsers: Member[];
  onlineUsers: Member[];
}

const EMOJI_LIST = ['👍', '❤️', '😂', '😮', '😢', '😡', '👏', '🎉', '🔥', '💯'];

export const ChatInterface: React.FC<ChatInterfaceProps> = ({
  participants,
  currentUser,
  entityName,
  entityType,
  onStartCall,
  onSendMessage,
  isTypingUsers = [],
  onlineUsers = [],
}) => {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [newMessage, setNewMessage] = useState('');
  const [replyToMessage, setReplyToMessage] = useState<ChatMessage | null>(null);
  const [editingMessage, setEditingMessage] = useState<ChatMessage | null>(null);
  const [selectedMessage, setSelectedMessage] = useState<ChatMessage | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [showEmojiPicker, setShowEmojiPicker] = useState(false);
  const [showAttachMenu, setShowAttachMenu] = useState(false);
  const [menuAnchor, setMenuAnchor] = useState<null | HTMLElement>(null);
  const [attachFiles, setAttachFiles] = useState<File[]>([]);
  const [isUploading, setIsUploading] = useState(false);
  const [showSearch, setShowSearch] = useState(false);
  const [filteredMessages, setFilteredMessages] = useState<ChatMessage[]>([]);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const messageInputRef = useRef<HTMLTextAreaElement>(null);

  // Mock messages for demo
  useEffect(() => {
    setMessages([
      {
        id: '1',
        sender: participants[0],
        content: 'Hey everyone! Welcome to the project chat.',
        timestamp: new Date(Date.now() - 3600000).toISOString(),
        type: 'text',
        status: 'read',
        reactions: [
          { emoji: '👍', users: [participants[1]], count: 1 },
          { emoji: '🎉', users: [participants[1], currentUser], count: 2 },
        ],
        is_starred: true,
      },
      {
        id: '2',
        sender: participants[1],
        content: 'Thanks for setting this up! Looking forward to collaborating.',
        timestamp: new Date(Date.now() - 3000000).toISOString(),
        type: 'text',
        status: 'read',
        reactions: [],
        reply_to: undefined,
      },
      {
        id: '3',
        sender: currentUser,
        content: 'I\'ve uploaded the project requirements document to the files section.',
        timestamp: new Date(Date.now() - 1800000).toISOString(),
        type: 'text',
        status: 'read',
        reactions: [
          { emoji: '👍', users: [participants[0], participants[1]], count: 2 },
        ],
        attachments: [
          {
            id: 'a1',
            name: 'Project_Requirements.pdf',
            size: 2048576,
            type: 'document',
            url: '/files/requirements.pdf',
          },
        ],
      },
    ]);
  }, [participants, currentUser]);

  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // Filter messages based on search
  useEffect(() => {
    if (searchQuery.trim()) {
      setFilteredMessages(
        messages.filter(msg =>
          msg.content.toLowerCase().includes(searchQuery.toLowerCase()) ||
          msg.sender.display_name.toLowerCase().includes(searchQuery.toLowerCase())
        )
      );
    } else {
      setFilteredMessages(messages);
    }
  }, [messages, searchQuery]);

  const handleSendMessage = async () => {
    if (!newMessage.trim() && attachFiles.length === 0) return;

    const message: ChatMessage = {
      id: Date.now().toString(),
      sender: currentUser,
      content: newMessage || '',
      timestamp: new Date().toISOString(),
      type: attachFiles.length > 0 ? 'file' : 'text',
      status: 'sending',
      reactions: [],
      reply_to: replyToMessage || undefined,
      attachments: attachFiles.map(file => ({
        id: Math.random().toString(),
        name: file.name,
        size: file.size,
        type: file.type.startsWith('image/') ? 'image' :
              file.type.startsWith('video/') ? 'video' :
              file.type.startsWith('audio/') ? 'audio' : 'document',
        url: URL.createObjectURL(file),
      })),
    };

    setMessages(prev => [...prev, message]);
    setNewMessage('');
    setAttachFiles([]);
    setReplyToMessage(null);

    // Simulate sending
    setTimeout(() => {
      setMessages(prev => 
        prev.map(m => 
          m.id === message.id 
            ? { ...m, status: 'sent' as const }
            : m
        )
      );
    }, 1000);

    // Call parent callback
    onSendMessage(newMessage, message.type, attachFiles);
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  };

  const handleReaction = (messageId: string, emoji: string) => {
    setMessages(prev =>
      prev.map(msg => {
        if (msg.id !== messageId) return msg;
        
        const existingReaction = msg.reactions.find(r => r.emoji === emoji);
        const userAlreadyReacted = existingReaction?.users.some(u => u.user_id === currentUser.user_id);
        
        if (userAlreadyReacted) {
          // Remove reaction
          return {
            ...msg,
            reactions: msg.reactions.map(r =>
              r.emoji === emoji
                ? {
                    ...r,
                    users: r.users.filter(u => u.user_id !== currentUser.user_id),
                    count: r.count - 1,
                  }
                : r
            ).filter(r => r.count > 0),
          };
        } else {
          // Add reaction
          if (existingReaction) {
            return {
              ...msg,
              reactions: msg.reactions.map(r =>
                r.emoji === emoji
                  ? {
                      ...r,
                      users: [...r.users, currentUser],
                      count: r.count + 1,
                    }
                  : r
              ),
            };
          } else {
            return {
              ...msg,
              reactions: [
                ...msg.reactions,
                { emoji, users: [currentUser], count: 1 },
              ],
            };
          }
        }
      })
    );
  };

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      setAttachFiles(Array.from(e.target.files));
    }
  };

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const getMessageStatusIcon = (status: string) => {
    switch (status) {
      case 'sending': return <CircularProgress size={12} />;
      case 'sent': return <CheckIcon sx={{ fontSize: 12, color: 'grey.500' }} />;
      case 'delivered': return <CheckCircleIcon sx={{ fontSize: 12, color: 'info.main' }} />;
      case 'read': return <CheckCircleIcon sx={{ fontSize: 12, color: 'success.main' }} />;
      case 'failed': return <ErrorIcon sx={{ fontSize: 12, color: 'error.main' }} />;
      default: return null;
    }
  };

  const getFileIcon = (type: string) => {
    switch (type) {
      case 'image': return <ImageIcon />;
      case 'video': return <VideoIcon />;
      case 'audio': return <AudioIcon />;
      default: return <FileIcon />;
    }
  };

  const handleMenuOpen = (event: React.MouseEvent<HTMLElement>, message: ChatMessage) => {
    setMenuAnchor(event.currentTarget);
    setSelectedMessage(message);
  };

  const handleMenuClose = () => {
    setMenuAnchor(null);
    setSelectedMessage(null);
  };

  const renderMessage = (message: ChatMessage) => {
    const isOwnMessage = message.sender.user_id === currentUser.user_id;
    const showAvatar = !isOwnMessage;

    return (
      <motion.div
        key={message.id}
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.3 }}
      >
        <Box
          sx={{
            display: 'flex',
            flexDirection: isOwnMessage ? 'row-reverse' : 'row',
            alignItems: 'flex-start',
            gap: 1,
            mb: 1,
            px: 2,
          }}
        >
          {showAvatar && (
            <Avatar sx={{ width: 32, height: 32 }}>
              {message.sender.display_name[0]}
            </Avatar>
          )}
          
          <Box
            sx={{
              maxWidth: '70%',
              display: 'flex',
              flexDirection: 'column',
              alignItems: isOwnMessage ? 'flex-end' : 'flex-start',
            }}
          >
            {!isOwnMessage && (
              <Typography variant="caption" color="text.secondary" sx={{ mb: 0.5, px: 1 }}>
                {message.sender.display_name}
              </Typography>
            )}
            
            {/* Reply indicator */}
            {message.reply_to && (
              <Card variant="outlined" sx={{ mb: 0.5, maxWidth: '100%' }}>
                <CardContent sx={{ py: 1, px: 2 }}>
                  <Typography variant="caption" color="text.secondary">
                    Replying to {message.reply_to.sender.display_name}
                  </Typography>
                  <Typography variant="body2" sx={{ 
                    opacity: 0.7,
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: 'nowrap',
                    maxWidth: 200,
                  }}>
                    {message.reply_to.content}
                  </Typography>
                </CardContent>
              </Card>
            )}

            <Paper
              elevation={1}
              sx={{
                p: 1.5,
                bgcolor: isOwnMessage ? 'primary.main' : 'background.paper',
                color: isOwnMessage ? 'primary.contrastText' : 'text.primary',
                borderRadius: 2,
                borderTopLeftRadius: !isOwnMessage ? 0.5 : 2,
                borderTopRightRadius: isOwnMessage ? 0.5 : 2,
                position: 'relative',
                maxWidth: '100%',
              }}
            >
              {/* Message content */}
              <Typography variant="body2" sx={{ wordBreak: 'break-word' }}>
                {message.content}
              </Typography>

              {/* Attachments */}
              {message.attachments && message.attachments.map((attachment) => (
                <Card key={attachment.id} variant="outlined" sx={{ mt: 1, maxWidth: 300 }}>
                  <CardContent sx={{ py: 1, px: 2 }}>
                    <Stack direction="row" alignItems="center" spacing={1}>
                      {getFileIcon(attachment.type)}
                      <Box sx={{ flexGrow: 1, minWidth: 0 }}>
                        <Typography variant="body2" noWrap>
                          {attachment.name}
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                          {formatFileSize(attachment.size)}
                        </Typography>
                      </Box>
                      <IconButton size="small">
                        <DownloadIcon fontSize="small" />
                      </IconButton>
                    </Stack>
                  </CardContent>
                </Card>
              ))}

              {/* Message metadata */}
              <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ mt: 0.5 }}>
                <Typography variant="caption" sx={{ opacity: 0.7 }}>
                  {new Date(message.timestamp).toLocaleTimeString([], { 
                    hour: '2-digit', 
                    minute: '2-digit' 
                  })}
                  {message.edited && ' (edited)'}
                </Typography>
                <Stack direction="row" alignItems="center" spacing={0.5}>
                  {message.is_starred && <StarIcon sx={{ fontSize: 12, color: 'warning.main' }} />}
                  {message.is_pinned && <BookmarkIcon sx={{ fontSize: 12, color: 'info.main' }} />}
                  {isOwnMessage && getMessageStatusIcon(message.status)}
                  <IconButton 
                    size="small" 
                    onClick={(e) => handleMenuOpen(e, message)}
                    sx={{ p: 0.25 }}
                  >
                    <MoreVertIcon sx={{ fontSize: 12 }} />
                  </IconButton>
                </Stack>
              </Stack>

              {/* Reactions */}
              {message.reactions.length > 0 && (
                <Stack direction="row" spacing={0.5} sx={{ mt: 1 }}>
                  {message.reactions.map((reaction) => (
                    <Chip
                      key={reaction.emoji}
                      label={`${reaction.emoji} ${reaction.count}`}
                      size="small"
                      clickable
                      onClick={() => handleReaction(message.id, reaction.emoji)}
                      sx={{ 
                        height: 20, 
                        fontSize: '0.7rem',
                        bgcolor: reaction.users.some(u => u.user_id === currentUser.user_id) 
                          ? 'primary.light' 
                          : 'background.default',
                      }}
                    />
                  ))}
                </Stack>
              )}
            </Paper>
          </Box>
        </Box>
      </motion.div>
    );
  };

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <Paper 
        elevation={1} 
        sx={{ 
          p: 2, 
          borderRadius: 0, 
          borderBottom: '1px solid',
          borderColor: 'divider',
        }}
      >
        <Stack direction="row" alignItems="center" justifyContent="space-between">
          <Box>
            <Typography variant="h6">{entityName}</Typography>
            <Stack direction="row" alignItems="center" spacing={1}>
              <Chip 
                label={`${participants.length} members`}
                size="small"
                variant="outlined"
              />
              <Badge
                badgeContent={onlineUsers.length}
                color="success"
                max={99}
              >
                <Typography variant="caption" color="text.secondary">
                  Online
                </Typography>
              </Badge>
            </Stack>
          </Box>
          
          <Stack direction="row" spacing={1}>
            <IconButton onClick={() => setShowSearch(!showSearch)}>
              <SearchIcon />
            </IconButton>
            <IconButton onClick={() => onStartCall('voice')}>
              <PhoneIcon />
            </IconButton>
            <IconButton onClick={() => onStartCall('video')}>
              <VideocamIcon />
            </IconButton>
          </Stack>
        </Stack>

        {/* Search Bar */}
        <Collapse in={showSearch}>
          <TextField
            fullWidth
            size="small"
            placeholder="Search messages..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            InputProps={{
              startAdornment: (
                <InputAdornment position="start">
                  <SearchIcon />
                </InputAdornment>
              ),
            }}
            sx={{ mt: 2 }}
          />
        </Collapse>
      </Paper>

      {/* Messages Area */}
      <Box sx={{ flexGrow: 1, overflow: 'auto', bgcolor: 'background.default' }}>
        {filteredMessages.map(renderMessage)}
        
        {/* Typing indicators */}
        {isTypingUsers.length > 0 && (
          <Box sx={{ px: 2, py: 1 }}>
            <Stack direction="row" alignItems="center" spacing={1}>
              <Avatar sx={{ width: 24, height: 24 }}>
                {isTypingUsers[0].display_name[0]}
              </Avatar>
              <Typography variant="caption" color="text.secondary">
                {isTypingUsers.length === 1
                  ? `${isTypingUsers[0].display_name} is typing...`
                  : `${isTypingUsers.length} people are typing...`
                }
              </Typography>
              <Box sx={{ display: 'flex', gap: 0.25 }}>
                {[0, 1, 2].map((dot) => (
                  <Box
                    key={dot}
                    sx={{
                      width: 4,
                      height: 4,
                      borderRadius: '50%',
                      bgcolor: 'text.secondary',
                      animation: 'pulse 1.4s ease-in-out infinite',
                      animationDelay: `${dot * 0.2}s`,
                    }}
                  />
                ))}
              </Box>
            </Stack>
          </Box>
        )}
        
        <div ref={messagesEndRef} />
      </Box>

      {/* Reply Preview */}
      {replyToMessage && (
        <Paper sx={{ p: 1, m: 1, bgcolor: 'action.hover' }}>
          <Stack direction="row" alignItems="center" justifyContent="space-between">
            <Stack direction="row" alignItems="center" spacing={1}>
              <ReplyIcon fontSize="small" />
              <Typography variant="caption">
                Replying to {replyToMessage.sender.display_name}
              </Typography>
              <Typography variant="caption" color="text.secondary" noWrap sx={{ maxWidth: 200 }}>
                {replyToMessage.content}
              </Typography>
            </Stack>
            <IconButton size="small" onClick={() => setReplyToMessage(null)}>
              <DeleteIcon fontSize="small" />
            </IconButton>
          </Stack>
        </Paper>
      )}

      {/* File Preview */}
      {attachFiles.length > 0 && (
        <Paper sx={{ p: 1, m: 1, bgcolor: 'action.hover' }}>
          <Stack spacing={1}>
            <Typography variant="caption" color="primary">
              {attachFiles.length} file(s) selected:
            </Typography>
            {attachFiles.map((file, index) => (
              <Stack key={index} direction="row" alignItems="center" justifyContent="space-between">
                <Stack direction="row" alignItems="center" spacing={1}>
                  {getFileIcon(file.type)}
                  <Typography variant="caption">
                    {file.name} ({formatFileSize(file.size)})
                  </Typography>
                </Stack>
                <IconButton 
                  size="small" 
                  onClick={() => setAttachFiles(files => files.filter((_, i) => i !== index))}
                >
                  <DeleteIcon fontSize="small" />
                </IconButton>
              </Stack>
            ))}
          </Stack>
        </Paper>
      )}

      {/* Message Input */}
      <Paper 
        elevation={2} 
        sx={{ 
          p: 2, 
          borderRadius: 0,
          borderTop: '1px solid',
          borderColor: 'divider',
        }}
      >
        <Stack direction="row" alignItems="flex-end" spacing={1}>
          <IconButton 
            onClick={() => setShowAttachMenu(true)}
            color={attachFiles.length > 0 ? 'primary' : 'default'}
          >
            <AttachFileIcon />
          </IconButton>
          
          <TextField
            fullWidth
            multiline
            maxRows={4}
            placeholder="Type a message..."
            value={newMessage}
            onChange={(e) => setNewMessage(e.target.value)}
            onKeyPress={handleKeyPress}
            disabled={isUploading}
            inputRef={messageInputRef}
          />
          
          <IconButton onClick={() => setShowEmojiPicker(true)}>
            <EmojiIcon />
          </IconButton>
          
          <IconButton 
            onClick={handleSendMessage}
            disabled={!newMessage.trim() && attachFiles.length === 0}
            color="primary"
          >
            <SendIcon />
          </IconButton>
        </Stack>

        {/* Uploading progress */}
        {isUploading && (
          <LinearProgress sx={{ mt: 1 }} />
        )}
      </Paper>

      {/* Context Menu */}
      <Menu
        anchorEl={menuAnchor}
        open={Boolean(menuAnchor)}
        onClose={handleMenuClose}
      >
        <MenuItem onClick={() => { handleMenuClose(); setReplyToMessage(selectedMessage); }}>
          <ListItemIcon><ReplyIcon /></ListItemIcon>
          <ListItemText>Reply</ListItemText>
        </MenuItem>
        <MenuItem onClick={() => { handleMenuClose(); /* handle copy */ }}>
          <ListItemIcon><CopyIcon /></ListItemIcon>
          <ListItemText>Copy</ListItemText>
        </MenuItem>
        <MenuItem onClick={() => { handleMenuClose(); /* handle star */ }}>
          <ListItemIcon>
            {selectedMessage?.is_starred ? <StarIcon /> : <StarBorderIcon />}
          </ListItemIcon>
          <ListItemText>{selectedMessage?.is_starred ? 'Unstar' : 'Star'}</ListItemText>
        </MenuItem>
        <MenuItem onClick={() => { handleMenuClose(); /* handle forward */ }}>
          <ListItemIcon><ForwardIcon /></ListItemIcon>
          <ListItemText>Forward</ListItemText>
        </MenuItem>
        <Divider />
        {selectedMessage?.sender.user_id === currentUser.user_id && (
          <>
            <MenuItem onClick={() => { handleMenuClose(); setEditingMessage(selectedMessage); }}>
              <ListItemIcon><EditIcon /></ListItemIcon>
              <ListItemText>Edit</ListItemText>
            </MenuItem>
            <MenuItem 
              onClick={() => { handleMenuClose(); /* handle delete */ }}
              sx={{ color: 'error.main' }}
            >
              <ListItemIcon><DeleteIcon color="error" /></ListItemIcon>
              <ListItemText>Delete</ListItemText>
            </MenuItem>
          </>
        )}
        <MenuItem onClick={() => { handleMenuClose(); /* handle flag */ }}>
          <ListItemIcon><FlagIcon /></ListItemIcon>
          <ListItemText>Report</ListItemText>
        </MenuItem>
      </Menu>

      {/* Emoji Picker Dialog */}
      <Dialog 
        open={showEmojiPicker} 
        onClose={() => setShowEmojiPicker(false)}
        PaperProps={{ sx: { borderRadius: 2 } }}
      >
        <DialogTitle>Add Reaction</DialogTitle>
        <DialogContent>
          <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
            {EMOJI_LIST.map((emoji) => (
              <Button
                key={emoji}
                onClick={() => {
                  setNewMessage(prev => prev + emoji);
                  setShowEmojiPicker(false);
                }}
                sx={{ minWidth: 48, height: 48, fontSize: '1.5rem' }}
              >
                {emoji}
              </Button>
            ))}
          </Stack>
        </DialogContent>
      </Dialog>

      {/* File Attach Dialog */}
      <Dialog 
        open={showAttachMenu} 
        onClose={() => setShowAttachMenu(false)}
        PaperProps={{ sx: { borderRadius: 2 } }}
      >
        <DialogTitle>Attach Files</DialogTitle>
        <DialogContent>
          <input
            type="file"
            multiple
            onChange={handleFileSelect}
            style={{ display: 'none' }}
            ref={fileInputRef}
          />
          <Stack spacing={2}>
            <Button
              fullWidth
              variant="outlined"
              startIcon={<FileIcon />}
              onClick={() => fileInputRef.current?.click()}
            >
              Choose Files
            </Button>
            <Button
              fullWidth
              variant="outlined"
              startIcon={<ImageIcon />}
              onClick={() => {
                if (fileInputRef.current) {
                  fileInputRef.current.accept = 'image/*';
                  fileInputRef.current.click();
                }
              }}
            >
              Images Only
            </Button>
            <Button
              fullWidth
              variant="outlined"
              startIcon={<VideoIcon />}
              onClick={() => {
                if (fileInputRef.current) {
                  fileInputRef.current.accept = 'video/*';
                  fileInputRef.current.click();
                }
              }}
            >
              Videos Only
            </Button>
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShowAttachMenu(false)}>Cancel</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default ChatInterface;