// Copyright (c) 2025 Saorsa Labs Limited

export interface ObjectInfo {
  oid_hex: string;
  size_bytes: number;
  timestamp: string;
}

export interface TipInfo {
  root_hex: string;
  count: number;
  signature_hex: string;
  timestamp: string;
}

export interface ContainerStats {
  initialized: boolean;
  current_tip: {
    root_hex: string;
    count: number;
    signature_hex: string;
  };
  timestamp: string;
}

/**
 * Service for managing container engine operations
 * 
 * @deprecated This service is deprecated. Use DocumentService with Yrs CRDT instead.
 * The container_* commands are no longer supported in the backend.
 */
export class ContainerService {
  /**
   * Initialize the container engine with user's ML-DSA keys
   * 
   * @deprecated Backend container_init command has been removed. Use doc_* commands instead.
   */
  static async init(): Promise<boolean> {
    console.warn('⚠️  ContainerService.init() is deprecated. Backend command no longer exists.');
    throw new Error('DEPRECATED: container_init removed. Use Yrs CRDT document commands instead.');
  }

  /**
   * Store an object in the container (pointer-only, encrypted with AEAD)
   * 
   * @deprecated Backend container_put_object command has been removed.
   */
  static async putObject(_bytes: Uint8Array): Promise<ObjectInfo> {
    console.warn('⚠️  ContainerService.putObject() is deprecated.');
    throw new Error('DEPRECATED: container_put_object removed. Use Yrs CRDT document commands instead.');
  }

  /**
   * Store a text object in the container
   * 
   * @deprecated Backend container commands have been removed.
   */
  static async putText(_text: string): Promise<ObjectInfo> {
    console.warn('⚠️  ContainerService.putText() is deprecated.');
    throw new Error('DEPRECATED: Use Yrs CRDT document commands instead.');
  }

  /**
   * Retrieve an object from the container by OID
   * 
   * @deprecated Backend container_get_object command has been removed.
   */
  static async getObject(_oidHex: string): Promise<Uint8Array> {
    console.warn('⚠️  ContainerService.getObject() is deprecated.');
    throw new Error('DEPRECATED: container_get_object removed. Use Yrs CRDT document commands instead.');
  }

  /**
   * Retrieve a text object from the container
   * 
   * @deprecated Backend container commands have been removed.
   */
  static async getText(_oidHex: string): Promise<string> {
    console.warn('⚠️  ContainerService.getText() is deprecated.');
    throw new Error('DEPRECATED: Use Yrs CRDT document commands instead.');
  }

  /**
   * Apply CRDT operations to the container
   * 
   * @deprecated Backend container_apply_ops command has been removed.
   */
  static async applyOps(_opsJson: string): Promise<TipInfo> {
    console.warn('⚠️  ContainerService.applyOps() is deprecated.');
    throw new Error('DEPRECATED: container_apply_ops removed. Use Yrs CRDT document commands instead.');
  }

  /**
   * Get the current tip of the container CRDT
   * 
   * @deprecated Backend container_current_tip command has been removed.
   */
  static async getCurrentTip(): Promise<TipInfo> {
    console.warn('⚠️  ContainerService.getCurrentTip() is deprecated.');
    throw new Error('DEPRECATED: container_current_tip removed. Use Yrs CRDT document commands instead.');
  }

  /**
   * Create a new post operation
   * 
   * @deprecated Backend container_create_post command has been removed.
   */
  static async createPost(_bodyMd: string): Promise<string> {
    console.warn('⚠️  ContainerService.createPost() is deprecated.');
    throw new Error('DEPRECATED: container_create_post removed. Use Yrs CRDT document commands instead.');
  }

  /**
   * List all objects in the container
   * 
   * @deprecated Backend container_list_objects command has been removed.
   */
  static async listObjects(): Promise<ObjectInfo[]> {
    console.warn('⚠️  ContainerService.listObjects() is deprecated.');
    throw new Error('DEPRECATED: container_list_objects removed. Use Yrs CRDT document commands instead.');
  }

  /**
   * Get container statistics
   * 
   * @deprecated Backend container_get_stats command has been removed.
   */
  static async getStats(): Promise<ContainerStats> {
    console.warn('⚠️  ContainerService.getStats() is deprecated.');
    throw new Error('DEPRECATED: container_get_stats removed. Use Yrs CRDT document commands instead.');
  }

  /**
   * Convert Uint8Array to hex string
   */
  static bytesToHex(bytes: Uint8Array): string {
    return Array.from(bytes)
      .map(b => b.toString(16).padStart(2, '0'))
      .join('');
  }

  /**
   * Convert hex string to Uint8Array
   */
  static hexToBytes(hex: string): Uint8Array {
    const bytes = new Uint8Array(hex.length / 2);
    for (let i = 0; i < hex.length; i += 2) {
      bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
    }
    return bytes;
  }

  /**
   * Format file size for display
   */
  static formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  /**
   * Format timestamp for display
   */
  static formatTimestamp(timestamp: string): string {
    try {
      const date = new Date(timestamp);
      return date.toLocaleString();
    } catch {
      return timestamp;
    }
  }
}