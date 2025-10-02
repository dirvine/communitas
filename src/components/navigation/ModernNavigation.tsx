import React, { useMemo, useState } from 'react';
import {
  Box,
  List,
  ListItemButton,
  ListItemText,
  IconButton,
  Tooltip,
  Typography,
  Stack,
  Collapse,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Avatar,
  Badge,
  Chip,
  alpha,
} from '@mui/material';
import { styled } from '@mui/material/styles';
import {
  Add as AddIcon,
  Delete as DeleteIcon,
  Business as OrganizationIcon,
  Groups as GroupIcon,
  Person as PersonIcon,
  Folder as FolderIcon,
  Tag as ChannelIcon,
  Assignment as ProjectIcon,
  ExpandMore,
  ExpandLess,
  VideoCall as VideoIcon,
  Call as CallIcon,
  ScreenShare as ScreenIcon,
  Search as SearchIcon,
  MoreVert as MoreIcon,
  Circle as OnlineIcon,
} from '@mui/icons-material';
import { useNavigation } from '../../contexts/NavigationContext';
import { useEntityDirectory } from '../../contexts/EntityDirectoryContext';
import { Organization, Group, PersonalUser, Channel, Project } from '../../types/collaboration';
import { EntitySyncIndicator } from '../sync/EntitySyncIndicator';
import { GlassCard } from '../ui/GlassCard';
import { ModernButton } from '../ui/ModernButton';
import { SearchInput } from '../ui/ModernInput';
import { EnhancedEntityDialog } from '../entity/EnhancedEntityDialog';
import { designTokens } from '../../styles/theme';

// Styled Components
const NavigationContainer = styled(GlassCard)(({ theme }) => ({
  width: 320,
  height: '100%',
  borderRadius: 0,
  borderTopRightRadius: designTokens.borderRadius.xl,
  borderBottomRightRadius: designTokens.borderRadius.xl,
  display: 'flex',
  flexDirection: 'column',
  overflow: 'hidden',
  background: theme.palette.mode === 'light'
    ? 'rgba(255, 255, 255, 0.95)'
    : 'rgba(17, 25, 40, 0.95)',
  backdropFilter: 'blur(20px) saturate(180%)',
  borderRight: `1px solid ${alpha(theme.palette.divider, 0.1)}`,
}));

const NavigationHeader = styled(Box)(({ theme }) => ({
  padding: theme.spacing(2.5),
  borderBottom: `1px solid ${alpha(theme.palette.divider, 0.1)}`,
  background: `linear-gradient(135deg, ${alpha(theme.palette.primary.main, 0.05)} 0%, ${alpha(theme.palette.secondary.main, 0.05)} 100%)`,
}));

const SectionHeader = styled(Box)(({ theme }) => ({
  padding: theme.spacing(1.5, 2),
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  background: alpha(theme.palette.background.default, 0.5),
  borderRadius: designTokens.borderRadius.md,
  margin: theme.spacing(1),
  transition: `all ${designTokens.transitions.normal}`,
  cursor: 'pointer',

  '&:hover': {
    background: alpha(theme.palette.primary.main, 0.05),
  },
}));

const StyledListItemButton = styled(ListItemButton)(({ theme }) => ({
  borderRadius: designTokens.borderRadius.md,
  margin: theme.spacing(0.5, 1),
  padding: theme.spacing(1.5),
  transition: `all ${designTokens.transitions.normal}`,

  '&:hover': {
    background: alpha(theme.palette.primary.main, 0.08),
    transform: 'translateX(4px)',

    // Show action buttons on hover
    '& .MuiIconButton-root': {
      opacity: 1,
      transform: 'scale(1)',
    },
  },

  '&.Mui-selected': {
    background: `linear-gradient(135deg, ${alpha(theme.palette.primary.main, 0.15)} 0%, ${alpha(theme.palette.primary.light, 0.1)} 100%)`,
    borderLeft: `3px solid ${theme.palette.primary.main}`,

    '&:hover': {
      background: `linear-gradient(135deg, ${alpha(theme.palette.primary.main, 0.2)} 0%, ${alpha(theme.palette.primary.light, 0.15)} 100%)`,
    },
  },
}));

const EntityAvatar = styled(Avatar)(({ theme }) => ({
  width: 36,
  height: 36,
  fontSize: '0.875rem',
  fontWeight: 600,
  background: designTokens.colors.primary.gradient,
  boxShadow: designTokens.shadows.sm,
}));

const OnlineIndicator = styled(Badge)(({ theme }) => ({
  '& .MuiBadge-badge': {
    backgroundColor: '#44b700',
    color: '#44b700',
    boxShadow: `0 0 0 2px ${theme.palette.background.paper}`,
    '&::after': {
      position: 'absolute',
      top: 0,
      left: 0,
      width: '100%',
      height: '100%',
      borderRadius: '50%',
      animation: 'ripple 1.2s infinite ease-in-out',
      border: '1px solid currentColor',
      content: '""',
    },
  },
  '@keyframes ripple': {
    '0%': {
      transform: 'scale(.8)',
      opacity: 1,
    },
    '100%': {
      transform: 'scale(2.4)',
      opacity: 0,
    },
  },
}));


const ActionButton = styled(IconButton)(({ theme }) => ({
  padding: theme.spacing(0.75),
  background: alpha(theme.palette.background.paper, 0.8),
  backdropFilter: 'blur(10px)',
  transition: `all ${designTokens.transitions.fast}`,
  opacity: 0, // Hidden by default
  transform: 'scale(0.8)',

  '&:hover': {
    background: alpha(theme.palette.primary.main, 0.1),
    transform: 'scale(1.1)',
  },
}));

interface ModernNavigationProps {
  currentUserId: string;
  onNavigate: (path: string, entity: any) => void;
  onVideoCall?: (entityId: string, entityType: string) => void;
  onAudioCall?: (entityId: string, entityType: string) => void;
  onScreenShare?: (entityId: string, entityType: string) => void;
  onOpenFiles?: (entityId: string, entityType: string) => void;
}

export const ModernNavigation: React.FC<ModernNavigationProps> = ({
  currentUserId,
  onNavigate,
  onVideoCall,
  onAudioCall,
  onScreenShare,
  onOpenFiles,
}) => {
  const nav = useNavigation();
  const {
    organizations,
    personalGroups,
    personalUsers,
    addOrganization,
    removeOrganization,
    addOrganizationChannel,
    removeOrganizationChannel,
    addOrganizationGroup,
    removeOrganizationGroup,
    addProject,
    removeProject,
    addPersonalGroup,
    removePersonalGroup,
    addPersonalUser,
    removePersonalUser,
  } = useEntityDirectory();

  const [searchQuery, setSearchQuery] = useState('');
  const [expandedSections, setExpandedSections] = useState<Record<string, boolean>>({
    organizations: true,
    groups: true,
    contacts: true,
  });
  const [dialogOpen, setDialogOpen] = useState(false);
  const [dialogEntityType, setDialogEntityType] = useState<'organization' | 'group' | 'contact'>('contact');

  const toggleSection = (section: string) => {
    setExpandedSections(prev => ({
      ...prev,
      [section]: !prev[section],
    }));
  };

  const openEntityDialog = (type: 'organization' | 'group' | 'contact') => {
    setDialogEntityType(type);
    setDialogOpen(true);
  };

  const handleNavigateContact = (contact: PersonalUser) => {
    nav.selectEntity('individual', contact.id, contact.name);
    onNavigate(`/user/${contact.id}`, contact);
  };

  // Filter entities based on search
  const filteredEntities = useMemo(() => {
    if (!searchQuery) {
      return { organizations, personalGroups, personalUsers };
    }

    const query = searchQuery.toLowerCase();
    return {
      organizations: organizations.filter(org =>
        org.name.toLowerCase().includes(query)
      ),
      personalGroups: personalGroups.filter(group =>
        group.name.toLowerCase().includes(query)
      ),
      personalUsers: personalUsers.filter(user =>
        user.name.toLowerCase().includes(query)
      ),
    };
  }, [organizations, personalGroups, personalUsers, searchQuery]);

  return (
    <NavigationContainer variant="light" hover={false}>
      <NavigationHeader>
        <Typography variant="h6" fontWeight={700} gutterBottom>
          Communitas
        </Typography>
        <SearchInput
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder="Search conversations..."
          icon={<SearchIcon />}
        />
      </NavigationHeader>

      <Box sx={{ flex: 1, overflowY: 'auto', position: 'relative' }}>
        {/* Organizations Section */}
        <SectionHeader onClick={() => toggleSection('organizations')}>
          <Stack direction="row" spacing={1} alignItems="center">
            <OrganizationIcon fontSize="small" color="primary" />
            <Typography variant="subtitle2" fontWeight={600}>
              Organizations
            </Typography>
            <Chip label={filteredEntities.organizations.length} size="small" />
          </Stack>
          <Stack direction="row" spacing={0.5} alignItems="center">
            <Tooltip title="Create Organization">
              <IconButton
                size="small"
                onClick={(e) => {
                  e.stopPropagation();
                  openEntityDialog('organization');
                }}
                sx={{ opacity: 0.7, '&:hover': { opacity: 1 } }}
              >
                <AddIcon fontSize="small" />
              </IconButton>
            </Tooltip>
            {expandedSections.organizations ? <ExpandLess /> : <ExpandMore />}
          </Stack>
        </SectionHeader>

        <Collapse in={expandedSections.organizations}>
          <List dense>
            {filteredEntities.organizations.map((org) => (
              <StyledListItemButton
                key={org.id}
                onClick={() => onNavigate('/organization', org)}
              >
                <Stack direction="row" spacing={2} alignItems="center" width="100%">
                  <EntityAvatar>
                    <OrganizationIcon fontSize="small" />
                  </EntityAvatar>
                  <Box flex={1}>
                    <Typography variant="body2" fontWeight={500}>
                      {org.name}
                    </Typography>
                    <Typography variant="caption" color="text.secondary">
                      {org.users.length} members
                    </Typography>
                  </Box>
                  <EntitySyncIndicator entityType="organization" entityId={org.id} />
                </Stack>
              </StyledListItemButton>
            ))}
          </List>
        </Collapse>

        {/* Groups Section */}
        <SectionHeader onClick={() => toggleSection('groups')}>
          <Stack direction="row" spacing={1} alignItems="center">
            <GroupIcon fontSize="small" color="primary" />
            <Typography variant="subtitle2" fontWeight={600}>
              Groups
            </Typography>
            <Chip label={filteredEntities.personalGroups.length} size="small" />
          </Stack>
          <Stack direction="row" spacing={0.5} alignItems="center">
            <Tooltip title="Create Group">
              <IconButton
                size="small"
                onClick={(e) => {
                  e.stopPropagation();
                  openEntityDialog('group');
                }}
                sx={{ opacity: 0.7, '&:hover': { opacity: 1 } }}
              >
                <AddIcon fontSize="small" />
              </IconButton>
            </Tooltip>
            {expandedSections.groups ? <ExpandLess /> : <ExpandMore />}
          </Stack>
        </SectionHeader>

        <Collapse in={expandedSections.groups}>
          <List dense>
            {filteredEntities.personalGroups.map((group) => (
              <StyledListItemButton
                key={group.id}
                onClick={() => onNavigate('/group', group)}
              >
                <Stack direction="row" spacing={2} alignItems="center" width="100%">
                  <EntityAvatar>
                    <GroupIcon fontSize="small" />
                  </EntityAvatar>
                  <Box flex={1}>
                    <Typography variant="body2" fontWeight={500}>
                      {group.name}
                    </Typography>
                    <Typography variant="caption" color="text.secondary">
                      {group.members.length} members
                    </Typography>
                  </Box>
                  <Stack direction="row" spacing={0.5}>
                    {onVideoCall && (
                      <ActionButton
                        size="small"
                        onClick={(e) => {
                          e.stopPropagation();
                          onVideoCall(group.id, 'group');
                        }}
                      >
                        <VideoIcon fontSize="small" />
                      </ActionButton>
                    )}
                    <EntitySyncIndicator entityType="group" entityId={group.id} />
                  </Stack>
                </Stack>
              </StyledListItemButton>
            ))}
          </List>
        </Collapse>

        {/* Contacts Section */}
        <SectionHeader onClick={() => toggleSection('contacts')}>
          <Stack direction="row" spacing={1} alignItems="center">
            <PersonIcon fontSize="small" color="primary" />
            <Typography variant="subtitle2" fontWeight={600}>
              Contacts
            </Typography>
            <Chip label={filteredEntities.personalUsers.length} size="small" />
          </Stack>
          <Stack direction="row" spacing={0.5} alignItems="center">
            <Tooltip title="Add Contact">
              <IconButton
                size="small"
                onClick={(e) => {
                  e.stopPropagation();
                  openEntityDialog('contact');
                }}
                sx={{ opacity: 0.7, '&:hover': { opacity: 1 } }}
              >
                <AddIcon fontSize="small" />
              </IconButton>
            </Tooltip>
            {expandedSections.contacts ? <ExpandLess /> : <ExpandMore />}
          </Stack>
        </SectionHeader>

        <Collapse in={expandedSections.contacts}>
          <List dense>
            {filteredEntities.personalUsers.map((user) => (
              <StyledListItemButton
                key={user.id}
                onClick={() => handleNavigateContact(user)}
              >
                <Stack direction="row" spacing={2} alignItems="center" width="100%">
                  <OnlineIndicator
                    overlap="circular"
                    anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                    variant="dot"
                  >
                    <EntityAvatar>
                      {user.name.charAt(0).toUpperCase()}
                    </EntityAvatar>
                  </OnlineIndicator>
                  <Box flex={1}>
                    <Typography variant="body2" fontWeight={500}>
                      {user.name}
                    </Typography>
                    <Typography variant="caption" color="text.secondary">
                      {user.relationship || 'Contact'}
                    </Typography>
                  </Box>
                  <Stack direction="row" spacing={0.5}>
                    {onVideoCall && (
                      <ActionButton
                        size="small"
                        onClick={(e) => {
                          e.stopPropagation();
                          onVideoCall(user.id, 'user');
                        }}
                      >
                        <VideoIcon fontSize="small" />
                      </ActionButton>
                    )}
                    {onAudioCall && (
                      <ActionButton
                        size="small"
                        onClick={(e) => {
                          e.stopPropagation();
                          onAudioCall(user.id, 'user');
                        }}
                      >
                        <CallIcon fontSize="small" />
                      </ActionButton>
                    )}
                  </Stack>
                </Stack>
              </StyledListItemButton>
            ))}
          </List>
        </Collapse>

      </Box>

      {/* Enhanced Entity Dialog */}
      <EnhancedEntityDialog
        open={dialogOpen}
        onClose={() => setDialogOpen(false)}
        entityType={dialogEntityType}
        isOnline={true} // TODO: Get from network status context
      />
    </NavigationContainer>
  );
};