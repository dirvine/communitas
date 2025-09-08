import React from 'react'
import { useParams } from 'react-router-dom'
import { Box, Typography, Stack, Button, Card, CardContent, Avatar } from '@mui/material'
import { Group as GroupIcon, Folder as FolderIcon, Call as CallIcon, VideoCall as VideoCallIcon } from '@mui/icons-material'

export const GroupPage: React.FC = () => {
  const { groupId } = useParams()

  const openFiles = () => {
    window.dispatchEvent(new CustomEvent('app:action', { detail: { action: 'storage', entityType: 'group', entityId: groupId } }))
  }

  const startAudio = () => {
    window.dispatchEvent(new CustomEvent('app:action', { detail: { action: 'call', entityType: 'group', entityId: groupId } }))
  }

  const startVideo = () => {
    window.dispatchEvent(new CustomEvent('app:action', { detail: { action: 'video', entityType: 'group', entityId: groupId } }))
  }

  return (
    <Box sx={{ p: 3, display: 'flex', flexDirection: 'column', gap: 2 }}>
      <Stack direction="row" alignItems="center" spacing={2}>
        <Avatar sx={{ bgcolor: 'primary.main' }}>
          <GroupIcon />
        </Avatar>
        <Box>
          <Typography variant="h5" fontWeight={700}>Group</Typography>
          <Typography variant="body2" color="text.secondary">ID: {groupId}</Typography>
        </Box>
      </Stack>

      <Stack direction="row" spacing={1}>
        <Button variant="contained" startIcon={<CallIcon />} onClick={startAudio}>Voice Call</Button>
        <Button variant="outlined" startIcon={<VideoCallIcon />} onClick={startVideo}>Video Call</Button>
        <Button variant="outlined" startIcon={<FolderIcon />} onClick={openFiles}>Open Files</Button>
      </Stack>

      <Card variant="outlined">
        <CardContent>
          <Typography variant="subtitle1" gutterBottom>Recent Activity</Typography>
          <Typography variant="body2" color="text.secondary">No recent messages. Start a call or open files to collaborate.</Typography>
        </CardContent>
      </Card>
    </Box>
  )
}

export default GroupPage
