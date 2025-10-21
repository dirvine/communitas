import React, { useState, useEffect } from 'react';
import { Box, Fab, Snackbar, Alert } from '@mui/material';
import { Phone, Message } from '@mui/icons-material';
import { webrtcService } from '../../services/webrtc/WebRTCService';
import { CallState } from '../../services/webrtc/types';
// Removed: SimpleCallInterface - using modern shell instead

export const SimpleCommunicationHub: React.FC = () => {
  const [currentCall, setCurrentCall] = useState<CallState | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const handleCallInitiated = (call: CallState) => {
      setCurrentCall(call);
    };

    const handleCallEnded = () => {
      setCurrentCall(null);
    };

    const handleError = (error: Error) => {
      setError(error.message);
    };

    webrtcService.on('callInitiated', handleCallInitiated);
    webrtcService.on('callEnded', handleCallEnded);
    webrtcService.on('error', handleError);

    return () => {
      webrtcService.off('callInitiated', handleCallInitiated);
      webrtcService.off('callEnded', handleCallEnded);
      webrtcService.off('error', handleError);
    };
  }, []);

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
      {currentCall && (
        <Box>Call interface placeholder - use modern shell</Box>
      )}

      {!currentCall && (
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

