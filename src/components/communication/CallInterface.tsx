import React, { useState, useEffect, useRef } from 'react';
import {
  Box,
  Paper,
  Typography,
  IconButton,
  Avatar,
  AvatarGroup,
  Stack,
  Chip,
  Button,
  Slider,
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  Divider,
  Card,
  CardContent,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  List,
  ListItem,
  ListItemButton,
  ListItemAvatar,
  Tooltip,
  Badge,
  Fab,
  Collapse,
  Alert,
  Grid,
} from '@mui/material';
import {
  Call as CallIcon,
  CallEnd as CallEndIcon,
  Mic as MicIcon,
  MicOff as MicOffIcon,
  Videocam as VideocamIcon,
  VideocamOff as VideocamOffIcon,
  ScreenShare as ScreenShareIcon,
  StopScreenShare as StopScreenShareIcon,
  VolumeUp as VolumeUpIcon,
  VolumeOff as VolumeOffIcon,
  Settings as SettingsIcon,
  Chat as ChatIcon,
  People as PeopleIcon,
  PersonAdd as PersonAddIcon,
  Stop as StopIcon,
  Fullscreen as FullscreenIcon,
  FullscreenExit as FullscreenExitIcon,
  PictureInPicture as PipIcon,
  MoreVert as MoreVertIcon,
  Send as SendIcon,
  EmojiEmotions as EmojiIcon,
  AttachFile as AttachFileIcon,
  Phone as PhoneIcon,
  VideoCall as VideoCallIcon,
  Close as CloseIcon,
  Minimize as MinimizeIcon,
  OpenInFull as MaximizeIcon,
  DragHandle as DragHandleIcon,
  FiberManualRecord as FiberManualRecordIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { Member } from '../../types/organization';

interface CallParticipant extends Member {
  is_muted: boolean;
  is_video_enabled: boolean;
  is_screen_sharing: boolean;
  connection_quality: 'excellent' | 'good' | 'fair' | 'poor';
  join_time: string;
  is_speaking: boolean;
}

interface ChatMessage {
  id: string;
  sender: Member;
  content: string;
  timestamp: string;
  type: 'text' | 'file' | 'emoji' | 'system';
  replied_to?: string;
}

interface CallInterfaceProps {
  callType: 'voice' | 'video';
  participants: CallParticipant[];
  currentUser: Member;
  onEndCall: () => void;
  onToggleMute: () => void;
  onToggleVideo: () => void;
  onToggleScreenShare: () => void;
  onInviteParticipant: (userId: string) => void;
  isMinimized?: boolean;
  onMinimize?: () => void;
  onMaximize?: () => void;
  entityName?: string; // Organization/Project/Group name
  entityType?: 'organization' | 'project' | 'group';
}

export const CallInterface: React.FC<CallInterfaceProps> = ({
  callType,
  participants,
  currentUser,
  onEndCall,
  onToggleMute,
  onToggleVideo,
  onToggleScreenShare,
  onInviteParticipant,
  isMinimized = false,
  onMinimize,
  onMaximize,
  entityName,
  entityType,
}) => {
  const [isMuted, setIsMuted] = useState(false);
  const [isVideoEnabled, setIsVideoEnabled] = useState(callType === 'video');
  const [isScreenSharing, setIsScreenSharing] = useState(false);
  const [volume, setVolume] = useState(50);
  const [isRecording, setIsRecording] = useState(false);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [isPipMode, setIsPipMode] = useState(false);
  const [showChat, setShowChat] = useState(false);
  const [showParticipants, setShowParticipants] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [newMessage, setNewMessage] = useState('');
  const [callDuration, setCallDuration] = useState(0);
  const [menuAnchor, setMenuAnchor] = useState<null | HTMLElement>(null);
  const [inviteDialogOpen, setInviteDialogOpen] = useState(false);

  const videoRef = useRef<HTMLVideoElement>(null);
  const chatScrollRef = useRef<HTMLDivElement>(null);

  // Call timer
  useEffect(() => {
    const timer = setInterval(() => {
      setCallDuration(prev => prev + 1);
    }, 1000);

    return () => clearInterval(timer);
  }, []);

  // Auto-scroll chat
  useEffect(() => {
    if (chatScrollRef.current) {
      chatScrollRef.current.scrollTop = chatScrollRef.current.scrollHeight;
    }
  }, [chatMessages]);

  const formatDuration = (seconds: number) => {
    const hrs = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    if (hrs > 0) {
      return `${hrs}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const handleToggleMute = () => {
    setIsMuted(!isMuted);
    onToggleMute();
  };

  const handleToggleVideo = () => {
    setIsVideoEnabled(!isVideoEnabled);
    onToggleVideo();
  };

  const handleToggleScreenShare = () => {
    setIsScreenSharing(!isScreenSharing);
    onToggleScreenShare();
  };

  const handleSendMessage = () => {
    if (newMessage.trim()) {
      const message: ChatMessage = {
        id: Date.now().toString(),
        sender: currentUser,
        content: newMessage,
        timestamp: new Date().toISOString(),
        type: 'text',
      };
      setChatMessages(prev => [...prev, message]);
      setNewMessage('');
    }
  };

  const handleMenuOpen = (event: React.MouseEvent<HTMLElement>) => {
    setMenuAnchor(event.currentTarget);
  };

  const handleMenuClose = () => {
    setMenuAnchor(null);
  };

  const getConnectionQualityColor = (quality: string) => {
    switch (quality) {
      case 'excellent': return 'success';
      case 'good': return 'info';
      case 'fair': return 'warning';
      case 'poor': return 'error';
      default: return 'default';
    }
  };

  // Minimized view
  if (isMinimized) {
    return (
      <motion.div
        initial={{ scale: 0, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        exit={{ scale: 0, opacity: 0 }}
        style={{ position: 'fixed', bottom: 20, right: 20, zIndex: 2000 }}
      >
        <Paper 
          elevation={8}
          sx={{ 
            p: 2, 
            borderRadius: 3, 
            background: 'linear-gradient(135deg, #1e3c72 0%, #2a5298 100%)',
            color: 'white',
            minWidth: 280,
          }}
        >
          <Stack spacing={2}>
            <Stack direction="row" alignItems="center" justifyContent="space-between">
              <Stack direction="row" alignItems="center" spacing={1}>
                <Badge 
                  badgeContent={participants.filter(p => p.is_speaking).length}
                  color="success"
                  invisible={participants.filter(p => p.is_speaking).length === 0}
                >
                  {callType === 'video' ? <VideoCallIcon /> : <PhoneIcon />}
                </Badge>
                <Box>
                  <Typography variant="subtitle2">
                    {entityName || 'Call'}
                  </Typography>
                  <Typography variant="caption">
                    {formatDuration(callDuration)}
                  </Typography>
                </Box>
              </Stack>
              <Stack direction="row" spacing={0.5}>
                <IconButton 
                  size="small" 
                  onClick={onMaximize}
                  sx={{ color: 'white' }}
                >
                  <MaximizeIcon fontSize="small" />
                </IconButton>
                <IconButton 
                  size="small" 
                  onClick={onEndCall}
                  sx={{ color: 'error.main' }}
                >
                  <CallEndIcon fontSize="small" />
                </IconButton>
              </Stack>
            </Stack>
            
            <Stack direction="row" alignItems="center" justifyContent="space-between">
              <AvatarGroup max={3} sx={{ '& .MuiAvatar-root': { width: 24, height: 24, fontSize: '0.75rem' } }}>
                {participants.map((participant) => (
                  <Avatar 
                    key={participant.user_id}
                    sx={{ 
                      border: participant.is_speaking ? '2px solid #4caf50' : 'none',
                      opacity: participant.is_muted ? 0.7 : 1,
                    }}
                  >
                    {participant.display_name[0]}
                  </Avatar>
                ))}
              </AvatarGroup>
              
              <Stack direction="row" spacing={0.5}>
                <IconButton 
                  size="small" 
                  onClick={handleToggleMute}
                  sx={{ 
                    color: isMuted ? 'error.main' : 'white',
                    bgcolor: isMuted ? 'rgba(244, 67, 54, 0.1)' : 'transparent'
                  }}
                >
                  {isMuted ? <MicOffIcon fontSize="small" /> : <MicIcon fontSize="small" />}
                </IconButton>
                {callType === 'video' && (
                  <IconButton 
                    size="small" 
                    onClick={handleToggleVideo}
                    sx={{ 
                      color: !isVideoEnabled ? 'error.main' : 'white',
                      bgcolor: !isVideoEnabled ? 'rgba(244, 67, 54, 0.1)' : 'transparent'
                    }}
                  >
                    {isVideoEnabled ? <VideocamIcon fontSize="small" /> : <VideocamOffIcon fontSize="small" />}
                  </IconButton>
                )}
              </Stack>
            </Stack>
          </Stack>
        </Paper>
      </motion.div>
    );
  }

  // Full interface
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.9 }}
      style={{ 
        position: 'fixed', 
        top: 0, 
        left: 0, 
        right: 0, 
        bottom: 0, 
        zIndex: 1500,
        backgroundColor: '#000',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      {/* Header Bar */}
      <Paper 
        square
        elevation={0}
        sx={{ 
          bgcolor: 'rgba(0, 0, 0, 0.8)', 
          color: 'white', 
          p: 1,
          borderBottom: '1px solid rgba(255, 255, 255, 0.1)'
        }}
      >
        <Stack direction="row" alignItems="center" justifyContent="space-between">
          <Stack direction="row" alignItems="center" spacing={2}>
            <DragHandleIcon sx={{ color: 'grey.400' }} />
            <Stack>
              <Typography variant="subtitle1">
                {entityName || 'Call'} • {entityType || 'Call'}
              </Typography>
              <Typography variant="caption" color="grey.300">
                {participants.length} participants • {formatDuration(callDuration)}
              </Typography>
            </Stack>
          </Stack>
          
          <Stack direction="row" spacing={1}>
            {isRecording && (
              <Chip
                label="Recording"
                color="error"
                size="small"
                sx={{ color: 'white' }}
              />
            )}
            <IconButton onClick={onMinimize} sx={{ color: 'white' }}>
              <MinimizeIcon />
            </IconButton>
            <IconButton onClick={onEndCall} sx={{ color: 'error.main' }}>
              <CloseIcon />
            </IconButton>
          </Stack>
        </Stack>
      </Paper>

      {/* Main Content Area */}
      <Box sx={{ flexGrow: 1, position: 'relative', overflow: 'hidden' }}>
        {/* Video Grid */}
        {callType === 'video' && (
          <Grid container sx={{ height: '100%' }}>
            {participants.map((participant, index) => (
              <Grid 
                key={participant.user_id} 
                item 
                xs={participants.length === 1 ? 12 : participants.length <= 4 ? 6 : 4}
              >
                <Box 
                  sx={{ 
                    position: 'relative', 
                    height: '100%', 
                    bgcolor: '#1a1a1a',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    border: participant.is_speaking ? '3px solid #4caf50' : '1px solid rgba(255, 255, 255, 0.1)',
                  }}
                >
                  {/* Video placeholder */}
                  <Avatar sx={{ width: 80, height: 80, fontSize: '2rem' }}>
                    {participant.display_name[0]}
                  </Avatar>
                  
                  {/* Participant info overlay */}
                  <Box
                    sx={{
                      position: 'absolute',
                      bottom: 8,
                      left: 8,
                      right: 8,
                      bgcolor: 'rgba(0, 0, 0, 0.7)',
                      borderRadius: 1,
                      p: 1,
                    }}
                  >
                    <Stack direction="row" alignItems="center" justifyContent="space-between">
                      <Typography variant="caption" color="white">
                        {participant.display_name}
                      </Typography>
                      <Stack direction="row" spacing={0.5} alignItems="center">
                        <Chip
                          size="small"
                          color={getConnectionQualityColor(participant.connection_quality) as any}
                          sx={{ height: 16, fontSize: '0.6rem' }}
                        />
                        {participant.is_muted && <MicOffIcon sx={{ fontSize: 14, color: 'error.main' }} />}
                        {!participant.is_video_enabled && <VideocamOffIcon sx={{ fontSize: 14, color: 'error.main' }} />}
                        {participant.is_screen_sharing && <ScreenShareIcon sx={{ fontSize: 14, color: 'info.main' }} />}
                      </Stack>
                    </Stack>
                  </Box>
                </Box>
              </Grid>
            ))}
          </Grid>
        )}

        {/* Voice-only interface */}
        {callType === 'voice' && (
          <Box 
            sx={{ 
              height: '100%', 
              display: 'flex', 
              alignItems: 'center', 
              justifyContent: 'center',
              background: 'radial-gradient(circle, #1e3c72 0%, #2a5298 100%)',
            }}
          >
            <Stack alignItems="center" spacing={4}>
              <AvatarGroup 
                max={6}
                sx={{ 
                  '& .MuiAvatar-root': { 
                    width: 80, 
                    height: 80, 
                    fontSize: '2rem',
                    border: '3px solid rgba(255, 255, 255, 0.2)'
                  } 
                }}
              >
                {participants.map((participant) => (
                  <Avatar 
                    key={participant.user_id}
                    sx={{ 
                      border: participant.is_speaking ? '3px solid #4caf50 !important' : undefined,
                      opacity: participant.is_muted ? 0.7 : 1,
                      transform: participant.is_speaking ? 'scale(1.1)' : 'scale(1)',
                      transition: 'all 0.3s ease',
                    }}
                  >
                    {participant.display_name[0]}
                  </Avatar>
                ))}
              </AvatarGroup>
              
              <Typography variant="h4" color="white" textAlign="center">
                {entityName || 'Voice Call'}
              </Typography>
              
              <Typography variant="h6" color="grey.300">
                {formatDuration(callDuration)}
              </Typography>
            </Stack>
          </Box>
        )}

        {/* Chat Sidebar */}
        <AnimatePresence>
          {showChat && (
            <motion.div
              initial={{ x: 400, opacity: 0 }}
              animate={{ x: 0, opacity: 1 }}
              exit={{ x: 400, opacity: 0 }}
              style={{
                position: 'absolute',
                top: 0,
                right: 0,
                bottom: 0,
                width: 400,
                zIndex: 10,
              }}
            >
              <Paper 
                square
                sx={{ 
                  height: '100%', 
                  bgcolor: 'rgba(0, 0, 0, 0.9)',
                  color: 'white',
                  display: 'flex',
                  flexDirection: 'column',
                }}
              >
                <Box sx={{ p: 2, borderBottom: '1px solid rgba(255, 255, 255, 0.1)' }}>
                  <Stack direction="row" alignItems="center" justifyContent="space-between">
                    <Typography variant="h6">Chat</Typography>
                    <IconButton onClick={() => setShowChat(false)} sx={{ color: 'white' }}>
                      <CloseIcon />
                    </IconButton>
                  </Stack>
                </Box>
                
                <Box 
                  ref={chatScrollRef}
                  sx={{ flexGrow: 1, overflow: 'auto', p: 1 }}
                >
                  {chatMessages.map((message) => (
                    <Box key={message.id} sx={{ mb: 2 }}>
                      <Stack direction="row" spacing={1} alignItems="flex-start">
                        <Avatar sx={{ width: 32, height: 32 }}>
                          {message.sender.display_name[0]}
                        </Avatar>
                        <Box sx={{ flexGrow: 1 }}>
                          <Stack direction="row" alignItems="center" spacing={1}>
                            <Typography variant="caption" color="grey.300">
                              {message.sender.display_name}
                            </Typography>
                            <Typography variant="caption" color="grey.500">
                              {new Date(message.timestamp).toLocaleTimeString()}
                            </Typography>
                          </Stack>
                          <Typography variant="body2" color="white">
                            {message.content}
                          </Typography>
                        </Box>
                      </Stack>
                    </Box>
                  ))}
                </Box>
                
                <Box sx={{ p: 2, borderTop: '1px solid rgba(255, 255, 255, 0.1)' }}>
                  <Stack direction="row" spacing={1} alignItems="flex-end">
                    <TextField
                      fullWidth
                      multiline
                      maxRows={3}
                      placeholder="Type a message..."
                      value={newMessage}
                      onChange={(e) => setNewMessage(e.target.value)}
                      onKeyPress={(e) => {
                        if (e.key === 'Enter' && !e.shiftKey) {
                          e.preventDefault();
                          handleSendMessage();
                        }
                      }}
                      sx={{
                        '& .MuiOutlinedInput-root': {
                          color: 'white',
                          '& fieldset': { borderColor: 'rgba(255, 255, 255, 0.3)' },
                          '&:hover fieldset': { borderColor: 'rgba(255, 255, 255, 0.5)' },
                          '&.Mui-focused fieldset': { borderColor: 'primary.main' },
                        },
                      }}
                    />
                    <IconButton 
                      onClick={handleSendMessage}
                      disabled={!newMessage.trim()}
                      sx={{ color: 'primary.main' }}
                    >
                      <SendIcon />
                    </IconButton>
                  </Stack>
                </Box>
              </Paper>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Participants Sidebar */}
        <AnimatePresence>
          {showParticipants && (
            <motion.div
              initial={{ x: -400, opacity: 0 }}
              animate={{ x: 0, opacity: 1 }}
              exit={{ x: -400, opacity: 0 }}
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                bottom: 0,
                width: 350,
                zIndex: 10,
              }}
            >
              <Paper 
                square
                sx={{ 
                  height: '100%', 
                  bgcolor: 'rgba(0, 0, 0, 0.9)',
                  color: 'white',
                  display: 'flex',
                  flexDirection: 'column',
                }}
              >
                <Box sx={{ p: 2, borderBottom: '1px solid rgba(255, 255, 255, 0.1)' }}>
                  <Stack direction="row" alignItems="center" justifyContent="space-between">
                    <Typography variant="h6">Participants ({participants.length})</Typography>
                    <Stack direction="row" spacing={1}>
                      <IconButton 
                        onClick={() => setInviteDialogOpen(true)}
                        sx={{ color: 'primary.main' }}
                      >
                        <PersonAddIcon />
                      </IconButton>
                      <IconButton onClick={() => setShowParticipants(false)} sx={{ color: 'white' }}>
                        <CloseIcon />
                      </IconButton>
                    </Stack>
                  </Stack>
                </Box>
                
                <List sx={{ flexGrow: 1, overflow: 'auto' }}>
                  {participants.map((participant) => (
                    <ListItem key={participant.user_id}>
                      <ListItemAvatar>
                        <Badge
                          overlap="circular"
                          anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                          badgeContent={
                            <Box
                              sx={{
                                width: 12,
                                height: 12,
                                borderRadius: '50%',
                                bgcolor: getConnectionQualityColor(participant.connection_quality) + '.main',
                                border: '2px solid black',
                              }}
                            />
                          }
                        >
                          <Avatar
                            sx={{
                              border: participant.is_speaking ? '2px solid #4caf50' : '1px solid rgba(255, 255, 255, 0.2)',
                            }}
                          >
                            {participant.display_name[0]}
                          </Avatar>
                        </Badge>
                      </ListItemAvatar>
                      <ListItemText
                        primary={
                          <Stack direction="row" alignItems="center" spacing={1}>
                            <Typography color="white">
                              {participant.display_name}
                            </Typography>
                            {participant.is_speaking && (
                              <Chip label="Speaking" color="success" size="small" />
                            )}
                          </Stack>
                        }
                        secondary={
                          <Stack direction="row" spacing={1} alignItems="center">
                            <Typography variant="caption" color="grey.400">
                              {participant.four_word_address}
                            </Typography>
                            {participant.is_muted && <MicOffIcon sx={{ fontSize: 12, color: 'error.main' }} />}
                            {!participant.is_video_enabled && callType === 'video' && (
                              <VideocamOffIcon sx={{ fontSize: 12, color: 'error.main' }} />
                            )}
                            {participant.is_screen_sharing && (
                              <ScreenShareIcon sx={{ fontSize: 12, color: 'info.main' }} />
                            )}
                          </Stack>
                        }
                      />
                    </ListItem>
                  ))}
                </List>
              </Paper>
            </motion.div>
          )}
        </AnimatePresence>
      </Box>

      {/* Control Bar */}
      <Paper 
        square
        elevation={0}
        sx={{ 
          bgcolor: 'rgba(0, 0, 0, 0.9)', 
          p: 2,
          borderTop: '1px solid rgba(255, 255, 255, 0.1)'
        }}
      >
        <Stack direction="row" alignItems="center" justifyContent="center" spacing={2}>
          {/* Primary Controls */}
          <IconButton
            onClick={handleToggleMute}
            sx={{
              bgcolor: isMuted ? 'error.main' : 'rgba(255, 255, 255, 0.1)',
              color: 'white',
              '&:hover': { bgcolor: isMuted ? 'error.dark' : 'rgba(255, 255, 255, 0.2)' },
            }}
          >
            {isMuted ? <MicOffIcon /> : <MicIcon />}
          </IconButton>

          {callType === 'video' && (
            <IconButton
              onClick={handleToggleVideo}
              sx={{
                bgcolor: !isVideoEnabled ? 'error.main' : 'rgba(255, 255, 255, 0.1)',
                color: 'white',
                '&:hover': { bgcolor: !isVideoEnabled ? 'error.dark' : 'rgba(255, 255, 255, 0.2)' },
              }}
            >
              {isVideoEnabled ? <VideocamIcon /> : <VideocamOffIcon />}
            </IconButton>
          )}

          <IconButton
            onClick={handleToggleScreenShare}
            sx={{
              bgcolor: isScreenSharing ? 'info.main' : 'rgba(255, 255, 255, 0.1)',
              color: 'white',
              '&:hover': { bgcolor: isScreenSharing ? 'info.dark' : 'rgba(255, 255, 255, 0.2)' },
            }}
          >
            {isScreenSharing ? <StopScreenShareIcon /> : <ScreenShareIcon />}
          </IconButton>

          {/* End Call */}
          <IconButton
            onClick={onEndCall}
            sx={{
              bgcolor: 'error.main',
              color: 'white',
              '&:hover': { bgcolor: 'error.dark' },
              mx: 2,
            }}
          >
            <CallEndIcon />
          </IconButton>

          {/* Secondary Controls */}
          <IconButton
            onClick={() => setShowParticipants(!showParticipants)}
            sx={{ color: showParticipants ? 'primary.main' : 'white' }}
          >
            <Badge badgeContent={participants.length} color="primary">
              <PeopleIcon />
            </Badge>
          </IconButton>

          <IconButton
            onClick={() => setShowChat(!showChat)}
            sx={{ color: showChat ? 'primary.main' : 'white' }}
          >
            <Badge badgeContent={chatMessages.length} color="primary">
              <ChatIcon />
            </Badge>
          </IconButton>

          <IconButton
            onClick={() => setIsRecording(!isRecording)}
            sx={{ color: isRecording ? 'error.main' : 'white' }}
          >
            {isRecording ? <StopIcon /> : <FiberManualRecordIcon />}
          </IconButton>

          <IconButton onClick={handleMenuOpen} sx={{ color: 'white' }}>
            <MoreVertIcon />
          </IconButton>
        </Stack>
      </Paper>

      {/* Settings Menu */}
      <Menu
        anchorEl={menuAnchor}
        open={Boolean(menuAnchor)}
        onClose={handleMenuClose}
        PaperProps={{
          sx: { bgcolor: 'rgba(0, 0, 0, 0.9)', color: 'white' },
        }}
      >
        <MenuItem onClick={() => { handleMenuClose(); setIsPipMode(!isPipMode); }}>
          <ListItemIcon><PipIcon sx={{ color: 'white' }} /></ListItemIcon>
          <ListItemText>Picture in Picture</ListItemText>
        </MenuItem>
        <MenuItem onClick={() => { handleMenuClose(); setIsFullscreen(!isFullscreen); }}>
          <ListItemIcon>
            {isFullscreen ? <FullscreenExitIcon sx={{ color: 'white' }} /> : <FullscreenIcon sx={{ color: 'white' }} />}
          </ListItemIcon>
          <ListItemText>{isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'}</ListItemText>
        </MenuItem>
        <MenuItem onClick={() => { handleMenuClose(); setShowSettings(true); }}>
          <ListItemIcon><SettingsIcon sx={{ color: 'white' }} /></ListItemIcon>
          <ListItemText>Settings</ListItemText>
        </MenuItem>
      </Menu>

      {/* Invite Dialog */}
      <Dialog 
        open={inviteDialogOpen} 
        onClose={() => setInviteDialogOpen(false)}
        PaperProps={{ sx: { bgcolor: 'background.paper', minWidth: 400 } }}
      >
        <DialogTitle>Invite Participants</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Four-word address or name"
            fullWidth
            variant="outlined"
            placeholder="brave-yellow-mountain-tree"
          />
          <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
            Enter a four-word address or search by name to invite someone to this call.
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setInviteDialogOpen(false)}>Cancel</Button>
          <Button variant="contained">Send Invitation</Button>
        </DialogActions>
      </Dialog>
    </motion.div>
  );
};

export default CallInterface;