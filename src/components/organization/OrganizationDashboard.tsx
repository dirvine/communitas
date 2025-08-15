import React, { useState, useEffect, useCallback } from 'react'
import {
  Box,
  Grid,
  Card,
  CardContent,
  Typography,
  Button,
  Chip,
  LinearProgress,
  Avatar,
  List,
  ListItem,
  ListItemAvatar,
  ListItemText,
  Divider,
  IconButton,
  Badge
} from '@mui/material'
import {
  Business as BusinessIcon,
  Group as GroupIcon,
  Work as WorkIcon,
  PersonAdd as PersonAddIcon,
  Settings as SettingsIcon,
  Add as AddIcon,
  MoreVert as MoreVertIcon,
  Sync as SyncIcon,
  SyncDisabled as SyncDisabledIcon
} from '@mui/icons-material'

import { OrganizationHierarchy } from '../../types/organization'
import { organizationService } from '../../services/organization/OrganizationService'
import * as stores from '../../services/stores'
import CreateOrganizationDialog from './CreateOrganizationDialog'
import CreateGroupDialog from './CreateGroupDialog'
import CreateProjectDialog from './CreateProjectDialog'
import InviteMemberDialog from './InviteMemberDialog'
import { useDHTSync, DHTSyncEvent } from '../../hooks/useDHTSync'

interface OrganizationDashboardProps {
  currentUserId?: string
}

const OrganizationDashboard: React.FC<OrganizationDashboardProps> = ({
  currentUserId = 'user_owner_123'
}) => {
  const [hierarchies, setHierarchies] = useState<OrganizationHierarchy[]>([])
  const [selectedOrgId, setSelectedOrgId] = useState<string>('')
  const [loading, setLoading] = useState(true)
  
  // Dialog states
  const [createOrgOpen, setCreateOrgOpen] = useState(false)
  const [createGroupOpen, setCreateGroupOpen] = useState(false)
  const [createProjectOpen, setCreateProjectOpen] = useState(false)
  const [inviteMemberOpen, setInviteMemberOpen] = useState(false)
  const [inviteTarget, setInviteTarget] = useState<{ type: 'organization' | 'group' | 'project', id: string } | null>(null)
  
  // Handle DHT sync events
  const handleDHTEvent = useCallback((event: DHTSyncEvent) => {
    switch (event.type) {
      case 'OrganizationCreated':
      case 'OrganizationUpdated':
      case 'OrganizationDeleted':
      case 'GroupCreated':
      case 'GroupUpdated':
      case 'GroupDeleted':
      case 'ProjectCreated':
      case 'ProjectUpdated':
      case 'ProjectDeleted':
      case 'MemberJoined':
      case 'MemberLeft':
      case 'MemberRoleChanged':
        // Refresh the organization data when these events occur
        bootstrap()
        break
    }
  }, [])
  
  // Get all organization IDs for subscription
  const organizationIds = hierarchies.map(h => h.organization.id)
  
  // Initialize DHT sync for real-time updates
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

  const handleCreateOrganization = async (data: any) => {
    try {
      await organizationService.createOrganization(data, currentUserId)
      await bootstrap()
      setCreateOrgOpen(false)
    } catch (error) {
      console.error('Error creating organization:', error)
    }
  }

  const handleCreateGroup = async (data: any) => {
    try {
      await organizationService.createGroup({
        ...data,
        organization_id: selectedOrgId
      }, currentUserId)
      await bootstrap()
      setCreateGroupOpen(false)
    } catch (error) {
      console.error('Error creating group:', error)
    }
  }

  const handleCreateProject = async (data: any) => {
    try {
      await organizationService.createProject({
        ...data,
        organization_id: selectedOrgId
      }, currentUserId)
      await bootstrap()
      setCreateProjectOpen(false)
    } catch (error) {
      console.error('Error creating project:', error)
    }
  }

  const handleInviteMember = async (data: any) => {
    try {
      if (inviteTarget) {
        await organizationService.inviteMember({
          entity_type: inviteTarget.type,
          entity_id: inviteTarget.id,
          invitee_address: data.address,
          role: data.role,
          message: data.message
        }, currentUserId)
        await bootstrap()
      }
      setInviteMemberOpen(false)
      setInviteTarget(null)
    } catch (error) {
      console.error('Error inviting member:', error)
    }
  }

  const getRoleColor = (role: string) => {
    switch (role) {
      case 'Owner': return 'error'
      case 'Admin': return 'warning'
      case 'Member': return 'primary'
      case 'Viewer': return 'info'
      case 'Guest': return 'default'
      default: return 'default'
    }
  }

  const formatStorageUsage = (used: number, total: number) => {
    const percentage = (used / total) * 100
    return { used: used.toFixed(1), total: total.toFixed(1), percentage }
  }

  if (loading) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>Loading organizations...</Typography>
        <LinearProgress sx={{ mt: 2 }} />
      </Box>
    )
  }

  if (hierarchies.length === 0) {
    return (
      <Box sx={{ p: 3, textAlign: 'center' }}>
        <BusinessIcon sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
        <Typography variant="h5" gutterBottom>No Organizations</Typography>
        <Typography color="text.secondary" paragraph>
          Create your first organization to start collaborating with your team.
        </Typography>
        <Button
          variant="contained"
          startIcon={<AddIcon />}
          onClick={() => setCreateOrgOpen(true)}
          size="large"
        >
          Create Organization
        </Button>
      </Box>
    )
  }

  return (
    <Box sx={{ p: 3 }}>
      {/* Header */}
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
          <Typography variant="h4" component="h1">
            Organization Dashboard
          </Typography>
          {/* Sync Status Indicator */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            {connected ? (
              <Badge badgeContent={peerCount} color="success">
                <SyncIcon 
                  color={syncing ? "primary" : "action"} 
                  sx={{ 
                    animation: syncing ? 'spin 2s linear infinite' : 'none',
                    '@keyframes spin': {
                      '0%': { transform: 'rotate(0deg)' },
                      '100%': { transform: 'rotate(360deg)' }
                    }
                  }} 
                />
              </Badge>
            ) : (
              <SyncDisabledIcon color="disabled" />
            )}
            <Typography variant="caption" color="text.secondary">
              {connected ? `Connected (${peerCount} peers)` : 'Offline'}
            </Typography>
          </Box>
        </Box>
        <Button
          variant="contained"
          startIcon={<AddIcon />}
          onClick={() => setCreateOrgOpen(true)}
        >
          Create Organization
        </Button>
      </Box>

      {/* Organization Selector */}
      <Grid container spacing={2} sx={{ mb: 3 }}>
        {hierarchies.map((hierarchy) => (
          <Grid item xs={12} sm={6} md={4} key={hierarchy.organization.id}>
            <Card 
              sx={{ 
                cursor: 'pointer',
                border: selectedOrgId === hierarchy.organization.id ? 2 : 1,
                borderColor: selectedOrgId === hierarchy.organization.id ? 'primary.main' : 'divider'
              }}
              onClick={() => setSelectedOrgId(hierarchy.organization.id)}
            >
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                  <BusinessIcon sx={{ mr: 1, color: 'primary.main' }} />
                  <Typography variant="h6" noWrap>
                    {hierarchy.organization.name}
                  </Typography>
                </Box>
                <Typography color="text.secondary" variant="body2" sx={{ mb: 2 }}>
                  {hierarchy.organization.description || 'No description'}
                </Typography>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <Typography variant="caption">
                    {hierarchy.total_members} members
                  </Typography>
                  <Typography variant="caption">
                    {formatStorageUsage(
                      hierarchy.total_storage_used_gb, 
                      hierarchy.organization.storage_quota.allocated_gb
                    ).percentage.toFixed(0)}% storage
                  </Typography>
                </Box>
              </CardContent>
            </Card>
          </Grid>
        ))}
      </Grid>

      {selectedHierarchy && (
        <Grid container spacing={3}>
          {/* Left Column - Organization Overview */}
          <Grid item xs={12} lg={4}>
            <Card sx={{ mb: 3 }}>
              <CardContent>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', mb: 2 }}>
                  <Box>
                    <Typography variant="h6">{selectedHierarchy.organization.name}</Typography>
                    <Typography color="text.secondary">
                      {selectedHierarchy.organization.description}
                    </Typography>
                  </Box>
                  <IconButton size="small">
                    <SettingsIcon />
                  </IconButton>
                </Box>
                
                <Divider sx={{ my: 2 }} />
                
                {/* Storage Usage */}
                <Typography variant="subtitle2" gutterBottom>Storage Usage</Typography>
                <Box sx={{ mb: 2 }}>
                  {(() => {
                    const storage = formatStorageUsage(
                      selectedHierarchy.total_storage_used_gb,
                      selectedHierarchy.organization.storage_quota.allocated_gb
                    )
                    return (
                      <>
                        <LinearProgress 
                          variant="determinate" 
                          value={storage.percentage} 
                          sx={{ mb: 1 }}
                        />
                        <Typography variant="caption">
                          {storage.used} GB of {storage.total} GB used
                        </Typography>
                      </>
                    )
                  })()}
                </Box>

                {/* Quick Stats */}
                <Grid container spacing={2}>
                  <Grid item xs={6}>
                    <Box sx={{ textAlign: 'center' }}>
                      <Typography variant="h4" color="primary">
                        {selectedHierarchy.groups.length}
                      </Typography>
                      <Typography variant="caption">Groups</Typography>
                    </Box>
                  </Grid>
                  <Grid item xs={6}>
                    <Box sx={{ textAlign: 'center' }}>
                      <Typography variant="h4" color="primary">
                        {selectedHierarchy.projects.length}
                      </Typography>
                      <Typography variant="caption">Projects</Typography>
                    </Box>
                  </Grid>
                </Grid>
              </CardContent>
            </Card>

            {/* Members List */}
            <Card>
              <CardContent>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                  <Typography variant="h6">Members ({selectedHierarchy.organization.members.length})</Typography>
                  <Button
                    size="small"
                    startIcon={<PersonAddIcon />}
                    onClick={() => {
                      setInviteTarget({ type: 'organization', id: selectedHierarchy.organization.id })
                      setInviteMemberOpen(true)
                    }}
                  >
                    Invite
                  </Button>
                </Box>
                <List dense>
                  {selectedHierarchy.organization.members.map((member) => (
                    <ListItem key={member.user_id} sx={{ px: 0 }}>
                      <ListItemAvatar>
                        <Avatar sx={{ width: 32, height: 32 }}>
                          {member.display_name.charAt(0)}
                        </Avatar>
                      </ListItemAvatar>
                      <ListItemText
                        primary={member.display_name}
                        secondary={member.four_word_address}
                      />
                      <Chip 
                        label={member.role} 
                        size="small" 
                        color={getRoleColor(member.role) as any}
                      />
                    </ListItem>
                  ))}
                </List>
              </CardContent>
            </Card>
          </Grid>

          {/* Right Column - Groups and Projects */}
          <Grid item xs={12} lg={8}>
            {/* Groups Section */}
            <Card sx={{ mb: 3 }}>
              <CardContent>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                  <Typography variant="h6">Groups ({selectedHierarchy.groups.length})</Typography>
                  <Button
                    variant="outlined"
                    size="small"
                    startIcon={<AddIcon />}
                    onClick={() => setCreateGroupOpen(true)}
                  >
                    New Group
                  </Button>
                </Box>
                
                {selectedHierarchy.groups.length === 0 ? (
                  <Typography color="text.secondary" sx={{ textAlign: 'center', py: 4 }}>
                    No groups yet. Create your first group to start team discussions.
                  </Typography>
                ) : (
                  <Grid container spacing={2}>
                    {selectedHierarchy.groups.map((group) => (
                      <Grid item xs={12} sm={6} key={group.id}>
                        <Card variant="outlined" sx={{ height: '100%' }}>
                          <CardContent>
                            <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                              <GroupIcon sx={{ mr: 1, color: 'info.main' }} />
                              <Typography variant="subtitle1" noWrap>
                                {group.name}
                              </Typography>
                              <IconButton size="small" sx={{ ml: 'auto' }}>
                                <MoreVertIcon />
                              </IconButton>
                            </Box>
                            <Typography color="text.secondary" variant="body2" sx={{ mb: 2 }}>
                              {group.description || 'No description'}
                            </Typography>
                            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                              <Typography variant="caption">
                                {group.members.length} members
                              </Typography>
                              <Button
                                size="small"
                                onClick={() => {
                                  setInviteTarget({ type: 'group', id: group.id })
                                  setInviteMemberOpen(true)
                                }}
                              >
                                Invite
                              </Button>
                            </Box>
                          </CardContent>
                        </Card>
                      </Grid>
                    ))}
                  </Grid>
                )}
              </CardContent>
            </Card>

            {/* Projects Section */}
            <Card>
              <CardContent>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                  <Typography variant="h6">Projects ({selectedHierarchy.projects.length})</Typography>
                  <Button
                    variant="outlined"
                    size="small"
                    startIcon={<AddIcon />}
                    onClick={() => setCreateProjectOpen(true)}
                  >
                    New Project
                  </Button>
                </Box>
                
                {selectedHierarchy.projects.length === 0 ? (
                  <Typography color="text.secondary" sx={{ textAlign: 'center', py: 4 }}>
                    No projects yet. Create your first project to start collaborating.
                  </Typography>
                ) : (
                  <Grid container spacing={2}>
                    {selectedHierarchy.projects.map((project) => (
                      <Grid item xs={12} sm={6} key={project.id}>
                        <Card variant="outlined" sx={{ height: '100%' }}>
                          <CardContent>
                            <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                              <WorkIcon sx={{ mr: 1, color: 'success.main' }} />
                              <Typography variant="subtitle1" noWrap>
                                {project.name}
                              </Typography>
                              <Chip 
                                label={project.priority} 
                                size="small" 
                                color={
                                  project.priority === 'critical' ? 'error' :
                                  project.priority === 'high' ? 'warning' :
                                  project.priority === 'medium' ? 'info' : 'default'
                                }
                                sx={{ ml: 1 }}
                              />
                              <IconButton size="small" sx={{ ml: 'auto' }}>
                                <MoreVertIcon />
                              </IconButton>
                            </Box>
                            <Typography color="text.secondary" variant="body2" sx={{ mb: 2 }}>
                              {project.description || 'No description'}
                            </Typography>
                            
                            {/* Project Storage */}
                            <Box sx={{ mb: 2 }}>
                              {(() => {
                                const storage = formatStorageUsage(
                                  project.storage_quota.used_gb,
                                  project.storage_quota.allocated_gb
                                )
                                return (
                                  <>
                                    <LinearProgress 
                                      variant="determinate" 
                                      value={storage.percentage}
                                      sx={{ mb: 0.5 }}
                                    />
                                    <Typography variant="caption">
                                      {storage.used} GB of {storage.total} GB
                                    </Typography>
                                  </>
                                )
                              })()}
                            </Box>

                            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                              <Typography variant="caption">
                                {project.members.length} members
                              </Typography>
                              <Button
                                size="small"
                                onClick={() => {
                                  setInviteTarget({ type: 'project', id: project.id })
                                  setInviteMemberOpen(true)
                                }}
                              >
                                Invite
                              </Button>
                            </Box>
                          </CardContent>
                        </Card>
                      </Grid>
                    ))}
                  </Grid>
                )}
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      )}

      {/* Dialogs */}
      <CreateOrganizationDialog
        open={createOrgOpen}
        onClose={() => setCreateOrgOpen(false)}
        onSubmit={handleCreateOrganization}
      />

      <CreateGroupDialog
        open={createGroupOpen}
        onClose={() => setCreateGroupOpen(false)}
        onSubmit={handleCreateGroup}
      />

      <CreateProjectDialog
        open={createProjectOpen}
        onClose={() => setCreateProjectOpen(false)}
        onSubmit={handleCreateProject}
      />

      <InviteMemberDialog
        open={inviteMemberOpen}
        onClose={() => {
          setInviteMemberOpen(false)
          setInviteTarget(null)
        }}
        onSubmit={handleInviteMember}
        entityType={inviteTarget?.type}
      />
    </Box>
  )
}

export default OrganizationDashboard
