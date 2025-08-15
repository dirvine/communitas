import React, { createContext, useContext, useReducer, useEffect, useCallback } from 'react';
import { useAuth, UserIdentity } from './AuthContext';
import { 
  cryptoManager, 
  KeyPair, 
  DerivedKey, 
  EncryptedData,
  encryptUserData,
  decryptUserData 
} from '../utils/crypto';
import { invoke } from '@tauri-apps/api/core';

// Encryption state types
export interface EncryptionKey {
  id: string;
  key: CryptoKey;
  purpose: 'master' | 'session' | 'file' | 'message' | 'storage';
  scope?: string; // organization:id or project:id
  createdAt: string;
  expiresAt?: string;
}

export interface EncryptionState {
  isInitialized: boolean;
  masterKey: CryptoKey | null;
  userKeyPair: KeyPair | null;
  sessionKeys: Map<string, EncryptionKey>;
  sharedKeys: Map<string, DerivedKey>;
  loading: boolean;
  error: string | null;
  encryptionEnabled: boolean;
}

export interface EncryptionContextType {
  state: EncryptionState;
  
  // Key management
  initializeEncryption: (user: UserIdentity) => Promise<boolean>;
  generateSessionKey: (scope?: string) => Promise<EncryptionKey>;
  deriveSharedKey: (otherUserPublicKey: string, purpose?: string) => Promise<DerivedKey>;
  getOrCreateKey: (purpose: string, scope?: string) => Promise<EncryptionKey>;
  
  // Encryption operations
  encryptText: (text: string, keyId?: string) => Promise<string>;
  decryptText: (encryptedData: string, keyId?: string) => Promise<string>;
  encryptFile: (file: File, keyId?: string) => Promise<EncryptedData>;
  encryptObject: (obj: any, keyId?: string) => Promise<string>;
  decryptObject: <T = any>(encryptedData: string, keyId?: string) => Promise<T>;
  
  // Key exchange
  exportPublicKey: () => Promise<string>;
  importPublicKey: (publicKeyString: string) => Promise<CryptoKey>;
  
  // Storage operations
  encryptForStorage: (data: any, scope?: string) => Promise<string>;
  decryptFromStorage: <T = any>(encryptedData: string, scope?: string) => Promise<T>;
  
  // Utility
  generateSecureHash: (data: string | ArrayBuffer) => Promise<string>;
  clearEncryption: () => void;
  toggleEncryption: (enabled: boolean) => void;
}

// Action types for encryption reducer
type EncryptionAction = 
  | { type: 'INITIALIZE_START' }
  | { type: 'INITIALIZE_SUCCESS'; payload: { masterKey: CryptoKey; keyPair: KeyPair } }
  | { type: 'INITIALIZE_ERROR'; payload: string }
  | { type: 'ADD_SESSION_KEY'; payload: EncryptionKey }
  | { type: 'ADD_SHARED_KEY'; payload: { id: string; key: DerivedKey } }
  | { type: 'SET_ERROR'; payload: string }
  | { type: 'CLEAR_ERROR' }
  | { type: 'CLEAR_ALL' }
  | { type: 'TOGGLE_ENCRYPTION'; payload: boolean }
  | { type: 'SET_LOADING'; payload: boolean };

// Initial state
const initialState: EncryptionState = {
  isInitialized: false,
  masterKey: null,
  userKeyPair: null,
  sessionKeys: new Map(),
  sharedKeys: new Map(),
  loading: false,
  error: null,
  encryptionEnabled: true,
};

// Reducer for encryption state management
function encryptionReducer(state: EncryptionState, action: EncryptionAction): EncryptionState {
  switch (action.type) {
    case 'INITIALIZE_START':
      return { ...state, loading: true, error: null };
      
    case 'INITIALIZE_SUCCESS':
      return {
        ...state,
        isInitialized: true,
        masterKey: action.payload.masterKey,
        userKeyPair: action.payload.keyPair,
        loading: false,
        error: null,
      };
      
    case 'INITIALIZE_ERROR':
      return {
        ...state,
        loading: false,
        error: action.payload,
        isInitialized: false,
      };
      
    case 'ADD_SESSION_KEY':
      const newSessionKeys = new Map(state.sessionKeys);
      newSessionKeys.set(action.payload.id, action.payload);
      return { ...state, sessionKeys: newSessionKeys };
      
    case 'ADD_SHARED_KEY':
      const newSharedKeys = new Map(state.sharedKeys);
      newSharedKeys.set(action.payload.id, action.payload.key);
      return { ...state, sharedKeys: newSharedKeys };
      
    case 'SET_ERROR':
      return { ...state, error: action.payload, loading: false };
      
    case 'CLEAR_ERROR':
      return { ...state, error: null };
      
    case 'TOGGLE_ENCRYPTION':
      return { ...state, encryptionEnabled: action.payload };
      
    case 'SET_LOADING':
      return { ...state, loading: action.payload };
      
    case 'CLEAR_ALL':
      return { ...initialState };
      
    default:
      return state;
  }
}

// Create encryption context
const EncryptionContext = createContext<EncryptionContextType | null>(null);

// Encryption provider component
export const EncryptionProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [state, dispatch] = useReducer(encryptionReducer, initialState);
  const { authState } = useAuth();

  // Initialize encryption when user logs in
  useEffect(() => {
    if (authState.isAuthenticated && authState.user && !state.isInitialized) {
      initializeEncryption(authState.user);
    } else if (!authState.isAuthenticated && state.isInitialized) {
      clearEncryption();
    }
  }, [authState.isAuthenticated, authState.user]);

  const initializeEncryption = useCallback(async (user: UserIdentity): Promise<boolean> => {
    dispatch({ type: 'INITIALIZE_START' });
    
    try {
      // Try to load existing keys from secure storage
      let masterKey: CryptoKey | null = null;
      let userKeyPair: KeyPair | null = null;
      
      try {
        const storedKeys = await invoke<{
          masterKey?: string;
          keyPair?: string;
        }>('get_encryption_keys', { userId: user.id });
        
        if (storedKeys.masterKey) {
          const masterKeyData = JSON.parse(storedKeys.masterKey);
          masterKey = await cryptoManager.importKey(masterKeyData);
        }
        
        if (storedKeys.keyPair) {
          const keyPairData = JSON.parse(storedKeys.keyPair);
          userKeyPair = {
            publicKey: await cryptoManager.importKey(
              keyPairData.publicKey, 
              'ECDH', 
              ['deriveKey', 'deriveBits']
            ),
            privateKey: await cryptoManager.importKey(
              keyPairData.privateKey, 
              'ECDH', 
              ['deriveKey', 'deriveBits']
            ),
            keyId: keyPairData.keyId,
            createdAt: keyPairData.createdAt,
            purpose: keyPairData.purpose,
          };
        }
      } catch (error) {
        console.log('No existing keys found, creating new ones');
      }
      
      // Create new keys if none exist
      if (!masterKey || !userKeyPair) {
        const encryptionContext = await cryptoManager.createUserEncryptionContext(user);
        masterKey = encryptionContext.masterKey;
        userKeyPair = encryptionContext.keyPair;
        
        // Store keys securely
        await invoke('store_encryption_keys', {
          userId: user.id,
          masterKey: JSON.stringify(await cryptoManager.exportKey(masterKey, 'raw')),
          keyPair: JSON.stringify({
            publicKey: await cryptoManager.exportKey(userKeyPair.publicKey, 'spki'),
            privateKey: await cryptoManager.exportKey(userKeyPair.privateKey, 'pkcs8'),
            keyId: userKeyPair.keyId,
            createdAt: userKeyPair.createdAt,
            purpose: userKeyPair.purpose,
          }),
        });
      }
      
      dispatch({ 
        type: 'INITIALIZE_SUCCESS', 
        payload: { masterKey, keyPair: userKeyPair } 
      });
      
      return true;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to initialize encryption';
      dispatch({ type: 'INITIALIZE_ERROR', payload: errorMessage });
      console.error('Encryption initialization failed:', error);
      return false;
    }
  }, []);

  const generateSessionKey = useCallback(async (scope?: string): Promise<EncryptionKey> => {
    if (!state.masterKey) {
      throw new Error('Encryption not initialized');
    }

    const sessionKey = await cryptoManager.generateEncryptionKey();
    const keyId = await cryptoManager.generateKeyId();
    
    const encryptionKey: EncryptionKey = {
      id: keyId,
      key: sessionKey,
      purpose: 'session',
      scope,
      createdAt: new Date().toISOString(),
      expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(), // 24 hours
    };
    
    dispatch({ type: 'ADD_SESSION_KEY', payload: encryptionKey });
    return encryptionKey;
  }, [state.masterKey]);

  const deriveSharedKey = useCallback(async (
    otherUserPublicKey: string,
    purpose: string = 'shared-encryption'
  ): Promise<DerivedKey> => {
    if (!state.userKeyPair) {
      throw new Error('User key pair not available');
    }

    // Import the other user's public key
    const otherPublicKey = await cryptoManager.importKey(
      JSON.parse(otherUserPublicKey),
      'ECDH',
      []
    );

    const sharedKey = await cryptoManager.deriveSharedKey(
      state.userKeyPair.privateKey,
      otherPublicKey,
      purpose
    );

    dispatch({ 
      type: 'ADD_SHARED_KEY', 
      payload: { id: sharedKey.keyId, key: sharedKey } 
    });

    return sharedKey;
  }, [state.userKeyPair]);

  const getOrCreateKey = useCallback(async (
    purpose: string,
    scope?: string
  ): Promise<EncryptionKey> => {
    // Look for existing key
    for (const [, key] of state.sessionKeys) {
      if (key.purpose === purpose && key.scope === scope) {
        // Check if key hasn't expired
        if (!key.expiresAt || new Date(key.expiresAt) > new Date()) {
          return key;
        }
      }
    }

    // Create new key
    return await generateSessionKey(scope);
  }, [state.sessionKeys, generateSessionKey]);

  const encryptText = useCallback(async (
    text: string,
    keyId?: string
  ): Promise<string> => {
    if (!state.encryptionEnabled) {
      return text; // Return unencrypted if disabled
    }

    let key: CryptoKey;
    
    if (keyId) {
      const encryptionKey = state.sessionKeys.get(keyId);
      if (!encryptionKey) {
        throw new Error(`Key ${keyId} not found`);
      }
      key = encryptionKey.key;
    } else if (state.masterKey) {
      key = state.masterKey;
    } else {
      throw new Error('No encryption key available');
    }

    return await cryptoManager.encryptText(text, key, keyId);
  }, [state.encryptionEnabled, state.masterKey, state.sessionKeys]);

  const decryptText = useCallback(async (
    encryptedData: string,
    keyId?: string
  ): Promise<string> => {
    if (!state.encryptionEnabled) {
      return encryptedData; // Return as-is if encryption disabled
    }

    let key: CryptoKey;
    
    if (keyId) {
      const encryptionKey = state.sessionKeys.get(keyId);
      if (!encryptionKey) {
        throw new Error(`Key ${keyId} not found`);
      }
      key = encryptionKey.key;
    } else if (state.masterKey) {
      key = state.masterKey;
    } else {
      throw new Error('No decryption key available');
    }

    return await cryptoManager.decryptText(encryptedData, key);
  }, [state.encryptionEnabled, state.masterKey, state.sessionKeys]);

  const encryptFile = useCallback(async (
    file: File,
    keyId?: string
  ): Promise<EncryptedData> => {
    let key: CryptoKey;
    
    if (keyId) {
      const encryptionKey = state.sessionKeys.get(keyId);
      if (!encryptionKey) {
        throw new Error(`Key ${keyId} not found`);
      }
      key = encryptionKey.key;
    } else if (state.masterKey) {
      key = state.masterKey;
    } else {
      throw new Error('No encryption key available');
    }

    return await cryptoManager.encryptFile(file, key);
  }, [state.masterKey, state.sessionKeys]);

  const encryptObject = useCallback(async (
    obj: any,
    keyId?: string
  ): Promise<string> => {
    const jsonString = JSON.stringify(obj);
    return await encryptText(jsonString, keyId);
  }, [encryptText]);

  const decryptObject = useCallback(async <T = any>(
    encryptedData: string,
    keyId?: string
  ): Promise<T> => {
    const jsonString = await decryptText(encryptedData, keyId);
    return JSON.parse(jsonString);
  }, [decryptText]);

  const exportPublicKey = useCallback(async (): Promise<string> => {
    if (!state.userKeyPair) {
      throw new Error('User key pair not available');
    }

    const exportedKey = await cryptoManager.exportKey(state.userKeyPair.publicKey, 'spki');
    return JSON.stringify(exportedKey);
  }, [state.userKeyPair]);

  const importPublicKey = useCallback(async (publicKeyString: string): Promise<CryptoKey> => {
    const keyData = JSON.parse(publicKeyString);
    return await cryptoManager.importKey(keyData, 'ECDH', []);
  }, []);

  const encryptForStorage = useCallback(async (
    data: any,
    scope?: string
  ): Promise<string> => {
    const key = await getOrCreateKey('storage', scope);
    return await encryptObject(data, key.id);
  }, [getOrCreateKey, encryptObject]);

  const decryptFromStorage = useCallback(async <T = any>(
    encryptedData: string,
    scope?: string
  ): Promise<T> => {
    // Try to find the appropriate key
    for (const [, key] of state.sessionKeys) {
      if (key.purpose === 'storage' && key.scope === scope) {
        try {
          return await decryptObject<T>(encryptedData, key.id);
        } catch (error) {
          // Try next key if this one fails
          continue;
        }
      }
    }

    // Fallback to master key
    return await decryptObject<T>(encryptedData);
  }, [state.sessionKeys, decryptObject]);

  const generateSecureHash = useCallback(async (
    data: string | ArrayBuffer
  ): Promise<string> => {
    return await cryptoManager.generateContentHash(data);
  }, []);

  const clearEncryption = useCallback((): void => {
    cryptoManager.clearCache();
    dispatch({ type: 'CLEAR_ALL' });
  }, []);

  const toggleEncryption = useCallback((enabled: boolean): void => {
    dispatch({ type: 'TOGGLE_ENCRYPTION', payload: enabled });
  }, []);

  const contextValue: EncryptionContextType = {
    state,
    initializeEncryption,
    generateSessionKey,
    deriveSharedKey,
    getOrCreateKey,
    encryptText,
    decryptText,
    encryptFile,
    encryptObject,
    decryptObject,
    exportPublicKey,
    importPublicKey,
    encryptForStorage,
    decryptFromStorage,
    generateSecureHash,
    clearEncryption,
    toggleEncryption,
  };

  return (
    <EncryptionContext.Provider value={contextValue}>
      {children}
    </EncryptionContext.Provider>
  );
};

// Custom hook to use encryption context
export const useEncryption = (): EncryptionContextType => {
  const context = useContext(EncryptionContext);
  if (!context) {
    throw new Error('useEncryption must be used within an EncryptionProvider');
  }
  return context;
};

export default EncryptionContext;