import React from 'react'
import { Box } from '@mui/material'
import MobileChatLayout from '../chat/MobileChatLayout'

/**
 * Enhanced Messages Tab with P2P Integration
 * 
 * This component integrates the comprehensive group chat system with:
 * - Real-time P2P messaging via Tauri commands
 * - Multi-group chat management
 * - User presence indicators
 * - Mobile-responsive design
 * - Message encryption and synchronization
 */
const MessagesTab: React.FC = () => {
  return (
    <Box sx={{ height: '100%', width: '100%' }}>
      <MobileChatLayout initialGroupId="general" standalone={false} />
    </Box>
  )
}

export default MessagesTab
