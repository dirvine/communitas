/**
 * TypeScript types for Saorsa Storage System
 * Frontend integration types for the Tauri storage commands
 */

// Storage Policy Types
export type StoragePolicy = 
  | "PrivateMax"
  | { PrivateScoped: { namespace: string } }
  | { GroupScoped: { group_id: string } }
  | "PublicMarkdown";

// Storage Metadata
export interface StorageMetadata {
  content_type: string;
  author: string;
  tags: string[];
  created_at: string; // ISO datetime string
  modified_at?: string | null;
  size: number;
  checksum: string;
}

// Storage Address
export interface StorageAddress {
  content_id: string;
  policy: StoragePolicy;
  namespace?: string | null;
  group_id?: string | null;
}

// Storage Location
export type StorageLocation = 
  | "Local"
  | { Dht: { replicas: number } }
  | { Group: { members: string[] } }
  | "Public";

// Retrieval Source
export type RetrievalSource = 
  | "Cache"
  | "Local"
  | "Dht"
  | "Group"
  | { Reconstructed: { from_chunks: number } };

// Request/Response Types for Tauri Commands

export interface StorageInitRequest {
  master_key_hex: string;
  config_path?: string | null;
}

export interface FrontendStorageRequest {
  content: number[]; // Vec<u8> as array of numbers
  content_type: string;
  policy: StoragePolicy;
  author: string;
  tags: string[];
  user_id: string;
  group_id?: string | null;
  namespace?: string | null;
}

export interface StorageResponse {
  address: StorageAddress;
  chunks_stored: number;
  total_size: number;
  encrypted_size: number;
  operation_time_ms: number;
  storage_location: StorageLocation;
}

export interface FrontendRetrievalRequest {
  address: StorageAddress;
  user_id: string;
  decryption_key_hex?: string | null;
}

export interface RetrievalResponse {
  content: number[]; // Vec<u8> as array of numbers
  metadata: StorageMetadata;
  source: RetrievalSource;
  operation_time_ms: number;
}

export interface ContentListRequest {
  user_id: string;
  policy_filter?: StoragePolicy | null;
  limit?: number | null;
}

export interface StorageEngineStats {
  total_content_items: number;
  total_bytes_stored: number;
  cache_hit_ratio: number;
  network_operations: number;
  successful_operations: number;
  failed_operations: number;
  avg_operation_time_ms: number;
  policy_distribution: Record<string, number>;
  last_updated: string; // ISO datetime string
}

export interface StorageErrorResponse {
  error_type: string;
  message: string;
  details?: string | null;
}

// Utility Types

export interface StoragePolicyLimits {
  max_content_size?: number | null;
  allows_binary_content: boolean;
  allows_sharing: boolean;
  requires_namespace_key: boolean;
  requires_group_key: boolean;
  requires_audit: boolean;
}

export interface StorageQuota {
  total_bytes: number;
  used_bytes: number;
  available_bytes: number;
  policy_limits: Record<string, number>;
}

// Helper functions for working with policies

export function getPolicyLimits(policy: StoragePolicy): StoragePolicyLimits {
  switch (policy) {
    case "PrivateMax":
      return {
        max_content_size: 100 * 1024 * 1024, // 100MB
        allows_binary_content: true,
        allows_sharing: false,
        requires_namespace_key: false,
        requires_group_key: false,
        requires_audit: false,
      };
    case "PublicMarkdown":
      return {
        max_content_size: 10 * 1024 * 1024, // 10MB
        allows_binary_content: false,
        allows_sharing: true,
        requires_namespace_key: false,
        requires_group_key: false,
        requires_audit: true,
      };
    default:
      if (typeof policy === 'object' && 'PrivateScoped' in policy) {
        return {
          max_content_size: 1024 * 1024 * 1024, // 1GB
          allows_binary_content: true,
          allows_sharing: false,
          requires_namespace_key: true,
          requires_group_key: false,
          requires_audit: false,
        };
      } else if (typeof policy === 'object' && 'GroupScoped' in policy) {
        return {
          max_content_size: 5 * 1024 * 1024 * 1024, // 5GB
          allows_binary_content: true,
          allows_sharing: true,
          requires_namespace_key: false,
          requires_group_key: true,
          requires_audit: false,
        };
      }
      throw new Error(`Unknown storage policy: ${JSON.stringify(policy)}`);
  }
}

export function validateContent(
  content: Uint8Array | number[],
  contentType: string,
  policy: StoragePolicy
): { valid: boolean; error?: string } {
  const limits = getPolicyLimits(policy);
  const size = content.length;

  // Check size limits
  if (limits.max_content_size && size > limits.max_content_size) {
    return {
      valid: false,
      error: `Content size ${size} bytes exceeds policy limit ${limits.max_content_size} bytes`,
    };
  }

  // Check binary content restrictions
  if (!limits.allows_binary_content && contentType !== 'text/markdown') {
    return {
      valid: false,
      error: 'Policy does not allow binary content',
    };
  }

  return { valid: true };
}

export function formatPolicyName(policy: StoragePolicy): string {
  switch (policy) {
    case "PrivateMax":
      return "Private Max Security";
    case "PublicMarkdown":
      return "Public Markdown";
    default:
      if (typeof policy === 'object' && 'PrivateScoped' in policy) {
        return `Private Scoped (${policy.PrivateScoped.namespace})`;
      } else if (typeof policy === 'object' && 'GroupScoped' in policy) {
        return `Group Scoped (${policy.GroupScoped.group_id})`;
      }
      return "Unknown Policy";
  }
}

export function formatStorageSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
}

export function formatOperationTime(ms: number): string {
  if (ms < 1000) {
    return `${ms}ms`;
  } else if (ms < 60000) {
    return `${(ms / 1000).toFixed(1)}s`;
  } else {
    return `${(ms / 60000).toFixed(1)}m`;
  }
}

// Content type utilities
export function inferContentType(filename: string): string {
  const ext = filename.toLowerCase().split('.').pop();
  
  const typeMap: Record<string, string> = {
    'md': 'text/markdown',
    'txt': 'text/plain',
    'json': 'application/json',
    'pdf': 'application/pdf',
    'jpg': 'image/jpeg',
    'jpeg': 'image/jpeg',
    'png': 'image/png',
    'gif': 'image/gif',
    'mp4': 'video/mp4',
    'mp3': 'audio/mpeg',
    'wav': 'audio/wav',
    'zip': 'application/zip',
    'doc': 'application/msword',
    'docx': 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
  };

  return typeMap[ext || ''] || 'application/octet-stream';
}

// Conversion utilities
export function uint8ArrayToNumbers(data: Uint8Array): number[] {
  return Array.from(data);
}

export function numbersToUint8Array(data: number[]): Uint8Array {
  return new Uint8Array(data);
}

export function stringToBytes(str: string): number[] {
  return uint8ArrayToNumbers(new TextEncoder().encode(str));
}

export function bytesToString(bytes: number[]): string {
  return new TextDecoder().decode(numbersToUint8Array(bytes));
}