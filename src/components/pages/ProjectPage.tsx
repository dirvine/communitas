import React from 'react'
import { useParams } from 'react-router-dom'
import { Box, Typography } from '@mui/material'
import { EntityChatView } from '../chat/EntityChatView'

export const ProjectPage: React.FC = () => {
  const { orgId, projectId } = useParams()

  if (!projectId) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>Project not found</Typography>
      </Box>
    )
  }

  return (
    <Box sx={{ height: '100vh' }}>
      <EntityChatView
        entityId={projectId}
        entityType="project"
        entityName={`Project ${projectId}`}
        currentUserId="current-user-id"
        currentUserFourWords="your-current-four-words"
      />
    </Box>
  )
}

export default ProjectPage