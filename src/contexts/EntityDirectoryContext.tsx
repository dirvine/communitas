import { nanoid } from 'nanoid';
import React, {
    createContext, ReactNode, useCallback,
    useContext,
    useEffect,
    useMemo,
    useState
} from 'react';
import {
    Channel, CollaborationCapabilities, Group, NetworkIdentity, Organization, PersonalUser, Project
} from '../types/collaboration';
// Mock data removed - now loading from backend
import { invoke } from '@tauri-apps/api/core';
import {
    AddExistingChannelInput, AddExistingContactInput, AddExistingGroupInput, AddExistingOrganizationInput, AddExistingProjectInput, CreateNewChannelInput, CreateNewContactInput, CreateNewGroupInput, CreateNewOrganizationInput, CreateNewProjectInput, EntityOperationResult,
    FourWordsValidationResult
} from '../types/entityOperations';
import { validateFourWordIdentity } from '../utils/identity';
import {
    markMessageStatus as markCachedMessageStatus,
    removeMessage as removeCachedMessage
} from '../utils/messageStore';
import { useAuth } from './AuthContext';

type SyncStatus = 'synced' | 'new' | 'dirty' | 'deleted' | 'error';

interface EntityMetadata {
  syncStatus?: SyncStatus;
  lastSyncedAt?: Date;
  error?: string;
}

interface EntityDirectoryState {
  organizations: Array<Organization & EntityMetadata>;
  personalGroups: Array<Group & EntityMetadata>;
  personalUsers: Array<PersonalUser & EntityMetadata>;
  operations: EntitySyncOperation[];
}

interface CreateOrganizationInput {
  name: string;
  description?: string;
}

interface CreateGroupInput {
  name: string;
  description?: string;
  organizationId?: string;
}

interface CreateChannelInput {
  organizationId: string;
  name: string;
  description?: string;
  isPrivate?: boolean;
}

interface CreateProjectInput {
  organizationId: string;
  name: string;
  description?: string;
}

interface CreatePersonalUserInput {
  name: string;
  relationship?: PersonalUser['relationship'];
  email?: string;
}

interface MessageOperationPayload {
  id: string;
  entityId: string;
  entityType: 'group' | 'channel' | 'user' | 'project' | 'organization';
  content: string;
  timestamp: string;
}

type EntitySyncEntityType = 'organization' | 'group' | 'channel' | 'project' | 'contact' | 'message' | 'storage';
type EntitySyncOperationType = 'create' | 'update' | 'delete' | 'resolve';

interface EntitySyncOperation {
  id: string;
  entityType: EntitySyncEntityType;
  operation: EntitySyncOperationType;
  payload: Record<string, any>;
  timestamp: string;
  attempts: number;
  status: 'pending' | 'processing' | 'failed';
  error?: string;
}

interface EntityDirectoryContextValue extends Omit<EntityDirectoryState, 'operations'> {
  operations: EntitySyncOperation[];
  // Create operations (generate new Four-Words)
  createOrganization: (input: CreateNewOrganizationInput) => Promise<EntityOperationResult>;
  createGroup: (input: CreateNewGroupInput) => Promise<EntityOperationResult>;
  createChannel: (input: CreateNewChannelInput) => Promise<EntityOperationResult>;
  createProject: (input: CreateNewProjectInput) => Promise<EntityOperationResult>;
  createContact: (input: CreateNewContactInput) => Promise<EntityOperationResult>;
  // Add operations (use existing Four-Words)
  addExistingOrganization: (input: AddExistingOrganizationInput) => Promise<EntityOperationResult>;
  addExistingGroup: (input: AddExistingGroupInput) => Promise<EntityOperationResult>;
  addExistingChannel: (input: AddExistingChannelInput) => Promise<EntityOperationResult>;
  addExistingProject: (input: AddExistingProjectInput) => Promise<EntityOperationResult>;
  addExistingContact: (input: AddExistingContactInput) => Promise<EntityOperationResult>;
  // Group member management
  addGroupMember: (groupId: string, userId: string, role?: string) => Promise<void>;
  removeGroupMember: (groupId: string, userId: string) => Promise<void>;
  getGroupMembers: (groupId: string) => Promise<Array<{ userId: string; role: string }>>;
  // Validation
  validateFourWords: (fourWords: string) => Promise<FourWordsValidationResult>;
  // Legacy methods (will refactor later)
  addOrganization: (input: CreateOrganizationInput) => Promise<EntityOperationResult>;
  removeOrganization: (organizationId: string) => void;
  addOrganizationGroup: (input: CreateGroupInput & { organizationId: string }) => Group;
  removeOrganizationGroup: (organizationId: string, groupId: string) => void;
  addOrganizationChannel: (input: CreateChannelInput) => Channel;
  removeOrganizationChannel: (organizationId: string, channelId: string) => void;
  addProject: (input: CreateProjectInput) => Project;
  removeProject: (organizationId: string, projectId: string) => void;
  addPersonalGroup: (input: CreateGroupInput) => Promise<EntityOperationResult>;
  removePersonalGroup: (groupId: string) => void;
  addPersonalUser: (input: CreatePersonalUserInput) => PersonalUser;
  removePersonalUser: (userId: string) => void;
  resetDirectory: () => void;
  enqueueOperation: (operation: EntitySyncOperation) => void;
  markOperationComplete: (operationId: string, newIdMap?: Record<string, string>) => void;
  markOperationFailed: (operationId: string, error: string) => void;
  setEntityStatus: (entityType: EntitySyncEntityType, entityId: string, status: SyncStatus) => void;
  queueMessage: (payload: MessageOperationPayload) => void;
}

const EntityDirectoryContext = createContext<EntityDirectoryContextValue | undefined>(undefined);

const STORAGE_KEY = 'communitas-entity-directory';

const defaultCapabilities: CollaborationCapabilities = {
  videoCall: true,
  audioCall: true,
  screenShare: true,
  fileShare: true,
  websitePublish: true,
};

const generateFourWords = async (): Promise<string> => {
  // Check if running in Tauri
  if (typeof window !== 'undefined' && '__TAURI__' in window) {
    // Use real saorsa-core four-word generation
    return await invoke<string>('generate_four_word_identity');
  }

  // Browser fallback: generate mock four-words for development
  const words = ['ocean', 'forest', 'mountain', 'river', 'eagle', 'wolf', 'star', 'moon',
                 'thunder', 'crystal', 'storm', 'phoenix', 'cloud', 'breeze', 'flame', 'frost'];
  const randomWords = Array.from({ length: 4 }, () =>
    words[Math.floor(Math.random() * words.length)]
  );
  return randomWords.join('-');
};

const createNetworkIdentity = (isOwned: boolean = true, providedFourWords?: string): NetworkIdentity => {
  const fourWords = providedFourWords || `temp-${nanoid(8)}`;
  return {
    fourWords,
    publicKey: `pk_${nanoid(12)}`,
    dhtAddress: `dht://${fourWords.replace(/-/g, '')}-${nanoid(6)}`,
    isOwned,
    isValidated: !!providedFourWords, // Only validated if provided
  };
};
const MAX_OPERATION_ATTEMPTS = 3;

const reviveDates = (state: any): EntityDirectoryState => ({
  organizations: (state?.organizations ?? []).map((org: any) => ({
    ...org,
    createdAt: new Date(org.createdAt),
    updatedAt: new Date(org.updatedAt),
    lastSyncedAt: org.lastSyncedAt ? new Date(org.lastSyncedAt) : undefined,
    channels: (org.channels ?? []).map((channel: any) => ({
      ...channel,
      createdAt: new Date(channel.createdAt),
      updatedAt: new Date(channel.updatedAt),
      lastSyncedAt: channel.lastSyncedAt ? new Date(channel.lastSyncedAt) : undefined,
    })),
    groups: (org.groups ?? []).map((group: any) => ({
      ...group,
      createdAt: new Date(group.createdAt),
      updatedAt: new Date(group.updatedAt),
      lastSyncedAt: group.lastSyncedAt ? new Date(group.lastSyncedAt) : undefined,
    })),
    users: (org.users ?? []).map((user: any) => ({
      ...user,
      createdAt: new Date(user.createdAt),
      updatedAt: new Date(user.updatedAt),
      joinedAt: new Date(user.joinedAt),
      lastSyncedAt: user.lastSyncedAt ? new Date(user.lastSyncedAt) : undefined,
    })),
    projects: (org.projects ?? []).map((project: any) => ({
      ...project,
      createdAt: new Date(project.createdAt),
      updatedAt: new Date(project.updatedAt),
      lastSyncedAt: project.lastSyncedAt ? new Date(project.lastSyncedAt) : undefined,
      startDate: project.startDate ? new Date(project.startDate) : undefined,
      endDate: project.endDate ? new Date(project.endDate) : undefined,
      milestones: (project.milestones ?? []).map((milestone: any) => ({
        ...milestone,
        dueDate: new Date(milestone.dueDate),
      })),
    })),
  })),
  personalGroups: (state?.personalGroups ?? []).map((group: any) => ({
    ...group,
    createdAt: new Date(group.createdAt),
    updatedAt: new Date(group.updatedAt),
    lastSyncedAt: group.lastSyncedAt ? new Date(group.lastSyncedAt) : undefined,
  })),
  personalUsers: (state?.personalUsers ?? []).map((user: any) => ({
    ...user,
    createdAt: new Date(user.createdAt),
    updatedAt: new Date(user.updatedAt),
    lastContact: user.lastContact ? new Date(user.lastContact) : undefined,
    lastSyncedAt: user.lastSyncedAt ? new Date(user.lastSyncedAt) : undefined,
  })),
  operations: (state?.operations ?? []).map((op: any) => ({
    ...op,
    attempts: op.attempts ?? 0,
    status: op.status ?? 'pending',
    timestamp: op.timestamp ?? new Date().toISOString(),
  })),
});

const serializeState = (state: EntityDirectoryState) => JSON.stringify(state);

const cloneDate = (value?: Date | string | null): Date | undefined => {
  if (!value) {
    return undefined;
  }
  return value instanceof Date ? new Date(value) : new Date(value);
};

const applySyncedMetadata = <T extends { createdAt?: Date; updatedAt?: Date }>(entity: T): T & EntityMetadata => ({
  ...entity,
  createdAt: cloneDate(entity.createdAt) ?? new Date(),
  updatedAt: cloneDate(entity.updatedAt) ?? new Date(),
  syncStatus: 'synced',
  lastSyncedAt: cloneDate(entity.updatedAt) ?? new Date(),
});

const cloneOrganizationGraph = (org: Organization): Organization & EntityMetadata => ({
  ...org,
  createdAt: cloneDate(org.createdAt) ?? new Date(),
  updatedAt: cloneDate(org.updatedAt) ?? new Date(),
  channels: org.channels.map(channel => (
    applySyncedMetadata({
      ...channel,
      createdAt: cloneDate(channel.createdAt) ?? new Date(),
      updatedAt: cloneDate(channel.updatedAt) ?? new Date(),
    })
  )),
  groups: org.groups.map(group => (
    applySyncedMetadata({
      ...group,
      createdAt: cloneDate(group.createdAt) ?? new Date(),
      updatedAt: cloneDate(group.updatedAt) ?? new Date(),
    })
  )),
  users: org.users.map(user => ({
    ...user,
    createdAt: cloneDate(user.createdAt) ?? new Date(),
    updatedAt: cloneDate(user.updatedAt) ?? new Date(),
    joinedAt: cloneDate(user.joinedAt) ?? new Date(),
  })),
  projects: org.projects.map(project => (
    applySyncedMetadata({
      ...project,
      createdAt: cloneDate(project.createdAt) ?? new Date(),
      updatedAt: cloneDate(project.updatedAt) ?? new Date(),
      startDate: cloneDate(project.startDate),
      endDate: cloneDate(project.endDate),
      milestones: project.milestones.map(milestone => ({
        ...milestone,
        dueDate: cloneDate(milestone.dueDate) ?? new Date(),
      })),
    })
  )),
  settings: { ...org.settings },
  syncStatus: 'synced',
  lastSyncedAt: cloneDate(org.updatedAt) ?? new Date(),
});

const clonePersonalGroupEntity = (group: Group): Group & EntityMetadata => (
  applySyncedMetadata({
    ...group,
    createdAt: cloneDate(group.createdAt) ?? new Date(),
    updatedAt: cloneDate(group.updatedAt) ?? new Date(),
  })
);

const clonePersonalUserEntity = (user: PersonalUser): PersonalUser & EntityMetadata => ({
  ...user,
  createdAt: cloneDate(user.createdAt) ?? new Date(),
  updatedAt: cloneDate(user.updatedAt) ?? new Date(),
  lastContact: cloneDate(user.lastContact),
  syncStatus: 'synced',
  lastSyncedAt: cloneDate(user.updatedAt) ?? new Date(),
});

const createInitialState = (): EntityDirectoryState => ({
  organizations: [], // Start empty - will load from backend
  personalGroups: [], // Start empty - will load from backend
  personalUsers: [], // Start empty - will load from backend
  operations: [],
});


interface EntityDirectoryProviderProps {
  children: ReactNode;
}

export const EntityDirectoryProvider: React.FC<EntityDirectoryProviderProps> = ({ children }) => {
  const { authState } = useAuth();
  const [state, setState] = useState<EntityDirectoryState>(() => {
    if (typeof window === 'undefined') {
      return createInitialState();
    }

    try {
      const raw = window.localStorage.getItem(STORAGE_KEY);
      if (!raw) {
        return createInitialState();
      }
      const parsed = JSON.parse(raw);
      return reviveDates(parsed);
    } catch (error) {
      console.warn('Failed to load entity directory from storage:', error);
      return createInitialState();
    }
  });

  useEffect(() => {
    if (typeof window === 'undefined') {
      return;
    }
    try {
      window.localStorage.setItem(STORAGE_KEY, serializeState(state));
    } catch (error) {
      console.warn('Failed to persist entity directory:', error);
    }
  }, [state]);

  const withMetadata = <T extends { id: string }>(entity: T, syncStatus: SyncStatus = 'new'): T & EntityMetadata => ({
    ...entity,
    syncStatus,
    lastSyncedAt: syncStatus === 'synced' ? new Date() : undefined,
  });

  const enqueueOperation = useCallback((operation: EntitySyncOperation) => {
    setState(prev => ({
      ...prev,
      operations: [...prev.operations, operation],
    }));
  }, []);

  const markOperationComplete = useCallback((operationId: string, newIdMap?: Record<string, string>) => {
    setState(prev => ({
      ...prev,
      operations: prev.operations.filter(op => op.id !== operationId),
      organizations: prev.organizations.map(org => {
        const mappedId = newIdMap?.[org.id];
        if (!mappedId) return org;
        return { ...org, id: mappedId, syncStatus: 'synced', lastSyncedAt: new Date() };
      }),
      personalGroups: prev.personalGroups.map(group => {
        const mappedId = newIdMap?.[group.id];
        if (!mappedId) return group;
        return { ...group, id: mappedId, syncStatus: 'synced', lastSyncedAt: new Date() };
      }),
      personalUsers: prev.personalUsers.map(user => {
        const mappedId = newIdMap?.[user.id];
        if (!mappedId) return user;
        return { ...user, id: mappedId, syncStatus: 'synced', lastSyncedAt: new Date() };
      }),
    }));
  }, []);

  const markOperationFailed = useCallback((operationId: string, error: string) => {
    let failedMessage: MessageOperationPayload | null = null;

    setState(prev => {
      const operation = prev.operations.find(op => op.id === operationId);
      const updatedOperations = prev.operations.map(op =>
        op.id === operationId
          ? {
              ...op,
              error,
              status: (op.attempts >= MAX_OPERATION_ATTEMPTS ? 'failed' : 'pending') as 'failed' | 'pending',
            }
          : op
      ) as EntitySyncOperation[];

      if (operation && operation.entityType === 'message' && operation.attempts >= MAX_OPERATION_ATTEMPTS) {
        failedMessage = operation.payload as MessageOperationPayload;
      }

      return {
        ...prev,
        operations: updatedOperations,
      };
    });

    if (failedMessage) {
      void markCachedMessageStatus(
        failedMessage.entityType,
        failedMessage.entityId,
        failedMessage.id,
        'failed',
      );
    }
  }, []);
  const startOperationProcessing = useCallback((operationId: string) => {
    setState(prev => ({
      ...prev,
      operations: prev.operations.map(op =>
        op.id === operationId
          ? { ...op, status: 'processing', attempts: op.attempts + 1, error: undefined }
          : op
      ),
    }));
  }, []);

  const setEntityStatus = useCallback((entityType: EntitySyncEntityType, entityId: string, status: SyncStatus) => {
    const mark = <T extends EntityMetadata>(entity: T): T => ({
      ...entity,
      syncStatus: status,
      lastSyncedAt: status === 'synced' ? new Date() : entity.lastSyncedAt,
    });

    setState(prev => ({
      ...prev,
      organizations: entityType === 'organization'
        ? prev.organizations.map(org => (org.id === entityId ? mark(org) : org))
        : prev.organizations,
      personalGroups: entityType === 'group'
        ? prev.personalGroups.map(group => (group.id === entityId ? mark(group) : group))
        : prev.personalGroups,
      personalUsers: entityType === 'contact'
        ? prev.personalUsers.map(user => (user.id === entityId ? mark(user) : user))
        : prev.personalUsers,
    }));
  }, []);

  const resetDirectory = useCallback(() => {
    setState(createInitialState());
  }, []);

  const queueCreateOperation = useCallback((entityType: EntitySyncEntityType, payload: Record<string, any>) => {
    enqueueOperation({
      id: `op-${nanoid(8)}`,
      entityType,
      operation: 'create',
      payload,
      timestamp: new Date().toISOString(),
      attempts: 0,
      status: 'pending',
    });
  }, [enqueueOperation]);

  const queueDeleteOperation = useCallback((entityType: EntitySyncEntityType, payload: any) => {
    enqueueOperation({
      id: `op-${nanoid(8)}`,
      entityType,
      operation: 'delete',
      payload,
      timestamp: new Date().toISOString(),
      attempts: 0,
      status: 'pending',
    });
  }, [enqueueOperation]);

  const purgeEntity = useCallback((entityType: EntitySyncEntityType, entityId: string) => {
    setState(prev => {
      switch (entityType) {
        case 'organization':
          return {
            ...prev,
            organizations: prev.organizations.filter(org => org.id !== entityId),
          };
        case 'group':
          return {
            ...prev,
            organizations: prev.organizations.map(org => ({
              ...org,
              groups: org.groups.filter(group => group.id !== entityId) as Array<Group & EntityMetadata>,
            })),
            personalGroups: prev.personalGroups.filter(group => group.id !== entityId),
          };
        case 'channel':
          return {
            ...prev,
            organizations: prev.organizations.map(org => ({
              ...org,
              channels: org.channels.filter(channel => channel.id !== entityId) as Array<Channel & EntityMetadata>,
            })),
          };
        case 'project':
          return {
            ...prev,
            organizations: prev.organizations.map(org => ({
              ...org,
              projects: org.projects.filter(project => project.id !== entityId) as Array<Project & EntityMetadata>,
            })),
          };
        case 'contact':
          return {
            ...prev,
            personalUsers: prev.personalUsers.filter(user => user.id !== entityId),
          };
        case 'message':
        case 'storage':
          return prev;
        default:
          return prev;
      }
    });
  }, []);

  const queueMessage = useCallback((payload: MessageOperationPayload) => {
    queueCreateOperation('message', payload);
  }, [queueCreateOperation]);

  // ============= Four-Words Validation =============
  const validateFourWords = useCallback(async (fourWords: string): Promise<FourWordsValidationResult> => {
    try {
      // First try local validation
      const isValid = await validateFourWordIdentity(fourWords);
      if (!isValid) {
        return {
          isValid: false,
          error: 'Invalid Four-Word format'
        };
      }

      // Try backend validation if available
      try {
        const backendValid = await invoke<boolean>('validate_four_words', { fourWords });
        if (!backendValid) {
          return {
            isValid: false,
            error: 'Four-Words not recognized by network'
          };
        }
      } catch {
        // Backend not available, local validation is enough for offline mode
      }

      const normalized = fourWords.trim().toLowerCase().replace(/\s+/g, '-');
      return {
        isValid: true,
        normalized
      };
    } catch (error) {
      return {
        isValid: false,
        error: error instanceof Error ? error.message : 'Validation failed'
      };
    }
  }, []);

  // ============= Create Operations (Generate New Four-Words) =============

  const createOrganization = useCallback(async (input: CreateNewOrganizationInput): Promise<EntityOperationResult> => {
    // In browser mode (no Tauri), allow offline creation
    // In Tauri mode, check connection but don't block creation
    const now = new Date();
    const tempId = `org-${nanoid(8)}`;

    // Generate new Four-Words for this organization
    const generatedFourWords = await generateFourWords();
    const networkIdentity = createNetworkIdentity(true, generatedFourWords); // isOwned = true

    const organization: Organization = {
      id: tempId,
      type: 'organization',
      name: input.displayName.trim(),
      description: input.description,
      networkIdentity,
      capabilities: { ...defaultCapabilities },
      owners: [],
      channels: [],
      groups: [],
      users: [],
      projects: [],
      settings: {
        allowGuestAccess: false,
        defaultChannelPermissions: [],
        websitePublishingEnabled: true,
      },
      createdAt: now,
      updatedAt: now,
    };

    const scaffoldChannels: Array<Channel & EntityMetadata> = ['general', 'announcements'].map((label, index) =>
      withMetadata({
        id: `channel-${nanoid(8)}`,
        type: 'channel',
        name: label,
        description: index === 0
          ? 'Organization-wide updates and coordination hub.'
          : 'Celebrate wins, broadcast timelines, and capture decisions.',
        organizationId: tempId,
        isPrivate: false,
        members: [],
        networkIdentity: createNetworkIdentity(true),
        capabilities: { ...defaultCapabilities },
        createdAt: now,
        updatedAt: now,
      })
    );

    const scaffoldProjects: Array<Project & EntityMetadata> = [
      {
        id: `project-${nanoid(8)}`,
        type: 'project' as const,
        name: `${input.displayName.trim()} Launch Plan`,
        description: 'Unified milestones to get your workspace live.',
        organizationId: tempId,
        leads: [],
        members: [],
        status: 'planning' as const,
        startDate: now,
        endDate: undefined,
        milestones: [
          {
            id: `milestone-${nanoid(6)}`,
            name: 'Kickoff & Alignment',
            description: 'Share vision, define success metrics, assign owners.',
            dueDate: new Date(now.getTime() + 1000 * 60 * 60 * 24 * 7),
            completed: false,
          },
        ],
        networkIdentity: createNetworkIdentity(true),
        capabilities: { ...defaultCapabilities },
        createdAt: now,
        updatedAt: now,
      },
      {
        id: `project-${nanoid(8)}`,
        type: 'project' as const,
        name: 'Async Collaboration Toolkit',
        description: 'Document rituals, templates, and automations to scale collaboration.',
        organizationId: tempId,
        leads: [],
        members: [],
        status: 'active' as const,
        startDate: now,
        endDate: undefined,
        milestones: [],
        networkIdentity: createNetworkIdentity(true),
        capabilities: { ...defaultCapabilities },
        createdAt: now,
        updatedAt: now,
      },
    ].map(project => withMetadata(project));

    organization.channels = scaffoldChannels;
    organization.projects = scaffoldProjects;

    const organizationWithMeta = withMetadata(organization);

    setState(prev => ({
      ...prev,
      organizations: [...prev.organizations, organizationWithMeta],
    }));

    queueCreateOperation('organization', organizationWithMeta);

    return {
      success: true,
      entityId: tempId,
      fourWords: networkIdentity.fourWords,
      isOwned: true,
      needsSync: true
    };
  }, [queueCreateOperation]);

  const createGroup = useCallback(async (input: CreateNewGroupInput): Promise<EntityOperationResult> => {
    // In browser mode (no Tauri), allow offline creation
    // In Tauri mode, check connection but don't block creation
    const now = new Date();
    const tempId = `group-${nanoid(8)}`;
    const generatedFourWords = await generateFourWords();
    const networkIdentity = createNetworkIdentity(true, generatedFourWords);

    const group: Group = {
      id: tempId,
      type: 'group',
      name: input.displayName.trim(),
      description: input.description,
      organizationId: input.organizationId,
      networkIdentity,
      capabilities: { ...defaultCapabilities },
      createdAt: now,
      updatedAt: now,
      members: [],
      admins: [],
      isPersonal: !input.organizationId,
    };

    const groupWithMeta = withMetadata(group);

    if (input.organizationId) {
      setState(prev => ({
        ...prev,
        organizations: prev.organizations.map(org =>
          org.id === input.organizationId
            ? { ...org, groups: [...org.groups, groupWithMeta], updatedAt: now }
            : org
        ),
      }));
    } else {
      setState(prev => ({
        ...prev,
        personalGroups: [...prev.personalGroups, groupWithMeta],
      }));
    }

    queueCreateOperation('group', groupWithMeta);

    return {
      success: true,
      entityId: tempId,
      fourWords: networkIdentity.fourWords,
      isOwned: true,
      needsSync: true
    };
  }, [queueCreateOperation]);

  const createChannel = useCallback(async (input: CreateNewChannelInput): Promise<EntityOperationResult> => {
    // In browser mode (no Tauri), allow offline creation
    // In Tauri mode, check connection but don't block creation
    const now = new Date();
    const tempId = `channel-${nanoid(8)}`;
    const generatedFourWords = await generateFourWords();
    const networkIdentity = createNetworkIdentity(true, generatedFourWords);

    const channel: Channel = {
      id: tempId,
      type: 'channel',
      name: input.displayName.trim(),
      description: input.description,
      organizationId: input.organizationId,
      isPrivate: Boolean(input.isPrivate),
      members: [],
      networkIdentity,
      capabilities: { ...defaultCapabilities },
      createdAt: now,
      updatedAt: now,
    };

    const channelWithMeta = withMetadata(channel);

    setState(prev => ({
      ...prev,
      organizations: prev.organizations.map(org =>
        org.id === input.organizationId
          ? { ...org, channels: [...org.channels, channelWithMeta], updatedAt: now }
          : org
      ),
    }));

    queueCreateOperation('channel', channelWithMeta);

    return {
      success: true,
      entityId: tempId,
      fourWords: networkIdentity.fourWords,
      isOwned: true,
      needsSync: true
    };
  }, [queueCreateOperation]);

  const createProject = useCallback(async (input: CreateNewProjectInput): Promise<EntityOperationResult> => {
    // In browser mode (no Tauri), allow offline creation
    // In Tauri mode, check connection but don't block creation
    const now = new Date();
    const tempId = `project-${nanoid(8)}`;
    const generatedFourWords = await generateFourWords();
    const networkIdentity = createNetworkIdentity(true, generatedFourWords);

    const project: Project = {
      id: tempId,
      type: 'project',
      name: input.displayName.trim(),
      description: input.description,
      organizationId: input.organizationId,
      leads: [],
      members: [],
      status: 'planning',
      networkIdentity,
      capabilities: { ...defaultCapabilities },
      createdAt: now,
      updatedAt: now,
      milestones: [],
    };

    const projectWithMeta = withMetadata(project);

    setState(prev => ({
      ...prev,
      organizations: prev.organizations.map(org =>
        org.id === input.organizationId
          ? { ...org, projects: [...org.projects, projectWithMeta], updatedAt: now }
          : org
      ),
    }));

    queueCreateOperation('project', projectWithMeta);

    return {
      success: true,
      entityId: tempId,
      fourWords: networkIdentity.fourWords,
      isOwned: true,
      needsSync: true
    };
  }, [queueCreateOperation]);


  // ============= Add Operations (Use Existing Four-Words) =============

  const addExistingOrganization = useCallback(async (input: AddExistingOrganizationInput): Promise<EntityOperationResult> => {
    // Validate Four-Words
    const validation = await validateFourWords(input.fourWords);
    if (!validation.isValid) {
      return {
        success: false,
        entityId: '',
        fourWords: input.fourWords,
        isOwned: false,
        needsSync: false,
        error: validation.error
      };
    }

    const now = new Date();
    const tempId = `org-ext-${nanoid(8)}`;
    const networkIdentity = createNetworkIdentity(false, validation.normalized); // isOwned = false

    const organization: Organization = {
      id: tempId,
      type: 'organization',
      name: input.displayName || validation.normalized!,
      description: `Joined organization: ${validation.normalized}`,
      networkIdentity,
      capabilities: { ...defaultCapabilities },
      owners: [],
      channels: [],
      groups: [],
      users: [],
      projects: [],
      settings: {
        allowGuestAccess: false,
        defaultChannelPermissions: [],
        websitePublishingEnabled: true,
      },
      createdAt: now,
      updatedAt: now,
    };

    const organizationWithMeta = withMetadata(organization, 'new');

    setState(prev => ({
      ...prev,
      organizations: [...prev.organizations, organizationWithMeta],
    }));

    // Queue operation to fetch details from network
    enqueueOperation({
      id: `fetch-${nanoid(8)}`,
      entityType: 'organization',
      operation: 'resolve',
      payload: { fourWords: validation.normalized, tempId },
      timestamp: new Date().toISOString(),
      attempts: 0,
      status: 'pending'
    });

    return {
      success: true,
      entityId: tempId,
      fourWords: validation.normalized!,
      isOwned: false,
      needsSync: true
    };
  }, [validateFourWords, enqueueOperation]);

  const addExistingGroup = useCallback(async (input: AddExistingGroupInput): Promise<EntityOperationResult> => {
    const validation = await validateFourWords(input.fourWords);
    if (!validation.isValid) {
      return {
        success: false,
        entityId: '',
        fourWords: input.fourWords,
        isOwned: false,
        needsSync: false,
        error: validation.error
      };
    }

    const now = new Date();
    const tempId = `group-ext-${nanoid(8)}`;
    const networkIdentity = createNetworkIdentity(false, validation.normalized);

    const group: Group = {
      id: tempId,
      type: 'group',
      name: input.displayName || validation.normalized!,
      description: `Joined group: ${validation.normalized}`,
      organizationId: input.organizationId,
      networkIdentity,
      capabilities: { ...defaultCapabilities },
      createdAt: now,
      updatedAt: now,
      members: [],
      admins: [],
      isPersonal: !input.organizationId,
    };

    const groupWithMeta = withMetadata(group, 'new');

    if (input.organizationId) {
      setState(prev => ({
        ...prev,
        organizations: prev.organizations.map(org =>
          org.id === input.organizationId
            ? { ...org, groups: [...org.groups, groupWithMeta], updatedAt: now }
            : org
        ),
      }));
    } else {
      setState(prev => ({
        ...prev,
        personalGroups: [...prev.personalGroups, groupWithMeta],
      }));
    }

    enqueueOperation({
      id: `fetch-${nanoid(8)}`,
      entityType: 'group',
      operation: 'resolve',
      payload: { fourWords: validation.normalized, tempId },
      timestamp: new Date().toISOString(),
      attempts: 0,
      status: 'pending'
    });

    return {
      success: true,
      entityId: tempId,
      fourWords: validation.normalized!,
      isOwned: false,
      needsSync: true
    };
  }, [validateFourWords, enqueueOperation]);

  const addExistingChannel = useCallback(async (input: AddExistingChannelInput): Promise<EntityOperationResult> => {
    const validation = await validateFourWords(input.fourWords);
    if (!validation.isValid) {
      return {
        success: false,
        entityId: '',
        fourWords: input.fourWords,
        isOwned: false,
        needsSync: false,
        error: validation.error
      };
    }

    const now = new Date();
    const tempId = `channel-ext-${nanoid(8)}`;
    const networkIdentity = createNetworkIdentity(false, validation.normalized);

    const channel: Channel = {
      id: tempId,
      type: 'channel',
      name: input.displayName || validation.normalized!,
      description: `Joined channel: ${validation.normalized}`,
      organizationId: input.organizationId,
      isPrivate: false,
      members: [],
      networkIdentity,
      capabilities: { ...defaultCapabilities },
      createdAt: now,
      updatedAt: now,
    };

    const channelWithMeta = withMetadata(channel, 'new');

    setState(prev => ({
      ...prev,
      organizations: prev.organizations.map(org =>
        org.id === input.organizationId
          ? { ...org, channels: [...org.channels, channelWithMeta], updatedAt: now }
          : org
      ),
    }));

    enqueueOperation({
      id: `fetch-${nanoid(8)}`,
      entityType: 'channel',
      operation: 'resolve',
      payload: { fourWords: validation.normalized, tempId },
      timestamp: new Date().toISOString(),
      attempts: 0,
      status: 'pending'
    });

    return {
      success: true,
      entityId: tempId,
      fourWords: validation.normalized!,
      isOwned: false,
      needsSync: true
    };
  }, [validateFourWords, enqueueOperation]);

  const addExistingProject = useCallback(async (input: AddExistingProjectInput): Promise<EntityOperationResult> => {
    const validation = await validateFourWords(input.fourWords);
    if (!validation.isValid) {
      return {
        success: false,
        entityId: '',
        fourWords: input.fourWords,
        isOwned: false,
        needsSync: false,
        error: validation.error
      };
    }

    const now = new Date();
    const tempId = `project-ext-${nanoid(8)}`;
    const networkIdentity = createNetworkIdentity(false, validation.normalized);

    const project: Project = {
      id: tempId,
      type: 'project',
      name: input.displayName || validation.normalized!,
      description: `Joined project: ${validation.normalized}`,
      organizationId: input.organizationId,
      leads: [],
      members: [],
      status: 'active',
      networkIdentity,
      capabilities: { ...defaultCapabilities },
      createdAt: now,
      updatedAt: now,
      milestones: [],
    };

    const projectWithMeta = withMetadata(project, 'new');

    setState(prev => ({
      ...prev,
      organizations: prev.organizations.map(org =>
        org.id === input.organizationId
          ? { ...org, projects: [...org.projects, projectWithMeta], updatedAt: now }
          : org
      ),
    }));

    enqueueOperation({
      id: `fetch-${nanoid(8)}`,
      entityType: 'project',
      operation: 'resolve',
      payload: { fourWords: validation.normalized, tempId },
      timestamp: new Date().toISOString(),
      attempts: 0,
      status: 'pending'
    });

    return {
      success: true,
      entityId: tempId,
      fourWords: validation.normalized!,
      isOwned: false,
      needsSync: true
    };
  }, [validateFourWords, enqueueOperation]);

  const addOrganization = useCallback(async (input: CreateOrganizationInput): Promise<EntityOperationResult> => {
    try {
      const now = new Date();
      const organization: Organization = {
        id: `org-${nanoid(8)}`,
        type: 'organization',
        name: input.name.trim(),
        description: input.description,
        networkIdentity: createNetworkIdentity(),
        capabilities: { ...defaultCapabilities },
        owners: [],
        channels: [],
        groups: [],
        users: [],
        projects: [],
        settings: {
          allowGuestAccess: false,
          defaultChannelPermissions: [],
          websitePublishingEnabled: true,
        },
        createdAt: now,
        updatedAt: now,
      };

      const organizationWithMeta = withMetadata(organization);

      setState(prev => ({
        ...prev,
        organizations: [...prev.organizations, organizationWithMeta],
      }));

      queueCreateOperation('organization', organizationWithMeta);

      return {
        success: true,
        entityId: organization.id,
        fourWords: organization.networkIdentity?.fourWords || '',
        isOwned: true,
        needsSync: true
      };
    } catch (error) {
      return {
        success: false,
        entityId: '',
        fourWords: '',
        isOwned: false,
        needsSync: false,
        error: error instanceof Error ? error.message : 'Failed to create organization'
      };
    }
  }, [queueCreateOperation, withMetadata]);

  const removeOrganization = useCallback((organizationId: string) => {
    let entityToDelete: Organization | null = null;

    setState(prev => {
      // Find the organization to delete
      const org = prev.organizations.find(o => o.id === organizationId);
      if (org) {
        entityToDelete = org;
      }

      return {
        ...prev,
        organizations: prev.organizations
          .map(org =>
            org.id === organizationId
              ? org.syncStatus === 'new'
                ? null
                : { ...org, syncStatus: 'deleted' }
              : org
          )
          .filter(Boolean) as Array<Organization & EntityMetadata>,
      };
    });

    if (entityToDelete) {
      queueDeleteOperation('organization', {
        id: organizationId,
        networkIdentity: entityToDelete.networkIdentity
      });
    }
  }, [queueDeleteOperation]);

  const addOrganizationGroup = useCallback((input: CreateGroupInput & { organizationId: string }): Group => {
    const now = new Date();
    const group: Group = {
      id: `group-${nanoid(8)}`,
      type: 'group',
      name: input.name.trim(),
      description: input.description,
      organizationId: input.organizationId,
      networkIdentity: createNetworkIdentity(),
      capabilities: { ...defaultCapabilities },
      createdAt: now,
      updatedAt: now,
      members: [],
      admins: [],
      isPersonal: false,
    };

    const groupWithMeta = withMetadata(group);

    setState(prev => ({
      ...prev,
      organizations: prev.organizations.map(org =>
        org.id === input.organizationId
          ? { ...org, groups: [...org.groups, groupWithMeta], updatedAt: now }
          : org
      ),
    }));

    queueCreateOperation('group', groupWithMeta);

    return groupWithMeta;
  }, [queueCreateOperation]);

  const removeOrganizationGroup = useCallback((organizationId: string, groupId: string) => {
    let entityToDelete: Group | null = null;

    setState(prev => {
      // Find the group to delete
      const org = prev.organizations.find(o => o.id === organizationId);
      if (org) {
        const group = org.groups.find(g => g.id === groupId);
        if (group) {
          entityToDelete = group;
        }
      }

      return {
        ...prev,
        organizations: prev.organizations.map(org =>
          org.id === organizationId
            ? {
                ...org,
                groups: org.groups
                  .map(group =>
                    group.id === groupId
                      ? group.syncStatus === 'new'
                        ? null
                        : { ...group, syncStatus: 'deleted' }
                      : group
                  )
                  .filter(Boolean) as Array<Group & EntityMetadata>,
                updatedAt: new Date(),
              }
            : org
        ),
      };
    });

    if (entityToDelete) {
      queueDeleteOperation('group', {
        id: groupId,
        networkIdentity: entityToDelete.networkIdentity
      });
    }
  }, [queueDeleteOperation]);

  const addOrganizationChannel = useCallback((input: CreateChannelInput): Channel => {
    const now = new Date();
    const channel: Channel = {
      id: `channel-${nanoid(8)}`,
      type: 'channel',
      name: input.name.trim(),
      description: input.description,
      organizationId: input.organizationId,
      isPrivate: Boolean(input.isPrivate),
      members: [],
      networkIdentity: createNetworkIdentity(),
      capabilities: { ...defaultCapabilities },
      createdAt: now,
      updatedAt: now,
    };

    const channelWithMeta = withMetadata(channel);

    setState(prev => ({
      ...prev,
      organizations: prev.organizations.map(org =>
        org.id === input.organizationId
          ? { ...org, channels: [...org.channels, channelWithMeta], updatedAt: now }
          : org
      ),
    }));

    queueCreateOperation('channel', channelWithMeta);

    return channelWithMeta;
  }, [queueCreateOperation]);

  const removeOrganizationChannel = useCallback((organizationId: string, channelId: string) => {
    setState(prev => ({
      ...prev,
      organizations: prev.organizations.map(org =>
        org.id === organizationId
          ? {
              ...org,
              channels: org.channels
                .map(channel =>
                  channel.id === channelId
                    ? channel.syncStatus === 'new'
                      ? null
                      : { ...channel, syncStatus: 'deleted' }
                    : channel
                )
                .filter(Boolean) as Array<Channel & EntityMetadata>,
              updatedAt: new Date(),
            }
          : org
      ),
    }));

    queueDeleteOperation('channel', channelId);
  }, [queueDeleteOperation]);

  const addProject = useCallback((input: CreateProjectInput): Project => {
    const now = new Date();
    const project: Project = {
      id: `project-${nanoid(8)}`,
      type: 'project',
      name: input.name.trim(),
      description: input.description,
      organizationId: input.organizationId,
      leads: [],
      members: [],
      status: 'planning',
      networkIdentity: createNetworkIdentity(),
      capabilities: { ...defaultCapabilities },
      createdAt: now,
      updatedAt: now,
      milestones: [],
    };

    const projectWithMeta = withMetadata(project);

    setState(prev => ({
      ...prev,
      organizations: prev.organizations.map(org =>
        org.id === input.organizationId
          ? { ...org, projects: [...org.projects, projectWithMeta], updatedAt: now }
          : org
      ),
    }));

    queueCreateOperation('project', projectWithMeta);

    return projectWithMeta;
  }, [queueCreateOperation]);

  const removeProject = useCallback((organizationId: string, projectId: string) => {
    setState(prev => ({
      ...prev,
      organizations: prev.organizations.map(org =>
        org.id === organizationId
          ? {
              ...org,
              projects: org.projects
                .map(project =>
                  project.id === projectId
                    ? project.syncStatus === 'new'
                      ? null
                      : { ...project, syncStatus: 'deleted' }
                    : project
                )
                .filter(Boolean) as Array<Project & EntityMetadata>,
              updatedAt: new Date(),
            }
          : org
      ),
    }));

    queueDeleteOperation('project', projectId);
  }, [queueDeleteOperation]);

  const addPersonalGroup = useCallback(async (input: CreateGroupInput): Promise<EntityOperationResult> => {
    try {
      const now = new Date();
      const group: Group = {
        id: `personal-group-${nanoid(8)}`,
        type: 'group',
        name: input.name.trim(),
        description: input.description,
        networkIdentity: createNetworkIdentity(),
        capabilities: { ...defaultCapabilities },
        createdAt: now,
        updatedAt: now,
        members: [],
        admins: [],
        isPersonal: true,
      };

      const groupWithMeta = withMetadata(group);

      setState(prev => ({
        ...prev,
        personalGroups: [...prev.personalGroups, groupWithMeta],
      }));

      queueCreateOperation('group', groupWithMeta);

      return {
        success: true,
        entityId: group.id,
        fourWords: group.networkIdentity?.fourWords || '',
        isOwned: true,
        needsSync: true
      };
    } catch (error) {
      return {
        success: false,
        entityId: '',
        fourWords: '',
        isOwned: false,
        needsSync: false,
        error: error instanceof Error ? error.message : 'Failed to create group'
      };
    }
  }, [queueCreateOperation, withMetadata]);

  const removePersonalGroup = useCallback((groupId: string) => {
    setState(prev => ({
      ...prev,
      personalGroups: prev.personalGroups
        .map(group =>
          group.id === groupId
            ? group.syncStatus === 'new'
              ? null
              : { ...group, syncStatus: 'deleted' }
            : group
        )
        .filter(Boolean) as Array<Group & EntityMetadata>,
    }));

    queueDeleteOperation('group', groupId);
  }, [queueDeleteOperation]);

  const createContact = useCallback(async (input: CreateNewContactInput): Promise<EntityOperationResult> => {
    try {
      // Try to generate real four-words from saorsa-core
      let fourWords: string;
      try {
        const generated = await invoke<string>('generate_four_word_identity');
        if (!generated) {
          throw new Error('No four-words generated');
        }
        fourWords = generated;
      } catch (error) {
        // Fallback to temp four-words if generation fails (offline)
        fourWords = `temp-${nanoid(8)}`;
        console.log('Offline mode: using temporary four-words', fourWords);
      }

      const now = new Date();
      const entityId = `contact-${nanoid(8)}`;

      const user: PersonalUser = {
        id: entityId,
        type: 'personal_user',
        name: input.displayName.trim(),
        description: input.email,
        userId: `user-${nanoid(8)}`,
        relationship: input.relationship ?? 'colleague',
        networkIdentity: {
          fourWords,
          publicKey: `pk_${nanoid(12)}`,
          dhtAddress: `dht://${fourWords.replace(/-/g, '')}-${nanoid(6)}`,
          isOwned: true,
          isValidated: !fourWords.startsWith('temp-'), // Only validated if real four-words
        },
        capabilities: { ...defaultCapabilities },
        createdAt: now,
        updatedAt: now,
        lastContact: now,
      };

      // Mark as 'new' since we just created it (needs sync)
      const userWithMeta = withMetadata(user, 'new');

      setState(prev => ({
        ...prev,
        personalUsers: [...prev.personalUsers, userWithMeta],
      }));

      // Queue for sync to create on network
      queueCreateOperation('contact', userWithMeta);

      return {
        success: true,
        entityId,
        fourWords,
        isOwned: true,
        needsSync: true,
      };
    } catch (error) {
      console.error('createContact error:', error);
      return {
        success: false,
        entityId: '',
        fourWords: '',
        isOwned: true,
        needsSync: false,
        error: error instanceof Error ? error.message : 'Failed to create contact',
      };
    }
  }, [queueCreateOperation]);

  const addPersonalUser = useCallback((input: CreatePersonalUserInput): PersonalUser => {
    const now = new Date();
    const user: PersonalUser = {
      id: `contact-${nanoid(8)}`,
      type: 'personal_user',
      name: input.name.trim(),
      description: input.email,
      userId: `user-${nanoid(8)}`,
      relationship: input.relationship ?? 'colleague',
      networkIdentity: createNetworkIdentity(),
      capabilities: { ...defaultCapabilities },
      createdAt: now,
      updatedAt: now,
      lastContact: now,
    };

    const userWithMeta = withMetadata(user);

    setState(prev => ({
      ...prev,
      personalUsers: [...prev.personalUsers, userWithMeta],
    }));

    queueCreateOperation('contact', userWithMeta);

    return userWithMeta;
  }, [queueCreateOperation]);

  const addExistingContact = useCallback(async (input: AddExistingContactInput): Promise<EntityOperationResult> => {
    try {
      // Normalize four-words input
      const normalizedFourWords = input.fourWords.trim().toLowerCase().replace(/\s+/g, '-');

      // Validate four-word format
      const validationResult = await validateFourWords(normalizedFourWords);
      if (!validationResult.isValid) {
        return {
          success: false,
          entityId: '',
          fourWords: normalizedFourWords,
          isOwned: false,
          needsSync: false,
          error: validationResult.error || 'Invalid Four-Word format',
        };
      }

      // Fetch identity from DHT
      interface DHTIdentity {
        id: string;
        four_words: string;
        display_name: string;
        public_key: string;
        dht_address: string;
        bio?: string;
        avatar_url?: string;
      }

      const dhtIdentity = await invoke<DHTIdentity>('core_fetch_identity', {
        fourWords: normalizedFourWords,
      });

      // Create PersonalUser from DHT data
      const now = new Date();
      const user: PersonalUser = {
        id: `contact-${nanoid(8)}`,
        type: 'personal_user',
        name: input.displayName || dhtIdentity.display_name,
        description: dhtIdentity.bio,
        userId: dhtIdentity.id,
        relationship: input.relationship ?? 'colleague',
        networkIdentity: {
          fourWords: dhtIdentity.four_words,
          publicKey: dhtIdentity.public_key,
          dhtAddress: dhtIdentity.dht_address,
          isOwned: false,
          isValidated: true,
        },
        capabilities: { ...defaultCapabilities },
        createdAt: now,
        updatedAt: now,
        lastContact: now,
      };

      // Mark as synced (not new) since we fetched from DHT
      const userWithMeta = withMetadata(user, 'synced');

      // Check for duplicates and add to state atomically
      let wasAdded = false;
      setState(prev => {
        // Check if contact already exists
        const existingContact = prev.personalUsers.find(
          u => u.networkIdentity.fourWords === dhtIdentity.four_words
        );

        if (existingContact) {
          // Don't add duplicate
          wasAdded = false;
          return prev;
        }

        // Add new contact
        wasAdded = true;
        return {
          ...prev,
          personalUsers: [...prev.personalUsers, userWithMeta],
        };
      });

      if (!wasAdded) {
        return {
          success: false,
          entityId: '',
          fourWords: dhtIdentity.four_words,
          isOwned: false,
          needsSync: false,
          error: 'Contact already exists with these four-words',
        };
      }

      return {
        success: true,
        entityId: user.id,
        fourWords: dhtIdentity.four_words,
        isOwned: false,
        needsSync: false,
      };
    } catch (error) {
      return {
        success: false,
        entityId: '',
        fourWords: input.fourWords,
        isOwned: false,
        needsSync: false,
        error: error instanceof Error ? error.message : 'Failed to add existing contact',
      };
    }
  }, [validateFourWords]);

  const removePersonalUser = useCallback((userId: string) => {
    setState(prev => ({
      ...prev,
      personalUsers: prev.personalUsers
        .map(user =>
          user.id === userId
            ? user.syncStatus === 'new'
              ? null
              : { ...user, syncStatus: 'deleted' }
            : user
        )
        .filter(Boolean) as Array<PersonalUser & EntityMetadata>,
    }));

    queueDeleteOperation('contact', userId);
  }, [queueDeleteOperation]);

  // ============= Group Member Management =============

  const addGroupMember = useCallback(async (groupId: string, userId: string, role: string = 'member'): Promise<void> => {
    try {
      // Call backend Tauri command
      await invoke('add_group_member', {
        groupId,
        userId,
        role
      });

      // Update local state to reflect the new member
      setState(prev => ({
        ...prev,
        organizations: prev.organizations.map(org => ({
          ...org,
          groups: org.groups.map(group =>
            group.id === groupId
              ? {
                  ...group,
                  members: [...group.members, userId],
                  syncStatus: 'synced' as SyncStatus,
                  updatedAt: new Date(),
                }
              : group
          ),
        })),
        personalGroups: prev.personalGroups.map(group =>
          group.id === groupId
            ? {
                ...group,
                members: [...group.members, userId],
                syncStatus: 'synced' as SyncStatus,
                updatedAt: new Date(),
              }
            : group
        ),
      }));
    } catch (error) {
      console.error('Failed to add group member:', error);
      throw error;
    }
  }, []);

  const removeGroupMember = useCallback(async (groupId: string, userId: string): Promise<void> => {
    try {
      // Call backend Tauri command
      await invoke('remove_group_member', {
        groupId,
        userId
      });

      // Update local state to remove the member
      setState(prev => ({
        ...prev,
        organizations: prev.organizations.map(org => ({
          ...org,
          groups: org.groups.map(group =>
            group.id === groupId
              ? {
                  ...group,
                  members: group.members.filter(id => id !== userId),
                  syncStatus: 'synced' as SyncStatus,
                  updatedAt: new Date(),
                }
              : group
          ),
        })),
        personalGroups: prev.personalGroups.map(group =>
          group.id === groupId
            ? {
                ...group,
                members: group.members.filter(id => id !== userId),
                syncStatus: 'synced' as SyncStatus,
                updatedAt: new Date(),
              }
            : group
        ),
      }));
    } catch (error) {
      console.error('Failed to remove group member:', error);
      throw error;
    }
  }, []);

  const getGroupMembers = useCallback(async (groupId: string): Promise<Array<{ userId: string; role: string }>> => {
    try {
      // Call backend Tauri command
      const members = await invoke<Array<[string, string]>>('get_group_members', {
        groupId
      });

      // Convert tuple array to object array
      return members.map(([userId, role]) => ({ userId, role }));
    } catch (error) {
      console.error('Failed to get group members:', error);
      throw error;
    }
  }, []);

  // Backend sync operations using saorsa-core
  const syncEntityToBackend = useCallback(async (entityType: EntitySyncEntityType, entity: any): Promise<string | null> => {
    try {
      // Check if we have network connectivity
      const isOnline = navigator.onLine;
      if (!isOnline) {
        console.log('Offline mode: queuing operation for later sync');
        return null;
      }

      // Sync based on entity type using saorsa-core API
      switch (entityType) {
        case 'organization': {
          // Create org identity with Four-Words
          const identityHex = await invoke<string>('core_new_four_word_identity', {
            fourWords: entity.networkIdentity.fourWords,
            displayName: entity.name
          });

          // Store org metadata in DHT
          await invoke('core_dht_put', {
            key: entity.networkIdentity.fourWords,
            value: JSON.stringify({
              type: 'organization',
              name: entity.name,
              description: entity.description,
              identityHex,
              createdAt: entity.createdAt.toISOString()
            })
          });

          return identityHex;
        }

        case 'group': {
          // Create group identity
          const groupHex = await invoke<string>('core_create_group', {
            displayName: entity.name,
            description: entity.description || '',
            isPublic: !entity.isPersonal
          });

          // Store in DHT
          await invoke('core_dht_put', {
            key: entity.networkIdentity.fourWords,
            value: JSON.stringify({
              type: 'group',
              name: entity.name,
              groupHex,
              organizationId: entity.organizationId,
              isPersonal: entity.isPersonal
            })
          });

          return groupHex;
        }

        case 'channel': {
          // Channels are part of groups in saorsa-core
          const channelData = {
            type: 'channel',
            name: entity.name,
            description: entity.description,
            organizationId: entity.organizationId,
            isPrivate: entity.isPrivate
          };

          await invoke('core_dht_put', {
            key: entity.networkIdentity.fourWords,
            value: JSON.stringify(channelData)
          });

          return entity.networkIdentity.fourWords;
        }

        case 'contact': {
          // Add contact identity
          const contactHex = await invoke<string>('core_add_contact', {
            fourWords: entity.networkIdentity.fourWords,
            displayName: entity.name
          });

          return contactHex;
        }

        case 'message': {
          // Send message via saorsa messaging
          const payload = entity as MessageOperationPayload;

          // Get current user for author_id
          if (!authState.user?.id) {
            console.error('Cannot send message: No authenticated user');
            return null;
          }

          switch (payload.entityType) {
            case 'channel': {
              // Use working send_message command from org_commands.rs
              await invoke('send_message', {
                request: {
                  channel_id: payload.entityId,
                  author_id: authState.user.id,
                  content: payload.content,
                  thread_id: null, // null for main channel messages
                },
              });
              return payload.id;
            }
            default: {
              console.warn(`Send not implemented for message entity type: ${payload.entityType}`);
              return null;
            }
          }
        }

        default:
          console.warn(`Sync not implemented for entity type: ${entityType}`);
          return null;
      }
    } catch (error) {
      console.error(`Failed to sync ${entityType} to backend:`, error);
      throw error;
    }
  }, [authState]);

  // Conflict resolution strategies
  type ConflictResolution = 'local' | 'remote' | 'merge' | 'manual';

  const resolveConflict = useCallback((localEntity: any, remoteEntity: any, strategy: ConflictResolution = 'remote'): any => {
    // Compare timestamps
    const localTime = new Date(localEntity.updatedAt || localEntity.createdAt).getTime();
    const remoteTime = new Date(remoteEntity.updatedAt || remoteEntity.createdAt).getTime();

    switch (strategy) {
      case 'local':
        // Keep local changes
        console.log('Conflict resolution: keeping local changes');
        return { ...localEntity, syncStatus: 'dirty' };

      case 'remote':
        // Accept remote changes (default)
        console.log('Conflict resolution: accepting remote changes');
        return { ...remoteEntity, syncStatus: 'synced' };

      case 'merge':
        // Merge changes (last-write-wins for conflicting fields)
        console.log('Conflict resolution: merging changes');
        if (localTime > remoteTime) {
          // Local is newer, preserve local changes but mark as dirty
          return {
            ...remoteEntity,
            ...localEntity,
            syncStatus: 'dirty',
            conflictResolved: new Date().toISOString()
          };
        } else {
          // Remote is newer, take remote changes
          return {
            ...localEntity,
            ...remoteEntity,
            syncStatus: 'synced',
            conflictResolved: new Date().toISOString()
          };
        }

      case 'manual':
        // Mark for manual resolution
        console.log('Conflict resolution: marking for manual resolution');
        return {
          ...localEntity,
          syncStatus: 'conflict',
          conflictData: {
            local: localEntity,
            remote: remoteEntity,
            detectedAt: new Date().toISOString()
          }
        };

      default:
        // Default to remote
        return remoteEntity;
    }
  }, []);

  // Check for version conflicts
  const detectConflict = useCallback((localEntity: any, remoteEntity: any): boolean => {
    // If no local entity, no conflict
    if (!localEntity) return false;

    // If sync status is 'new', it hasn't been synced yet
    if (localEntity.syncStatus === 'new') return false;

    // Compare version fields if available
    if (localEntity.version && remoteEntity.version) {
      return localEntity.version !== remoteEntity.version;
    }

    // Compare timestamps
    const localTime = new Date(localEntity.updatedAt || localEntity.createdAt).getTime();
    const remoteTime = new Date(remoteEntity.updatedAt || remoteEntity.createdAt).getTime();

    // If local has been modified after last sync
    const lastSync = localEntity.lastSyncedAt ? new Date(localEntity.lastSyncedAt).getTime() : 0;
    if (localTime > lastSync && remoteTime > lastSync) {
      // Both modified since last sync = conflict
      return true;
    }

    return false;
  }, []);

  // Resolve existing entities from the network
  const resolveEntityFromNetwork = useCallback(async (entityType: EntitySyncEntityType, fourWords: string): Promise<any> => {
    try {
      // Validate Four-Words first
      const isValid = await invoke<boolean>('validate_four_words', { fourWords });
      if (!isValid) {
        throw new Error('Invalid Four-Words identifier');
      }

      // Get entity data from DHT
      const entityData = await invoke<string>('core_dht_get', { key: fourWords });
      if (!entityData) {
        throw new Error('Entity not found in network');
      }

      const parsed = JSON.parse(entityData);

      // Validate entity type matches
      if (parsed.type !== entityType &&
          !(entityType === 'contact' && parsed.type === 'user')) {
        throw new Error(`Entity type mismatch: expected ${entityType}, got ${parsed.type}`);
      }

      // Check for conflicts with local entity
      const localEntity = findLocalEntityByFourWords(entityType, fourWords);
      if (localEntity && detectConflict(localEntity, parsed)) {
        // Resolve conflict (default to remote for now)
        const resolved = resolveConflict(localEntity, parsed, 'merge');
        return resolved;
      }

      return parsed;
    } catch (error) {
      console.error(`Failed to resolve ${entityType} from network:`, error);
      throw error;
    }
  }, [detectConflict, resolveConflict]);

  // Helper to find local entity by Four-Words
  const findLocalEntityByFourWords = useCallback((entityType: EntitySyncEntityType, fourWords: string): any => {
    const normalizedFourWords = fourWords.toLowerCase().trim();

    switch (entityType) {
      case 'organization':
        return state.organizations.find(org =>
          org.networkIdentity.fourWords.toLowerCase() === normalizedFourWords
        );
      case 'group':
        // Check both org groups and personal groups
        for (const org of state.organizations) {
          const group = org.groups.find(g =>
            g.networkIdentity.fourWords.toLowerCase() === normalizedFourWords
          );
          if (group) return group;
        }
        return state.personalGroups.find(g =>
          g.networkIdentity.fourWords.toLowerCase() === normalizedFourWords
        );
      case 'channel':
        for (const org of state.organizations) {
          const channel = org.channels.find(c =>
            c.networkIdentity.fourWords.toLowerCase() === normalizedFourWords
          );
          if (channel) return channel;
        }
        return null;
      case 'project':
        for (const org of state.organizations) {
          const project = org.projects.find(p =>
            p.networkIdentity.fourWords.toLowerCase() === normalizedFourWords
          );
          if (project) return project;
        }
        return null;
      case 'contact':
        return state.personalUsers.find(u =>
          u.networkIdentity.fourWords.toLowerCase() === normalizedFourWords
        );
      default:
        return null;
    }
  }, [state]);

  const handleOperation = useCallback(async (operation: EntitySyncOperation) => {
    switch (operation.operation) {
      case 'create': {
        try {
          // Sync to backend
          const backendId = await syncEntityToBackend(operation.entityType, operation.payload);

          if (backendId) {
            // Update local entity with backend ID
            setEntityStatus(operation.entityType, operation.payload.id, 'synced');

            // Store ID mapping for future reference
            if (operation.payload.id !== backendId) {
              markOperationComplete(operation.id, { [operation.payload.id]: backendId });
            } else {
              markOperationComplete(operation.id);
            }
          } else {
            // Offline or sync deferred
            setEntityStatus(operation.entityType, operation.payload.id, 'new');
            // Keep operation in queue for later retry
          }
        } catch (error) {
          markOperationFailed(operation.id, error instanceof Error ? error.message : 'Sync failed');
          setEntityStatus(operation.entityType, operation.payload.id, 'error');
        }
        break;
      }

      case 'delete': {
        try {
          if (operation.entityType === 'message') {
            const payload = operation.payload as MessageOperationPayload;
            await removeCachedMessage(payload.entityType, payload.entityId, payload.id);
          } else {
            // Local-first deletion: purge from local storage
            // CRDT sync will propagate deletion to peers via gossip networking
            purgeEntity(operation.entityType, operation.payload.id);
          }
          markOperationComplete(operation.id);
        } catch (error) {
          markOperationFailed(operation.id, error instanceof Error ? error.message : 'Delete failed');
        }
        break;
      }

      case 'update': {
        try {
          // Sync update to backend (local storage + CRDT)
          await syncEntityToBackend(operation.entityType, operation.payload);
          setEntityStatus(operation.entityType, operation.payload.id, 'synced');
          markOperationComplete(operation.id);
        } catch (error) {
          markOperationFailed(operation.id, error instanceof Error ? error.message : 'Update failed');
          setEntityStatus(operation.entityType, operation.payload.id, 'error');
        }
        break;
      }

      case 'resolve': {
        try {
          // Resolve entity from network (for "Add existing" operations)
          const networkEntity = await resolveEntityFromNetwork(
            operation.entityType,
            operation.payload.fourWords
          );

          // Update local entity with network data
          const tempId = operation.payload.tempId;
          if (tempId && networkEntity) {
            // Map temporary ID to real network ID
            markOperationComplete(operation.id, { [tempId]: networkEntity.id || networkEntity.identityHex });
            setEntityStatus(operation.entityType, tempId, 'synced');
          } else {
            markOperationComplete(operation.id);
          }
        } catch (error) {
          markOperationFailed(operation.id, error instanceof Error ? error.message : 'Resolve failed');
          setEntityStatus(operation.entityType, operation.payload.tempId, 'error');
        }
        break;
      }

      default:
        markOperationFailed(operation.id, `Unsupported operation ${operation.operation}`);
        break;
    }
  }, [markOperationComplete, markOperationFailed, purgeEntity, setEntityStatus, syncEntityToBackend, resolveEntityFromNetwork]);

  useEffect(() => {
    if (typeof window !== 'undefined' && !navigator.onLine) {
      return;
    }

    const nextOperation = state.operations.find(op =>
      op.status === 'pending' || (op.status === 'failed' && op.attempts < MAX_OPERATION_ATTEMPTS)
    );

    if (!nextOperation) {
      return;
    }

    let cancelled = false;
    let processing = false;

    const process = async () => {
      if (cancelled || processing) return;
      processing = true;

      startOperationProcessing(nextOperation.id);
      try {
        await handleOperation(nextOperation);
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Synchronization failed';
        markOperationFailed(nextOperation.id, message);
      } finally {
        processing = false;
      }
    };

    // Debounce to prevent rapid re-execution
    const timer = setTimeout(process, 100);

    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [state.operations, handleOperation, markOperationFailed, startOperationProcessing]);

  const value = useMemo<EntityDirectoryContextValue>(() => ({
    organizations: state.organizations
      .filter(org => org.syncStatus !== 'deleted')
      .map(org => ({
        ...org,
        groups: org.groups.filter(g => g.syncStatus !== 'deleted'),
        channels: org.channels.filter(c => c.syncStatus !== 'deleted'),
        projects: org.projects.filter(p => p.syncStatus !== 'deleted'),
      })),
    personalGroups: state.personalGroups.filter(group => group.syncStatus !== 'deleted'),
    personalUsers: state.personalUsers.filter(user => user.syncStatus !== 'deleted'),
    operations: state.operations,
    // New create/add methods
    createOrganization,
    createGroup,
    createChannel,
    createProject,
    createContact,
    addExistingOrganization,
    addExistingGroup,
    addExistingChannel,
    addExistingProject,
    addExistingContact,
    validateFourWords,
    // Group member management
    addGroupMember,
    removeGroupMember,
    getGroupMembers,
    // Legacy methods
    addOrganization,
    removeOrganization,
    addOrganizationGroup,
    removeOrganizationGroup,
    addOrganizationChannel,
    removeOrganizationChannel,
    addProject,
    removeProject,
    addPersonalGroup,
    removePersonalGroup,
    addPersonalUser,
    removePersonalUser,
    resetDirectory,
    enqueueOperation,
    markOperationComplete,
    markOperationFailed,
    setEntityStatus,
    queueMessage,
  }), [
    state.organizations,
    state.personalGroups,
    state.personalUsers,
    state.operations,
    createOrganization,
    createGroup,
    createChannel,
    createProject,
    createContact,
    addExistingOrganization,
    addExistingGroup,
    addExistingChannel,
    addExistingProject,
    addExistingContact,
    validateFourWords,
    addGroupMember,
    removeGroupMember,
    getGroupMembers,
    addOrganization,
    removeOrganization,
    addOrganizationGroup,
    removeOrganizationGroup,
    addOrganizationChannel,
    removeOrganizationChannel,
    addProject,
    removeProject,
    addPersonalGroup,
    removePersonalGroup,
    addPersonalUser,
    removePersonalUser,
    resetDirectory,
    enqueueOperation,
    markOperationComplete,
    markOperationFailed,
    setEntityStatus,
    queueMessage,
  ]);

  return (
    <EntityDirectoryContext.Provider value={value}>
      {children}
    </EntityDirectoryContext.Provider>
  );
};

export const useEntityDirectory = () => {
  const context = useContext(EntityDirectoryContext);
  if (!context) {
    throw new Error('useEntityDirectory must be used within an EntityDirectoryProvider');
  }
  return context;
};
