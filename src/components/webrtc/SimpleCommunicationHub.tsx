import { Message, Phone } from '@mui/icons-material';
import { Alert, Box, Fab, Snackbar } from '@mui/material';
import React, { useEffect, useState } from 'react';
import type { CallInfo } from '../../services/webrtc/types';
import { createAudioOnlyConstraints } from '../../services/webrtc/types';
import { webrtcService } from '../../services/webrtc/WebRTCService';
// Removed: SimpleCallInterface - using modern shell instead

export const SimpleCommunicationHub: React.FC = () => {
  const [currentCall, setCurrentCall] = useState<CallInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let unsubscribe: (() => void) | null = null;

    const setupEventListener = async () => {
      try {
        unsubscribe = await webrtcService.subscribeToCallEvents((event) => {
          // Handle different event types
          if (event.type === 'call-initiated' || event.type === 'incoming-call') {
            // We'd need to fetch call info separately or the event should contain it
            // For now, just clear any error
            setError(null);
          } else if (event.type === 'call-ended') {
            setCurrentCall(null);
          } else if (event.type === 'call-error') {
            setError(event.error);
          }
        });
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to subscribe to call events');
      }
    };

    setupEventListener();

    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }, []);

  const handleStartCall = async () => {
    try {
      await webrtcService.initiateCall('demo-contact', createAudioOnlyConstraints());
    } catch (error) {
      console.error('Failed to start call:', error);
    }
  };

  const handleSendMessage = () => {
    // TODO: Messaging not yet implemented in WebRTC service
    console.log('Message button clicked - WebRTC messaging not yet implemented');
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

