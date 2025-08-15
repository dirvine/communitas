import React, { useState, useEffect } from 'react'
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
  IconButton,
  Badge,
  Avatar,
  Chip,
  TextField,
  InputAdornment,
  Paper
} from '@mui/material'
import {
  Home as HomeIcon,
  Business as BusinessIcon,
  Group as GroupIcon,
  Work as WorkIcon,
  Person as PersonIcon,
  ExpandLess,
  ExpandMore,
  Add as AddIcon,
  Search as SearchIcon,
  Folder as FolderIcon,
  FolderOpen as FolderOpenIcon,
  Circle as CircleIcon
} from '@mui/icons-material'
import { useNavigation } from '../../contexts/NavigationContext'
import { organizationService } from '../../services/organization/OrganizationService'
import { Organization, Group, Project } from '../../types/organization'
import { motion, AnimatePresence } from 'framer-motion'

interface ContextAwareSidebarProps {
  currentUserId: string
}

const ContextAwareSidebar: React.FC<ContextAwareSidebarProps> = ({ currentUserId }) => {
  const { state, switchToPersonal, switchToOrganization, selectEntity } = useNavigation()
  const [organizations, setOrganizations] = useState<Organization[]>([])
  const [expandedOrgs, setExpandedOrgs] = useState<Set<string>>(new Set())
  const [searchTerm, setSearchTerm] = useState('')
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    loadOrganizations()
  }, [currentUserId])

  const loadOrganizations = async () => {
    setLoading(true)
    try {
      const orgs = await organizationService.getUserOrganizations(currentUserId)
      setOrganizations(orgs)
    } catch (error) {
      console.error('Failed to load organizations:', error)
    } finally {
      setLoading(false)
    }
  }

  const toggleOrgExpansion = (orgId: string) => {
    setExpandedOrgs(prev => {
      const newSet = new Set(prev)
      if (newSet.has(orgId)) {
        newSet.delete(orgId)
      } else {
        newSet.add(orgId)
      }
      return newSet
    })
  }

  const handlePersonalClick = () => {
    switchToPersonal()
  }

  const handleOrganizationClick = (org: Organization) => {
    switchToOrganization(org.id, org.name)
  }

  const handleEntityClick = (type: 'group' | 'project' | 'individual', entity: any, orgId: string, orgName: string) => {
    // First switch to organization context if not already there
    if (state.organizationId !== orgId) {
      switchToOrganization(orgId, orgName)
    }
    // Then select the entity
    selectEntity(type, entity.id, entity.name || entity.display_name)
  }

  const filteredOrganizations = organizations.filter(org =>
    org.name.toLowerCase().includes(searchTerm.toLowerCase())
  )

  return (
    <Paper
      elevation={2}
      sx={{
        width: 280,
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        borderRadius: 0
      }}
    >
      {/* Search Bar */}
      <Box sx={{ p: 2 }}>
        <TextField
          fullWidth
          size="small"
          placeholder="Search organizations..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          InputProps={{
            startAdornment: (
              <InputAdornment position="start">
                <SearchIcon fontSize="small" />
              </InputAdornment>
            )
          }}
        />
      </Box>

      <Divider />

      {/* Navigation List */}
      <List
        sx={{
          flexGrow: 1,
          overflow: 'auto',
          '& .MuiListItemButton-root': {
            borderRadius: 1,
            mx: 1,
            mb: 0.5
          }
        }}
      >
        {/* Personal Space */}
        <ListItem disablePadding>
          <ListItemButton
            selected={state.context === 'personal'}
            onClick={handlePersonalClick}
            sx={{
              bgcolor: state.context === 'personal' ? 'action.selected' : 'transparent'
            }}
          >
            <ListItemIcon>
              <HomeIcon color={state.context === 'personal' ? 'primary' : 'inherit'} />
            </ListItemIcon>
            <ListItemText 
              primary="Personal Space"
              secondary="Your contacts & groups"
            />
          </ListItemButton>
        </ListItem>

        <Divider sx={{ my: 1 }} />

        {/* Organizations Section */}
        <ListItem>
          <Typography variant="overline" color="text.secondary">
            Organizations ({filteredOrganizations.length})
          </Typography>
        </ListItem>

        <AnimatePresence>
          {filteredOrganizations.map(org => (
            <motion.div
              key={org.id}
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -20 }}
              transition={{ duration: 0.2 }}
            >
              {/* Organization Item */}
              <ListItem disablePadding>
                <ListItemButton
                  selected={state.organizationId === org.id}
                  onClick={() => handleOrganizationClick(org)}
                  sx={{
                    bgcolor: state.organizationId === org.id ? 'action.selected' : 'transparent'
                  }}
                >
                  <ListItemIcon>
                    <BusinessIcon color={state.organizationId === org.id ? 'primary' : 'inherit'} />
                  </ListItemIcon>
                  <ListItemText
                    primary={org.name}
                    secondary={
                      <Box sx={{ display: 'flex', gap: 0.5 }}>
                        <Chip label={`${org.members?.length || 0} members`} size="small" />
                      </Box>
                    }
                  />
                  <IconButton
                    size="small"
                    onClick={(e) => {
                      e.stopPropagation()
                      toggleOrgExpansion(org.id)
                    }}
                  >
                    {expandedOrgs.has(org.id) ? <ExpandLess /> : <ExpandMore />}
                  </IconButton>
                </ListItemButton>
              </ListItem>

              {/* Organization Sub-items */}
              <Collapse in={expandedOrgs.has(org.id)} timeout="auto" unmountOnExit>
                <List component="div" disablePadding sx={{ pl: 4 }}>
                  {/* Groups */}
                  <ListItem disablePadding>
                    <ListItemButton
                      sx={{ py: 0.5 }}
                      selected={state.organizationId === org.id && state.entityType === 'group'}
                    >
                      <ListItemIcon>
                        <GroupIcon fontSize="small" />
                      </ListItemIcon>
                      <ListItemText 
                        primary="Groups"
                        secondary={`${org.groups?.length || 0} groups`}
                      />
                      <IconButton size="small">
                        <AddIcon fontSize="small" />
                      </IconButton>
                    </ListItemButton>
                  </ListItem>

                  {/* Projects */}
                  <ListItem disablePadding>
                    <ListItemButton
                      sx={{ py: 0.5 }}
                      selected={state.organizationId === org.id && state.entityType === 'project'}
                    >
                      <ListItemIcon>
                        <WorkIcon fontSize="small" />
                      </ListItemIcon>
                      <ListItemText 
                        primary="Projects"
                        secondary={`${org.projects?.length || 0} projects`}
                      />
                      <IconButton size="small">
                        <AddIcon fontSize="small" />
                      </IconButton>
                    </ListItemButton>
                  </ListItem>

                  {/* Members */}
                  <ListItem disablePadding>
                    <ListItemButton
                      sx={{ py: 0.5 }}
                      selected={state.organizationId === org.id && state.entityType === 'individual'}
                    >
                      <ListItemIcon>
                        <PersonIcon fontSize="small" />
                      </ListItemIcon>
                      <ListItemText 
                        primary="Members"
                        secondary={`${org.members?.length || 0} members`}
                      />
                    </ListItemButton>
                  </ListItem>

                  {/* Specific Groups */}
                  {org.groups?.map((group: Group) => (
                    <ListItem key={group.id} disablePadding sx={{ pl: 2 }}>
                      <ListItemButton
                        sx={{ py: 0.5 }}
                        selected={state.entityId === group.id}
                        onClick={() => handleEntityClick('group', group, org.id, org.name)}
                      >
                        <ListItemIcon>
                          <CircleIcon sx={{ fontSize: 8 }} />
                        </ListItemIcon>
                        <ListItemText 
                          primary={group.name}
                          primaryTypographyProps={{ variant: 'body2' }}
                        />
                        <Badge badgeContent={group.members?.length || 0} color="default" />
                      </ListItemButton>
                    </ListItem>
                  ))}

                  {/* Specific Projects */}
                  {org.projects?.map((project: Project) => (
                    <ListItem key={project.id} disablePadding sx={{ pl: 2 }}>
                      <ListItemButton
                        sx={{ py: 0.5 }}
                        selected={state.entityId === project.id}
                        onClick={() => handleEntityClick('project', project, org.id, org.name)}
                      >
                        <ListItemIcon>
                          <CircleIcon sx={{ fontSize: 8 }} />
                        </ListItemIcon>
                        <ListItemText 
                          primary={project.name}
                          primaryTypographyProps={{ variant: 'body2' }}
                        />
                        <Chip 
                          label={project.priority} 
                          size="small"
                          color={
                            project.priority === 'high' ? 'warning' :
                            project.priority === 'critical' ? 'error' :
                            'default'
                          }
                        />
                      </ListItemButton>
                    </ListItem>
                  ))}
                </List>
              </Collapse>
            </motion.div>
          ))}
        </AnimatePresence>

        {/* Add Organization Button */}
        <ListItem>
          <ListItemButton
            sx={{
              border: 1,
              borderColor: 'divider',
              borderStyle: 'dashed',
              justifyContent: 'center'
            }}
          >
            <ListItemIcon>
              <AddIcon />
            </ListItemIcon>
            <ListItemText primary="Create Organization" />
          </ListItemButton>
        </ListItem>
      </List>

      {/* Footer Status */}
      <Divider />
      <Box sx={{ p: 2, bgcolor: 'background.default' }}>
        <Typography variant="caption" color="text.secondary">
          Context: {state.context === 'personal' ? 'Personal' : state.organizationName}
        </Typography>
      </Box>
    </Paper>
  )
}

export default ContextAwareSidebar