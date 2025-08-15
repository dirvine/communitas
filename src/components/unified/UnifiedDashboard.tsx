import React, { useState, useEffect } from 'react';
import {
  Box,
  Paper,
  Typography,
  IconButton,
  Avatar,
  Badge,
  Chip,
  Tab,
  Tabs,
  Tooltip,
  Button,
  Card,
  CardContent,
  Grid,
  Fade,
  Zoom,
  Grow,
  alpha,
  useTheme,
  LinearProgress,
  AvatarGroup,
  Stack,
  Divider,
} from '@mui/material';
import {
  Chat as ChatIcon,
  Folder as FolderIcon,
  VideoCall as VideoCallIcon,
  Notifications as NotificationsIcon,
  Search as SearchIcon,
  Add as AddIcon,
  CloudQueue as CloudIcon,
  Speed as SpeedIcon,
  Security as SecurityIcon,
  Groups as GroupsIcon,
  TrendingUp as TrendingUpIcon,
  FiberManualRecord as OnlineIcon,
  AccessTime as RecentIcon,
  Star as StarIcon,
  AttachFile as AttachmentIcon,
  Mic as VoiceIcon,
  PhotoCamera as CameraIcon,
  ScreenShare as ScreenShareIcon,
  KeyboardVoice as VoiceMessageIcon,
  Image as ImageIcon,
  Description as DocumentIcon,
  PlayCircle as VideoIcon,
  CloudDone as SyncedIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { ChatInterface } from '../chat/ChatInterface';
import { FileManager } from '../storage/FileManager';

// Wrap MUI components with motion for animations
const MotionBox = motion(Box);
const MotionCard = motion(Card);
const MotionPaper = motion(Paper);

interface UnifiedDashboardProps {
  userId: string;
  userName: string;
  fourWords?: string;
}

interface QuickAction {
  id: string;
  title: string;
  icon: React.ReactNode;
  color: string;
  gradient: string;
  action: () => void;
  badge?: number;
}

interface RecentActivity {
  id: string;
  type: 'message' | 'file' | 'call' | 'share';
  title: string;
  subtitle: string;
  timestamp: Date;
  avatar?: string;
  icon: React.ReactNode;
}

export const UnifiedDashboard: React.FC<UnifiedDashboardProps> = ({
  userId,
  userName,
  fourWords = 'ocean-forest-moon-star',
}) => {
  const theme = useTheme();
  const [activeTab, setActiveTab] = useState(0);
  const [showNotifications, setShowNotifications] = useState(false);
  const [onlineUsers] = useState(12);
  const [unreadMessages] = useState(5);
  const [pendingFiles] = useState(3);

  const quickActions: QuickAction[] = [
    {
      id: 'new-chat',
      title: 'Start Chat',
      icon: <ChatIcon />,
      color: '#00BFA5',
      gradient: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
      action: () => console.log('Start chat'),
      badge: unreadMessages,
    },
    {
      id: 'video-call',
      title: 'Video Call',
      icon: <VideoCallIcon />,
      color: '#FF6B6B',
      gradient: 'linear-gradient(135deg, #f093fb 0%, #f5576c 100%)',
      action: () => console.log('Start video call'),
    },
    {
      id: 'open-storage',
      title: 'Open Storage',
      icon: <FolderIcon />,
      color: '#4ECDC4',
      gradient: 'linear-gradient(135deg, #fa709a 0%, #fee140 100%)',
      action: () => {
        const event = new CustomEvent('open-storage-workspace', { detail: { scope: 'personal', userId } })
        window.dispatchEvent(event)
      },
      badge: pendingFiles,
    },
    {
      id: 'screen-share',
      title: 'Screen Share',
      icon: <ScreenShareIcon />,
      color: '#FF8C42',
      gradient: 'linear-gradient(135deg, #a8edea 0%, #fed6e3 100%)',
      action: () => console.log('Screen share'),
    },
  ];

  const recentActivities: RecentActivity[] = [
    {
      id: '1',
      type: 'message',
      title: 'Alice Johnson',
      subtitle: 'Sent you a message about the project',
      timestamp: new Date(Date.now() - 300000),
      icon: <ChatIcon fontSize="small" />,
    },
    {
      id: '2',
      type: 'file',
      title: 'Bob Chen',
      subtitle: 'Shared project-specs.pdf',
      timestamp: new Date(Date.now() - 1800000),
      icon: <DocumentIcon fontSize="small" />,
    },
    {
      id: '3',
      type: 'call',
      title: 'Team Standup',
      subtitle: 'Video call ended (45 min)',
      timestamp: new Date(Date.now() - 3600000),
      icon: <VideoCallIcon fontSize="small" />,
    },
    {
      id: '4',
      type: 'share',
      title: 'Sarah Kim',
      subtitle: 'Added you to "Design Assets" folder',
      timestamp: new Date(Date.now() - 7200000),
      icon: <FolderIcon fontSize="small" />,
    },
  ];

const containerVariants: any = {
    hidden: { opacity: 0 },
    visible: {
      opacity: 1,
      transition: {
        staggerChildren: 0.1,
      },
    },
  };

const itemVariants: any = {
    hidden: { y: 20, opacity: 0 },
    visible: {
      y: 0,
      opacity: 1,
      transition: {
        type: 'spring',
        stiffness: 100,
      },
    },
  };

  return (
    <Box
      sx={{
        height: '100vh',
        display: 'flex',
        flexDirection: 'column',
        background: theme.palette.mode === 'dark'
          ? 'linear-gradient(135deg, #1e3c72 0%, #2a5298 100%)'
          : 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
      }}
    >
      {/* Stunning Header */}
      <MotionPaper
        initial={{ y: -50, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ duration: 0.5 }}
        elevation={0}
        sx={{
          p: 3,
          background: alpha(theme.palette.background.paper, 0.9),
          backdropFilter: 'blur(20px)',
          borderBottom: `1px solid ${alpha(theme.palette.divider, 0.1)}`,
        }}
      >
        <Grid container alignItems="center" spacing={3}>
          <Grid xs={12} md={4}>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
              <Badge
                overlap="circular"
                anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                badgeContent={
                  <OnlineIcon sx={{ color: '#44b700', fontSize: 12 }} />
                }
              >
                <Avatar
                  sx={{
                    width: 56,
                    height: 56,
                    background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                  }}
                >
                  {userName[0]}
                </Avatar>
              </Badge>
              <Box>
                <Typography variant="h5" fontWeight={600}>
                  Welcome back, {userName}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {fourWords}
                </Typography>
              </Box>
            </Box>
          </Grid>

          <Grid xs={12} md={4}>
            <Stack direction="row" spacing={2} justifyContent="center">
              <Chip
                icon={<GroupsIcon />}
                label={`${onlineUsers} Online`}
                color="success"
                variant="outlined"
              />
              <Chip
                icon={<ChatIcon />}
                label={`${unreadMessages} Unread`}
                color="primary"
                variant="outlined"
              />
              <Chip
                icon={<CloudIcon />}
                label="All Synced"
                color="info"
                variant="outlined"
              />
            </Stack>
          </Grid>

          <Grid xs={12} md={4}>
            <Box sx={{ display: 'flex', gap: 1, justifyContent: 'flex-end' }}>
              <IconButton
                sx={{
                  background: alpha(theme.palette.primary.main, 0.1),
                  '&:hover': {
                    background: alpha(theme.palette.primary.main, 0.2),
                  },
                }}
              >
                <SearchIcon />
              </IconButton>
              <Badge badgeContent={3} color="error">
                <IconButton
                  sx={{
                    background: alpha(theme.palette.primary.main, 0.1),
                    '&:hover': {
                      background: alpha(theme.palette.primary.main, 0.2),
                    },
                  }}
                  onClick={() => setShowNotifications(!showNotifications)}
                >
                  <NotificationsIcon />
                </IconButton>
              </Badge>
            </Box>
          </Grid>
        </Grid>
      </MotionPaper>

      {/* Quick Actions - Beautiful animated cards */}
      <Box sx={{ px: 3, py: 2 }}>
        <motion.div
          variants={containerVariants}
          initial="hidden"
          animate="visible"
        >
          <Grid container spacing={2}>
            {quickActions.map((action, index) => (
              <Grid xs={6} sm={3} key={action.id}>
                <MotionCard
                  variants={itemVariants}
                  whileHover={{ scale: 1.05, rotate: 1 }}
                  whileTap={{ scale: 0.95 }}
                  sx={{
                    background: action.gradient,
                    color: 'white',
                    cursor: 'pointer',
                    position: 'relative',
                    overflow: 'hidden',
                    '&::before': {
                      content: '""',
                      position: 'absolute',
                      top: 0,
                      left: 0,
                      right: 0,
                      bottom: 0,
                      background: 'rgba(255,255,255,0.1)',
                      transform: 'translateX(-100%)',
                      transition: 'transform 0.3s',
                    },
                    '&:hover::before': {
                      transform: 'translateX(0)',
                    },
                  }}
                  onClick={action.action}
                >
                  <CardContent sx={{ textAlign: 'center', py: 3 }}>
                    <Badge badgeContent={action.badge} color="error">
                      <Box sx={{ fontSize: 40, mb: 1 }}>{action.icon}</Box>
                    </Badge>
                    <Typography variant="body1" fontWeight={500}>
                      {action.title}
                    </Typography>
                  </CardContent>
                </MotionCard>
              </Grid>
            ))}
          </Grid>
        </motion.div>
      </Box>

      {/* Main Content Area with Tabs */}
      <MotionBox
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ delay: 0.3 }}
        sx={{
          flex: 1,
          mx: 3,
          mb: 3,
          display: 'flex',
          flexDirection: 'column',
          background: alpha(theme.palette.background.paper, 0.95),
          backdropFilter: 'blur(20px)',
          borderRadius: 3,
          overflow: 'hidden',
          boxShadow: '0 20px 40px rgba(0,0,0,0.1)',
        }}
      >
        <Box
          sx={{
            borderBottom: 1,
            borderColor: 'divider',
            background: alpha(theme.palette.primary.main, 0.05),
          }}
        >
          <Tabs
            value={activeTab}
            onChange={(e, v) => setActiveTab(v)}
            sx={{
              '& .MuiTab-root': {
                minHeight: 64,
                fontSize: '1rem',
                fontWeight: 500,
              },
            }}
          >
            <Tab
              icon={<ChatIcon />}
              label="Messages"
              iconPosition="start"
            />
            <Tab
              icon={<FolderIcon />}
              label="Files"
              iconPosition="start"
            />
            <Tab
              icon={<VideoCallIcon />}
              label="Meetings"
              iconPosition="start"
            />
            <Tab
              icon={<RecentIcon />}
              label="Activity"
              iconPosition="start"
            />
          </Tabs>
        </Box>

        <Box sx={{ flex: 1, overflow: 'hidden' }}>
          <AnimatePresence mode="wait">
            {activeTab === 0 && (
              <MotionBox
                key="chat"
                initial={{ opacity: 0, x: 20 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -20 }}
                sx={{ height: '100%' }}
              >
                <Grid container sx={{ height: '100%' }}>
                  <Grid xs={12} md={4} sx={{ borderRight: 1, borderColor: 'divider' }}>
                    {/* Chat List */}
                    <Box sx={{ p: 2 }}>
                      <Typography variant="h6" gutterBottom>
                        Recent Conversations
                      </Typography>
                      {['Alice Johnson', 'Bob Chen', 'Team Chat', 'Project Alpha'].map((chat, i) => (
                        <Paper
                          key={i}
                          sx={{
                            p: 2,
                            mb: 1,
                            cursor: 'pointer',
                            transition: 'all 0.3s',
                            '&:hover': {
                              transform: 'translateX(4px)',
                              boxShadow: 2,
                            },
                          }}
                        >
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                            <Badge
                              variant="dot"
                              color="success"
                              invisible={i > 1}
                            >
                              <Avatar>{chat[0]}</Avatar>
                            </Badge>
                            <Box sx={{ flex: 1 }}>
                              <Typography variant="body1" fontWeight={500}>
                                {chat}
                              </Typography>
                              <Typography variant="caption" color="text.secondary">
                                Last message 5 min ago
                              </Typography>
                            </Box>
                            {i === 0 && (
                              <Badge badgeContent={2} color="primary" />
                            )}
                          </Box>
                        </Paper>
                      ))}
                    </Box>
                  </Grid>
                  <Grid xs={12} md={8}>
                    <ChatInterface
                      chatId="1"
                      chatName="Alice Johnson"
                      chatType="direct"
                    />
                  </Grid>
                </Grid>
              </MotionBox>
            )}

            {activeTab === 1 && (
              <MotionBox
                key="files"
                initial={{ opacity: 0, x: 20 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -20 }}
                sx={{ height: '100%' }}
              >
                <FileManager />
              </MotionBox>
            )}

            {activeTab === 2 && (
              <MotionBox
                key="meetings"
                initial={{ opacity: 0, x: 20 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -20 }}
                sx={{ p: 4, textAlign: 'center' }}
              >
                <Grid container spacing={3} justifyContent="center">
                  <Grid xs={12}>
                    <Typography variant="h4" gutterBottom>
                      Start or Join a Meeting
                    </Typography>
                  </Grid>
                  <Grid xs={12} sm={6} md={4}>
                    <MotionCard
                      whileHover={{ scale: 1.05 }}
                      sx={{
                        background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                        color: 'white',
                        cursor: 'pointer',
                        minHeight: 200,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                      }}
                    >
                      <CardContent sx={{ textAlign: 'center' }}>
                        <VideoCallIcon sx={{ fontSize: 60, mb: 2 }} />
                        <Typography variant="h6">
                          Start Instant Meeting
                        </Typography>
                        <Typography variant="body2" sx={{ mt: 1, opacity: 0.9 }}>
                          Start a video call right now
                        </Typography>
                      </CardContent>
                    </MotionCard>
                  </Grid>
                  <Grid xs={12} sm={6} md={4}>
                    <MotionCard
                      whileHover={{ scale: 1.05 }}
                      sx={{
                        background: 'linear-gradient(135deg, #f093fb 0%, #f5576c 100%)',
                        color: 'white',
                        cursor: 'pointer',
                        minHeight: 200,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                      }}
                    >
                      <CardContent sx={{ textAlign: 'center' }}>
                        <VoiceIcon sx={{ fontSize: 60, mb: 2 }} />
                        <Typography variant="h6">
                          Voice Call
                        </Typography>
                        <Typography variant="body2" sx={{ mt: 1, opacity: 0.9 }}>
                          Start a voice-only call
                        </Typography>
                      </CardContent>
                    </MotionCard>
                  </Grid>
                  <Grid xs={12} sm={6} md={4}>
                    <MotionCard
                      whileHover={{ scale: 1.05 }}
                      sx={{
                        background: 'linear-gradient(135deg, #fa709a 0%, #fee140 100%)',
                        color: 'white',
                        cursor: 'pointer',
                        minHeight: 200,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                      }}
                    >
                      <CardContent sx={{ textAlign: 'center' }}>
                        <ScreenShareIcon sx={{ fontSize: 60, mb: 2 }} />
                        <Typography variant="h6">
                          Share Screen
                        </Typography>
                        <Typography variant="body2" sx={{ mt: 1, opacity: 0.9 }}>
                          Share your screen with others
                        </Typography>
                      </CardContent>
                    </MotionCard>
                  </Grid>
                </Grid>

                {/* Recent Meetings */}
                <Box sx={{ mt: 6 }}>
                  <Typography variant="h5" gutterBottom>
                    Recent Meetings
                  </Typography>
                  <Grid container spacing={2}>
                    {['Team Standup', 'Project Review', 'Client Demo'].map((meeting, i) => (
                      <Grid xs={12} key={i}>
                        <Paper sx={{ p: 2 }}>
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                            <Avatar sx={{ bgcolor: 'primary.main' }}>
                              <VideoCallIcon />
                            </Avatar>
                            <Box sx={{ flex: 1 }}>
                              <Typography variant="body1" fontWeight={500}>
                                {meeting}
                              </Typography>
                              <Typography variant="caption" color="text.secondary">
                                {i === 0 ? 'Today at 10:00 AM' : `${i + 1} days ago`}
                              </Typography>
                            </Box>
                            <AvatarGroup max={4}>
                              <Avatar sx={{ width: 32, height: 32 }}>A</Avatar>
                              <Avatar sx={{ width: 32, height: 32 }}>B</Avatar>
                              <Avatar sx={{ width: 32, height: 32 }}>C</Avatar>
                              <Avatar sx={{ width: 32, height: 32 }}>+5</Avatar>
                            </AvatarGroup>
                            <Button variant="outlined" size="small">
                              View Recording
                            </Button>
                          </Box>
                        </Paper>
                      </Grid>
                    ))}
                  </Grid>
                </Box>
              </MotionBox>
            )}

            {activeTab === 3 && (
              <MotionBox
                key="activity"
                initial={{ opacity: 0, x: 20 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -20 }}
                sx={{ p: 3 }}
              >
                <Typography variant="h5" gutterBottom>
                  Recent Activity
                </Typography>
                <Stack spacing={2}>
                  {recentActivities.map((activity) => (
                    <MotionPaper
                      key={activity.id}
                      initial={{ opacity: 0, x: -20 }}
                      animate={{ opacity: 1, x: 0 }}
                      whileHover={{ x: 4 }}
                      sx={{
                        p: 2,
                        display: 'flex',
                        alignItems: 'center',
                        gap: 2,
                        cursor: 'pointer',
                      }}
                    >
                      <Avatar
                        sx={{
                          bgcolor: alpha(theme.palette.primary.main, 0.1),
                          color: 'primary.main',
                        }}
                      >
                        {activity.icon}
                      </Avatar>
                      <Box sx={{ flex: 1 }}>
                        <Typography variant="body1" fontWeight={500}>
                          {activity.title}
                        </Typography>
                        <Typography variant="body2" color="text.secondary">
                          {activity.subtitle}
                        </Typography>
                      </Box>
                      <Typography variant="caption" color="text.secondary">
                        {new Date(activity.timestamp).toLocaleTimeString()}
                      </Typography>
                    </MotionPaper>
                  ))}
                </Stack>
              </MotionBox>
            )}
          </AnimatePresence>
        </Box>
      </MotionBox>

      {/* Floating Action Button */}
      <Zoom in={true}>
        <Box
          sx={{
            position: 'fixed',
            bottom: 24,
            right: 24,
          }}
        >
          <IconButton
            sx={{
              width: 64,
              height: 64,
              background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
              color: 'white',
              boxShadow: '0 8px 32px rgba(102, 126, 234, 0.4)',
              '&:hover': {
                transform: 'scale(1.1)',
                boxShadow: '0 12px 40px rgba(102, 126, 234, 0.6)',
              },
            }}
          >
            <AddIcon sx={{ fontSize: 32 }} />
          </IconButton>
        </Box>
      </Zoom>
    </Box>
  );
};