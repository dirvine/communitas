/**
 * Post-Quantum Cryptography Context
 * Provides PQC encryption and signing operations for the frontend
 */

import React, { createContext, useContext, useReducer, useEffect, useCallback } from 'react';
import { useAuth, UserIdentity } from './AuthContext';
import { 
  pqcCrypto, 
  PqcKeyPair, 
  PqcSignature, 
  PqcEncryptionResult, 
  PqcVerificationResult,
  PqcInfo 
} from '../utils/pqcCrypto';

// PQC state types
export interface PqcKeyManager {
  id: string;
  keyPair: PqcKeyPair;
  purpose: 'signing' | 'encryption' | 'identity';
  scope?: string; // organization:id or project:id
  createdAt: string;
  expiresAt?: string;
}

export interface PqcEncryptionState {
  isInitialized: boolean;
  userSigningKeyPair: PqcKeyPair | null;
  userEncryptionKeyPair: PqcKeyPair | null;
  sessionKeys: Map<string, PqcKeyManager>;
  sharedKeys: Map<string, PqcKeyPair>;
  pqcInfo: PqcInfo | null;
  loading: boolean;
  error: string | null;
  pqcEnabled: boolean;
}

export interface PqcEncryptionContextType {
  state: PqcEncryptionState;
  
  // Initialization
  initializePqc: (user: UserIdentity) => Promise<boolean>;
  
  // Key management
  generateSigningKeyPair: (scope?: string) => Promise<PqcKeyManager>;
  generateEncryptionKeyPair: (scope?: string) => Promise<PqcKeyManager>;
  getOrCreateSigningKey: (scope?: string) => Promise<PqcKeyManager>;
  getOrCreateEncryptionKey: (scope?: string) => Promise<PqcKeyManager>;
  
  // Signing operations
  signData: (data: string | Uint8Array, keyId?: string, context?: string) => Promise<PqcSignature>;
  verifySignature: (
    data: string | Uint8Array, 
    signature: PqcSignature, 
    publicKey: number[], 
    context?: string
  ) => Promise<PqcVerificationResult>;
  
  // Encryption operations
  encryptData: (data: string | Uint8Array, publicKey: number[]) => Promise<PqcEncryptionResult>;
  decryptData: (
    encryptedData: PqcEncryptionResult, 
    secretKey?: number[]
  ) => Promise<Uint8Array>;
  
  // File operations
  encryptFile: (file: File, publicKey: number[]) => Promise<{
    encryptedData: PqcEncryptionResult;
    metadata: {
      originalName: string;
      originalSize: number;
      mimeType: string;
    };
  }>;
  decryptFile: (
    encryptedData: PqcEncryptionResult,
    metadata: {
      originalName: string;
      mimeType: string;
    },
    secretKey?: number[]
  ) => Promise<File>;
  
  // Message operations
  signAndEncryptMessage: (
    message: string,
    recipientPublicKey: number[],
    context?: string
  ) => Promise<{
    encryptedData: PqcEncryptionResult;
    signature: PqcSignature;
  }>;
  
  verifyAndDecryptMessage: (
    encryptedData: PqcEncryptionResult,
    signature: PqcSignature,
    senderPublicKey: number[],
    context?: string
  ) => Promise<string>;
  
  // Key exchange
  exportSigningPublicKey: () => Promise<number[] | null>;
  exportEncryptionPublicKey: () => Promise<number[] | null>;
  
  // Utility
  getPqcInfo: () => Promise<PqcInfo>;
  clearPqc: () => void;
  togglePqc: (enabled: boolean) => void;
}

// Action types for PQC reducer
type PqcAction = 
  | { type: 'INITIALIZE_START' }
  | { type: 'INITIALIZE_SUCCESS'; payload: { 
      signingKeyPair: PqcKeyPair; 
      encryptionKeyPair: PqcKeyPair;
      pqcInfo: PqcInfo;
    } }
  | { type: 'INITIALIZE_ERROR'; payload: string }
  | { type: 'ADD_SESSION_KEY'; payload: PqcKeyManager }
  | { type: 'ADD_SHARED_KEY'; payload: { id: string; keyPair: PqcKeyPair } }
  | { type: 'SET_ERROR'; payload: string }
  | { type: 'CLEAR_ERROR' }
  | { type: 'CLEAR_ALL' }
  | { type: 'TOGGLE_PQC'; payload: boolean }
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_PQC_INFO'; payload: PqcInfo };

// Initial state
const initialState: PqcEncryptionState = {
  isInitialized: false,
  userSigningKeyPair: null,
  userEncryptionKeyPair: null,
  sessionKeys: new Map(),
  sharedKeys: new Map(),
  pqcInfo: null,
  loading: false,
  error: null,
  pqcEnabled: true,
};

// Reducer for PQC state management
function pqcReducer(state: PqcEncryptionState, action: PqcAction): PqcEncryptionState {
  switch (action.type) {
    case 'INITIALIZE_START':
      return { ...state, loading: true, error: null };
      
    case 'INITIALIZE_SUCCESS':
      return {
        ...state,
        isInitialized: true,
        userSigningKeyPair: action.payload.signingKeyPair,
        userEncryptionKeyPair: action.payload.encryptionKeyPair,
        pqcInfo: action.payload.pqcInfo,
        loading: false,
        error: null,
      };
      
    case 'INITIALIZE_ERROR':
      return {
        ...state,
        isInitialized: false,
        loading: false,
        error: action.payload,
      };
      
    case 'ADD_SESSION_KEY':
      const newSessionKeys = new Map(state.sessionKeys);
      newSessionKeys.set(action.payload.id, action.payload);
      return { ...state, sessionKeys: newSessionKeys };
      
    case 'ADD_SHARED_KEY':
      const newSharedKeys = new Map(state.sharedKeys);
      newSharedKeys.set(action.payload.id, action.payload.keyPair);
      return { ...state, sharedKeys: newSharedKeys };
      
    case 'SET_ERROR':
      return { ...state, error: action.payload, loading: false };
      
    case 'CLEAR_ERROR':
      return { ...state, error: null };
      
    case 'CLEAR_ALL':
      return { ...initialState };
      
    case 'TOGGLE_PQC':
      return { ...state, pqcEnabled: action.payload };
      
    case 'SET_LOADING':
      return { ...state, loading: action.payload };
      
    case 'SET_PQC_INFO':
      return { ...state, pqcInfo: action.payload };
      
    default:
      return state;
  }
}

// Create contexts
const PqcEncryptionContext = createContext<PqcEncryptionContextType | undefined>(undefined);

// Provider component
export function PqcEncryptionProvider({ children }: { children: React.ReactNode }) {
  const [state, dispatch] = useReducer(pqcReducer, initialState);
  const { user } = useAuth();

  // Initialize PQC when user logs in
  const initializePqc = useCallback(async (user: UserIdentity): Promise<boolean> => {
    if (!state.pqcEnabled) {
      return false;
    }

    dispatch({ type: 'INITIALIZE_START' });

    try {
      // Get PQC info
      const pqcInfo = await pqcCrypto.getPqcInfo();
      
      // Generate user key pairs
      const signingKeyPair = await pqcCrypto.generateSigningKeyPair();
      const encryptionKeyPair = await pqcCrypto.generateEncryptionKeyPair();

      dispatch({ 
        type: 'INITIALIZE_SUCCESS', 
        payload: { 
          signingKeyPair, 
          encryptionKeyPair,
          pqcInfo 
        } 
      });

      console.log('PQC initialized successfully for user:', user.fourWordAddress);
      return true;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown PQC initialization error';
      dispatch({ type: 'INITIALIZE_ERROR', payload: errorMessage });
      console.error('PQC initialization failed:', error);
      return false;
    }
  }, [state.pqcEnabled]);

  // Auto-initialize when user is available
  useEffect(() => {
    if (user && !state.isInitialized && state.pqcEnabled) {
      initializePqc(user);
    }
  }, [user, state.isInitialized, state.pqcEnabled, initializePqc]);

  // Key management functions
  const generateSigningKeyPair = useCallback(async (scope?: string): Promise<PqcKeyManager> => {
    const keyPair = await pqcCrypto.generateSigningKeyPair();
    const keyManager: PqcKeyManager = {
      id: `signing-${Date.now()}`,
      keyPair,
      purpose: 'signing',
      scope,
      createdAt: new Date().toISOString(),
    };
    
    dispatch({ type: 'ADD_SESSION_KEY', payload: keyManager });
    return keyManager;
  }, []);

  const generateEncryptionKeyPair = useCallback(async (scope?: string): Promise<PqcKeyManager> => {
    const keyPair = await pqcCrypto.generateEncryptionKeyPair();
    const keyManager: PqcKeyManager = {
      id: `encryption-${Date.now()}`,
      keyPair,
      purpose: 'encryption',
      scope,
      createdAt: new Date().toISOString(),
    };
    
    dispatch({ type: 'ADD_SESSION_KEY', payload: keyManager });
    return keyManager;
  }, []);

  const getOrCreateSigningKey = useCallback(async (scope?: string): Promise<PqcKeyManager> => {
    const keyId = `signing-${scope || 'default'}`;
    const existingKey = state.sessionKeys.get(keyId);
    
    if (existingKey) {
      return existingKey;
    }
    
    return await generateSigningKeyPair(scope);
  }, [state.sessionKeys, generateSigningKeyPair]);

  const getOrCreateEncryptionKey = useCallback(async (scope?: string): Promise<PqcKeyManager> => {
    const keyId = `encryption-${scope || 'default'}`;
    const existingKey = state.sessionKeys.get(keyId);
    
    if (existingKey) {
      return existingKey;
    }
    
    return await generateEncryptionKeyPair(scope);
  }, [state.sessionKeys, generateEncryptionKeyPair]);

  // Signing operations
  const signData = useCallback(async (
    data: string | Uint8Array,
    keyId?: string,
    context: string = 'default'
  ): Promise<PqcSignature> => {
    const dataBytes = typeof data === 'string' ? pqcCrypto.utils.stringToBytes(data) : data;
    
    let secretKey: number[];
    if (keyId) {
      const keyManager = state.sessionKeys.get(keyId);
      if (!keyManager || keyManager.purpose !== 'signing') {
        throw new Error('Invalid signing key ID');
      }
      secretKey = keyManager.keyPair.secret_key;
    } else if (state.userSigningKeyPair) {
      secretKey = state.userSigningKeyPair.secret_key;
    } else {
      throw new Error('No signing key available');
    }
    
    return await pqcCrypto.signData(dataBytes, secretKey, context);
  }, [state.sessionKeys, state.userSigningKeyPair]);

  const verifySignature = useCallback(async (
    data: string | Uint8Array,
    signature: PqcSignature,
    publicKey: number[],
    context: string = 'default'
  ): Promise<PqcVerificationResult> => {
    const dataBytes = typeof data === 'string' ? pqcCrypto.utils.stringToBytes(data) : data;
    return await pqcCrypto.verifySignature(dataBytes, signature.signature, publicKey, context);
  }, []);

  // Encryption operations
  const encryptData = useCallback(async (
    data: string | Uint8Array,
    publicKey: number[]
  ): Promise<PqcEncryptionResult> => {
    const dataBytes = typeof data === 'string' ? pqcCrypto.utils.stringToBytes(data) : data;
    return await pqcCrypto.encryptData(dataBytes, publicKey);
  }, []);

  const decryptData = useCallback(async (
    encryptedData: PqcEncryptionResult,
    secretKey?: number[]
  ): Promise<Uint8Array> => {
    const key = secretKey || state.userEncryptionKeyPair?.secret_key;
    if (!key) {
      throw new Error('No decryption key available');
    }
    
    return await pqcCrypto.decryptData(
      encryptedData.ciphertext,
      encryptedData.nonce,
      encryptedData.ml_kem_ciphertext,
      key
    );
  }, [state.userEncryptionKeyPair]);

  // File operations
  const encryptFile = useCallback(async (
    file: File,
    publicKey: number[]
  ) => {
    const result = await pqcCrypto.encryptFile(file, publicKey);
    return {
      encryptedData: result.encryptedData,
      metadata: {
        originalName: result.originalName,
        originalSize: result.originalSize,
        mimeType: result.mimeType,
      },
    };
  }, []);

  const decryptFile = useCallback(async (
    encryptedData: PqcEncryptionResult,
    metadata: { originalName: string; mimeType: string },
    secretKey?: number[]
  ): Promise<File> => {
    const key = secretKey || state.userEncryptionKeyPair?.secret_key;
    if (!key) {
      throw new Error('No decryption key available');
    }
    
    return await pqcCrypto.decryptFile(
      encryptedData,
      key,
      metadata.originalName,
      metadata.mimeType
    );
  }, [state.userEncryptionKeyPair]);

  // Message operations
  const signAndEncryptMessage = useCallback(async (
    message: string,
    recipientPublicKey: number[],
    context: string = 'message'
  ) => {
    if (!state.userSigningKeyPair) {
      throw new Error('No signing key available');
    }
    
    const messageBytes = pqcCrypto.utils.stringToBytes(message);
    return await pqcCrypto.signAndEncrypt(
      messageBytes,
      recipientPublicKey,
      state.userSigningKeyPair.secret_key,
      context
    );
  }, [state.userSigningKeyPair]);

  const verifyAndDecryptMessage = useCallback(async (
    encryptedData: PqcEncryptionResult,
    signature: PqcSignature,
    senderPublicKey: number[],
    context: string = 'message'
  ): Promise<string> => {
    if (!state.userEncryptionKeyPair) {
      throw new Error('No decryption key available');
    }
    
    const decryptedBytes = await pqcCrypto.verifyAndDecrypt(
      encryptedData,
      signature,
      state.userEncryptionKeyPair.secret_key,
      senderPublicKey,
      context
    );
    
    return pqcCrypto.utils.bytesToString(decryptedBytes);
  }, [state.userEncryptionKeyPair]);

  // Key export functions
  const exportSigningPublicKey = useCallback(async (): Promise<number[] | null> => {
    return state.userSigningKeyPair?.public_key || null;
  }, [state.userSigningKeyPair]);

  const exportEncryptionPublicKey = useCallback(async (): Promise<number[] | null> => {
    return state.userEncryptionKeyPair?.public_key || null;
  }, [state.userEncryptionKeyPair]);

  // Utility functions
  const getPqcInfo = useCallback(async (): Promise<PqcInfo> => {
    if (state.pqcInfo) {
      return state.pqcInfo;
    }
    
    const info = await pqcCrypto.getPqcInfo();
    dispatch({ type: 'SET_PQC_INFO', payload: info });
    return info;
  }, [state.pqcInfo]);

  const clearPqc = useCallback(() => {
    pqcCrypto.clearCache();
    dispatch({ type: 'CLEAR_ALL' });
  }, []);

  const togglePqc = useCallback((enabled: boolean) => {
    dispatch({ type: 'TOGGLE_PQC', payload: enabled });
  }, []);

  const contextValue: PqcEncryptionContextType = {
    state,
    initializePqc,
    generateSigningKeyPair,
    generateEncryptionKeyPair,
    getOrCreateSigningKey,
    getOrCreateEncryptionKey,
    signData,
    verifySignature,
    encryptData,
    decryptData,
    encryptFile,
    decryptFile,
    signAndEncryptMessage,
    verifyAndDecryptMessage,
    exportSigningPublicKey,
    exportEncryptionPublicKey,
    getPqcInfo,
    clearPqc,
    togglePqc,
  };

  return (
    <PqcEncryptionContext.Provider value={contextValue}>
      {children}
    </PqcEncryptionContext.Provider>
  );
}

// Hook to use PQC encryption context
export function usePqcEncryption(): PqcEncryptionContextType {
  const context = useContext(PqcEncryptionContext);
  if (context === undefined) {
    throw new Error('usePqcEncryption must be used within a PqcEncryptionProvider');
  }
  return context;
}

// Export context for direct access if needed
export { PqcEncryptionContext };