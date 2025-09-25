import React from 'react'
import { useParams } from 'react-router-dom'
import { Box, Typography } from '@mui/material'
import { EntityChatView } from '../chat/EntityChatView'

export const ChannelPage: React.FC = () => {
  const { orgId, channelId } = useParams()

  if (!channelId) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>Channel not found</Typography>
      </Box>
    )
  }

  return (
    <Box sx={{ height: '100vh' }}>
      <EntityChatView
        entityId={channelId}
        entityType="channel"
        entityName={`Channel #${channelId}`}
        currentUserId="current-user-id"
        currentUserFourWords="your-current-four-words"
      />
    </Box>
  )
}

export default ChannelPage