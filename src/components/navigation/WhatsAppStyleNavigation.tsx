import React, { useMemo, useState } from 'react';
import {
  Box,
  Paper,
  List,
  ListItemButton,
  ListItemText,
  ListItemSecondaryAction,
  IconButton,
  Tooltip,
  Divider,
  Typography,
  Stack,
  Collapse,
  ListSubheader,
} from '@mui/material';
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
} from '@mui/icons-material';
import { useNavigation } from '../../contexts/NavigationContext';
import { useEntityDirectory } from '../../contexts/EntityDirectoryContext';
import { Organization, Group, PersonalUser, Channel, Project } from '../../types/collaboration';
import { EntitySyncIndicator } from '../sync/EntitySyncIndicator';

interface WhatsAppStyleNavigationProps {
  currentUserId: string;
  onNavigate: (path: string, entity: any) => void;
  onVideoCall?: (entityId: string, entityType: string) => void;
  onAudioCall?: (entityId: string, entityType: string) => void;
  onScreenShare?: (entityId: string, entityType: string) => void;
  onOpenFiles?: (entityId: string, entityType: string) => void;
}
const ActionButtons: React.FC<{
  entityId: string;
  entityType: string;
  onVideoCall?: (entityId: string, entityType: string) => void;
  onAudioCall?: (entityId: string, entityType: string) => void;
  onScreenShare?: (entityId: string, entityType: string) => void;
  onOpenFiles?: (entityId: string, entityType: string) => void;
}> = ({ entityId, entityType, onVideoCall, onAudioCall, onScreenShare, onOpenFiles }) => (
  <Stack direction="row" spacing={0.5} alignItems="center">
    {onVideoCall && (
      <Tooltip title="Start video call">
        <IconButton size="small" onClick={(event) => { event.stopPropagation(); onVideoCall(entityId, entityType); }}>
          <VideoIcon fontSize="small" />
        </IconButton>
      </Tooltip>
    )}
    {onAudioCall && (
      <Tooltip title="Start audio call">
        <IconButton size="small" onClick={(event) => { event.stopPropagation(); onAudioCall(entityId, entityType); }}>
          <CallIcon fontSize="small" />
        </IconButton>
      </Tooltip>
    )}
    {onScreenShare && (
      <Tooltip title="Share screen">
        <IconButton size="small" onClick={(event) => { event.stopPropagation(); onScreenShare(entityId, entityType); }}>
          <ScreenIcon fontSize="small" />
        </IconButton>
      </Tooltip>
    )}
    {onOpenFiles && (
      <Tooltip title="Open files">
        <IconButton size="small" onClick={(event) => { event.stopPropagation(); onOpenFiles(entityId, entityType); }}>
          <FolderIcon fontSize="small" />
        </IconButton>
      </Tooltip>
    )}
  </Stack>
);

const promptForName = (label: string) => {
  const value = window.prompt(label);
  return value ? value.trim() : '';
};
export const WhatsAppStyleNavigation: React.FC<WhatsAppStyleNavigationProps> = ({
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
    addOrganizationGroup,
    removeOrganizationGroup,
    addOrganizationChannel,
    removeOrganizationChannel,
    addProject,
    removeProject,
    addPersonalGroup,
    removePersonalGroup,
    addPersonalUser,
    removePersonalUser,
    resetDirectory,
  } = useEntityDirectory();

  const [expandedOrganizationId, setExpandedOrganizationId] = useState<string | null>(null);

  const sortedPersonalGroups = useMemo(
    () => [...personalGroups].sort((a, b) => a.name.localeCompare(b.name)),
    [personalGroups]
  );
  const sortedContacts = useMemo(
    () => [...personalUsers].sort((a, b) => a.name.localeCompare(b.name)),
    [personalUsers]
  );
  const sortedOrganizations = useMemo(
    () => [...organizations].sort((a, b) => a.name.localeCompare(b.name)),
    [organizations]
  );

  const ensureExpanded = (organizationId: string) => {
    setExpandedOrganizationId(prev => (prev === organizationId ? prev : organizationId));
  };

  const handleNavigateGroup = (group: Group, organization?: Organization) => {
    if (organization) {
      nav.switchToOrganization(organization.id, organization.name);
      nav.selectEntity('group', group.id, group.name);
      onNavigate(`/org/${organization.id}/group/${group.id}`, { organization, group });
    } else {
      nav.switchToPersonal();
      nav.selectEntity('group', group.id, group.name);
      onNavigate(`/group/${group.id}`, group);
    }
  };

  const handleNavigateChannel = (organization: Organization, channel: Channel) => {
    nav.switchToOrganization(organization.id, organization.name);
    nav.selectEntity('channel', channel.id, channel.name);
    onNavigate(`/org/${organization.id}/channel/${channel.id}`, { organization, channel });
  };

  const handleNavigateProject = (organization: Organization, project: Project) => {
    nav.switchToOrganization(organization.id, organization.name);
    nav.selectEntity('project', project.id, project.name);
    onNavigate(`/org/${organization.id}/project/${project.id}`, { organization, project });
  };

  const handleNavigateOrganization = (organization: Organization) => {
    nav.switchToOrganization(organization.id, organization.name);
    onNavigate(`/org/${organization.id}`, organization);
  };

  const handleNavigateContact = (contact: PersonalUser) => {
    nav.switchToPersonal();
    nav.selectEntity('individual', contact.id, contact.name);
    onNavigate(`/user/${contact.id}`, contact);
  };

  const createPersonalGroup = () => {
    const name = promptForName('Enter personal group name');
    if (!name) return;
    const group = addPersonalGroup({ name });
    handleNavigateGroup(group);
  };

  const createContact = () => {
    const name = promptForName('Enter contact name');
    if (!name) return;
    const contact = addPersonalUser({ name });
    handleNavigateContact(contact);
  };

  const createOrganization = () => {
    const name = promptForName('Enter organization name');
    if (!name) return;
    const organization = addOrganization({ name });
    ensureExpanded(organization.id);
    handleNavigateOrganization(organization);
  };

  const createOrganizationChannel = (organization: Organization) => {
    const name = promptForName(`Channel name for ${organization.name}`);
    if (!name) return;
    const channel = addOrganizationChannel({ organizationId: organization.id, name });
    ensureExpanded(organization.id);
    handleNavigateChannel(organization, channel);
  };

  const createOrganizationGroup = (organization: Organization) => {
    const name = promptForName(`Group name for ${organization.name}`);
    if (!name) return;
    const group = addOrganizationGroup({ organizationId: organization.id, name });
    ensureExpanded(organization.id);
    handleNavigateGroup(group, organization);
  };

  const createProject = (organization: Organization) => {
    const name = promptForName(`Project name for ${organization.name}`);
    if (!name) return;
    const project = addProject({ organizationId: organization.id, name });
    ensureExpanded(organization.id);
    handleNavigateProject(organization, project);
  };

  return (
    <Paper
      elevation={0}
      sx={{
        width: 320,
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        borderRight: theme => `1px solid ${theme.palette.divider}`,
      }}
    >
      <Box sx={{ p: 2, pb: 1 }}>
        <Stack direction="row" alignItems="center" justifyContent="space-between">
          <Typography variant="h6" fontWeight={600}>
            Entities
          </Typography>
          <Stack direction="row" spacing={1}>
            <Tooltip title="Create organization">
              <IconButton size="small" onClick={createOrganization}>
                <OrganizationIcon />
              </IconButton>
            </Tooltip>
            <Tooltip title="Reset directory">
              <IconButton size="small" onClick={() => resetDirectory()}>
                <DeleteIcon />
              </IconButton>
            </Tooltip>
          </Stack>
        </Stack>
      </Box>

      <Divider />

      <List
        dense
        subheader={
          <ListSubheader disableGutters component="div" sx={{ px: 2, py: 1 }}>
            <Stack direction="row" alignItems="center" justifyContent="space-between">
              <Stack direction="row" spacing={1} alignItems="center">
                <GroupIcon fontSize="small" />
                <Typography variant="subtitle2">Personal Groups</Typography>
              </Stack>
              <Tooltip title="Add personal group">
                <IconButton size="small" onClick={createPersonalGroup}>
                  <AddIcon fontSize="small" />
                </IconButton>
              </Tooltip>
            </Stack>
          </ListSubheader>
        }
      >
        {sortedPersonalGroups.length === 0 && (
          <ListItemButton disabled>
            <ListItemText primary="No personal groups yet" />
          </ListItemButton>
        )}
        {sortedPersonalGroups.map(group => (
          <ListItemButton key={group.id} onClick={() => handleNavigateGroup(group)}>
            <ListItemText
              primary={
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  {group.name}
                  <EntitySyncIndicator
                    syncStatus={group.syncStatus}
                    lastSyncedAt={group.lastSyncedAt}
                    syncError={group.syncError}
                    size="small"
                    variant="icon"
                  />
                </Box>
              }
              secondary={group.networkIdentity.fourWords}
            />
            <ListItemSecondaryAction>
              <Stack direction="row" spacing={0.5} alignItems="center">
                <ActionButtons
                  entityId={group.id}
                  entityType="group"
                  onVideoCall={onVideoCall}
                  onAudioCall={onAudioCall}
                  onScreenShare={onScreenShare}
                  onOpenFiles={onOpenFiles}
                />
                <Tooltip title="Remove group">
                  <IconButton
                    size="small"
                    edge="end"
                    onClick={(event) => {
                      event.stopPropagation();
                      removePersonalGroup(group.id);
                    }}
                  >
                    <DeleteIcon fontSize="small" />
                  </IconButton>
                </Tooltip>
              </Stack>
            </ListItemSecondaryAction>
          </ListItemButton>
        ))}
      </List>

      <Divider />

      <List
        dense
        subheader={
          <ListSubheader disableGutters component="div" sx={{ px: 2, py: 1 }}>
            <Stack direction="row" alignItems="center" justifyContent="space-between">
              <Stack direction="row" spacing={1} alignItems="center">
                <PersonIcon fontSize="small" />
                <Typography variant="subtitle2">Contacts</Typography>
              </Stack>
              <Tooltip title="Add contact">
                <IconButton size="small" onClick={createContact}>
                  <AddIcon fontSize="small" />
                </IconButton>
              </Tooltip>
            </Stack>
          </ListSubheader>
        }
      >
        {sortedContacts.length === 0 && (
          <ListItemButton disabled>
            <ListItemText primary="No contacts yet" />
          </ListItemButton>
        )}
        {sortedContacts.map(contact => (
          <ListItemButton key={contact.id} onClick={() => handleNavigateContact(contact)}>
            <ListItemText
              primary={
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  {contact.name}
                  <EntitySyncIndicator
                    syncStatus={contact.syncStatus}
                    lastSyncedAt={contact.lastSyncedAt}
                    syncError={contact.syncError}
                    size="small"
                    variant="icon"
                  />
                </Box>
              }
              secondary={contact.networkIdentity.fourWords}
            />
            <ListItemSecondaryAction>
              <Stack direction="row" spacing={0.5} alignItems="center">
                <ActionButtons
                  entityId={contact.id}
                  entityType="user"
                  onVideoCall={onVideoCall}
                  onAudioCall={onAudioCall}
                  onScreenShare={onScreenShare}
                  onOpenFiles={onOpenFiles}
                />
                <Tooltip title="Remove contact">
                  <IconButton
                    size="small"
                    edge="end"
                    onClick={(event) => {
                      event.stopPropagation();
                      removePersonalUser(contact.id);
                    }}
                  >
                    <DeleteIcon fontSize="small" />
                  </IconButton>
                </Tooltip>
              </Stack>
            </ListItemSecondaryAction>
          </ListItemButton>
        ))}
      </List>

      <Divider />

      <List
        dense
        subheader={
          <ListSubheader disableGutters component="div" sx={{ px: 2, py: 1 }}>
            <Stack direction="row" alignItems="center" justifyContent="space-between">
              <Stack direction="row" spacing={1} alignItems="center">
                <OrganizationIcon fontSize="small" />
                <Typography variant="subtitle2">Organizations</Typography>
              </Stack>
              <Tooltip title="Create organization">
                <IconButton size="small" onClick={createOrganization}>
                  <AddIcon fontSize="small" />
                </IconButton>
              </Tooltip>
            </Stack>
          </ListSubheader>
        }
        sx={{ flex: 1, overflowY: 'auto' }}
      >
        {sortedOrganizations.length === 0 && (
          <ListItemButton disabled>
            <ListItemText primary="No organizations yet" />
          </ListItemButton>
        )}
        {sortedOrganizations.map(organization => {
          const expanded = expandedOrganizationId === organization.id;
          return (
            <Box key={organization.id}>
              <ListItemButton
                onClick={() => {
                  ensureExpanded(organization.id);
                  handleNavigateOrganization(organization);
                }}
              >
                <ListItemText
                  primary={organization.name}
                  secondary={organization.networkIdentity.fourWords}
                />
                <ListItemSecondaryAction>
                  <Stack direction="row" spacing={0.5} alignItems="center">
                    <IconButton size="small" onClick={(event) => { event.stopPropagation(); setExpandedOrganizationId(expanded ? null : organization.id); }}>
                      {expanded ? <ExpandLess fontSize="small" /> : <ExpandMore fontSize="small" />}
                    </IconButton>
                    <Tooltip title="Remove organization">
                      <IconButton
                        size="small"
                        onClick={(event) => {
                          event.stopPropagation();
                          removeOrganization(organization.id);
                          if (expandedOrganizationId === organization.id) {
                            setExpandedOrganizationId(null);
                          }
                        }}
                      >
                        <DeleteIcon fontSize="small" />
                      </IconButton>
                    </Tooltip>
                  </Stack>
                </ListItemSecondaryAction>
              </ListItemButton>
              <Collapse in={expanded} timeout="auto" unmountOnExit>
                <Box sx={{ pl: 2 }}>
                  <Typography variant="caption" sx={{ pl: 1, pt: 1, display: 'block', fontWeight: 600 }}>
                    Channels
                  </Typography>
                  <List dense disablePadding>
                    {organization.channels.map(channel => (
                      <ListItemButton key={channel.id} sx={{ pl: 2 }} onClick={() => handleNavigateChannel(organization, channel)}>
                        <ListItemText primary={channel.name} secondary={channel.networkIdentity.fourWords} />
                        <ListItemSecondaryAction>
                          <Stack direction="row" spacing={0.5} alignItems="center">
                            <ActionButtons
                              entityId={channel.id}
                              entityType="channel"
                              onVideoCall={onVideoCall}
                              onAudioCall={onAudioCall}
                              onScreenShare={onScreenShare}
                              onOpenFiles={onOpenFiles}
                            />
                            <Tooltip title="Remove channel">
                              <IconButton size="small" onClick={(event) => { event.stopPropagation(); removeOrganizationChannel(organization.id, channel.id); }}>
                                <DeleteIcon fontSize="small" />
                              </IconButton>
                            </Tooltip>
                          </Stack>
                        </ListItemSecondaryAction>
                      </ListItemButton>
                    ))}
                    <ListItemButton sx={{ pl: 2 }} onClick={(event) => { event.stopPropagation(); createOrganizationChannel(organization); }}>
                      <ListItemText primary="Add channel" />
                      <AddIcon fontSize="small" />
                    </ListItemButton>
                  </List>

                  <Typography variant="caption" sx={{ pl: 1, pt: 2, display: 'block', fontWeight: 600 }}>
                    Groups
                  </Typography>
                  <List dense disablePadding>
                    {organization.groups.map(group => (
                      <ListItemButton key={group.id} sx={{ pl: 2 }} onClick={() => handleNavigateGroup(group, organization)}>
                        <ListItemText primary={group.name} secondary={group.networkIdentity.fourWords} />
                        <ListItemSecondaryAction>
                          <Stack direction="row" spacing={0.5} alignItems="center">
                            <ActionButtons
                              entityId={group.id}
                              entityType="group"
                              onVideoCall={onVideoCall}
                              onAudioCall={onAudioCall}
                              onScreenShare={onScreenShare}
                              onOpenFiles={onOpenFiles}
                            />
                            <Tooltip title="Remove group">
                              <IconButton size="small" onClick={(event) => { event.stopPropagation(); removeOrganizationGroup(organization.id, group.id); }}>
                                <DeleteIcon fontSize="small" />
                              </IconButton>
                            </Tooltip>
                          </Stack>
                        </ListItemSecondaryAction>
                      </ListItemButton>
                    ))}
                    <ListItemButton sx={{ pl: 2 }} onClick={(event) => { event.stopPropagation(); createOrganizationGroup(organization); }}>
                      <ListItemText primary="Add group" />
                      <AddIcon fontSize="small" />
                    </ListItemButton>
                  </List>

                  <Typography variant="caption" sx={{ pl: 1, pt: 2, display: 'block', fontWeight: 600 }}>
                    Projects
                  </Typography>
                  <List dense disablePadding>
                    {organization.projects.map(project => (
                      <ListItemButton key={project.id} sx={{ pl: 2 }} onClick={() => handleNavigateProject(organization, project)}>
                        <ListItemText primary={project.name} secondary={project.networkIdentity.fourWords} />
                        <ListItemSecondaryAction>
                          <Tooltip title="Remove project">
                            <IconButton size="small" onClick={(event) => { event.stopPropagation(); removeProject(organization.id, project.id); }}>
                              <DeleteIcon fontSize="small" />
                            </IconButton>
                          </Tooltip>
                        </ListItemSecondaryAction>
                      </ListItemButton>
                    ))}
                    <ListItemButton sx={{ pl: 2 }} onClick={(event) => { event.stopPropagation(); createProject(organization); }}>
                      <ListItemText primary="Add project" />
                      <AddIcon fontSize="small" />
                    </ListItemButton>
                  </List>
                </Box>
              </Collapse>
              <Divider />
            </Box>
          );
        })}
      </List>
    </Paper>
  );
};

export default WhatsAppStyleNavigation;
