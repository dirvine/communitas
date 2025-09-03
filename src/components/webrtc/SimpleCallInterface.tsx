import React, { useState, useEffect } from 'react';
import { Box, Button, Typography, Avatar, IconButton, Paper } from '@mui/material';
import {
  CallEnd,
  Mic,
  MicOff,
  Videocam,
  VideocamOff,
  VolumeUp,
  VolumeOff
} from '@mui/icons-material';
import { CallState, webRTCService } from '../../services/webrtc/WebRTCService';

interface SimpleCallInterfaceProps {
  call: CallState;
  onEndCall: () => void;
}

export const SimpleCallInterface: React.FC<SimpleCallInterfaceProps> = ({
  call,
  onEndCall
}) => {
  const [isMicOn, setIsMicOn] = useState(true);
  const [isVideoOn, setIsVideoOn] = useState(call.type === 'video');
  const [isSpeakerOn, setIsSpeakerOn] = useState(true);
  const [callDuration, setCallDuration] = useState(0);

  useEffect(() => {
    const interval = setInterval(() => {
      setCallDuration(prev => prev + 1);
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  const formatDuration = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  const handleToggleMic = () => {
    webRTCService.toggleAudio();
    setIsMicOn(!isMicOn);
  };

  const handleToggleVideo = () => {
    webRTCService.toggleVideo();
    setIsVideoOn(!isVideoOn);
  };

  const handleToggleSpeaker = () => {
    // In a real implementation, this would control audio output
    setIsSpeakerOn(!isSpeakerOn);
  };

  return (
    <Box
      sx={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: '#000',
        color: 'white',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'space-between',
        zIndex: 9999,
        p: 4
      }}
    >
      {/* Header with call info */}
      <Box sx={{ textAlign: 'center', mt: 4 }}>
        <Typography variant="h4" sx={{ mb: 2 }}>
          {call.type === 'video' ? 'Video Call' : 'Audio Call'}
        </Typography>
        <Typography variant="h6" sx={{ mb: 1, opacity: 0.8 }}>
          {formatDuration(callDuration)}
        </Typography>
        <Typography variant="body1" sx={{ opacity: 0.6 }}>
          {call.participants.length} participant{call.participants.length !== 1 ? 's' : ''}
        </Typography>
      </Box>

      {/* Participant avatars/videos */}
      <Box sx={{ display: 'flex', gap: 2, mb: 4 }}>
        {call.participants.map((participant) => (
          <Paper
            key={participant.id}
            elevation={3}
            sx={{
              width: 120,
              height: 120,
              borderRadius: '50%',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              backgroundColor: 'rgba(255, 255, 255, 0.1)',
              border: participant.isVideoEnabled ? '2px solid #4caf50' : 'none'
            }}
          >
            {participant.isVideoEnabled ? (
              <Box sx={{
                width: '100%',
                height: '100%',
                borderRadius: '50%',
                backgroundColor: '#333',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center'
              }}>
                <Typography variant="body2" color="white">
                  Video On
                </Typography>
              </Box>
            ) : (
              <Avatar
                sx={{
                  width: '100%',
                  height: '100%',
                  fontSize: '2rem',
                  bgcolor: 'primary.main'
                }}
              >
                {participant.displayName.charAt(0).toUpperCase()}
              </Avatar>
            )}
          </Paper>
        ))}
      </Box>

      {/* Call controls */}
      <Box sx={{
        display: 'flex',
        gap: 2,
        alignItems: 'center',
        mb: 4,
        p: 2,
        borderRadius: 3,
        backgroundColor: 'rgba(255, 255, 255, 0.1)'
      }}>
        {/* Microphone toggle */}
        <IconButton
          onClick={handleToggleMic}
          sx={{
            bgcolor: isMicOn ? 'rgba(255, 255, 255, 0.2)' : 'error.main',
            color: 'white',
            '&:hover': {
              bgcolor: isMicOn ? 'rgba(255, 255, 255, 0.3)' : 'error.dark'
            }
          }}
          size="large"
        >
          {isMicOn ? <Mic /> : <MicOff />}
        </IconButton>

        {/* Video toggle (only show for video calls) */}
        {call.type === 'video' && (
          <IconButton
            onClick={handleToggleVideo}
            sx={{
              bgcolor: isVideoOn ? 'rgba(255, 255, 255, 0.2)' : 'error.main',
              color: 'white',
              '&:hover': {
                bgcolor: isVideoOn ? 'rgba(255, 255, 255, 0.3)' : 'error.dark'
              }
            }}
            size="large"
          >
            {isVideoOn ? <Videocam /> : <VideocamOff />}
          </IconButton>
        )}

        {/* Speaker toggle */}
        <IconButton
          onClick={handleToggleSpeaker}
          sx={{
            bgcolor: isSpeakerOn ? 'rgba(255, 255, 255, 0.2)' : 'error.main',
            color: 'white',
            '&:hover': {
              bgcolor: isSpeakerOn ? 'rgba(255, 255, 255, 0.3)' : 'error.dark'
            }
          }}
          size="large"
        >
          {isSpeakerOn ? <VolumeUp /> : <VolumeOff />}
        </IconButton>

        {/* End call button */}
        <Button
          variant="contained"
          color="error"
          onClick={onEndCall}
          startIcon={<CallEnd />}
          size="large"
          sx={{
            borderRadius: '50px',
            px: 4,
            py: 2,
            ml: 2,
            bgcolor: 'error.main',
            '&:hover': {
              bgcolor: 'error.dark'
            }
          }}
        >
          End Call
        </Button>
      </Box>
    </Box>
  );
};

