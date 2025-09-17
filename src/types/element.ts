// Element System Types
// Defines the unified architecture for all collaboration entities

export type ElementType =
  | 'personal-individual'
  | 'personal-group'
  | 'organization-individual'
  | 'organization-group'
  | 'organization-project'
  | 'organization-channel';

export type ElementScope = 'personal' | 'organization';

export interface ElementIdentity {
  id: string;
  type: ElementType;
  scope: ElementScope;
  name: string;
  description?: string;
  avatar?: string;
  fourWords: string;
  publicKey: string;
  dhtAddress: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface ElementCapabilities {
  text: boolean;
  voice: boolean;
  video: boolean;
  screenShare: boolean;
  storage: boolean;
  webDrive: boolean;
  fileSharing: boolean;
  collaborativeEditing: boolean;
}

export interface ElementMembership {
  userId: string;
  role: 'owner' | 'admin' | 'member' | 'guest';
  permissions: string[];
  joinedAt: Date;
  isActive: boolean;
}

export interface ElementStorage {
  totalSize: number;
  usedSize: number;
  fileCount: number;
  webDriveEnabled: boolean;
  webDriveAddress?: string;
  encryptionEnabled: boolean;
  accessControl: 'public' | 'members-only' | 'private';
}

export interface ElementCommunication {
  activeCall?: {
    id: string;
    type: 'voice' | 'video' | 'screen-share';
    participants: string[];
    startedAt: Date;
  };
  unreadCount: number;
  lastActivity: Date;
  typingUsers: string[];
}

export interface ElementMetadata {
  tags: string[];
  category?: string;
  priority?: 'low' | 'medium' | 'high';
  status?: 'active' | 'archived' | 'deleted';
  customFields?: Record<string, any>;
}

export interface Element {
  identity: ElementIdentity;
  capabilities: ElementCapabilities;
  membership: ElementMembership[];
  storage: ElementStorage;
  communication: ElementCommunication;
  metadata: ElementMetadata;

  // Context-specific data
  organizationId?: string; // For organization-scoped elements
  parentId?: string; // For hierarchical elements (channels in orgs, etc.)
  children?: string[]; // Child elements
}

// Element Actions
export interface ElementAction {
  id: string;
  type: 'text' | 'voice' | 'video' | 'screen-share' | 'file-share' | 'storage' | 'web-drive';
  userId: string;
  timestamp: Date;
  data: any;
}

// Element Events
export type ElementEventType =
  | 'member-joined'
  | 'member-left'
  | 'message-sent'
  | 'call-started'
  | 'call-ended'
  | 'file-uploaded'
  | 'storage-updated'
  | 'settings-changed';

export interface ElementEvent {
  id: string;
  elementId: string;
  type: ElementEventType;
  userId: string;
  timestamp: Date;
  data: any;
}

// Element Context for UI
export interface ElementContext {
  currentElement: Element;
  userRole: 'owner' | 'admin' | 'member' | 'guest';
  userPermissions: string[];
  isOnline: boolean;
  activeUsers: string[];
  recentActivity: ElementEvent[];
}

// Element Factory Functions
export interface ElementConfig {
  type: ElementType;
  name: string;
  description?: string;
  organizationId?: string;
  parentId?: string;
  initialMembers?: string[];
  capabilities?: Partial<ElementCapabilities>;
  storage?: Partial<ElementStorage>;
  metadata?: Partial<ElementMetadata>;
}

export interface ElementTemplate {
  id: string;
  name: string;
  description: string;
  type: ElementType;
  defaultCapabilities: ElementCapabilities;
  defaultStorage: ElementStorage;
  defaultMetadata: ElementMetadata;
  icon: string;
  color: string;
}