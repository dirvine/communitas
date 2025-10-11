import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';

// Dynamic import of Tauri API with fallback
let invoke: any = async (cmd: string, args?: any) => {
  // Try to get Tauri from window first
  if (typeof window !== 'undefined' && (window as any).__TAURI__?.core?.invoke) {
    return (window as any).__TAURI__.core.invoke(cmd, args);
  }

  // Try dynamic import
  try {
    const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
    return tauriInvoke(cmd, args);
  } catch (error) {
    console.warn(`Tauri not available, using mock mode for ${cmd}`);
    // Minimal mock for browser dev
    throw new Error(`Command ${cmd} not available in browser mode`);
  }
};

// User identity interface
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

// Permission system
export interface Permission {
  resource: string;
  actions: string[];
  scope?: string;
}

// Authentication state
export interface AuthState {
  isAuthenticated: boolean;
  user: UserIdentity | null;
  loading: boolean;
  error: string | null;
}

// Session info from Rust backend
interface SessionInfo {
  session_id: string;
  four_words: string;
  display_name: string;
}

// Vault info from Rust backend
interface VaultInfo {
  four_words: string;
  display_name: string;
  created_at: number;
  last_accessed: number;
  size_bytes: number;
}

// Authentication context
export interface AuthContextType {
  // State
  authState: AuthState;

  // Authentication methods
  login: (fourWordAddress: string, password?: string) => Promise<boolean>;
  logout: () => Promise<void>;
  createIdentity: (name: string, options?: { fourWords?: string; password?: string }) => Promise<UserIdentity>;
  registerPasskey: () => Promise<boolean>;
  signInWithPasskey: () => Promise<boolean>;

  // Identity management
  updateProfile: (updates: Partial<UserIdentity['profile']>) => Promise<void>;

  // Network identity
  connectToNetwork: () => Promise<boolean>;
  disconnectFromNetwork: () => Promise<void>;
  getNetworkStatus: () => Promise<{ connected: boolean; peers: number }>;

  // Utility methods
  hasPermission: (resource: string, action: string) => boolean;
  isOwner: (resourceOwnerId: string) => boolean;
  canAccess: (resource: string, requiredPermissions: string[]) => boolean;

  // Vault management
  listVaults: () => Promise<VaultInfo[]>;
  isFirstLaunch: () => Promise<boolean>;

  // Configuration management
  getConfig: () => Promise<any>;
  setAutoLogin: (enabled: boolean) => Promise<void>;
  setKeyringEnabled: (enabled: boolean) => Promise<void>;
  getRecentIdentities: () => Promise<any[]>;

  // OS integration
  getOsUsername: () => Promise<string>;
  enableAutoLogin: (fourWords: string, password: string) => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

interface AuthProviderProps {
  children: ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [authState, setAuthState] = useState<AuthState>({
    isAuthenticated: false,
    user: null,
    loading: true,
    error: null,
  });

  // Initialize authentication state
  useEffect(() => {
    initializeAuth();
  }, []);

  const initializeAuth = async () => {
    try {
      // Initialize encrypted storage backend
      await invoke('auth_initialize');

      // Try auto-login first (uses keyring if enabled)
      const autoLoginSession = await invoke('auth_try_auto_login') as SessionInfo | null;

      if (autoLoginSession) {
        // Auto-login successful
        const identity: UserIdentity = {
          id: autoLoginSession.session_id,
          name: autoLoginSession.display_name,
          fourWordAddress: autoLoginSession.four_words,
          publicKey: 'from-vault',
          profile: {},
          permissions: [],
          createdAt: new Date().toISOString(),
          lastActive: new Date().toISOString(),
        };

        setAuthState({
          isAuthenticated: true,
          user: identity,
          loading: false,
          error: null,
        });

        console.log('✅ Auto-login successful:', identity.fourWordAddress);

        // Initialize CoreContext with P2P networking (non-blocking)
        try {
          console.log('🌐 Initializing CoreContext with networking...');
          await invoke('core_initialize', {
            fourWords: identity.fourWordAddress,
            displayName: identity.name,
            deviceName: navigator.platform || 'Desktop',
            deviceType: 'Desktop'
          });
          console.log('✅ CoreContext initialized with P2P networking');
        } catch (err) {
          console.warn('⚠️ CoreContext initialization failed (app will work in local mode):', err);
        }

        // Try to connect to network (non-blocking)
        connectToNetwork().catch(err =>
          console.warn('Network connection failed (non-fatal):', err)
        );

        return;
      }

      // No auto-login, check if there's an active session
      const session = await invoke('auth_get_session') as SessionInfo | null;

      if (session) {
        // Restore session
        const identity: UserIdentity = {
          id: session.session_id,
          name: session.display_name,
          fourWordAddress: session.four_words,
          publicKey: 'from-vault',
          profile: {},
          permissions: [],
          createdAt: new Date().toISOString(),
          lastActive: new Date().toISOString(),
        };

        setAuthState({
          isAuthenticated: true,
          user: identity,
          loading: false,
          error: null,
        });

        console.log('✅ Session restored:', identity.fourWordAddress);

        // Initialize CoreContext with P2P networking (non-blocking)
        try {
          console.log('🌐 Initializing CoreContext with networking...');
          await invoke('core_initialize', {
            fourWords: identity.fourWordAddress,
            displayName: identity.name,
            deviceName: navigator.platform || 'Desktop',
            deviceType: 'Desktop'
          });
          console.log('✅ CoreContext initialized with P2P networking');
        } catch (err) {
          console.warn('⚠️ CoreContext initialization failed (app will work in local mode):', err);
        }
      } else {
        setAuthState({
          isAuthenticated: false,
          user: null,
          loading: false,
          error: null,
        });
      }
    } catch (error) {
      console.error('Failed to initialize authentication:', error);
      setAuthState({
        isAuthenticated: false,
        user: null,
        loading: false,
        error: error instanceof Error ? error.message : 'Authentication initialization failed',
      });
    }
  };

  const login = async (fourWordAddress: string, password?: string): Promise<boolean> => {
    try {
      setAuthState(prev => ({ ...prev, loading: true, error: null }));

      let session: SessionInfo;

      // If no four-word address provided, try password-only login
      if (!fourWordAddress && password) {
        console.log('🔍 Attempting password-only login...');
        session = await invoke('auth_login_password_only', { password });
      } else {
        // Standard login with four-word address
        if (!fourWordAddress || !fourWordAddress.includes('-')) {
          throw new Error('Invalid four-word address format');
        }

        session = await invoke('auth_login', {
          fourWords: fourWordAddress,
          password: password || fourWordAddress
        });
      }

      // Create identity from session
      const identity: UserIdentity = {
        id: session.session_id,
        name: session.display_name,
        fourWordAddress: session.four_words,
        publicKey: 'from-vault',
        profile: {},
        permissions: [],
        createdAt: new Date().toISOString(),
        lastActive: new Date().toISOString(),
      };

      setAuthState({
        isAuthenticated: true,
        user: identity,
        loading: false,
        error: null,
      });

      console.log('✅ Login successful:', identity.fourWordAddress);

      // Initialize CoreContext with P2P networking (non-blocking)
      try {
        console.log('🌐 Initializing CoreContext with networking...');
        await invoke('core_initialize', {
          fourWords: identity.fourWordAddress,
          displayName: identity.name,
          deviceName: navigator.platform || 'Desktop',
          deviceType: 'Desktop'
        });
        console.log('✅ CoreContext initialized with P2P networking');
      } catch (err) {
        console.warn('⚠️ CoreContext initialization failed (app will work in local mode):', err);
      }

      // Try to connect to network (non-blocking)
      connectToNetwork().catch(err =>
        console.warn('Network connection failed (non-fatal):', err)
      );

      return true;
    } catch (error) {
      console.error('Login failed:', error);
      setAuthState(prev => ({
        ...prev,
        loading: false,
        error: error instanceof Error ? error.message : 'Login failed',
      }));
      return false;
    }
  };

  const logout = async (): Promise<void> => {
    try {
      // Call Rust logout command
      await invoke('auth_logout');

      setAuthState({
        isAuthenticated: false,
        user: null,
        loading: false,
        error: null,
      });

      console.log('✅ Logout successful');
    } catch (error) {
      console.error('Logout failed:', error);
      // Force logout even if cleanup fails
      setAuthState({
        isAuthenticated: false,
        user: null,
        loading: false,
        error: null,
      });
    }
  };

  const createIdentity = async (name: string, options?: { fourWords?: string; password?: string }): Promise<UserIdentity> => {
    try {
      setAuthState(prev => ({ ...prev, loading: true, error: null }));

      // Generate four words if not provided
      let fourWordAddress = options?.fourWords || '';
      if (!fourWordAddress) {
        fourWordAddress = await invoke('generate_four_word_identity');
      }

      // Create vault in Rust backend
      const password = options?.password || fourWordAddress;
      await invoke('auth_create_vault', {
        fourWords: fourWordAddress,
        password,
        displayName: name,
      });

      // Auto-login with new vault
      const session = await invoke('auth_login', {
        fourWords: fourWordAddress,
        password
      }) as SessionInfo;

      const newIdentity: UserIdentity = {
        id: session.session_id,
        name,
        fourWordAddress,
        publicKey: 'from-vault',
        profile: {},
        permissions: [],
        createdAt: new Date().toISOString(),
        lastActive: new Date().toISOString(),
      };

      setAuthState({
        isAuthenticated: true,
        user: newIdentity,
        loading: false,
        error: null,
      });

      console.log('✅ Identity created:', newIdentity.fourWordAddress);

      // Initialize CoreContext with P2P networking (non-blocking)
      try {
        console.log('🌐 Initializing CoreContext with networking...');
        await invoke('core_initialize', {
          fourWords: newIdentity.fourWordAddress,
          displayName: name,
          deviceName: navigator.platform || 'Desktop',
          deviceType: 'Desktop'
        });
        console.log('✅ CoreContext initialized with P2P networking');
      } catch (err) {
        console.warn('⚠️ CoreContext initialization failed (app will work in local mode):', err);
      }

      return newIdentity;
    } catch (error) {
      console.error('Identity creation failed:', error);
      setAuthState(prev => ({
        ...prev,
        loading: false,
        error: error instanceof Error ? error.message : 'Identity creation failed',
      }));
      throw error;
    }
  };

  const listVaults = async (): Promise<VaultInfo[]> => {
    try {
      return await invoke('auth_list_vaults');
    } catch (error) {
      console.error('Failed to list vaults:', error);
      return [];
    }
  };

  // Passkey registration (WebAuthn via browser API + backend storage)
  const registerPasskey = async (): Promise<boolean> => {
    try {
      // Check if running in Tauri
      const isTauri = !!(window as any).__TAURI__;

      if (isTauri) {
        throw new Error('Touch ID / Face ID is not supported in the desktop app. It will be available in the web version.');
      }

      if (typeof window === 'undefined' || !(window as any).PublicKeyCredential) {
        console.error('WebAuthn not supported');
        throw new Error('Touch ID / Face ID not supported on this device');
      }

      if (!authState.user?.fourWordAddress) {
        console.error('No authenticated user');
        throw new Error('Please log in first');
      }

      console.log('🔐 Starting passkey registration...');

      const challenge = crypto.getRandomValues(new Uint8Array(32));
      const userId = crypto.getRandomValues(new Uint8Array(16));

      // Step 1: Create WebAuthn credential (Touch ID/Face ID prompt)
      const credential = await (navigator.credentials as any).create({
        publicKey: {
          challenge,
          rp: { name: 'Communitas' },
          user: {
            id: userId,
            name: authState.user.fourWordAddress,
            displayName: authState.user.name
          },
          pubKeyCredParams: [{ type: 'public-key', alg: -7 }],
          timeout: 60000,
          authenticatorSelection: { userVerification: 'preferred' },
        },
      });

      if (!credential) {
        throw new Error('Failed to create passkey');
      }

      console.log('✅ WebAuthn credential created');

      // Step 2: Save passkey to backend
      const deviceName = `${navigator.platform} - ${new Date().toLocaleDateString()}`;
      await invoke('auth_passkey_register', {
        fourWords: authState.user.fourWordAddress,
        deviceName
      });

      console.log('✅ Passkey saved to backend');
      return true;
    } catch (e) {
      console.error('Passkey registration failed:', e);
      throw e;
    }
  };

  const signInWithPasskey = async (): Promise<boolean> => {
    try {
      if (typeof window === 'undefined' || !(window as any).PublicKeyCredential) {
        return false;
      }

      const challenge = crypto.getRandomValues(new Uint8Array(32));

      const credential = await (navigator.credentials as any).get({
        publicKey: {
          challenge,
          timeout: 60000,
          userVerification: 'preferred'
        }
      });

      if (credential) {
        // After passkey verification, still need to call our login
        // This is a simplified version - in production you'd verify the credential first
        console.log('Passkey verified, but still need vault password');
        return false;
      }

      return false;
    } catch (e) {
      console.warn('Passkey sign-in failed', e);
      return false;
    }
  };

  const updateProfile = async (updates: Partial<UserIdentity['profile']>): Promise<void> => {
    if (!authState.user) throw new Error('Not authenticated');

    // Update local state immediately (optimistic)
    setAuthState(prev => ({
      ...prev,
      user: prev.user ? {
        ...prev.user,
        profile: { ...prev.user.profile, ...updates }
      } : null,
    }));

    console.log('✅ Profile updated (local)');
  };

  const connectToNetwork = async (): Promise<boolean> => {
    try {
      // Use Tauri network commands
      await invoke('connect_to_network');
      console.log('✅ Connected to network');
      return true;
    } catch (error) {
      console.error('Network connection failed:', error);
      return false;
    }
  };

  const disconnectFromNetwork = async (): Promise<void> => {
    try {
      await invoke('disconnect_from_network');
      console.log('📵 Disconnected from network');
    } catch (error) {
      console.error('Network disconnection failed:', error);
    }
  };

  const getNetworkStatus = async (): Promise<{ connected: boolean; peers: number }> => {
    try {
      const status = await invoke('get_network_status') as { connected: boolean; peers: number };
      return status;
    } catch (error) {
      console.error('Failed to get network status:', error);
      return { connected: false, peers: 0 };
    }
  };

  // Permission utilities
  const hasPermission = (resource: string, action: string): boolean => {
    if (!authState.user) return false;

    return authState.user.permissions.some(
      permission =>
        permission.resource === resource &&
        permission.actions.includes(action)
    );
  };

  const isOwner = (resourceOwnerId: string): boolean => {
    return authState.user?.id === resourceOwnerId;
  };

  const canAccess = (resource: string, requiredPermissions: string[]): boolean => {
    if (!authState.user) return false;

    return requiredPermissions.every(permission =>
      hasPermission(resource, permission)
    );
  };

  // Configuration management
  const getConfig = async (): Promise<any> => {
    try {
      return await invoke('auth_get_config');
    } catch (error) {
      console.error('Failed to get config:', error);
      return null;
    }
  };

  const setAutoLogin = async (enabled: boolean): Promise<void> => {
    try {
      await invoke('auth_set_auto_login', { enabled });
      console.log(`Auto-login ${enabled ? 'enabled' : 'disabled'}`);
    } catch (error) {
      console.error('Failed to set auto-login:', error);
      throw error;
    }
  };

  const setKeyringEnabled = async (enabled: boolean): Promise<void> => {
    try {
      await invoke('auth_set_keyring_enabled', { enabled });
      console.log(`Keyring ${enabled ? 'enabled' : 'disabled'}`);
    } catch (error) {
      console.error('Failed to set keyring:', error);
      throw error;
    }
  };

  const getRecentIdentities = async (): Promise<any[]> => {
    try {
      return await invoke('auth_get_recent_identities');
    } catch (error) {
      console.error('Failed to get recent identities:', error);
      return [];
    }
  };

  // Check if this is first launch (no vaults exist)
  const isFirstLaunch = async (): Promise<boolean> => {
    try {
      const vaults = await listVaults();
      return vaults.length === 0;
    } catch (error) {
      console.error('Failed to check first launch:', error);
      return false;
    }
  };

  // Get OS username for default display name
  const getOsUsername = async (): Promise<string> => {
    try {
      const username = await invoke('get_os_username') as string;
      console.log('Got OS username:', username);
      return username;
    } catch (error) {
      console.error('Failed to get OS username:', error);
      return 'User';
    }
  };

  // Enable auto-login by storing password in keyring
  const enableAutoLogin = async (fourWords: string, password: string): Promise<void> => {
    try {
      // Register passkey stores password in keyring
      await invoke('auth_passkey_register', {
        fourWords,
        deviceName: `${navigator.platform} - Auto-login`,
      });

      // Enable auto-login in config
      await setAutoLogin(true);
      await setKeyringEnabled(true);

      console.log('✅ Auto-login enabled for:', fourWords);
    } catch (error) {
      console.error('Failed to enable auto-login:', error);
      throw error;
    }
  };

  const contextValue: AuthContextType = {
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
  };

  return (
    <AuthContext.Provider value={contextValue}>
      {children}
    </AuthContext.Provider>
  );
};

// Custom hook for using auth context
export const useAuth = (): AuthContextType => {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};

// HOC for protecting routes
export interface ProtectedRouteProps {
  children: ReactNode;
  requiredPermissions?: { resource: string; actions: string[] };
  fallback?: ReactNode;
}

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
