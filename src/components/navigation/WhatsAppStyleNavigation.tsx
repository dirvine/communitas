import React, { useState, useEffect } from 'react';
import {
  Box,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  ListItemSecondaryAction,
  IconButton,
  Avatar,
  Badge,
  Typography,
  Button,
  Collapse,
  Divider,
  Paper,
  Slide,
  Chip,
  Stack,
  Tooltip,
  alpha,
  TextField,
  InputAdornment,
} from '@mui/material';
import {
  Business as OrganizationIcon,
  Groups as GroupIcon,
  Person as PersonIcon,
  VideoCall as VideoIcon,
  Call as CallIcon,
  ScreenShare as ScreenIcon,
  Folder as FolderIcon,
  Tag as ChannelIcon,
  Assignment as ProjectIcon,
  ArrowBack as BackIcon,
  ExpandMore,
  ExpandLess,
  Add as AddIcon,
  Search as SearchIcon,
} from '@mui/icons-material';
import { Organization, Group, PersonalUser, Channel, Project, OrganizationUser } from '../../types/collaboration';
import { useNavigation } from '../../contexts/NavigationContext';

// Stable, top-level subcomponents to avoid remounts (prevents input focus loss)
type ActionCallbacks = {
  onVideoCall?: (entityId: string, entityType: string) => void;
  onAudioCall?: (entityId: string, entityType: string) => void;
  onScreenShare?: (entityId: string, entityType: string) => void;
  onOpenFiles?: (entityId: string, entityType: string) => void;
};

const CollaborationActions: React.FC<{
  entityId: string;
  entityType: string;
  size?: 'small' | 'medium';
} & ActionCallbacks> = ({ entityId, entityType, size = 'small', onVideoCall, onAudioCall, onScreenShare, onOpenFiles }) => (
  <Box className="entity-actions" sx={{ display: 'flex', gap: 0.5, opacity: 0, transition: 'opacity 0.2s' }}>
    <Tooltip title="Video Call">
      <IconButton
        size={size}
        onClick={(e) => {
          e.stopPropagation();
          onVideoCall?.(entityId, entityType);
        }}
        sx={{ color: 'primary.main', '&:hover': { bgcolor: alpha('#4CAF50', 0.1) } }}
      >
        <VideoIcon fontSize={size} />
      </IconButton>
    </Tooltip>
    <Tooltip title="Audio Call">
      <IconButton
        size={size}
        onClick={(e) => {
          e.stopPropagation();
          onAudioCall?.(entityId, entityType);
        }}
        sx={{ color: 'primary.main', '&:hover': { bgcolor: alpha('#2196F3', 0.1) } }}
      >
        <CallIcon fontSize={size} />
      </IconButton>
    </Tooltip>
    <Tooltip title="Screen Share">
      <IconButton
        size={size}
        onClick={(e) => {
          e.stopPropagation();
          onScreenShare?.(entityId, entityType);
        }}
        sx={{ color: 'primary.main', '&:hover': { bgcolor: alpha('#FF9800', 0.1) } }}
      >
        <ScreenIcon fontSize={size} />
      </IconButton>
    </Tooltip>
    <Tooltip title="Files & Website">
      <IconButton
        size={size}
        onClick={(e) => {
          e.stopPropagation();
          onOpenFiles?.(entityId, entityType);
        }}
        sx={{ color: 'primary.main', '&:hover': { bgcolor: alpha('#9C27B0', 0.1) } }}
      >
        <FolderIcon fontSize={size} />
      </IconButton>
    </Tooltip>
  </Box>
);

interface MainNavigationProps extends ActionCallbacks {
  organizations: Organization[];
  personalGroups: Group[];
  personalUsers: PersonalUser[];
  showOrganizations: boolean;
  setShowOrganizations: (v: boolean) => void;
  onOrganizationSelected: (org: Organization) => void;
  onNavigate: (path: string, entity: any) => void;
  nav: ReturnType<typeof useNavigation>;
}

const MainNavigation: React.FC<MainNavigationProps> = ({
  organizations,
  personalGroups,
  personalUsers,
  showOrganizations,
  setShowOrganizations,
  onOrganizationSelected,
  onNavigate,
  nav,
  onVideoCall,
  onAudioCall,
  onScreenShare,
  onOpenFiles,
}) => {
  const [searchInput, setSearchInput] = useState('');
  const [debouncedTerm, setDebouncedTerm] = useState('');
  useEffect(() => {
    const id = setTimeout(() => setDebouncedTerm(searchInput.trim().toLowerCase()), 180);
    return () => clearTimeout(id);
  }, [searchInput]);
  const matchesQuery = (text?: string) => !debouncedTerm || (text ?? '').toLowerCase().includes(debouncedTerm);
  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Global Search */}
      <Box sx={{ p: 2, pb: 0 }}>
        <TextField
          fullWidth
          size="small"
          value={searchInput}
          onChange={(e) => setSearchInput(e.target.value)}
          placeholder="Search organizations, groups, contacts…"
          InputProps={{
            startAdornment: (
              <InputAdornment position="start">
                <SearchIcon fontSize="small" />
              </InputAdornment>
            ),
          }}
        />
      </Box>
      {/* Organizations Button at Top */}
      <Box sx={{ p: 2 }}>
        <Button
          fullWidth
          variant="contained"
          startIcon={<OrganizationIcon />}
          endIcon={showOrganizations ? <ExpandLess /> : <ExpandMore />}
          onClick={() => setShowOrganizations(!showOrganizations)}
          sx={{
            justifyContent: 'space-between',
            background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
            '&:hover': { background: 'linear-gradient(135deg, #764ba2 0%, #667eea 100%)' },
          }}
        >
          Organizations ({organizations.length})
        </Button>
        <Collapse in={showOrganizations} timeout="auto" unmountOnExit>
          <Paper elevation={3} sx={{ mt: 1, maxHeight: 300, overflow: 'auto' }}>
            <List dense>
              {organizations.filter(org => matchesQuery(org.name) || matchesQuery(org.networkIdentity.fourWords)).map(org => (
                <ListItemButton
                  key={org.id}
                  onClick={() => { onOrganizationSelected(org); }}
                  sx={{ '&:hover': { bgcolor: alpha('#667eea', 0.1) } }}
                >
                  <ListItemIcon>
                    <Avatar sx={{ width: 32, height: 32 }}>{org.name[0]}</Avatar>
                  </ListItemIcon>
                  <ListItemText primary={org.name} secondary={org.networkIdentity.fourWords} />
                </ListItemButton>
              ))}
              <Divider />
              <ListItemButton>
                <ListItemIcon>
                  <AddIcon />
                </ListItemIcon>
                <ListItemText primary="Create Organization" />
              </ListItemButton>
            </List>
          </Paper>
        </Collapse>
      </Box>
      <Divider />
      {/* Groups */}
      <Box sx={{ flex: 1, overflow: 'auto' }}>
        <Box sx={{ p: 2, pb: 1 }}>
          <Typography variant="subtitle2" color="text.secondary" sx={{ mb: 1 }}>GROUPS</Typography>
        </Box>
        <List dense>
          {personalGroups.filter(group => matchesQuery(group.name) || matchesQuery(group.networkIdentity.fourWords)).map(group => (
            <ListItem
              key={group.id}
              sx={{ position: 'relative', '& .entity-actions': { opacity: 0 }, '&:hover': { bgcolor: 'action.hover', '& .entity-actions': { opacity: 1 } } }}
              secondaryAction={<CollaborationActions entityId={group.id} entityType="group" onVideoCall={onVideoCall} onAudioCall={onAudioCall} onScreenShare={onScreenShare} onOpenFiles={onOpenFiles} />}
            >
              <ListItemButton onClick={() => { nav.selectEntity('group', group.id, group.name); onNavigate(`/group/${group.id}`, group); }} sx={{ pr: 16 }}>
                <ListItemIcon>
                  <Avatar sx={{ width: 40, height: 40 }}><GroupIcon /></Avatar>
                </ListItemIcon>
                <ListItemText
                  primary={<Typography variant="body1" fontWeight={500}>{group.name}</Typography>}
                  secondary={<Typography variant="caption" color="text.secondary">{group.members.length} members • {group.networkIdentity.fourWords}</Typography>}
                />
              </ListItemButton>
            </ListItem>
          ))}
        </List>
        <Divider sx={{ my: 1 }} />
        {/* Contacts */}
        <Box sx={{ p: 2, pb: 1 }}>
          <Typography variant="subtitle2" color="text.secondary" sx={{ mb: 1 }}>CONTACTS</Typography>
        </Box>
        <List dense>
          {personalUsers.filter(user => matchesQuery(user.name) || matchesQuery(user.networkIdentity.fourWords)).map(user => (
            <ListItem
              key={user.id}
              sx={{ position: 'relative', '& .entity-actions': { opacity: 0 }, '&:hover': { bgcolor: 'action.hover', '& .entity-actions': { opacity: 1 } } }}
              secondaryAction={<CollaborationActions entityId={user.id} entityType="user" onVideoCall={onVideoCall} onAudioCall={onAudioCall} onScreenShare={onScreenShare} onOpenFiles={onOpenFiles} />}
            >
              <ListItemButton onClick={() => { nav.selectEntity('individual', user.id, user.name); onNavigate(`/user/${user.id}`, user); }} sx={{ pr: 16 }}>
                <ListItemIcon>
                  <Badge variant="dot" color="success" invisible={Math.random() > 0.5} anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}>
                    <Avatar sx={{ width: 40, height: 40 }}>{user.name[0]}</Avatar>
                  </Badge>
                </ListItemIcon>
                <ListItemText
                  primary={<Typography variant="body1" fontWeight={500}>{user.name}</Typography>}
                  secondary={<Typography variant="caption" color="text.secondary">{user.relationship} • {user.networkIdentity.fourWords}</Typography>}
                />
              </ListItemButton>
            </ListItem>
          ))}
        </List>
      </Box>
    </Box>
  );
};

interface OrgNavigationProps extends ActionCallbacks {
  org: Organization;
  expandedSections: { channels: boolean; projects: boolean; groups: boolean; users: boolean };
  toggleSection: (k: keyof OrgNavigationProps['expandedSections']) => void;
  onNavigate: (path: string, entity: any) => void;
  nav: ReturnType<typeof useNavigation>;
  onBack: () => void;
}

const OrganizationNavigation: React.FC<OrgNavigationProps> = ({ org, expandedSections, toggleSection, onNavigate, nav, onVideoCall, onAudioCall, onScreenShare, onOpenFiles, onBack }) => (
  <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
    {/* Organization Header */}
    <Box sx={{ p: 2, bgcolor: 'primary.main', color: 'primary.contrastText' }}>
      <Stack direction="row" alignItems="center" spacing={1}>
        <IconButton size="small" onClick={onBack} sx={{ color: 'inherit' }}>
          <BackIcon />
        </IconButton>
        <Avatar sx={{ width: 32, height: 32, bgcolor: 'primary.dark' }}>{org.name[0]}</Avatar>
        <Box sx={{ flex: 1 }}>
          <Typography variant="subtitle1" fontWeight={600}>{org.name}</Typography>
          <Typography variant="caption" sx={{ opacity: 0.9 }}>{org.networkIdentity.fourWords}</Typography>
        </Box>
      </Stack>
    </Box>
    <Box sx={{ flex: 1, overflow: 'auto' }}>
      {/* Channels */}
      <List dense>
        <ListItemButton onClick={() => toggleSection('channels')}>
          <ListItemIcon>
            <ChannelIcon />
          </ListItemIcon>
          <ListItemText primary={<Stack direction="row" alignItems="center" spacing={1}><Typography variant="subtitle2" fontWeight={600}>CHANNELS</Typography><Chip label={org.channels.length} size="small" /></Stack>} />
          {expandedSections.channels ? <ExpandLess /> : <ExpandMore />}
        </ListItemButton>
          <Collapse in={expandedSections.channels} timeout="auto" unmountOnExit>
            {org.channels.map(channel => (
            <ListItem key={channel.id} sx={{ pl: 3, '& .entity-actions': { opacity: 0 }, '&:hover .entity-actions': { opacity: 1 } }}
              secondaryAction={<CollaborationActions entityId={channel.id} entityType="channel" onVideoCall={onVideoCall} onAudioCall={onAudioCall} onScreenShare={onScreenShare} onOpenFiles={onOpenFiles} />}
            >
              <ListItemButton onClick={() => { nav.selectEntity('channel', channel.id, channel.name); onNavigate(`/org/${org.id}/channel/${channel.id}`, channel); }} sx={{ pr: 16 }}>
                <ListItemIcon><Typography variant="h6" color="text.secondary">#</Typography></ListItemIcon>
                <ListItemText primary={channel.name} secondary={`${channel.members.length} members`} />
              </ListItemButton>
            </ListItem>
          ))}
        </Collapse>
      </List>
      <Divider />
      {/* Projects */}
      <List dense>
        <ListItemButton onClick={() => toggleSection('projects')}>
          <ListItemIcon>
            <ProjectIcon />
          </ListItemIcon>
          <ListItemText primary={<Stack direction="row" alignItems="center" spacing={1}><Typography variant="subtitle2" fontWeight={600}>PROJECTS</Typography><Chip label={org.projects.length} size="small" /></Stack>} />
          {expandedSections.projects ? <ExpandLess /> : <ExpandMore />}
        </ListItemButton>
          <Collapse in={expandedSections.projects} timeout="auto" unmountOnExit>
            {org.projects.map(project => (
            <ListItem key={project.id} sx={{ pl: 3, '& .entity-actions': { opacity: 0 }, '&:hover .entity-actions': { opacity: 1 } }}
              secondaryAction={<CollaborationActions entityId={project.id} entityType="project" onVideoCall={onVideoCall} onAudioCall={onAudioCall} onScreenShare={onScreenShare} onOpenFiles={onOpenFiles} />}
            >
              <ListItemButton onClick={() => { nav.selectEntity('project', project.id, project.name); onNavigate(`/org/${org.id}/project/${project.id}`, project); }} sx={{ pr: 16 }}>
                <ListItemIcon><ProjectIcon /></ListItemIcon>
                <ListItemText primary={project.name} secondary={`${project.members.length} members • ${project.status}`} />
              </ListItemButton>
            </ListItem>
          ))}
        </Collapse>
      </List>
      <Divider />
      {/* Org Groups */}
      <List dense>
        <ListItemButton onClick={() => toggleSection('groups')}>
          <ListItemIcon>
            <GroupIcon />
          </ListItemIcon>
          <ListItemText primary={<Stack direction="row" alignItems="center" spacing={1}><Typography variant="subtitle2" fontWeight={600}>GROUPS</Typography><Chip label={org.groups.length} size="small" /></Stack>} />
          {expandedSections.groups ? <ExpandLess /> : <ExpandMore />}
        </ListItemButton>
          <Collapse in={expandedSections.groups} timeout="auto" unmountOnExit>
            {org.groups.map(group => (
            <ListItem key={group.id} sx={{ pl: 3 }}
              secondaryAction={<CollaborationActions entityId={group.id} entityType="org-group" onVideoCall={onVideoCall} onAudioCall={onAudioCall} onScreenShare={onScreenShare} onOpenFiles={onOpenFiles} />}
            >
              <ListItemButton onClick={() => { nav.selectEntity('group', group.id, group.name); onNavigate(`/org/${org.id}/group/${group.id}`, group); }} sx={{ pr: 16 }}>
                <ListItemIcon><Avatar sx={{ width: 32, height: 32 }}><GroupIcon fontSize="small" /></Avatar></ListItemIcon>
                <ListItemText primary={group.name} secondary={`${group.members.length} members`} />
              </ListItemButton>
            </ListItem>
          ))}
        </Collapse>
      </List>
      <Divider />
      {/* Users */}
      <List dense>
        <ListItemButton onClick={() => toggleSection('users')}>
          <ListItemIcon>
            <PersonIcon />
          </ListItemIcon>
        
          <ListItemText primary={<Stack direction="row" alignItems="center" spacing={1}><Typography variant="subtitle2" fontWeight={600}>MEMBERS</Typography><Chip label={org.users.length} size="small" /></Stack>} />
          {expandedSections.users ? <ExpandLess /> : <ExpandMore />}
        </ListItemButton>
          <Collapse in={expandedSections.users} timeout="auto" unmountOnExit>
            {org.users.map(user => (
            <ListItem key={user.id} sx={{ pl: 3 }}
              secondaryAction={<CollaborationActions entityId={user.id} entityType="org-user" onVideoCall={onVideoCall} onAudioCall={onAudioCall} onScreenShare={onScreenShare} onOpenFiles={onOpenFiles} />}
            >
              <ListItemButton onClick={() => onNavigate(`/org/${org.id}/user/${user.id}`, user)}>
                <ListItemIcon>
                  <Badge variant="dot" color="success" invisible={Math.random() > 0.3}>
                    <Avatar sx={{ width: 32, height: 32 }}>{user.name[0]}</Avatar>
                  </Badge>
                </ListItemIcon>
                <ListItemText primary={user.name} secondary={<Stack direction="row" spacing={1} alignItems="center"><Chip label={user.role} size="small" variant="outlined" /></Stack>} />
              </ListItemButton>
            </ListItem>
          ))}
        </Collapse>
      </List>
    </Box>
  </Box>
);

interface WhatsAppStyleNavigationProps {
  organizations: Organization[];
  personalGroups: Group[];
  personalUsers: PersonalUser[];
  currentUserId: string;
  onNavigate: (path: string, entity: any) => void;
  onVideoCall?: (entityId: string, entityType: string) => void;
  onAudioCall?: (entityId: string, entityType: string) => void;
  onScreenShare?: (entityId: string, entityType: string) => void;
  onOpenFiles?: (entityId: string, entityType: string) => void;
}

export const WhatsAppStyleNavigation: React.FC<WhatsAppStyleNavigationProps> = ({
  organizations,
  personalGroups,
  personalUsers,
  currentUserId,
  onNavigate,
  onVideoCall,
  onAudioCall,
  onScreenShare,
  onOpenFiles,
}) => {
  const nav = useNavigation();
  const [showOrganizations, setShowOrganizations] = useState(false);
  const [selectedOrganization, setSelectedOrganization] = useState<Organization | null>(null);
  const [expandedSections, setExpandedSections] = useState({
    channels: true,
    projects: true,
    groups: true,
    users: true,
  });


  const handleOrganizationClick = (org: Organization) => {
    setSelectedOrganization(org);
    setShowOrganizations(false);
    nav.switchToOrganization(org.id, org.name);
    onNavigate(`/org/${org.id}`, org);
  };

  const handleBackToMain = () => {
    setSelectedOrganization(null);
    onNavigate('/', null);
  };

  const toggleSection = (section: keyof typeof expandedSections) => {
    setExpandedSections(prev => ({
      ...prev,
      [section]: !prev[section],
    }));
  };

  // Collaboration action buttons component
  const CollaborationActions = ({ 
    entityId, 
    entityType,
    size = 'small' 
  }: { 
    entityId: string; 
    entityType: string;
    size?: 'small' | 'medium';
  }) => (
    <Box className="entity-actions" sx={{ display: 'flex', gap: 0.5, opacity: 0, transition: 'opacity 0.2s' }}>
      <Tooltip title="Video Call">
        <IconButton
          size={size}
          onClick={(e) => {
            e.stopPropagation();
            onVideoCall?.(entityId, entityType);
          }}
          sx={{
            color: 'primary.main',
            '&:hover': { bgcolor: alpha('#4CAF50', 0.1) },
          }}
        >
          <VideoIcon fontSize={size} />
        </IconButton>
      </Tooltip>
      <Tooltip title="Audio Call">
        <IconButton
          size={size}
          onClick={(e) => {
            e.stopPropagation();
            onAudioCall?.(entityId, entityType);
          }}
          sx={{
            color: 'primary.main',
            '&:hover': { bgcolor: alpha('#2196F3', 0.1) },
          }}
        >
          <CallIcon fontSize={size} />
        </IconButton>
      </Tooltip>
      <Tooltip title="Screen Share">
        <IconButton
          size={size}
          onClick={(e) => {
            e.stopPropagation();
            onScreenShare?.(entityId, entityType);
          }}
          sx={{
            color: 'primary.main',
            '&:hover': { bgcolor: alpha('#FF9800', 0.1) },
          }}
        >
          <ScreenIcon fontSize={size} />
        </IconButton>
      </Tooltip>
      <Tooltip title="Files & Website">
        <IconButton
          size={size}
          onClick={(e) => {
            e.stopPropagation();
            onOpenFiles?.(entityId, entityType);
          }}
          sx={{
            color: 'primary.main',
            '&:hover': { bgcolor: alpha('#9C27B0', 0.1) },
          }}
        >
          <FolderIcon fontSize={size} />
        </IconButton>
      </Tooltip>
    </Box>
  );

  // Main navigation legacy inline version removed

  

  return (
    <Box sx={{ width: 320, height: '100%', position: 'relative' }}>
      {/* Main Navigation */}
      <Paper
        elevation={0}
        sx={{
          width: '100%',
          height: '100%',
          borderRight: 1,
          borderColor: 'divider',
          bgcolor: 'background.paper',
        }}
      >
        <MainNavigation
          organizations={organizations}
          personalGroups={personalGroups}
          personalUsers={personalUsers}
          showOrganizations={showOrganizations}
          setShowOrganizations={setShowOrganizations}
          onOrganizationSelected={handleOrganizationClick}
          onNavigate={onNavigate}
          nav={nav}
          onVideoCall={onVideoCall}
          onAudioCall={onAudioCall}
          onScreenShare={onScreenShare}
          onOpenFiles={onOpenFiles}
        />
      </Paper>

      {/* Organization Navigation Overlay */}
      <Slide direction="right" in={!!selectedOrganization} mountOnEnter unmountOnExit>
        <Paper
          elevation={4}
          sx={{
            position: 'absolute',
            top: 0,
            left: 0,
            width: '100%',
            height: '100%',
            zIndex: 10,
            bgcolor: 'background.paper',
          }}
        >
          {selectedOrganization && (
            <OrganizationNavigation
              org={selectedOrganization}
              expandedSections={expandedSections}
              toggleSection={toggleSection}
              onNavigate={onNavigate}
              nav={nav}
              onVideoCall={onVideoCall}
              onAudioCall={onAudioCall}
              onScreenShare={onScreenShare}
              onOpenFiles={onOpenFiles}
              onBack={handleBackToMain}
            />
          )}
        </Paper>
      </Slide>
    </Box>
  );
};
