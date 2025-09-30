import React, { useState, useEffect } from 'react';
import {
  Box,
  Grid,
  Card,
  CardContent,
  Typography,
  Stack,
  Avatar,
  Chip,
  IconButton,
  Divider,
  List,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Paper,
  LinearProgress,
  Badge,
  Tooltip,
  useTheme,
} from '@mui/material';

// Import modern UI components
import { GlassCard } from '../ui/GlassCard';
import { ModernButton } from '../ui/ModernButton';
import { ModernLoader } from '../ui/ModernLoader';
import {
  Storage as StorageIcon,
  Web as WebsiteIcon,
  Security as LockIcon,
  Message as MessageIcon,
  VideoCall as VideoCallIcon,
  Call as CallIcon,
  Description as FileIcon,
  AccessTime as RecentIcon,
  Folder as FolderIcon,
  People as GroupIcon,
  Business as OrgIcon,
  Upload as UploadIcon,
  Download as DownloadIcon,
  Edit as EditIcon,
  Share as ShareIcon,
  Add as AddIcon,
  TrendingUp as ActivityIcon,
  CloudDone as SyncedIcon,
} from '@mui/icons-material';
import { motion } from 'framer-motion';
import { useNavigation } from '../../contexts/NavigationContext';
import { useAuth } from '../../contexts/AuthContext';
import { EndpointStatusDisplay } from '../network/EndpointStatusDisplay';

interface RecentActivity {
  id: string;
  type: 'message' | 'file' | 'call' | 'upload' | 'download' | 'edit';
  title: string;
  subtitle: string;
  timestamp: Date;
  avatar?: string;
  entityType?: string;
  entityId?: string;
}

interface StorageStats {
  websiteFiles: number;
  dataFiles: number;
  totalSize: string;
  lastSync: Date;
}

const MotionCard = motion(GlassCard);
const MotionBox = motion(Box);

export const PersonalHomeDashboard: React.FC = () => {
  const theme = useTheme();
  const { selectEntity, switchToPersonal } = useNavigation();
  const { authState } = useAuth();
  const [recentActivity, setRecentActivity] = useState<RecentActivity[]>([]);
  const [storageStats, setStorageStats] = useState<StorageStats>({
    websiteFiles: 0,
    dataFiles: 0,
    totalSize: '0 MB',
    lastSync: new Date(),
  });

  // Load real recent activity from local storage and services
  useEffect(() => {
    const loadRecentActivity = async () => {
      try {
        // Try to load real activity data from localStorage
        const storedActivity = localStorage.getItem('communitas-recent-activity');
        if (storedActivity) {
          const parsed = JSON.parse(storedActivity);
          const validActivity = parsed.map((item: any) => ({
            ...item,
            timestamp: new Date(item.timestamp)
          })).filter((item: any) => item.id && item.type && item.title);
          setRecentActivity(validActivity.slice(0, 10)); // Show last 10 items
        } else {
          // No stored activity, start with empty state
          setRecentActivity([]);
        }
        // TODO: In future, integrate with messaging/file APIs for real-time data
      } catch (error) {
        console.log('No recent activity data available, starting fresh');
        setRecentActivity([]);
      }
    };
    loadRecentActivity();
  }, []);

  // Load real storage statistics  
  useEffect(() => {
    const loadStorageStats = async () => {
      try {
        // TODO: Replace with real storage API calls to saorsa-core
        // For now, provide safe defaults until real data is available
        setStorageStats({
          websiteFiles: 0,
          dataFiles: 0, 
          totalSize: 'No data available',
          lastSync: new Date(),
        });
      } catch (error) {
        console.log('Storage stats not available');
      }
    };
    loadStorageStats();
  }, []);

  const getActivityIcon = (type: string) => {
    switch (type) {
      case 'message':
        return <MessageIcon />;
      case 'file':
      case 'edit':
        return <FileIcon />;
      case 'call':
        return <VideoCallIcon />;
      case 'upload':
        return <UploadIcon />;
      case 'download':
        return <DownloadIcon />;
      default:
        return <ActivityIcon />;
    }
  };

  const getActivityColor = (type: string) => {
    switch (type) {
      case 'message':
        return 'primary';
      case 'file':
      case 'edit':
        return 'info';
      case 'call':
        return 'success';
      case 'upload':
        return 'secondary';
      case 'download':
        return 'warning';
      default:
        return 'default';
    }
  };

  const handleStorageClick = (area: 'website' | 'data') => {
    // Navigate to personal context first, then show storage
    switchToPersonal();
    setTimeout(() => {
      // Dispatch custom event to open storage workspace with specific area
      window.dispatchEvent(new CustomEvent('open-storage-workspace', { 
        detail: { area } 
      }));
    }, 100);
  };

  const formatTimeAgo = (date: Date) => {
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMins / 60);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    return date.toLocaleDateString();
  };

  return (
    <Box sx={{ p: 3, maxWidth: 1200, mx: 'auto' }}>
      {/* Welcome Header */}
      <MotionBox
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        sx={{ mb: 4 }}
      >
        <Stack direction="row" alignItems="center" spacing={2} sx={{ mb: 2 }}>
          <Avatar
            sx={{
              width: 56,
              height: 56,
              bgcolor: 'primary.main',
              fontSize: '1.5rem',
              fontWeight: 600,
            }}
          >
            {authState.user?.name.charAt(0).toUpperCase() || 'U'}
          </Avatar>
          <Box sx={{ flex: 1 }}>
            <Typography variant="h4" fontWeight={600} gutterBottom>
              Welcome back, {authState.user?.name || 'User'}!
            </Typography>
            <Typography variant="body1" color="text.secondary">
              {authState.user?.fourWordAddress || 'Local Mode'}
            </Typography>
          </Box>
        </Stack>

        {/* Network Status */}
        <Box sx={{ mb: 2 }}>
          <EndpointStatusDisplay />
        </Box>
      </MotionBox>

      <Grid container spacing={3}>
        {/* Storage Quick Access */}
        <Grid size={{ xs: 12, md: 6 }}>
          <MotionCard
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.1 }}
            elevation={2}
          >
            <CardContent>
              <Stack direction="row" alignItems="center" spacing={2} sx={{ mb: 2 }}>
                <StorageIcon color="primary" />
                <Typography variant="h6" fontWeight={600}>
                  My Storage Disks
                </Typography>
                <Chip label="2 Disks" size="small" color="primary" />
              </Stack>

              <Stack spacing={2}>
                {/* Website Storage */}
                <Paper
                  elevation={1}
                  sx={{
                    p: 2,
                    border: '1px solid',
                    borderColor: 'divider',
                    cursor: 'pointer',
                    '&:hover': {
                      borderColor: 'primary.main',
                      bgcolor: 'action.hover',
                    },
                  }}
                  onClick={() => handleStorageClick('website')}
                >
                  <Stack direction="row" alignItems="center" spacing={2}>
                    <WebsiteIcon sx={{ color: 'info.main' }} />
                    <Box sx={{ flex: 1 }}>
                      <Typography variant="subtitle2" fontWeight={600}>
                        Website Storage
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        Plain markdown files • {storageStats.websiteFiles} files
                      </Typography>
                    </Box>
                    <Chip label="Public" size="small" color="info" variant="outlined" />
                  </Stack>
                </Paper>

                {/* Data Storage */}
                <Paper
                  elevation={1}
                  sx={{
                    p: 2,
                    border: '1px solid',
                    borderColor: 'divider',
                    cursor: 'pointer',
                    '&:hover': {
                      borderColor: 'secondary.main',
                      bgcolor: 'action.hover',
                    },
                  }}
                  onClick={() => handleStorageClick('data')}
                >
                  <Stack direction="row" alignItems="center" spacing={2}>
                    <LockIcon sx={{ color: 'secondary.main' }} />
                    <Box sx={{ flex: 1 }}>
                      <Typography variant="subtitle2" fontWeight={600}>
                        Data Storage
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        Encrypted files • {storageStats.dataFiles} files
                      </Typography>
                    </Box>
                    <Chip label="Encrypted" size="small" color="secondary" variant="outlined" />
                  </Stack>
                </Paper>
              </Stack>

              <Divider sx={{ my: 2 }} />
              
              <Stack direction="row" alignItems="center" spacing={1}>
                <SyncedIcon fontSize="small" color="success" />
                <Typography variant="caption" color="text.secondary">
                  Last synced {formatTimeAgo(storageStats.lastSync)}
                </Typography>
                <Box sx={{ flexGrow: 1 }} />
                <Typography variant="caption" color="text.secondary">
                  {storageStats.totalSize} used
                </Typography>
              </Stack>
            </CardContent>
          </MotionCard>
        </Grid>

        {/* Recent Activity */}
        <Grid size={{ xs: 12, md: 6 }}>
          <MotionCard
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            elevation={2}
          >
            <CardContent>
              <Stack direction="row" alignItems="center" spacing={2} sx={{ mb: 2 }}>
                <RecentIcon color="primary" />
                <Typography variant="h6" fontWeight={600}>
                  Recent Activity
                </Typography>
                <Badge badgeContent={recentActivity.length} color="primary">
                  <Box />
                </Badge>
              </Stack>

              <List dense>
                {recentActivity.map((activity, index) => (
                  <ListItemButton
                    key={activity.id}
                    sx={{
                      borderRadius: 1,
                      mb: 1,
                      border: '1px solid transparent',
                      '&:hover': {
                        border: '1px solid',
                        borderColor: 'primary.main',
                      },
                    }}
                  >
                    <ListItemIcon>
                      {activity.avatar ? (
                        <Avatar sx={{ width: 32, height: 32 }}>
                          {activity.avatar}
                        </Avatar>
                      ) : (
                        <Chip
                          icon={getActivityIcon(activity.type)}
                          label=""
                          size="small"
                          color={getActivityColor(activity.type) as any}
                          sx={{ width: 32, height: 32, '& .MuiChip-label': { display: 'none' } }}
                        />
                      )}
                    </ListItemIcon>
                    <ListItemText
                      primary={activity.title}
                      secondary={
                        <Stack direction="row" alignItems="center" spacing={1}>
                          <Typography variant="caption" color="text.secondary">
                            {activity.subtitle}
                          </Typography>
                          <Typography variant="caption" color="text.secondary">
                            • {formatTimeAgo(activity.timestamp)}
                          </Typography>
                        </Stack>
                      }
                    />
                  </ListItemButton>
                ))}
              </List>

              {recentActivity.length === 0 && (
                <Box
                  sx={{
                    textAlign: 'center',
                    py: 4,
                    color: 'text.secondary',
                  }}
                >
                  <ActivityIcon sx={{ fontSize: 48, mb: 1, opacity: 0.5 }} />
                  <Typography variant="body2">
                    No recent activity
                  </Typography>
                </Box>
              )}
            </CardContent>
          </MotionCard>
        </Grid>

        {/* Quick Actions */}
        <Grid size={12}>
          <MotionCard
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.3 }}
            elevation={2}
          >
            <CardContent>
              <Typography variant="h6" fontWeight={600} gutterBottom>
                Quick Actions
              </Typography>
              <Stack direction="row" spacing={2} flexWrap="wrap">
                <ModernButton
                  variant="contained"
                  gradient={true}
                  startIcon={<AddIcon />}
                  onClick={() => handleStorageClick('website')}
                >
                  Create Website Page
                </ModernButton>
                <ModernButton
                  variant="contained"
                  gradient={true}
                  startIcon={<UploadIcon />}
                  onClick={() => handleStorageClick('data')}
                >
                  Upload Files
                </ModernButton>
                <ModernButton
                  variant="contained"
                  gradient={true}
                  startIcon={<MessageIcon />}
                  onClick={() => {
                    // TODO: Navigate to messages
                  }}
                >
                  New Message
                </ModernButton>
                <ModernButton
                  variant="contained"
                  gradient={true}
                  startIcon={<VideoCallIcon />}
                  onClick={() => {
                    // TODO: Start video call
                  }}
                >
                  Start Call
                </ModernButton>
              </Stack>
            </CardContent>
          </MotionCard>
        </Grid>
      </Grid>
    </Box>
  );
};