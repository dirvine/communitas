import React, {
  createContext,
  useContext,
  useMemo,
  useState,
  useEffect,
  ReactNode,
} from 'react';

import { fourWordsToStorage, fourWordsToDisplay } from '../utils/identity';

const STORAGE_KEYS = {
  identities: 'communitas.auth.identities.v1',
  session: 'communitas.auth.session.v1',
  config: 'communitas.auth.config.v1',
};

type StoredIdentity = {
  id: string;
  fourWords: string;
  displayName: string;
  password: string;
  createdAt: number;
  lastUsed: number;
  hasPasskey: boolean;
  passkeyDeviceName?: string;
};

type StoredSession = {
  sessionId: string;
  fourWords: string;
  displayName: string;
  startedAt: number;
};

type AuthConfig = {
  autoLoginEnabled: boolean;
  keyringEnabled: boolean;
};

type NetworkStatus = {
  connected: boolean;
  peers: number;
};

export interface Permission {
  resource: string;
  actions: string[];
  scope?: string;
}

export interface UserIdentity {
  id: string;
  publicKey: string;
  fourWordAddress: string;
  name: string;
  avatar?: string;
  profile: {
    bio?: string;
    organization?: string;
    location?: string;
    website?: string;
    socialLinks?: {
      github?: string;
      twitter?: string;
      linkedin?: string;
    };
  };
  permissions: Permission[];
  createdAt: string;
  lastActive: string;
}

export interface AuthState {
  isAuthenticated: boolean;
  user: UserIdentity | null;
  loading: boolean;
  error: string | null;
}

interface VaultInfo {
  four_words: string;
  display_name: string;
  created_at: number;
  last_accessed: number;
  size_bytes: number;
}

export interface AuthContextType {
  authState: AuthState;
  login: (fourWordAddress: string, password?: string) => Promise<boolean>;
  logout: () => Promise<void>;
  createIdentity: (name: string, options?: { fourWords?: string; password?: string }) => Promise<UserIdentity>;
  registerPasskey: (fourWords: string, deviceName?: string) => Promise<boolean>;
  signInWithPasskey: (fourWords?: string) => Promise<boolean>;
  updateProfile: (updates: Partial<UserIdentity['profile']>) => Promise<void>;
  connectToNetwork: () => Promise<boolean>;
  disconnectFromNetwork: () => Promise<void>;
  getNetworkStatus: () => Promise<{ connected: boolean; peers: number }>;
  hasPermission: (resource: string, action: string) => boolean;
  isOwner: (resourceOwnerId: string) => boolean;
  canAccess: (resource: string, requiredPermissions: string[]) => boolean;
  listVaults: () => Promise<VaultInfo[]>;
  isFirstLaunch: () => Promise<boolean>;
  getConfig: () => Promise<AuthConfig>;
  setAutoLogin: (enabled: boolean) => Promise<void>;
  setKeyringEnabled: (enabled: boolean) => Promise<void>;
  getRecentIdentities: () => Promise<Array<{ four_words: string; display_name: string; last_used: number; has_passkey: boolean }>>;
  getOsUsername: () => Promise<string>;
  enableAutoLogin: (fourWords: string, password: string) => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

const WORD_BANK = [
  ['ocean', 'forest', 'mountain', 'desert', 'river', 'valley', 'island', 'prairie'],
  ['bright', 'shadow', 'crystal', 'silver', 'golden', 'misty', 'ember', 'aurora'],
  ['lion', 'hawk', 'wolf', 'otter', 'sparrow', 'falcon', 'lynx', 'otter'],
  ['star', 'moon', 'sun', 'cloud', 'storm', 'wind', 'fire', 'ice'],
];

const generateRandomFourWords = (): string => {
  const words = WORD_BANK.map(group => group[Math.floor(Math.random() * group.length)]);
  return words.join('-');
};

const loadIdentities = (): StoredIdentity[] => {
  try {
    const raw = localStorage.getItem(STORAGE_KEYS.identities);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as StoredIdentity[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
};

const saveIdentities = (identities: StoredIdentity[]) => {
  localStorage.setItem(STORAGE_KEYS.identities, JSON.stringify(identities));
};

const loadConfig = (): AuthConfig => {
  try {
    const raw = localStorage.getItem(STORAGE_KEYS.config);
    if (!raw) {
      return { autoLoginEnabled: true, keyringEnabled: true };
    }
    const parsed = JSON.parse(raw) as AuthConfig;
    return {
      autoLoginEnabled: parsed.autoLoginEnabled ?? true,
      keyringEnabled: parsed.keyringEnabled ?? true,
    };
  } catch {
    return { autoLoginEnabled: true, keyringEnabled: true };
  }
};

const saveConfig = (config: AuthConfig) => {
  localStorage.setItem(STORAGE_KEYS.config, JSON.stringify(config));
};

const loadSession = (): StoredSession | null => {
  try {
    const raw = localStorage.getItem(STORAGE_KEYS.session);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as StoredSession;
    if (parsed && parsed.fourWords) {
      return parsed;
    }
    return null;
  } catch {
    return null;
  }
};

const saveSession = (session: StoredSession | null) => {
  if (session) {
    localStorage.setItem(STORAGE_KEYS.session, JSON.stringify(session));
  } else {
    localStorage.removeItem(STORAGE_KEYS.session);
  }
};

const createSession = (identity: StoredIdentity): StoredSession => ({
  sessionId: `session-${Date.now().toString(36)}-${Math.random().toString(16).slice(2)}`,
  fourWords: identity.fourWords,
  displayName: identity.displayName,
  startedAt: Date.now(),
});

const buildUserIdentity = (identity: StoredIdentity, sessionId: string): UserIdentity => ({
  id: sessionId,
  name: identity.displayName,
  fourWordAddress: identity.fourWords,
  publicKey: `pk_${identity.id}`,
  profile: {},
  permissions: [{ resource: '*', actions: ['*'] }],
  createdAt: new Date(identity.createdAt).toISOString(),
  lastActive: new Date(identity.lastUsed).toISOString(),
});

export interface ProtectedRouteProps {
  children: ReactNode;
  requiredPermissions?: { resource: string; actions: string[] };
  fallback?: ReactNode;
}

export const AuthProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [identities, setIdentities] = useState<StoredIdentity[]>(() => loadIdentities());
  const [config, setConfig] = useState<AuthConfig>(() => loadConfig());
  const [networkStatus, setNetworkStatus] = useState<NetworkStatus>({ connected: false, peers: 0 });
  const [authState, setAuthState] = useState<AuthState>({
    isAuthenticated: false,
    user: null,
    loading: true,
    error: null,
  });

  useEffect(() => {
    const session = loadSession();
    if (session) {
      const identity = identities.find(item => item.fourWords === session.fourWords);
      if (identity) {
        setAuthState({
          isAuthenticated: true,
          user: buildUserIdentity(identity, session.sessionId),
          loading: false,
          error: null,
        });
        return;
      }
      saveSession(null);
    }
    setAuthState(prev => ({ ...prev, loading: false }));
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    saveIdentities(identities);
  }, [identities]);

  useEffect(() => {
    saveConfig(config);
  }, [config]);

  const setActiveIdentity = (identity: StoredIdentity): StoredSession => {
    const session = createSession(identity);
    saveSession(session);
    setIdentities(prev =>
      prev.map(item => (item.id === identity.id ? { ...item, lastUsed: Date.now() } : item)),
    );
    setAuthState({
      isAuthenticated: true,
      user: buildUserIdentity(identity, session.sessionId),
      loading: false,
      error: null,
    });
    return session;
  };

  const login = async (fourWordAddress: string, password?: string): Promise<boolean> => {
    const normalized = fourWordsToStorage(fourWordAddress);
    const identity = identities.find(item => item.fourWords === normalized);
    if (!identity) {
      setAuthState(prev => ({
        ...prev,
        loading: false,
        error: 'Identity not found',
      }));
      return false;
    }

    const secret = password ?? identity.password;
    if (identity.password !== secret) {
      setAuthState(prev => ({
        ...prev,
        loading: false,
        error: 'Invalid password',
      }));
      return false;
    }

    setActiveIdentity(identity);
    return true;
  };

  const logout = async (): Promise<void> => {
    saveSession(null);
    setAuthState({
      isAuthenticated: false,
      user: null,
      loading: false,
      error: null,
    });
    setNetworkStatus({ connected: false, peers: 0 });
  };

  const createIdentity = async (
    name: string,
    options?: { fourWords?: string; password?: string },
  ): Promise<UserIdentity> => {
    const normalized = fourWordsToStorage(options?.fourWords ?? generateRandomFourWords());
    const password = options?.password ?? normalized;

    if (identities.some(identity => identity.fourWords === normalized)) {
      throw new Error('Identity already exists on this device');
    }

    const stored: StoredIdentity = {
      id: `id-${Date.now().toString(36)}-${Math.random().toString(16).slice(2)}`,
      fourWords: normalized,
      displayName: name || fourWordsToDisplay(normalized),
      password,
      createdAt: Date.now(),
      lastUsed: Date.now(),
      hasPasskey: false,
    };

    setIdentities(prev => [...prev, stored]);
    const session = setActiveIdentity(stored);
    return buildUserIdentity(stored, session.sessionId);
  };

  const registerPasskey = async (fourWords: string, deviceName?: string): Promise<boolean> => {
    const normalized = fourWordsToStorage(fourWords);
    const identity = identities.find(item => item.fourWords === normalized);
    if (!identity) {
      return false;
    }

    const updated: StoredIdentity = {
      ...identity,
      hasPasskey: true,
      passkeyDeviceName: deviceName || navigator.platform || 'Current Device',
    };

    setIdentities(prev => prev.map(item => (item.id === identity.id ? updated : item)));

    if (authState.user && authState.user.fourWordAddress === normalized) {
      setAuthState(prev => prev.user
        ? {
            ...prev,
            user: { ...prev.user, name: updated.displayName },
          }
        : prev);
    }

    return true;
  };

  const signInWithPasskey = async (fourWords?: string): Promise<boolean> => {
    const normalized = fourWords ? fourWordsToStorage(fourWords) : authState.user?.fourWordAddress;
    if (!normalized) return false;

    const identity = identities.find(item => item.fourWords === normalized);
    if (!identity || !identity.hasPasskey) return false;

    setActiveIdentity(identity);
    return true;
  };

  const updateProfile = async (updates: Partial<UserIdentity['profile']>) => {
    setAuthState(prev => {
      if (!prev.user) return prev;
      return {
        ...prev,
        user: {
          ...prev.user,
          profile: { ...prev.user.profile, ...updates },
        },
      };
    });
  };

  const connectToNetwork = async (): Promise<boolean> => {
    if (!authState.user) return false;
    const peers = 3 + Math.floor(Math.random() * 5);
    setNetworkStatus({ connected: true, peers });
    return true;
  };

  const disconnectFromNetwork = async (): Promise<void> => {
    setNetworkStatus({ connected: false, peers: 0 });
  };

  const getNetworkStatus = async (): Promise<{ connected: boolean; peers: number }> => networkStatus;

  const hasPermission = (resource: string, action: string): boolean => {
    if (!authState.user) return false;
    if (authState.user.permissions.some(perm => perm.resource === '*' || perm.resource === resource)) {
      return true;
    }
    return false;
  };

  const isOwner = (resourceOwnerId: string): boolean => {
    if (!authState.user) return false;
    return (
      authState.user.id === resourceOwnerId ||
      authState.user.fourWordAddress === resourceOwnerId
    );
  };

  const canAccess = (resource: string, requiredPermissions: string[]): boolean => {
    if (!authState.user) return false;
    return requiredPermissions.every(action => hasPermission(resource, action));
  };

  const listVaults = async (): Promise<VaultInfo[]> =>
    identities.map(identity => ({
      four_words: identity.fourWords,
      display_name: identity.displayName,
      created_at: identity.createdAt,
      last_accessed: identity.lastUsed,
      size_bytes: 1024 * 12,
    }));

  const isFirstLaunch = async (): Promise<boolean> => identities.length === 0;

  const getConfig = async (): Promise<AuthConfig> => config;

  const setAutoLogin = async (enabled: boolean) => {
    setConfig(prev => ({ ...prev, autoLoginEnabled: enabled }));
  };

  const setKeyringEnabled = async (enabled: boolean) => {
    setConfig(prev => ({ ...prev, keyringEnabled: enabled }));
  };

  const getRecentIdentities = async () =>
    identities
      .slice()
      .sort((a, b) => b.lastUsed - a.lastUsed)
      .map(identity => ({
        four_words: identity.fourWords,
        display_name: identity.displayName,
        last_used: Math.floor(identity.lastUsed / 1000),
        has_passkey: identity.hasPasskey,
      }));

  const getOsUsername = async (): Promise<string> => {
    const platform = navigator.platform || 'Guest';
    const parts = platform.split(' ');
    return parts.length ? parts[0] : 'Guest';
  };

  const enableAutoLogin = async (fourWords: string, password: string) => {
    const normalized = fourWordsToStorage(fourWords);
    const identity = identities.find(item => item.fourWords === normalized);
    if (!identity) {
      await createIdentity(fourWordsToDisplay(normalized), { fourWords: normalized, password });
    } else {
      await login(normalized, password);
    }
    await setAutoLogin(true);
    await setKeyringEnabled(true);
  };

  const contextValue: AuthContextType = useMemo(() => ({
    authState,
    login,
    logout,
    createIdentity,
    registerPasskey,
    signInWithPasskey,
    updateProfile,
    connectToNetwork,
    disconnectFromNetwork,
    getNetworkStatus,
    hasPermission,
    isOwner,
    canAccess,
    listVaults,
    isFirstLaunch,
    getConfig,
    setAutoLogin,
    setKeyringEnabled,
    getRecentIdentities,
    getOsUsername,
    enableAutoLogin,
  }), [authState, identities, config, networkStatus]);

  return (
    <AuthContext.Provider value={contextValue}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = (): AuthContextType => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};

export const ProtectedRoute: React.FC<ProtectedRouteProps> = ({
  children,
  requiredPermissions,
  fallback = <div>Access denied</div>,
}) => {
  const { authState, canAccess } = useAuth();

  if (!authState.isAuthenticated) {
    return <>{fallback}</>;
  }

  if (requiredPermissions && !canAccess(requiredPermissions.resource, requiredPermissions.actions)) {
    return <>{fallback}</>;
  }

  return <>{children}</>;
};
