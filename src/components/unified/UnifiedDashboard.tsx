import React, { useMemo, useState } from 'react';
import {
  alpha,
  useTheme,
} from '@mui/material/styles';
import {
  Avatar,
  AvatarGroup,
  Badge,
  Box,
  Button,
  ButtonBase,
  Chip,
  Divider,
  Grid,
  IconButton,
  LinearProgress,
  Stack,
  Tab,
  Tabs,
  Tooltip,
  Typography,
} from '@mui/material';
import {
  ArrowForward as ArrowForwardIcon,
  Chat as ChatIcon,
  CloudDone as CloudDoneIcon,
  CloudQueue as CloudIcon,
  Description as DocumentIcon,
  Folder as FolderIcon,
  Groups as GroupsIcon,
  Lan as LanIcon,
  Notifications as NotificationsIcon,
  ScreenShare as ScreenShareIcon,
  Security as SecurityIcon,
  Speed as SpeedIcon,
  TrendingUp as TrendingUpIcon,
  VideoCall as VideoCallIcon,
} from '@mui/icons-material';
import { formatDistanceToNow } from 'date-fns';
import { motion, Variants } from 'framer-motion';

import { GlassCard } from '../ui/GlassCard';
import { ChatInterface } from '../chat/ChatInterface';
import { FileManager } from '../storage/FileManager';

const MotionBox = motion(Box);
const MotionStack = motion(Stack);

interface UnifiedDashboardProps {
  userId: string;
  userName: string;
  fourWords?: string;
}

interface QuickAction {
  id: string;
  title: string;
  description: string;
  icon: React.ReactNode;
  gradient: string;
  action: () => void;
  badge?: number;
}

interface InsightMetric {
  id: string;
  label: string;
  value: string;
  helper: string;
  icon: React.ReactNode;
  accent: string;
  progress?: number;
}

interface RecentActivity {
  id: string;
  title: string;
  subtitle: string;
  icon: React.ReactNode;
  timestamp: Date;
  tone: string;
}

const heroVariants: Variants = {
  hidden: { opacity: 0, y: 20 },
  visible: {
    opacity: 1,
    y: 0,
    transition: { duration: 0.6 },
  },
};

const sectionVariants: Variants = {
  hidden: { opacity: 0, y: 30 },
  visible: ({ delay }: { delay: number }) => ({
    opacity: 1,
    y: 0,
    transition: { duration: 0.6, delay },
  }),
};

export const UnifiedDashboard: React.FC<UnifiedDashboardProps> = ({
  userId,
  userName,
  fourWords = 'ocean-forest-moon-star',
}) => {
  const theme = useTheme();
  const [focusTab, setFocusTab] = useState<'messages' | 'storage'>('messages');

  const onlineUsers = 12;
  const unreadMessages = 5;
  const pendingFiles = 3;

  const quickActions: QuickAction[] = useMemo(() => [
    {
      id: 'chat',
      title: 'Start a Conversation',
      description: 'Spin up a private room with end-to-end encryption.',
      icon: <ChatIcon fontSize="inherit" />,
      gradient: 'linear-gradient(135deg, #6366F1 0%, #8B5CF6 100%)',
      action: () => window.dispatchEvent(new CustomEvent('open-chat-composer', { detail: { userId } })),
      badge: unreadMessages,
    },
    {
      id: 'video-call',
      title: 'Launch a Video Room',
      description: 'Secure HD meetings with zero centralized fallback.',
      icon: <VideoCallIcon fontSize="inherit" />,
      gradient: 'linear-gradient(135deg, #F97316 0%, #F43F5E 100%)',
      action: () => window.dispatchEvent(new CustomEvent('start-video-call', { detail: { userId } })),
    },
    {
      id: 'open-storage',
      title: 'Open Your Vault',
      description: 'Navigate personal, shared, and org storage at once.',
      icon: <FolderIcon fontSize="inherit" />,
      gradient: 'linear-gradient(135deg, #22D3EE 0%, #14B8A6 100%)',
      action: () => window.dispatchEvent(new CustomEvent('open-storage-workspace', { detail: { scope: 'personal', userId } })),
      badge: pendingFiles,
    },
    {
      id: 'share-screen',
      title: 'Share Your Screen',
      description: 'Collaborate live with low-latency quantum-safe channels.',
      icon: <ScreenShareIcon fontSize="inherit" />,
      gradient: 'linear-gradient(135deg, #F59E0B 0%, #84CC16 100%)',
      action: () => window.dispatchEvent(new CustomEvent('start-screen-share', { detail: { userId } })),
    },
  ], [pendingFiles, unreadMessages, userId]);

  const insights: InsightMetric[] = useMemo(() => [
    {
      id: 'network',
      label: 'Network Health',
      value: `${onlineUsers} peers online`,
      helper: 'Mesh is resilient across 4 regions',
      icon: <LanIcon fontSize="small" />,
      accent: theme.palette.primary.main,
      progress: 92,
    },
    {
      id: 'sync',
      label: 'Sync Coverage',
      value: '97% complete',
      helper: 'Last backlog cleared 4 minutes ago',
      icon: <CloudDoneIcon fontSize="small" />,
      accent: theme.palette.success.main,
      progress: 97,
    },
    {
      id: 'security',
      label: 'Security Posture',
      value: 'All systems hardened',
      helper: 'Passkeys + ML-DSA in active rotation',
      icon: <SecurityIcon fontSize="small" />,
      accent: theme.palette.secondary.main,
      progress: 100,
    },
  ], [onlineUsers, theme.palette.primary.main, theme.palette.secondary.main, theme.palette.success.main]);

  const recentActivities: RecentActivity[] = useMemo(() => [
    {
      id: '1',
      title: 'Alice Johnson',
      subtitle: 'Shared a thread update in "P2P Launch Sprint"',
      icon: <ChatIcon fontSize="small" />,
      timestamp: new Date(Date.now() - 300000),
      tone: theme.palette.primary.main,
    },
    {
      id: '2',
      title: 'Bob Chen',
      subtitle: 'Uploaded architecture-v3.pdf to Shared Vault',
      icon: <DocumentIcon fontSize="small" />,
      timestamp: new Date(Date.now() - 1800000),
      tone: theme.palette.success.main,
    },
    {
      id: '3',
      title: 'Team Standup',
      subtitle: '45-minute call recorded & synced to workspace',
      icon: <VideoCallIcon fontSize="small" />,
      timestamp: new Date(Date.now() - 3600000),
      tone: theme.palette.info.main,
    },
    {
      id: '4',
      title: 'Sarah Kim',
      subtitle: 'Granted you access to Design Assets collection',
      icon: <FolderIcon fontSize="small" />,
      timestamp: new Date(Date.now() - 7200000),
      tone: theme.palette.warning.main,
    },
  ], [theme.palette.info.main, theme.palette.primary.main, theme.palette.success.main, theme.palette.warning.main]);

  const workspaceHighlights = useMemo(() => [
    {
      id: 'org',
      title: 'Organization Mode',
      description: 'Switch to shared operations, approvals, and broadcast updates for your teams.',
      icon: <GroupsIcon />,
      accent: theme.palette.primary.main,
      actionLabel: 'View organizations',
      action: () => window.dispatchEvent(new CustomEvent('app:navigate', { detail: '/org/overview' })),
    },
    {
      id: 'projects',
      title: 'Projects Canvas',
      description: 'Plan, assign, and sync files with deterministic consensus across every node.',
      icon: <SpeedIcon />,
      accent: theme.palette.secondary.main,
      actionLabel: 'Open projects',
      action: () => window.dispatchEvent(new CustomEvent('app:navigate', { detail: '/project/overview' })),
    },
    {
      id: 'storage',
      title: 'Identity Vaults',
      description: 'Personal, shared, and public spaces encrypted with Reed-Solomon redundancy.',
      icon: <CloudIcon />,
      accent: theme.palette.success.main,
      actionLabel: 'Go to storage',
      action: () => window.dispatchEvent(new CustomEvent('open-storage-workspace', { detail: { scope: 'personal', userId } })),
    },
  ], [theme.palette.primary.main, theme.palette.secondary.main, theme.palette.success.main, userId]);

  return (
    <Box sx={{ position: 'relative', minHeight: '100%', py: { xs: 5, md: 7 }, px: { xs: 2, sm: 3, lg: 4 } }}>
      <Box
        sx={{
          position: 'absolute',
          inset: 0,
          background: theme.palette.mode === 'dark'
            ? `radial-gradient(circle at 10% 20%, ${alpha(theme.palette.primary.light, 0.25)} 0%, transparent 45%),
               radial-gradient(circle at 80% 10%, ${alpha(theme.palette.secondary.light, 0.2)} 0%, transparent 50%),
               radial-gradient(circle at 50% 80%, ${alpha(theme.palette.success.light, 0.18)} 0%, transparent 55%)`
            : `radial-gradient(circle at 10% 20%, ${alpha(theme.palette.primary.main, 0.18)} 0%, transparent 45%),
               radial-gradient(circle at 80% 10%, ${alpha(theme.palette.secondary.main, 0.14)} 0%, transparent 50%),
               radial-gradient(circle at 50% 80%, ${alpha(theme.palette.success.main, 0.15)} 0%, transparent 55%)`,
          filter: 'saturate(140%)',
          zIndex: 0,
        }}
      />
      <Box
        sx={{
          position: 'absolute',
          inset: 0,
          backgroundImage: `linear-gradient(${alpha(theme.palette.common.white, theme.palette.mode === 'dark' ? 0.04 : 0.08)} 1px, transparent 1px),
            linear-gradient(90deg, ${alpha(theme.palette.common.white, theme.palette.mode === 'dark' ? 0.04 : 0.08)} 1px, transparent 1px)` ,
          backgroundSize: '120px 120px',
          maskImage: 'radial-gradient(circle at center, rgba(0,0,0,0.7) 0%, transparent 70%)',
          pointerEvents: 'none',
          zIndex: 0,
        }}
      />

      <Stack spacing={{ xs: 4, md: 5 }} sx={{ position: 'relative', zIndex: 1, maxWidth: 1280, mx: 'auto' }}>
        <MotionBox variants={heroVariants} initial="hidden" animate="visible">
          <GlassCard variant="light" glow>
            <Box sx={{ position: 'relative', overflow: 'hidden' }}>
              <Box
                sx={{
                  position: 'absolute',
                  top: '-20%',
                  right: '-10%',
                  width: 280,
                  height: 280,
                  background: 'radial-gradient(circle, rgba(99,102,241,0.28) 0%, transparent 60%)',
                  filter: 'blur(20px)',
                }}
              />
              <Stack direction={{ xs: 'column', md: 'row' }} spacing={{ xs: 3, md: 6 }} sx={{ p: { xs: 3, md: 4 } }}>
                <Stack spacing={2} flex={1}>
                  <Stack direction="row" spacing={2} alignItems="center">
                    <Badge
                      overlap="circular"
                      anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                      badgeContent={<Box sx={{ width: 14, height: 14, bgcolor: 'success.main', borderRadius: '50%', border: '2px solid white' }} />}
                    >
                      <Avatar
                        sx={{
                          width: 64,
                          height: 64,
                          fontSize: 28,
                          fontWeight: 600,
                          background: 'linear-gradient(135deg, #6366F1 0%, #8B5CF6 100%)',
                        }}
                      >
                        {userName[0]}
                      </Avatar>
                    </Badge>
                    <Stack spacing={0.5}>
                      <Typography variant="h4" fontWeight={700} sx={{ letterSpacing: '-0.02em' }}>
                        Welcome back, {userName}
                      </Typography>
                      <Typography variant="body1" color="text.secondary">
                        Orchestrate messaging, storage, and live collaboration from a single control surface.
                      </Typography>
                      <Stack direction="row" spacing={1} flexWrap="wrap">
                        <Chip
                          label={fourWords}
                          icon={<GroupsIcon fontSize="small" />}
                          variant="outlined"
                          sx={{ borderColor: 'primary.main', fontWeight: 500 }}
                        />
                        <Chip
                          label="Quantum-secured"
                          icon={<SecurityIcon fontSize="small" />}
                          color="success"
                          variant="outlined"
                        />
                        <Chip
                          label={`${onlineUsers} peers live`}
                          icon={<TrendingUpIcon fontSize="small" />}
                          variant="outlined"
                          sx={{ borderColor: alpha(theme.palette.success.main, 0.4) }}
                        />
                      </Stack>
                    </Stack>
                  </Stack>
                  <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}>
                    <Button
                      variant="contained"
                      onClick={() => window.dispatchEvent(new CustomEvent('open-chat-composer', { detail: { userId } }))}
                      endIcon={<ArrowForwardIcon />}
                      sx={{
                        px: 3.5,
                        py: 1.5,
                        borderRadius: 3,
                        boxShadow: '0 20px 45px rgba(99, 102, 241, 0.25)',
                      }}
                    >
                      Launch Messenger
                    </Button>
                    <Button
                      variant="outlined"
                      color="inherit"
                      onClick={() => window.dispatchEvent(new CustomEvent('open-invite-dialog'))}
                      sx={{ px: 3.5, py: 1.5, borderRadius: 3 }}
                    >
                      Invite collaborators
                    </Button>
                  </Stack>
                </Stack>
                <Divider orientation="vertical" flexItem sx={{ display: { xs: 'none', md: 'block' }, opacity: 0.3 }} />
                <Stack spacing={2} sx={{ minWidth: { md: 240 } }}>
                  <Typography variant="overline" color="text.secondary">
                    live signal
                  </Typography>
                  <Stack spacing={1.5}>
                    {insights.map((metric) => (
                      <Box
                        key={metric.id}
                        sx={{
                          p: 2,
                          borderRadius: 3,
                          background: alpha(metric.accent, theme.palette.mode === 'dark' ? 0.2 : 0.1),
                          border: `1px solid ${alpha(metric.accent, 0.2)}`,
                        }}
                      >
                        <Stack direction="row" spacing={1.5} alignItems="center">
                          <Box
                            sx={{
                              width: 36,
                              height: 36,
                              borderRadius: '50%',
                              display: 'grid',
                              placeItems: 'center',
                              background: alpha(metric.accent, 0.25),
                              color: metric.accent,
                            }}
                          >
                            {metric.icon}
                          </Box>
                          <Box sx={{ flex: 1 }}>
                            <Typography variant="subtitle2" fontWeight={600}>
                              {metric.label}
                            </Typography>
                            <Typography variant="body2" color="text.secondary">
                              {metric.helper}
                            </Typography>
                          </Box>
                          <Typography variant="subtitle1" fontWeight={600}>
                            {metric.value}
                          </Typography>
                        </Stack>
                        {typeof metric.progress === 'number' && (
                          <LinearProgress
                            variant="determinate"
                            value={metric.progress}
                            sx={{
                              mt: 1.5,
                              height: 6,
                              borderRadius: 3,
                              backgroundColor: alpha(metric.accent, 0.15),
                              '& .MuiLinearProgress-bar': {
                                borderRadius: 3,
                                background: `linear-gradient(90deg, ${metric.accent}, ${alpha(metric.accent, 0.6)})`,
                              },
                            }}
                          />
                        )}
                      </Box>
                    ))}
                  </Stack>
                </Stack>
              </Stack>
            </Box>
          </GlassCard>
        </MotionBox>

        <MotionBox
          variants={sectionVariants}
          initial="hidden"
          animate="visible"
          custom={{ delay: 0.15 }}
        >
          <Grid container spacing={3}>
            <Grid item xs={12} lg={7}>
              <GlassCard variant="light" hover>
                <Stack spacing={3} sx={{ p: { xs: 3, md: 4 } }}>
                  <Stack direction="row" alignItems="center" justifyContent="space-between">
                    <Box>
                      <Typography variant="h6" fontWeight={600}>
                        Priority Actions
                      </Typography>
                      <Typography variant="body2" color="text.secondary">
                        The quickest routes to impact across your nodes and teams.
                      </Typography>
                    </Box>
                    <Tooltip title="View notifications">
                      <IconButton onClick={() => window.dispatchEvent(new CustomEvent('open-notifications-center'))}>
                        <Badge color="error" variant="dot">
                          <NotificationsIcon />
                        </Badge>
                      </IconButton>
                    </Tooltip>
                  </Stack>

                  <Grid container spacing={2}>
                    {quickActions.map((action) => (
                      <Grid item xs={12} md={6} key={action.id}>
                        <motion.div whileHover={{ y: -4, scale: 1.01 }} whileTap={{ scale: 0.98 }}>
                          <GlassCard
                            variant="colored"
                            hover
                            onClick={action.action}
                            sx={{
                              cursor: 'pointer',
                              background: action.gradient,
                              color: 'common.white',
                              minHeight: 150,
                            }}
                          >
                            <ButtonBase
                              focusRipple
                              sx={{
                                width: '100%',
                                textAlign: 'left',
                                display: 'flex',
                                alignItems: 'flex-start',
                                gap: 2,
                                p: 3,
                                color: 'inherit',
                              }}
                            >
                              <Box
                                sx={{
                                  width: 48,
                                  height: 48,
                                  borderRadius: '16px',
                                  background: 'rgba(255, 255, 255, 0.15)',
                                  display: 'grid',
                                  placeItems: 'center',
                                  fontSize: 24,
                                }}
                              >
                                {action.badge ? (
                                  <Badge badgeContent={action.badge} color="error">
                                    {action.icon}
                                  </Badge>
                                ) : (
                                  action.icon
                                )}
                              </Box>
                              <Box sx={{ flex: 1 }}>
                                <Typography variant="subtitle1" fontWeight={600} gutterBottom>
                                  {action.title}
                                </Typography>
                                <Typography variant="body2" sx={{ opacity: 0.9 }}>
                                  {action.description}
                                </Typography>
                              </Box>
                              <ArrowForwardIcon sx={{ opacity: 0.7 }} />
                            </ButtonBase>
                          </GlassCard>
                        </motion.div>
                      </Grid>
                    ))}
                  </Grid>
                </Stack>
              </GlassCard>
            </Grid>

            <Grid item xs={12} lg={5}>
              <GlassCard variant="light">
                <Stack spacing={3} sx={{ p: { xs: 3, md: 4 } }}>
                  <Box>
                    <Typography variant="h6" fontWeight={600}>
                      Operational Pulse
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      Live health across your decentralized footprint.
                    </Typography>
                  </Box>

                  <Stack spacing={2.5}>
                    <Box>
                      <Typography variant="overline" color="text.secondary">
                        Active collaborators
                      </Typography>
                      <Stack direction="row" spacing={2} alignItems="center" sx={{ mt: 1 }}>
                        <AvatarGroup max={5} spacing="small">
                          {['Alice', 'Bob', 'Sarah', 'Diego', 'Lin'].map((name) => (
                            <Avatar key={name} sx={{ bgcolor: alpha(theme.palette.primary.main, 0.85) }}>
                              {name[0]}
                            </Avatar>
                          ))}
                        </AvatarGroup>
                        <Typography variant="subtitle1" fontWeight={600}>
                          {onlineUsers} live
                        </Typography>
                      </Stack>
                    </Box>

                    <Divider flexItem sx={{ opacity: 0.1 }} />

                    {insights.map((metric) => (
                      <Stack key={metric.id} spacing={1.5}>
                        <Stack direction="row" alignItems="center" spacing={1.5}>
                          <Box
                            sx={{
                              width: 12,
                              height: 12,
                              borderRadius: '50%',
                              background: metric.accent,
                            }}
                          />
                          <Typography variant="subtitle2" fontWeight={600}>
                            {metric.label}
                          </Typography>
                        </Stack>
                        <Typography variant="h5" fontWeight={600}>
                          {metric.value}
                        </Typography>
                        <Typography variant="body2" color="text.secondary">
                          {metric.helper}
                        </Typography>
                        {typeof metric.progress === 'number' && (
                          <LinearProgress
                            variant="determinate"
                            value={metric.progress}
                            sx={{
                              height: 6,
                              borderRadius: 3,
                              backgroundColor: alpha(metric.accent, 0.15),
                              '& .MuiLinearProgress-bar': {
                                borderRadius: 3,
                                background: metric.accent,
                              },
                            }}
                          />
                        )}
                      </Stack>
                    ))}
                  </Stack>
                </Stack>
              </GlassCard>
            </Grid>
          </Grid>
        </MotionBox>

        <MotionStack
          direction={{ xs: 'column', lg: 'row' }}
          spacing={3}
          variants={sectionVariants}
          initial="hidden"
          animate="visible"
          custom={{ delay: 0.3 }}
        >
          <GlassCard variant="light" sx={{ flex: 1 }}>
            <Stack spacing={3} sx={{ p: { xs: 3, md: 4 } }}>
              <Box>
                <Typography variant="h6" fontWeight={600}>
                  Activity Timeline
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  The latest signal across conversations, files, and calls.
                </Typography>
              </Box>

              <Stack spacing={2.5}>
                {recentActivities.map((activity, index) => (
                  <Stack
                    key={activity.id}
                    direction="row"
                    spacing={2.5}
                    alignItems="flex-start"
                    sx={{
                      position: 'relative',
                      '&::before': index === recentActivities.length - 1 ? undefined : {
                        content: '""',
                        position: 'absolute',
                        left: 26,
                        top: 44,
                        bottom: -12,
                        width: 2,
                        background: alpha(activity.tone, 0.2),
                      },
                    }}
                  >
                    <Box
                      sx={{
                        width: 52,
                        height: 52,
                        borderRadius: '18px',
                        background: alpha(activity.tone, 0.15),
                        display: 'grid',
                        placeItems: 'center',
                        color: activity.tone,
                        flexShrink: 0,
                      }}
                    >
                      {activity.icon}
                    </Box>
                    <Stack spacing={0.75}>
                      <Typography variant="subtitle1" fontWeight={600}>
                        {activity.title}
                      </Typography>
                      <Typography variant="body2" color="text.secondary">
                        {activity.subtitle}
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        {formatDistanceToNow(activity.timestamp, { addSuffix: true })}
                      </Typography>
                    </Stack>
                  </Stack>
                ))}
              </Stack>
            </Stack>
          </GlassCard>

          <Stack spacing={3} sx={{ width: { xs: '100%', lg: '28%' } }}>
            {workspaceHighlights.map((workspace) => (
              <motion.div key={workspace.id} whileHover={{ y: -4 }}>
                <GlassCard variant="dark" hover onClick={workspace.action} sx={{ cursor: 'pointer' }}>
                  <Stack spacing={2} sx={{ p: 3 }}>
                    <Stack direction="row" spacing={1.5} alignItems="center">
                      <Box
                        sx={{
                          width: 40,
                          height: 40,
                          borderRadius: '14px',
                          background: alpha(workspace.accent, 0.18),
                          display: 'grid',
                          placeItems: 'center',
                          color: workspace.accent,
                        }}
                      >
                        {workspace.icon}
                      </Box>
                      <Typography variant="subtitle1" fontWeight={600}>
                        {workspace.title}
                      </Typography>
                    </Stack>
                    <Typography variant="body2" color="text.secondary">
                      {workspace.description}
                    </Typography>
                    <Button
                      variant="text"
                      color="inherit"
                      endIcon={<ArrowForwardIcon />}
                      sx={{ alignSelf: 'flex-start', fontWeight: 600 }}
                    >
                      {workspace.actionLabel}
                    </Button>
                  </Stack>
                </GlassCard>
              </motion.div>
            ))}
          </Stack>
        </MotionStack>

        <MotionBox
          variants={sectionVariants}
          initial="hidden"
          animate="visible"
          custom={{ delay: 0.45 }}
        >
          <GlassCard variant="light">
            <Stack spacing={3} sx={{ p: { xs: 3, md: 4 } }}>
              <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} alignItems={{ xs: 'flex-start', md: 'center' }} justifyContent="space-between">
                <Box>
                  <Typography variant="h6" fontWeight={600}>
                    Focus Canvas
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Dive straight into live conversations or secure storage without leaving the dashboard.
                  </Typography>
                </Box>
                <Tabs
                  value={focusTab}
                  onChange={(_, value) => setFocusTab(value)}
                  variant="standard"
                  sx={{
                    '& .MuiTab-root': {
                      textTransform: 'none',
                      fontWeight: 600,
                    },
                  }}
                >
                  <Tab value="messages" icon={<ChatIcon />} iconPosition="start" label="Messenger" />
                  <Tab value="storage" icon={<FolderIcon />} iconPosition="start" label="Storage" />
                </Tabs>
              </Stack>

              <Box
                sx={{
                  borderRadius: 3,
                  overflow: 'hidden',
                  border: `1px solid ${alpha(theme.palette.divider, 0.4)}`,
                  backgroundColor: alpha(theme.palette.background.paper, 0.6),
                  height: { xs: 420, md: 460 },
                  display: 'flex',
                }}
              >
                {focusTab === 'messages' ? (
                  <ChatInterface
                    chatId="communitas-general"
                    chatName="Core Launch Team"
                    chatType="group"
                    participants={onlineUsers}
                    onStartCall={(type) => window.dispatchEvent(new CustomEvent('start-call', { detail: { type } }))}
                  />
                ) : (
                  <FileManager
                    organizationId="org-demo"
                    onFileSelect={(file) => window.dispatchEvent(new CustomEvent('storage:file-select', { detail: file }))}
                  />
                )}
              </Box>
            </Stack>
          </GlassCard>
        </MotionBox>
      </Stack>
    </Box>
  );
};

export default UnifiedDashboard;
