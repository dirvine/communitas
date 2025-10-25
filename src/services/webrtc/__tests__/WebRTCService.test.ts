import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { WebRTCService } from '../WebRTCService';
import type { MediaConstraints, CallEvent, CallEventType, MediaDevice } from '../types';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Get mocked functions (they're already mocked by vitest config)
const mockInvoke = invoke as ReturnType<typeof vi.fn>;
const mockListen = listen as ReturnType<typeof vi.fn>;

describe('WebRTCService', () => {
  let service: WebRTCService;

  beforeEach(() => {
    vi.clearAllMocks();
    service = new WebRTCService();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('Initialization', () => {
    it('should initialize successfully', () => {
      expect(service).toBeDefined();
      expect(service).toBeInstanceOf(WebRTCService);
    });

    it('should subscribe to events on initialization', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.initialize();

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_subscribe_events');
    });

    it('should handle initialization errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Backend not ready'));

      await expect(service.initialize()).rejects.toThrow('Backend not ready');
    });
  });

  describe('Call Lifecycle', () => {
    it('should initiate a call with valid parameters', async () => {
      const mockCallId = 'call_test-uuid-1234';
      mockInvoke.mockResolvedValueOnce(mockCallId);

      const constraints: MediaConstraints = {
        has_audio: true,
        has_video: true,
        has_screen_share: false,
      };

      const callId = await service.initiateCall('ocean-forest-moon-star', constraints);

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_initiate_call', {
        targetFourWords: 'ocean-forest-moon-star',
        hasAudio: true,
        hasVideo: true,
        hasScreenShare: false,
      });
      expect(callId).toBe(mockCallId);
    });

    it('should reject invalid four-word address', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid target identity'));

      const constraints: MediaConstraints = {
        has_audio: true,
        has_video: false,
        has_screen_share: false,
      };

      await expect(
        service.initiateCall('invalid-address', constraints)
      ).rejects.toThrow('Invalid target identity');
    });

    it('should require audio or video to be enabled', async () => {
      mockInvoke.mockRejectedValueOnce(
        new Error('Call must have at least audio or video enabled')
      );

      const constraints: MediaConstraints = {
        has_audio: false,
        has_video: false,
        has_screen_share: false,
      };

      await expect(
        service.initiateCall('ocean-forest-moon-star', constraints)
      ).rejects.toThrow('Call must have at least audio or video enabled');
    });

    it('should accept an incoming call', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.acceptCall('call_test-uuid');

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_accept_call', {
        callId: 'call_test-uuid',
      });
    });

    it('should reject invalid call ID on accept', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Invalid call ID'));

      await expect(service.acceptCall('invalid-id')).rejects.toThrow('Invalid call ID');
    });

    it('should reject a call', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.rejectCall('call_test-uuid');

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_reject_call', {
        callId: 'call_test-uuid',
      });
    });

    it('should end an active call', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.endCall('call_test-uuid');

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_end_call', {
        callId: 'call_test-uuid',
      });
    });

    it('should handle ending non-existent call', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Call not found'));

      await expect(service.endCall('non-existent-call')).rejects.toThrow('Call not found');
    });
  });

  describe('Media Controls', () => {
    it('should enable video', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.setVideoEnabled('call_test-uuid', true);

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_set_video_enabled', {
        callId: 'call_test-uuid',
        enabled: true,
      });
    });

    it('should disable video', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.setVideoEnabled('call_test-uuid', false);

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_set_video_enabled', {
        callId: 'call_test-uuid',
        enabled: false,
      });
    });

    it('should handle video control errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Call not found'));

      await expect(service.setVideoEnabled('non-existent', true)).rejects.toThrow(
        'Call not found'
      );
    });

    it('should enable audio', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.setAudioEnabled('call_test-uuid', true);

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_set_audio_enabled', {
        callId: 'call_test-uuid',
        enabled: true,
      });
    });

    it('should disable audio (mute)', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.setAudioEnabled('call_test-uuid', false);

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_set_audio_enabled', {
        callId: 'call_test-uuid',
        enabled: false,
      });
    });

    it('should handle audio control errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Call not found'));

      await expect(service.setAudioEnabled('non-existent', true)).rejects.toThrow(
        'Call not found'
      );
    });

    it('should start screen sharing', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.startScreenShare('call_test-uuid');

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_start_screen_share', {
        callId: 'call_test-uuid',
      });
    });

    it('should stop screen sharing', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.stopScreenShare('call_test-uuid');

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_stop_screen_share', {
        callId: 'call_test-uuid',
      });
    });

    it('should handle screen share errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Call not found'));

      await expect(service.startScreenShare('non-existent')).rejects.toThrow('Call not found');
    });
  });

  describe('Device Enumeration', () => {
    it('should get available media devices', async () => {
      const mockDevices: MediaDevice[] = [
        {
          device_id: 'audio_input_1',
          label: 'Default Microphone',
          kind: 'audioinput',
        },
        {
          device_id: 'audio_output_1',
          label: 'Default Speakers',
          kind: 'audiooutput',
        },
        {
          device_id: 'video_input_1',
          label: 'Built-in Camera',
          kind: 'videoinput',
        },
      ];

      mockInvoke.mockResolvedValueOnce(mockDevices);

      const devices = await service.getMediaDevices();

      expect(mockInvoke).toHaveBeenCalledWith('webrtc_get_media_devices');
      expect(devices).toEqual(mockDevices);
      expect(devices).toHaveLength(3);
    });

    it('should handle device enumeration errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Device access denied'));

      await expect(service.getMediaDevices()).rejects.toThrow('Device access denied');
    });
  });

  describe('Event Handling', () => {
    it('should subscribe to call events', async () => {
      const mockCallback = vi.fn();
      const mockUnsubscribe = vi.fn();

      mockListen.mockResolvedValueOnce(mockUnsubscribe);

      const unsubscribe = await service.subscribeToCallEvents(mockCallback);

      expect(mockListen).toHaveBeenCalled();
      expect(unsubscribe).toBe(mockUnsubscribe);
    });

    it('should handle incoming call events', async () => {
      const mockCallback = vi.fn();
      let eventHandler: ((event: { payload: CallEvent }) => void) | undefined;

      mockListen.mockImplementation((eventName: string, handler: typeof eventHandler) => {
        eventHandler = handler;
        return Promise.resolve(vi.fn());
      });

      await service.subscribeToCallEvents(mockCallback);

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

      if (eventHandler) {
        eventHandler({ payload: incomingCallEvent });
        expect(mockCallback).toHaveBeenCalledWith(incomingCallEvent);
      }
    });
  });

  describe('Call State Management', () => {
    it('should track active calls', async () => {
      const mockCallId = 'call_test-uuid-1234';
      mockInvoke.mockResolvedValueOnce(mockCallId);

      const constraints: MediaConstraints = {
        has_audio: true,
        has_video: true,
        has_screen_share: false,
      };

      const callId = await service.initiateCall('ocean-forest-moon-star', constraints);

      expect(callId).toBe(mockCallId);
    });

    it('should handle multiple concurrent calls', async () => {
      const mockCallId1 = 'call_test-uuid-1';
      const mockCallId2 = 'call_test-uuid-2';

      mockInvoke
        .mockResolvedValueOnce(mockCallId1)
        .mockResolvedValueOnce(mockCallId2);

      const constraints: MediaConstraints = {
        has_audio: true,
        has_video: true,
        has_screen_share: false,
      };

      const callId1 = await service.initiateCall('ocean-forest-moon-star', constraints);
      const callId2 = await service.initiateCall('river-mountain-sky-rain', constraints);

      expect(callId1).toBe(mockCallId1);
      expect(callId2).toBe(mockCallId2);
      expect(callId1).not.toBe(callId2);
    });
  });

  describe('Error Handling', () => {
    it('should handle network errors gracefully', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network unavailable'));

      const constraints: MediaConstraints = {
        has_audio: true,
        has_video: false,
        has_screen_share: false,
      };

      await expect(
        service.initiateCall('ocean-forest-moon-star', constraints)
      ).rejects.toThrow('Network unavailable');
    });

    it('should handle backend initialization errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('WebRTC service not initialized'));

      await expect(service.acceptCall('call_test-uuid')).rejects.toThrow(
        'WebRTC service not initialized'
      );
    });
  });
});
