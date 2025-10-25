import {
    CallEnd,
    Mic,
    MicOff, ScreenShare,
    StopScreenShare, Videocam,
    VideocamOff, VolumeOff, VolumeUp
} from '@mui/icons-material';
import {
    Avatar, Box, Button, Chip, Dialog, DialogActions, DialogContent, IconButton, Paper, Typography
} from '@mui/material';
import React, { useEffect, useRef, useState } from 'react';

export interface CallParticipant {
  id: string;
  displayName: string;
  avatar?: string;
  isMuted?: boolean;
  hasVideo?: boolean;
  isScreenSharing?: boolean;
}

export interface CallUIProps {
  open: boolean;
  callType: 'audio' | 'video';
  direction: 'incoming' | 'outgoing';
  participant: CallParticipant;
  localStream?: MediaStream;
  remoteStream?: MediaStream;
  onAccept?: () => void;
  onReject?: () => void;
  onEnd?: () => void;
  onToggleMute?: () => void;
  onToggleVideo?: () => void;
  onToggleSpeaker?: () => void;
  onToggleScreenShare?: () => void;
}

export const CallUI: React.FC<CallUIProps> = ({
  open,
  callType,
  direction,
  participant,
  localStream,
  remoteStream,
  onAccept,
  onReject,
  onEnd,
  onToggleMute,
  onToggleVideo,
  onToggleSpeaker,
  onToggleScreenShare
}) => {
  const [isMuted, setIsMuted] = useState(false);
  const [isVideoEnabled, setIsVideoEnabled] = useState(callType === 'video');
  const [isSpeakerOn, setIsSpeakerOn] = useState(true);
  const [isScreenSharing, setIsScreenSharing] = useState(participant.isScreenSharing || false);
  const [callDuration, setCallDuration] = useState(0);
  const [callStartTime, setCallStartTime] = useState<number | null>(null);

  const localVideoRef = useRef<HTMLVideoElement>(null);
  const remoteVideoRef = useRef<HTMLVideoElement>(null);

  // Timer for call duration
  useEffect(() => {
    let interval: NodeJS.Timeout;
    if (callStartTime && direction === 'outgoing') {
      interval = setInterval(() => {
        setCallDuration(Math.floor((Date.now() - callStartTime) / 1000));
      }, 1000);
    }
    return () => {
      if (interval) clearInterval(interval);
    };
  }, [callStartTime, direction]);

  // Handle local stream
  useEffect(() => {
    if (localVideoRef.current && localStream) {
      localVideoRef.current.srcObject = localStream;
    }
  }, [localStream]);

  // Handle remote stream
  useEffect(() => {
    if (remoteVideoRef.current && remoteStream) {
      remoteVideoRef.current.srcObject = remoteStream;
      if (!callStartTime) {
        setCallStartTime(Date.now());
      }
    }
  }, [remoteStream, callStartTime]);

  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  const handleToggleMute = () => {
    setIsMuted(!isMuted);
    onToggleMute?.();
  };

  const handleToggleVideo = () => {
    setIsVideoEnabled(!isVideoEnabled);
    onToggleVideo?.();
  };

  const handleToggleSpeaker = () => {
    setIsSpeakerOn(!isSpeakerOn);
    onToggleSpeaker?.();
  };

  const handleToggleScreenShare = () => {
    setIsScreenSharing(!isScreenSharing);
    onToggleScreenShare?.();
  };

  const getCallStatusText = () => {
    if (direction === 'incoming') {
      return 'Incoming call...';
    }
    if (remoteStream) {
      return formatDuration(callDuration);
    }
    return 'Calling...';
  };

  return (
    <Dialog
      open={open}
      maxWidth="md"
      fullWidth
      PaperProps={{
        sx: {
          bgcolor: 'background.paper',
          borderRadius: 3,
          overflow: 'hidden'
        }
      }}
    >
      <DialogContent sx={{ p: 0, position: 'relative' }}>
        {/* Video Display Area */}
        <Box
          sx={{
            position: 'relative',
            height: callType === 'video' ? 400 : 200,
            bgcolor: 'black',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center'
          }}
        >
          {callType === 'video' && remoteStream ? (
            <video
              ref={remoteVideoRef}
              autoPlay
              playsInline
              style={{
                width: '100%',
                height: '100%',
                objectFit: 'cover'
              }}
            />
          ) : (
            // Audio call view or no remote stream yet
            <Box
              sx={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                gap: 2,
                color: 'white'
              }}
            >
              <Avatar
                src={participant.avatar}
                sx={{
                  width: 120,
                  height: 120,
                  bgcolor: 'primary.main',
                  fontSize: '3rem'
                }}
              >
                {participant.displayName.charAt(0).toUpperCase()}
              </Avatar>
              <Typography variant="h5" fontWeight="bold">
                {participant.displayName}
              </Typography>
              <Chip
                label={getCallStatusText()}
                sx={{
                  bgcolor: 'rgba(255, 255, 255, 0.2)',
                  color: 'white',
                  fontSize: '0.9rem'
                }}
              />
            </Box>
          )}

          {/* Local video thumbnail (picture-in-picture) */}
          {callType === 'video' && localStream && (
            <Paper
              elevation={4}
              sx={{
                position: 'absolute',
                bottom: 16,
                right: 16,
                width: 120,
                height: 90,
                borderRadius: 2,
                overflow: 'hidden'
              }}
            >
              <video
                ref={localVideoRef}
                autoPlay
                playsInline
                muted
                style={{
                  width: '100%',
                  height: '100%',
                  objectFit: 'cover'
                }}
              />
            </Paper>
          )}
        </Box>

        {/* Call Controls */}
        <Box
          sx={{
            p: 3,
            display: 'flex',
            justifyContent: 'center',
            gap: 2,
            bgcolor: 'background.default'
          }}
        >
          {/* Mute/Unmute */}
          <IconButton
            onClick={handleToggleMute}
            sx={{
              bgcolor: isMuted ? 'error.main' : 'grey.700',
              color: 'white',
              '&:hover': {
                bgcolor: isMuted ? 'error.dark' : 'grey.800'
              },
              width: 56,
              height: 56
            }}
          >
            {isMuted ? <MicOff /> : <Mic />}
          </IconButton>

          {/* Video toggle (video calls only) */}
          {callType === 'video' && (
            <IconButton
              onClick={handleToggleVideo}
              sx={{
                bgcolor: !isVideoEnabled ? 'error.main' : 'grey.700',
                color: 'white',
                '&:hover': {
                  bgcolor: !isVideoEnabled ? 'error.dark' : 'grey.800'
                },
                width: 56,
                height: 56
              }}
            >
              {!isVideoEnabled ? <VideocamOff /> : <Videocam />}
            </IconButton>
          )}

          {/* Speaker toggle */}
          <IconButton
            onClick={handleToggleSpeaker}
            sx={{
              bgcolor: !isSpeakerOn ? 'error.main' : 'grey.700',
              color: 'white',
              '&:hover': {
                bgcolor: !isSpeakerOn ? 'error.dark' : 'grey.800'
              },
              width: 56,
              height: 56
            }}
          >
            {!isSpeakerOn ? <VolumeOff /> : <VolumeUp />}
          </IconButton>

          {/* Screen share toggle */}
          <IconButton
            onClick={handleToggleScreenShare}
            sx={{
              bgcolor: isScreenSharing ? 'primary.main' : 'grey.700',
              color: 'white',
              '&:hover': {
                bgcolor: isScreenSharing ? 'primary.dark' : 'grey.800'
              },
              width: 56,
              height: 56
            }}
          >
            {isScreenSharing ? <StopScreenShare /> : <ScreenShare />}
          </IconButton>
        </Box>
      </DialogContent>

      <DialogActions
        sx={{
          p: 3,
          justifyContent: 'center',
          bgcolor: 'background.default'
        }}
      >
        {direction === 'incoming' ? (
          // Incoming call buttons
          <>
            <Button
              variant="contained"
              color="error"
              startIcon={<CallEnd />}
              onClick={onReject}
              sx={{
                minWidth: 120,
                bgcolor: 'error.main',
                '&:hover': { bgcolor: 'error.dark' }
              }}
            >
              Decline
            </Button>
            <Button
              variant="contained"
              color="success"
              startIcon={<CallEnd sx={{ transform: 'rotate(135deg)' }} />}
              onClick={onAccept}
              sx={{
                minWidth: 120,
                bgcolor: 'success.main',
                '&:hover': { bgcolor: 'success.dark' }
              }}
            >
              Accept
            </Button>
          </>
        ) : (
          // Outgoing/connected call button
          <Button
            variant="contained"
            color="error"
            startIcon={<CallEnd />}
            onClick={onEnd}
            sx={{
              minWidth: 120,
              bgcolor: 'error.main',
              '&:hover': { bgcolor: 'error.dark' }
            }}
          >
            End Call
          </Button>
        )}
      </DialogActions>
    </Dialog>
  );
};

export default CallUI;
