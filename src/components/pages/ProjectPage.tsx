import React from 'react'
import { useParams } from 'react-router-dom'
import { Box, Typography } from '@mui/material'
import { EntityChatView } from '../chat/EntityChatView'
import EntityContentView from '../entity/EntityContentView'
import { useAuth } from '../auth'
import { useEntityDirectory } from '../../contexts/EntityDirectoryContext'

export const ProjectPage: React.FC = () => {
  const { orgId, projectId } = useParams()
  const { authState } = useAuth()
  const { organizations } = useEntityDirectory()

  if (!projectId) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>Project not found</Typography>
      </Box>
    )
  }

  // Find the project in the organization
  const organization = orgId ? organizations.find(org => org.id === orgId) : null
  const project = organization?.projects?.find(p => p.id === projectId)

  const hydratedProject = project && !(project as any).syncStatus
    ? ({ ...project, syncStatus: 'synced', lastSyncedAt: new Date() } as typeof project)
    : project

  if (!hydratedProject) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>Project "{projectId}" not found</Typography>
      </Box>
    )
  }

  return (
    <Box sx={{ height: '100vh', display: 'flex', gap: 2, p: 2, boxSizing: 'border-box' }}>
      <Box sx={{ flex: 3, minWidth: 0 }}>
        <EntityChatView
          entityId={projectId}
          entityType="project"
          entityName={hydratedProject.name}
          currentUserId={authState.user?.id || 'user-owner-123'}
          currentUserFourWords={authState.user?.fourWordAddress || 'ocean-forest-moon-star'}
          fourWordAddress={hydratedProject.networkIdentity.fourWords}
        />
      </Box>
      <Box sx={{ flex: 2, minWidth: 0 }}>
        <EntityContentView
          entityType="project"
          entityId={hydratedProject.id}
          entityName={hydratedProject.name}
          fourWords={hydratedProject.networkIdentity.fourWords}
        />
      </Box>
    </Box>
  )
}

export default ProjectPage
