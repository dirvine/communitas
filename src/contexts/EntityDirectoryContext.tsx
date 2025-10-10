import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  ReactNode,
} from 'react';
import { nanoid } from 'nanoid';

import {
  Organization,
  Group,
  PersonalUser,
  Channel,
  Project,
  CollaborationCapabilities,
  NetworkIdentity,
} from '../types/collaboration';
import {
  CreateNewOrganizationInput,
  CreateNewGroupInput,
  CreateNewChannelInput,
  CreateNewProjectInput,
  CreateNewContactInput,
  AddExistingOrganizationInput,
  AddExistingGroupInput,
  AddExistingChannelInput,
  AddExistingProjectInput,
  AddExistingContactInput,
  EntityOperationResult,
  FourWordsValidationResult,
} from '../types/entityOperations';
import { mockOrganizations, mockPersonalGroups, mockPersonalUsers } from '../data/mockCollaborationData';
import { fourWordsToStorage, fourWordsToDisplay } from '../utils/identity';

type SyncStatus = 'synced' | 'new' | 'dirty' | 'deleted' | 'error';

interface EntityMetadata {
  syncStatus: SyncStatus;
  lastSyncedAt?: number;
  error?: string;
}

interface EntitySyncOperation {
  id: string;
  entityType: 'organization' | 'group' | 'channel' | 'project' | 'contact' | 'message' | 'storage';
  operation: 'create' | 'update' | 'delete' | 'resolve';
  payload: Record<string, unknown>;
  timestamp: number;
  status: 'pending' | 'completed' | 'failed';
  attempts: number;
  error?: string;
}

interface EntityDirectoryContextValue {
  organizations: Array<Organization & EntityMetadata>;
  personalGroups: Array<Group & EntityMetadata>;
  personalUsers: Array<PersonalUser & EntityMetadata>;
  operations: EntitySyncOperation[];
  createOrganization: (input: CreateNewOrganizationInput) => Promise<EntityOperationResult>;
  createGroup: (input: CreateNewGroupInput) => Promise<EntityOperationResult>;
  createChannel: (input: CreateNewChannelInput) => Promise<EntityOperationResult>;
  createProject: (input: CreateNewProjectInput) => Promise<EntityOperationResult>;
  createContact: (input: CreateNewContactInput) => Promise<EntityOperationResult>;
  addExistingOrganization: (input: AddExistingOrganizationInput) => Promise<EntityOperationResult>;
  addExistingGroup: (input: AddExistingGroupInput) => Promise<EntityOperationResult>;
  addExistingChannel: (input: AddExistingChannelInput) => Promise<EntityOperationResult>;
  addExistingProject: (input: AddExistingProjectInput) => Promise<EntityOperationResult>;
  addExistingContact: (input: AddExistingContactInput) => Promise<EntityOperationResult>;
  validateFourWords: (fourWords: string) => Promise<FourWordsValidationResult>;
  addOrganization: (input: { name: string; description?: string }) => Promise<EntityOperationResult>;
  removeOrganization: (organizationId: string) => void;
  addOrganizationGroup: (input: { organizationId: string; name: string; description?: string }) => Group;
  removeOrganizationGroup: (organizationId: string, groupId: string) => void;
  addOrganizationChannel: (input: { organizationId: string; name: string; description?: string; isPrivate?: boolean }) => Channel;
  removeOrganizationChannel: (organizationId: string, channelId: string) => void;
  addProject: (input: { organizationId: string; name: string; description?: string }) => Project;
  removeProject: (organizationId: string, projectId: string) => void;
  addPersonalGroup: (input: { name: string; description?: string }) => Promise<EntityOperationResult>;
  removePersonalGroup: (groupId: string) => void;
  addPersonalUser: (input: { name: string; relationship?: PersonalUser['relationship']; email?: string }) => PersonalUser;
  removePersonalUser: (userId: string) => void;
  resetDirectory: () => void;
  enqueueOperation: (operation: EntitySyncOperation) => void;
  markOperationComplete: (operationId: string, newIdMap?: Record<string, string>) => void;
  markOperationFailed: (operationId: string, error: string) => void;
  setEntityStatus: (entityType: EntitySyncOperation['entityType'], entityId: string, status: SyncStatus) => void;
  queueMessage: (payload: { id: string; entityId: string; entityType: EntitySyncOperation['entityType']; content: string; timestamp: string }) => void;
}

const STORAGE_KEY = 'communitas.entity-directory.v2';

const DEFAULT_CAPABILITIES: CollaborationCapabilities = {
  audioCall: true,
  videoCall: true,
  screenShare: true,
  fileShare: true,
  websitePublish: true,
};

const createNetworkIdentity = (fourWords?: string, isOwned = true): NetworkIdentity => {
  const normalized = fourWords ? fourWordsToStorage(fourWords) : fourWordsToStorage(generateFourWords());
  return {
    fourWords: normalized,
    publicKey: `pk_${nanoid(12)}`,
    dhtAddress: `dht://${nanoid(16)}`,
    isOwned,
    isValidated: true,
  };
};

const wrapOrganization = (org: Organization): any => ({
  ...org,
  syncStatus: 'synced' as const,
  lastSyncedAt: Date.now(),
});

const wrapGroup = (group: Group): any => ({
  ...group,
  syncStatus: 'synced' as const,
  lastSyncedAt: Date.now(),
});

const wrapPersonalUser = (user: PersonalUser): any => ({
  ...user,
  syncStatus: 'synced' as const,
  lastSyncedAt: Date.now(),
});

const normaliseMockOrg = (org: Organization): Organization & EntityMetadata => {
  const normalisedChannels = (org.channels ?? []).map(channel => ({
    ...channel,
    networkIdentity: createNetworkIdentity(fourWordsToStorage(channel.networkIdentity?.fourWords ?? generateFourWords())),
  }));

  const normalisedGroups = (org.groups ?? []).map(group => ({
    ...group,
    networkIdentity: createNetworkIdentity(fourWordsToStorage(group.networkIdentity?.fourWords ?? generateFourWords())),
  }));

  const normalisedProjects = (org.projects ?? []).map(project => ({
    ...project,
    networkIdentity: createNetworkIdentity(fourWordsToStorage(project.networkIdentity?.fourWords ?? generateFourWords())),
  }));

  return wrapOrganization({
    ...org,
    networkIdentity: createNetworkIdentity(fourWordsToStorage(org.networkIdentity?.fourWords ?? generateFourWords())),
    channels: normalisedChannels,
    groups: normalisedGroups,
    projects: normalisedProjects,
  });
};

const normaliseMockGroup = (group: Group): Group & EntityMetadata =>
  wrapGroup({
    ...group,
    networkIdentity: createNetworkIdentity(fourWordsToStorage(group.networkIdentity?.fourWords ?? generateFourWords())),
  });

const normaliseMockUser = (user: PersonalUser): PersonalUser & EntityMetadata =>
  wrapPersonalUser({
    ...user,
    networkIdentity: createNetworkIdentity(fourWordsToStorage(user.networkIdentity?.fourWords ?? generateFourWords())),
  });

const generateFourWords = () => {
  const words = [
    ['ocean', 'forest', 'prairie', 'valley', 'desert', 'island', 'sunset', 'harbor'],
    ['bright', 'golden', 'silver', 'crystal', 'ember', 'shadow', 'morning', 'midnight'],
    ['hawk', 'otter', 'lynx', 'sparrow', 'storm', 'firefly', 'orca', 'aurora'],
    ['star', 'moon', 'nova', 'cloud', 'rain', 'wind', 'flame', 'glow'],
  ];

  return words
    .map(group => group[Math.floor(Math.random() * group.length)])
    .join('-');
};

type DirectoryState = {
  organizations: Array<Organization & EntityMetadata>;
  personalGroups: Array<Group & EntityMetadata>;
  personalUsers: Array<PersonalUser & EntityMetadata>;
};

const loadState = (): DirectoryState => {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return {
        organizations: mockOrganizations.map(normaliseMockOrg),
        personalGroups: mockPersonalGroups.map(normaliseMockGroup),
        personalUsers: mockPersonalUsers.map(normaliseMockUser),
      };
    }
    const parsed = JSON.parse(raw) as DirectoryState;
    if (!parsed.organizations || !parsed.personalGroups || !parsed.personalUsers) {
      throw new Error('Invalid cached directory');
    }
    return parsed;
  } catch {
    return {
      organizations: mockOrganizations.map(normaliseMockOrg),
      personalGroups: mockPersonalGroups.map(normaliseMockGroup),
      personalUsers: mockPersonalUsers.map(normaliseMockUser),
    };
  }
};

const persistState = (state: DirectoryState) => {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
};

const EntityDirectoryContext = createContext<EntityDirectoryContextValue | undefined>(undefined);

export const EntityDirectoryProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [organizations, setOrganizations] = useState<Array<Organization & EntityMetadata>>(() => loadState().organizations);
  const [personalGroups, setPersonalGroups] = useState<Array<Group & EntityMetadata>>(() => loadState().personalGroups);
  const [personalUsers, setPersonalUsers] = useState<Array<PersonalUser & EntityMetadata>>(() => loadState().personalUsers);
  const [operations, setOperations] = useState<EntitySyncOperation[]>([]);

  useEffect(() => {
    persistState({ organizations, personalGroups, personalUsers });
  }, [organizations, personalGroups, personalUsers]);

  const recordOperation = useCallback((operation: Omit<EntitySyncOperation, 'timestamp'>) => {
    setOperations(prev => [
      {
        ...operation,
        timestamp: Date.now(),
      },
      ...prev.slice(0, 49),
    ]);
  }, []);

  const validateFourWords = useCallback(async (fourWords: string): Promise<FourWordsValidationResult> => {
    const normalized = fourWordsToStorage(fourWords);
    if (!/^[a-z]+(?:-[a-z]+){3}$/.test(normalized)) {
      return { isValid: false, error: 'Four-words must contain four lowercase words separated by dashes' };
    }
    const exists =
      organizations.some(org => org.networkIdentity?.fourWords === normalized) ||
      personalGroups.some(group => group.networkIdentity?.fourWords === normalized) ||
      personalUsers.some(user => user.networkIdentity?.fourWords === normalized);
    if (exists) {
      return { isValid: false, error: 'This identity is already linked to your workspace' };
    }
    return { isValid: true, normalized };
  }, [organizations, personalGroups, personalUsers]);

  const buildOperationResult = (
    entityId: string,
    fourWords: string,
    isOwned: boolean,
  ): EntityOperationResult => ({
    success: true,
    entityId,
    fourWords,
    isOwned,
    needsSync: false,
  });

  const createOrganization = useCallback(async (input: CreateNewOrganizationInput): Promise<EntityOperationResult> => {
    const fourWords = fourWordsToStorage(generateFourWords());
    const now = new Date();
    const orgId = `org-${nanoid(8)}`;
    const defaultChannel: Channel = {
      id: `channel-${nanoid(8)}`,
      type: 'channel',
      organizationId: orgId,
      name: 'general',
      description: 'Company-wide announcements and coordination.',
      isPrivate: false,
      members: [],
      networkIdentity: createNetworkIdentity(),
      capabilities: DEFAULT_CAPABILITIES,
      createdAt: now,
      updatedAt: now,
    };

    const organization: Organization & EntityMetadata = wrapOrganization({
      id: orgId,
      type: 'organization',
      name: input.displayName.trim(),
      description: input.description,
      networkIdentity: createNetworkIdentity(fourWords),
      capabilities: DEFAULT_CAPABILITIES,
      owners: [],
      channels: [defaultChannel],
      groups: [],
      projects: [],
      users: [],
      settings: {
        allowGuestAccess: false,
        defaultChannelPermissions: [],
        websitePublishingEnabled: true,
      },
      createdAt: now,
      updatedAt: now,
    });

    setOrganizations(prev => [...prev, organization]);
    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'organization',
      operation: 'create',
      payload: { organizationId: orgId },
      status: 'completed',
      attempts: 1,
    });

    return buildOperationResult(orgId, fourWords, true);
  }, [recordOperation]);

  const createGroup = useCallback(async (input: CreateNewGroupInput): Promise<EntityOperationResult> => {
    const fourWords = fourWordsToStorage(generateFourWords());
    const group: Group & EntityMetadata = wrapGroup({
      id: `group-${nanoid(8)}`,
      type: 'group',
      name: input.displayName.trim(),
      description: input.description,
      organizationId: input.organizationId,
      members: [],
      admins: [],
      isPersonal: !input.organizationId,
      networkIdentity: createNetworkIdentity(fourWords),
      capabilities: DEFAULT_CAPABILITIES,
      createdAt: new Date(),
      updatedAt: new Date(),
    });

    if (input.organizationId) {
      setOrganizations(prev =>
        prev.map(org =>
          org.id === input.organizationId
            ? { ...org, groups: [...(org.groups ?? []), group], updatedAt: new Date() }
            : org,
        ),
      );
    } else {
      setPersonalGroups(prev => [...prev, group]);
    }

    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'group',
      operation: 'create',
      payload: { groupId: group.id, organizationId: input.organizationId },
      status: 'completed',
      attempts: 1,
    });

    return buildOperationResult(group.id, fourWords, true);
  }, [recordOperation]);

  const createChannel = useCallback(async (input: CreateNewChannelInput): Promise<EntityOperationResult> => {
    const org = organizations.find(item => item.id === input.organizationId);
    if (!org) {
      throw new Error('Organization not found');
    }

    const fourWords = fourWordsToStorage(generateFourWords());
    const now = new Date();
    const channel: Channel = {
      id: `channel-${nanoid(8)}`,
      type: 'channel',
      name: input.displayName.trim(),
      description: input.description,
      organizationId: org.id,
      isPrivate: Boolean(input.isPrivate),
      members: [],
      networkIdentity: createNetworkIdentity(fourWords),
      capabilities: DEFAULT_CAPABILITIES,
      createdAt: now,
      updatedAt: now,
    };

    setOrganizations(prev =>
      prev.map(item =>
        item.id === org.id
          ? { ...item, channels: [...(item.channels ?? []), channel], updatedAt: now }
          : item,
      ),
    );

    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'channel',
      operation: 'create',
      payload: { channelId: channel.id, organizationId: org.id },
      status: 'completed',
      attempts: 1,
    });

    return buildOperationResult(channel.id, fourWords, true);
  }, [organizations, recordOperation]);

  const createProject = useCallback(async (input: CreateNewProjectInput): Promise<EntityOperationResult> => {
    const org = organizations.find(item => item.id === input.organizationId);
    if (!org) throw new Error('Organization not found');

    const projectId = `project-${nanoid(8)}`;
    const fourWords = fourWordsToStorage(generateFourWords());
    const project: Project = {
      id: projectId,
      type: 'project',
      name: input.displayName.trim(),
      description: input.description,
      organizationId: org.id,
      leads: [],
      members: [],
      status: 'planning',
      networkIdentity: createNetworkIdentity(fourWords),
      capabilities: DEFAULT_CAPABILITIES,
      createdAt: new Date(),
      updatedAt: new Date(),
      milestones: [],
    };

    setOrganizations(prev =>
      prev.map(item =>
        item.id === org.id
          ? { ...item, projects: [...(item.projects ?? []), project], updatedAt: new Date() }
          : item,
      ),
    );

    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'project',
      operation: 'create',
      payload: { projectId, organizationId: org.id },
      status: 'completed',
      attempts: 1,
    });

    return buildOperationResult(projectId, fourWords, true);
  }, [organizations, recordOperation]);

  const createContact = useCallback(async (input: CreateNewContactInput): Promise<EntityOperationResult> => {
    const fourWords = fourWordsToStorage(generateFourWords());
    const userId = `user-${nanoid(8)}`;
    const user: PersonalUser & EntityMetadata = wrapPersonalUser({
      id: userId,
      userId,
      type: 'personal_user',
      name: input.displayName.trim(),
      relationship: input.relationship ?? 'friend',
      email: input.email,
      tags: [],
      networkIdentity: createNetworkIdentity(fourWords, false),
      capabilities: DEFAULT_CAPABILITIES,
      createdAt: new Date(),
      updatedAt: new Date(),
    });

    setPersonalUsers(prev => [...prev, user]);
    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'contact',
      operation: 'create',
      payload: { contactId: user.id },
      status: 'completed',
      attempts: 1,
    });

    return buildOperationResult(user.id, fourWords, true);
  }, [recordOperation]);

  const addExistingOrganization = useCallback(
    async (input: AddExistingOrganizationInput): Promise<EntityOperationResult> => {
      const fourWords = fourWordsToStorage(input.fourWords);
      const orgId = `org-${nanoid(8)}`;
      const now = new Date();

      const organization: Organization & EntityMetadata = wrapOrganization({
        id: orgId,
        type: 'organization',
        name: input.displayName?.trim() || fourWordsToDisplay(fourWords),
        description: undefined,
        networkIdentity: createNetworkIdentity(fourWords, false),
        capabilities: DEFAULT_CAPABILITIES,
        owners: [],
        channels: [],
        groups: [],
        projects: [],
        users: [],
        settings: {
          allowGuestAccess: false,
          defaultChannelPermissions: [],
          websitePublishingEnabled: true,
        },
        createdAt: now,
        updatedAt: now,
      });

      setOrganizations(prev => [...prev, organization]);
      recordOperation({
        id: `op-${nanoid(6)}`,
        entityType: 'organization',
        operation: 'resolve',
        payload: { organizationId: orgId, fourWords },
        status: 'completed',
        attempts: 1,
      });

      return buildOperationResult(orgId, fourWords, false);
    },
    [recordOperation],
  );

  const addExistingGroup = useCallback(async (input: AddExistingGroupInput): Promise<EntityOperationResult> => {
    const fourWords = fourWordsToStorage(input.fourWords);
    const group: Group & EntityMetadata = wrapGroup({
      id: `group-${nanoid(8)}`,
      type: 'group',
      name: input.displayName?.trim() || fourWordsToDisplay(fourWords),
      description: undefined,
      organizationId: input.organizationId,
      members: [],
      admins: [],
      isPersonal: !input.organizationId,
      networkIdentity: createNetworkIdentity(fourWords, false),
      capabilities: DEFAULT_CAPABILITIES,
      createdAt: new Date(),
      updatedAt: new Date(),
    });

    if (input.organizationId) {
      setOrganizations(prev =>
        prev.map(item =>
          item.id === input.organizationId
            ? { ...item, groups: [...(item.groups ?? []), group] }
            : item,
        ),
      );
    } else {
      setPersonalGroups(prev => [...prev, group]);
    }

    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'group',
      operation: 'resolve',
      payload: { groupId: group.id, fourWords },
      status: 'completed',
      attempts: 1,
    });

    return buildOperationResult(group.id, fourWords, false);
  }, [recordOperation]);

  const addExistingChannel = useCallback(async (input: AddExistingChannelInput): Promise<EntityOperationResult> => {
    const fourWords = fourWordsToStorage(input.fourWords);
    const channel: Channel = {
      id: `channel-${nanoid(8)}`,
      type: 'channel',
      name: input.displayName?.trim() || fourWordsToDisplay(fourWords),
      description: undefined,
      organizationId: input.organizationId,
      isPrivate: false,
      members: [],
      networkIdentity: createNetworkIdentity(fourWords, false),
      capabilities: DEFAULT_CAPABILITIES,
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    setOrganizations(prev =>
      prev.map(org =>
        org.id === input.organizationId
          ? { ...org, channels: [...(org.channels ?? []), channel] }
          : org,
      ),
    );

    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'channel',
      operation: 'resolve',
      payload: { channelId: channel.id, fourWords },
      status: 'completed',
      attempts: 1,
    });

    return buildOperationResult(channel.id, fourWords, false);
  }, [recordOperation]);

  const addExistingProject = useCallback(async (input: AddExistingProjectInput): Promise<EntityOperationResult> => {
    const fourWords = fourWordsToStorage(input.fourWords);
    const project: Project = {
      id: `project-${nanoid(8)}`,
      type: 'project',
      name: input.displayName?.trim() || fourWordsToDisplay(fourWords),
      description: undefined,
      organizationId: input.organizationId,
      leads: [],
      members: [],
      status: 'planning',
      networkIdentity: createNetworkIdentity(fourWords, false),
      capabilities: DEFAULT_CAPABILITIES,
      createdAt: new Date(),
      updatedAt: new Date(),
      milestones: [],
    };

    setOrganizations(prev =>
      prev.map(org =>
        org.id === input.organizationId
          ? { ...org, projects: [...(org.projects ?? []), project] }
          : org,
      ),
    );

    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'project',
      operation: 'resolve',
      payload: { projectId: project.id, fourWords },
      status: 'completed',
      attempts: 1,
    });

    return buildOperationResult(project.id, fourWords, false);
  }, [recordOperation]);

  const addExistingContact = useCallback(async (input: AddExistingContactInput): Promise<EntityOperationResult> => {
    const fourWords = fourWordsToStorage(input.fourWords);
    const userId = `user-${nanoid(8)}`;
    const user: PersonalUser & EntityMetadata = wrapPersonalUser({
      id: userId,
      userId,
      type: 'personal_user',
      name: input.displayName?.trim() || fourWordsToDisplay(fourWords),
      relationship: input.relationship ?? 'friend',
      email: undefined,
      tags: [],
      networkIdentity: createNetworkIdentity(fourWords, false),
      capabilities: DEFAULT_CAPABILITIES,
      createdAt: new Date(),
      updatedAt: new Date(),
    });

    setPersonalUsers(prev => [...prev, user]);
    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'contact',
      operation: 'resolve',
      payload: { contactId: user.id, fourWords },
      status: 'completed',
      attempts: 1,
    });

    return buildOperationResult(user.id, fourWords, false);
  }, [recordOperation]);

  const addOrganization = useCallback((input: { name: string; description?: string }) => {
    return createOrganization({ displayName: input.name, description: input.description });
  }, [createOrganization]);

  const removeOrganization = useCallback((organizationId: string) => {
    setOrganizations(prev => prev.filter(org => org.id !== organizationId));
    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'organization',
      operation: 'delete',
      payload: { organizationId },
      status: 'completed',
      attempts: 1,
    });
  }, [recordOperation]);

  const addOrganizationGroup = useCallback((input: { organizationId: string; name: string; description?: string }): Group => {
    const fourWords = fourWordsToStorage(generateFourWords());
    const base: Group = {
      id: `group-${nanoid(8)}`,
      type: 'group',
      name: input.name.trim(),
      description: input.description,
      organizationId: input.organizationId,
      members: [],
      admins: [],
      isPersonal: false,
      networkIdentity: createNetworkIdentity(fourWords),
      capabilities: DEFAULT_CAPABILITIES,
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    setOrganizations(prev =>
      prev.map(org =>
        org.id === input.organizationId
          ? { ...org, groups: [...(org.groups ?? []), wrapGroup(base)], updatedAt: new Date() }
          : org,
      ),
    );

    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'group',
      operation: 'create',
      payload: { groupId: base.id, organizationId: input.organizationId },
      status: 'completed',
      attempts: 1,
    });

    return base;
  }, [recordOperation]);

  const removeOrganizationGroup = useCallback((organizationId: string, groupId: string) => {
    setOrganizations(prev =>
      prev.map(org =>
        org.id === organizationId
          ? { ...org, groups: (org.groups ?? []).filter(group => group.id !== groupId) }
          : org,
      ),
    );
    setPersonalGroups(prev => prev.filter(group => group.id !== groupId));
    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'group',
      operation: 'delete',
      payload: { groupId, organizationId },
      status: 'completed',
      attempts: 1,
    });
  }, [recordOperation]);

  const addOrganizationChannel = useCallback((input: { organizationId: string; name: string; description?: string; isPrivate?: boolean }): Channel => {
    const fourWords = fourWordsToStorage(generateFourWords());
    const channel: Channel = {
      id: `channel-${nanoid(8)}`,
      type: 'channel',
      name: input.name.trim(),
      description: input.description,
      organizationId: input.organizationId,
      isPrivate: Boolean(input.isPrivate),
      members: [],
      networkIdentity: createNetworkIdentity(fourWords),
      capabilities: DEFAULT_CAPABILITIES,
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    setOrganizations(prev =>
      prev.map(org =>
        org.id === input.organizationId
          ? { ...org, channels: [...(org.channels ?? []), channel], updatedAt: new Date() }
          : org,
      ),
    );

    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'channel',
      operation: 'create',
      payload: { channelId: channel.id, organizationId: input.organizationId },
      status: 'completed',
      attempts: 1,
    });

    return channel;
  }, [recordOperation]);

  const removeOrganizationChannel = useCallback((organizationId: string, channelId: string) => {
    setOrganizations(prev =>
      prev.map(org =>
        org.id === organizationId
          ? { ...org, channels: (org.channels ?? []).filter(channel => channel.id !== channelId) }
          : org,
      ),
    );
    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'channel',
      operation: 'delete',
      payload: { channelId, organizationId },
      status: 'completed',
      attempts: 1,
    });
  }, [recordOperation]);

  const addProject = useCallback((input: { organizationId: string; name: string; description?: string }): Project => {
    const fourWords = fourWordsToStorage(generateFourWords());
    const project: Project = {
      id: `project-${nanoid(8)}`,
      type: 'project',
      name: input.name.trim(),
      description: input.description,
      organizationId: input.organizationId,
      leads: [],
      members: [],
      status: 'planning',
      networkIdentity: createNetworkIdentity(fourWords),
      capabilities: DEFAULT_CAPABILITIES,
      createdAt: new Date(),
      updatedAt: new Date(),
      milestones: [],
    };

    setOrganizations(prev =>
      prev.map(org =>
        org.id === input.organizationId
          ? { ...org, projects: [...(org.projects ?? []), project], updatedAt: new Date() }
          : org,
      ),
    );

    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'project',
      operation: 'create',
      payload: { projectId: project.id, organizationId: input.organizationId },
      status: 'completed',
      attempts: 1,
    });

    return project;
  }, [recordOperation]);

  const removeProject = useCallback((organizationId: string, projectId: string) => {
    setOrganizations(prev =>
      prev.map(org =>
        org.id === organizationId
          ? { ...org, projects: (org.projects ?? []).filter(project => project.id !== projectId) }
          : org,
      ),
    );
    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'project',
      operation: 'delete',
      payload: { projectId, organizationId },
      status: 'completed',
      attempts: 1,
    });
  }, [recordOperation]);

  const addPersonalGroup = useCallback(async (input: { name: string; description?: string }) => {
    return createGroup({
      displayName: input.name,
      description: input.description,
    });
  }, [createGroup]);

  const removePersonalGroup = useCallback((groupId: string) => {
    setPersonalGroups(prev => prev.filter(group => group.id !== groupId));
    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'group',
      operation: 'delete',
      payload: { groupId },
      status: 'completed',
      attempts: 1,
    });
  }, [recordOperation]);

  const addPersonalUser = useCallback((input: { name: string; relationship?: PersonalUser['relationship']; email?: string }): PersonalUser => {
    const fourWords = fourWordsToStorage(generateFourWords());
    const userId = `user-${nanoid(8)}`;
    const user: PersonalUser = {
      id: userId,
      userId,
      type: 'personal_user',
      name: input.name.trim(),
      relationship: input.relationship ?? 'colleague',
      email: input.email,
      tags: [],
      networkIdentity: createNetworkIdentity(fourWords, false),
      capabilities: DEFAULT_CAPABILITIES,
      createdAt: new Date(),
      updatedAt: new Date(),
    };
    setPersonalUsers(prev => [...prev, wrapPersonalUser(user)]);
    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'contact',
      operation: 'create',
      payload: { contactId: user.id },
      status: 'completed',
      attempts: 1,
    });
    return user;
  }, [recordOperation]);

  const removePersonalUser = useCallback((userId: string) => {
    setPersonalUsers(prev => prev.filter(user => user.id !== userId));
    recordOperation({
      id: `op-${nanoid(6)}`,
      entityType: 'contact',
      operation: 'delete',
      payload: { contactId: userId },
      status: 'completed',
      attempts: 1,
    });
  }, [recordOperation]);

  const resetDirectory = useCallback(() => {
    const state = loadState();
    setOrganizations(state.organizations);
    setPersonalGroups(state.personalGroups);
    setPersonalUsers(state.personalUsers);
    setOperations([]);
  }, []);

  const enqueueOperation = useCallback((operation: EntitySyncOperation) => {
    setOperations(prev => [operation, ...prev.slice(0, 49)]);
  }, []);

  const markOperationComplete = useCallback((operationId: string, newIdMap?: Record<string, string>) => {
    setOperations(prev =>
      prev.map(operation =>
        operation.id === operationId
          ? { ...operation, status: 'completed', attempts: operation.attempts + 1 }
          : operation,
      ),
    );

    if (newIdMap) {
      const [[tempId, newId]] = Object.entries(newIdMap);
      setOrganizations(prev =>
        prev.map(org => (org.id === tempId ? { ...org, id: newId, syncStatus: 'synced' } : org)),
      );
    }
  }, []);

  const markOperationFailed = useCallback((operationId: string, error: string) => {
    setOperations(prev =>
      prev.map(operation =>
        operation.id === operationId
          ? { ...operation, status: 'failed', attempts: operation.attempts + 1, error }
          : operation,
      ),
    );
  }, []);

  const setEntityStatus = useCallback((entityType: EntitySyncOperation['entityType'], entityId: string, status: SyncStatus) => {
    const apply = <T extends { id: string }>(collection: T[], updater: (item: T) => T): T[] =>
      collection.map(item => (item.id === entityId ? updater(item) : item));

    if (entityType === 'organization') {
      setOrganizations(prev => apply(prev, org => ({ ...org, syncStatus: status, lastSyncedAt: Date.now() as any })));
    } else if (entityType === 'group') {
      setPersonalGroups(prev => apply(prev, group => ({ ...group, syncStatus: status, lastSyncedAt: Date.now() as any })));
    } else if (entityType === 'contact') {
      setPersonalUsers(prev => apply(prev, user => ({ ...user, syncStatus: status, lastSyncedAt: Date.now() as any })));
    }
  }, []);

  const queueMessage = useCallback((payload: { id: string; entityId: string; entityType: EntitySyncOperation['entityType']; content: string; timestamp: string }) => {
    recordOperation({
      id: `msg-${payload.id}`,
      entityType: 'message',
      operation: 'create',
      payload,
      status: 'completed',
      attempts: 1,
    });
  }, [recordOperation]);

  const value = useMemo<EntityDirectoryContextValue>(() => ({
    organizations,
    personalGroups,
    personalUsers,
    operations,
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
    organizations,
    personalGroups,
    personalUsers,
    operations,
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

  return <EntityDirectoryContext.Provider value={value}>{children}</EntityDirectoryContext.Provider>;
};

export const useEntityDirectory = () => {
  const context = useContext(EntityDirectoryContext);
  if (!context) {
    throw new Error('useEntityDirectory must be used within an EntityDirectoryProvider');
  }
  return context;
};
