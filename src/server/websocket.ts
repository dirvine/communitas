// WebSocket server for WebRTC signaling and real-time communication
import { WebSocketServer, WebSocket } from 'ws';
import { IncomingMessage } from 'http';

interface WebSocketMessage {
  type: 'offer' | 'answer' | 'ice-candidate' | 'call-ended' | 'join-room' | 'leave-room';
  entityId?: string;
  entityType?: string;
  sdp?: any;
  candidate?: any;
  userId?: string;
}

interface ConnectedClient {
  ws: WebSocket;
  userId: string;
  entityId?: string;
  entityType?: string;
}

export class CommunicationWebSocketServer {
  private wss: WebSocketServer;
  private clients: Map<string, ConnectedClient> = new Map();
  private rooms: Map<string, Set<string>> = new Map(); // entityId -> Set of userIds

  constructor(server: any) {
    // Create WebSocket server on /ws path to avoid conflicts with Vite HMR
    this.wss = new WebSocketServer({ 
      server,
      path: '/ws',
      verifyClient: (info: { origin: string; secure: boolean; req: IncomingMessage }) => {
        // Add basic verification if needed
        return true;
      }
    });

    this.wss.on('connection', this.handleConnection.bind(this));
    console.log('🔌 WebSocket server started on /ws');
  }

  private handleConnection(ws: WebSocket, req: IncomingMessage) {
    console.log('🔗 New WebSocket connection');
    
    const clientId = this.generateClientId();
    
    ws.on('message', (data) => {
      try {
        const message: WebSocketMessage = JSON.parse(data.toString());
        this.handleMessage(clientId, ws, message);
      } catch (error) {
        console.error('❌ Invalid WebSocket message:', error);
        ws.send(JSON.stringify({ type: 'error', message: 'Invalid message format' }));
      }
    });

    ws.on('close', () => {
      console.log('🔌 WebSocket connection closed');
      this.handleDisconnection(clientId);
    });

    ws.on('error', (error) => {
      console.error('❌ WebSocket error:', error);
    });

    // Send connection confirmation
    ws.send(JSON.stringify({ type: 'connected', clientId }));
  }

  private handleMessage(clientId: string, ws: WebSocket, message: WebSocketMessage) {
    console.log('📨 Received message:', message.type, 'from', clientId);

    switch (message.type) {
      case 'join-room':
        this.handleJoinRoom(clientId, ws, message);
        break;
      case 'leave-room':
        this.handleLeaveRoom(clientId, message);
        break;
      case 'offer':
      case 'answer':
      case 'ice-candidate':
        this.handleWebRTCSignaling(clientId, message);
        break;
      case 'call-ended':
        this.handleCallEnded(clientId, message);
        break;
      default:
        ws.send(JSON.stringify({ type: 'error', message: `Unknown message type: ${message.type}` }));
    }
  }

  private handleJoinRoom(clientId: string, ws: WebSocket, message: WebSocketMessage) {
    if (!message.entityId || !message.userId) {
      ws.send(JSON.stringify({ type: 'error', message: 'entityId and userId required' }));
      return;
    }

    // Store client info
    this.clients.set(clientId, {
      ws,
      userId: message.userId,
      entityId: message.entityId,
      entityType: message.entityType
    });

    // Add to room
    if (!this.rooms.has(message.entityId)) {
      this.rooms.set(message.entityId, new Set());
    }
    this.rooms.get(message.entityId)!.add(message.userId);

    console.log(`👥 User ${message.userId} joined room ${message.entityId}`);
    
    // Notify other users in the room
    this.broadcastToRoom(message.entityId, {
      type: 'user-joined',
      userId: message.userId,
      entityId: message.entityId
    }, [message.userId]);

    ws.send(JSON.stringify({ type: 'joined-room', entityId: message.entityId }));
  }

  private handleLeaveRoom(clientId: string, message: WebSocketMessage) {
    const client = this.clients.get(clientId);
    if (!client || !client.entityId) return;

    const room = this.rooms.get(client.entityId);
    if (room) {
      room.delete(client.userId);
      if (room.size === 0) {
        this.rooms.delete(client.entityId);
      }
    }

    // Notify other users
    this.broadcastToRoom(client.entityId, {
      type: 'user-left',
      userId: client.userId,
      entityId: client.entityId
    }, [client.userId]);

    console.log(`👋 User ${client.userId} left room ${client.entityId}`);
  }

  private handleWebRTCSignaling(clientId: string, message: WebSocketMessage) {
    const client = this.clients.get(clientId);
    if (!client || !client.entityId) {
      console.error('❌ Client not in room for signaling');
      return;
    }

    // Forward signaling message to all other users in the room
    this.broadcastToRoom(client.entityId, {
      ...message,
      fromUserId: client.userId
    }, [client.userId]);

    console.log(`📡 Forwarded ${message.type} from ${client.userId} to room ${client.entityId}`);
  }

  private handleCallEnded(clientId: string, message: WebSocketMessage) {
    const client = this.clients.get(clientId);
    if (!client || !client.entityId) return;

    // Notify all users in the room that the call ended
    this.broadcastToRoom(client.entityId, {
      type: 'call-ended',
      fromUserId: client.userId,
      entityId: client.entityId
    });

    console.log(`📞 Call ended by ${client.userId} in room ${client.entityId}`);
  }

  private handleDisconnection(clientId: string) {
    const client = this.clients.get(clientId);
    if (client && client.entityId) {
      this.handleLeaveRoom(clientId, { type: 'leave-room' });
    }
    this.clients.delete(clientId);
  }

  private broadcastToRoom(entityId: string, message: any, excludeUsers: string[] = []) {
    const room = this.rooms.get(entityId);
    if (!room) return;

    const messageStr = JSON.stringify(message);
    
    this.clients.forEach((client, clientId) => {
      if (client.entityId === entityId && 
          !excludeUsers.includes(client.userId) &&
          client.ws.readyState === WebSocket.OPEN) {
        client.ws.send(messageStr);
      }
    });
  }

  private generateClientId(): string {
    return `client_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  public getStats() {
    return {
      connectedClients: this.clients.size,
      activeRooms: this.rooms.size,
      totalUsersInRooms: Array.from(this.rooms.values()).reduce((sum, room) => sum + room.size, 0)
    };
  }
}

export default CommunicationWebSocketServer;