import React from 'react'
import { useParams } from 'react-router-dom'
import { Box, Typography } from '@mui/material'
import { EntityChatView } from '../chat/EntityChatView'
import EntityContentView from '../entity/EntityContentView'
import { useAuth } from '../auth'
import { useEntityDirectory } from '../../contexts/EntityDirectoryContext'

export const ChannelPage: React.FC = () => {
  const { orgId, channelId } = useParams()
  const { authState } = useAuth()
  const { organizations } = useEntityDirectory()

  if (!channelId) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>Channel not found</Typography>
      </Box>
    )
  }

  // Find the channel in the organization
  const organization = orgId ? organizations.find(org => org.id === orgId) : null
  const channel = organization?.channels?.find(c => c.id === channelId)

  const hydratedChannel = channel && !(channel as any).syncStatus
    ? ({ ...channel, syncStatus: 'synced', lastSyncedAt: new Date() } as typeof channel)
    : channel

  if (!hydratedChannel) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>Channel "#{channelId}" not found</Typography>
      </Box>
    )
  }

  return (
    <Box sx={{ height: '100vh', display: 'flex', gap: 2, p: 2, boxSizing: 'border-box' }}>
      <Box sx={{ flex: 3, minWidth: 0 }}>
        <EntityChatView
          entityId={channelId}
          entityType="channel"
          entityName={hydratedChannel.name}
          currentUserId={authState.user?.id || 'user-owner-123'}
          currentUserFourWords={authState.user?.fourWordAddress || 'ocean-forest-moon-star'}
          fourWordAddress={hydratedChannel.networkIdentity.fourWords}
        />
      </Box>
      <Box sx={{ flex: 2, minWidth: 0 }}>
        <EntityContentView
          entityType="channel"
          entityId={hydratedChannel.id}
          entityName={hydratedChannel.name}
          fourWords={hydratedChannel.networkIdentity.fourWords}
        />
      </Box>
    </Box>
  )
}

export default ChannelPage
