import { ElementTemplate, ElementCapabilities, ElementStorage, ElementMetadata, ElementType } from '../../types/element';

export const ELEMENT_TEMPLATES: Record<ElementType, ElementTemplate> = {
  'personal-individual': {
    id: 'personal-individual',
    name: 'Personal Space',
    description: 'Your private individual workspace with full communication and storage capabilities',
    type: 'personal-individual',
    defaultCapabilities: {
      text: true,
      voice: true,
      video: true,
      screenShare: true,
      storage: true,
      webDrive: true,
      fileSharing: true,
      collaborativeEditing: true,
    },
    defaultStorage: {
      totalSize: 10737418240, // 10GB
      usedSize: 0,
      fileCount: 0,
      webDriveEnabled: true,
      webDriveAddress: undefined, // Will be generated
      encryptionEnabled: true,
      accessControl: 'private',
    },
    defaultMetadata: {
      tags: ['personal', 'private'],
      category: 'personal',
      priority: 'high',
      status: 'active',
      customFields: {},
    },
    icon: 'person',
    color: '#2196F3',
  },

  'personal-group': {
    id: 'personal-group',
    name: 'Personal Group',
    description: 'Private group for friends and family with shared communication and storage',
    type: 'personal-group',
    defaultCapabilities: {
      text: true,
      voice: true,
      video: true,
      screenShare: true,
      storage: true,
      webDrive: true,
      fileSharing: true,
      collaborativeEditing: true,
    },
    defaultStorage: {
      totalSize: 53687091200, // 50GB
      usedSize: 0,
      fileCount: 0,
      webDriveEnabled: true,
      webDriveAddress: undefined,
      encryptionEnabled: true,
      accessControl: 'members-only',
    },
    defaultMetadata: {
      tags: ['personal', 'group', 'shared'],
      category: 'social',
      priority: 'medium',
      status: 'active',
      customFields: {},
    },
    icon: 'groups',
    color: '#4CAF50',
  },

  'organization-individual': {
    id: 'organization-individual',
    name: 'Organization Member Space',
    description: 'Individual workspace within an organization with org-specific capabilities',
    type: 'organization-individual',
    defaultCapabilities: {
      text: true,
      voice: true,
      video: true,
      screenShare: true,
      storage: true,
      webDrive: true,
      fileSharing: true,
      collaborativeEditing: true,
    },
    defaultStorage: {
      totalSize: 5368709120, // 5GB
      usedSize: 0,
      fileCount: 0,
      webDriveEnabled: true,
      webDriveAddress: undefined,
      encryptionEnabled: true,
      accessControl: 'private',
    },
    defaultMetadata: {
      tags: ['organization', 'individual', 'work'],
      category: 'work',
      priority: 'high',
      status: 'active',
      customFields: {},
    },
    icon: 'business',
    color: '#FF9800',
  },

  'organization-group': {
    id: 'organization-group',
    name: 'Organization Team',
    description: 'Team workspace within an organization for collaboration',
    type: 'organization-group',
    defaultCapabilities: {
      text: true,
      voice: true,
      video: true,
      screenShare: true,
      storage: true,
      webDrive: true,
      fileSharing: true,
      collaborativeEditing: true,
    },
    defaultStorage: {
      totalSize: 107374182400, // 100GB
      usedSize: 0,
      fileCount: 0,
      webDriveEnabled: true,
      webDriveAddress: undefined,
      encryptionEnabled: true,
      accessControl: 'members-only',
    },
    defaultMetadata: {
      tags: ['organization', 'team', 'collaboration'],
      category: 'work',
      priority: 'high',
      status: 'active',
      customFields: {},
    },
    icon: 'group_work',
    color: '#9C27B0',
  },

  'organization-project': {
    id: 'organization-project',
    name: 'Project Workspace',
    description: 'Dedicated project space with full collaboration tools and storage',
    type: 'organization-project',
    defaultCapabilities: {
      text: true,
      voice: true,
      video: true,
      screenShare: true,
      storage: true,
      webDrive: true,
      fileSharing: true,
      collaborativeEditing: true,
    },
    defaultStorage: {
      totalSize: 536870912000, // 500GB
      usedSize: 0,
      fileCount: 0,
      webDriveEnabled: true,
      webDriveAddress: undefined,
      encryptionEnabled: true,
      accessControl: 'members-only',
    },
    defaultMetadata: {
      tags: ['organization', 'project', 'collaboration'],
      category: 'work',
      priority: 'high',
      status: 'active',
      customFields: {
        startDate: null,
        endDate: null,
        milestones: [],
      },
    },
    icon: 'assignment',
    color: '#3F51B5',
  },

  'organization-channel': {
    id: 'organization-channel',
    name: 'Communication Channel',
    description: 'Public communication channel for organization-wide discussions',
    type: 'organization-channel',
    defaultCapabilities: {
      text: true,
      voice: true,
      video: true,
      screenShare: true,
      storage: true,
      webDrive: true, // Organization channels have web drives for shared resources
      fileSharing: true,
      collaborativeEditing: false, // Channels focus on communication
    },
    defaultStorage: {
      totalSize: 10737418240, // 10GB shared
      usedSize: 0,
      fileCount: 0,
      webDriveEnabled: true,
      webDriveAddress: undefined, // Will be generated
      encryptionEnabled: true,
      accessControl: 'members-only',
    },
    defaultMetadata: {
      tags: ['organization', 'channel', 'communication'],
      category: 'communication',
      priority: 'medium',
      status: 'active',
      customFields: {
        topic: '',
        isPrivate: false,
      },
    },
    icon: 'tag',
    color: '#607D8B',
  },
};

export function getElementTemplate(type: ElementType): ElementTemplate {
  return ELEMENT_TEMPLATES[type];
}

export function getAllElementTemplates(): ElementTemplate[] {
  return Object.values(ELEMENT_TEMPLATES);
}

export function getElementTemplatesByScope(scope: 'personal' | 'organization'): ElementTemplate[] {
  return Object.values(ELEMENT_TEMPLATES).filter(template =>
    template.type.startsWith(scope)
  );
}