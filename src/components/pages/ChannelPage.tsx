import React from 'react'
import { useParams } from 'react-router-dom'
import { Box, Typography } from '@mui/material'
import { EntityChatView } from '../chat/EntityChatView'
import { useAuth } from '../auth'
import {
  mockOrganizations,
} from '../../data/mockCollaborationData'

export const ChannelPage: React.FC = () => {
  const { orgId, channelId } = useParams()
  const { authState } = useAuth()

  if (!channelId) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>Channel not found</Typography>
      </Box>
    )
  }

  // Find the channel in the organization
  const organization = orgId ? mockOrganizations.find(org => org.id === orgId) : null
  const channel = organization?.channels?.find(c => c.id === channelId)

  if (!channel) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>Channel "#{channelId}" not found</Typography>
      </Box>
    )
  }

  return (
    <Box sx={{ height: '100vh' }}>
      <EntityChatView
        entityId={channelId}
        entityType="channel"
        entityName={channel.name}
        currentUserId={authState.user?.id || "user-owner-123"}
        currentUserFourWords={authState.user?.fourWordAddress || "ocean-forest-moon-star"}
        fourWordAddress={channel.networkIdentity.fourWords}
      />
    </Box>
  )
}

export default ChannelPage