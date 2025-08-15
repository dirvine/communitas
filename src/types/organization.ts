export interface Permission {
  id: string
  name: string
  description: string
}

export type Role = 'Owner' | 'Admin' | 'Member' | 'Viewer' | 'Guest'

export interface RolePermissions {
  role: Role
  permissions: string[]
  inherited?: boolean
}

export interface Member {
  user_id: string
  display_name: string
  four_word_address: string
  email?: string
  role: Role
  joined_at: Date
  last_active?: Date
  profile_image?: string
}

export interface StorageQuota {
  allocated_gb: number
  used_gb: number
  available_gb: number
  last_updated: Date
}

export interface Organization {
  id: string
  name: string
  description?: string
  created_at: Date
  updated_at: Date
  
  // Ownership and permissions
  owner_id: string
  members: Member[]
  
  // Storage and file system
  has_file_system: true  // Organizations always have file systems
  storage_quota: StorageQuota
  
  // Settings
  settings: {
    visibility: 'public' | 'private' | 'invite_only'
    default_member_role: Role
    allow_member_invitations: boolean
    require_approval_for_joins: boolean
  }
  
  // Child entities
  groups: Group[]
  projects: Project[]
}

export interface Group {
  id: string
  name: string
  description?: string
  organization_id: string
  created_at: Date
  updated_at: Date
  
  // Ownership and permissions
  creator_id: string
  members: Member[]
  
  // Chat-only functionality
  has_file_system: false  // Groups never have file systems
  chat_settings: {
    message_retention_days: number
    allow_file_sharing: boolean  // Uses parent org storage
    allow_voice_messages: boolean
    allow_video_calls: boolean
  }
  
  // Inherited permissions from organization
  inherited_permissions: boolean
}

export interface Project {
  id: string
  name: string
  description?: string
  organization_id: string
  parent_group_id?: string  // Projects can optionally belong to a group
  created_at: Date
  updated_at: Date
  
  // Ownership and permissions
  owner_id: string
  members: Member[]
  
  // Project-specific file system
  has_file_system: true  // Projects always have file systems
  storage_quota: StorageQuota
  
  // Project management
  status: 'active' | 'paused' | 'completed' | 'archived'
  deadline?: Date
  priority: 'low' | 'medium' | 'high' | 'critical'
  
  // Collaboration settings
  settings: {
    allow_external_collaborators: boolean
    require_approval_for_changes: boolean
    version_control_enabled: boolean
    backup_enabled: boolean
  }
}

export interface OrganizationHierarchy {
  organization: Organization
  groups: Group[]
  projects: Project[]
  total_members: number
  total_storage_used_gb: number
}

export interface CreateOrganizationRequest {
  name: string
  description?: string
  visibility: 'public' | 'private' | 'invite_only'
  initial_storage_gb: number
}

export interface CreateGroupRequest {
  name: string
  description?: string
  organization_id: string
  initial_members?: string[]  // user_ids
}

export interface CreateProjectRequest {
  name: string
  description?: string
  organization_id: string
  parent_group_id?: string
  deadline?: Date
  priority: 'low' | 'medium' | 'high' | 'critical'
  initial_storage_gb: number
  initial_members?: string[]  // user_ids
}

export interface InviteMemberRequest {
  entity_type: 'organization' | 'group' | 'project'
  entity_id: string
  invitee_address: string  // four-word address or user ID
  role: Role
  message?: string
}

export interface PermissionCheck {
  entity_type: 'organization' | 'group' | 'project'
  entity_id: string
  user_id: string
  permission: string
  granted: boolean
  inherited_from?: {
    entity_type: 'organization' | 'group' | 'project'
    entity_id: string
  }
}
