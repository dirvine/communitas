import React from 'react'
import { useParams } from 'react-router-dom'
import { Box, Typography } from '@mui/material'
import { EntityChatView } from '../chat/EntityChatView'

export const GroupPage: React.FC = () => {
  const { groupId } = useParams()

  if (!groupId) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>Group not found</Typography>
      </Box>
    )
  }

  return (
    <Box sx={{ height: '100vh' }}>
      <EntityChatView
        entityId={groupId}
        entityType="group"
        entityName={`Group ${groupId}`}
        currentUserId="current-user-id"
        currentUserFourWords="your-current-four-words"
      />
    </Box>
  )
}

export default GroupPage
