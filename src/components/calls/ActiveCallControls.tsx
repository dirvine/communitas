// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

/**
 * Active Call Controls Component
 *
 * Provides UI controls for managing an active WebRTC call:
 * - Video toggle (on/off)
 * - Audio toggle (mute/unmute)
 * - Screen share toggle (start/stop)
 * - End call button
 */

import {
    CallEnd, Mic,
    MicOff,
    ScreenShare,
    StopScreenShare, Videocam,
    VideocamOff
} from '@mui/icons-material';
import {
    Box, CircularProgress, IconButton, Tooltip
} from '@mui/material';
import { useState } from 'react';

export interface ActiveCallControlsProps {
  /** Call ID for the active call */
  callId: string;

  /** Whether video is currently enabled */
  isVideoEnabled: boolean;

  /** Whether audio is currently enabled (not muted) */
  isAudioEnabled: boolean;

  /** Whether screen sharing is active */
  isScreenSharing: boolean;

  /** Callback when video toggle is clicked */
  onVideoToggle: (enabled: boolean) => Promise<void> | void;

  /** Callback when audio toggle is clicked */
  onAudioToggle: (enabled: boolean) => Promise<void> | void;

  /** Callback when screen share toggle is clicked */
  onScreenShareToggle: (active: boolean) => Promise<void> | void;

  /** Callback when end call is clicked */
  onEndCall: () => Promise<void> | void;
}

/**
 * ActiveCallControls Component
 *
 * Renders control buttons for managing an active WebRTC call.
 */
export function ActiveCallControls({
  callId,
  isVideoEnabled,
  isAudioEnabled,
  isScreenSharing,
  onVideoToggle,
  onAudioToggle,
  onScreenShareToggle,
  onEndCall,
}: ActiveCallControlsProps): JSX.Element {
  const [isVideoToggling, setIsVideoToggling] = useState(false);
  const [isAudioToggling, setIsAudioToggling] = useState(false);
  const [isScreenToggling, setIsScreenToggling] = useState(false);
  const [isEnding, setIsEnding] = useState(false);

  const handleVideoToggle = async () => {
    setIsVideoToggling(true);
    try {
      await onVideoToggle(!isVideoEnabled);
    } catch (error) {
      console.error('Failed to toggle video:', error);
    } finally {
      setIsVideoToggling(false);
    }
  };

  const handleAudioToggle = async () => {
    setIsAudioToggling(true);
    try {
      await onAudioToggle(!isAudioEnabled);
    } catch (error) {
      console.error('Failed to toggle audio:', error);
    } finally {
      setIsAudioToggling(false);
    }
  };

  const handleScreenShareToggle = async () => {
    setIsScreenToggling(true);
    try {
      await onScreenShareToggle(!isScreenSharing);
    } catch (error) {
      console.error('Failed to toggle screen share:', error);
    } finally {
      setIsScreenToggling(false);
    }
  };

  const handleEndCall = async () => {
    setIsEnding(true);
    try {
      await onEndCall();
    } catch (error) {
      console.error('Failed to end call:', error);
    } finally {
      setIsEnding(false);
    }
  };

  return (
    <Box
      sx={{
        display: 'flex',
        gap: 2,
        justifyContent: 'center',
        alignItems: 'center',
        padding: 2,
      }}
      role="group"
      aria-label="Call controls"
    >
      {/* Video Toggle */}
      <Tooltip title={isVideoEnabled ? 'Turn off camera' : 'Turn on camera'}>
        <IconButton
          onClick={handleVideoToggle}
          disabled={isVideoToggling}
          aria-label={isVideoEnabled ? 'Turn off video' : 'Turn on video'}
          aria-pressed={isVideoEnabled}
          tabIndex={0}
          color={isVideoEnabled ? 'primary' : 'default'}
          sx={{
            backgroundColor: isVideoEnabled ? 'primary.main' : 'action.disabled',
            color: isVideoEnabled ? 'primary.contrastText' : 'text.primary',
            '&:hover': {
              backgroundColor: isVideoEnabled ? 'primary.dark' : 'action.hover',
            },
          }}
        >
          {isVideoToggling ? (
            <CircularProgress size={24} />
          ) : isVideoEnabled ? (
            <Videocam />
          ) : (
            <VideocamOff />
          )}
        </IconButton>
      </Tooltip>

      {/* Audio Toggle */}
      <Tooltip title={isAudioEnabled ? 'Mute microphone' : 'Unmute microphone'}>
        <IconButton
          onClick={handleAudioToggle}
          disabled={isAudioToggling}
          aria-label={isAudioEnabled ? 'Mute audio' : 'Unmute audio'}
          aria-pressed={isAudioEnabled}
          tabIndex={0}
          color={isAudioEnabled ? 'primary' : 'default'}
          sx={{
            backgroundColor: isAudioEnabled ? 'primary.main' : 'action.disabled',
            color: isAudioEnabled ? 'primary.contrastText' : 'text.primary',
            '&:hover': {
              backgroundColor: isAudioEnabled ? 'primary.dark' : 'action.hover',
            },
          }}
        >
          {isAudioToggling ? (
            <CircularProgress size={24} />
          ) : isAudioEnabled ? (
            <Mic />
          ) : (
            <MicOff />
          )}
        </IconButton>
      </Tooltip>

      {/* Screen Share Toggle */}
      <Tooltip title={isScreenSharing ? 'Stop sharing screen' : 'Share screen'}>
        <IconButton
          onClick={handleScreenShareToggle}
          disabled={isScreenToggling}
          aria-label={isScreenSharing ? 'Stop screen share' : 'Start screen share'}
          aria-pressed={isScreenSharing}
          tabIndex={0}
          color={isScreenSharing ? 'primary' : 'default'}
          sx={{
            backgroundColor: isScreenSharing ? 'primary.main' : 'action.disabled',
            color: isScreenSharing ? 'primary.contrastText' : 'text.primary',
            '&:hover': {
              backgroundColor: isScreenSharing ? 'primary.dark' : 'action.hover',
            },
          }}
        >
          {isScreenToggling ? (
            <CircularProgress size={24} />
          ) : isScreenSharing ? (
            <StopScreenShare />
          ) : (
            <ScreenShare />
          )}
        </IconButton>
      </Tooltip>

      {/* End Call */}
      <Tooltip title="End call">
        <IconButton
          onClick={handleEndCall}
          disabled={isEnding}
          aria-label="End call"
          tabIndex={0}
          sx={{
            backgroundColor: 'error.main',
            color: 'error.contrastText',
            '&:hover': {
              backgroundColor: 'error.dark',
            },
          }}
        >
          {isEnding ? <CircularProgress size={24} /> : <CallEnd />}
        </IconButton>
      </Tooltip>
    </Box>
  );
}
