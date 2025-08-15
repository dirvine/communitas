import { 
  Organization, 
  Group, 
  PersonalUser, 
  Channel, 
  Project, 
  OrganizationUser,
  SharedFile,
  PublishedWebsite 
} from '../types/collaboration';

// Helper to generate four-word identities
const generateFourWords = () => {
  const words = [
    ['ocean', 'forest', 'mountain', 'desert', 'river', 'valley', 'island', 'prairie'],
    ['blue', 'green', 'golden', 'silver', 'crystal', 'shadow', 'bright', 'misty'],
    ['eagle', 'wolf', 'bear', 'fox', 'owl', 'hawk', 'lion', 'tiger'],
    ['star', 'moon', 'sun', 'cloud', 'storm', 'wind', 'fire', 'ice']
  ];
  return words.map(group => group[Math.floor(Math.random() * group.length)]).join('-');
};

// Mock Organizations
export const mockOrganizations: Organization[] = [
  {
    id: 'org-1',
    type: 'organization',
    name: 'Acme Corporation',
    description: 'Global technology solutions',
    networkIdentity: {
      fourWords: 'ocean-blue-eagle-star',
      publicKey: 'pk_acme_123',
      dhtAddress: 'dht://acme'
    },
    capabilities: {
      videoCall: true,
      audioCall: true,
      screenShare: true,
      fileShare: true,
      websitePublish: true
    },
    owners: ['user-1'],
    channels: [
      {
        id: 'channel-1',
        type: 'channel',
        name: 'general',
        organizationId: 'org-1',
        isPrivate: false,
        members: ['user-1', 'user-2', 'user-3'],
        networkIdentity: {
          fourWords: 'valley-green-wolf-moon',
          publicKey: 'pk_channel_1',
          dhtAddress: 'dht://channel-1'
        },
        capabilities: {
          videoCall: true,
          audioCall: true,
          screenShare: true,
          fileShare: true,
          websitePublish: true
        },
        createdAt: new Date('2024-01-01'),
        updatedAt: new Date('2024-01-15')
      },
      {
        id: 'channel-2',
        type: 'channel',
        name: 'engineering',
        organizationId: 'org-1',
        isPrivate: false,
        members: ['user-1', 'user-2'],
        networkIdentity: {
          fourWords: 'mountain-silver-fox-cloud',
          publicKey: 'pk_channel_2',
          dhtAddress: 'dht://channel-2'
        },
        capabilities: {
          videoCall: true,
          audioCall: true,
          screenShare: true,
          fileShare: true,
          websitePublish: true
        },
        createdAt: new Date('2024-01-01'),
        updatedAt: new Date('2024-01-15')
      },
      {
        id: 'channel-3',
        type: 'channel',
        name: 'marketing',
        organizationId: 'org-1',
        isPrivate: false,
        members: ['user-3', 'user-4'],
        networkIdentity: {
          fourWords: 'desert-golden-hawk-storm',
          publicKey: 'pk_channel_3',
          dhtAddress: 'dht://channel-3'
        },
        capabilities: {
          videoCall: true,
          audioCall: true,
          screenShare: true,
          fileShare: true,
          websitePublish: true
        },
        createdAt: new Date('2024-01-01'),
        updatedAt: new Date('2024-01-15')
      }
    ],
    groups: [
      {
        id: 'group-org-1',
        type: 'group',
        name: 'Development Team',
        organizationId: 'org-1',
        members: ['user-1', 'user-2'],
        admins: ['user-1'],
        isPersonal: false,
        networkIdentity: {
          fourWords: 'river-crystal-lion-wind',
          publicKey: 'pk_group_org_1',
          dhtAddress: 'dht://group-org-1'
        },
        capabilities: {
          videoCall: true,
          audioCall: true,
          screenShare: true,
          fileShare: true,
          websitePublish: true
        },
        createdAt: new Date('2024-01-05'),
        updatedAt: new Date('2024-01-20')
      },
      {
        id: 'group-org-2',
        type: 'group',
        name: 'Design Team',
        organizationId: 'org-1',
        members: ['user-3', 'user-4'],
        admins: ['user-3'],
        isPersonal: false,
        networkIdentity: {
          fourWords: 'island-shadow-tiger-fire',
          publicKey: 'pk_group_org_2',
          dhtAddress: 'dht://group-org-2'
        },
        capabilities: {
          videoCall: true,
          audioCall: true,
          screenShare: true,
          fileShare: true,
          websitePublish: true
        },
        createdAt: new Date('2024-01-05'),
        updatedAt: new Date('2024-01-20')
      }
    ],
    users: [
      {
        id: 'org-user-1',
        type: 'user',
        name: 'Alice Johnson',
        organizationId: 'org-1',
        userId: 'user-1',
        role: 'owner',
        permissions: ['all'],
        joinedAt: new Date('2024-01-01'),
        networkIdentity: {
          fourWords: 'prairie-bright-owl-ice',
          publicKey: 'pk_alice',
          dhtAddress: 'dht://alice'
        },
        capabilities: {
          videoCall: true,
          audioCall: true,
          screenShare: true,
          fileShare: true,
          websitePublish: true
        },
        createdAt: new Date('2024-01-01'),
        updatedAt: new Date('2024-01-15')
      },
      {
        id: 'org-user-2',
        type: 'user',
        name: 'Bob Chen',
        organizationId: 'org-1',
        userId: 'user-2',
        role: 'admin',
        permissions: ['manage_users', 'manage_content'],
        joinedAt: new Date('2024-01-02'),
        networkIdentity: {
          fourWords: 'ocean-misty-bear-sun',
          publicKey: 'pk_bob',
          dhtAddress: 'dht://bob'
        },
        capabilities: {
          videoCall: true,
          audioCall: true,
          screenShare: true,
          fileShare: true,
          websitePublish: true
        },
        createdAt: new Date('2024-01-02'),
        updatedAt: new Date('2024-01-15')
      },
      {
        id: 'org-user-3',
        type: 'user',
        name: 'Carol Davis',
        organizationId: 'org-1',
        userId: 'user-3',
        role: 'member',
        permissions: ['read', 'write'],
        joinedAt: new Date('2024-01-03'),
        networkIdentity: {
          fourWords: 'forest-blue-eagle-cloud',
          publicKey: 'pk_carol',
          dhtAddress: 'dht://carol'
        },
        capabilities: {
          videoCall: true,
          audioCall: true,
          screenShare: true,
          fileShare: true,
          websitePublish: true
        },
        createdAt: new Date('2024-01-03'),
        updatedAt: new Date('2024-01-15')
      }
    ],
    projects: [
      {
        id: 'project-1',
        type: 'project',
        name: 'Website Redesign',
        organizationId: 'org-1',
        leads: ['user-1'],
        members: ['user-1', 'user-2', 'user-3'],
        status: 'active',
        startDate: new Date('2024-01-10'),
        endDate: new Date('2024-06-01'),
        milestones: [
          {
            id: 'milestone-1',
            name: 'Design Phase',
            description: 'Complete all design mockups',
            dueDate: new Date('2024-02-15'),
            completed: false
          }
        ],
        networkIdentity: {
          fourWords: 'mountain-green-wolf-star',
          publicKey: 'pk_project_1',
          dhtAddress: 'dht://project-1'
        },
        capabilities: {
          videoCall: true,
          audioCall: true,
          screenShare: true,
          fileShare: true,
          websitePublish: true
        },
        createdAt: new Date('2024-01-10'),
        updatedAt: new Date('2024-01-20')
      },
      {
        id: 'project-2',
        type: 'project',
        name: 'Mobile App Development',
        organizationId: 'org-1',
        leads: ['user-2'],
        members: ['user-1', 'user-2'],
        status: 'planning',
        milestones: [],
        networkIdentity: {
          fourWords: 'valley-silver-fox-moon',
          publicKey: 'pk_project_2',
          dhtAddress: 'dht://project-2'
        },
        capabilities: {
          videoCall: true,
          audioCall: true,
          screenShare: true,
          fileShare: true,
          websitePublish: true
        },
        createdAt: new Date('2024-01-15'),
        updatedAt: new Date('2024-01-20')
      }
    ],
    settings: {
      allowGuestAccess: false,
      defaultChannelPermissions: ['read', 'write'],
      fileStorageLimit: 10737418240, // 10GB
      websitePublishingEnabled: true,
      customDomain: 'acme.communitas'
    },
    createdAt: new Date('2024-01-01'),
    updatedAt: new Date('2024-01-20')
  },
  {
    id: 'org-2',
    type: 'organization',
    name: 'Tech Startup Inc',
    description: 'Innovation in AI and blockchain',
    networkIdentity: {
      fourWords: 'desert-crystal-hawk-wind',
      publicKey: 'pk_techstartup',
      dhtAddress: 'dht://techstartup'
    },
    capabilities: {
      videoCall: true,
      audioCall: true,
      screenShare: true,
      fileShare: true,
      websitePublish: true
    },
    owners: ['user-1'],
    channels: [],
    groups: [],
    users: [],
    projects: [],
    settings: {
      allowGuestAccess: true,
      defaultChannelPermissions: ['read'],
      websitePublishingEnabled: true
    },
    createdAt: new Date('2024-01-10'),
    updatedAt: new Date('2024-01-20')
  }
];

// Mock Personal Groups
export const mockPersonalGroups: Group[] = [
  {
    id: 'personal-group-1',
    type: 'group',
    name: 'Family',
    members: ['user-1', 'user-5', 'user-6'],
    admins: ['user-1'],
    isPersonal: true,
    networkIdentity: {
      fourWords: 'island-bright-lion-storm',
      publicKey: 'pk_family',
      dhtAddress: 'dht://family'
    },
    capabilities: {
      videoCall: true,
      audioCall: true,
      screenShare: true,
      fileShare: true,
      websitePublish: true
    },
    createdAt: new Date('2024-01-01'),
    updatedAt: new Date('2024-01-15')
  },
  {
    id: 'personal-group-2',
    type: 'group',
    name: 'Friends',
    members: ['user-1', 'user-7', 'user-8', 'user-9'],
    admins: ['user-1'],
    isPersonal: true,
    networkIdentity: {
      fourWords: 'prairie-shadow-tiger-fire',
      publicKey: 'pk_friends',
      dhtAddress: 'dht://friends'
    },
    capabilities: {
      videoCall: true,
      audioCall: true,
      screenShare: true,
      fileShare: true,
      websitePublish: true
    },
    createdAt: new Date('2024-01-05'),
    updatedAt: new Date('2024-01-18')
  },
  {
    id: 'personal-group-3',
    type: 'group',
    name: 'Book Club',
    members: ['user-1', 'user-10', 'user-11'],
    admins: ['user-1', 'user-10'],
    isPersonal: true,
    networkIdentity: {
      fourWords: 'river-misty-owl-ice',
      publicKey: 'pk_bookclub',
      dhtAddress: 'dht://bookclub'
    },
    capabilities: {
      videoCall: true,
      audioCall: true,
      screenShare: true,
      fileShare: true,
      websitePublish: true
    },
    createdAt: new Date('2024-01-12'),
    updatedAt: new Date('2024-01-19')
  }
];

// Mock Personal Users/Contacts
export const mockPersonalUsers: PersonalUser[] = [
  {
    id: 'personal-user-1',
    type: 'personal_user',
    name: 'David Smith',
    userId: 'user-5',
    relationship: 'friend',
    lastContact: new Date('2024-01-19'),
    networkIdentity: {
      fourWords: 'ocean-golden-bear-sun',
      publicKey: 'pk_david',
      dhtAddress: 'dht://david'
    },
    capabilities: {
      videoCall: true,
      audioCall: true,
      screenShare: true,
      fileShare: true,
      websitePublish: true
    },
    createdAt: new Date('2024-01-01'),
    updatedAt: new Date('2024-01-19')
  },
  {
    id: 'personal-user-2',
    type: 'personal_user',
    name: 'Emma Wilson',
    userId: 'user-6',
    relationship: 'colleague',
    lastContact: new Date('2024-01-18'),
    networkIdentity: {
      fourWords: 'forest-silver-eagle-cloud',
      publicKey: 'pk_emma',
      dhtAddress: 'dht://emma'
    },
    capabilities: {
      videoCall: true,
      audioCall: true,
      screenShare: true,
      fileShare: true,
      websitePublish: true
    },
    createdAt: new Date('2024-01-02'),
    updatedAt: new Date('2024-01-18')
  },
  {
    id: 'personal-user-3',
    type: 'personal_user',
    name: 'Frank Zhang',
    userId: 'user-7',
    relationship: 'friend',
    lastContact: new Date('2024-01-17'),
    networkIdentity: {
      fourWords: 'mountain-crystal-wolf-moon',
      publicKey: 'pk_frank',
      dhtAddress: 'dht://frank'
    },
    capabilities: {
      videoCall: true,
      audioCall: true,
      screenShare: true,
      fileShare: true,
      websitePublish: true
    },
    createdAt: new Date('2024-01-03'),
    updatedAt: new Date('2024-01-17')
  },
  {
    id: 'personal-user-4',
    type: 'personal_user',
    name: 'Grace Lee',
    userId: 'user-8',
    relationship: 'contact',
    lastContact: new Date('2024-01-15'),
    networkIdentity: {
      fourWords: 'valley-green-fox-star',
      publicKey: 'pk_grace',
      dhtAddress: 'dht://grace'
    },
    capabilities: {
      videoCall: true,
      audioCall: true,
      screenShare: true,
      fileShare: true,
      websitePublish: true
    },
    createdAt: new Date('2024-01-05'),
    updatedAt: new Date('2024-01-15')
  },
  {
    id: 'personal-user-5',
    type: 'personal_user',
    name: 'Henry Brown',
    userId: 'user-9',
    relationship: 'colleague',
    networkIdentity: {
      fourWords: 'desert-blue-hawk-wind',
      publicKey: 'pk_henry',
      dhtAddress: 'dht://henry'
    },
    capabilities: {
      videoCall: true,
      audioCall: true,
      screenShare: true,
      fileShare: true,
      websitePublish: true
    },
    createdAt: new Date('2024-01-08'),
    updatedAt: new Date('2024-01-16')
  }
];

// Mock Shared Files
export const mockSharedFiles: SharedFile[] = [
  {
    id: 'file-1',
    name: 'README.md',
    path: '/docs/README.md',
    size: 2048,
    mimeType: 'text/markdown',
    networkIdentity: {
      fourWords: 'river-bright-lion-storm',
      publicKey: 'pk_file_1',
      dhtAddress: 'dht://file-1'
    },
    owner: 'user-1',
    sharedWith: ['user-2', 'user-3'],
    permissions: {
      read: true,
      write: true,
      delete: false,
      share: true
    },
    createdAt: new Date('2024-01-10'),
    modifiedAt: new Date('2024-01-15'),
    version: 3
  },
  {
    id: 'file-2',
    name: 'index.html',
    path: '/website/index.html',
    size: 5120,
    mimeType: 'text/html',
    networkIdentity: {
      fourWords: 'island-shadow-tiger-fire',
      publicKey: 'pk_file_2',
      dhtAddress: 'dht://file-2'
    },
    forwardIdentity: {
      fourWords: 'prairie-misty-owl-ice',
      publicKey: 'pk_file_2_forward',
      dhtAddress: 'dht://file-2-forward'
    },
    owner: 'user-1',
    sharedWith: [],
    permissions: {
      read: true,
      write: true,
      delete: true,
      share: true
    },
    createdAt: new Date('2024-01-12'),
    modifiedAt: new Date('2024-01-18'),
    version: 5
  },
  {
    id: 'file-3',
    name: 'presentation.pdf',
    path: '/documents/presentation.pdf',
    size: 1048576,
    mimeType: 'application/pdf',
    networkIdentity: {
      fourWords: 'ocean-golden-bear-sun',
      publicKey: 'pk_file_3',
      dhtAddress: 'dht://file-3'
    },
    owner: 'user-2',
    sharedWith: ['user-1', 'user-3', 'user-4'],
    permissions: {
      read: true,
      write: false,
      delete: false,
      share: false
    },
    createdAt: new Date('2024-01-14'),
    modifiedAt: new Date('2024-01-14'),
    version: 1
  }
];

// Mock Published Website
export const mockPublishedWebsite: PublishedWebsite = {
  id: 'website-1',
  name: 'Acme Corp Public Site',
  domain: 'ocean-blue-eagle-star',
  files: mockSharedFiles.filter(f => f.mimeType.includes('html') || f.mimeType.includes('markdown')),
  indexFile: 'index.html',
  networkIdentity: {
    fourWords: 'ocean-blue-eagle-star',
    publicKey: 'pk_website_1',
    dhtAddress: 'dht://website-1'
  },
  published: true,
  publishedAt: new Date('2024-01-15'),
  analytics: {
    views: 1234,
    uniqueVisitors: 567,
    bandwidth: 10485760 // 10MB
  }
};