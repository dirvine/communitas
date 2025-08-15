import React, { useState, useEffect } from 'react';
import { Box, Fab, Snackbar, Alert } from '@mui/material';
import { Phone, Message } from '@mui/icons-material';
import { webRTCService, CallState } from '../../services/webrtc/WebRTCService';
import { SimpleCallInterface } from './SimpleCallInterface';

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

    webRTCService.on('callInitiated', handleCallInitiated);
    webRTCService.on('callEnded', handleCallEnded);
    webRTCService.on('error', handleError);

    return () => {
      webRTCService.off('callInitiated', handleCallInitiated);
      webRTCService.off('callEnded', handleCallEnded);
      webRTCService.off('error', handleError);
    };
  }, []);

  const handleStartCall = async () => {
    try {
      await webRTCService.initiateCall('demo-contact', 'audio');
    } catch (error) {
      console.error('Failed to start call:', error);
    }
  };


  const handleEndCall = async () => {
    await webRTCService.endCall();
  };

  const handleSendMessage = () => {
    webRTCService.sendMessage('Hello from WebRTC!');
  };

  return (
    <>
      {currentCall && (
        <SimpleCallInterface
          call={currentCall}
          onEndCall={handleEndCall}
        />
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

