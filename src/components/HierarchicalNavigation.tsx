import React, { useState, useEffect, useCallback } from 'react';
import {
  Box,
  Drawer,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Collapse,
  IconButton,
  Typography,
  Avatar,
  Chip,
  Divider,
  TextField,
  InputAdornment,
  Paper,
  Breadcrumbs,
  Link,
  Button,
  Stack,
  Badge,
  Tooltip,
} from '@mui/material';
import {
  Home as HomeIcon,
  Business as BusinessIcon,
  Group as GroupIcon,
  Folder as FolderIcon,
  Person as PersonIcon,
  ExpandLess,
  ExpandMore,
  Search as SearchIcon,
  Add as AddIcon,
  Phone as PhoneIcon,
  Videocam as VideocamIcon,
  Storage as StorageIcon,
  NavigateNext as NavigateNextIcon,
  Dashboard as DashboardIcon,
  Chat as ChatIcon,
  Description as DescriptionIcon,
  Settings as SettingsIcon,
  PersonAdd as PersonAddIcon,
  CreateNewFolder as CreateNewFolderIcon,
  GroupAdd as GroupAddIcon,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { 
  Organization, 
  Group, 
  Project, 
  OrganizationHierarchy,
  CreateOrganizationRequest,
  CreateGroupRequest,
  CreateProjectRequest,
} from '../types/organization';

interface NavigationLevel {
  type: 'personal' | 'organization' | 'project' | 'group';
  id?: string;
  name: string;
  parent?: NavigationLevel;
}

interface HierarchicalNavigationProps {
  onNavigate: (level: NavigationLevel, entity?: Organization | Project | Group) => void;
  onCall: (entityType: string, entityId: string, callType: 'voice' | 'video') => void;
  currentUserId?: string;
}

export const HierarchicalNavigation: React.FC<HierarchicalNavigationProps> = ({
  onNavigate,
  onCall,
  currentUserId = 'user_default',
}) => {
  const [organizations, setOrganizations] = useState<Organization[]>([]);
  const [currentHierarchy, setCurrentHierarchy] = useState<OrganizationHierarchy | null>(null);
  const [currentLevel, setCurrentLevel] = useState<NavigationLevel>({ 
    type: 'personal', 
    name: 'Personal Space' 
  });
  const [expandedOrgs, setExpandedOrgs] = useState<Set<string>>(new Set());
  const [expandedProjects, setExpandedProjects] = useState<Set<string>>(new Set());
  const [searchQuery, setSearchQuery] = useState('');
  const [loading, setLoading] = useState(false);

  // Load user's organizations
  useEffect(() => {
    loadUserOrganizations();
  }, [currentUserId]);

  const loadUserOrganizations = async () => {
    try {
      setLoading(true);
      const orgs = await invoke<Organization[]>('get_user_organizations_dht', {
        userId: currentUserId,
      });
      setOrganizations(orgs);
    } catch (error) {
      console.error('Failed to load organizations:', error);
    } finally {
      setLoading(false);
    }
  };

  const loadOrganizationHierarchy = async (orgId: string) => {
    try {
      const hierarchy = await invoke<OrganizationHierarchy>('get_organization_hierarchy', {
        orgId,
      });
      setCurrentHierarchy(hierarchy);
    } catch (error) {
      console.error('Failed to load organization hierarchy:', error);
    }
  };

  const handleOrgToggle = (orgId: string) => {
    const newExpanded = new Set(expandedOrgs);
    if (newExpanded.has(orgId)) {
      newExpanded.delete(orgId);
    } else {
      newExpanded.add(orgId);
      loadOrganizationHierarchy(orgId);
    }
    setExpandedOrgs(newExpanded);
  };

  const handleProjectToggle = (projectId: string) => {
    const newExpanded = new Set(expandedProjects);
    if (newExpanded.has(projectId)) {
      newExpanded.delete(projectId);
    } else {
      newExpanded.add(projectId);
    }
    setExpandedProjects(newExpanded);
  };

  const handleNavigation = (level: NavigationLevel, entity?: Organization | Project | Group) => {
    setCurrentLevel(level);
    onNavigate(level, entity);
    
    // Load hierarchy if navigating to an organization
    if (level.type === 'organization' && level.id) {
      loadOrganizationHierarchy(level.id);
    }
  };

  const handleCreateOrganization = async () => {
    const name = prompt('Enter organization name:');
    if (!name) return;

    try {
      const request: CreateOrganizationRequest = {
        name,
        description: '',
        visibility: 'private',
        initial_storage_gb: 10,
      };
      
      const org = await invoke<Organization>('create_organization_dht', { request });
      await loadUserOrganizations();
      handleNavigation({ type: 'organization', id: org.id, name: org.name }, org);
    } catch (error) {
      console.error('Failed to create organization:', error);
    }
  };

  const handleCreateGroup = async (orgId: string) => {
    const name = prompt('Enter group name:');
    if (!name) return;

    try {
      const request: CreateGroupRequest = {
        name,
        organization_id: orgId,
      };
      
      await invoke('create_group_dht', { request });
      await loadOrganizationHierarchy(orgId);
    } catch (error) {
      console.error('Failed to create group:', error);
    }
  };

  const handleCreateProject = async (orgId: string) => {
    const name = prompt('Enter project name:');
    if (!name) return;

    try {
      const request: CreateProjectRequest = {
        name,
        organization_id: orgId,
        priority: 'medium',
        initial_storage_gb: 5,
      };
      
      await invoke('create_project_dht', { request });
      await loadOrganizationHierarchy(orgId);
    } catch (error) {
      console.error('Failed to create project:', error);
    }
  };

  const renderBreadcrumbs = () => {
    const breadcrumbs: NavigationLevel[] = [];
    let current: NavigationLevel | undefined = currentLevel;
    
    while (current) {
      breadcrumbs.unshift(current);
      current = current.parent;
    }

    return (
      <Breadcrumbs 
        separator={<NavigateNextIcon fontSize="small" />}
        sx={{ mb: 2 }}
      >
        {breadcrumbs.map((crumb, index) => {
          const isLast = index === breadcrumbs.length - 1;
          const Icon = crumb.type === 'personal' ? HomeIcon :
                       crumb.type === 'organization' ? BusinessIcon :
                       crumb.type === 'project' ? FolderIcon : GroupIcon;
          
          return isLast ? (
            <Typography key={crumb.id || crumb.type} color="text.primary" sx={{ display: 'flex', alignItems: 'center' }}>
              <Icon sx={{ mr: 0.5, fontSize: 20 }} />
              {crumb.name}
            </Typography>
          ) : (
            <Link
              key={crumb.id || crumb.type}
              color="inherit"
              href="#"
              onClick={() => handleNavigation(crumb)}
              sx={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}
            >
              <Icon sx={{ mr: 0.5, fontSize: 20 }} />
              {crumb.name}
            </Link>
          );
        })}
      </Breadcrumbs>
    );
  };

  const renderOrganizationItem = (org: Organization) => {
    const isExpanded = expandedOrgs.has(org.id);
    const hierarchy = currentHierarchy?.organization.id === org.id ? currentHierarchy : null;

    return (
      <Box key={org.id}>
        <ListItemButton 
          onClick={() => handleNavigation({ 
            type: 'organization', 
            id: org.id, 
            name: org.name,
            parent: { type: 'personal', name: 'Personal Space' }
          }, org)}
          sx={{ 
            pl: 2,
            backgroundColor: currentLevel.id === org.id ? 'action.selected' : 'transparent',
            '&:hover': { backgroundColor: 'action.hover' }
          }}
        >
          <ListItemIcon>
            <BusinessIcon color="primary" />
          </ListItemIcon>
          <ListItemText 
            primary={org.name}
            secondary={`${hierarchy?.total_members || 0} members • ${org.storage_quota.used_gb.toFixed(1)}GB used`}
          />
          <Stack direction="row" spacing={1}>
            <Tooltip title="Voice Call">
              <IconButton 
                size="small" 
                onClick={(e) => {
                  e.stopPropagation();
                  onCall('organization', org.id, 'voice');
                }}
              >
                <PhoneIcon fontSize="small" />
              </IconButton>
            </Tooltip>
            <Tooltip title="Video Call">
              <IconButton 
                size="small"
                onClick={(e) => {
                  e.stopPropagation();
                  onCall('organization', org.id, 'video');
                }}
              >
                <VideocamIcon fontSize="small" />
              </IconButton>
            </Tooltip>
            <Tooltip title="Storage">
              <IconButton size="small">
                <Badge 
                  badgeContent={`${org.storage_quota.used_gb.toFixed(0)}G`} 
                  color="secondary"
                >
                  <StorageIcon fontSize="small" />
                </Badge>
              </IconButton>
            </Tooltip>
            <IconButton
              size="small"
              onClick={(e) => {
                e.stopPropagation();
                handleOrgToggle(org.id);
              }}
            >
              {isExpanded ? <ExpandLess /> : <ExpandMore />}
            </IconButton>
          </Stack>
        </ListItemButton>

        <Collapse in={isExpanded} timeout="auto" unmountOnExit>
          <List component="div" disablePadding>
            {/* Projects Section */}
            {hierarchy && hierarchy.projects.length > 0 && (
              <>
                <ListItem sx={{ pl: 4, py: 0.5 }}>
                  <Typography variant="overline" color="text.secondary">
                    Projects ({hierarchy.projects.length})
                  </Typography>
                  <IconButton 
                    size="small" 
                    sx={{ ml: 'auto' }}
                    onClick={() => handleCreateProject(org.id)}
                  >
                    <AddIcon fontSize="small" />
                  </IconButton>
                </ListItem>
                {hierarchy.projects.map(project => (
                  <ListItemButton
                    key={project.id}
                    sx={{ pl: 5 }}
                    onClick={() => handleNavigation({
                      type: 'project',
                      id: project.id,
                      name: project.name,
                      parent: { 
                        type: 'organization', 
                        id: org.id, 
                        name: org.name,
                        parent: { type: 'personal', name: 'Personal Space' }
                      }
                    }, project)}
                  >
                    <ListItemIcon>
                      <FolderIcon fontSize="small" color="secondary" />
                    </ListItemIcon>
                    <ListItemText 
                      primary={project.name}
                      secondary={
                        <Stack direction="row" spacing={1} alignItems="center">
                          <Chip 
                            label={project.priority} 
                            size="small" 
                            color={project.priority === 'high' ? 'error' : 
                                   project.priority === 'critical' ? 'error' : 'default'}
                          />
                          <Typography variant="caption">
                            {project.storage_quota.used_gb.toFixed(1)}GB
                          </Typography>
                        </Stack>
                      }
                    />
                    <Stack direction="row" spacing={0.5}>
                      <IconButton size="small" onClick={(e) => {
                        e.stopPropagation();
                        onCall('project', project.id, 'voice');
                      }}>
                        <PhoneIcon fontSize="small" />
                      </IconButton>
                      <IconButton size="small" onClick={(e) => {
                        e.stopPropagation();
                        onCall('project', project.id, 'video');
                      }}>
                        <VideocamIcon fontSize="small" />
                      </IconButton>
                    </Stack>
                  </ListItemButton>
                ))}
              </>
            )}

            {/* Groups Section */}
            {hierarchy && hierarchy.groups.length > 0 && (
              <>
                <ListItem sx={{ pl: 4, py: 0.5 }}>
                  <Typography variant="overline" color="text.secondary">
                    Groups ({hierarchy.groups.length})
                  </Typography>
                  <IconButton 
                    size="small" 
                    sx={{ ml: 'auto' }}
                    onClick={() => handleCreateGroup(org.id)}
                  >
                    <GroupAddIcon fontSize="small" />
                  </IconButton>
                </ListItem>
                {hierarchy.groups.map(group => (
                  <ListItemButton
                    key={group.id}
                    sx={{ pl: 5 }}
                    onClick={() => handleNavigation({
                      type: 'group',
                      id: group.id,
                      name: group.name,
                      parent: { 
                        type: 'organization', 
                        id: org.id, 
                        name: org.name,
                        parent: { type: 'personal', name: 'Personal Space' }
                      }
                    }, group)}
                  >
                    <ListItemIcon>
                      <GroupIcon fontSize="small" color="action" />
                    </ListItemIcon>
                    <ListItemText 
                      primary={group.name}
                      secondary={`${group.members.length} members • Chat only`}
                    />
                    <Stack direction="row" spacing={0.5}>
                      <IconButton size="small" onClick={(e) => {
                        e.stopPropagation();
                        onCall('group', group.id, 'voice');
                      }}>
                        <PhoneIcon fontSize="small" />
                      </IconButton>
                      <IconButton size="small" onClick={(e) => {
                        e.stopPropagation();
                        onCall('group', group.id, 'video');
                      }}>
                        <VideocamIcon fontSize="small" />
                      </IconButton>
                    </Stack>
                  </ListItemButton>
                ))}
              </>
            )}

            {/* Add buttons when no items */}
            {hierarchy && hierarchy.projects.length === 0 && hierarchy.groups.length === 0 && (
              <Box sx={{ pl: 4, py: 2 }}>
                <Stack direction="row" spacing={1}>
                  <Button
                    startIcon={<CreateNewFolderIcon />}
                    size="small"
                    onClick={() => handleCreateProject(org.id)}
                  >
                    Add Project
                  </Button>
                  <Button
                    startIcon={<GroupAddIcon />}
                    size="small"
                    onClick={() => handleCreateGroup(org.id)}
                  >
                    Add Group
                  </Button>
                </Stack>
              </Box>
            )}
          </List>
        </Collapse>
      </Box>
    );
  };

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      {/* Search Bar */}
      <Box sx={{ p: 2 }}>
        <TextField
          fullWidth
          size="small"
          placeholder="Search organizations, projects, groups..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          InputProps={{
            startAdornment: (
              <InputAdornment position="start">
                <SearchIcon />
              </InputAdornment>
            ),
          }}
        />
      </Box>

      <Divider />

      {/* Breadcrumb Navigation */}
      <Box sx={{ px: 2, pt: 2 }}>
        {renderBreadcrumbs()}
      </Box>

      {/* Personal Space / Home */}
      <List sx={{ flexGrow: 1, overflow: 'auto' }}>
        <ListItemButton
          onClick={() => handleNavigation({ type: 'personal', name: 'Personal Space' })}
          selected={currentLevel.type === 'personal'}
        >
          <ListItemIcon>
            <HomeIcon color="primary" />
          </ListItemIcon>
          <ListItemText 
            primary="Personal Space"
            secondary="Your private workspace"
          />
        </ListItemButton>

        <Divider sx={{ my: 1 }} />

        {/* Organizations Section */}
        <ListItem sx={{ px: 2, py: 0.5 }}>
          <Typography variant="overline" color="text.secondary">
            Organizations ({organizations.length})
          </Typography>
          <Tooltip title="Create Organization">
            <IconButton 
              size="small" 
              sx={{ ml: 'auto' }}
              onClick={handleCreateOrganization}
            >
              <AddIcon />
            </IconButton>
          </Tooltip>
        </ListItem>

        {organizations.length === 0 ? (
          <Box sx={{ p: 3, textAlign: 'center' }}>
            <Typography variant="body2" color="text.secondary" gutterBottom>
              No organizations yet
            </Typography>
            <Button
              variant="outlined"
              startIcon={<AddIcon />}
              onClick={handleCreateOrganization}
              size="small"
            >
              Create Your First Organization
            </Button>
          </Box>
        ) : (
          organizations
            .filter(org => 
              searchQuery === '' || 
              org.name.toLowerCase().includes(searchQuery.toLowerCase())
            )
            .map(renderOrganizationItem)
        )}
      </List>

      <Divider />

      {/* Quick Actions */}
      <Box sx={{ p: 1 }}>
        <Stack direction="row" spacing={1} justifyContent="center">
          <Tooltip title="Dashboard">
            <IconButton size="small">
              <DashboardIcon />
            </IconButton>
          </Tooltip>
          <Tooltip title="Messages">
            <IconButton size="small">
              <Badge badgeContent={3} color="error">
                <ChatIcon />
              </Badge>
            </IconButton>
          </Tooltip>
          <Tooltip title="Files">
            <IconButton size="small">
              <DescriptionIcon />
            </IconButton>
          </Tooltip>
          <Tooltip title="Settings">
            <IconButton size="small">
              <SettingsIcon />
            </IconButton>
          </Tooltip>
        </Stack>
      </Box>
    </Box>
  );
};

export default HierarchicalNavigation;