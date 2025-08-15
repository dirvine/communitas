import React, { useState, useEffect } from 'react';
import {
  Box,
  Grid,
  Card,
  CardContent,
  CardActions,
  CardHeader,
  Typography,
  Button,
  IconButton,
  Avatar,
  AvatarGroup,
  Chip,
  Stack,
  Paper,
  LinearProgress,
  Tab,
  Tabs,
  Badge,
  Tooltip,
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  Divider,
  TextField,
  InputAdornment,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  FormControl,
  InputLabel,
  Select,
  SelectChangeEvent,
  CircularProgress,
  Skeleton,
  Alert,
  Fade,
  Zoom,
  Grow,
} from '@mui/material';
import {
  Business as BusinessIcon,
  People as PeopleIcon,
  Folder as FolderIcon,
  Group as GroupIcon,
  Storage as StorageIcon,
  TrendingUp as TrendingUpIcon,
  Schedule as ScheduleIcon,
  Assignment as TaskIcon,
  Phone as PhoneIcon,
  Videocam as VideocamIcon,
  Chat as ChatIcon,
  MoreVert as MoreVertIcon,
  Add as AddIcon,
  Search as SearchIcon,
  FilterList as FilterIcon,
  GridView as GridViewIcon,
  ViewList as ListViewIcon,
  CloudUpload as UploadIcon,
  CloudDownload as DownloadIcon,
  Share as ShareIcon,
  Edit as EditIcon,
  Delete as DeleteIcon,
  Star as StarIcon,
  StarBorder as StarBorderIcon,
  Notifications as NotificationsIcon,
  Settings as SettingsIcon,
  Description as FileIcon,
  PlayCircle as PlayIcon,
  PauseCircle as PauseIcon,
  CheckCircle as CheckCircleIcon,
  Error as ErrorIcon,
  Warning as WarningIcon,
  Info as InfoIcon,
  Speed as SpeedIcon,
  AccountTree as AccountTreeIcon,
  Timeline as TimelineIcon,
  Assessment as AssessmentIcon,
  Forum as ForumIcon,
  FolderOpen as FolderOpenIcon,
  CreateNewFolder as CreateNewFolderIcon,
  GroupAdd as GroupAddIcon,
  PersonAdd as PersonAddIcon,
  Link as LinkIcon,
  QrCode as QrCodeIcon,
  ContentCopy as CopyIcon,
  Refresh as RefreshIcon,
  ExpandMore as ExpandMoreIcon,
  NavigateNext as NavigateNextIcon,
  Dashboard as DashboardIcon,
  CalendarMonth as CalendarIcon,
  AttachFile as AttachFileIcon,
  Image as ImageIcon,
  VideoLibrary as VideoIcon,
  AudioFile as AudioIcon,
  Code as CodeIcon,
  Archive as ArchiveIcon,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { 
  Organization, 
  Project, 
  Group, 
  OrganizationHierarchy,
  Member,
  CreateProjectRequest,
  CreateGroupRequest,
} from '../../types/organization';
import { motion, AnimatePresence } from 'framer-motion';

interface OrganizationViewProps {
  organization: Organization;
  hierarchy: OrganizationHierarchy;
  onNavigate: (type: string, entity: any) => void;
  onCall: (entityType: string, entityId: string, callType: 'voice' | 'video') => void;
  onRefresh: () => void;
}

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

const TabPanel: React.FC<TabPanelProps> = ({ children, value, index }) => (
  <div hidden={value !== index} style={{ height: '100%' }}>
    {value === index && <Box sx={{ height: '100%', p: 3 }}>{children}</Box>}
  </div>
);

const MotionCard = motion(Card);

export const OrganizationView: React.FC<OrganizationViewProps> = ({
  organization,
  hierarchy,
  onNavigate,
  onCall,
  onRefresh,
}) => {
  const [activeTab, setActiveTab] = useState(0);
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [searchQuery, setSearchQuery] = useState('');
  const [filterType, setFilterType] = useState<'all' | 'projects' | 'groups'>('all');
  const [sortBy, setSortBy] = useState<'name' | 'date' | 'activity'>('activity');
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [createType, setCreateType] = useState<'project' | 'group'>('project');
  const [loading, setLoading] = useState(false);
  const [menuAnchor, setMenuAnchor] = useState<null | HTMLElement>(null);
  const [selectedEntity, setSelectedEntity] = useState<Project | Group | null>(null);
  const [starred, setStarred] = useState<Set<string>>(new Set());

  // Mock data for recent activity
  const recentActivity = [
    { id: 1, user: 'Alice', action: 'created project', target: 'Q1 Planning', time: '5 min ago', type: 'project' },
    { id: 2, user: 'Bob', action: 'uploaded files to', target: 'Design Assets', time: '1 hour ago', type: 'file' },
    { id: 3, user: 'Charlie', action: 'started a call in', target: 'Engineering Team', time: '2 hours ago', type: 'call' },
    { id: 4, user: 'Diana', action: 'joined', target: 'Marketing Group', time: '3 hours ago', type: 'member' },
    { id: 5, user: 'Eve', action: 'completed task in', target: 'Website Redesign', time: '5 hours ago', type: 'task' },
  ];

  // Mock data for recent files
  const recentFiles = [
    { id: 1, name: 'Product Roadmap.pdf', size: '2.3 MB', modified: '1 hour ago', type: 'pdf', owner: 'Alice' },
    { id: 2, name: 'Team Photo.jpg', size: '4.1 MB', modified: '3 hours ago', type: 'image', owner: 'Bob' },
    { id: 3, name: 'Budget 2024.xlsx', size: '156 KB', modified: '1 day ago', type: 'spreadsheet', owner: 'Charlie' },
    { id: 4, name: 'Meeting Recording.mp4', size: '245 MB', modified: '2 days ago', type: 'video', owner: 'Diana' },
    { id: 5, name: 'Source Code.zip', size: '34 MB', modified: '3 days ago', type: 'archive', owner: 'Eve' },
  ];

  const handleCreateEntity = async () => {
    setLoading(true);
    try {
      if (createType === 'project') {
        const request: CreateProjectRequest = {
          name: 'New Project',
          organization_id: organization.id,
          priority: 'medium',
          initial_storage_gb: 5,
        };
        await invoke('create_project_dht', { request });
      } else {
        const request: CreateGroupRequest = {
          name: 'New Group',
          organization_id: organization.id,
        };
        await invoke('create_group_dht', { request });
      }
      setCreateDialogOpen(false);
      onRefresh();
    } catch (error) {
      console.error('Failed to create entity:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleStarToggle = (entityId: string) => {
    const newStarred = new Set(starred);
    if (newStarred.has(entityId)) {
      newStarred.delete(entityId);
    } else {
      newStarred.add(entityId);
    }
    setStarred(newStarred);
  };

  const getFileIcon = (type: string) => {
    switch (type) {
      case 'pdf': return <FileIcon color="error" />;
      case 'image': return <ImageIcon color="success" />;
      case 'video': return <VideoIcon color="primary" />;
      case 'spreadsheet': return <AssessmentIcon color="warning" />;
      case 'archive': return <ArchiveIcon color="action" />;
      default: return <FileIcon />;
    }
  };

  const getActivityIcon = (type: string) => {
    switch (type) {
      case 'project': return <FolderIcon color="primary" />;
      case 'file': return <AttachFileIcon color="action" />;
      case 'call': return <PhoneIcon color="success" />;
      case 'member': return <PersonAddIcon color="secondary" />;
      case 'task': return <TaskIcon color="warning" />;
      default: return <InfoIcon />;
    }
  };

  const renderOverviewTab = () => (
    <Grid container spacing={3}>
      {/* Statistics Cards */}
      <Grid item xs={12}>
        <Grid container spacing={2}>
          <Grid item xs={12} sm={6} md={3}>
            <MotionCard
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3 }}
              elevation={2}
            >
              <CardContent>
                <Stack direction="row" justifyContent="space-between" alignItems="center">
                  <Box>
                    <Typography color="text.secondary" variant="body2">
                      Total Projects
                    </Typography>
                    <Typography variant="h4">
                      {hierarchy.projects.length}
                    </Typography>
                  </Box>
                  <Avatar sx={{ bgcolor: 'primary.main' }}>
                    <FolderIcon />
                  </Avatar>
                </Stack>
              </CardContent>
            </MotionCard>
          </Grid>

          <Grid item xs={12} sm={6} md={3}>
            <MotionCard
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3, delay: 0.1 }}
              elevation={2}
            >
              <CardContent>
                <Stack direction="row" justifyContent="space-between" alignItems="center">
                  <Box>
                    <Typography color="text.secondary" variant="body2">
                      Total Groups
                    </Typography>
                    <Typography variant="h4">
                      {hierarchy.groups.length}
                    </Typography>
                  </Box>
                  <Avatar sx={{ bgcolor: 'secondary.main' }}>
                    <GroupIcon />
                  </Avatar>
                </Stack>
              </CardContent>
            </MotionCard>
          </Grid>

          <Grid item xs={12} sm={6} md={3}>
            <MotionCard
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3, delay: 0.2 }}
              elevation={2}
            >
              <CardContent>
                <Stack direction="row" justifyContent="space-between" alignItems="center">
                  <Box>
                    <Typography color="text.secondary" variant="body2">
                      Active Members
                    </Typography>
                    <Typography variant="h4">
                      {hierarchy.total_members}
                    </Typography>
                  </Box>
                  <Avatar sx={{ bgcolor: 'success.main' }}>
                    <PeopleIcon />
                  </Avatar>
                </Stack>
              </CardContent>
            </MotionCard>
          </Grid>

          <Grid item xs={12} sm={6} md={3}>
            <MotionCard
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3, delay: 0.3 }}
              elevation={2}
            >
              <CardContent>
                <Stack direction="row" justifyContent="space-between" alignItems="center">
                  <Box>
                    <Typography color="text.secondary" variant="body2">
                      Storage Used
                    </Typography>
                    <Typography variant="h4">
                      {hierarchy.total_storage_used_gb.toFixed(1)}
                      <Typography component="span" variant="body1"> GB</Typography>
                    </Typography>
                  </Box>
                  <Avatar sx={{ bgcolor: 'warning.main' }}>
                    <StorageIcon />
                  </Avatar>
                </Stack>
              </CardContent>
            </MotionCard>
          </Grid>
        </Grid>
      </Grid>

      {/* Quick Actions */}
      <Grid item xs={12}>
        <Paper elevation={1} sx={{ p: 2 }}>
          <Typography variant="h6" gutterBottom>
            Quick Actions
          </Typography>
          <Stack direction="row" spacing={2} sx={{ flexWrap: 'wrap', gap: 1 }}>
            <Button
              variant="contained"
              startIcon={<CreateNewFolderIcon />}
              onClick={() => {
                setCreateType('project');
                setCreateDialogOpen(true);
              }}
            >
              New Project
            </Button>
            <Button
              variant="contained"
              color="secondary"
              startIcon={<GroupAddIcon />}
              onClick={() => {
                setCreateType('group');
                setCreateDialogOpen(true);
              }}
            >
              New Group
            </Button>
            <Button
              variant="outlined"
              startIcon={<PersonAddIcon />}
              onClick={() => onNavigate('invite_members', organization)}
            >
              Invite Members
            </Button>
            <Button
              variant="outlined"
              startIcon={<PhoneIcon />}
              color="success"
              onClick={() => onCall('organization', organization.id, 'voice')}
            >
              Start Call
            </Button>
            <Button
              variant="outlined"
              startIcon={<VideocamIcon />}
              color="primary"
              onClick={() => onCall('organization', organization.id, 'video')}
            >
              Video Meeting
            </Button>
          </Stack>
        </Paper>
      </Grid>

      {/* Recent Activity & Files */}
      <Grid item xs={12} md={6}>
        <Paper elevation={1} sx={{ p: 2, height: 400, overflow: 'hidden' }}>
          <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 2 }}>
            <Typography variant="h6">
              Recent Activity
            </Typography>
            <IconButton size="small">
              <RefreshIcon />
            </IconButton>
          </Stack>
          <Box sx={{ height: 'calc(100% - 48px)', overflow: 'auto' }}>
            <Stack spacing={2}>
              {recentActivity.map((activity) => (
                <Fade in key={activity.id}>
                  <Stack direction="row" spacing={2} alignItems="center">
                    <Avatar sx={{ width: 32, height: 32, bgcolor: 'primary.light' }}>
                      {getActivityIcon(activity.type)}
                    </Avatar>
                    <Box sx={{ flexGrow: 1 }}>
                      <Typography variant="body2">
                        <strong>{activity.user}</strong> {activity.action} <strong>{activity.target}</strong>
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        {activity.time}
                      </Typography>
                    </Box>
                  </Stack>
                </Fade>
              ))}
            </Stack>
          </Box>
        </Paper>
      </Grid>

      <Grid item xs={12} md={6}>
        <Paper elevation={1} sx={{ p: 2, height: 400, overflow: 'hidden' }}>
          <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 2 }}>
            <Typography variant="h6">
              Recent Files
            </Typography>
            <Button size="small" endIcon={<NavigateNextIcon />}>
              View All
            </Button>
          </Stack>
          <Box sx={{ height: 'calc(100% - 48px)', overflow: 'auto' }}>
            <Stack spacing={1}>
              {recentFiles.map((file) => (
                <Paper
                  key={file.id}
                  variant="outlined"
                  sx={{ 
                    p: 1.5,
                    cursor: 'pointer',
                    '&:hover': { bgcolor: 'action.hover' },
                    transition: 'all 0.2s',
                  }}
                >
                  <Stack direction="row" spacing={2} alignItems="center">
                    {getFileIcon(file.type)}
                    <Box sx={{ flexGrow: 1, minWidth: 0 }}>
                      <Typography variant="body2" noWrap>
                        {file.name}
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        {file.size} • {file.modified} • {file.owner}
                      </Typography>
                    </Box>
                    <Stack direction="row" spacing={0.5}>
                      <IconButton size="small">
                        <DownloadIcon fontSize="small" />
                      </IconButton>
                      <IconButton size="small">
                        <ShareIcon fontSize="small" />
                      </IconButton>
                    </Stack>
                  </Stack>
                </Paper>
              ))}
            </Stack>
          </Box>
        </Paper>
      </Grid>
    </Grid>
  );

  const renderProjectCard = (project: Project) => (
    <Grid item xs={12} sm={6} md={4} key={project.id}>
      <MotionCard
        whileHover={{ scale: 1.02 }}
        elevation={2}
        sx={{ 
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          position: 'relative',
          overflow: 'visible',
        }}
      >
        <CardHeader
          avatar={
            <Avatar sx={{ bgcolor: 'primary.main' }}>
              <FolderIcon />
            </Avatar>
          }
          action={
            <Stack direction="row" spacing={0.5}>
              <IconButton 
                size="small"
                onClick={() => handleStarToggle(project.id)}
              >
                {starred.has(project.id) ? <StarIcon color="warning" /> : <StarBorderIcon />}
              </IconButton>
              <IconButton
                size="small"
                onClick={(e) => {
                  setSelectedEntity(project);
                  setMenuAnchor(e.currentTarget);
                }}
              >
                <MoreVertIcon />
              </IconButton>
            </Stack>
          }
          title={project.name}
          subheader={
            <Stack direction="row" spacing={1}>
              <Chip 
                label={project.status} 
                size="small"
                color={project.status === 'active' ? 'success' : 'default'}
              />
              <Chip 
                label={project.priority} 
                size="small"
                color={project.priority === 'high' || project.priority === 'critical' ? 'error' : 'default'}
              />
            </Stack>
          }
        />
        <CardContent sx={{ flexGrow: 1 }}>
          <Typography variant="body2" color="text.secondary" gutterBottom>
            {project.description || 'No description available'}
          </Typography>
          
          <Box sx={{ mt: 2 }}>
            <Stack direction="row" justifyContent="space-between" sx={{ mb: 1 }}>
              <Typography variant="caption">Storage</Typography>
              <Typography variant="caption">
                {project.storage_quota.used_gb.toFixed(1)} / {project.storage_quota.allocated_gb} GB
              </Typography>
            </Stack>
            <LinearProgress 
              variant="determinate" 
              value={(project.storage_quota.used_gb / project.storage_quota.allocated_gb) * 100}
              sx={{ mb: 2 }}
            />
          </Box>

          {project.deadline && (
            <Stack direction="row" spacing={1} alignItems="center" sx={{ mb: 1 }}>
              <ScheduleIcon fontSize="small" color="action" />
              <Typography variant="caption" color="text.secondary">
                Due: {new Date(project.deadline).toLocaleDateString()}
              </Typography>
            </Stack>
          )}

          <AvatarGroup max={4} sx={{ justifyContent: 'flex-start' }}>
            {project.members.slice(0, 4).map((member) => (
              <Tooltip key={member.user_id} title={member.display_name}>
                <Avatar sx={{ width: 28, height: 28 }}>
                  {member.display_name[0]}
                </Avatar>
              </Tooltip>
            ))}
          </AvatarGroup>
        </CardContent>
        <CardActions>
          <Button 
            size="small" 
            onClick={() => onNavigate('project', project)}
          >
            Open
          </Button>
          <Button 
            size="small" 
            startIcon={<PhoneIcon />}
            onClick={() => onCall('project', project.id, 'voice')}
          >
            Call
          </Button>
          <Button 
            size="small" 
            startIcon={<ChatIcon />}
          >
            Chat
          </Button>
        </CardActions>
      </MotionCard>
    </Grid>
  );

  const renderGroupCard = (group: Group) => (
    <Grid item xs={12} sm={6} md={4} key={group.id}>
      <MotionCard
        whileHover={{ scale: 1.02 }}
        elevation={2}
        sx={{ 
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        <CardHeader
          avatar={
            <Avatar sx={{ bgcolor: 'secondary.main' }}>
              <GroupIcon />
            </Avatar>
          }
          action={
            <Stack direction="row" spacing={0.5}>
              <IconButton 
                size="small"
                onClick={() => handleStarToggle(group.id)}
              >
                {starred.has(group.id) ? <StarIcon color="warning" /> : <StarBorderIcon />}
              </IconButton>
              <IconButton
                size="small"
                onClick={(e) => {
                  setSelectedEntity(group);
                  setMenuAnchor(e.currentTarget);
                }}
              >
                <MoreVertIcon />
              </IconButton>
            </Stack>
          }
          title={group.name}
          subheader={
            <Stack direction="row" spacing={1}>
              <Chip 
                icon={<ChatIcon />}
                label="Chat Only" 
                size="small"
                color="info"
              />
              <Chip 
                label={`${group.members.length} members`} 
                size="small"
              />
            </Stack>
          }
        />
        <CardContent sx={{ flexGrow: 1 }}>
          <Typography variant="body2" color="text.secondary" gutterBottom>
            {group.description || 'Chat-only group for team communication'}
          </Typography>
          
          <Stack spacing={1} sx={{ mt: 2 }}>
            <Stack direction="row" spacing={1} alignItems="center">
              <Badge color="success" variant="dot">
                <ChatIcon fontSize="small" color="action" />
              </Badge>
              <Typography variant="caption">
                Active conversations
              </Typography>
            </Stack>
            
            <Stack direction="row" spacing={1} alignItems="center">
              <VideocamIcon fontSize="small" color="action" />
              <Typography variant="caption">
                Video calls {group.chat_settings.allow_video_calls ? 'enabled' : 'disabled'}
              </Typography>
            </Stack>
          </Stack>

          <Box sx={{ mt: 2 }}>
            <AvatarGroup max={4} sx={{ justifyContent: 'flex-start' }}>
              {group.members.slice(0, 4).map((member) => (
                <Tooltip key={member.user_id} title={member.display_name}>
                  <Avatar sx={{ width: 28, height: 28 }}>
                    {member.display_name[0]}
                  </Avatar>
                </Tooltip>
              ))}
            </AvatarGroup>
          </Box>
        </CardContent>
        <CardActions>
          <Button 
            size="small" 
            onClick={() => onNavigate('group', group)}
          >
            Open Chat
          </Button>
          <Button 
            size="small" 
            color="success"
            startIcon={<PhoneIcon />}
            onClick={() => onCall('group', group.id, 'voice')}
          >
            Call
          </Button>
          <Button 
            size="small" 
            color="primary"
            startIcon={<VideocamIcon />}
            onClick={() => onCall('group', group.id, 'video')}
          >
            Video
          </Button>
        </CardActions>
      </MotionCard>
    </Grid>
  );

  const renderProjectsTab = () => (
    <Box>
      {/* Search and Filter Bar */}
      <Paper elevation={1} sx={{ p: 2, mb: 3 }}>
        <Stack direction="row" spacing={2} alignItems="center">
          <TextField
            size="small"
            placeholder="Search projects..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            sx={{ flexGrow: 1 }}
            InputProps={{
              startAdornment: (
                <InputAdornment position="start">
                  <SearchIcon />
                </InputAdornment>
              ),
            }}
          />
          
          <FormControl size="small" sx={{ minWidth: 120 }}>
            <InputLabel>Sort By</InputLabel>
            <Select
              value={sortBy}
              onChange={(e: SelectChangeEvent) => setSortBy(e.target.value as any)}
              label="Sort By"
            >
              <MenuItem value="name">Name</MenuItem>
              <MenuItem value="date">Date</MenuItem>
              <MenuItem value="activity">Activity</MenuItem>
            </Select>
          </FormControl>

          <IconButton 
            onClick={() => setViewMode(viewMode === 'grid' ? 'list' : 'grid')}
          >
            {viewMode === 'grid' ? <ListViewIcon /> : <GridViewIcon />}
          </IconButton>

          <Button
            variant="contained"
            startIcon={<AddIcon />}
            onClick={() => {
              setCreateType('project');
              setCreateDialogOpen(true);
            }}
          >
            New Project
          </Button>
        </Stack>
      </Paper>

      {/* Projects Grid/List */}
      <Grid container spacing={3}>
        {hierarchy.projects
          .filter(p => p.name.toLowerCase().includes(searchQuery.toLowerCase()))
          .map(renderProjectCard)}
      </Grid>

      {hierarchy.projects.length === 0 && (
        <Paper sx={{ p: 4, textAlign: 'center' }}>
          <FolderOpenIcon sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
          <Typography variant="h6" gutterBottom>
            No projects yet
          </Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
            Create your first project to start collaborating
          </Typography>
          <Button
            variant="contained"
            startIcon={<CreateNewFolderIcon />}
            onClick={() => {
              setCreateType('project');
              setCreateDialogOpen(true);
            }}
          >
            Create First Project
          </Button>
        </Paper>
      )}
    </Box>
  );

  const renderGroupsTab = () => (
    <Box>
      {/* Search and Filter Bar */}
      <Paper elevation={1} sx={{ p: 2, mb: 3 }}>
        <Stack direction="row" spacing={2} alignItems="center">
          <TextField
            size="small"
            placeholder="Search groups..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            sx={{ flexGrow: 1 }}
            InputProps={{
              startAdornment: (
                <InputAdornment position="start">
                  <SearchIcon />
                </InputAdornment>
              ),
            }}
          />

          <Button
            variant="contained"
            color="secondary"
            startIcon={<AddIcon />}
            onClick={() => {
              setCreateType('group');
              setCreateDialogOpen(true);
            }}
          >
            New Group
          </Button>
        </Stack>
      </Paper>

      {/* Groups Grid */}
      <Grid container spacing={3}>
        {hierarchy.groups
          .filter(g => g.name.toLowerCase().includes(searchQuery.toLowerCase()))
          .map(renderGroupCard)}
      </Grid>

      {hierarchy.groups.length === 0 && (
        <Paper sx={{ p: 4, textAlign: 'center' }}>
          <ForumIcon sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
          <Typography variant="h6" gutterBottom>
            No groups yet
          </Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
            Create a group to start team discussions
          </Typography>
          <Button
            variant="contained"
            color="secondary"
            startIcon={<GroupAddIcon />}
            onClick={() => {
              setCreateType('group');
              setCreateDialogOpen(true);
            }}
          >
            Create First Group
          </Button>
        </Paper>
      )}
    </Box>
  );

  const renderMembersTab = () => (
    <Box>
      {/* Members Management Bar */}
      <Paper elevation={1} sx={{ p: 2, mb: 3 }}>
        <Stack direction="row" spacing={2} alignItems="center">
          <TextField
            size="small"
            placeholder="Search members..."
            sx={{ flexGrow: 1 }}
            InputProps={{
              startAdornment: (
                <InputAdornment position="start">
                  <SearchIcon />
                </InputAdornment>
              ),
            }}
          />
          <Button
            variant="contained"
            startIcon={<PersonAddIcon />}
            onClick={() => onNavigate('invite_members', organization)}
          >
            Invite Members
          </Button>
        </Stack>
      </Paper>

      {/* Members Grid */}
      <Grid container spacing={3}>
        {organization.members.map((member) => (
          <Grid item xs={12} sm={6} md={4} lg={3} key={member.user_id}>
            <Paper elevation={2} sx={{ p: 2 }}>
              <Stack spacing={2} alignItems="center">
                <Badge
                  overlap="circular"
                  anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                  badgeContent={
                    <Chip 
                      label={member.role} 
                      size="small"
                      color={member.role === 'Owner' ? 'error' : 
                             member.role === 'Admin' ? 'warning' : 'default'}
                    />
                  }
                >
                  <Avatar sx={{ width: 64, height: 64 }}>
                    {member.display_name[0]}
                  </Avatar>
                </Badge>
                
                <Box sx={{ textAlign: 'center' }}>
                  <Typography variant="subtitle1">
                    {member.display_name}
                  </Typography>
                  <Typography variant="caption" color="text.secondary">
                    {member.four_word_address}
                  </Typography>
                </Box>

                <Stack direction="row" spacing={1}>
                  <IconButton size="small" color="primary">
                    <ChatIcon />
                  </IconButton>
                  <IconButton size="small" color="success">
                    <PhoneIcon />
                  </IconButton>
                  <IconButton size="small">
                    <VideocamIcon />
                  </IconButton>
                </Stack>
              </Stack>
            </Paper>
          </Grid>
        ))}
      </Grid>
    </Box>
  );

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <Paper elevation={0} sx={{ p: 3, borderBottom: 1, borderColor: 'divider' }}>
        <Stack direction="row" alignItems="center" justifyContent="space-between">
          <Stack direction="row" spacing={2} alignItems="center">
            <Avatar sx={{ width: 56, height: 56, bgcolor: 'primary.main' }}>
              <BusinessIcon sx={{ fontSize: 32 }} />
            </Avatar>
            <Box>
              <Typography variant="h4">
                {organization.name}
              </Typography>
              <Stack direction="row" spacing={2} alignItems="center">
                <Chip 
                  icon={<PeopleIcon />}
                  label={`${hierarchy.total_members} members`}
                />
                <Chip 
                  icon={<StorageIcon />}
                  label={`${hierarchy.total_storage_used_gb.toFixed(1)} GB used`}
                />
                <Chip 
                  label={organization.settings.visibility}
                  color={organization.settings.visibility === 'public' ? 'success' : 'default'}
                />
              </Stack>
            </Box>
          </Stack>

          <Stack direction="row" spacing={1}>
            <Button
              variant="contained"
              color="success"
              startIcon={<PhoneIcon />}
              onClick={() => onCall('organization', organization.id, 'voice')}
            >
              Voice Call
            </Button>
            <Button
              variant="contained"
              color="primary"
              startIcon={<VideocamIcon />}
              onClick={() => onCall('organization', organization.id, 'video')}
            >
              Video Call
            </Button>
            <IconButton onClick={() => onNavigate('settings', organization)}>
              <SettingsIcon />
            </IconButton>
          </Stack>
        </Stack>
      </Paper>

      {/* Tabs */}
      <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
        <Tabs value={activeTab} onChange={(_, newValue) => setActiveTab(newValue)}>
          <Tab label="Overview" icon={<DashboardIcon />} iconPosition="start" />
          <Tab label={`Projects (${hierarchy.projects.length})`} icon={<FolderIcon />} iconPosition="start" />
          <Tab label={`Groups (${hierarchy.groups.length})`} icon={<GroupIcon />} iconPosition="start" />
          <Tab label={`Members (${organization.members.length})`} icon={<PeopleIcon />} iconPosition="start" />
        </Tabs>
      </Box>

      {/* Tab Panels */}
      <Box sx={{ flexGrow: 1, overflow: 'auto' }}>
        <TabPanel value={activeTab} index={0}>
          {renderOverviewTab()}
        </TabPanel>
        <TabPanel value={activeTab} index={1}>
          {renderProjectsTab()}
        </TabPanel>
        <TabPanel value={activeTab} index={2}>
          {renderGroupsTab()}
        </TabPanel>
        <TabPanel value={activeTab} index={3}>
          {renderMembersTab()}
        </TabPanel>
      </Box>

      {/* Context Menu */}
      <Menu
        anchorEl={menuAnchor}
        open={Boolean(menuAnchor)}
        onClose={() => setMenuAnchor(null)}
      >
        <MenuItem onClick={() => {
          setMenuAnchor(null);
          onNavigate('edit', selectedEntity);
        }}>
          <ListItemIcon>
            <EditIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText>Edit</ListItemText>
        </MenuItem>
        <MenuItem onClick={() => {
          setMenuAnchor(null);
          onNavigate('share', selectedEntity);
        }}>
          <ListItemIcon>
            <ShareIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText>Share</ListItemText>
        </MenuItem>
        <MenuItem onClick={() => {
          setMenuAnchor(null);
          onNavigate('duplicate', selectedEntity);
        }}>
          <ListItemIcon>
            <CopyIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText>Duplicate</ListItemText>
        </MenuItem>
        <Divider />
        <MenuItem 
          onClick={() => {
            setMenuAnchor(null);
            onNavigate('delete', selectedEntity);
          }}
          sx={{ color: 'error.main' }}
        >
          <ListItemIcon>
            <DeleteIcon fontSize="small" color="error" />
          </ListItemIcon>
          <ListItemText>Delete</ListItemText>
        </MenuItem>
      </Menu>

      {/* Create Dialog */}
      <Dialog open={createDialogOpen} onClose={() => setCreateDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>
          Create New {createType === 'project' ? 'Project' : 'Group'}
        </DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Name"
            fullWidth
            variant="outlined"
          />
          <TextField
            margin="dense"
            label="Description"
            fullWidth
            multiline
            rows={3}
            variant="outlined"
          />
          {createType === 'project' && (
            <>
              <FormControl fullWidth margin="dense">
                <InputLabel>Priority</InputLabel>
                <Select label="Priority" defaultValue="medium">
                  <MenuItem value="low">Low</MenuItem>
                  <MenuItem value="medium">Medium</MenuItem>
                  <MenuItem value="high">High</MenuItem>
                  <MenuItem value="critical">Critical</MenuItem>
                </Select>
              </FormControl>
              <TextField
                margin="dense"
                label="Storage Allocation (GB)"
                type="number"
                fullWidth
                defaultValue={5}
                variant="outlined"
              />
            </>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateDialogOpen(false)}>Cancel</Button>
          <Button onClick={handleCreateEntity} variant="contained" disabled={loading}>
            {loading ? <CircularProgress size={24} /> : 'Create'}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default OrganizationView;