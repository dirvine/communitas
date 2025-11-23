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
    return false;
  }

  static async putObject(_bytes: Uint8Array): Promise<ObjectInfo> {
    console.warn('⚠️  ContainerService.putObject() is deprecated.');
    return { oid_hex: '', size_bytes: 0, timestamp: new Date().toISOString() };
  }

  static async putText(_text: string): Promise<ObjectInfo> {
    console.warn('⚠️  ContainerService.putText() is deprecated.');
    return { oid_hex: '', size_bytes: 0, timestamp: new Date().toISOString() };
  }

  static async getObject(_oidHex: string): Promise<Uint8Array> {
    console.warn('⚠️  ContainerService.getObject() is deprecated.');
    return new Uint8Array();
  }

  static async getText(_oidHex: string): Promise<string> {
    console.warn('⚠️  ContainerService.getText() is deprecated.');
    return '';
  }

  static async applyOps(_opsJson: string): Promise<TipInfo> {
    console.warn('⚠️  ContainerService.applyOps() is deprecated.');
    return { root_hex: '', count: 0, signature_hex: '', timestamp: new Date().toISOString() };
  }

  static async getCurrentTip(): Promise<TipInfo> {
    console.warn('⚠️  ContainerService.getCurrentTip() is deprecated.');
    return { root_hex: '', count: 0, signature_hex: '', timestamp: new Date().toISOString() };
  }

  static async createPost(_bodyMd: string): Promise<string> {
    console.warn('⚠️  ContainerService.createPost() is deprecated.');
    return '';
  }

  static async listObjects(): Promise<ObjectInfo[]> {
    console.warn('⚠️  ContainerService.listObjects() is deprecated.');
    return [];
  }

  static async getStats(): Promise<ContainerStats> {
    console.warn('⚠️  ContainerService.getStats() is deprecated.');
    return { initialized: false, current_tip: { root_hex: '', count: 0, signature_hex: '' }, timestamp: new Date().toISOString() };
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