/**
 * Storage System Type Definitions
 * Defines all types used in the Saorsa storage system
 */

export enum StoragePolicyType {
  PrivateMax = 'PrivateMax',
  PrivateScoped = 'PrivateScoped', 
  GroupScoped = 'GroupScoped',
  PublicMarkdown = 'PublicMarkdown'
}

export enum EncryptionMode {
  ChaCha20Poly1305Local = 'ChaCha20Poly1305Local',
  ChaCha20Poly1305Derived = 'ChaCha20Poly1305Derived',
  ChaCha20Poly1305Shared = 'ChaCha20Poly1305Shared',
  Convergent = 'Convergent'
}

export enum DeduplicationScope {
  None = 'None',
  User = 'User',
  Group = 'Group', 
  Global = 'Global'
}

export interface StoragePolicy {
  type: StoragePolicyType;
  encryptionMode: EncryptionMode;
  deduplicationScope: DeduplicationScope;
  allowSharing: boolean;
  requiresAudit?: boolean;
  namespace?: string;
  groupId?: string;
  maxContentSize?: number;
}

export interface EncryptedContent {
  ciphertext: Uint8Array;
  nonce: Uint8Array;
  contentAddress: string;
  algorithm: string;
}

export interface StorageMetadata {
  contentType: string;
  author: string;
  tags: string[];
  createdAt: Date;
  modifiedAt?: Date;
  size: number;
  checksum: string;
}

export interface ContentChunk {
  data: Uint8Array;
  index: number;
  totalChunks: number;
  checksum: string;
  address: string;
}

export interface StorageAddress {
  contentId: string;
  policy: StoragePolicyType;
  namespace?: string;
  groupId?: string;
}

export interface PolicyTransitionResult {
  requiresReEncryption: boolean;
  newContentAddress?: string;
  migrationTasks?: string[];
}

export interface GroupKeyInfo {
  keyId: string;
  encryptedKey: Uint8Array;
  createdAt: Date;
  version: number;
}

export interface NamespaceConfig {
  masterKey: Uint8Array;
  derivationInfo: string;
  version: number;
}

export interface StorageQuota {
  totalBytes: number;
  usedBytes: number;
  availableBytes: number;
  policyLimits: Record<StoragePolicyType, number>;
}

export interface StorageError extends Error {
  code: string;
  policy?: StoragePolicyType;
  contentAddress?: string;
  recoverable: boolean;
}

export class StoragePolicyError extends Error implements StorageError {
  code: string;
  policy?: StoragePolicyType;
  contentAddress?: string;
  recoverable: boolean;

  constructor(message: string, code: string, policy?: StoragePolicyType, recoverable = false) {
    super(message);
    this.name = 'StoragePolicyError';
    this.code = code;
    this.policy = policy;
    this.recoverable = recoverable;
  }
}

export class EncryptionError extends Error implements StorageError {
  code: string;
  policy?: StoragePolicyType;
  contentAddress?: string;
  recoverable: boolean;

  constructor(message: string, code: string, recoverable = false) {
    super(message);
    this.name = 'EncryptionError';
    this.code = code;
    this.recoverable = recoverable;
  }
}

export class NamespaceError extends Error implements StorageError {
  code: string;
  policy?: StoragePolicyType;
  contentAddress?: string;
  recoverable: boolean;

  constructor(message: string, code: string, recoverable = false) {
    super(message);
    this.name = 'NamespaceError';
    this.code = code;
    this.recoverable = recoverable;
  }
}