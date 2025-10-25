import { Alert, Box, Snackbar } from '@mui/material';
import React, { useCallback, useEffect, useState } from 'react';
import { webRTCService } from '../../services/communication/WebRTCService';
import { CallParticipant, CallUI } from '../ui/CallUI';

export interface CallState {
  isActive: boolean;
  callType: 'audio' | 'video';
  direction: 'incoming' | 'outgoing';
  participant: CallParticipant | null;
  localStream: MediaStream | null;
  remoteStream: MediaStream | null;
}

export const CallManager: React.FC = () => {
  const [callState, setCallState] = useState<CallState>({
    isActive: false,
    callType: 'audio',
    direction: 'outgoing',
    participant: null,
    localStream: null,
    remoteStream: null
  });

  const [notification, setNotification] = useState<{
    open: boolean;
    message: string;
    severity: 'success' | 'error' | 'info';
  }>({
    open: false,
    message: '',
    severity: 'info'
  });

  // WebRTC event listeners
  useEffect(() => {
    const handleShowCallUI = (data: any) => {
      const participant: CallParticipant = {
        id: data.entityId,
        displayName: data.entityId, // TODO: Get actual display name from entity service
        avatar: undefined,
        isMuted: false,
        hasVideo: data.callType === 'video'
      };

      setCallState(prev => ({
        ...prev,
        isActive: true,
        callType: data.callType || 'audio',
        direction: data.direction,
        participant
      }));
    };

    const handleIncomingCall = (data: any) => {
      const participant: CallParticipant = {
        id: data.entityId,
        displayName: data.entityId, // TODO: Get actual display name from entity service
        avatar: undefined,
        isMuted: false,
        hasVideo: data.callType === 'video'
      };

      setCallState({
        isActive: true,
        callType: data.callType || 'audio',
        direction: 'incoming',
        participant,
        localStream: null,
        remoteStream: null
      });
    };

    const handleRemoteStream = (data: any) => {
      setCallState(prev => ({
        ...prev,
        remoteStream: data.stream
      }));
    };

    const handleHideCallUI = (data: any) => {
      setCallState({
        isActive: false,
        callType: 'audio',
        direction: 'outgoing',
        participant: null,
        localStream: null,
        remoteStream: null
      });
    };

    const handleRemoteAudioStateChanged = (data: any) => {
      setCallState(prev => ({
        ...prev,
        participant: prev.participant ? {
          ...prev.participant,
          isMuted: !data.enabled
        } : null
      }));
    };

    const handleRemoteVideoStateChanged = (data: any) => {
      setCallState(prev => ({
        ...prev,
        participant: prev.participant ? {
          ...prev.participant,
          hasVideo: data.enabled
        } : null
      }));
    };

    const handleRemoteScreenShareStarted = (data: any) => {
      setCallState(prev => ({
        ...prev,
        participant: prev.participant ? {
          ...prev.participant,
          isScreenSharing: true
        } : null
      }));
    };

    const handleRemoteScreenShareStopped = (data: any) => {
      setCallState(prev => ({
        ...prev,
        participant: prev.participant ? {
          ...prev.participant,
          isScreenSharing: false
        } : null
      }));
    };

    const handleCallEnded = (data: any) => {
      setCallState({
        isActive: false,
        callType: 'audio',
        direction: 'outgoing',
        participant: null,
        localStream: null,
        remoteStream: null
      });

      setNotification({
        open: true,
        message: 'Call ended',
        severity: 'info'
      });
    };

    // Register event listeners
    webRTCService.on('showCallUI', handleShowCallUI);
    webRTCService.on('incomingCall', handleIncomingCall);
    webRTCService.on('remoteStream', handleRemoteStream);
    webRTCService.on('remoteAudioStateChanged', handleRemoteAudioStateChanged);
    webRTCService.on('remoteVideoStateChanged', handleRemoteVideoStateChanged);
    webRTCService.on('remoteScreenShareStarted', handleRemoteScreenShareStarted);
    webRTCService.on('remoteScreenShareStopped', handleRemoteScreenShareStopped);
    webRTCService.on('hideCallUI', handleHideCallUI);
    webRTCService.on('callEnded', handleCallEnded);

    return () => {
      // Cleanup listeners
      webRTCService.off('showCallUI', handleShowCallUI);
      webRTCService.off('incomingCall', handleIncomingCall);
      webRTCService.off('remoteStream', handleRemoteStream);
      webRTCService.off('remoteAudioStateChanged', handleRemoteAudioStateChanged);
      webRTCService.off('remoteVideoStateChanged', handleRemoteVideoStateChanged);
      webRTCService.off('remoteScreenShareStarted', handleRemoteScreenShareStarted);
      webRTCService.off('remoteScreenShareStopped', handleRemoteScreenShareStopped);
      webRTCService.off('hideCallUI', handleHideCallUI);
      webRTCService.off('callEnded', handleCallEnded);
    };
  }, []);

  const handleAcceptCall = useCallback(async () => {
    if (!callState.participant) return;

    try {
      // The WebRTC service handles the acceptance internally
      // when the user clicks accept in the UI
      console.log('Accepting call from:', callState.participant.id);

      setNotification({
        open: true,
        message: 'Call connected',
        severity: 'success'
      });
    } catch (error) {
      console.error('Failed to accept call:', error);
      setNotification({
        open: true,
        message: 'Failed to accept call',
        severity: 'error'
      });
    }
  }, [callState.participant]);

  const handleRejectCall = useCallback(() => {
    if (!callState.participant) return;

    webRTCService.endCall(callState.participant.id);
    setCallState({
      isActive: false,
      callType: 'audio',
      direction: 'outgoing',
      participant: null,
      localStream: null,
      remoteStream: null
    });

    setNotification({
      open: true,
      message: 'Call declined',
      severity: 'info'
    });
  }, [callState.participant]);

  const handleEndCall = useCallback(() => {
    if (!callState.participant) return;

    webRTCService.endCall(callState.participant.id);
    setCallState({
      isActive: false,
      callType: 'audio',
      direction: 'outgoing',
      participant: null,
      localStream: null,
      remoteStream: null
    });

    setNotification({
      open: true,
      message: 'Call ended',
      severity: 'info'
    });
  }, [callState.participant]);

  const handleToggleMute = useCallback(() => {
    if (!callState.participant) return;

    const newMutedState = webRTCService.toggleAudio(callState.participant.id);
    console.log('Toggle mute:', newMutedState);

    // Update local participant state
    setCallState(prev => ({
      ...prev,
      participant: prev.participant ? {
        ...prev.participant,
        isMuted: !newMutedState
      } : null
    }));
  }, [callState.participant]);

  const handleToggleVideo = useCallback(() => {
    if (!callState.participant) return;

    const newVideoState = webRTCService.toggleVideo(callState.participant.id);
    console.log('Toggle video:', newVideoState);

    // Update local participant state
    setCallState(prev => ({
      ...prev,
      participant: prev.participant ? {
        ...prev.participant,
        hasVideo: newVideoState
      } : null
    }));
  }, [callState.participant]);

  const handleToggleSpeaker = useCallback(() => {
    // TODO: Implement speaker on/off functionality
    console.log('Toggle speaker');
    setNotification({
      open: true,
      message: 'Speaker toggle not yet implemented',
      severity: 'info'
    });
  }, []);

  const handleToggleScreenShare = useCallback(async () => {
    if (!callState.participant) return;

    try {
      const mediaState = webRTCService.getMediaState();
      if (mediaState.screenSharing) {
        // Stop screen sharing
        await webRTCService.stopScreenShare(callState.participant.id);
        setNotification({
          open: true,
          message: 'Screen sharing stopped',
          severity: 'info'
        });
      } else {
        // Start screen sharing
        const success = await webRTCService.startScreenShare(callState.participant.id);
        if (success) {
          setNotification({
            open: true,
            message: 'Screen sharing started',
            severity: 'success'
          });
        } else {
          setNotification({
            open: true,
            message: 'Failed to start screen sharing',
            severity: 'error'
          });
        }
      }
    } catch (error) {
      console.error('Screen share toggle failed:', error);
      setNotification({
        open: true,
        message: 'Screen share toggle failed',
        severity: 'error'
      });
    }
  }, [callState.participant]);

  const handleCloseNotification = () => {
    setNotification(prev => ({ ...prev, open: false }));
  };

  // Public methods that can be called from outside
  React.useImperativeHandle(React.forwardRef(() => ({})), () => ({
    startAudioCall: async (entityId: string, entityType: string) => {
      try {
        const participant: CallParticipant = {
          id: entityId,
          displayName: entityId, // TODO: Get actual display name
          avatar: undefined,
          isMuted: false,
          hasVideo: false
        };

        setCallState({
          isActive: true,
          callType: 'audio',
          direction: 'outgoing',
          participant,
          localStream: null,
          remoteStream: null
        });

        await webRTCService.startAudioCall(entityId, entityType);
      } catch (error) {
        console.error('Failed to start audio call:', error);
        setCallState(prev => ({ ...prev, isActive: false }));
        setNotification({
          open: true,
          message: 'Failed to start audio call',
          severity: 'error'
        });
      }
    },

    startVideoCall: async (entityId: string, entityType: string) => {
      try {
        const participant: CallParticipant = {
          id: entityId,
          displayName: entityId, // TODO: Get actual display name
          avatar: undefined,
          isMuted: false,
          hasVideo: true
        };

        setCallState({
          isActive: true,
          callType: 'video',
          direction: 'outgoing',
          participant,
          localStream: null,
          remoteStream: null
        });

        await webRTCService.startVideoCall(entityId, entityType);
      } catch (error) {
        console.error('Failed to start video call:', error);
        setCallState(prev => ({ ...prev, isActive: false }));
        setNotification({
          open: true,
          message: 'Failed to start video call',
          severity: 'error'
        });
      }
    }
  }));

  return (
    <Box>
      {callState.participant && (
        <CallUI
          open={callState.isActive}
          callType={callState.callType}
          direction={callState.direction}
          participant={callState.participant}
          localStream={callState.localStream}
          remoteStream={callState.remoteStream}
          onAccept={callState.direction === 'incoming' ? handleAcceptCall : undefined}
          onReject={callState.direction === 'incoming' ? handleRejectCall : undefined}
          onEnd={callState.direction === 'outgoing' ? handleEndCall : undefined}
          onToggleMute={handleToggleMute}
          onToggleVideo={handleToggleVideo}
          onToggleSpeaker={handleToggleSpeaker}
          onToggleScreenShare={handleToggleScreenShare}
        />
      )}

      <Snackbar
        open={notification.open}
        autoHideDuration={4000}
        onClose={handleCloseNotification}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
      >
        <Alert
          onClose={handleCloseNotification}
          severity={notification.severity}
          sx={{ width: '100%' }}
        >
          {notification.message}
        </Alert>
      </Snackbar>
    </Box>
  );
};

// Create a singleton instance
export const callManager = React.createRef<any>();

export default CallManager;
