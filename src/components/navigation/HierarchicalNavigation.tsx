import React, { useState } from 'react';
import {
  Box,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Collapse,
  Divider,
  Typography,
  Chip,
  IconButton,
  Avatar,
  Badge,
  Tooltip,
  alpha,
} from '@mui/material';
import {
  Business as OrganizationIcon,
  Groups as GroupsIcon,
  Person as PersonIcon,
  ExpandLess,
  ExpandMore,
  Tag as ChannelIcon,
  Assignment as ProjectIcon,
  Add as AddIcon,
  VideoCall as VideoIcon,
  ScreenShare as ScreenIcon,
  Folder as FolderIcon,
  Language as WebsiteIcon,
  Call as CallIcon,
} from '@mui/icons-material';
import { Organization, Group, PersonalUser, Channel, Project } from '../../types/collaboration';

interface HierarchicalNavigationProps {
  organizations: Organization[];
  personalGroups: Group[];
  personalUsers: PersonalUser[];
  currentPath: string;
  onNavigate: (path: string, entity: any) => void;
  onStartCall?: (entityId: string, type: 'video' | 'audio') => void;
  onScreenShare?: (entityId: string) => void;
  onOpenFiles?: (entityId: string) => void;
  onPublishWebsite?: (entityId: string) => void;
}

export const HierarchicalNavigation: React.FC<HierarchicalNavigationProps> = ({
  organizations,
  personalGroups,
  personalUsers,
  currentPath,
  onNavigate,
  onStartCall,
  onScreenShare,
  onOpenFiles,
  onPublishWebsite,
}) => {
  const [expandedOrgs, setExpandedOrgs] = useState<string[]>([]);
  const [expandedSections, setExpandedSections] = useState({
    organizations: true,
    personalGroups: true,
    personalUsers: true,
  });

  const toggleOrganization = (orgId: string) => {
    setExpandedOrgs(prev =>
      prev.includes(orgId)
        ? prev.filter(id => id !== orgId)
        : [...prev, orgId]
    );
  };

  const toggleSection = (section: keyof typeof expandedSections) => {
    setExpandedSections(prev => ({
      ...prev,
      [section]: !prev[section],
    }));
  };

  const CollaborationActions = ({ entityId }: { entityId: string }) => (
    <Box sx={{ display: 'flex', gap: 0.5 }}>
      <Tooltip title="Video Call">
        <IconButton
          size="small"
          onClick={(e) => {
            e.stopPropagation();
            onStartCall?.(entityId, 'video');
          }}
        >
          <VideoIcon fontSize="small" />
        </IconButton>
      </Tooltip>
      <Tooltip title="Audio Call">
        <IconButton
          size="small"
          onClick={(e) => {
            e.stopPropagation();
            onStartCall?.(entityId, 'audio');
          }}
        >
          <CallIcon fontSize="small" />
        </IconButton>
      </Tooltip>
      <Tooltip title="Screen Share">
        <IconButton
          size="small"
          onClick={(e) => {
            e.stopPropagation();
            onScreenShare?.(entityId);
          }}
        >
          <ScreenIcon fontSize="small" />
        </IconButton>
      </Tooltip>
      <Tooltip title="Files">
        <IconButton
          size="small"
          onClick={(e) => {
            e.stopPropagation();
            onOpenFiles?.(entityId);
          }}
        >
          <FolderIcon fontSize="small" />
        </IconButton>
      </Tooltip>
      <Tooltip title="Publish Website">
        <IconButton
          size="small"
          onClick={(e) => {
            e.stopPropagation();
            onPublishWebsite?.(entityId);
          }}
        >
          <WebsiteIcon fontSize="small" />
        </IconButton>
      </Tooltip>
    </Box>
  );

  return (
    <Box sx={{ width: '100%', maxWidth: 360 }}>
      {/* Organizations Section */}
      <List component="nav" dense>
        <ListItemButton onClick={() => toggleSection('organizations')}>
          <ListItemIcon>
            <OrganizationIcon />
          </ListItemIcon>
          <ListItemText 
            primary={
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <Typography variant="subtitle2" fontWeight={600}>
                  Organizations
                </Typography>
                <Chip label={organizations.length} size="small" />
              </Box>
            }
          />
          {expandedSections.organizations ? <ExpandLess /> : <ExpandMore />}
        </ListItemButton>
        
        <Collapse in={expandedSections.organizations} timeout="auto" unmountOnExit>
          <List component="div" disablePadding>
            {organizations.map(org => (
              <Box key={org.id}>
                <ListItemButton
                  sx={{ pl: 3 }}
                  selected={currentPath === `/org/${org.id}`}
                  onClick={() => onNavigate(`/org/${org.id}`, org)}
                >
                  <ListItemIcon>
                    <Avatar sx={{ width: 32, height: 32 }}>
                      {org.name[0]}
                    </Avatar>
                  </ListItemIcon>
                  <ListItemText 
                    primary={org.name}
                    secondary={org.networkIdentity.fourWords}
                  />
                  <IconButton
                    size="small"
                    onClick={(e) => {
                      e.stopPropagation();
                      toggleOrganization(org.id);
                    }}
                  >
                    {expandedOrgs.includes(org.id) ? <ExpandLess /> : <ExpandMore />}
                  </IconButton>
                </ListItemButton>

                <Collapse in={expandedOrgs.includes(org.id)} timeout="auto" unmountOnExit>
                  <List component="div" disablePadding>
                    {/* Channels */}
                    <ListItem sx={{ pl: 5 }}>
                      <ListItemIcon>
                        <ChannelIcon fontSize="small" />
                      </ListItemIcon>
                      <ListItemText 
                        primary={
                          <Typography variant="caption" color="text.secondary">
                            CHANNELS ({org.channels.length})
                          </Typography>
                        }
                      />
                    </ListItem>
                    {org.channels.map(channel => (
                      <ListItemButton
                        key={channel.id}
                        sx={{ pl: 6 }}
                        onClick={() => onNavigate(`/org/${org.id}/channel/${channel.id}`, channel)}
                      >
                        <ListItemText 
                          primary={`# ${channel.name}`}
                          secondary={
                            <Box sx={{ display: 'flex', alignItems: 'center', mt: 0.5 }}>
                              <CollaborationActions entityId={channel.id} />
                            </Box>
                          }
                        />
                      </ListItemButton>
                    ))}

                    {/* Groups */}
                    <ListItem sx={{ pl: 5 }}>
                      <ListItemIcon>
                        <GroupsIcon fontSize="small" />
                      </ListItemIcon>
                      <ListItemText 
                        primary={
                          <Typography variant="caption" color="text.secondary">
                            GROUPS ({org.groups.length})
                          </Typography>
                        }
                      />
                    </ListItem>
                    {org.groups.map(group => (
                      <ListItemButton
                        key={group.id}
                        sx={{ pl: 6 }}
                        onClick={() => onNavigate(`/org/${org.id}/group/${group.id}`, group)}
                      >
                        <ListItemText 
                          primary={group.name}
                          secondary={
                            <Box sx={{ display: 'flex', alignItems: 'center', mt: 0.5 }}>
                              <CollaborationActions entityId={group.id} />
                            </Box>
                          }
                        />
                      </ListItemButton>
                    ))}

                    {/* Projects */}
                    <ListItem sx={{ pl: 5 }}>
                      <ListItemIcon>
                        <ProjectIcon fontSize="small" />
                      </ListItemIcon>
                      <ListItemText 
                        primary={
                          <Typography variant="caption" color="text.secondary">
                            PROJECTS ({org.projects.length})
                          </Typography>
                        }
                      />
                    </ListItem>
                    {org.projects.map(project => (
                      <ListItemButton
                        key={project.id}
                        sx={{ pl: 6 }}
                        onClick={() => onNavigate(`/org/${org.id}/project/${project.id}`, project)}
                      >
                        <ListItemText 
                          primary={project.name}
                          secondary={
                            <Box sx={{ display: 'flex', alignItems: 'center', mt: 0.5 }}>
                              <Chip 
                                label={project.status} 
                                size="small" 
                                color={project.status === 'active' ? 'success' : 'default'}
                                sx={{ mr: 1 }}
                              />
                              <CollaborationActions entityId={project.id} />
                            </Box>
                          }
                        />
                      </ListItemButton>
                    ))}

                    {/* Organization Users */}
                    <ListItem sx={{ pl: 5 }}>
                      <ListItemIcon>
                        <PersonIcon fontSize="small" />
                      </ListItemIcon>
                      <ListItemText 
                        primary={
                          <Typography variant="caption" color="text.secondary">
                            MEMBERS ({org.users.length})
                          </Typography>
                        }
                      />
                    </ListItem>
                    {org.users.slice(0, 5).map(user => (
                      <ListItemButton
                        key={user.id}
                        sx={{ pl: 6 }}
                        onClick={() => onNavigate(`/org/${org.id}/user/${user.id}`, user)}
                      >
                        <ListItemIcon>
                          <Badge
                            variant="dot"
                            color="success"
                            invisible={Math.random() > 0.7}
                          >
                            <Avatar sx={{ width: 24, height: 24 }}>
                              {user.name[0]}
                            </Avatar>
                          </Badge>
                        </ListItemIcon>
                        <ListItemText 
                          primary={user.name}
                          secondary={
                            <Box sx={{ display: 'flex', alignItems: 'center', mt: 0.5 }}>
                              <CollaborationActions entityId={user.id} />
                            </Box>
                          }
                        />
                      </ListItemButton>
                    ))}
                    {org.users.length > 5 && (
                      <ListItem sx={{ pl: 6 }}>
                        <Typography variant="caption" color="text.secondary">
                          +{org.users.length - 5} more members
                        </Typography>
                      </ListItem>
                    )}
                  </List>
                </Collapse>
              </Box>
            ))}
            
            <ListItemButton sx={{ pl: 3 }}>
              <ListItemIcon>
                <AddIcon />
              </ListItemIcon>
              <ListItemText primary="Create Organization" />
            </ListItemButton>
          </List>
        </Collapse>
      </List>

      <Divider />

      {/* Personal Groups Section */}
      <List component="nav" dense>
        <ListItemButton onClick={() => toggleSection('personalGroups')}>
          <ListItemIcon>
            <GroupsIcon />
          </ListItemIcon>
          <ListItemText 
            primary={
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <Typography variant="subtitle2" fontWeight={600}>
                  Personal Groups
                </Typography>
                <Chip label={personalGroups.length} size="small" />
              </Box>
            }
          />
          {expandedSections.personalGroups ? <ExpandLess /> : <ExpandMore />}
        </ListItemButton>
        
        <Collapse in={expandedSections.personalGroups} timeout="auto" unmountOnExit>
          <List component="div" disablePadding>
            {personalGroups.map(group => (
              <ListItemButton
                key={group.id}
                sx={{ pl: 3 }}
                selected={currentPath === `/group/${group.id}`}
                onClick={() => onNavigate(`/group/${group.id}`, group)}
              >
                <ListItemIcon>
                  <Avatar sx={{ width: 32, height: 32 }}>
                    {group.name[0]}
                  </Avatar>
                </ListItemIcon>
                <ListItemText 
                  primary={group.name}
                  secondary={
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mt: 0.5 }}>
                      <Typography variant="caption">
                        {group.members.length} members
                      </Typography>
                      <CollaborationActions entityId={group.id} />
                    </Box>
                  }
                />
              </ListItemButton>
            ))}
            
            <ListItemButton sx={{ pl: 3 }}>
              <ListItemIcon>
                <AddIcon />
              </ListItemIcon>
              <ListItemText primary="Create Group" />
            </ListItemButton>
          </List>
        </Collapse>
      </List>

      <Divider />

      {/* Personal Users Section */}
      <List component="nav" dense>
        <ListItemButton onClick={() => toggleSection('personalUsers')}>
          <ListItemIcon>
            <PersonIcon />
          </ListItemIcon>
          <ListItemText 
            primary={
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <Typography variant="subtitle2" fontWeight={600}>
                  Personal Contacts
                </Typography>
                <Chip label={personalUsers.length} size="small" />
              </Box>
            }
          />
          {expandedSections.personalUsers ? <ExpandLess /> : <ExpandMore />}
        </ListItemButton>
        
        <Collapse in={expandedSections.personalUsers} timeout="auto" unmountOnExit>
          <List component="div" disablePadding>
            {personalUsers.map(user => (
              <ListItemButton
                key={user.id}
                sx={{ pl: 3 }}
                selected={currentPath === `/user/${user.id}`}
                onClick={() => onNavigate(`/user/${user.id}`, user)}
              >
                <ListItemIcon>
                  <Badge
                    variant="dot"
                    color="success"
                    invisible={Math.random() > 0.5}
                  >
                    <Avatar sx={{ width: 32, height: 32 }}>
                      {user.name[0]}
                    </Avatar>
                  </Badge>
                </ListItemIcon>
                <ListItemText 
                  primary={user.name}
                  secondary={
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mt: 0.5 }}>
                      <Typography variant="caption">
                        {user.relationship}
                      </Typography>
                      <CollaborationActions entityId={user.id} />
                    </Box>
                  }
                />
              </ListItemButton>
            ))}
            
            <ListItemButton sx={{ pl: 3 }}>
              <ListItemIcon>
                <AddIcon />
              </ListItemIcon>
              <ListItemText primary="Add Contact" />
            </ListItemButton>
          </List>
        </Collapse>
      </List>
    </Box>
  );
};