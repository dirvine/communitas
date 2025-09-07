import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { offlineStorage } from '../services/storage/OfflineStorageService';

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
    console.warn(`Tauri not available, simulating ${cmd}`, args);
    
    // Simulate successful responses for development
    if (cmd === 'core_initialize') {
      return true;
    }
    if (cmd === 'authenticate_user') {
      return {
        id: 'mock_id',
        fourWordAddress: args?.fourWordAddress || 'mock-address',
        publicKey: 'mock_public_key',
        privateKey: 'mock_private_key',
        profile: { name: 'Mock User' },
        permissions: [],
        createdAt: new Date(),
        lastActive: new Date(),
        isVerified: true,
        networkStatus: 'connected',
      };
    }
    
    throw new Error(`Command ${cmd} not available in mock mode`);
  }
};

// User identity interface
export interface UserIdentity {
  id: string;
  publicKey: string;
  fourWordAddress: string;
  name: string;
  avatar?: string;
  email?: string;
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

// Authentication context
export interface AuthContextType {
  // State
  authState: AuthState;
  
  // Authentication methods
  login: (fourWordAddress: string, password?: string) => Promise<boolean>;
  logout: () => Promise<void>;
  createIdentity: (name: string, email?: string, options?: { fourWords?: string; publicKey?: number[]; seedPhrase?: string; dhtPacket?: unknown; password?: string }) => Promise<UserIdentity>;
  registerPasskey: () => Promise<boolean>;
  signInWithPasskey: () => Promise<boolean>;
  
  // Identity management
  updateProfile: (updates: Partial<UserIdentity['profile']>) => Promise<void>;
  updatePermissions: (permissions: Permission[]) => Promise<void>;
  
  // Cryptographic operations
  signMessage: (message: string) => Promise<string>;
  verifySignature: (message: string, signature: string, publicKey: string) => Promise<boolean>;
  
  // Network identity
  connectToNetwork: () => Promise<boolean>;
  disconnectFromNetwork: () => Promise<void>;
  getNetworkStatus: () => Promise<{ connected: boolean; peers: number }>;
  
  // Utility methods
  hasPermission: (resource: string, action: string) => boolean;
  isOwner: (resourceOwnerId: string) => boolean;
  canAccess: (resource: string, requiredPermissions: string[]) => boolean;
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
      setAuthState(prev => ({ ...prev, loading: true, error: null }));
      
      // First check offline storage for cached identity
      const cachedIdentity = await offlineStorage.get<UserIdentity>('current_identity');
      
      if (cachedIdentity) {
        console.log('📦 Restored identity from offline storage');
        setAuthState({
          isAuthenticated: true,
          user: cachedIdentity,
          loading: false,
          error: null,
        });
      }
      
      // Try to get from backend (will update cache if online)
      try {
        const existingSession = await (invoke('get_current_identity') as Promise<UserIdentity | null>);
        
        if (existingSession) {
          // Update offline storage
          await offlineStorage.store('current_identity', existingSession, {
            encrypt: true,
            syncOnline: true
          });
          
          // Verify the session is still valid
          const networkStatus = await getNetworkStatus();
          
          setAuthState({
            isAuthenticated: true,
            user: existingSession,
            loading: false,
            error: null,
          });
          
          if (networkStatus.connected) {
            console.log('✅ Authentication restored, connected to network');
          } else {
            console.log('📵 Authentication restored, working offline');
          }
        } else if (!cachedIdentity) {
          setAuthState(prev => ({ ...prev, loading: false }));
        }
      } catch (backendError) {
        // Backend not available, but we have cached identity
        if (cachedIdentity) {
          console.log('📵 Backend unavailable, using cached identity');
        } else {
          setAuthState(prev => ({ ...prev, loading: false }));
        }
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

      // Attempt to connect with the provided credentials
      let identity: UserIdentity | null = null
      try {
        identity = await (invoke('authenticate_identity', { fourWordAddress, password }) as Promise<UserIdentity>)
      } catch {
        // Fallback in browser/dev when backend not available: use offline storage
        const cached = await offlineStorage.get<UserIdentity>('current_identity')
        if (cached && cached.fourWordAddress === fourWordAddress) {
          identity = cached
        }
      }
      if (!identity) throw new Error('Invalid credentials or offline')

      // Connect to the P2P network
      const connected = await connectToNetwork();
      
      setAuthState({
        isAuthenticated: true,
        user: {
          ...identity,
          lastActive: new Date().toISOString(),
        },
        loading: false,
        error: null,
      });

      // Store session for persistence
      await invoke('store_identity_session', { identity });
      
      // Also store in offline storage
      await offlineStorage.store('current_identity', identity, {
        encrypt: true,
        syncOnline: true
      });
      
      console.log('✅ Login successful:', identity.fourWordAddress);
      return connected;
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
      // Disconnect from network
      await disconnectFromNetwork();
      
      // Clear stored session
      await invoke('clear_identity_session');
      
      // Clear offline storage
      await offlineStorage.store('current_identity', null);
      
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

  // Passkey registration and sign-in (backend-aware; browser fallback)
  const registerPasskey = async (): Promise<boolean> => {
    try {
      try { return await (invoke('register_passkey') as Promise<boolean>); } catch {}
      if (typeof window === 'undefined' || !(window as any).PublicKeyCredential) return false;
      const challenge = Uint8Array.from(crypto.getRandomValues(new Uint8Array(32)));
      const cred = await (navigator.credentials as any).create({
        publicKey: {
          challenge,
          rp: { name: 'Communitas' },
          user: { id: Uint8Array.from(crypto.getRandomValues(new Uint8Array(16))), name: 'user', displayName: 'Communitas User' },
          pubKeyCredParams: [{ type: 'public-key', alg: -7 }],
          timeout: 60000,
          authenticatorSelection: { userVerification: 'preferred' },
        },
      });
      await offlineStorage.store('passkey_registered', true);
      return !!cred;
    } catch (e) {
      console.warn('Passkey registration failed', e);
      return false;
    }
  };

  const signInWithPasskey = async (): Promise<boolean> => {
    try {
      try { return await (invoke('sign_in_with_passkey') as Promise<boolean>); } catch {}
      if (typeof window === 'undefined' || !(window as any).PublicKeyCredential) return false;
      const challenge = Uint8Array.from(crypto.getRandomValues(new Uint8Array(32)));
      await (navigator.credentials as any).get({ publicKey: { challenge, timeout: 60000, userVerification: 'preferred' } });
      const cached = await offlineStorage.get<UserIdentity>('current_identity');
      if (cached) {
        setAuthState({ isAuthenticated: true, user: cached, loading: false, error: null });
        return true;
      }
      return false;
    } catch (e) {
      console.warn('Passkey sign-in failed', e);
      return false;
    }
  };

  const createIdentity = async (name: string, email?: string, options?: { fourWords?: string; publicKey?: number[]; seedPhrase?: string; dhtPacket?: unknown; password?: string }): Promise<UserIdentity> => {
    try {
      setAuthState(prev => ({ ...prev, loading: true, error: null }));

      // Four words from options or fallback generator
      let fourWordAddress = options?.fourWords || ''
      if (!fourWordAddress) {
        const words = ['ocean', 'forest', 'mountain', 'river', 'valley', 'desert', 'island', 'lake', 'meadow', 'prairie', 'canyon', 'glacier', 'savanna', 'tundra', 'reef', 'delta'];
        const pick = () => words[Math.floor(Math.random() * words.length)];
        fourWordAddress = `${pick()}-${pick()}-${pick()}-${pick()}`;
      }

      // Initialize the core context with this identity
      const success = await (invoke('core_initialize', {
        fourWords: fourWordAddress,
        displayName: name,
        deviceName: 'Desktop',
        deviceType: 'Desktop'
      }) as Promise<boolean>);

      if (!success) {
        throw new Error('Failed to initialize identity');
      }

      // Create the identity object
      const newIdentity: UserIdentity = {
        id: `id_${Date.now()}`,
        name,
        email,
        fourWordAddress,
        publicKey: 'generated',
        profile: {},
        permissions: [],
        createdAt: new Date().toISOString(),
        lastActive: new Date().toISOString(),
      };

      // Auto-login with the new identity
      setAuthState(prev => ({
        ...prev,
        isAuthenticated: true,
        user: newIdentity,
        loading: false,
        error: null,
      }));
      
      // Store in offline storage
      await offlineStorage.store('current_identity', newIdentity, {
        encrypt: true,
        syncOnline: true
      });
      // Password-based locator + encrypted profile (browser fallback; backend optional)
      if (options?.password) {
        try {
          const enc = new TextEncoder()
          const salt = crypto.getRandomValues(new Uint8Array(16))
          const keyMaterial = await crypto.subtle.importKey('raw', enc.encode(options.password), { name: 'PBKDF2' }, false, ['deriveKey'])
          const key = await crypto.subtle.deriveKey({ name: 'PBKDF2', salt, iterations: 100_000, hash: 'SHA-256' }, keyMaterial, { name: 'AES-GCM', length: 256 }, false, ['encrypt'])
          const iv = crypto.getRandomValues(new Uint8Array(12))
          const payload = enc.encode(JSON.stringify({ fourWordAddress, name, createdAt: Date.now() }))
          const ciphertext = new Uint8Array(await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, key, payload))
          const locatorBytes = new Uint8Array(await crypto.subtle.digest('SHA-256', enc.encode(options.password)))
          const locator = Array.from(locatorBytes).map(b => b.toString(16).padStart(2,'0')).join('')
          const blob = { salt: Array.from(salt), iv: Array.from(iv), ciphertext: Array.from(ciphertext) }
          try { await invoke('store_encrypted_profile', { locator, blob }); } catch {}
          await offlineStorage.store('password_locator', { locator, blob })
        } catch (e) {
          console.warn('Password-based storage failed:', e)
        }
      }
      if (options?.dhtPacket) {
        try { await invoke('store_identity_packet', { packet: options.dhtPacket }); } catch {}
      }
      
      console.log('✅ Identity created:', newIdentity.fourWordAddress);
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

  const updateProfile = async (updates: Partial<UserIdentity['profile']>): Promise<void> => {
    if (!authState.user) throw new Error('Not authenticated');

    try {
      const updatedUser = await (invoke('update_user_profile', {
        userId: authState.user.id,
        updates,
      }) as Promise<UserIdentity>);

      setAuthState(prev => ({
        ...prev,
        user: updatedUser,
      }));

      console.log('✅ Profile updated');
    } catch (error) {
      console.error('Profile update failed:', error);
      throw error;
    }
  };

  const updatePermissions = async (permissions: Permission[]): Promise<void> => {
    if (!authState.user) throw new Error('Not authenticated');

    try {
      const updatedUser = await (invoke('update_user_permissions', {
        userId: authState.user.id,
        permissions,
      }) as Promise<UserIdentity>);

      setAuthState(prev => ({
        ...prev,
        user: updatedUser,
      }));

      console.log('✅ Permissions updated');
    } catch (error) {
      console.error('Permission update failed:', error);
      throw error;
    }
  };

  const signMessage = async (message: string): Promise<string> => {
    if (!authState.user) throw new Error('Not authenticated');

    try {
      return await (invoke('sign_message', {
        message,
        userId: authState.user.id,
      }) as Promise<string>);
    } catch (error) {
      console.error('Message signing failed:', error);
      throw error;
    }
  };

  const verifySignature = async (
    message: string,
    signature: string,
    publicKey: string
  ): Promise<boolean> => {
    try {
      return await (invoke('verify_signature', {
        message,
        signature,
        publicKey,
      }) as Promise<boolean>);
    } catch (error) {
      console.error('Signature verification failed:', error);
      return false;
    }
  };

  const connectToNetwork = async (): Promise<boolean> => {
    try {
      return await (invoke('connect_to_network') as Promise<boolean>);
    } catch (error) {
      console.error('Network connection failed:', error);
      return false;
    }
  };

  const disconnectFromNetwork = async (): Promise<void> => {
    try {
      await invoke('disconnect_from_network');
    } catch (error) {
      console.error('Network disconnection failed:', error);
    }
  };

  const getNetworkStatus = async (): Promise<{ connected: boolean; peers: number }> => {
    try {
      return await (invoke('get_network_status') as Promise<{ connected: boolean; peers: number }>);
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

  // Provide default no-op until functions are defined above (ensure in-scope)
  const contextValue: AuthContextType = {
    authState,
    login,
    logout,
    createIdentity,
    registerPasskey: registerPasskey || (async () => false),
    signInWithPasskey: signInWithPasskey || (async () => false),
    updateProfile,
    updatePermissions,
    signMessage,
    verifySignature,
    connectToNetwork,
    disconnectFromNetwork,
    getNetworkStatus,
    hasPermission,
    isOwner,
    canAccess,
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
