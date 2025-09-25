import React from 'react'
import { useParams } from 'react-router-dom'
import { Box, Typography } from '@mui/material'
import { EntityChatView } from '../chat/EntityChatView'
import { useAuth } from '../auth'
import {
  mockPersonalUsers,
  mockOrganizations,
} from '../../data/mockCollaborationData'

export const UserPage: React.FC = () => {
  const { userId, orgId } = useParams()
  const { authState } = useAuth()

  if (!userId) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>User not found</Typography>
      </Box>
    )
  }

  // Find the user in either personal contacts or organization members
  let user = mockPersonalUsers.find(u => u.id === userId)
  
  if (!user && orgId) {
    // Look in organization's channels and groups for users
    const organization = mockOrganizations.find(org => org.id === orgId)
    if (organization) {
      // Check all channels for this user
      for (const channel of organization.channels || []) {
        if (channel.members?.includes(userId)) {
          // Create a user object based on channel membership
          user = {
            id: userId,
            type: 'personal_user',
            name: `User ${userId}`,
            userId: userId,
            relationship: 'colleague',
            lastContact: new Date(),
            networkIdentity: {
              fourWords: `user-${userId}-network-identity`,
              publicKey: `pk_${userId}`,
              dhtAddress: `dht://${userId}`
            },
            capabilities: {
              videoCall: true,
              audioCall: true,
              screenShare: true,
              fileShare: true,
              websitePublish: true
            },
            createdAt: new Date(),
            updatedAt: new Date()
          }
          break
        }
      }
      
      // Check all groups for this user if not found in channels
      if (!user) {
        for (const group of organization.groups || []) {
          if (group.members?.includes(userId)) {
            user = {
              id: userId,
              type: 'personal_user',
              name: `User ${userId}`,
              userId: userId,
              relationship: 'colleague',
              lastContact: new Date(),
              networkIdentity: {
                fourWords: `user-${userId}-network-identity`,
                publicKey: `pk_${userId}`,
                dhtAddress: `dht://${userId}`
              },
              capabilities: {
                videoCall: true,
                audioCall: true,
                screenShare: true,
                fileShare: true,
                websitePublish: true
              },
              createdAt: new Date(),
              updatedAt: new Date()
            }
            break
          }
        }
      }
    }
  }

  if (!user) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>User "{userId}" not found</Typography>
      </Box>
    )
  }

  return (
    <Box sx={{ height: '100vh' }}>
      <EntityChatView
        entityId={userId}
        entityType="user"
        entityName={user.name}
        currentUserId={authState.user?.id || "user-owner-123"}
        currentUserFourWords={authState.user?.fourWordAddress || "ocean-forest-moon-star"}
        fourWordAddress={user.networkIdentity.fourWords}
      />
    </Box>
  )
}

export default UserPage
