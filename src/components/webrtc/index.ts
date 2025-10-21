export { WebRTCService, webrtcService } from '../../services/webrtc/WebRTCService';
// Removed: SimpleCallInterface - using modern shell instead
export { SimpleCommunicationHub } from './SimpleCommunicationHub';

// Export WebRTC types from types.ts
export type {
  CallState,
  CallInfo,
  CallEvent,
  MediaDevice,
  MediaConstraints,
  CallDirection,
  CallEventType
} from '../../services/webrtc/types';

