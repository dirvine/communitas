const rtcConfiguration: RTCConfiguration = {
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'stun:stun1.l.google.com:19302' },
    { urls: 'stun:stun2.l.google.com:19302' },
  ]
};

export interface CallParticipant {
  id: string;
  fourWordAddress: string;
  displayName: string;
  isAudioEnabled: boolean;
  isVideoEnabled: boolean;
  isScreenSharing: boolean;
  connectionQuality: 'excellent' | 'good' | 'fair' | 'poor';
}

export interface CallState {
  id: string;
  participants: CallParticipant[];
  isActive: boolean;
  startTime: Date;
  duration: number;
  type: 'audio' | 'video' | 'screen-share';
  isGroupCall: boolean;
}

export interface MediaDevices {
  audioInputs: MediaDeviceInfo[];
  audioOutputs: MediaDeviceInfo[];
  videoInputs: MediaDeviceInfo[];
}

export interface ChatMessage {
  id: string;
  senderId: string;
  senderName: string;
  content: string;
  timestamp: Date;
  type: 'text' | 'file' | 'emoji-reaction';
  threadId?: string;
  attachments?: FileAttachment[];
  reactions?: MessageReaction[];
}

export interface FileAttachment {
  id: string;
  name: string;
  size: number;
  type: string;
  url: string;
  progress?: number;
}

export interface MessageReaction {
  emoji: string;
  userId: string;
  userName: string;
  timestamp: Date;
}

export interface ScreenShareOptions {
  includeAudio: boolean;
  cursor: 'always' | 'motion' | 'never';
  displaySurface: 'monitor' | 'window' | 'application';
}

type EventHandler = (...args: any[]) => void;

export class WebRTCService {
  private peerConnections: Map<string, RTCPeerConnection> = new Map();
  private localStream: MediaStream | null = null;
  private screenStream: MediaStream | null = null;
  private currentCall: CallState | null = null;
  private mediaDevices: MediaDevices | null = null;
  private isInitialized: boolean = false;
  private chatHistory: ChatMessage[] = [];
  private eventHandlers: Map<string, EventHandler[]> = new Map();

  constructor() {
    this.initialize();
  }

  private async initialize(): Promise<void> {
    try {
      await this.updateMediaDevices();
      this.setupEventListeners();
      this.isInitialized = true;
      this.emit('initialized');
    } catch (error) {
      console.error('Failed to initialize WebRTC service:', error);
      this.emit('error', error);
    }
  }

  private setupEventListeners(): void {
    navigator.mediaDevices.addEventListener('devicechange', () => {
      this.updateMediaDevices();
    });
  }

  private async updateMediaDevices(): Promise<void> {
    try {
      const devices = await navigator.mediaDevices.enumerateDevices();
      
      this.mediaDevices = {
        audioInputs: devices.filter(device => device.kind === 'audioinput'),
        audioOutputs: devices.filter(device => device.kind === 'audiooutput'),
        videoInputs: devices.filter(device => device.kind === 'videoinput')
      };
      
      this.emit('devicesUpdated', this.mediaDevices);
    } catch (error) {
      console.error('Failed to enumerate media devices:', error);
    }
  }

  async initiateCall(participantId: string, type: 'audio' | 'video'): Promise<void> {
    try {
      const stream = await this.getUserMedia(type === 'video');
      this.localStream = stream;
      
      const peerConnection = this.createPeerConnection(participantId);
      stream.getTracks().forEach(track => {
        peerConnection.addTrack(track, stream);
      });
      
      const offer = await peerConnection.createOffer({
        offerToReceiveAudio: true,
        offerToReceiveVideo: type === 'video'
      });
      
      await peerConnection.setLocalDescription(offer);
      
      await this.sendSignalingMessage(participantId, {
        type: 'offer',
        sdp: offer,
        callType: type
      });
      
      const callId = 'call_' + Date.now().toString();
      this.currentCall = {
        id: callId,
        participants: [
          {
            id: 'local',
            fourWordAddress: 'self',
            displayName: 'You',
            isAudioEnabled: true,
            isVideoEnabled: type === 'video',
            isScreenSharing: false,
            connectionQuality: 'excellent'
          }
        ],
        isActive: true,
        startTime: new Date(),
        duration: 0,
        type,
        isGroupCall: false
      };
      
      this.emit('callInitiated', this.currentCall);
    } catch (error) {
      console.error('Failed to initiate call:', error);
      this.emit('error', error);
    }
  }

  async answerCall(callId: string, accept: boolean): Promise<void> {
    if (!accept) {
      this.emit('callDeclined', callId);
      return;
    }
    
    try {
      this.emit('callAnswered', callId);
    } catch (error) {
      console.error('Failed to answer call:', error);
      this.emit('error', error);
    }
  }

  async endCall(): Promise<void> {
    if (!this.currentCall) return;
    
    try {
      this.peerConnections.forEach(pc => {
        pc.close();
      });
      this.peerConnections.clear();
      
      if (this.localStream) {
        this.localStream.getTracks().forEach(track => track.stop());
        this.localStream = null;
      }
      
      if (this.screenStream) {
        this.screenStream.getTracks().forEach(track => track.stop());
        this.screenStream = null;
      }
      
      const endedCall = this.currentCall;
      this.currentCall = null;
      
      this.emit('callEnded', endedCall);
    } catch (error) {
      console.error('Failed to end call:', error);
      this.emit('error', error);
    }
  }

  toggleAudio(): void {
    if (this.localStream) {
      const audioTrack = this.localStream.getAudioTracks()[0];
      if (audioTrack) {
        audioTrack.enabled = !audioTrack.enabled;
        this.emit('audioToggled', audioTrack.enabled);
      }
    }
  }

  toggleVideo(): void {
    if (this.localStream) {
      const videoTrack = this.localStream.getVideoTracks()[0];
      if (videoTrack) {
        videoTrack.enabled = !videoTrack.enabled;
        this.emit('videoToggled', videoTrack.enabled);
      }
    }
  }

  sendMessage(content: string): void {
    const messageId = 'msg_' + Date.now().toString();
    const message: ChatMessage = {
      id: messageId,
      senderId: 'local',
      senderName: 'You',
      content,
      timestamp: new Date(),
      type: 'text'
    };
    
    this.chatHistory.push(message);
    this.emit('messageAdded', message);
  }

  sendFile(file: File): void {
    if (file.size > 100 * 1024 * 1024) {
      this.emit('error', new Error('File size exceeds 100MB limit'));
      return;
    }
    
    const attachment: FileAttachment = {
      id: 'file_' + Date.now().toString(),
      name: file.name,
      size: file.size,
      type: file.type,
      url: URL.createObjectURL(file),
      progress: 0
    };
    
    const message: ChatMessage = {
      id: 'msg_' + Date.now().toString(),
      senderId: 'local',
      senderName: 'You',
      content: 'Sent file: ' + file.name,
      timestamp: new Date(),
      type: 'file',
      attachments: [attachment]
    };
    
    this.chatHistory.push(message);
    this.emit('messageAdded', message);
  }

  // Event emitter methods
  on(event: string, handler: EventHandler): void {
    if (!this.eventHandlers.has(event)) {
      this.eventHandlers.set(event, []);
    }
    this.eventHandlers.get(event)!.push(handler);
  }

  off(event: string, handler: EventHandler): void {
    const handlers = this.eventHandlers.get(event);
    if (handlers) {
      const index = handlers.indexOf(handler);
      if (index > -1) {
        handlers.splice(index, 1);
      }
    }
  }

  private emit(event: string, ...args: any[]): void {
    const handlers = this.eventHandlers.get(event);
    if (handlers) {
      handlers.forEach(handler => {
        try {
          handler(...args);
        } catch (error) {
          console.error('Error in event handler:', error);
        }
      });
    }
  }

  private createPeerConnection(participantId: string): RTCPeerConnection {
    const pc = new RTCPeerConnection(rtcConfiguration);
    
    pc.onicecandidate = (event) => {
      if (event.candidate) {
        this.sendSignalingMessage(participantId, {
          type: 'ice-candidate',
          candidate: event.candidate
        });
      }
    };
    
    pc.ontrack = (event) => {
      this.emit('remoteStreamAdded', {
        participantId,
        stream: event.streams[0]
      });
    };
    
    this.peerConnections.set(participantId, pc);
    return pc;
  }

  private async getUserMedia(video: boolean): Promise<MediaStream> {
    return await navigator.mediaDevices.getUserMedia({
      audio: {
        echoCancellation: true,
        noiseSuppression: true,
        autoGainControl: true
      },
      video: video ? {
        width: { ideal: 1280 },
        height: { ideal: 720 },
        frameRate: { ideal: 30 }
      } : false
    });
  }

  private async sendSignalingMessage(participantId: string, message: any): Promise<void> {
    // For Phase 1: Use localStorage as a simple signaling mechanism
    // In production, this would connect to a real signaling server
    const signalingKey = `webrtc_signal_${participantId}`;
    const signalingData = {
      from: 'local',
      to: participantId,
      message,
      timestamp: Date.now()
    };

    // Store in localStorage for demo purposes
    localStorage.setItem(signalingKey, JSON.stringify(signalingData));

    // Emit for UI updates
    this.emit('signalingMessage', { participantId, message });

    // Simulate receiving answer after a short delay (for demo)
    if (message.type === 'offer') {
      setTimeout(() => {
        this.handleIncomingAnswer(participantId, message);
      }, 1000);
    }
  }

  private async handleIncomingAnswer(participantId: string, offer: any): Promise<void> {
    try {
      // Create answer for demo purposes
      const peerConnection = this.createPeerConnection(participantId);

      // Add local stream if available
      if (this.localStream) {
        this.localStream.getTracks().forEach(track => {
          peerConnection.addTrack(track, this.localStream!);
        });
      }

      // Set remote description (simulated)
      await peerConnection.setRemoteDescription(new RTCSessionDescription({
        type: 'offer',
        sdp: offer.sdp
      }));

      // Create and set local description
      const answer = await peerConnection.createAnswer();
      await peerConnection.setLocalDescription(answer);

      // Simulate sending answer back
      this.emit('signalingMessage', {
        participantId,
        message: { type: 'answer', sdp: answer }
      });

      // Add remote participant to call
      if (this.currentCall) {
        this.currentCall.participants.push({
          id: participantId,
          fourWordAddress: participantId,
          displayName: participantId,
          isAudioEnabled: true,
          isVideoEnabled: offer.callType === 'video',
          isScreenSharing: false,
          connectionQuality: 'good'
        });

        this.emit('callUpdated', this.currentCall);
      }

    } catch (error) {
      console.error('Failed to handle incoming answer:', error);
      this.emit('error', error);
    }
  }

  get isReady(): boolean {
    return this.isInitialized;
  }

  get currentCallState(): CallState | null {
    return this.currentCall;
  }

  get availableDevices(): MediaDevices | null {
    return this.mediaDevices;
  }

  get messages(): ChatMessage[] {
    return this.chatHistory;
  }
}

export const webRTCService = new WebRTCService();

