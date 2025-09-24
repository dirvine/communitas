import React, { useState, useEffect, useRef, useCallback } from 'react';
import {
  Box,
  Paper,
  TextField,
  IconButton,
  List,
  ListItem,
  ListItemText,
  ListItemAvatar,
  Avatar,
  Typography,
  Divider,
  Stack,
  Chip,
  CircularProgress,
  Alert,
  Tooltip,
  Menu,
  MenuItem,
  ListItemIcon,
  InputAdornment,
  Badge,
} from '@mui/material';
import {
  Send as SendIcon,
  AttachFile as AttachIcon,
  EmojiEmotions as EmojiIcon,
  MoreVert as MoreIcon,
  Reply as ReplyIcon,
  Forward as ForwardIcon,
  Delete as DeleteIcon,
  Edit as EditIcon,
  Check as CheckIcon,
  DoneAll as DoneAllIcon,
  Schedule as PendingIcon,
  Error as ErrorIcon,
  Lock as EncryptedIcon,
  Image as ImageIcon,
  VideoFile as VideoIcon,
  AudioFile as AudioIcon,
  Description as DocumentIcon,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { format, isToday, isYesterday } from 'date-fns';

interface Message {
  id: string;
  sender_id: string;
  sender_name: string;
  sender_four_words: string;
  content: string;
  timestamp: string;
  status: 'pending' | 'sent' | 'delivered' | 'read' | 'error';
  edited?: boolean;
  reply_to?: string;
  attachments?: Attachment[];
  encrypted: boolean;
  reactions?: Reaction[];
}

interface Attachment {
  id: string;
  name: string;
  size: number;
  type: 'image' | 'video' | 'audio' | 'document' | 'other';
  url?: string;
  thumbnail?: string;
}

interface Reaction {
  emoji: string;
  user_ids: string[];
}

interface MessagesPanelProps {
  entityType: 'individual' | 'group' | 'channel' | 'project';
  entityId: string;
  entityName: string;
  fourWords: string;
  permissions: string[];
}

const MessagesPanel: React.FC<MessagesPanelProps> = ({
  entityType,
  entityId,
  entityName,
  fourWords,
  permissions,
}) => {
  const [messages, setMessages] = useState<Message[]>([]);
  const [newMessage, setNewMessage] = useState('');
  const [loading, setLoading] = useState(true);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedMessage, setSelectedMessage] = useState<string | null>(null);
  const [editingMessage, setEditingMessage] = useState<string | null>(null);
  const [editContent, setEditContent] = useState('');
  const [menuAnchor, setMenuAnchor] = useState<null | HTMLElement>(null);
  const [typingUsers, setTypingUsers] = useState<string[]>([]);
  const messagesEndRef = useRef<null | HTMLDivElement>(null);
  const messageListRef = useRef<null | HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const canSend = permissions.includes('write') || permissions.includes('admin');
  const canDelete = permissions.includes('admin');
  const canEdit = permissions.includes('write') || permissions.includes('admin');

  // Load messages
  const loadMessages = useCallback(async () => {
    try {
      setLoading(true);
      const result = await invoke('core_messages_list', {
        entityId,
        limit: 100,
        offset: 0,
      });
      setMessages(result as Message[]);
      setError(null);
    } catch (err) {
      console.error('Failed to load messages:', err);
      setError('Failed to load messages');
    } finally {
      setLoading(false);
    }
  }, [entityId]);

  // Send message
  const sendMessage = async () => {
    if (!newMessage.trim() || !canSend || sending) return;

    setSending(true);
    try {
      const message = await invoke('core_messages_send', {
        entityId,
        content: newMessage.trim(),
        encrypted: true,
      });

      setMessages(prev => [...prev, message as Message]);
      setNewMessage('');
      scrollToBottom();
    } catch (err) {
      console.error('Failed to send message:', err);
      setError('Failed to send message');
    } finally {
      setSending(false);
    }
  };

  // Edit message
  const handleEdit = async () => {
    if (!editingMessage || !editContent.trim()) return;

    try {
      await invoke('core_messages_edit', {
        entityId,
        messageId: editingMessage,
        newContent: editContent.trim(),
      });

      setMessages(prev => prev.map(msg =>
        msg.id === editingMessage
          ? { ...msg, content: editContent.trim(), edited: true }
          : msg
      ));

      setEditingMessage(null);
      setEditContent('');
    } catch (err) {
      console.error('Failed to edit message:', err);
      setError('Failed to edit message');
    }
  };

  // Delete message
  const handleDelete = async (messageId: string) => {
    if (!canDelete) return;

    try {
      await invoke('core_messages_delete', {
        entityId,
        messageId,
      });

      setMessages(prev => prev.filter(msg => msg.id !== messageId));
    } catch (err) {
      console.error('Failed to delete message:', err);
      setError('Failed to delete message');
    }
  };

  // Handle file attachment
  const handleFileSelect = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = event.target.files;
    if (!files || files.length === 0) return;

    // TODO: Implement file upload through Tauri
    console.log('File upload not yet implemented');
  };

  // Real-time updates via events
  useEffect(() => {
    loadMessages();

    // Subscribe to message events
    const handleNewMessage = (event: CustomEvent) => {
      if (event.detail.entityId === entityId) {
        setMessages(prev => [...prev, event.detail.message]);
        scrollToBottom();
      }
    };

    const handleMessageEdit = (event: CustomEvent) => {
      if (event.detail.entityId === entityId) {
        setMessages(prev => prev.map(msg =>
          msg.id === event.detail.messageId
            ? { ...msg, ...event.detail.updates }
            : msg
        ));
      }
    };

    const handleMessageDelete = (event: CustomEvent) => {
      if (event.detail.entityId === entityId) {
        setMessages(prev => prev.filter(msg => msg.id !== event.detail.messageId));
      }
    };

    const handleTyping = (event: CustomEvent) => {
      if (event.detail.entityId === entityId) {
        setTypingUsers(event.detail.users);
      }
    };

    window.addEventListener('message:new', handleNewMessage as EventListener);
    window.addEventListener('message:edit', handleMessageEdit as EventListener);
    window.addEventListener('message:delete', handleMessageDelete as EventListener);
    window.addEventListener('message:typing', handleTyping as EventListener);

    // Refresh on entity:refresh event
    const handleRefresh = (event: CustomEvent) => {
      if (event.detail.entityId === entityId) {
        loadMessages();
      }
    };
    window.addEventListener('entity:refresh', handleRefresh as EventListener);

    return () => {
      window.removeEventListener('message:new', handleNewMessage as EventListener);
      window.removeEventListener('message:edit', handleMessageEdit as EventListener);
      window.removeEventListener('message:delete', handleMessageDelete as EventListener);
      window.removeEventListener('message:typing', handleTyping as EventListener);
      window.removeEventListener('entity:refresh', handleRefresh as EventListener);
    };
  }, [entityId, loadMessages]);

  // Auto-scroll to bottom
  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  // Format timestamp
  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    if (isToday(date)) {
      return format(date, 'HH:mm');
    } else if (isYesterday(date)) {
      return `Yesterday ${format(date, 'HH:mm')}`;
    } else {
      return format(date, 'dd/MM/yyyy HH:mm');
    }
  };

  // Get attachment icon
  const getAttachmentIcon = (type: string) => {
    switch (type) {
      case 'image': return <ImageIcon />;
      case 'video': return <VideoIcon />;
      case 'audio': return <AudioIcon />;
      case 'document': return <DocumentIcon />;
      default: return <AttachIcon />;
    }
  };

  // Get status icon
  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'pending': return <PendingIcon fontSize="small" />;
      case 'sent': return <CheckIcon fontSize="small" />;
      case 'delivered': return <DoneAllIcon fontSize="small" />;
      case 'read': return <DoneAllIcon fontSize="small" color="primary" />;
      case 'error': return <ErrorIcon fontSize="small" color="error" />;
      default: return null;
    }
  };

  if (loading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%' }}>
        <CircularProgress />
      </Box>
    );
  }

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Error Alert */}
      {error && (
        <Alert severity="error" onClose={() => setError(null)} sx={{ mb: 1 }}>
          {error}
        </Alert>
      )}

      {/* Messages List */}
      <Box
        ref={messageListRef}
        sx={{
          flex: 1,
          overflowY: 'auto',
          p: 2,
          bgcolor: theme => theme.palette.mode === 'dark' ? 'grey.900' : 'grey.50',
        }}
      >
        <List>
          {messages.map((message, index) => (
            <React.Fragment key={message.id}>
              {index > 0 && <Divider variant="inset" component="li" />}
              <ListItem
                alignItems="flex-start"
                onMouseEnter={() => setSelectedMessage(message.id)}
                onMouseLeave={() => setSelectedMessage(null)}
                sx={{
                  '&:hover': { bgcolor: 'action.hover' },
                  position: 'relative',
                }}
              >
                <ListItemAvatar>
                  <Avatar sx={{ bgcolor: 'primary.main' }}>
                    {message.sender_name.charAt(0).toUpperCase()}
                  </Avatar>
                </ListItemAvatar>

                <ListItemText
                  primary={
                    <Stack direction="row" alignItems="center" spacing={1}>
                      <Typography variant="subtitle2" fontWeight="bold">
                        {message.sender_name}
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        {message.sender_four_words}
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        {formatTimestamp(message.timestamp)}
                      </Typography>
                      {message.encrypted && (
                        <Tooltip title="Encrypted">
                          <EncryptedIcon fontSize="small" color="action" />
                        </Tooltip>
                      )}
                      {message.edited && (
                        <Typography variant="caption" color="text.secondary">
                          (edited)
                        </Typography>
                      )}
                    </Stack>
                  }
                  secondary={
                    <Box>
                      {editingMessage === message.id ? (
                        <Stack direction="row" spacing={1} alignItems="center">
                          <TextField
                            fullWidth
                            size="small"
                            value={editContent}
                            onChange={(e) => setEditContent(e.target.value)}
                            onKeyPress={(e) => {
                              if (e.key === 'Enter' && !e.shiftKey) {
                                e.preventDefault();
                                handleEdit();
                              }
                            }}
                            autoFocus
                          />
                          <IconButton size="small" onClick={handleEdit}>
                            <CheckIcon />
                          </IconButton>
                          <IconButton
                            size="small"
                            onClick={() => {
                              setEditingMessage(null);
                              setEditContent('');
                            }}
                          >
                            <CloseIcon />
                          </IconButton>
                        </Stack>
                      ) : (
                        <>
                          <Typography variant="body2" component="span">
                            {message.content}
                          </Typography>

                          {/* Attachments */}
                          {message.attachments && message.attachments.length > 0 && (
                            <Stack direction="row" spacing={1} sx={{ mt: 1 }}>
                              {message.attachments.map(attachment => (
                                <Chip
                                  key={attachment.id}
                                  icon={getAttachmentIcon(attachment.type)}
                                  label={attachment.name}
                                  size="small"
                                  variant="outlined"
                                  onClick={() => console.log('Open attachment:', attachment)}
                                />
                              ))}
                            </Stack>
                          )}

                          {/* Reactions */}
                          {message.reactions && message.reactions.length > 0 && (
                            <Stack direction="row" spacing={0.5} sx={{ mt: 0.5 }}>
                              {message.reactions.map((reaction, idx) => (
                                <Chip
                                  key={idx}
                                  label={`${reaction.emoji} ${reaction.user_ids.length}`}
                                  size="small"
                                  variant="filled"
                                  sx={{ height: 24 }}
                                />
                              ))}
                            </Stack>
                          )}
                        </>
                      )}
                    </Box>
                  }
                />

                {/* Message Actions */}
                {selectedMessage === message.id && !editingMessage && (
                  <Stack
                    direction="row"
                    spacing={0.5}
                    sx={{
                      position: 'absolute',
                      top: 8,
                      right: 8,
                    }}
                  >
                    <Tooltip title="Reply">
                      <IconButton size="small">
                        <ReplyIcon fontSize="small" />
                      </IconButton>
                    </Tooltip>

                    {canEdit && (
                      <Tooltip title="Edit">
                        <IconButton
                          size="small"
                          onClick={() => {
                            setEditingMessage(message.id);
                            setEditContent(message.content);
                          }}
                        >
                          <EditIcon fontSize="small" />
                        </IconButton>
                      </Tooltip>
                    )}

                    <Tooltip title="Forward">
                      <IconButton size="small">
                        <ForwardIcon fontSize="small" />
                      </IconButton>
                    </Tooltip>

                    {canDelete && (
                      <Tooltip title="Delete">
                        <IconButton
                          size="small"
                          onClick={() => handleDelete(message.id)}
                        >
                          <DeleteIcon fontSize="small" />
                        </IconButton>
                      </Tooltip>
                    )}

                    <IconButton
                      size="small"
                      onClick={(e) => setMenuAnchor(e.currentTarget)}
                    >
                      <MoreIcon fontSize="small" />
                    </IconButton>
                  </Stack>
                )}

                {/* Status Icon */}
                <Box sx={{ position: 'absolute', bottom: 8, right: 8 }}>
                  {getStatusIcon(message.status)}
                </Box>
              </ListItem>
            </React.Fragment>
          ))}
        </List>

        {/* Typing Indicator */}
        {typingUsers.length > 0 && (
          <Box sx={{ p: 2 }}>
            <Typography variant="body2" color="text.secondary">
              {typingUsers.join(', ')} {typingUsers.length === 1 ? 'is' : 'are'} typing...
            </Typography>
          </Box>
        )}

        <div ref={messagesEndRef} />
      </Box>

      {/* Message Input */}
      {canSend && (
        <Paper
          elevation={3}
          sx={{
            p: 2,
            borderTop: '1px solid',
            borderColor: 'divider',
          }}
        >
          <Stack direction="row" spacing={1} alignItems="flex-end">
            <input
              ref={fileInputRef}
              type="file"
              hidden
              multiple
              onChange={handleFileSelect}
            />

            <IconButton onClick={() => fileInputRef.current?.click()}>
              <AttachIcon />
            </IconButton>

            <TextField
              fullWidth
              multiline
              maxRows={4}
              placeholder={`Message ${entityName}...`}
              value={newMessage}
              onChange={(e) => setNewMessage(e.target.value)}
              onKeyPress={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault();
                  sendMessage();
                }
              }}
              disabled={sending}
              InputProps={{
                endAdornment: (
                  <InputAdornment position="end">
                    <IconButton size="small">
                      <EmojiIcon />
                    </IconButton>
                  </InputAdornment>
                ),
              }}
            />

            <IconButton
              color="primary"
              onClick={sendMessage}
              disabled={!newMessage.trim() || sending}
            >
              {sending ? <CircularProgress size={24} /> : <SendIcon />}
            </IconButton>
          </Stack>
        </Paper>
      )}

      {/* More Options Menu */}
      <Menu
        anchorEl={menuAnchor}
        open={Boolean(menuAnchor)}
        onClose={() => setMenuAnchor(null)}
      >
        <MenuItem onClick={() => setMenuAnchor(null)}>
          <ListItemIcon>
            <ReplyIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText>Reply</ListItemText>
        </MenuItem>
        <MenuItem onClick={() => setMenuAnchor(null)}>
          <ListItemIcon>
            <ForwardIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText>Forward</ListItemText>
        </MenuItem>
        <Divider />
        <MenuItem onClick={() => setMenuAnchor(null)}>
          <ListItemText>Copy Text</ListItemText>
        </MenuItem>
        <MenuItem onClick={() => setMenuAnchor(null)}>
          <ListItemText>Pin Message</ListItemText>
        </MenuItem>
      </Menu>
    </Box>
  );
};

export default MessagesPanel;