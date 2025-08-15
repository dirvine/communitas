import React, { useState, useRef, useEffect } from 'react';
import {
  Box,
  Paper,
  TextField,
  IconButton,
  Avatar,
  Typography,
  List,
  ListItem,
  ListItemAvatar,
  ListItemText,
  Divider,
  Chip,
  Badge,
  InputAdornment,
  Menu,
  MenuItem,
  Tooltip,
} from '@mui/material';
import {
  Send as SendIcon,
  AttachFile as AttachFileIcon,
  EmojiEmotions as EmojiIcon,
  Mic as MicIcon,
  VideocamOutlined as VideoIcon,
  CallOutlined as CallIcon,
  MoreVert as MoreIcon,
  Done as DoneIcon,
  DoneAll as DoneAllIcon,
  Search as SearchIcon,
} from '@mui/icons-material';

interface Message {
  id: string;
  sender: string;
  senderAvatar?: string;
  content: string;
  timestamp: Date;
  status: 'sending' | 'sent' | 'delivered' | 'read';
  attachments?: Array<{
    type: 'image' | 'video' | 'file' | 'audio';
    url: string;
    name: string;
  }>;
  reactions?: Array<{
    emoji: string;
    users: string[];
  }>;
  replyTo?: string;
}

interface ChatInterfaceProps {
  chatId: string;
  chatName: string;
  chatType: 'direct' | 'group' | 'channel';
  participants?: number;
  onSendMessage?: (content: string, attachments?: File[]) => void;
  onStartCall?: (type: 'voice' | 'video') => void;
}

export const ChatInterface: React.FC<ChatInterfaceProps> = ({
  chatId,
  chatName,
  chatType,
  participants = 1,
  onSendMessage,
  onStartCall,
}) => {
  const [message, setMessage] = useState('');
  const [messages, setMessages] = useState<Message[]>([
    {
      id: '1',
      sender: 'Alice Johnson',
      content: 'Hey! Have you seen the new P2P features?',
      timestamp: new Date(Date.now() - 3600000),
      status: 'read',
    },
    {
      id: '2',
      sender: 'You',
      content: 'Yes! The Reed-Solomon erasure coding is amazing for reliability.',
      timestamp: new Date(Date.now() - 3000000),
      status: 'delivered',
    },
    {
      id: '3',
      sender: 'Alice Johnson',
      content: 'Exactly! And the four-word addressing makes it so easy to connect.',
      timestamp: new Date(Date.now() - 2400000),
      status: 'read',
    },
    {
      id: '4',
      sender: 'Bob Chen',
      content: 'Just uploaded the project files to our shared space. Check them out!',
      timestamp: new Date(Date.now() - 1800000),
      status: 'read',
      attachments: [
        { type: 'file', url: '#', name: 'project-specs.pdf' },
        { type: 'file', url: '#', name: 'architecture.png' },
      ],
    },
    {
      id: '5',
      sender: 'You',
      content: 'Great! Looking at them now.',
      timestamp: new Date(Date.now() - 900000),
      status: 'delivered',
      reactions: [
        { emoji: '👍', users: ['Alice Johnson', 'Bob Chen'] },
      ],
    },
  ]);
  const [isTyping, setIsTyping] = useState(false);
  const [attachedFiles, setAttachedFiles] = useState<File[]>([]);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSend = () => {
    if (message.trim() || attachedFiles.length > 0) {
      const newMessage: Message = {
        id: Date.now().toString(),
        sender: 'You',
        content: message,
        timestamp: new Date(),
        status: 'sending',
        attachments: attachedFiles.map(file => ({
          type: file.type.startsWith('image/') ? 'image' : 'file',
          url: URL.createObjectURL(file),
          name: file.name,
        })),
      };

      setMessages([...messages, newMessage]);
      setMessage('');
      setAttachedFiles([]);

      // Simulate message status updates
      setTimeout(() => {
        setMessages(prev => prev.map(msg => 
          msg.id === newMessage.id ? { ...msg, status: 'sent' } : msg
        ));
      }, 500);

      setTimeout(() => {
        setMessages(prev => prev.map(msg => 
          msg.id === newMessage.id ? { ...msg, status: 'delivered' } : msg
        ));
      }, 1500);

      if (onSendMessage) {
        onSendMessage(message, attachedFiles);
      }
    }
  };

  const handleFileAttach = (event: React.ChangeEvent<HTMLInputElement>) => {
    if (event.target.files) {
      setAttachedFiles(Array.from(event.target.files));
    }
  };

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString('en-US', { 
      hour: 'numeric', 
      minute: '2-digit' 
    });
  };

  const MessageStatus = ({ status }: { status: Message['status'] }) => {
    switch (status) {
      case 'sending':
        return <CircularProgress size={12} />;
      case 'sent':
        return <DoneIcon sx={{ fontSize: 16 }} />;
      case 'delivered':
        return <DoneAllIcon sx={{ fontSize: 16 }} />;
      case 'read':
        return <DoneAllIcon sx={{ fontSize: 16, color: 'primary.main' }} />;
      default:
        return null;
    }
  };

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Chat Header */}
      <Paper
        elevation={0}
        sx={{
          p: 2,
          borderBottom: 1,
          borderColor: 'divider',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
          <Badge
            overlap="circular"
            anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
            variant="dot"
            sx={{
              '& .MuiBadge-badge': {
                backgroundColor: '#44b700',
                color: '#44b700',
              },
            }}
          >
            <Avatar>{chatName[0]}</Avatar>
          </Badge>
          <Box>
            <Typography variant="h6" component="div">
              {chatName}
            </Typography>
            <Typography variant="caption" color="text.secondary">
              {chatType === 'group' ? `${participants} participants` : 'Active now'}
            </Typography>
          </Box>
        </Box>
        <Box>
          <Tooltip title="Voice call">
            <IconButton onClick={() => onStartCall?.('voice')}>
              <CallIcon />
            </IconButton>
          </Tooltip>
          <Tooltip title="Video call">
            <IconButton onClick={() => onStartCall?.('video')}>
              <VideoIcon />
            </IconButton>
          </Tooltip>
          <IconButton>
            <SearchIcon />
          </IconButton>
          <IconButton>
            <MoreIcon />
          </IconButton>
        </Box>
      </Paper>

      {/* Messages Area */}
      <Box
        sx={{
          flex: 1,
          overflow: 'auto',
          p: 2,
          backgroundColor: 'background.default',
        }}
      >
        <List>
          {messages.map((msg, index) => (
            <ListItem
              key={msg.id}
              sx={{
                flexDirection: msg.sender === 'You' ? 'row-reverse' : 'row',
                gap: 1,
                px: 0,
              }}
            >
              {msg.sender !== 'You' && (
                <ListItemAvatar>
                  <Avatar sx={{ width: 32, height: 32 }}>
                    {msg.sender[0]}
                  </Avatar>
                </ListItemAvatar>
              )}
              <Paper
                elevation={1}
                sx={{
                  p: 1.5,
                  maxWidth: '70%',
                  backgroundColor: msg.sender === 'You' 
                    ? 'primary.main' 
                    : 'background.paper',
                  color: msg.sender === 'You' 
                    ? 'primary.contrastText' 
                    : 'text.primary',
                }}
              >
                {msg.sender !== 'You' && (
                  <Typography variant="caption" display="block" fontWeight={600}>
                    {msg.sender}
                  </Typography>
                )}
                <Typography variant="body2">{msg.content}</Typography>
                
                {msg.attachments && (
                  <Box sx={{ mt: 1 }}>
                    {msg.attachments.map((attachment, i) => (
                      <Chip
                        key={i}
                        label={attachment.name}
                        size="small"
                        icon={<AttachFileIcon />}
                        sx={{ mr: 0.5, mb: 0.5 }}
                      />
                    ))}
                  </Box>
                )}

                {msg.reactions && (
                  <Box sx={{ mt: 0.5 }}>
                    {msg.reactions.map((reaction, i) => (
                      <Chip
                        key={i}
                        label={`${reaction.emoji} ${reaction.users.length}`}
                        size="small"
                        sx={{ mr: 0.5 }}
                      />
                    ))}
                  </Box>
                )}

                <Box sx={{ 
                  display: 'flex', 
                  alignItems: 'center', 
                  gap: 0.5, 
                  mt: 0.5,
                  justifyContent: msg.sender === 'You' ? 'flex-end' : 'flex-start',
                }}>
                  <Typography variant="caption" sx={{ opacity: 0.7 }}>
                    {formatTime(msg.timestamp)}
                  </Typography>
                  {msg.sender === 'You' && <MessageStatus status={msg.status} />}
                </Box>
              </Paper>
            </ListItem>
          ))}
        </List>
        
        {isTyping && (
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, pl: 2 }}>
            <Avatar sx={{ width: 24, height: 24 }}>A</Avatar>
            <Typography variant="caption" color="text.secondary">
              Alice is typing...
            </Typography>
          </Box>
        )}
        
        <div ref={messagesEndRef} />
      </Box>

      {/* Message Input */}
      <Paper
        elevation={3}
        sx={{
          p: 2,
          borderTop: 1,
          borderColor: 'divider',
        }}
      >
        {attachedFiles.length > 0 && (
          <Box sx={{ mb: 1 }}>
            {attachedFiles.map((file, index) => (
              <Chip
                key={index}
                label={file.name}
                onDelete={() => {
                  setAttachedFiles(files => files.filter((_, i) => i !== index));
                }}
                size="small"
                sx={{ mr: 0.5 }}
              />
            ))}
          </Box>
        )}
        
        <Box sx={{ display: 'flex', gap: 1, alignItems: 'flex-end' }}>
          <input
            type="file"
            ref={fileInputRef}
            onChange={handleFileAttach}
            style={{ display: 'none' }}
            multiple
          />
          
          <IconButton onClick={() => fileInputRef.current?.click()}>
            <AttachFileIcon />
          </IconButton>
          
          <TextField
            fullWidth
            multiline
            maxRows={4}
            placeholder="Type a message..."
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyPress={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                handleSend();
              }
            }}
            InputProps={{
              endAdornment: (
                <InputAdornment position="end">
                  <IconButton>
                    <EmojiIcon />
                  </IconButton>
                </InputAdornment>
              ),
            }}
          />
          
          {message.trim() || attachedFiles.length > 0 ? (
            <IconButton color="primary" onClick={handleSend}>
              <SendIcon />
            </IconButton>
          ) : (
            <IconButton>
              <MicIcon />
            </IconButton>
          )}
        </Box>
      </Paper>
    </Box>
  );
};

// Add missing import
import { CircularProgress } from '@mui/material';