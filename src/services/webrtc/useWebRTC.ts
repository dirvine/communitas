// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

/**
 * useWebRTC Hook
 *
 * React hook for managing WebRTC calls, media controls, and call events.
 * Provides state management and lifecycle methods for voice, video, and screen sharing.
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import type { CallEvent, MediaConstraints, MediaDevice } from './types';
import { CallEventType, isIncomingCallEvent } from './types';
import { webrtcService } from './WebRTCService';

/**
 * WebRTC hook state
 */
export interface UseWebRTCState {
  // Initialization
  isInitialized: boolean;

  // Call state
  currentCall: CallEvent | null;
  incomingCall: CallEvent | null;

  // Media state
  devices: MediaDevice[];
  isVideoEnabled: boolean;
  isAudioEnabled: boolean;
  isScreenSharing: boolean;

  // Error state
  error: Error | null;

  // Call lifecycle methods
  initiateCall: (targetFourWords: string, constraints: MediaConstraints) => Promise<string>;
  acceptCall: (callId: string) => Promise<void>;
  rejectCall: (callId: string) => Promise<void>;
  endCall: (callId: string) => Promise<void>;

  // Media control methods
  setVideoEnabled: (callId: string, enabled: boolean) => Promise<void>;
  setAudioEnabled: (callId: string, enabled: boolean) => Promise<void>;
  startScreenShare: (callId: string) => Promise<void>;
  stopScreenShare: (callId: string) => Promise<void>;

  // Device management
  loadDevices: () => Promise<void>;

  // Error handling
  clearError: () => void;
}

/**
 * useWebRTC Hook
 *
 * Manages WebRTC call lifecycle, media controls, and event handling.
 */
export function useWebRTC(): UseWebRTCState {
  // State
  const [isInitialized, setIsInitialized] = useState(false);
  const [currentCall, setCurrentCall] = useState<CallEvent | null>(null);
  const [incomingCall, setIncomingCall] = useState<CallEvent | null>(null);
  const [devices, setDevices] = useState<MediaDevice[]>([]);
  const [isVideoEnabled, setIsVideoEnabled] = useState(false);
  const [isAudioEnabled, setIsAudioEnabled] = useState(false);
  const [isScreenSharing, setIsScreenSharing] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  // Refs for cleanup
  const unsubscribeRef = useRef<(() => void) | null>(null);

  // Handle call events
  const handleCallEvent = useCallback((event: CallEvent) => {
    switch (event.type) {
      case CallEventType.IncomingCall:
        if (isIncomingCallEvent(event)) {
          setIncomingCall(event);
        }
        break;

      case CallEventType.CallInitiated:
      case CallEventType.CallAccepted:
        setCurrentCall(event);
        setIncomingCall(null);
        break;

      case CallEventType.CallEnded:
      case CallEventType.CallRejected:
        setCurrentCall(null);
        setIncomingCall(null);
        setIsVideoEnabled(false);
        setIsAudioEnabled(false);
        setIsScreenSharing(false);
        break;

      case CallEventType.VideoStateChanged:
        if ('enabled' in event) {
          setIsVideoEnabled(event.enabled);
        }
        break;

      case CallEventType.AudioStateChanged:
        if ('enabled' in event) {
          setIsAudioEnabled(event.enabled);
        }
        break;

      case CallEventType.ScreenShareStateChanged:
        if ('active' in event) {
          setIsScreenSharing(event.active);
        }
        break;

      case CallEventType.CallError:
        if ('error' in event) {
          setError(new Error(event.error));
        }
        break;
    }
  }, []);

  // Initialize service on mount
  useEffect(() => {
    const initializeService = async () => {
      try {
        await webrtcService.initialize();
        setIsInitialized(true);

        // Subscribe to call events
        const unsubscribe = await webrtcService.subscribeToCallEvents(handleCallEvent);
        unsubscribeRef.current = unsubscribe;
      } catch (err) {
        setError(err instanceof Error ? err : new Error(String(err)));
        setIsInitialized(false);
      }
    };

    initializeService();

    // Cleanup on unmount
    return () => {
      if (unsubscribeRef.current) {
        unsubscribeRef.current();
      }
    };
  }, [handleCallEvent]);

  // Call lifecycle methods
  const initiateCall = useCallback(
    async (targetFourWords: string, constraints: MediaConstraints): Promise<string> => {
      try {
        const callId = await webrtcService.initiateCall(targetFourWords, constraints);
        return callId;
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        throw error;
      }
    },
    []
  );

  const acceptCall = useCallback(async (callId: string): Promise<void> => {
    try {
      await webrtcService.acceptCall(callId);
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      throw error;
    }
  }, []);

  const rejectCall = useCallback(async (callId: string): Promise<void> => {
    try {
      await webrtcService.rejectCall(callId);
      setIncomingCall(null);
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      throw error;
    }
  }, []);

  const endCall = useCallback(async (callId: string): Promise<void> => {
    try {
      await webrtcService.endCall(callId);
      setCurrentCall(null);
      setIsVideoEnabled(false);
      setIsAudioEnabled(false);
      setIsScreenSharing(false);
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      throw error;
    }
  }, []);

  // Media control methods
  const setVideoEnabledMethod = useCallback(
    async (callId: string, enabled: boolean): Promise<void> => {
      try {
        await webrtcService.setVideoEnabled(callId, enabled);
        setIsVideoEnabled(enabled);
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        throw error;
      }
    },
    []
  );

  const setAudioEnabledMethod = useCallback(
    async (callId: string, enabled: boolean): Promise<void> => {
      try {
        await webrtcService.setAudioEnabled(callId, enabled);
        setIsAudioEnabled(enabled);
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        throw error;
      }
    },
    []
  );

  const startScreenShare = useCallback(async (callId: string): Promise<void> => {
    try {
      await webrtcService.startScreenShare(callId);
      setIsScreenSharing(true);
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      throw error;
    }
  }, []);

  const stopScreenShare = useCallback(async (callId: string): Promise<void> => {
    try {
      await webrtcService.stopScreenShare(callId);
      setIsScreenSharing(false);
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      throw error;
    }
  }, []);

  // Device management
  const loadDevices = useCallback(async (): Promise<void> => {
    try {
      const mediaDevices = await webrtcService.getMediaDevices();
      setDevices(mediaDevices);
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      setError(error);
      throw error;
    }
  }, []);

  // Error handling
  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    isInitialized,
    currentCall,
    incomingCall,
    devices,
    isVideoEnabled,
    isAudioEnabled,
    isScreenSharing,
    error,
    initiateCall,
    acceptCall,
    rejectCall,
    endCall,
    setVideoEnabled: setVideoEnabledMethod,
    setAudioEnabled: setAudioEnabledMethod,
    startScreenShare,
    stopScreenShare,
    loadDevices,
    clearError,
  };
}
