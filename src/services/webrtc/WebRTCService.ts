// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

/**
 * WebRTC Service
 *
 * High-level service for managing WebRTC voice, video, and screen sharing calls.
 * Provides a TypeScript-friendly API over Tauri commands that connect to the
 * Rust backend with saorsa-webrtc and gossip overlay network integration.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { MediaConstraints, MediaDevice, CallEvent } from './types';

/**
 * WebRTC service for managing calls
 */
export class WebRTCService {
  private initialized = false;

  constructor() {
    // Service is ready to use
  }

  /**
   * Initialize the WebRTC service
   * Subscribes to call events from the backend
   */
  async initialize(): Promise<void> {
    try {
      await invoke('webrtc_subscribe_events');
      this.initialized = true;
    } catch (error) {
      throw new Error(`Failed to initialize WebRTC service: ${error}`);
    }
  }

  /**
   * Check if service is initialized
   */
  isInitialized(): boolean {
    return this.initialized;
  }

  /**
   * Initiate a call to another peer
   *
   * @param targetFourWords - Four-word address of the peer to call
   * @param constraints - Media constraints (audio, video, screen share)
   * @returns Call ID as a string
   */
  async initiateCall(
    targetFourWords: string,
    constraints: MediaConstraints
  ): Promise<string> {
    try {
      const callId = await invoke<string>('webrtc_initiate_call', {
        targetFourWords,
        hasAudio: constraints.has_audio,
        hasVideo: constraints.has_video,
        hasScreenShare: constraints.has_screen_share || false,
      });
      return callId;
    } catch (error) {
      throw new Error(`Failed to initiate call: ${error}`);
    }
  }

  /**
   * Accept an incoming call
   *
   * @param callId - ID of the call to accept
   */
  async acceptCall(callId: string): Promise<void> {
    try {
      await invoke('webrtc_accept_call', { callId });
    } catch (error) {
      throw new Error(`Failed to accept call: ${error}`);
    }
  }

  /**
   * Reject an incoming call
   *
   * @param callId - ID of the call to reject
   */
  async rejectCall(callId: string): Promise<void> {
    try {
      await invoke('webrtc_reject_call', { callId });
    } catch (error) {
      throw new Error(`Failed to reject call: ${error}`);
    }
  }

  /**
   * End an active call
   *
   * @param callId - ID of the call to end
   */
  async endCall(callId: string): Promise<void> {
    try {
      await invoke('webrtc_end_call', { callId });
    } catch (error) {
      throw new Error(`Failed to end call: ${error}`);
    }
  }

  /**
   * Enable or disable video in an active call
   *
   * @param callId - ID of the call
   * @param enabled - Whether to enable or disable video
   */
  async setVideoEnabled(callId: string, enabled: boolean): Promise<void> {
    try {
      await invoke('webrtc_set_video_enabled', { callId, enabled });
    } catch (error) {
      throw new Error(`Failed to set video enabled: ${error}`);
    }
  }

  /**
   * Enable or disable audio in an active call
   *
   * @param callId - ID of the call
   * @param enabled - Whether to enable or disable audio (mute/unmute)
   */
  async setAudioEnabled(callId: string, enabled: boolean): Promise<void> {
    try {
      await invoke('webrtc_set_audio_enabled', { callId, enabled });
    } catch (error) {
      throw new Error(`Failed to set audio enabled: ${error}`);
    }
  }

  /**
   * Start screen sharing in an active call
   *
   * @param callId - ID of the call
   */
  async startScreenShare(callId: string): Promise<void> {
    try {
      await invoke('webrtc_start_screen_share', { callId });
    } catch (error) {
      throw new Error(`Failed to start screen share: ${error}`);
    }
  }

  /**
   * Stop screen sharing in an active call
   *
   * @param callId - ID of the call
   */
  async stopScreenShare(callId: string): Promise<void> {
    try {
      await invoke('webrtc_stop_screen_share', { callId });
    } catch (error) {
      throw new Error(`Failed to stop screen share: ${error}`);
    }
  }

  /**
   * Get available media devices
   *
   * @returns List of available audio and video devices
   */
  async getMediaDevices(): Promise<MediaDevice[]> {
    try {
      const devices = await invoke<MediaDevice[]>('webrtc_get_media_devices');
      return devices;
    } catch (error) {
      throw new Error(`Failed to get media devices: ${error}`);
    }
  }

  /**
   * Subscribe to call events
   *
   * @param callback - Function to call when a call event occurs
   * @returns Unsubscribe function
   */
  async subscribeToCallEvents(
    callback: (event: CallEvent) => void
  ): Promise<UnlistenFn> {
    try {
      // Subscribe to all WebRTC event types
      const unsubscribe = await listen<CallEvent>('webrtc:call-event', (event) => {
        callback(event.payload);
      });

      return unsubscribe;
    } catch (error) {
      throw new Error(`Failed to subscribe to call events: ${error}`);
    }
  }
}

/**
 * Singleton instance of WebRTC service
 */
export const webrtcService = new WebRTCService();
