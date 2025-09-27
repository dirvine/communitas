import { Message } from '../components/chat/EntityChatView';

const storageKey = (entityType: string, entityId: string) =>
  `communitas-messages:${entityType}:${entityId}`;

const dispatchUpdate = (entityType: string, entityId: string) => {
  if (typeof window !== 'undefined') {
    window.dispatchEvent(
      new CustomEvent('messages:updated', { detail: { entityType, entityId } })
    );
  }
};

export const loadMessages = (entityType: string, entityId: string): Message[] => {
  if (typeof window === 'undefined') return [];
  const raw = window.localStorage.getItem(storageKey(entityType, entityId));
  if (!raw) return [];
  try {
    const parsed: Message[] = JSON.parse(raw);
    return parsed.map(message => ({
      ...message,
      timestamp: message.timestamp,
    }));
  } catch (error) {
    console.warn('Failed to load stored messages:', error);
    return [];
  }
};

export const saveMessages = (entityType: string, entityId: string, messages: Message[]) => {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.setItem(storageKey(entityType, entityId), JSON.stringify(messages));
    dispatchUpdate(entityType, entityId);
  } catch (error) {
    console.warn('Failed to persist messages:', error);
  }
};

export const upsertMessage = (entityType: string, entityId: string, message: Message) => {
  const existing = loadMessages(entityType, entityId);
  const index = existing.findIndex(m => m.id === message.id);
  const updated = index >= 0
    ? [...existing.slice(0, index), message, ...existing.slice(index + 1)]
    : [...existing, message];
  saveMessages(entityType, entityId, updated);
};

export const markMessageStatus = (
  entityType: string,
  entityId: string,
  messageId: string,
  status: Message['status'],
) => {
  const existing = loadMessages(entityType, entityId);
  const updated = existing.map(message =>
    message.id === messageId ? { ...message, status } : message
  );
  saveMessages(entityType, entityId, updated);
};

export const removeMessage = (entityType: string, entityId: string, messageId: string) => {
  const existing = loadMessages(entityType, entityId);
  const updated = existing.filter(message => message.id !== messageId);
  saveMessages(entityType, entityId, updated);
};
