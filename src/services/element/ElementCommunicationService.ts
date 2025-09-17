import { EventEmitter } from 'events';
import { Element } from '../../types/element';
import { ChatMessage, FileAttachment, MessageReaction } from '../../services/webrtc/WebRTCService';

export interface MessageData {
  content: string;
  attachments?: File[];
  threadId?: string;
}

export class ElementCommunicationService extends EventEmitter {
  private element: Element;
  private currentUserId: string;
  private messages: ChatMessage[] = [];
  private typingTimeouts = new Map<string, NodeJS.Timeout>();

  constructor(element: Element, currentUserId: string) {
    super();
    this.element = element;
    this.currentUserId = currentUserId;
  }

  async initialize(): Promise<void> {
    // Initialize communication channels
    this.emit('initialized');
  }

  async sendMessage(content: string, attachments?: File[]): Promise<void> {
    if (!this.element.capabilities.text) {
      throw new Error('Text messaging is not enabled for this element');
    }

    const message: ChatMessage = {
      id: this.generateId(),
      senderId: this.currentUserId,
      senderName: this.currentUserId, // TODO: Get actual display name
      content,
      timestamp: new Date(),
      type: 'text',
      attachments: attachments?.map(file => ({
        id: this.generateId(),
        name: file.name,
        size: file.size,
        type: file.type,
        url: URL.createObjectURL(file), // Temporary URL
      })),
    };

    this.messages.push(message);
    this.emit('message-sent', message);

    // Simulate receiving the message (for now)
    setTimeout(() => {
      this.emit('message-received', message);
    }, 100);
  }

  async sendVoiceMessage(audioBlob: Blob): Promise<void> {
    if (!this.element.capabilities.voice) {
      throw new Error('Voice messaging is not enabled for this element');
    }

    const attachment: FileAttachment = {
      id: this.generateId(),
      name: 'Voice Message',
      size: audioBlob.size,
      type: audioBlob.type,
      url: URL.createObjectURL(audioBlob),
    };

    const message: ChatMessage = {
      id: this.generateId(),
      senderId: this.currentUserId,
      senderName: this.currentUserId,
      content: '',
      timestamp: new Date(),
      type: 'file',
      attachments: [attachment],
    };

    this.messages.push(message);
    this.emit('message-sent', message);
    this.emit('message-received', message);
  }

  async addReaction(messageId: string, emoji: string): Promise<void> {
    const message = this.messages.find(m => m.id === messageId);
    if (!message) return;

    const reaction: MessageReaction = {
      emoji,
      userId: this.currentUserId,
      userName: this.currentUserId,
      timestamp: new Date(),
    };

    message.reactions = message.reactions || [];
    message.reactions.push(reaction);

    this.emit('reaction-added', { messageId, reaction });
  }

  async startTyping(): Promise<void> {
    this.emit('user-typing', this.currentUserId);

    // Clear existing timeout
    const existingTimeout = this.typingTimeouts.get(this.currentUserId);
    if (existingTimeout) {
      clearTimeout(existingTimeout);
    }

    // Set new timeout
    const timeout = setTimeout(() => {
      this.emit('user-stopped-typing', this.currentUserId);
      this.typingTimeouts.delete(this.currentUserId);
    }, 3000);

    this.typingTimeouts.set(this.currentUserId, timeout);
  }

  async stopTyping(): Promise<void> {
    const timeout = this.typingTimeouts.get(this.currentUserId);
    if (timeout) {
      clearTimeout(timeout);
      this.typingTimeouts.delete(this.currentUserId);
    }
    this.emit('user-stopped-typing', this.currentUserId);
  }

  getMessages(limit: number = 50, offset: number = 0): ChatMessage[] {
    return this.messages
      .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime())
      .slice(offset, offset + limit)
      .reverse();
  }

  getMessage(messageId: string): ChatMessage | null {
    return this.messages.find(m => m.id === messageId) || null;
  }

  searchMessages(query: string): ChatMessage[] {
    const lowercaseQuery = query.toLowerCase();
    return this.messages.filter(message =>
      message.content.toLowerCase().includes(lowercaseQuery) ||
      message.attachments?.some(att => att.name.toLowerCase().includes(lowercaseQuery))
    );
  }

  getUnreadCount(): number {
    // For now, return a mock count
    return Math.floor(Math.random() * 5);
  }

  markAsRead(messageId: string): void {
    // TODO: Implement read status tracking
    this.emit('messages-read', [messageId]);
  }

  async cleanup(): Promise<void> {
    // Clear all typing timeouts
    for (const timeout of this.typingTimeouts.values()) {
      clearTimeout(timeout);
    }
    this.typingTimeouts.clear();

    // Clear message attachments URLs
    for (const message of this.messages) {
      message.attachments?.forEach(att => {
        URL.revokeObjectURL(att.url);
      });
    }

    this.messages = [];
    this.removeAllListeners();
  }

  private generateId(): string {
    return `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}