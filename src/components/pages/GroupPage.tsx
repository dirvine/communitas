import React from 'react'
import { useParams } from 'react-router-dom'
import { Box, Typography } from '@mui/material'
import { EntityChatView } from '../chat/EntityChatView'
import { useAuth } from '../auth'
import {
  mockPersonalGroups,
  mockOrganizations,
} from '../../data/mockCollaborationData'

export const GroupPage: React.FC = () => {
  const { groupId, orgId } = useParams()
  const { authState } = useAuth()

  if (!groupId) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>Group not found</Typography>
      </Box>
    )
  }

  // Find the group in either personal groups or organization groups
  let group = mockPersonalGroups.find(g => g.id === groupId)
  
  if (!group && orgId) {
    // Look in organization groups
    const organization = mockOrganizations.find(org => org.id === orgId)
    group = organization?.groups?.find(g => g.id === groupId)
  }

  if (!group) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>Group "{groupId}" not found</Typography>
      </Box>
    )
  }

  return (
    <Box sx={{ height: '100vh' }}>
      <EntityChatView
        entityId={groupId}
        entityType="group"
        entityName={group.name}
        currentUserId={authState.user?.id || "user-owner-123"}
        currentUserFourWords={authState.user?.fourWordAddress || "ocean-forest-moon-star"}
        fourWordAddress={group.networkIdentity.fourWords}
      />
    </Box>
  )
}

export default GroupPage
