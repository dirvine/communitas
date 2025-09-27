import React from 'react'
import { useParams } from 'react-router-dom'
import { Box, Typography } from '@mui/material'
import { EntityChatView } from '../chat/EntityChatView'
import EntityContentView from '../entity/EntityContentView'
import { useAuth } from '../auth'
import { useEntityDirectory } from '../../contexts/EntityDirectoryContext'

export const GroupPage: React.FC = () => {
  const { groupId, orgId } = useParams()
  const { authState } = useAuth()
  const { personalGroups, organizations } = useEntityDirectory()

  if (!groupId) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>Group not found</Typography>
      </Box>
    )
  }

  // Find the group in either personal groups or organization groups
  let group = personalGroups.find(g => g.id === groupId)
  
  if (!group && orgId) {
    // Look in organization groups
    const organization = organizations.find(org => org.id === orgId)
    group = organization?.groups?.find(g => g.id === groupId)
  }

  if (group && !(group as any).syncStatus) {
    group = {
      ...group,
      syncStatus: 'synced',
      lastSyncedAt: new Date(),
    } as typeof group
  }

  if (!group) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>Group "{groupId}" not found</Typography>
      </Box>
    )
  }

  return (
    <Box sx={{ height: '100vh', display: 'flex', gap: 2, p: 2, boxSizing: 'border-box' }}>
      <Box sx={{ flex: 3, minWidth: 0 }}>
        <EntityChatView
          entityId={groupId}
          entityType="group"
          entityName={group.name}
          currentUserId={authState.user?.id || 'user-owner-123'}
          currentUserFourWords={authState.user?.fourWordAddress || 'ocean-forest-moon-star'}
          fourWordAddress={group.networkIdentity.fourWords}
        />
      </Box>
      <Box sx={{ flex: 2, minWidth: 0 }}>
        <EntityContentView
          entityType="group"
          entityId={group.id}
          entityName={group.name}
          fourWords={group.networkIdentity.fourWords}
        />
      </Box>
    </Box>
  )
}

export default GroupPage
