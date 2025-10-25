// WebRTC and WebSocket service for audio/video calling
export class WebRTCService {
  private websocket: WebSocket | null = null;
  private localStream: MediaStream | null = null;
  private peerConnections: Map<string, RTCPeerConnection> = new Map();
  private screenTrack: MediaStreamTrack | null = null;
  private isInitialized = false;
  private listeners: Map<string, Function[]> = new Map();

  constructor() {
    this.initializeWebSocket();
  }

  private initializeWebSocket() {
    try {
      // Environment-aware WebSocket URL selection
      let wsUrl: string;
      
      // Detect Tauri environment
      const isTauri = typeof window !== 'undefined' && 
                      typeof (window as any).__TAURI__ !== 'undefined';
      
      if (isTauri) {
        // In Tauri, use a fixed backend URL since window.location.host is empty
        // This should be configurable in production, for now use localhost
        wsUrl = 'ws://localhost:3001/ws';
        console.log('🔌 Tauri environment detected, using fixed backend URL');
      } else {
        // In browser, use same host as current page for WebSocket connection
        const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
        wsUrl = `${protocol}//${window.location.host}/ws`;
        console.log('🔌 Browser environment detected, using proxy URL');
      }
      
      console.log('🔌 Connecting to WebSocket:', wsUrl);
      this.websocket = new WebSocket(wsUrl);
      
      this.websocket.onopen = () => {
        console.log('🔌 WebSocket connected for WebRTC');
        this.isInitialized = true;
        this.emit('connectionChanged', { connected: true });
      };

      this.websocket.onmessage = (event) => {
        this.handleWebSocketMessage(JSON.parse(event.data));
      };

      this.websocket.onclose = () => {
        console.log('🔌 WebSocket disconnected');
        this.isInitialized = false;
        this.emit('connectionChanged', { connected: false });
        // Reconnect after 3 seconds
        setTimeout(() => this.initializeWebSocket(), 3000);
      };

      this.websocket.onerror = (error) => {
        console.error('❌ WebSocket error:', error);
      };
    } catch (error) {
      console.error('Failed to initialize WebSocket:', error);
    }
  }

  private handleWebSocketMessage(message: any) {
    switch (message.type) {
      case 'offer':
        this.handleOffer(message);
        break;
      case 'answer':
        this.handleAnswer(message);
        break;
      case 'ice-candidate':
        this.handleIceCandidate(message);
        break;
      case 'call-ended':
        this.handleCallEnded(message);
        break;
      case 'audio-state-changed':
        this.handleAudioStateChanged(message);
        break;
      case 'video-state-changed':
        this.handleVideoStateChanged(message);
        break;
      case 'screen-share-started':
        this.handleScreenShareStarted(message);
        break;
      case 'screen-share-stopped':
        this.handleScreenShareStopped(message);
        break;
    }
  }

  async startAudioCall(entityId: string, entityType: string): Promise<void> {
    try {
      console.log(`🎵 Starting audio call for ${entityType} ${entityId}`);
      
      // First join the room
      await this.joinRoom(entityId, entityType);
      
      // Get user media (audio only)
      this.localStream = await navigator.mediaDevices.getUserMedia({ 
        audio: true, 
        video: false 
      });

      // Create peer connection
      const peerConnection = this.createPeerConnection(entityId);
      
      // Add local stream to peer connection
      this.localStream.getTracks().forEach(track => {
        if (this.localStream) {
          peerConnection.addTrack(track, this.localStream);
        }
      });

      // Create offer
      const offer = await peerConnection.createOffer();
      await peerConnection.setLocalDescription(offer);

      // Send offer through WebSocket
      this.sendMessage({
        type: 'offer',
        entityId,
        entityType,
        sdp: offer
      });

      // Show calling UI
      this.showCallUI(entityId, entityType, 'audio', 'outgoing');

    } catch (error) {
      console.error('Failed to start audio call:', error);
      alert('Failed to start audio call. Please check your microphone permissions.');
    }
  }

  async startVideoCall(entityId: string, entityType: string): Promise<void> {
    try {
      console.log(`📹 Starting video call for ${entityType} ${entityId}`);
      
      // First join the room
      await this.joinRoom(entityId, entityType);
      
      // Get user media (audio + video)
      this.localStream = await navigator.mediaDevices.getUserMedia({ 
        audio: true, 
        video: true 
      });

      // Create peer connection
      const peerConnection = this.createPeerConnection(entityId);
      
      // Add local stream to peer connection
      this.localStream.getTracks().forEach(track => {
        if (this.localStream) {
          peerConnection.addTrack(track, this.localStream);
        }
      });

      // Create offer
      const offer = await peerConnection.createOffer();
      await peerConnection.setLocalDescription(offer);

      // Send offer through WebSocket
      this.sendMessage({
        type: 'offer',
        entityId,
        entityType,
        sdp: offer
      });

      // Show calling UI
      this.showCallUI(entityId, entityType, 'video', 'outgoing');

    } catch (error) {
      console.error('Failed to start video call:', error);
      alert('Failed to start video call. Please check your camera and microphone permissions.');
    }
  }

  private createPeerConnection(entityId: string): RTCPeerConnection {
    const configuration = {
      iceServers: [
        { urls: 'stun:stun.l.google.com:19302' },
        { urls: 'stun:stun1.l.google.com:19302' }
      ]
    };

    const peerConnection = new RTCPeerConnection(configuration);

    peerConnection.onicecandidate = (event) => {
      if (event.candidate) {
        this.sendMessage({
          type: 'ice-candidate',
          entityId,
          candidate: event.candidate
        });
      }
    };

    peerConnection.ontrack = (event) => {
      console.log('📡 Received remote stream');
      // Display remote stream via UI event emitter
      this.emit('remoteStream', { entityId, stream: event.streams[0] });
    };

    peerConnection.onconnectionstatechange = () => {
      console.log(`🔗 Connection state: ${peerConnection.connectionState}`);
      if (peerConnection.connectionState === 'disconnected' || 
          peerConnection.connectionState === 'failed') {
        this.endCall(entityId);
      }
    };

    this.peerConnections.set(entityId, peerConnection);
    return peerConnection;
  }

  private async handleOffer(message: any) {
    try {
      const peerConnection = this.createPeerConnection(message.entityId);
      
      await peerConnection.setRemoteDescription(message.sdp);
      
      // Get user media for answering call
      this.localStream = await navigator.mediaDevices.getUserMedia({
        audio: true,
        video: message.sdp.sdp.includes('video')
      });

      // Add local stream
      this.localStream.getTracks().forEach(track => {
        if (this.localStream) {
          peerConnection.addTrack(track, this.localStream);
        }
      });

      // Create answer
      const answer = await peerConnection.createAnswer();
      await peerConnection.setLocalDescription(answer);

      // Send answer
      this.sendMessage({
        type: 'answer',
        entityId: message.entityId,
        sdp: answer
      });

      // TODO: Show incoming call UI
      this.showCallUI(message.entityId, message.entityType, 
                     message.sdp.sdp.includes('video') ? 'video' : 'audio', 
                     'incoming');

    } catch (error) {
      console.error('Failed to handle offer:', error);
    }
  }

  private async handleAnswer(message: any) {
    const peerConnection = this.peerConnections.get(message.entityId);
    if (peerConnection) {
      await peerConnection.setRemoteDescription(message.sdp);
    }
  }

  private async handleIceCandidate(message: any) {
    const peerConnection = this.peerConnections.get(message.entityId);
    if (peerConnection) {
      await peerConnection.addIceCandidate(message.candidate);
    }
  }

  private handleCallEnded(message: any) {
    this.endCall(message.entityId);
  }

  private handleAudioStateChanged(message: any) {
    console.log(`Remote audio state changed for ${message.entityId}: ${message.enabled ? 'enabled' : 'disabled'}`);
    this.emit('remoteAudioStateChanged', {
      entityId: message.entityId,
      enabled: message.enabled
    });
  }

  private handleVideoStateChanged(message: any) {
    console.log(`Remote video state changed for ${message.entityId}: ${message.enabled ? 'enabled' : 'disabled'}`);
    this.emit('remoteVideoStateChanged', {
      entityId: message.entityId,
      enabled: message.enabled
    });
  }

  private handleScreenShareStarted(message: any) {
    console.log(`Remote screen share started for ${message.entityId}`);
    this.emit('remoteScreenShareStarted', {
      entityId: message.entityId
    });
  }

  private handleScreenShareStopped(message: any) {
    console.log(`Remote screen share stopped for ${message.entityId}`);
    this.emit('remoteScreenShareStopped', {
      entityId: message.entityId
    });
  }

  endCall(entityId: string) {
    console.log(`📞 Ending call for ${entityId}`);
    
    // Clean up peer connection
    const peerConnection = this.peerConnections.get(entityId);
    if (peerConnection) {
      peerConnection.close();
      this.peerConnections.delete(entityId);
    }

    // Stop local stream
    if (this.localStream) {
      this.localStream.getTracks().forEach(track => track.stop());
      this.localStream = null;
    }

    // Send end call message
    this.sendMessage({
      type: 'call-ended',
      entityId
    });

    // TODO: Hide call UI
    this.hideCallUI(entityId);
  }

  private sendMessage(message: any) {
    if (this.websocket?.readyState === WebSocket.OPEN) {
      this.websocket.send(JSON.stringify(message));
    } else {
      console.warn('WebSocket not connected, cannot send message');
    }
  }

  private showCallUI(entityId: string, entityType: string, callType: 'audio' | 'video', direction: 'incoming' | 'outgoing') {
    console.log(`📱 Showing ${direction} ${callType} call UI for ${entityType} ${entityId}`);

    // Emit event to show call UI through CallManager
    this.emit('showCallUI', {
      entityId,
      entityType,
      callType,
      direction
    });

    // For incoming calls, also emit incoming call event
    if (direction === 'incoming') {
      this.emit('incomingCall', {
        entityId,
        entityType,
        callType
      });
    }

    // Fallback UI until CallManager is properly mounted
    // This ensures users get some feedback even if the new UI isn't wired in
    if (typeof window !== 'undefined') {
      if (direction === 'outgoing') {
        alert(`${callType === 'video' ? '📹' : '🎵'} Calling ${entityType} ${entityId}...`);
      } else {
        const accept = confirm(`${callType === 'video' ? '📹' : '🎵'} Incoming ${callType} call from ${entityType} ${entityId}. Accept?`);
        if (!accept) {
          this.endCall(entityId);
        }
      }
    }
  }

  private displayRemoteStream(stream: MediaStream, entityId: string) {
    console.log(`📺 Displaying remote stream for ${entityId}`, stream);

    // Emit event to update remote stream in CallManager
    this.emit('remoteStream', {
      entityId,
      stream
    });
  }

  private hideCallUI(entityId: string) {
    console.log(`📱 Hiding call UI for ${entityId}`);

    // Emit event to hide call UI through CallManager
    this.emit('hideCallUI', { entityId });
  }

  private async joinRoom(entityId: string, entityType: string): Promise<void> {
    return new Promise((resolve, reject) => {
      if (!this.isConnected()) {
        reject(new Error('WebSocket not connected'));
        return;
      }

      // Generate a user ID (in real app, this would come from auth)
      const userId = `user_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

      // Listen for join confirmation
      const handleMessage = (event: MessageEvent) => {
        try {
          const message = JSON.parse(event.data);
          if (message.type === 'joined-room' && message.entityId === entityId) {
            this.websocket?.removeEventListener('message', handleMessage);
            console.log(`✅ Joined room ${entityId}`);
            resolve();
          }
        } catch (error) {
          console.error('Error parsing join response:', error);
        }
      };

      this.websocket?.addEventListener('message', handleMessage);

      // Send join room message
      this.sendMessage({
        type: 'join-room',
        entityId,
        entityType,
        userId
      });

      // Timeout after 5 seconds
      setTimeout(() => {
        this.websocket?.removeEventListener('message', handleMessage);
        reject(new Error('Join room timeout'));
      }, 5000);
    });
  }

  // Event emitter methods for UI integration
  on(event: string, callback: Function): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event)!.push(callback);
  }

  off(event: string, callback: Function): void {
    const callbacks = this.listeners.get(event);
    if (callbacks) {
      const index = callbacks.indexOf(callback);
      if (index > -1) {
        callbacks.splice(index, 1);
      }
    }
  }

  private emit(event: string, data?: any): void {
    const callbacks = this.listeners.get(event) || [];
    callbacks.forEach(callback => {
      try {
        callback(data);
      } catch (error) {
        console.error(`Error in WebRTC event callback for ${event}:`, error);
      }
    });
  }

  // Toggle audio mute/unmute
  toggleAudio(entityId: string): boolean {
    if (this.localStream) {
      const audioTracks = this.localStream.getAudioTracks();
      if (audioTracks.length > 0) {
        const enabled = !audioTracks[0].enabled;
        audioTracks[0].enabled = enabled;

        // Send audio state change to remote peer
        this.sendMessage({
          type: 'audio-state-changed',
          entityId,
          enabled
        });

        return enabled;
      }
    }
    return false;
  }

  // Toggle video on/off
  toggleVideo(entityId: string): boolean {
    if (this.localStream) {
      const videoTracks = this.localStream.getVideoTracks();
      if (videoTracks.length > 0) {
        const enabled = !videoTracks[0].enabled;
        videoTracks[0].enabled = enabled;

        // Send video state change to remote peer
        this.sendMessage({
          type: 'video-state-changed',
          entityId,
          enabled
        });

        return enabled;
      }
    }
    return false;
  }

  // Start screen sharing
  async startScreenShare(entityId: string): Promise<boolean> {
    try {
      // Get screen sharing stream
      const screenStream = await navigator.mediaDevices.getDisplayMedia({
        video: true,
        audio: false // Screen sharing typically doesn't include audio
      });

      // Replace video track in existing peer connection
      const peerConnection = this.peerConnections.get(entityId);
      if (peerConnection && this.localStream) {
        const screenVideoTrack = screenStream.getVideoTracks()[0];
        const sender = peerConnection.getSenders().find(s =>
          s.track && s.track.kind === 'video'
        );

        if (sender && screenVideoTrack) {
          await sender.replaceTrack(screenVideoTrack);

          // Store screen track for cleanup
          this.screenTrack = screenVideoTrack;

          // Listen for when user stops sharing via browser UI
          screenVideoTrack.onended = () => {
            this.stopScreenShare(entityId);
          };

          // Send screen share state change
          this.sendMessage({
            type: 'screen-share-started',
            entityId
          });

          return true;
        }
      }
      return false;
    } catch (error) {
      console.error('Failed to start screen share:', error);
      return false;
    }
  }

  // Stop screen sharing
  async stopScreenShare(entityId: string): Promise<boolean> {
    try {
      if (this.screenTrack) {
        this.screenTrack.stop();
        this.screenTrack = null;
      }

      // Restore camera video if we have one
      const peerConnection = this.peerConnections.get(entityId);
      if (peerConnection && this.localStream) {
        const cameraVideoTrack = this.localStream.getVideoTracks()[0];
        const sender = peerConnection.getSenders().find(s =>
          s.track && s.track.kind === 'video'
        );

        if (sender && cameraVideoTrack) {
          await sender.replaceTrack(cameraVideoTrack);
        }
      }

      // Send screen share state change
      this.sendMessage({
        type: 'screen-share-stopped',
        entityId
      });

      return true;
    } catch (error) {
      console.error('Failed to stop screen share:', error);
      return false;
    }
  }

  // Get current audio/video state
  getMediaState(): { audioEnabled: boolean; videoEnabled: boolean; screenSharing: boolean } {
    let audioEnabled = false;
    let videoEnabled = false;
    let screenSharing = false;

    if (this.localStream) {
      const audioTracks = this.localStream.getAudioTracks();
      const videoTracks = this.localStream.getVideoTracks();

      audioEnabled = audioTracks.length > 0 && audioTracks[0].enabled;
      videoEnabled = videoTracks.length > 0 && videoTracks[0].enabled;
    }

    screenSharing = this.screenTrack !== null;

    return { audioEnabled, videoEnabled, screenSharing };
  }

  isConnected(): boolean {
    return this.isInitialized && this.websocket?.readyState === WebSocket.OPEN;
  }

  // Destroy service and cleanup resources
  destroy(): void {
    // Close all peer connections
    this.peerConnections.forEach(pc => pc.close());
    this.peerConnections.clear();

    // Close WebSocket
    if (this.websocket) {
      this.websocket.close();
      this.websocket = null;
    }

    // Stop local stream
    if (this.localStream) {
      this.localStream.getTracks().forEach(track => track.stop());
      this.localStream = null;
    }

    // Clear listeners
    this.listeners.clear();
    this.isInitialized = false;
  }
}

// Create singleton instance
export const webRTCService = new WebRTCService();
export default webRTCService;