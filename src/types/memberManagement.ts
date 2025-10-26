// Member Management Types
// Interfaces for CRDT-based member management across all entity types

/** Entity types that support member management */
export type MemberEntityType = 'organization' | 'group' | 'channel' | 'project' | 'individual'

/** Member role types */
export type MemberRole = 'owner' | 'admin' | 'member' | 'guest'

/** Member information returned by the backend */
export interface MemberInfo {
  member_id: string
  role: MemberRole
  joined_at: number
  deleted: boolean
}

/** Runtime validator for API boundaries */
export function normalizeMemberRole(role: string): MemberRole {
  const validRoles: MemberRole[] = ['owner', 'admin', 'member', 'guest']
  return validRoles.includes(role as MemberRole) 
    ? (role as MemberRole) 
    : 'guest' // Default fallback
}

/** Request to add a new member */
export interface AddMemberRequest {
  entity_type: MemberEntityType
  entity_id: string
  member_id: string
  role: MemberRole
}

/** Request to remove a member */
export interface RemoveMemberRequest {
  entity_type: MemberEntityType
  entity_id: string
  member_id: string
  deleted_by: string
}

/** Request to update a member's role */
export interface UpdateRoleRequest {
  entity_type: MemberEntityType
  entity_id: string
  member_id: string
  new_role: MemberRole
}

/** Error types for member operations */
export enum MemberError {
  AlreadyExists = 'MEMBER_ALREADY_EXISTS',
  NotFound = 'MEMBER_NOT_FOUND',
  CrdtError = 'CRDT_ERROR',
  Unknown = 'UNKNOWN_ERROR',
}

/** Result wrapper for member operations */
export interface MemberOperationResult<T = void> {
  success: boolean
  data?: T
  error?: {
    type: MemberError
    message: string
  }
}
