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

export interface Group {
  id: string
  name: string
  members: string[]
  created_at: Date
}

export interface StorageMetrics {
  total_messages: number
  storage_used_mb: number
  dht_operations: number
  replication_factor: number
}

export interface NetworkMetrics {
  connected_peers: number
  bandwidth_up_kbps: number
  bandwidth_down_kbps: number
  avg_latency_ms: number
  packet_loss_percent: number
}

export interface IdentityInfo {
  four_word_address: string
  display_name?: string
  created_at: string
  is_primary: boolean
  verification_status: string
  public_key_hex: string
}

export interface StorageBackendInfo {
  backend_type: string
  key_count: number
  is_available: boolean
}

export interface IdentityGenerationParams {
  display_name?: string
  use_hardware_entropy: boolean
  pow_difficulty: number
}
EOF < /dev/null