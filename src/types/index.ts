export interface NetworkHealth {
  status: string
  peer_count: number
  nat_type: string
  bandwidth_kbps: number
  avg_latency_ms: number
}

export interface Message {
  id: string
  content: string
  sender: string
  timestamp: Date
  group_id?: string
}

export interface UserPresence {
  user_id: string
  status: 'online' | 'away' | 'busy' | 'offline'
  last_seen: string
  activity?: string
}

export interface NetworkMetrics {
  connected_peers: number
  bandwidth_up_kbps: number
  bandwidth_down_kbps: number
  avg_latency_ms: number
  packet_loss_percent: number
}

export interface IdentityInfo {
  id: string
  address: string
  four_word_address: string
  display_name: string
  created_at: string
  is_active: boolean
  is_primary: boolean
  public_key_hex: string
  verification_status: 'verified' | 'unverified' | 'pending'
}

export interface StorageBackendInfo {
  backend_type: string
  is_available: boolean
  description: string
  key_count: number
}

export interface IdentityGenerationParams {
  display_name?: string
  use_secure_storage: boolean
  use_hardware_entropy?: boolean
}

// Organization hierarchy types
// Collaboration types (Group, Organization, etc.)
export { Group } from './collaboration'
// CRDT types
export * from './crdt'
// Document CRDT types
export * from './documents'
export * from './organization'



