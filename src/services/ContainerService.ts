// Copyright (c) 2025 Saorsa Labs Limited
//
// Container service for managing CRDT container operations

import { invoke } from '@tauri-apps/api/core';

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
 */
export class ContainerService {
  /**
   * Initialize the container engine with user's ML-DSA keys
   */
  static async init(): Promise<boolean> {
    try {
      return await invoke<boolean>('container_init');
    } catch (error) {
      console.error('Failed to initialize container:', error);
      throw error;
    }
  }

  /**
   * Store an object in the container (pointer-only, encrypted with AEAD)
   */
  static async putObject(bytes: Uint8Array): Promise<ObjectInfo> {
    try {
      return await invoke<ObjectInfo>('container_put_object', { bytes });
    } catch (error) {
      console.error('Failed to store object:', error);
      throw error;
    }
  }

  /**
   * Store a text object in the container
   */
  static async putText(text: string): Promise<ObjectInfo> {
    const encoder = new TextEncoder();
    const bytes = encoder.encode(text);
    return this.putObject(bytes);
  }

  /**
   * Retrieve an object from the container by OID
   *
   * Note: Tauri serializes Rust Vec<u8> as number[], so we convert to Uint8Array
   */
  static async getObject(oidHex: string): Promise<Uint8Array> {
    try {
      // Tauri serializes Vec<u8> as number[], not Uint8Array
      const rawBytes = await invoke<number[]>('container_get_object', { oidHex });
      // Convert to Uint8Array for proper BufferSource compatibility
      return new Uint8Array(rawBytes);
    } catch (error) {
      console.error('Failed to retrieve object:', error);
      throw error;
    }
  }

  /**
   * Retrieve a text object from the container
   */
  static async getText(oidHex: string): Promise<string> {
    const bytes = await this.getObject(oidHex);
    const decoder = new TextDecoder();
    // bytes is now guaranteed to be Uint8Array (BufferSource)
    return decoder.decode(bytes);
  }

  /**
   * Apply CRDT operations to the container
   */
  static async applyOps(opsJson: string): Promise<TipInfo> {
    try {
      return await invoke<TipInfo>('container_apply_ops', { opsJson });
    } catch (error) {
      console.error('Failed to apply operations:', error);
      throw error;
    }
  }

  /**
   * Get the current tip of the container CRDT
   */
  static async getCurrentTip(): Promise<TipInfo> {
    try {
      return await invoke<TipInfo>('container_current_tip');
    } catch (error) {
      console.error('Failed to get current tip:', error);
      throw error;
    }
  }

  /**
   * Create a new post operation
   */
  static async createPost(bodyMd: string): Promise<string> {
    try {
      return await invoke<string>('container_create_post', { bodyMd });
    } catch (error) {
      console.error('Failed to create post:', error);
      throw error;
    }
  }

  /**
   * List all objects in the container
   */
  static async listObjects(): Promise<ObjectInfo[]> {
    try {
      return await invoke<ObjectInfo[]>('container_list_objects');
    } catch (error) {
      console.error('Failed to list objects:', error);
      throw error;
    }
  }

  /**
   * Get container statistics
   */
  static async getStats(): Promise<ContainerStats> {
    try {
      return await invoke<ContainerStats>('container_get_stats');
    } catch (error) {
      console.error('Failed to get container stats:', error);
      throw error;
    }
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