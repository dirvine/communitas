import React, { useState, useEffect, useRef, useCallback } from 'react';
import {
  Box,
  Paper,
  Typography,
  IconButton,
  Avatar,
  Badge,
  Chip,
  Button,
  Fab,
  Tooltip,
  Stack,
  Divider,
  LinearProgress,
  alpha,
  useTheme,
  useMediaQuery,
} from '@mui/material';
import {
  Chat as ChatIcon,
  VideoCall as VideoIcon,
  Call as VoiceIcon,
  CallEnd as CallIcon,
  ScreenShare as ScreenShareIcon,
  Folder as StorageIcon,
  Web as WebDriveIcon,
  People as MembersIcon,
  Settings as SettingsIcon,
  Close as CloseIcon,
  Mic as MicIcon,
  MicOff as MicOffIcon,
  Videocam as VideocamIcon,
  VideocamOff as VideocamOffIcon,
  VolumeUp as VolumeUpIcon,
  VolumeOff as VolumeOffIcon,
  MoreVert as MoreIcon,
  Send as SendIcon,
  AttachFile as AttachFileIcon,
  Image as ImageIcon,
  Description as DocumentIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { Element as ElementType, ElementContext } from '../../types/element';
import { ChatInterface } from '../chat/ChatInterface';
import { FileManager } from '../storage/FileManager';
import { WebRTCService } from '../../services/webrtc/WebRTCService';
import { ElementStorageService } from '../../services/element/ElementStorageService';
import { ElementCommunicationService } from '../../services/element/ElementCommunicationService';

const MotionPaper = motion(Paper);
const MotionBox = motion(Box);

interface ElementProps {
  element: ElementType;
  currentUserId: string;
  onClose?: () => void;
  onMinimize?: () => void;
  initialView?: 'chat' | 'files' | 'webdrive' | 'members' | 'settings';
}

interface CallControlsProps {
  isInCall: boolean;
  isAudioEnabled: boolean;
  isVideoEnabled: boolean;
  isScreenSharing: boolean;
  onToggleAudio: () => void;
  onToggleVideo: () => void;
  onToggleScreenShare: () => void;
  onEndCall: () => void;
}

const CallControls: React.FC<CallControlsProps> = ({
  isInCall,
  isAudioEnabled,
  isVideoEnabled,
  isScreenSharing,
  onToggleAudio,
  onToggleVideo,
  onToggleScreenShare,
  onEndCall,
}) => {
  if (!isInCall) return null;

  return (
    <MotionBox
      initial={{ y: 100, opacity: 0 }}
      animate={{ y: 0, opacity: 1 }}
      exit={{ y: 100, opacity: 0 }}
      sx={{
        position: 'fixed',
        bottom: 20,
        left: '50%',
        transform: 'translateX(-50%)',
        zIndex: 1300,
      }}
    >
      <Paper
        elevation={8}
        sx={{
          p: 2,
          borderRadius: 4,
          background: (theme) => alpha(theme.palette.background.paper, 0.95),
          backdropFilter: 'blur(20px)',
        }}
      >
        <Stack direction="row" spacing={2} alignItems="center">
          <Tooltip title={isAudioEnabled ? 'Mute' : 'Unmute'}>
            <IconButton
              onClick={onToggleAudio}
              sx={{
                backgroundColor: isAudioEnabled ? 'success.main' : 'error.main',
                color: 'white',
                '&:hover': {
                  backgroundColor: isAudioEnabled ? 'success.dark' : 'error.dark',
                },
              }}
            >
              {isAudioEnabled ? <MicIcon /> : <MicOffIcon />}
            </IconButton>
          </Tooltip>

          <Tooltip title={isVideoEnabled ? 'Turn off camera' : 'Turn on camera'}>
            <IconButton
              onClick={onToggleVideo}
              sx={{
                backgroundColor: isVideoEnabled ? 'success.main' : 'grey.500',
                color: 'white',
                '&:hover': {
                  backgroundColor: isVideoEnabled ? 'success.dark' : 'grey.700',
                },
              }}
            >
              {isVideoEnabled ? <VideocamIcon /> : <VideocamOffIcon />}
            </IconButton>
          </Tooltip>

          <Tooltip title={isScreenSharing ? 'Stop sharing' : 'Share screen'}>
            <IconButton
              onClick={onToggleScreenShare}
              sx={{
                backgroundColor: isScreenSharing ? 'warning.main' : 'grey.500',
                color: 'white',
                '&:hover': {
                  backgroundColor: isScreenSharing ? 'warning.dark' : 'grey.700',
                },
              }}
            >
              <ScreenShareIcon />
            </IconButton>
          </Tooltip>

          <Divider orientation="vertical" flexItem />

          <Tooltip title="End call">
            <IconButton
              onClick={onEndCall}
              sx={{
                backgroundColor: 'error.main',
                color: 'white',
                '&:hover': {
                  backgroundColor: 'error.dark',
                },
              }}
            >
              <CallIcon />
            </IconButton>
          </Tooltip>
        </Stack>
      </Paper>
    </MotionBox>
  );
};

export const Element: React.FC<ElementProps> = ({
  element,
  currentUserId,
  onClose,
  onMinimize,
  initialView = 'chat',
}) => {
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('md'));

  const [activeView, setActiveView] = useState<'chat' | 'files' | 'webdrive' | 'members' | 'settings'>(initialView);
  const [isInCall, setIsInCall] = useState(false);
  const [callType, setCallType] = useState<'voice' | 'video' | 'screen-share' | null>(null);
  const [isAudioEnabled, setIsAudioEnabled] = useState(true);
  const [isVideoEnabled, setIsVideoEnabled] = useState(false);
  const [isScreenSharing, setIsScreenSharing] = useState(false);
  const [unreadCount, setUnreadCount] = useState(element.communication.unreadCount);
  const [typingUsers, setTypingUsers] = useState<string[]>([]);

  const webRTCService = useRef<WebRTCService | null>(null);
  const storageService = useRef<ElementStorageService | null>(null);
  const communicationService = useRef<ElementCommunicationService | null>(null);

  // Initialize services
  useEffect(() => {
    storageService.current = new ElementStorageService(element);
    communicationService.current = new ElementCommunicationService(element, currentUserId);

    // Set up event listeners
    const handleMessageReceived = () => {
      setUnreadCount(prev => prev + 1);
    };

    const handleUserTyping = (userId: string) => {
      setTypingUsers(prev => [...new Set([...prev, userId])]);
      setTimeout(() => {
        setTypingUsers(prev => prev.filter(id => id !== userId));
      }, 3000);
    };

    // Subscribe to events
    communicationService.current.on('message-received', handleMessageReceived);
    communicationService.current.on('user-typing', handleUserTyping);

    return () => {
      storageService.current?.cleanup();
      communicationService.current?.cleanup();
    };
  }, [element, currentUserId]);

  // Get user role in this element
  const userMembership = element.membership.find(m => m.userId === currentUserId);
  const userRole = userMembership?.role || 'guest';
  const userPermissions = userMembership?.permissions || [];

  const elementContext: ElementContext = {
    currentElement: element,
    userRole,
    userPermissions,
    isOnline: true, // TODO: Implement presence detection
    activeUsers: element.membership.filter(m => m.isActive).map(m => m.userId),
    recentActivity: [], // TODO: Implement activity tracking
  };

  const handleStartCall = useCallback(async (type: 'voice' | 'video' | 'screen-share') => {
    try {
      setCallType(type);
      setIsInCall(true);
      setIsVideoEnabled(type === 'video');
      setIsScreenSharing(type === 'screen-share');

      // TODO: Integrate with actual WebRTC service
      console.log(`Starting ${type} call for element ${element.identity.id}`);
    } catch (error) {
      console.error('Failed to start call:', error);
      setIsInCall(false);
    }
  }, [element]);

  const handleEndCall = useCallback(async () => {
    try {
      setIsInCall(false);
      setCallType(null);
      setIsVideoEnabled(false);
      setIsScreenSharing(false);

      // TODO: Integrate with actual WebRTC service
      console.log('Ending call');
    } catch (error) {
      console.error('Failed to end call:', error);
    }
  }, []);

  const handleToggleAudio = useCallback(() => {
    setIsAudioEnabled(prev => !prev);
    // TODO: Integrate with WebRTC service
  }, []);

  const handleToggleVideo = useCallback(() => {
    setIsVideoEnabled(prev => !prev);
    // TODO: Integrate with WebRTC service
  }, []);

  const handleToggleScreenShare = useCallback(() => {
    setIsScreenSharing(prev => !prev);
    // TODO: Integrate with WebRTC service
  }, []);

  const handleSendMessage = useCallback(async (content: string, attachments?: File[]) => {
    if (!communicationService.current) return;

    try {
      await communicationService.current.sendMessage(content, attachments);
      setUnreadCount(0); // Reset unread count when sending
    } catch (error) {
      console.error('Failed to send message:', error);
    }
  }, []);

  const handleFileUpload = useCallback(async (files: File[]) => {
    if (!storageService.current) return;

    try {
      for (const file of files) {
        await storageService.current.uploadFile(file);
      }
    } catch (error) {
      console.error('Failed to upload files:', error);
    }
  }, []);

  const renderHeader = () => (
    <Box
      sx={{
        p: 2,
        borderBottom: `1px solid ${alpha(theme.palette.divider, 0.1)}`,
        background: alpha(theme.palette.background.paper, 0.8),
        backdropFilter: 'blur(10px)',
      }}
    >
      <Stack direction="row" alignItems="center" spacing={2}>
        <Badge
          variant="dot"
          color="success"
          invisible={!elementContext.isOnline}
        >
          <Avatar
            sx={{
              width: 40,
              height: 40,
              background: element.identity.type.startsWith('personal')
                ? 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)'
                : 'linear-gradient(135deg, #f093fb 0%, #f5576c 100%)',
            }}
          >
            {element.identity.name[0]}
          </Avatar>
        </Badge>

        <Box sx={{ flex: 1, minWidth: 0 }}>
          <Typography variant="h6" fontWeight={600} noWrap>
            {element.identity.name}
          </Typography>
          <Stack direction="row" spacing={1} alignItems="center">
            <Typography variant="body2" color="text.secondary">
              {element.identity.fourWords}
            </Typography>
            {element.communication.activeCall && (
              <Chip
                size="small"
                label={`${element.communication.activeCall.type} call`}
                color="primary"
                variant="outlined"
              />
            )}
            {typingUsers.length > 0 && (
              <Typography variant="caption" color="primary" sx={{ fontStyle: 'italic' }}>
                {typingUsers.length === 1
                  ? `${typingUsers[0]} is typing...`
                  : `${typingUsers.length} people typing...`
                }
              </Typography>
            )}
          </Stack>
        </Box>

        <Stack direction="row" spacing={1}>
          {/* Quick Actions */}
          {element.capabilities.voice && (
            <Tooltip title="Voice Call">
              <IconButton
                size="small"
                onClick={() => handleStartCall('voice')}
                disabled={isInCall}
                sx={{
                  color: 'primary.main',
                  '&:hover': { backgroundColor: alpha('#2196F3', 0.1) },
                }}
              >
                <VoiceIcon fontSize="small" />
              </IconButton>
            </Tooltip>
          )}

          {element.capabilities.video && (
            <Tooltip title="Video Call">
              <IconButton
                size="small"
                onClick={() => handleStartCall('video')}
                disabled={isInCall}
                sx={{
                  color: 'primary.main',
                  '&:hover': { backgroundColor: alpha('#4CAF50', 0.1) },
                }}
              >
                <VideoIcon fontSize="small" />
              </IconButton>
            </Tooltip>
          )}

          {element.capabilities.screenShare && (
            <Tooltip title="Screen Share">
              <IconButton
                size="small"
                onClick={() => handleStartCall('screen-share')}
                disabled={isInCall}
                sx={{
                  color: 'primary.main',
                  '&:hover': { backgroundColor: alpha('#FF9800', 0.1) },
                }}
              >
                <ScreenShareIcon fontSize="small" />
              </IconButton>
            </Tooltip>
          )}

          <Tooltip title="More options">
            <IconButton size="small">
              <MoreIcon fontSize="small" />
            </IconButton>
          </Tooltip>

          {onMinimize && (
            <Tooltip title="Minimize">
              <IconButton size="small" onClick={onMinimize}>
                <Typography variant="body2">−</Typography>
              </IconButton>
            </Tooltip>
          )}

          {onClose && (
            <Tooltip title="Close">
              <IconButton size="small" onClick={onClose}>
                <CloseIcon fontSize="small" />
              </IconButton>
            </Tooltip>
          )}
        </Stack>
      </Stack>

      {/* Tab Navigation */}
      <Stack direction="row" spacing={0} sx={{ mt: 2 }}>
        {element.capabilities.text && (
          <Button
            variant={activeView === 'chat' ? 'contained' : 'text'}
            size="small"
            startIcon={<ChatIcon />}
            onClick={() => setActiveView('chat')}
            sx={{ minWidth: 'auto', px: 2 }}
          >
            Chat
            {unreadCount > 0 && (
              <Badge
                badgeContent={unreadCount}
                color="error"
                sx={{ ml: 1 }}
              />
            )}
          </Button>
        )}

        {element.capabilities.storage && (
          <Button
            variant={activeView === 'files' ? 'contained' : 'text'}
            size="small"
            startIcon={<StorageIcon />}
            onClick={() => setActiveView('files')}
            sx={{ minWidth: 'auto', px: 2 }}
          >
            Files
          </Button>
        )}

        {element.capabilities.webDrive && (
          <Button
            variant={activeView === 'webdrive' ? 'contained' : 'text'}
            size="small"
            startIcon={<WebDriveIcon />}
            onClick={() => setActiveView('webdrive')}
            sx={{ minWidth: 'auto', px: 2 }}
          >
            Web Drive
          </Button>
        )}

        <Button
          variant={activeView === 'members' ? 'contained' : 'text'}
          size="small"
          startIcon={<MembersIcon />}
          onClick={() => setActiveView('members')}
          sx={{ minWidth: 'auto', px: 2 }}
        >
          Members ({element.membership.filter(m => m.isActive).length})
        </Button>

        {userPermissions.includes('manage_settings') && (
          <Button
            variant={activeView === 'settings' ? 'contained' : 'text'}
            size="small"
            startIcon={<SettingsIcon />}
            onClick={() => setActiveView('settings')}
            sx={{ minWidth: 'auto', px: 2 }}
          >
            Settings
          </Button>
        )}
      </Stack>
    </Box>
  );

  const renderContent = () => {
    switch (activeView) {
      case 'chat':
        return element.capabilities.text ? (
          <ChatInterface
            chatId={element.identity.id}
            chatName={element.identity.name}
            chatType={element.identity.type.includes('group') || element.identity.type.includes('channel') ? 'group' : 'direct'}
            participants={element.membership.filter(m => m.isActive).length}
            onSendMessage={handleSendMessage}
          />
        ) : (
          <Box sx={{ p: 4, textAlign: 'center' }}>
            <Typography color="text.secondary">
              Text messaging is not enabled for this element.
            </Typography>
          </Box>
        );

      case 'files':
        return element.capabilities.storage ? (
          <FileManager
            elementId={element.identity.id}
            currentUserId={currentUserId}
            onFileUpload={handleFileUpload}
            capabilities={element.capabilities}
          />
        ) : (
          <Box sx={{ p: 4, textAlign: 'center' }}>
            <Typography color="text.secondary">
              File storage is not enabled for this element.
            </Typography>
          </Box>
        );

      case 'webdrive':
        return element.capabilities.webDrive ? (
          <Box sx={{ p: 4 }}>
            <Typography variant="h6" gutterBottom>
              Web Drive
            </Typography>
            <Typography color="text.secondary" sx={{ mb: 2 }}>
              Access your files via: {element.storage.webDriveAddress}
            </Typography>
            {/* TODO: Implement web drive interface */}
            <Typography color="text.secondary">
              Web drive interface coming soon...
            </Typography>
          </Box>
        ) : (
          <Box sx={{ p: 4, textAlign: 'center' }}>
            <Typography color="text.secondary">
              Web drive is not enabled for this element.
            </Typography>
          </Box>
        );

      case 'members':
        return (
          <Box sx={{ p: 2 }}>
            <Typography variant="h6" gutterBottom>
              Members
            </Typography>
            <Stack spacing={1}>
              {element.membership
                .filter(m => m.isActive)
                .map(member => (
                  <Stack key={member.userId} direction="row" alignItems="center" spacing={2}>
                    <Avatar sx={{ width: 32, height: 32 }}>
                      {member.userId[0]}
                    </Avatar>
                    <Box sx={{ flex: 1 }}>
                      <Typography variant="body2" fontWeight={500}>
                        {member.userId === currentUserId ? 'You' : member.userId}
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        {member.role}
                      </Typography>
                    </Box>
                    <Chip label={member.role} size="small" variant="outlined" />
                  </Stack>
                ))}
            </Stack>
          </Box>
        );

      case 'settings':
        return userPermissions.includes('manage_settings') ? (
          <Box sx={{ p: 4 }}>
            <Typography variant="h6" gutterBottom>
              Element Settings
            </Typography>
            {/* TODO: Implement settings interface */}
            <Typography color="text.secondary">
              Settings interface coming soon...
            </Typography>
          </Box>
        ) : (
          <Box sx={{ p: 4, textAlign: 'center' }}>
            <Typography color="text.secondary">
              You don't have permission to access settings.
            </Typography>
          </Box>
        );

      default:
        return null;
    }
  };

  return (
    <MotionPaper
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.95 }}
      transition={{ duration: 0.2 }}
      sx={{
        width: isMobile ? '100%' : 800,
        height: isMobile ? '100%' : 600,
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
        borderRadius: isMobile ? 0 : 2,
      }}
      elevation={8}
    >
      {renderHeader()}
      <Box sx={{ flex: 1, overflow: 'auto' }}>
        {renderContent()}
      </Box>

      {/* Call Controls */}
      <AnimatePresence>
        {isInCall && (
          <CallControls
            isInCall={isInCall}
            isAudioEnabled={isAudioEnabled}
            isVideoEnabled={isVideoEnabled}
            isScreenSharing={isScreenSharing}
            onToggleAudio={handleToggleAudio}
            onToggleVideo={handleToggleVideo}
            onToggleScreenShare={handleToggleScreenShare}
            onEndCall={handleEndCall}
          />
        )}
      </AnimatePresence>
    </MotionPaper>
  );
};