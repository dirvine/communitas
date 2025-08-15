import React, { useState, useEffect } from 'react';
import {
  Box,
  Paper,
  Typography,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  ListItemAvatar,
  Avatar,
  Divider,
  IconButton,
  Badge,
  Chip,
  Stack,
  TextField,
  InputAdornment,
  Button,
  Card,
  CardContent,
  CardActions,
  LinearProgress,
  Tooltip,
  Tab,
  Tabs,
  Collapse,
  Alert,
} from '@mui/material';
import {
  Person as PersonIcon,
  Group as GroupIcon,
  Folder as FolderIcon,
  Description as FileIcon,
  Phone as PhoneIcon,
  Videocam as VideocamIcon,
  Chat as ChatIcon,
  Storage as StorageIcon,
  Settings as SettingsIcon,
  Add as AddIcon,
  Star as StarIcon,
  Schedule as ScheduleIcon,
  Assignment as TaskIcon,
  Link as LinkIcon,
  CloudUpload as UploadIcon,
  CloudDownload as DownloadIcon,
  Search as SearchIcon,
  MoreVert as MoreIcon,
  Edit as EditIcon,
  Delete as DeleteIcon,
  Share as ShareIcon,
  PersonAdd as PersonAddIcon,
  FolderOpen as FolderOpenIcon,
  NotificationsActive as NotificationIcon,
  Security as SecurityIcon,
  Public as PublicIcon,
  Lock as LockIcon,
  VpnLock as VpnLockIcon,
} from '@mui/icons-material';
import { Organization, Project, Group, Member } from '../types/organization';

interface ContextAwareSidebarProps {
  context: {
    type: 'personal' | 'organization' | 'project' | 'group';
    entity?: Organization | Project | Group;
  };
  onAction: (action: string, data?: any) => void;
  onMemberSelect: (member: Member) => void;
  onFileSelect: (file: any) => void;
}

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

const TabPanel: React.FC<TabPanelProps> = ({ children, value, index }) => {
  return (
    <div hidden={value !== index} style={{ height: '100%' }}>
      {value === index && <Box sx={{ height: '100%' }}>{children}</Box>}
    </div>
  );
};

export const ContextAwareSidebar: React.FC<ContextAwareSidebarProps> = ({
  context,
  onAction,
  onMemberSelect,
  onFileSelect,
}) => {
  const [activeTab, setActiveTab] = useState(0);
  const [searchQuery, setSearchQuery] = useState('');
  const [recentFiles, setRecentFiles] = useState<any[]>([]);
  const [activities, setActivities] = useState<any[]>([]);

  // Reset tab when context changes
  useEffect(() => {
    setActiveTab(0);
    loadContextData();
  }, [context]);

  const loadContextData = async () => {
    // Load context-specific data (files, activities, etc.)
    // This would fetch from the backend based on context
    setRecentFiles([
      { id: '1', name: 'Project Plan.md', size: '24KB', modified: '2 hours ago', type: 'markdown' },
      { id: '2', name: 'Meeting Notes.pdf', size: '1.2MB', modified: '1 day ago', type: 'pdf' },
      { id: '3', name: 'Design Assets.zip', size: '45MB', modified: '3 days ago', type: 'archive' },
    ]);

    setActivities([
      { id: '1', user: 'Alice', action: 'uploaded', target: 'Design.fig', time: '30 min ago' },
      { id: '2', user: 'Bob', action: 'commented on', target: 'Project Plan', time: '1 hour ago' },
      { id: '3', user: 'Charlie', action: 'joined', target: 'the project', time: '2 hours ago' },
    ]);
  };

  const renderPersonalContext = () => (
    <Box sx={{ p: 2 }}>
      <Typography variant="h6" gutterBottom>
        Quick Access
      </Typography>
      
      <Stack spacing={2}>
        <Card variant="outlined">
          <CardContent>
            <Typography variant="subtitle2" color="text.secondary">
              Your Storage
            </Typography>
            <Box sx={{ mt: 1 }}>
              <LinearProgress variant="determinate" value={35} sx={{ mb: 1 }} />
              <Typography variant="caption">
                3.5 GB of 10 GB used
              </Typography>
            </Box>
          </CardContent>
          <CardActions>
            <Button size="small" startIcon={<UploadIcon />}>
              Upload
            </Button>
            <Button size="small" startIcon={<FolderOpenIcon />}>
              Browse
            </Button>
          </CardActions>
        </Card>

        <Card variant="outlined">
          <CardContent>
            <Typography variant="subtitle2" gutterBottom>
              Recent Contacts
            </Typography>
            <List dense>
              {['Alice', 'Bob', 'Charlie'].map((name) => (
                <ListItemButton key={name} onClick={() => onAction('open_chat', name)}>
                  <ListItemAvatar>
                    <Avatar sx={{ width: 32, height: 32 }}>{name[0]}</Avatar>
                  </ListItemAvatar>
                  <ListItemText primary={name} />
                  <IconButton size="small">
                    <ChatIcon fontSize="small" />
                  </IconButton>
                </ListItemButton>
              ))}
            </List>
          </CardContent>
        </Card>

        <Button
          fullWidth
          variant="outlined"
          startIcon={<AddIcon />}
          onClick={() => onAction('create_organization')}
        >
          Create Organization
        </Button>
      </Stack>
    </Box>
  );

  const renderOrganizationContext = () => {
    const org = context.entity as Organization;
    if (!org) return null;

    return (
      <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
        {/* Organization Header */}
        <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
          <Stack direction="row" alignItems="center" spacing={1}>
            <Avatar sx={{ bgcolor: 'primary.main' }}>
              {org.name[0]}
            </Avatar>
            <Box sx={{ flexGrow: 1 }}>
              <Typography variant="subtitle1">{org.name}</Typography>
              <Stack direction="row" spacing={1}>
                <Chip 
                  size="small" 
                  icon={org.settings.visibility === 'public' ? <PublicIcon /> : <LockIcon />}
                  label={org.settings.visibility}
                />
                <Chip 
                  size="small" 
                  icon={<PersonIcon />}
                  label={`${org.members.length} members`}
                />
              </Stack>
            </Box>
            <IconButton size="small" onClick={() => onAction('org_settings', org)}>
              <SettingsIcon />
            </IconButton>
          </Stack>
        </Box>

        {/* Tabs */}
        <Tabs
          value={activeTab}
          onChange={(_, newValue) => setActiveTab(newValue)}
          variant="fullWidth"
        >
          <Tab label="Members" />
          <Tab label="Files" />
          <Tab label="Activity" />
        </Tabs>

        {/* Tab Panels */}
        <Box sx={{ flexGrow: 1, overflow: 'auto' }}>
          <TabPanel value={activeTab} index={0}>
            <Box sx={{ p: 2 }}>
              <Stack direction="row" alignItems="center" sx={{ mb: 2 }}>
                <TextField
                  size="small"
                  placeholder="Search members..."
                  fullWidth
                  InputProps={{
                    startAdornment: (
                      <InputAdornment position="start">
                        <SearchIcon />
                      </InputAdornment>
                    ),
                  }}
                />
                <IconButton onClick={() => onAction('invite_member', org)}>
                  <PersonAddIcon />
                </IconButton>
              </Stack>
              
              <List>
                {org.members.map((member) => (
                  <ListItemButton 
                    key={member.user_id}
                    onClick={() => onMemberSelect(member)}
                  >
                    <ListItemAvatar>
                      <Avatar>{member.display_name[0]}</Avatar>
                    </ListItemAvatar>
                    <ListItemText 
                      primary={member.display_name}
                      secondary={
                        <Stack direction="row" spacing={1}>
                          <Chip label={member.role} size="small" />
                          <Typography variant="caption">
                            {member.four_word_address}
                          </Typography>
                        </Stack>
                      }
                    />
                    <Stack direction="row" spacing={0.5}>
                      <IconButton size="small">
                        <ChatIcon fontSize="small" />
                      </IconButton>
                      <IconButton size="small">
                        <PhoneIcon fontSize="small" />
                      </IconButton>
                    </Stack>
                  </ListItemButton>
                ))}
              </List>
            </Box>
          </TabPanel>

          <TabPanel value={activeTab} index={1}>
            <Box sx={{ p: 2 }}>
              <Button
                fullWidth
                variant="outlined"
                startIcon={<UploadIcon />}
                sx={{ mb: 2 }}
                onClick={() => onAction('upload_file', org)}
              >
                Upload Files
              </Button>
              
              <Typography variant="subtitle2" gutterBottom>
                Recent Files
              </Typography>
              <List>
                {recentFiles.map((file) => (
                  <ListItemButton 
                    key={file.id}
                    onClick={() => onFileSelect(file)}
                  >
                    <ListItemIcon>
                      <FileIcon />
                    </ListItemIcon>
                    <ListItemText 
                      primary={file.name}
                      secondary={`${file.size} • ${file.modified}`}
                    />
                    <IconButton size="small">
                      <DownloadIcon fontSize="small" />
                    </IconButton>
                  </ListItemButton>
                ))}
              </List>
            </Box>
          </TabPanel>

          <TabPanel value={activeTab} index={2}>
            <Box sx={{ p: 2 }}>
              <List>
                {activities.map((activity) => (
                  <ListItem key={activity.id}>
                    <ListItemAvatar>
                      <Avatar sx={{ width: 32, height: 32 }}>
                        {activity.user[0]}
                      </Avatar>
                    </ListItemAvatar>
                    <ListItemText 
                      primary={
                        <Typography variant="body2">
                          <strong>{activity.user}</strong> {activity.action}{' '}
                          <strong>{activity.target}</strong>
                        </Typography>
                      }
                      secondary={activity.time}
                    />
                  </ListItem>
                ))}
              </List>
            </Box>
          </TabPanel>
        </Box>

        {/* Storage Info */}
        <Box sx={{ p: 2, borderTop: 1, borderColor: 'divider' }}>
          <Typography variant="subtitle2" gutterBottom>
            Organization Storage
          </Typography>
          <LinearProgress 
            variant="determinate" 
            value={(org.storage_quota.used_gb / org.storage_quota.allocated_gb) * 100}
            sx={{ mb: 1 }}
          />
          <Typography variant="caption">
            {org.storage_quota.used_gb.toFixed(1)} GB of {org.storage_quota.allocated_gb} GB used
          </Typography>
        </Box>
      </Box>
    );
  };

  const renderProjectContext = () => {
    const project = context.entity as Project;
    if (!project) return null;

    return (
      <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
        {/* Project Header */}
        <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
          <Stack spacing={1}>
            <Typography variant="h6">{project.name}</Typography>
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
              {project.deadline && (
                <Chip 
                  icon={<ScheduleIcon />}
                  label={new Date(project.deadline).toLocaleDateString()} 
                  size="small"
                />
              )}
            </Stack>
          </Stack>
        </Box>

        {/* Project Actions */}
        <Box sx={{ p: 2 }}>
          <Stack spacing={1}>
            <Button
              fullWidth
              variant="contained"
              startIcon={<TaskIcon />}
              onClick={() => onAction('create_task', project)}
            >
              Create Task
            </Button>
            <Button
              fullWidth
              variant="outlined"
              startIcon={<UploadIcon />}
              onClick={() => onAction('upload_file', project)}
            >
              Upload Document
            </Button>
            <Button
              fullWidth
              variant="outlined"
              startIcon={<PersonAddIcon />}
              onClick={() => onAction('add_member', project)}
            >
              Add Member
            </Button>
          </Stack>
        </Box>

        <Divider />

        {/* Project Stats */}
        <Box sx={{ p: 2 }}>
          <Typography variant="subtitle2" gutterBottom>
            Project Statistics
          </Typography>
          <Stack spacing={1}>
            <Stack direction="row" justifyContent="space-between">
              <Typography variant="body2">Tasks</Typography>
              <Typography variant="body2" fontWeight="bold">12 / 45</Typography>
            </Stack>
            <Stack direction="row" justifyContent="space-between">
              <Typography variant="body2">Files</Typography>
              <Typography variant="body2" fontWeight="bold">234</Typography>
            </Stack>
            <Stack direction="row" justifyContent="space-between">
              <Typography variant="body2">Members</Typography>
              <Typography variant="body2" fontWeight="bold">{project.members.length}</Typography>
            </Stack>
          </Stack>
        </Box>

        <Divider />

        {/* Project Settings */}
        <Box sx={{ p: 2 }}>
          <Typography variant="subtitle2" gutterBottom>
            Project Settings
          </Typography>
          <List dense>
            <ListItem>
              <ListItemIcon>
                <SecurityIcon fontSize="small" />
              </ListItemIcon>
              <ListItemText 
                primary="Version Control"
                secondary={project.settings.version_control_enabled ? 'Enabled' : 'Disabled'}
              />
            </ListItem>
            <ListItem>
              <ListItemIcon>
                <VpnLockIcon fontSize="small" />
              </ListItemIcon>
              <ListItemText 
                primary="Backup"
                secondary={project.settings.backup_enabled ? 'Enabled' : 'Disabled'}
              />
            </ListItem>
          </List>
        </Box>
      </Box>
    );
  };

  const renderGroupContext = () => {
    const group = context.entity as Group;
    if (!group) return null;

    return (
      <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
        {/* Group Header */}
        <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
          <Stack spacing={1}>
            <Typography variant="h6">{group.name}</Typography>
            <Stack direction="row" spacing={1}>
              <Chip 
                icon={<GroupIcon />}
                label={`${group.members.length} members`} 
                size="small"
              />
              <Chip 
                icon={<ChatIcon />}
                label="Chat Only" 
                size="small"
                color="info"
              />
            </Stack>
          </Stack>
        </Box>

        {/* Group Actions */}
        <Box sx={{ p: 2 }}>
          <Stack spacing={1}>
            <Button
              fullWidth
              variant="contained"
              color="success"
              startIcon={<PhoneIcon />}
              onClick={() => onAction('start_voice_call', group)}
            >
              Start Voice Call
            </Button>
            <Button
              fullWidth
              variant="contained"
              color="primary"
              startIcon={<VideocamIcon />}
              onClick={() => onAction('start_video_call', group)}
            >
              Start Video Call
            </Button>
            <Button
              fullWidth
              variant="outlined"
              startIcon={<PersonAddIcon />}
              onClick={() => onAction('invite_to_group', group)}
            >
              Invite Members
            </Button>
          </Stack>
        </Box>

        <Divider />

        {/* Chat Settings */}
        <Box sx={{ p: 2 }}>
          <Typography variant="subtitle2" gutterBottom>
            Chat Settings
          </Typography>
          <List dense>
            <ListItem>
              <ListItemText 
                primary="Message Retention"
                secondary={`${group.chat_settings.message_retention_days} days`}
              />
            </ListItem>
            <ListItem>
              <ListItemText 
                primary="File Sharing"
                secondary={group.chat_settings.allow_file_sharing ? 'Allowed' : 'Disabled'}
              />
            </ListItem>
            <ListItem>
              <ListItemText 
                primary="Voice Messages"
                secondary={group.chat_settings.allow_voice_messages ? 'Allowed' : 'Disabled'}
              />
            </ListItem>
            <ListItem>
              <ListItemText 
                primary="Video Calls"
                secondary={group.chat_settings.allow_video_calls ? 'Allowed' : 'Disabled'}
              />
            </ListItem>
          </List>
        </Box>

        <Divider />

        {/* Active Members */}
        <Box sx={{ p: 2, flexGrow: 1, overflow: 'auto' }}>
          <Typography variant="subtitle2" gutterBottom>
            Active Members
          </Typography>
          <List dense>
            {group.members.map((member) => (
              <ListItemButton 
                key={member.user_id}
                onClick={() => onMemberSelect(member)}
              >
                <ListItemAvatar>
                  <Badge
                    overlap="circular"
                    anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                    variant="dot"
                    color="success"
                  >
                    <Avatar sx={{ width: 32, height: 32 }}>
                      {member.display_name[0]}
                    </Avatar>
                  </Badge>
                </ListItemAvatar>
                <ListItemText 
                  primary={member.display_name}
                  secondary={member.role}
                />
              </ListItemButton>
            ))}
          </List>
        </Box>
      </Box>
    );
  };

  // Render based on context
  const renderContent = () => {
    switch (context.type) {
      case 'personal':
        return renderPersonalContext();
      case 'organization':
        return renderOrganizationContext();
      case 'project':
        return renderProjectContext();
      case 'group':
        return renderGroupContext();
      default:
        return null;
    }
  };

  return (
    <Paper 
      elevation={0} 
      sx={{ 
        height: '100%', 
        borderLeft: 1, 
        borderColor: 'divider',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}
    >
      {renderContent()}
    </Paper>
  );
};

export default ContextAwareSidebar;