import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { useWebRTC } from '../useWebRTC';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { CallEvent, CallEventType, MediaDevice } from '../types';

// Mock the Tauri API modules
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

// Get mocked functions
const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);

describe('useWebRTC Hook', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('Initialization', () => {
    it('should initialize with default state', () => {
      mockInvoke.mockResolvedValue(undefined);

      const { result } = renderHook(() => useWebRTC());

      expect(result.current.isInitialized).toBe(false);
      expect(result.current.currentCall).toBeNull();
      expect(result.current.incomingCall).toBeNull();
      expect(result.current.devices).toEqual([]);
      expect(result.current.isVideoEnabled).toBe(false);
      expect(result.current.isAudioEnabled).toBe(false);
      expect(result.current.isScreenSharing).toBe(false);
      expect(result.current.error).toBeNull();
    });

    it('should initialize service on mount', async () => {
      mockInvoke.mockResolvedValue(undefined);
      mockListen.mockResolvedValue(vi.fn());

      const { result } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(result.current.isInitialized).toBe(true);
      });

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_subscribe_events');
    });

    it('should handle initialization errors', async () => {
      mockInvoke.mockRejectedValue(new Error('Backend not ready'));

      const { result } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(result.current.error).toBeTruthy();
      });

      expect(result.current.isInitialized).toBe(false);
    });
  });

  describe('Call Lifecycle', () => {
    it('should initiate a call', async () => {
      const mockCallId = 'call_test-uuid-1234';
      mockInvoke.mockResolvedValueOnce(undefined); // initialize
      mockListen.mockResolvedValue(vi.fn());
      mockInvoke.mockResolvedValueOnce(mockCallId); // initiate call

      const { result } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(result.current.isInitialized).toBe(true);
      });

      let callId: string | undefined;
      await act(async () => {
        callId = await result.current.initiateCall('ocean-forest-moon-star', {
          has_audio: true,
          has_video: true,
          has_screen_share: false,
        });
      });

      expect(callId).toBe(mockCallId);
      expect(mockInvoke).toHaveBeenCalledWith('webrtc_initiate_call', {
        targetFourWords: 'ocean-forest-moon-star',
        hasAudio: true,
        hasVideo: true,
        hasScreenShare: false,
      });
    });

    it('should accept an incoming call', async () => {
      mockInvoke.mockResolvedValue(undefined);
      mockListen.mockResolvedValue(vi.fn());

      const { result } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(result.current.isInitialized).toBe(true);
      });

      await act(async () => {
        await result.current.acceptCall('call_test-uuid');
      });

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_accept_call', {
        callId: 'call_test-uuid',
      });
    });

    it('should reject an incoming call', async () => {
      mockInvoke.mockResolvedValue(undefined);
      mockListen.mockResolvedValue(vi.fn());

      const { result } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(result.current.isInitialized).toBe(true);
      });

      await act(async () => {
        await result.current.rejectCall('call_test-uuid');
      });

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_reject_call', {
        callId: 'call_test-uuid',
      });
    });

    it('should end an active call', async () => {
      mockInvoke.mockResolvedValue(undefined);
      mockListen.mockResolvedValue(vi.fn());

      const { result } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(result.current.isInitialized).toBe(true);
      });

      await act(async () => {
        await result.current.endCall('call_test-uuid');
      });

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_end_call', {
        callId: 'call_test-uuid',
      });
    });
  });

  describe('Media Controls', () => {
    it('should toggle video', async () => {
      mockInvoke.mockResolvedValue(undefined);
      mockListen.mockResolvedValue(vi.fn());

      const { result } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(result.current.isInitialized).toBe(true);
      });

      // Enable video
      await act(async () => {
        await result.current.setVideoEnabled('call_test-uuid', true);
      });

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_set_video_enabled', {
        callId: 'call_test-uuid',
        enabled: true,
      });

      // Disable video
      await act(async () => {
        await result.current.setVideoEnabled('call_test-uuid', false);
      });

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_set_video_enabled', {
        callId: 'call_test-uuid',
        enabled: false,
      });
    });

    it('should toggle audio', async () => {
      mockInvoke.mockResolvedValue(undefined);
      mockListen.mockResolvedValue(vi.fn());

      const { result } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(result.current.isInitialized).toBe(true);
      });

      // Enable audio
      await act(async () => {
        await result.current.setAudioEnabled('call_test-uuid', true);
      });

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_set_audio_enabled', {
        callId: 'call_test-uuid',
        enabled: true,
      });

      // Disable audio (mute)
      await act(async () => {
        await result.current.setAudioEnabled('call_test-uuid', false);
      });

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_set_audio_enabled', {
        callId: 'call_test-uuid',
        enabled: false,
      });
    });

    it('should start and stop screen sharing', async () => {
      mockInvoke.mockResolvedValue(undefined);
      mockListen.mockResolvedValue(vi.fn());

      const { result } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(result.current.isInitialized).toBe(true);
      });

      // Start screen share
      await act(async () => {
        await result.current.startScreenShare('call_test-uuid');
      });

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_start_screen_share', {
        callId: 'call_test-uuid',
      });

      // Stop screen share
      await act(async () => {
        await result.current.stopScreenShare('call_test-uuid');
      });

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_stop_screen_share', {
        callId: 'call_test-uuid',
      });
    });
  });

  describe('Device Management', () => {
    it('should load media devices', async () => {
      const mockDevices: MediaDevice[] = [
        {
          device_id: 'audio_input_1',
          label: 'Default Microphone',
          kind: 'audioinput',
        },
        {
          device_id: 'video_input_1',
          label: 'Built-in Camera',
          kind: 'videoinput',
        },
      ];

      mockInvoke.mockResolvedValueOnce(undefined); // initialize
      mockListen.mockResolvedValue(vi.fn());
      mockInvoke.mockResolvedValueOnce(mockDevices); // get devices

      const { result } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(result.current.isInitialized).toBe(true);
      });

      await act(async () => {
        await result.current.loadDevices();
      });

      expect(result.current.devices).toEqual(mockDevices);
      expect(mockInvoke).toHaveBeenCalledWith('webrtc_get_media_devices');
    });

    it('should handle device loading errors', async () => {
      mockInvoke.mockResolvedValueOnce(undefined); // initialize
      mockListen.mockResolvedValue(vi.fn());
      mockInvoke.mockRejectedValueOnce(new Error('Device access denied'));

      const { result } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(result.current.isInitialized).toBe(true);
      });

      await act(async () => {
        try {
          await result.current.loadDevices();
        } catch (error) {
          // Expected to throw
        }
      });

      expect(result.current.error).toBeTruthy();
    });
  });

  describe('Event Handling', () => {
    it('should handle incoming call events', async () => {
      let eventHandler: ((event: { payload: CallEvent }) => void) | undefined;

      mockInvoke.mockResolvedValue(undefined);
      mockListen.mockImplementation((eventName: string, handler: typeof eventHandler) => {
        eventHandler = handler;
        return Promise.resolve(vi.fn());
      });

      const { result } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(result.current.isInitialized).toBe(true);
      });

      // Simulate incoming call event
      const incomingCallEvent: CallEvent = {
        type: 'incoming-call' as CallEventType,
        call_id: 'call_test-uuid',
        timestamp: new Date().toISOString(),
        peer_four_words: 'ocean-forest-moon-star',
        constraints: {
          has_audio: true,
          has_video: true,
          has_screen_share: false,
        },
      };

      act(() => {
        if (eventHandler) {
          eventHandler({ payload: incomingCallEvent });
        }
      });

      await waitFor(() => {
        expect(result.current.incomingCall).toEqual(incomingCallEvent);
      });
    });

    it('should handle call accepted events', async () => {
      let eventHandler: ((event: { payload: CallEvent }) => void) | undefined;

      mockInvoke.mockResolvedValue(undefined);
      mockListen.mockImplementation((eventName: string, handler: typeof eventHandler) => {
        eventHandler = handler;
        return Promise.resolve(vi.fn());
      });

      const { result } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(result.current.isInitialized).toBe(true);
      });

      // Simulate call accepted event
      const callAcceptedEvent: CallEvent = {
        type: 'call-accepted' as CallEventType,
        call_id: 'call_test-uuid',
        timestamp: new Date().toISOString(),
      };

      act(() => {
        if (eventHandler) {
          eventHandler({ payload: callAcceptedEvent });
        }
      });

      await waitFor(() => {
        expect(result.current.currentCall).toBeTruthy();
      });
    });

    it('should handle call ended events', async () => {
      let eventHandler: ((event: { payload: CallEvent }) => void) | undefined;

      mockInvoke.mockResolvedValue(undefined);
      mockListen.mockImplementation((eventName: string, handler: typeof eventHandler) => {
        eventHandler = handler;
        return Promise.resolve(vi.fn());
      });

      const { result } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(result.current.isInitialized).toBe(true);
      });

      // Simulate call ended event
      const callEndedEvent: CallEvent = {
        type: 'call-ended' as CallEventType,
        call_id: 'call_test-uuid',
        timestamp: new Date().toISOString(),
      };

      act(() => {
        if (eventHandler) {
          eventHandler({ payload: callEndedEvent });
        }
      });

      await waitFor(() => {
        expect(result.current.currentCall).toBeNull();
      });
    });

    it('should handle media state change events', async () => {
      let eventHandler: ((event: { payload: CallEvent }) => void) | undefined;

      mockInvoke.mockResolvedValue(undefined);
      mockListen.mockImplementation((eventName: string, handler: typeof eventHandler) => {
        eventHandler = handler;
        return Promise.resolve(vi.fn());
      });

      const { result } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(result.current.isInitialized).toBe(true);
      });

      // Simulate video state changed event
      const videoStateEvent: CallEvent = {
        type: 'video-state-changed' as CallEventType,
        call_id: 'call_test-uuid',
        timestamp: new Date().toISOString(),
        enabled: true,
      };

      act(() => {
        if (eventHandler) {
          eventHandler({ payload: videoStateEvent });
        }
      });

      await waitFor(() => {
        expect(result.current.isVideoEnabled).toBe(true);
      });

      // Simulate audio state changed event
      const audioStateEvent: CallEvent = {
        type: 'audio-state-changed' as CallEventType,
        call_id: 'call_test-uuid',
        timestamp: new Date().toISOString(),
        enabled: false,
      };

      act(() => {
        if (eventHandler) {
          eventHandler({ payload: audioStateEvent });
        }
      });

      await waitFor(() => {
        expect(result.current.isAudioEnabled).toBe(false);
      });

      // Simulate screen share state changed event
      const screenShareEvent: CallEvent = {
        type: 'screen-share-state-changed' as CallEventType,
        call_id: 'call_test-uuid',
        timestamp: new Date().toISOString(),
        active: true,
      };

      act(() => {
        if (eventHandler) {
          eventHandler({ payload: screenShareEvent });
        }
      });

      await waitFor(() => {
        expect(result.current.isScreenSharing).toBe(true);
      });
    });
  });

  describe('Error Handling', () => {
    it('should handle call initiation errors', async () => {
      mockInvoke.mockResolvedValueOnce(undefined); // initialize
      mockListen.mockResolvedValue(vi.fn());
      mockInvoke.mockRejectedValueOnce(new Error('Network unavailable'));

      const { result } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(result.current.isInitialized).toBe(true);
      });

      await act(async () => {
        try {
          await result.current.initiateCall('ocean-forest-moon-star', {
            has_audio: true,
            has_video: false,
            has_screen_share: false,
          });
        } catch (error) {
          // Expected to throw
        }
      });

      expect(result.current.error).toBeTruthy();
    });

    it('should clear errors', async () => {
      mockInvoke.mockResolvedValue(undefined);
      mockListen.mockResolvedValue(vi.fn());

      const { result } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(result.current.isInitialized).toBe(true);
      });

      // Set error through failed operation
      mockInvoke.mockRejectedValueOnce(new Error('Test error'));
      await act(async () => {
        try {
          await result.current.loadDevices();
        } catch (error) {
          // Expected to throw
        }
      });

      expect(result.current.error).toBeTruthy();

      // Clear error
      act(() => {
        result.current.clearError();
      });

      expect(result.current.error).toBeNull();
    });
  });

  describe('Cleanup', () => {
    it('should cleanup on unmount', async () => {
      const mockUnsubscribe = vi.fn();
      mockInvoke.mockResolvedValue(undefined);
      mockListen.mockResolvedValue(mockUnsubscribe);

      const { unmount } = renderHook(() => useWebRTC());

      await waitFor(() => {
        expect(mockListen).toHaveBeenCalled();
      });

      unmount();

      expect(mockUnsubscribe).toHaveBeenCalled();
    });
  });
});
