// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

/**
 * WebRTC Service Types
 * 
 * TypeScript types for WebRTC voice, video, and screen sharing functionality.
 * These types mirror the Rust backend API defined in communitas-desktop/src/webrtc_commands.rs
 */

/**
 * Media device information
 */
export interface MediaDevice {
  /** Unique device identifier */
  device_id: string;
  /** Human-readable device name */
  label: string;
  /** Device type: 'audioinput' | 'audiooutput' | 'videoinput' */
  kind: 'audioinput' | 'audiooutput' | 'videoinput';
}

/**
 * Media constraints for a call
 */
export interface MediaConstraints {
  /** Enable audio in the call */
  has_audio: boolean;
  /** Enable video in the call */
  has_video: boolean;
  /** Enable screen sharing (added as separate track) */
  has_screen_share?: boolean;
}

/**
 * Call state
 */
export enum CallState {
  /** Call is being initiated (outgoing) */
  Initiating = 'initiating',
  /** Call is ringing (incoming) */
  Ringing = 'ringing',
  /** Call is active and connected */
  Active = 'active',
  /** Call has ended */
  Ended = 'ended',
  /** Call was rejected */
  Rejected = 'rejected',
}

/**
 * Call direction
 */
export enum CallDirection {
  /** Outgoing call (we initiated) */
  Outgoing = 'outgoing',
  /** Incoming call (peer initiated) */
  Incoming = 'incoming',
}

/**
 * Call information
 */
export interface CallInfo {
  /** Unique call identifier (UUID) */
  call_id: string;
  /** Four-word address of the peer */
  peer_four_words: string;
  /** Call direction */
  direction: CallDirection;
  /** Current call state */
  state: CallState;
  /** Media constraints for the call */
  constraints: MediaConstraints;
  /** Is video currently enabled */
  is_video_enabled: boolean;
  /** Is audio currently enabled (not muted) */
  is_audio_enabled: boolean;
  /** Is screen sharing active */
  is_screen_sharing: boolean;
  /** Call start time (ISO string) */
  started_at?: string;
  /** Call end time (ISO string) */
  ended_at?: string;
}

/**
 * Call event types
 */
export enum CallEventType {
  /** Call has been initiated */
  CallInitiated = 'call-initiated',
  /** Incoming call received */
  IncomingCall = 'incoming-call',
  /** Call has been accepted */
  CallAccepted = 'call-accepted',
  /** Call has been rejected */
  CallRejected = 'call-rejected',
  /** Call has ended */
  CallEnded = 'call-ended',
  /** Video state changed */
  VideoStateChanged = 'video-state-changed',
  /** Audio state changed */
  AudioStateChanged = 'audio-state-changed',
  /** Screen share state changed */
  ScreenShareStateChanged = 'screen-share-state-changed',
  /** Call error occurred */
  CallError = 'call-error',
}

/**
 * Base call event
 */
export interface BaseCallEvent {
  /** Event type */
  type: CallEventType;
  /** Call ID */
  call_id: string;
  /** Event timestamp (ISO string) */
  timestamp: string;
}

/**
 * Call initiated event
 */
export interface CallInitiatedEvent extends BaseCallEvent {
  type: CallEventType.CallInitiated;
  /** Target peer four-word address */
  peer_four_words: string;
  /** Media constraints */
  constraints: MediaConstraints;
}

/**
 * Incoming call event
 */
export interface IncomingCallEvent extends BaseCallEvent {
  type: CallEventType.IncomingCall;
  /** Caller's four-word address */
  peer_four_words: string;
  /** Media constraints requested by caller */
  constraints: MediaConstraints;
}

/**
 * Call accepted event
 */
export interface CallAcceptedEvent extends BaseCallEvent {
  type: CallEventType.CallAccepted;
}

/**
 * Call rejected event
 */
export interface CallRejectedEvent extends BaseCallEvent {
  type: CallEventType.CallRejected;
  /** Reason for rejection (optional) */
  reason?: string;
}

/**
 * Call ended event
 */
export interface CallEndedEvent extends BaseCallEvent {
  type: CallEventType.CallEnded;
  /** Reason for ending (optional) */
  reason?: string;
  /** Call duration in seconds */
  duration?: number;
}

/**
 * Video state changed event
 */
export interface VideoStateChangedEvent extends BaseCallEvent {
  type: CallEventType.VideoStateChanged;
  /** New video enabled state */
  enabled: boolean;
}

/**
 * Audio state changed event
 */
export interface AudioStateChangedEvent extends BaseCallEvent {
  type: CallEventType.AudioStateChanged;
  /** New audio enabled state (false = muted) */
  enabled: boolean;
}

/**
 * Screen share state changed event
 */
export interface ScreenShareStateChangedEvent extends BaseCallEvent {
  type: CallEventType.ScreenShareStateChanged;
  /** New screen share state */
  active: boolean;
}

/**
 * Call error event
 */
export interface CallErrorEvent extends BaseCallEvent {
  type: CallEventType.CallError;
  /** Error message */
  error: string;
  /** Error code (optional) */
  code?: string;
}

/**
 * Union type of all call events
 */
export type CallEvent =
  | CallInitiatedEvent
  | IncomingCallEvent
  | CallAcceptedEvent
  | CallRejectedEvent
  | CallEndedEvent
  | VideoStateChangedEvent
  | AudioStateChangedEvent
  | ScreenShareStateChangedEvent
  | CallErrorEvent;

/**
 * Call statistics
 */
export interface CallStatistics {
  /** Call ID */
  call_id: string;
  /** Total packets sent */
  packets_sent: number;
  /** Total packets received */
  packets_received: number;
  /** Total bytes sent */
  bytes_sent: number;
  /** Total bytes received */
  bytes_received: number;
  /** Packets lost */
  packets_lost: number;
  /** Round-trip time in milliseconds */
  rtt_ms: number;
  /** Jitter in milliseconds */
  jitter_ms: number;
  /** Bitrate in kbps */
  bitrate_kbps: number;
}

/**
 * WebRTC service configuration
 */
export interface WebRTCConfig {
  /** Enable automatic call answering for testing */
  auto_answer?: boolean;
  /** Default media constraints */
  default_constraints?: MediaConstraints;
  /** Enable debug logging */
  debug?: boolean;
}

/**
 * Type guard for call events
 */
export function isCallEvent(event: unknown): event is CallEvent {
  return (
    typeof event === 'object' &&
    event !== null &&
    'type' in event &&
    'call_id' in event &&
    'timestamp' in event
  );
}

/**
 * Type guard for incoming call events
 */
export function isIncomingCallEvent(event: CallEvent): event is IncomingCallEvent {
  return event.type === CallEventType.IncomingCall;
}

/**
 * Type guard for call ended events
 */
export function isCallEndedEvent(event: CallEvent): event is CallEndedEvent {
  return event.type === CallEventType.CallEnded;
}

/**
 * Create default media constraints
 */
export function createDefaultConstraints(): MediaConstraints {
  return {
    has_audio: true,
    has_video: true,
    has_screen_share: false,
  };
}

/**
 * Create audio-only constraints
 */
export function createAudioOnlyConstraints(): MediaConstraints {
  return {
    has_audio: true,
    has_video: false,
    has_screen_share: false,
  };
}

/**
 * Create video call constraints
 */
export function createVideoCallConstraints(): MediaConstraints {
  return {
    has_audio: true,
    has_video: true,
    has_screen_share: false,
  };
}
