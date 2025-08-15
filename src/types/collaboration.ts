// Collaboration Platform Types
// Defines the hierarchical structure for organizations, groups, and users
// with universal collaboration capabilities

export interface NetworkIdentity {
  fourWords: string; // e.g., "ocean-forest-moon-star"
  publicKey: string;
  dhtAddress: string;
}

export interface CollaborationCapabilities {
  videoCall: boolean;
  audioCall: boolean;
  screenShare: boolean;
  fileShare: boolean;
  websitePublish: boolean;
}

export interface BaseEntity {
  id: string;
  name: string;
  description?: string;
  avatar?: string;
  networkIdentity: NetworkIdentity;
  capabilities: CollaborationCapabilities;
  createdAt: Date;
  updatedAt: Date;
}

// Organization entities
export interface Organization extends BaseEntity {
  type: 'organization';
  owners: string[];
  channels: Channel[];
  groups: Group[];
  users: OrganizationUser[];
  projects: Project[];
  settings: OrganizationSettings;
}

export interface Channel extends BaseEntity {
  type: 'channel';
  organizationId: string;
  isPrivate: boolean;
  members: string[];
  pinnedMessages?: string[];
  topic?: string;
}

export interface Group extends BaseEntity {
  type: 'group';
  organizationId?: string; // Optional - personal groups don't have this
  members: string[];
  admins: string[];
  isPersonal: boolean;
}

export interface OrganizationUser extends BaseEntity {
  type: 'user';
  organizationId: string;
  userId: string; // Reference to actual user
  role: 'owner' | 'admin' | 'member' | 'guest';
  permissions: string[];
  joinedAt: Date;
}

export interface Project extends BaseEntity {
  type: 'project';
  organizationId: string;
  leads: string[];
  members: string[];
  status: 'planning' | 'active' | 'completed' | 'archived';
  startDate?: Date;
  endDate?: Date;
  milestones: Milestone[];
}

export interface Milestone {
  id: string;
  name: string;
  description: string;
  dueDate: Date;
  completed: boolean;
}

// Personal space entities
export interface PersonalUser extends BaseEntity {
  type: 'personal_user';
  userId: string;
  relationship: 'contact' | 'friend' | 'colleague' | 'blocked';
  lastContact?: Date;
}

// File sharing with network identity
export interface SharedFile {
  id: string;
  name: string;
  path: string;
  size: number;
  mimeType: string;
  networkIdentity: NetworkIdentity; // Each file has its own identity
  forwardIdentity?: NetworkIdentity; // Optional forward identity
  owner: string;
  sharedWith: string[];
  permissions: FilePermissions;
  createdAt: Date;
  modifiedAt: Date;
  version: number;
}

export interface FilePermissions {
  read: boolean;
  write: boolean;
  delete: boolean;
  share: boolean;
}

// Website publishing
export interface PublishedWebsite {
  id: string;
  name: string;
  domain: string; // Four-word domain
  files: SharedFile[];
  indexFile: string;
  networkIdentity: NetworkIdentity;
  published: boolean;
  publishedAt?: Date;
  analytics?: WebsiteAnalytics;
}

export interface WebsiteAnalytics {
  views: number;
  uniqueVisitors: number;
  bandwidth: number;
}

// Settings
export interface OrganizationSettings {
  allowGuestAccess: boolean;
  defaultChannelPermissions: string[];
  fileStorageLimit?: number;
  websitePublishingEnabled: boolean;
  customDomain?: string;
}

// Communication features
export interface CallSession {
  id: string;
  type: 'video' | 'audio';
  participants: string[];
  startedAt: Date;
  endedAt?: Date;
  recording?: SharedFile;
}

export interface ScreenShareSession {
  id: string;
  sharerId: string;
  viewers: string[];
  startedAt: Date;
  endedAt?: Date;
  quality: 'low' | 'medium' | 'high' | '4k';
}

// Navigation structure
export interface NavigationItem {
  id: string;
  label: string;
  icon: string;
  path: string;
  badge?: number;
  children?: NavigationItem[];
}

export interface AppNavigation {
  organizations: Organization[];
  personalGroups: Group[];
  personalUsers: PersonalUser[];
}