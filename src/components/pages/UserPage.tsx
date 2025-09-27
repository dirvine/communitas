import React from 'react'
import { useParams } from 'react-router-dom'
import { Box, Typography } from '@mui/material'
import { EntityChatView } from '../chat/EntityChatView'
import EntityContentView from '../entity/EntityContentView'
import { useAuth } from '../auth'
import { useEntityDirectory } from '../../contexts/EntityDirectoryContext'

export const UserPage: React.FC = () => {
  const { userId, orgId } = useParams()
  const { authState } = useAuth()
  const { personalUsers, organizations } = useEntityDirectory()

  if (!userId) {
    return (
      <Box sx={{ p: 3 }}>
        <Typography>User not found</Typography>
      </Box>
    )
  }

  // Find the user in either personal contacts or organization members
  let user = personalUsers.find(u => u.id === userId)
  
  if (!user && orgId) {
    // Look in organization's channels and groups for users
    const organization = organizations.find(org => org.id === orgId)
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
            updatedAt: new Date(),
            syncStatus: 'synced',
            lastSyncedAt: new Date()
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
              updatedAt: new Date(),
              syncStatus: 'synced',
              lastSyncedAt: new Date()
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
    <Box sx={{ height: '100vh', display: 'flex', gap: 2, p: 2, boxSizing: 'border-box' }}>
      <Box sx={{ flex: 3, minWidth: 0 }}>
        <EntityChatView
          entityId={userId}
          entityType="user"
          entityName={user.name}
          currentUserId={authState.user?.id || 'user-owner-123'}
          currentUserFourWords={authState.user?.fourWordAddress || 'ocean-forest-moon-star'}
          fourWordAddress={user.networkIdentity.fourWords}
        />
      </Box>
      <Box sx={{ flex: 2, minWidth: 0 }}>
        <EntityContentView
          entityType="individual"
          entityId={user.id}
          entityName={user.name}
          fourWords={user.networkIdentity.fourWords}
        />
      </Box>
    </Box>
  )
}

export default UserPage
