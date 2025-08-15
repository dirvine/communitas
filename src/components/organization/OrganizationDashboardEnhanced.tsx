import React, { useState, useEffect, useCallback } from 'react'
import {
  Box,
  Grid,
  Card,
  CardContent,
  CardActions,
  Typography,
  Button,
  Chip,
  Avatar,
  AvatarGroup,
  IconButton,
  Stack,
  Badge,
  Tooltip,
  Paper,
  TextField,
  InputAdornment,
  Tabs,
  Tab,
  Divider,
} from '@mui/material'
import {
  Business as BusinessIcon,
  Group as GroupIcon,
  Work as WorkIcon,
  Person as PersonIcon,
  PersonAdd as PersonAddIcon,
  Settings as SettingsIcon,
  Add as AddIcon,
  MoreVert as MoreVertIcon,
  VideoCall,
  Call,
  AttachFile,
  Message,
  Search,
  FolderOpen,
} from '@mui/icons-material'
import { motion } from 'framer-motion'

import { OrganizationHierarchy } from '../../types/organization'
import { organizationService } from '../../services/organization/OrganizationService'
import * as stores from '../../services/stores'
import CreateOrganizationDialog from './CreateOrganizationDialog'
import CreateGroupDialog from './CreateGroupDialog'
import CreateProjectDialog from './CreateProjectDialog'
import InviteMemberDialog from './InviteMemberDialog'
import { useDHTSync, DHTSyncEvent } from '../../hooks/useDHTSync'

interface TabPanelProps {
  children?: React.ReactNode
  index: number
  value: number
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props
  return (
    <div hidden={value !== index} {...other}>
      {value === index && <Box sx={{ p: 3 }}>{children}</Box>}
    </div>
  )
}

interface CommunicationEntity {
  id: string
  name: string
  type: 'project' | 'group' | 'individual'
  description?: string
  memberCount?: number
  members?: { id: string; name: string; avatar?: string }[]
  status?: 'active' | 'inactive'
  lastActivity?: string
  avatar?: string
  role?: string
}

interface OrganizationDashboardEnhancedProps {
  currentUserId?: string
}

const OrganizationDashboardEnhanced: React.FC<OrganizationDashboardEnhancedProps> = ({
  currentUserId = 'user_owner_123'
}) => {
  const [hierarchies, setHierarchies] = useState<OrganizationHierarchy[]>([])
  const [selectedOrgId, setSelectedOrgId] = useState<string>('')
  const [loading, setLoading] = useState(true)
  const [searchTerm, setSearchTerm] = useState('')
  const [currentTab, setCurrentTab] = useState(0)
  
  // Dialog states
  const [createOrgOpen, setCreateOrgOpen] = useState(false)
  const [createGroupOpen, setCreateGroupOpen] = useState(false)
  const [createProjectOpen, setCreateProjectOpen] = useState(false)
  const [inviteMemberOpen, setInviteMemberOpen] = useState(false)
  
  // Handle DHT sync events
  const handleDHTEvent = useCallback((event: DHTSyncEvent) => {
    // Refresh on relevant events
    bootstrap()
  }, [])
  
  const organizationIds = hierarchies.map(h => h.organization.id)
  
  const { 
    connected, 
    syncing, 
    peerCount,
    lastSync 
  } = useDHTSync({
    userId: currentUserId,
    entityIds: organizationIds,
    onEvent: handleDHTEvent,
    autoReconnect: true
  })

  useEffect(() => {
    bootstrap()
  }, [currentUserId])

  const bootstrap = async () => {
    try {
      setLoading(true)
      await stores.initLocalStores()
      const userOrgs = await organizationService.getUserOrganizations(currentUserId)
      
      const hierarchyPromises = userOrgs.map(org => 
        organizationService.getOrganizationHierarchy(org.id)
      )
      
      const hierarchyResults = await Promise.all(hierarchyPromises)
      const validHierarchies = hierarchyResults.filter(h => h !== null) as OrganizationHierarchy[]
      
      setHierarchies(validHierarchies)
      if (validHierarchies.length > 0 && !selectedOrgId) {
        setSelectedOrgId(validHierarchies[0].organization.id)
      }
    } catch (error) {
      console.error('Error loading organizations:', error)
    } finally {
      setLoading(false)
    }
  }

  const selectedHierarchy = hierarchies.find(h => h.organization.id === selectedOrgId)

  // Transform data for unified display
  const getProjects = (): CommunicationEntity[] => {
    if (!selectedHierarchy) return []
    return selectedHierarchy.projects.map(p => ({
      id: p.id,
      name: p.name,
      type: 'project' as const,
      description: p.description,
      memberCount: p.members?.length || 0,
      members: p.members?.map(m => ({ id: m.user_id, name: m.display_name || m.user_id })),
      status: 'active' as const,
      lastActivity: '2 hours ago',
    }))
  }

  const getGroups = (): CommunicationEntity[] => {
    if (!selectedHierarchy) return []
    return selectedHierarchy.groups.map(g => ({
      id: g.id,
      name: g.name,
      type: 'group' as const,
      description: g.description,
      memberCount: g.members?.length || 0,
      members: g.members?.map(m => ({ id: m.user_id, name: m.display_name || m.user_id })),
      status: 'active' as const,
      lastActivity: '1 hour ago',
    }))
  }

  const getIndividuals = (): CommunicationEntity[] => {
    if (!selectedHierarchy) return []
    const allMembers = new Map<string, any>()
    
    // Collect all unique members from org, groups, and projects
    selectedHierarchy.organization.members?.forEach(m => {
      allMembers.set(m.user_id, { id: m.user_id, name: m.display_name || m.user_id, role: m.role })
    })
    selectedHierarchy.groups.forEach(g => {
      g.members?.forEach(m => allMembers.set(m.user_id, { id: m.user_id, name: m.display_name || m.user_id, role: 'member' }))
    })
    selectedHierarchy.projects.forEach(p => {
      p.members?.forEach(m => allMembers.set(m.user_id, { id: m.user_id, name: m.display_name || m.user_id, role: 'member' }))
    })
    
    return Array.from(allMembers.values()).map(m => ({
      id: m.id,
      name: m.name || m.id,
      type: 'individual' as const,
      role: m.role,
      status: 'active' as const,
      lastActivity: '5 minutes ago',
    }))
  }

  const handleVideoCall = (entity: CommunicationEntity) => {
    console.log('Starting video call with', entity.name)
  }

  const handleVoiceCall = (entity: CommunicationEntity) => {
    console.log('Starting voice call with', entity.name)
  }

  const handleFileShare = (entity: CommunicationEntity) => {
    console.log('Sharing files with', entity.name)
  }

  const handleMessage = (entity: CommunicationEntity) => {
    console.log('Opening chat with', entity.name)
  }

  const renderEntityCard = (entity: CommunicationEntity, index: number) => (
    <Grid item xs={12} sm={6} md={4} key={entity.id}>
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: index * 0.1 }}
      >
        <Card 
          sx={{ 
            height: '100%',
            display: 'flex',
            flexDirection: 'column',
            '&:hover': {
              boxShadow: 6,
              transform: 'translateY(-4px)',
            },
            transition: 'all 0.3s ease',
          }}
        >
          <CardContent sx={{ flexGrow: 1 }}>
            <Stack direction="row" alignItems="flex-start" justifyContent="space-between" mb={2}>
              <Avatar sx={{ width: 48, height: 48, bgcolor: 
                entity.type === 'project' ? 'primary.main' : 
                entity.type === 'group' ? 'secondary.main' : 
                'success.main' 
              }}>
                {entity.type === 'project' ? <WorkIcon /> :
                 entity.type === 'group' ? <GroupIcon /> :
                 <PersonIcon />}
              </Avatar>
              <Stack direction="row" spacing={0.5}>
                <Chip
                  label={entity.type}
                  size="small"
                  color={
                    entity.type === 'project' ? 'primary' :
                    entity.type === 'group' ? 'secondary' :
                    'success'
                  }
                  variant="outlined"
                />
                <IconButton size="small">
                  <MoreVertIcon />
                </IconButton>
              </Stack>
            </Stack>

            <Typography variant="h6" fontWeight={600} gutterBottom>
              {entity.name}
            </Typography>
            
            {entity.description && (
              <Typography 
                variant="body2" 
                color="text.secondary" 
                gutterBottom
                sx={{
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  display: '-webkit-box',
                  WebkitLineClamp: 2,
                  WebkitBoxOrient: 'vertical',
                }}
              >
                {entity.description}
              </Typography>
            )}

            {entity.type !== 'individual' && entity.members && (
              <Stack direction="row" alignItems="center" spacing={1} mt={2}>
                <AvatarGroup max={3} sx={{ '& .MuiAvatar-root': { width: 24, height: 24, fontSize: 12 } }}>
                  {entity.members.slice(0, 3).map(member => (
                    <Avatar key={member.id} sx={{ bgcolor: 'primary.main' }}>
                      {member.name.charAt(0)}
                    </Avatar>
                  ))}
                </AvatarGroup>
                <Typography variant="caption" color="text.secondary">
                  {entity.memberCount} members
                </Typography>
              </Stack>
            )}

            {entity.type === 'individual' && entity.role && (
              <Chip 
                label={entity.role} 
                size="small" 
                color="default" 
                sx={{ mt: 1 }}
              />
            )}

            <Typography variant="caption" color="text.secondary" display="block" mt={1}>
              Active {entity.lastActivity}
            </Typography>
          </CardContent>

          <CardActions sx={{ justifyContent: 'space-around', borderTop: 1, borderColor: 'divider' }}>
            <Tooltip title="Message">
              <IconButton 
                color="primary"
                onClick={() => handleMessage(entity)}
              >
                <Message />
              </IconButton>
            </Tooltip>
            
            <Tooltip title="Voice Call">
              <IconButton 
                color="primary"
                onClick={() => handleVoiceCall(entity)}
              >
                <Call />
              </IconButton>
            </Tooltip>
            
            <Tooltip title="Video Call">
              <IconButton 
                color="primary"
                onClick={() => handleVideoCall(entity)}
              >
                <VideoCall />
              </IconButton>
            </Tooltip>
            
            <Tooltip title="Share Files">
              <IconButton 
                color="primary"
                onClick={() => handleFileShare(entity)}
              >
                <AttachFile />
              </IconButton>
            </Tooltip>
          </CardActions>
        </Card>
      </motion.div>
    </Grid>
  )

  const projects = getProjects()
  const groups = getGroups()
  const individuals = getIndividuals()

  const filteredProjects = projects.filter(p => 
    p.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    p.description?.toLowerCase().includes(searchTerm.toLowerCase())
  )
  const filteredGroups = groups.filter(g => 
    g.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    g.description?.toLowerCase().includes(searchTerm.toLowerCase())
  )
  const filteredIndividuals = individuals.filter(i => 
    i.name.toLowerCase().includes(searchTerm.toLowerCase())
  )

  if (loading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh' }}>
        <Typography>Loading organizations...</Typography>
      </Box>
    )
  }

  if (hierarchies.length === 0) {
    return (
      <Box sx={{ p: 3 }}>
        <Paper sx={{ p: 4, textAlign: 'center' }}>
          <BusinessIcon sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
          <Typography variant="h5" gutterBottom>No Organizations Yet</Typography>
          <Typography variant="body2" color="text.secondary" mb={3}>
            Create your first organization to start collaborating
          </Typography>
          <Button 
            variant="contained" 
            startIcon={<AddIcon />}
            onClick={() => setCreateOrgOpen(true)}
          >
            Create Organization
          </Button>
        </Paper>
        
        <CreateOrganizationDialog
          open={createOrgOpen}
          onClose={() => setCreateOrgOpen(false)}
          onSubmit={async (data) => {
            await organizationService.createOrganization(data, currentUserId)
            await bootstrap()
            setCreateOrgOpen(false)
          }}
        />
      </Box>
    )
  }

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Organization Selector and Header */}
      <Paper sx={{ p: 3, mb: 2 }}>
        <Stack direction="row" alignItems="center" justifyContent="space-between" mb={2}>
          <Stack direction="row" spacing={2} alignItems="center">
            <Avatar sx={{ width: 48, height: 48, bgcolor: 'primary.main' }}>
              <BusinessIcon />
            </Avatar>
            <Box>
              <Typography variant="h5" fontWeight={600}>
                {selectedHierarchy?.organization.name}
              </Typography>
              <Typography variant="body2" color="text.secondary">
                {selectedHierarchy?.organization.description}
              </Typography>
            </Box>
          </Stack>
          
          <Stack direction="row" spacing={1}>
            <Button
              variant="outlined"
              startIcon={<PersonAddIcon />}
              onClick={() => setInviteMemberOpen(true)}
            >
              Invite Members
            </Button>
            <Button
              variant="contained"
              startIcon={<AddIcon />}
              onClick={() => setCreateOrgOpen(true)}
            >
              New Organization
            </Button>
            <IconButton>
              <SettingsIcon />
            </IconButton>
          </Stack>
        </Stack>

        {/* Organization Tabs */}
        {hierarchies.length > 1 && (
          <Stack direction="row" spacing={1} sx={{ overflowX: 'auto' }}>
            {hierarchies.map(h => (
              <Chip
                key={h.organization.id}
                label={h.organization.name}
                onClick={() => setSelectedOrgId(h.organization.id)}
                color={h.organization.id === selectedOrgId ? 'primary' : 'default'}
                variant={h.organization.id === selectedOrgId ? 'filled' : 'outlined'}
              />
            ))}
          </Stack>
        )}
      </Paper>

      {/* Search Bar */}
      <Paper sx={{ p: 2, mb: 2 }}>
        <TextField
          fullWidth
          placeholder="Search projects, groups, and individuals..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          InputProps={{
            startAdornment: (
              <InputAdornment position="start">
                <Search />
              </InputAdornment>
            ),
          }}
        />
      </Paper>

      {/* Content Tabs */}
      <Paper sx={{ flex: 1, overflow: 'auto' }}>
        <Tabs 
          value={currentTab} 
          onChange={(_, newValue) => setCurrentTab(newValue)}
          sx={{ borderBottom: 1, borderColor: 'divider' }}
        >
          <Tab label={`Projects (${projects.length})`} icon={<WorkIcon />} iconPosition="start" />
          <Tab label={`Groups (${groups.length})`} icon={<GroupIcon />} iconPosition="start" />
          <Tab label={`Individuals (${individuals.length})`} icon={<PersonIcon />} iconPosition="start" />
        </Tabs>

        <TabPanel value={currentTab} index={0}>
          <Stack direction="row" justifyContent="space-between" alignItems="center" mb={2}>
            <Typography variant="h6">Projects</Typography>
            <Button
              variant="outlined"
              startIcon={<AddIcon />}
              onClick={() => setCreateProjectOpen(true)}
            >
              New Project
            </Button>
          </Stack>
          <Grid container spacing={3}>
            {filteredProjects.map((project, index) => renderEntityCard(project, index))}
          </Grid>
          {filteredProjects.length === 0 && (
            <Box sx={{ textAlign: 'center', mt: 4 }}>
              <FolderOpen sx={{ fontSize: 48, color: 'text.secondary' }} />
              <Typography variant="h6" color="text.secondary" mt={2}>
                No projects yet
              </Typography>
              <Typography variant="body2" color="text.secondary">
                Create your first project to start collaborating
              </Typography>
            </Box>
          )}
        </TabPanel>

        <TabPanel value={currentTab} index={1}>
          <Stack direction="row" justifyContent="space-between" alignItems="center" mb={2}>
            <Typography variant="h6">Groups</Typography>
            <Button
              variant="outlined"
              startIcon={<AddIcon />}
              onClick={() => setCreateGroupOpen(true)}
            >
              New Group
            </Button>
          </Stack>
          <Grid container spacing={3}>
            {filteredGroups.map((group, index) => renderEntityCard(group, index))}
          </Grid>
          {filteredGroups.length === 0 && (
            <Box sx={{ textAlign: 'center', mt: 4 }}>
              <GroupIcon sx={{ fontSize: 48, color: 'text.secondary' }} />
              <Typography variant="h6" color="text.secondary" mt={2}>
                No groups yet
              </Typography>
              <Typography variant="body2" color="text.secondary">
                Create a group to collaborate with team members
              </Typography>
            </Box>
          )}
        </TabPanel>

        <TabPanel value={currentTab} index={2}>
          <Stack direction="row" justifyContent="space-between" alignItems="center" mb={2}>
            <Typography variant="h6">Individuals</Typography>
            <Button
              variant="outlined"
              startIcon={<PersonAddIcon />}
              onClick={() => setInviteMemberOpen(true)}
            >
              Invite Member
            </Button>
          </Stack>
          <Grid container spacing={3}>
            {filteredIndividuals.map((individual, index) => renderEntityCard(individual, index))}
          </Grid>
          {filteredIndividuals.length === 0 && (
            <Box sx={{ textAlign: 'center', mt: 4 }}>
              <PersonIcon sx={{ fontSize: 48, color: 'text.secondary' }} />
              <Typography variant="h6" color="text.secondary" mt={2}>
                No members yet
              </Typography>
              <Typography variant="body2" color="text.secondary">
                Invite people to join your organization
              </Typography>
            </Box>
          )}
        </TabPanel>
      </Paper>

      {/* Dialogs */}
      <CreateOrganizationDialog
        open={createOrgOpen}
        onClose={() => setCreateOrgOpen(false)}
        onSubmit={async (data) => {
          await organizationService.createOrganization(data, currentUserId)
          await bootstrap()
          setCreateOrgOpen(false)
        }}
      />

      <CreateGroupDialog
        open={createGroupOpen}
        onClose={() => setCreateGroupOpen(false)}
        organizationId={selectedOrgId}
        onSubmit={async (data) => {
          await organizationService.createGroup({
            ...data,
            organization_id: selectedOrgId
          }, currentUserId)
          await bootstrap()
          setCreateGroupOpen(false)
        }}
      />

      <CreateProjectDialog
        open={createProjectOpen}
        onClose={() => setCreateProjectOpen(false)}
        organizationId={selectedOrgId}
        onSubmit={async (data) => {
          await organizationService.createProject({
            ...data,
            organization_id: selectedOrgId
          }, currentUserId)
          await bootstrap()
          setCreateProjectOpen(false)
        }}
      />

      <InviteMemberDialog
        open={inviteMemberOpen}
        onClose={() => setInviteMemberOpen(false)}
        targetType="organization"
        targetId={selectedOrgId}
        onSubmit={async (data) => {
          console.log('Inviting member:', data)
          setInviteMemberOpen(false)
        }}
      />
    </Box>
  )
}

export default OrganizationDashboardEnhanced