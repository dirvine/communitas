import React, { useState } from 'react';
import { Box, Fab, Snackbar, Alert } from '@mui/material';
import { Phone, Message } from '@mui/icons-material';
import { webrtcService } from '../../services/webrtc/WebRTCService';
// Removed: SimpleCallInterface - using modern shell instead

export const SimpleCommunicationHub: React.FC = () => {
  const [error, setError] = useState<string | null>(null);

  // Note: Event listener functionality removed - use modern shell call interface instead
  // WebRTC service events are handled via Tauri event listeners in the modern shell

  const handleStartCall = async () => {
    try {
      await webrtcService.initiateCall('demo-contact', { has_audio: true, has_video: false });
    } catch (error) {
      console.error('Failed to start call:', error);
    }
  };


  const handleEndCall = async () => {
    // WebRTC service method needs call ID - this is a placeholder
    console.log('End call functionality needs refactoring');
  };

  const handleSendMessage = () => {
    // Messaging through WebRTC needs refactoring
    console.log('Send message functionality needs refactoring');
  };

  return (
    <>
      {/* Removed: SimpleCallInterface - using modern shell instead */}
      {/* Call UI is handled by modern shell prototype */}

      {(
        <Box
          sx={{
            position: 'fixed',
            bottom: 16,
            right: 16,
            display: 'flex',
            flexDirection: 'column',
            gap: 2,
            zIndex: 1000
          }}
        >
          <Fab
            color="primary"
            onClick={handleSendMessage}
            size="medium"
          >
            <Message />
          </Fab>

          <Fab
            color="secondary"
            onClick={handleStartCall}
            size="large"
          >
            <Phone />
          </Fab>
        </Box>
      )}

      <Snackbar
        open={Boolean(error)}
        autoHideDuration={6000}
        onClose={() => setError(null)}
      >
        <Alert onClose={() => setError(null)} severity="error" sx={{ width: '100%' }}>
          {error}
        </Alert>
      </Snackbar>
    </>
  );
};

