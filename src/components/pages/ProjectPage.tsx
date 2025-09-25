import React from 'react'
import { useParams } from 'react-router-dom'
import { Box, Typography } from '@mui/material'
import { EntityChatView } from '../chat/EntityChatView'
import { useAuth } from '../auth'
import {
  mockOrganizations,
} from '../../data/mockCollaborationData'

export const ProjectPage: React.FC = () => {
  const { orgId, projectId } = useParams()
  const { authState } = useAuth()

  if (!projectId) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>Project not found</Typography>
      </Box>
    )
  }

  // Find the project in the organization
  const organization = orgId ? mockOrganizations.find(org => org.id === orgId) : null
  const project = organization?.projects?.find(p => p.id === projectId)

  if (!project) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>Project "{projectId}" not found</Typography>
      </Box>
    )
  }

  return (
    <Box sx={{ height: '100vh' }}>
      <EntityChatView
        entityId={projectId}
        entityType="project"
        entityName={project.name}
        currentUserId={authState.user?.id || "user-owner-123"}
        currentUserFourWords={authState.user?.fourWordAddress || "ocean-forest-moon-star"}
        fourWordAddress={project.networkIdentity.fourWords}
      />
    </Box>
  )
}

export default ProjectPage