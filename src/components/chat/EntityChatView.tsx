import React, { useState, useEffect, useRef, useCallback } from 'react';
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
  Tab,
  Tabs,
  Grid,
  Slide,
  useMediaQuery,
  useTheme,
  Drawer,
  SwipeableDrawer,
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
  Group as GroupIcon,
  Person as PersonIcon,
  Business as BusinessIcon,
  Work as ProjectIcon,
  Tag as ChannelIcon,
  Add as AddIcon,
  Remove as RemoveIcon,
  Forum as ThreadIcon,
  Close as CloseIcon,
  ArrowBack as ArrowBackIcon,
  PushPin as PinIcon,
  Notifications as NotificationIcon,
  Settings as SettingsIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { validateFourWordIdentity } from '../../utils/identity';
import { webRTCService } from '../../services/communication/WebRTCService';
import { backendService } from '../../services/api/BackendService';
import { useEntityDirectory } from '../../contexts/EntityDirectoryContext';
import { loadMessages as loadStoredMessages, upsertMessage } from '../../utils/messageStore';

// Transform backend message format to component format
const transformBackendMessage = (backendMessage: any): Message => {
  return {
    id: backendMessage.id || backendMessage.message_id,
    sender: {
      id: backendMessage.sender_id || backendMessage.sender?.id,
      name: backendMessage.sender_name || backendMessage.sender?.name || 'Unknown',
      fourWordAddress: backendMessage.sender_four_words || backendMessage.sender?.four_words || 'unknown-user-address',
      avatar: backendMessage.sender?.avatar,
    },
    content: backendMessage.content || backendMessage.text || '',
    timestamp: backendMessage.timestamp || backendMessage.created_at || new Date().toISOString(),
    type: backendMessage.message_type || backendMessage.type || 'text',
    status: backendMessage.status || 'read',
    reactions: backendMessage.reactions || [],
    replyTo: backendMessage.reply_to ? {
      id: backendMessage.reply_to.id,
      sender: backendMessage.reply_to.sender_name,
      preview: backendMessage.reply_to.content?.slice(0, 50) + '...',
    } : undefined,
    threadId: backendMessage.thread_id,
    threadCount: backendMessage.thread_count || 0,
    attachments: backendMessage.attachments || [],
    edited: backendMessage.edited || false,
    pinned: backendMessage.pinned || false,
    starred: backendMessage.starred || false,
  };
};

// Transform backend member format to component format  
const transformBackendMember = (backendMember: any): Member => {
  return {
    id: backendMember.user_id || backendMember.id,
    name: backendMember.display_name || backendMember.name || 'Unknown',
    fourWordAddress: backendMember.four_words || backendMember.four_word_address || 'unknown-member-address',
    avatar: backendMember.avatar,
    role: backendMember.role || 'member',
    status: backendMember.status || 'offline',
    lastSeen: backendMember.last_seen || new Date().toISOString(),
  };
};

export interface Message {
  id: string;
  sender: {
    id: string;
    name: string;
    avatar?: string;
    fourWordAddress: string;
  };
  content: string;
  timestamp: string;
  type: 'text' | 'file' | 'image' | 'video' | 'audio' | 'system';
  status: 'sending' | 'sent' | 'delivered' | 'read' | 'failed';
  replyTo?: {
    id: string;
    sender: string;
    preview: string;
  };
  threadId?: string;
  threadCount?: number;
  reactions: Array<{
    emoji: string;
    users: string[];
  }>;
  attachments?: Array<{
    id: string;
    name: string;
    size: number;
    type: 'image' | 'video' | 'audio' | 'document';
    url: string;
    thumbnail?: string;
  }>;
  edited?: boolean;
  pinned?: boolean;
  starred?: boolean;
}

interface Thread {
  id: string;
  parentMessageId: string;
  messages: Message[];
  participants: string[];
  lastActivity: string;
}

interface Member {
  id: string;
  name: string;
  fourWordAddress: string;
  avatar?: string;
  role: 'owner' | 'admin' | 'member';
  status: 'online' | 'away' | 'busy' | 'offline';
  lastSeen: string;
}

interface EntityChatViewProps {
  entityId: string;
  entityType: 'group' | 'user' | 'channel' | 'project' | 'organization';
  entityName?: string;
  currentUserId: string;
  currentUserFourWords: string;
}

export const EntityChatView: React.FC<EntityChatViewProps> = ({
  entityId,
  entityType,
  entityName,
  currentUserId,
  currentUserFourWords,
}) => {
  const { queueMessage } = useEntityDirectory();
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('md'));
  const isTablet = useMediaQuery(theme.breakpoints.between('md', 'lg'));
  
  const [messages, setMessages] = useState<Message[]>([]);
  const [members, setMembers] = useState<Member[]>([]);
  const [loading, setLoading] = useState(true);
  const [sending, setSending] = useState(false);
  const [newMessage, setNewMessage] = useState('');
  const [replyingTo, setReplyingTo] = useState<Message | null>(null);
  const [activeThread, setActiveThread] = useState<Thread | null>(null);
  const [showMembers, setShowMembers] = useState(false);
  const [showAddMember, setShowAddMember] = useState(false);
  const [newMemberAddress, setNewMemberAddress] = useState('');
  const [newMemberRole, setNewMemberRole] = useState<'admin' | 'member'>('member');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedTab, setSelectedTab] = useState(0); // 0: Chat, 1: Members, 2: Files
  const [menuAnchor, setMenuAnchor] = useState<HTMLElement | null>(null);
  const [selectedMessage, setSelectedMessage] = useState<Message | null>(null);
  const [membersDrawerOpen, setMembersDrawerOpen] = useState(false);
  
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const messageInputRef = useRef<HTMLInputElement>(null);
  const dataRequestIdRef = useRef(0);

  const loadLocalMessages = useCallback(() => {
    const stored = loadStoredMessages(entityType, entityId);
    if (stored.length > 0) {
      setMessages(stored);
    }
  }, [entityId, entityType]);

  // Load entity data and messages
  useEffect(() => {
    const requestId = ++dataRequestIdRef.current;

    // Reset per-entity transient state before loading
    setReplyingTo(null);
    setActiveThread(null);
    setSelectedMessage(null);
    setMenuAnchor(null);
    setMembersDrawerOpen(false);
    setShowAddMember(false);
    setNewMemberAddress('');
    setNewMemberRole('member');
    setSearchQuery('');
    setNewMessage('');
    setSending(false);

    loadLocalMessages();
    loadEntityData(requestId);
  }, [entityId, entityType, loadLocalMessages]);

  useEffect(() => {
    const handler = (event: Event) => {
      const detail = (event as CustomEvent).detail;
      if (!detail) return;
      if (detail.entityId === entityId && detail.entityType === entityType) {
        loadLocalMessages();
      }
    };

    if (typeof window !== 'undefined') {
      window.addEventListener('messages:updated', handler as EventListener);
      return () => window.removeEventListener('messages:updated', handler as EventListener);
    }
    return () => undefined;
  }, [entityId, entityType, loadLocalMessages]);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const guardStateUpdate = (requestId: number, updater: () => void) => {
    if (dataRequestIdRef.current === requestId) {
      updater();
    }
  };

  const loadEntityData = async (requestId: number) => {
    guardStateUpdate(requestId, () => setLoading(true));
    try {
      // Always ensure we have something rendered
      guardStateUpdate(requestId, () => {
        const current = loadStoredMessages(entityType, entityId);
        if (current.length === 0) {
          setMessages(generateDemoMessages());
        }
      });

      if (entityType !== 'user') {
        const fallbackMembers = generateDemoMembers();
        guardStateUpdate(requestId, () => setMembers(fallbackMembers));
      } else {
        const fallbackMemberList: Member[] = [
          {
            id: entityId,
            name: entityName || 'Contact',
            fourWordAddress: 'contact-user-four-words',
            role: 'member',
            status: Math.random() > 0.5 ? 'online' : 'offline',
            lastSeen: new Date().toISOString(),
          },
        ];
        guardStateUpdate(requestId, () => setMembers(fallbackMemberList));
      }

      // Try to load real data, but don't block rendering if it fails
      try {
        const entityMessages = await loadMessages();
        guardStateUpdate(requestId, () => setMessages(entityMessages));

        if (entityType !== 'user') {
          const entityMembers = await loadMembers();
          guardStateUpdate(requestId, () => setMembers(entityMembers));
        }
      } catch (backendError) {
        console.log('Using fallback data - backend not available:', backendError);
        // Fallback data already set above
      }
    } catch (error) {
      console.error('Failed to load entity data:', error);
      guardStateUpdate(requestId, () => {
        setMessages(generateDemoMessages());
        setMembers(entityType !== 'user' ? generateDemoMembers() : []);
      });
    } finally {
      guardStateUpdate(requestId, () => setLoading(false));
    }
  };

  const loadMessages = async (): Promise<Message[]> => {
    try {
      // Check if backend service is available before calling
      if (!backendService || typeof backendService.getMessages !== 'function') {
        console.log('Backend service not available, using demo messages');
        return generateDemoMessages();
      }

      // Use environment-aware backend service
      const messages = await backendService.getMessages(entityType, entityId);
      
      // Transform backend message format to component format
      return messages.map(transformBackendMessage);
    } catch (error) {
      console.log('Backend service failed, using demo messages:', error);
      // Fallback to demo data if backend fails
      return generateDemoMessages();
    }
  };

  const loadMembers = async (): Promise<Member[]> => {
    try {
      // Check if backend service is available before calling
      if (!backendService || typeof backendService.getMembers !== 'function') {
        console.log('Backend service not available, using demo members');
        return generateDemoMembers();
      }

      // Use environment-aware backend service
      const members = await backendService.getMembers(entityType, entityId);
      
      // Transform backend member format to component format
      return members.map(transformBackendMember);
    } catch (error) {
      console.log('Backend service failed, using demo members:', error);
      // Fallback to demo data if backend fails  
      return generateDemoMembers();
    }
  };

  const generateDemoMessages = (): Message[] => {
    const demoMessages: Message[] = [
      {
        id: '1',
        sender: {
          id: 'user1',
          name: 'Alice Johnson',
          fourWordAddress: 'ocean-forest-moon-star',
        },
        content: `Hey team! Just wanted to check in about the project progress. How are things going?`,
        timestamp: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
        type: 'text',
        status: 'read',
        reactions: [{ emoji: '👍', users: ['user2', 'user3'] }],
        threadCount: 3,
      },
      {
        id: '2',
        sender: {
          id: 'user2',
          name: 'Bob Chen',
          fourWordAddress: 'mountain-river-cloud-wind',
        },
        content: `Great progress! I've finished the UI mockups. Want to take a look?`,
        timestamp: new Date(Date.now() - 1.5 * 60 * 60 * 1000).toISOString(),
        type: 'text',
        status: 'read',
        reactions: [],
        replyTo: {
          id: '1',
          sender: 'Alice Johnson',
          preview: 'Hey team! Just wanted to check...'
        }
      },
      {
        id: '3',
        sender: {
          id: currentUserId,
          name: 'You',
          fourWordAddress: currentUserFourWords,
        },
        content: `Awesome work everyone! The mockups look fantastic. Should we schedule a call to discuss next steps?`,
        timestamp: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
        type: 'text',
        status: 'read',
        reactions: [],
      },
    ];
    
    return demoMessages;
  };

  const generateDemoMembers = (): Member[] => {
    return [
      {
        id: 'user1',
        name: 'Alice Johnson',
        fourWordAddress: 'ocean-forest-moon-star',
        role: 'admin',
        status: 'online',
        lastSeen: new Date().toISOString(),
      },
      {
        id: 'user2',
        name: 'Bob Chen',
        fourWordAddress: 'mountain-river-cloud-wind',
        role: 'member',
        status: 'away',
        lastSeen: new Date(Date.now() - 15 * 60 * 1000).toISOString(),
      },
      {
        id: 'user3',
        name: 'Carol Davis',
        fourWordAddress: 'storm-ember-shadow-stone',
        role: 'member',
        status: 'offline',
        lastSeen: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
      },
    ];
  };

  const handleSendMessage = async () => {
    if (!newMessage.trim() || sending) return;

    setSending(true);
    const optimisticId = `temp-${Date.now()}`;
    try {
      const timestamp = new Date().toISOString();
      const message: Message = {
        id: optimisticId,
        sender: {
          id: currentUserId,
          name: 'You',
          fourWordAddress: currentUserFourWords,
        },
        content: newMessage.trim(),
        timestamp,
        type: 'text',
        status: 'sending',
        reactions: [],
        replyTo: replyingTo
          ? {
              id: replyingTo.id,
              sender: replyingTo.sender.name,
              preview: replyingTo.content.slice(0, 50) + '...',
            }
          : undefined,
      };

      setMessages(prev => [...prev, message]);
      setNewMessage('');
      setReplyingTo(null);
      upsertMessage(entityType, entityId, message);

      queueMessage({
        id: optimisticId,
        entityId,
        entityType: entityType === 'organization' ? 'group' : entityType,
        content: message.content,
        timestamp,
      });
    } catch (error) {
      console.error('Failed to send message:', error);
      // Update message status to failed
      setMessages(prev => prev.map(msg => 
        msg.id === optimisticId
          ? { ...msg, status: 'failed' as const }
          : msg
      ));
    } finally {
      setSending(false);
    }
  };

  const handleAddMember = async () => {
    if (!newMemberAddress.trim()) return;

    try {
      const isValid = await validateFourWordIdentity(newMemberAddress);
      if (!isValid) {
        alert('Invalid four-word address format');
        return;
      }

      // Check if member already exists
      const exists = members.some(m => m.fourWordAddress === newMemberAddress);
      if (exists) {
        alert('Member already in this ' + entityType);
        return;
      }

      // Call real backend API using environment-aware service
      const success = await backendService.addMember(entityType, entityId, newMemberAddress, newMemberRole);
      if (!success) {
        throw new Error('Backend add member failed');
      }

      // For demo, add to local state
      const newMember: Member = {
        id: `member-${Date.now()}`,
        name: `User ${newMemberAddress.split('-')[0]}`,
        fourWordAddress: newMemberAddress,
        role: newMemberRole,
        status: 'offline',
        lastSeen: new Date().toISOString(),
      };

      setMembers(prev => [...prev, newMember]);
      setNewMemberAddress('');
      setShowAddMember(false);

    } catch (error) {
      console.error('Failed to add member:', error);
      alert('Failed to add member. Please try again.');
    }
  };

  const handleRemoveMember = async (member: Member) => {
    try {
      // Call real backend API using environment-aware service
      const success = await backendService.removeMember(entityType, entityId, member.id, member.fourWordAddress);
      if (!success) {
        throw new Error('Backend remove member failed');
      }

      // For demo, remove from local state
      setMembers(prev => prev.filter(m => m.id !== member.id));
    } catch (error) {
      console.error('Failed to remove member:', error);
      alert('Failed to remove member. Please try again.');
    }
  };

  const handleStartAudioCall = async () => {
    try {
      await webRTCService.startAudioCall(entityId, entityType);
    } catch (error) {
      console.error('Failed to start audio call:', error);
      alert('Failed to start audio call. Please check your microphone permissions.');
    }
  };

  const handleStartVideoCall = async () => {
    try {
      await webRTCService.startVideoCall(entityId, entityType);
    } catch (error) {
      console.error('Failed to start video call:', error);
      alert('Failed to start video call. Please check your camera permissions.');
    }
  };

  const handleCreateThread = async (message: Message) => {
    try {
      // Create thread using environment-aware backend service
      const threadId = await backendService.createThread(message.id, entityType, entityId);
      
      // TODO: Open thread view UI
      console.log('Thread created:', threadId);
      setActiveThread({ 
        id: threadId, 
        parentMessageId: message.id,
        messages: [],
        participants: [],
        lastActivity: new Date().toISOString()
      });
    } catch (error) {
      console.error('Failed to create thread:', error);
      alert('Failed to create thread. Please try again.');
    }
  };

  const getEntityIcon = () => {
    switch (entityType) {
      case 'group':
        return <GroupIcon />;
      case 'user':
        return <PersonIcon />;
      case 'channel':
        return <ChannelIcon />;
      case 'project':
        return <ProjectIcon />;
      case 'organization':
        return <BusinessIcon />;
      default:
        return <GroupIcon />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'online':
        return 'success';
      case 'away':
        return 'warning';
      case 'busy':
        return 'error';
      default:
        return 'default';
    }
  };

  const formatTimeAgo = (timestamp: string) => {
    const now = new Date();
    const time = new Date(timestamp);
    const diffMs = now.getTime() - time.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'now';
    if (diffMins < 60) return `${diffMins}m`;
    if (diffHours < 24) return `${diffHours}h`;
    return `${diffDays}d`;
  };

  if (loading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '400px' }}>
        <CircularProgress />
      </Box>
    );
  }

  return (
    <Box sx={{ 
      height: '100vh', 
      display: 'flex', 
      flexDirection: isMobile ? 'column' : 'row',
      maxWidth: '100vw',
      overflow: 'hidden'
    }}>
      {/* Main Chat Area */}
      <Box sx={{ 
        flex: 1, 
        display: 'flex', 
        flexDirection: 'column',
        minWidth: 0, // Allow flex shrink
        height: '100%'
      }}>
        {/* Header */}
        <Paper elevation={2} sx={{ 
          p: isMobile ? 1.5 : 2, 
          borderRadius: 0,
          zIndex: theme.zIndex.appBar - 1
        }}>
          <Stack direction="row" alignItems="center" spacing={2}>
            <Avatar sx={{ bgcolor: 'primary.main', width: 40, height: 40 }}>
              {getEntityIcon()}
            </Avatar>
            <Box sx={{ flex: 1, minWidth: 0 }}>
              <Typography 
                variant={isMobile ? "subtitle1" : "h6"} 
                fontWeight={600}
                noWrap
              >
                {entityName || `${entityType} ${entityId}`}
              </Typography>
              <Typography variant="body2" color="text.secondary" noWrap>
                {entityType === 'user' ? 'Direct Message' : `${members.length} members`}
              </Typography>
            </Box>
            
            <Stack direction="row" spacing={0.5}>
              {/* Members Button - Always visible */}
              {entityType !== 'user' && (
                <Tooltip title="View Members">
                  <IconButton 
                    onClick={() => isMobile ? setMembersDrawerOpen(true) : setSelectedTab(1)}
                    color={selectedTab === 1 && !isMobile ? "primary" : "default"}
                  >
                    <Badge badgeContent={members.length} color="primary" max={99}>
                      <GroupIcon />
                    </Badge>
                  </IconButton>
                </Tooltip>
              )}
              
              <Tooltip title="Audio Call">
                <IconButton onClick={handleStartAudioCall} size={isMobile ? "small" : "medium"}>
                  <PhoneIcon />
                </IconButton>
              </Tooltip>
              <Tooltip title="Video Call">
                <IconButton onClick={handleStartVideoCall} size={isMobile ? "small" : "medium"}>
                  <VideocamIcon />
                </IconButton>
              </Tooltip>
              {!isMobile && (
                <>
                  <Tooltip title="Search">
                    <IconButton size="small">
                      <SearchIcon />
                    </IconButton>
                  </Tooltip>
                  <Tooltip title="More Options">
                    <IconButton size="small">
                      <MoreVertIcon />
                    </IconButton>
                  </Tooltip>
                </>
              )}
            </Stack>
          </Stack>

          {/* Desktop Tabs - Only show on desktop */}
          {!isMobile && (
            <Tabs value={selectedTab} onChange={(e, v) => setSelectedTab(v)} sx={{ mt: 1 }}>
              <Tab label="Chat" />
              {entityType !== 'user' && <Tab label={`Members (${members.length})`} />}
              <Tab label="Files" />
            </Tabs>
          )}
        </Paper>

        {/* Chat Content - Always visible on mobile, tab-based on desktop */}
        {(isMobile || selectedTab === 0) && (
          <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
            {/* Messages */}
            <Box sx={{ 
              flex: 1, 
              overflow: 'auto', 
              p: isMobile ? 1 : 2,
              maxHeight: isMobile ? 'calc(100vh - 180px)' : 'calc(100vh - 220px)'
            }}>
              <Stack spacing={1}>
                {messages.map((message) => (
                  <motion.div
                    key={message.id}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.2 }}
                  >
                    <Card variant="outlined" sx={{ p: 2 }}>
                      <Stack direction="row" spacing={2}>
                        <Avatar sx={{ width: 32, height: 32 }}>
                          {message.sender.name[0]}
                        </Avatar>
                        <Box sx={{ flex: 1 }}>
                          <Stack direction="row" alignItems="center" spacing={1} sx={{ mb: 0.5 }}>
                            <Typography variant="subtitle2" fontWeight={600}>
                              {message.sender.name}
                            </Typography>
                            <Typography variant="caption" color="text.secondary">
                              {message.sender.fourWordAddress}
                            </Typography>
                            <Typography variant="caption" color="text.secondary">
                              {formatTimeAgo(message.timestamp)}
                            </Typography>
                            {message.status === 'sending' && <CircularProgress size={12} />}
                            {message.status === 'failed' && <ErrorIcon color="error" sx={{ fontSize: 16 }} />}
                          </Stack>

                          {message.replyTo && (
                            <Box sx={{ p: 1, bgcolor: 'action.hover', borderRadius: 1, mb: 1 }}>
                              <Typography variant="caption" color="primary">
                                Replying to {message.replyTo.sender}
                              </Typography>
                              <Typography variant="body2" sx={{ opacity: 0.7 }}>
                                {message.replyTo.preview}
                              </Typography>
                            </Box>
                          )}

                          <Typography variant="body1" sx={{ mb: 1 }}>
                            {message.content}
                          </Typography>

                          <Stack direction="row" alignItems="center" spacing={1}>
                            {message.reactions.map((reaction, idx) => (
                              <Chip
                                key={idx}
                                label={`${reaction.emoji} ${reaction.users.length}`}
                                size="small"
                                variant="outlined"
                                sx={{ height: 24 }}
                              />
                            ))}
                            <IconButton size="small" onClick={() => setReplyingTo(message)}>
                              <ReplyIcon fontSize="small" />
                            </IconButton>
                            {message.threadCount && (
                              <Button
                                size="small"
                                startIcon={<ThreadIcon />}
                                onClick={() => handleCreateThread(message)}
                              >
                                {message.threadCount} replies
                              </Button>
                            )}
                          </Stack>
                        </Box>
                      </Stack>
                    </Card>
                  </motion.div>
                ))}
                <div ref={messagesEndRef} />
              </Stack>
            </Box>

            {/* Reply Preview */}
            <AnimatePresence>
              {replyingTo && (
                <Slide direction="up" in={true}>
                  <Paper elevation={2} sx={{ p: 2, m: 1, borderRadius: 2 }}>
                    <Stack direction="row" alignItems="center" spacing={2}>
                      <Box sx={{ flex: 1 }}>
                        <Typography variant="caption" color="primary">
                          Replying to {replyingTo.sender.name}
                        </Typography>
                        <Typography variant="body2" sx={{ opacity: 0.7 }}>
                          {replyingTo.content.slice(0, 100)}...
                        </Typography>
                      </Box>
                      <IconButton size="small" onClick={() => setReplyingTo(null)}>
                        <CloseIcon />
                      </IconButton>
                    </Stack>
                  </Paper>
                </Slide>
              )}
            </AnimatePresence>

            {/* Message Input */}
            <Box sx={{ 
              p: isMobile ? 1.5 : 2, 
              bgcolor: 'background.paper', 
              borderTop: 1, 
              borderColor: 'divider',
              position: 'sticky',
              bottom: 0,
              zIndex: 1
            }}>
              <Stack direction="row" spacing={1} alignItems="flex-end">
                <IconButton>
                  <AttachFileIcon />
                </IconButton>
                <TextField
                  ref={messageInputRef}
                  fullWidth
                  multiline
                  maxRows={4}
                  placeholder={`Message ${entityName || entityType}...`}
                  value={newMessage}
                  onChange={(e) => setNewMessage(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && !e.shiftKey) {
                      e.preventDefault();
                      handleSendMessage();
                    }
                  }}
                  variant="outlined"
                  size="small"
                />
                <IconButton>
                  <EmojiIcon />
                </IconButton>
                <IconButton 
                  color="primary" 
                  onClick={handleSendMessage}
                  disabled={!newMessage.trim() || sending}
                >
                  <SendIcon />
                </IconButton>
              </Stack>
            </Box>
          </Box>
        )}

        {/* Desktop Members Panel */}
        {!isMobile && selectedTab === 1 && entityType !== 'user' && (
          <Box sx={{ p: 2, height: '100%', overflow: 'auto' }}>
            <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ mb: 2 }}>
              <Typography variant="h6">Members ({members.length})</Typography>
              <Button
                variant="outlined"
                startIcon={<AddIcon />}
                onClick={() => setShowAddMember(true)}
              >
                Add Member
              </Button>
            </Stack>

            <List>
              {members.map((member) => (
                <ListItem key={member.id} disablePadding>
                  <ListItemButton sx={{ borderRadius: 1, mb: 0.5 }}>
                    <ListItemAvatar>
                      <Badge
                        variant="dot"
                        color={getStatusColor(member.status) as any}
                        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                      >
                        <Avatar sx={{ width: 40, height: 40 }}>{member.name[0]}</Avatar>
                      </Badge>
                    </ListItemAvatar>
                    <ListItemText
                      primary={
                        <Typography variant="subtitle2" fontWeight={600}>
                          {member.name}
                        </Typography>
                      }
                      secondary={
                        <Stack spacing={0.5}>
                          <Typography variant="caption" color="text.secondary">
                            {member.fourWordAddress}
                          </Typography>
                          <Stack direction="row" alignItems="center" spacing={1}>
                            <Chip 
                              label={member.role} 
                              size="small" 
                              variant={member.role === 'admin' ? 'filled' : 'outlined'}
                              color={member.role === 'admin' ? 'primary' : 'default'}
                            />
                            <Typography variant="caption" color="text.secondary">
                              {member.status === 'online' ? '🟢 Online' : `Last seen ${formatTimeAgo(member.lastSeen)}`}
                            </Typography>
                          </Stack>
                        </Stack>
                      }
                    />
                    {member.id !== currentUserId && (
                      <Tooltip title="Remove Member">
                        <IconButton
                          edge="end"
                          size="small"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleRemoveMember(member);
                          }}
                          color="error"
                        >
                          <RemoveIcon />
                        </IconButton>
                      </Tooltip>
                    )}
                  </ListItemButton>
                </ListItem>
              ))}
            </List>
          </Box>
        )}

        {/* Desktop Files Panel */}
        {!isMobile && selectedTab === 2 && (
          <Box sx={{ p: 2 }}>
            <Typography variant="h6" gutterBottom>Files</Typography>
            <Typography color="text.secondary">File sharing coming soon...</Typography>
          </Box>
        )}
      </Box>


      {/* Mobile Members Drawer */}
      {isMobile && entityType !== 'user' && (
        <SwipeableDrawer
          anchor="right"
          open={membersDrawerOpen}
          onClose={() => setMembersDrawerOpen(false)}
          onOpen={() => setMembersDrawerOpen(true)}
          PaperProps={{
            sx: { 
              width: '85vw',
              maxWidth: 400
            }
          }}
        >
          <Box sx={{ p: 2 }}>
            <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ mb: 2 }}>
              <Typography variant="h6">Members ({members.length})</Typography>
              <IconButton onClick={() => setMembersDrawerOpen(false)}>
                <CloseIcon />
              </IconButton>
            </Stack>
            
            <Button
              variant="outlined"
              fullWidth
              startIcon={<AddIcon />}
              onClick={() => setShowAddMember(true)}
              sx={{ mb: 2 }}
            >
              Add Member
            </Button>

            <List dense>
              {members.map((member) => (
                <ListItem key={member.id} disablePadding>
                  <ListItemButton sx={{ borderRadius: 1, mb: 0.5 }}>
                    <ListItemAvatar>
                      <Badge
                        variant="dot"
                        color={getStatusColor(member.status) as any}
                        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                      >
                        <Avatar sx={{ width: 36, height: 36 }}>{member.name[0]}</Avatar>
                      </Badge>
                    </ListItemAvatar>
                    <ListItemText
                      primary={
                        <Typography variant="subtitle2" fontWeight={600}>
                          {member.name}
                        </Typography>
                      }
                      secondary={
                        <Stack spacing={0.5}>
                          <Typography variant="caption" color="text.secondary" noWrap>
                            {member.fourWordAddress}
                          </Typography>
                          <Stack direction="row" alignItems="center" spacing={1}>
                            <Chip 
                              label={member.role} 
                              size="small" 
                              variant={member.role === 'admin' ? 'filled' : 'outlined'}
                              color={member.role === 'admin' ? 'primary' : 'default'}
                            />
                            <Typography variant="caption" color="text.secondary">
                              {member.status === 'online' ? '🟢' : '⚫'}
                            </Typography>
                          </Stack>
                        </Stack>
                      }
                    />
                    {member.id !== currentUserId && (
                      <Tooltip title="Remove Member">
                        <IconButton
                          edge="end"
                          size="small"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleRemoveMember(member);
                          }}
                          color="error"
                        >
                          <RemoveIcon />
                        </IconButton>
                      </Tooltip>
                    )}
                  </ListItemButton>
                </ListItem>
              ))}
            </List>
          </Box>
        </SwipeableDrawer>
      )}
      {/* Add Member Dialog */}
      <Dialog open={showAddMember} onClose={() => setShowAddMember(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Add Member to {entityType}</DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ mt: 1 }}>
            <TextField
              fullWidth
              label="Four-Word Address"
              placeholder="ocean-forest-moon-star"
              value={newMemberAddress}
              onChange={(e) => setNewMemberAddress(e.target.value.toLowerCase())}
              helperText="Enter the person's four-word identity"
            />
            <TextField
              select
              fullWidth
              label="Role"
              value={newMemberRole}
              onChange={(e) => setNewMemberRole(e.target.value as 'admin' | 'member')}
              SelectProps={{ native: true }}
            >
              <option value="member">Member</option>
              <option value="admin">Admin</option>
            </TextField>
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShowAddMember(false)}>Cancel</Button>
          <Button
            variant="contained"
            onClick={handleAddMember}
            disabled={!newMemberAddress.trim()}
          >
            Add Member
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default EntityChatView;
