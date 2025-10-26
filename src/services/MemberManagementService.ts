// Member Management Service
// Provides a clean TypeScript interface to the CRDT-based member management backend

import { invoke } from '@tauri-apps/api/core'
import type {
    AddMemberRequest, MemberEntityType,
    MemberInfo, MemberOperationResult, RemoveMemberRequest,
    UpdateRoleRequest
} from '../types/memberManagement'
import { MemberError, normalizeMemberRole } from '../types/memberManagement'

class MemberManagementService {
  /**
   * Add a member to an entity
   */
  async addMember(request: AddMemberRequest): Promise<MemberOperationResult> {
    try {
      await invoke('member_add', { request })
      return { success: true }
    } catch (error) {
      return this.handleError(error)
    }
  }

  /**
   * List all members of an entity
   */
  async listMembers(
    entityType: MemberEntityType,
    entityId: string
  ): Promise<MemberOperationResult<MemberInfo[]>> {
    try {
      const members = await invoke<MemberInfo[]>('member_list', {
        entityType,
        entityId,
      })
      // Normalize roles from backend
      const normalized = members.map(m => ({
        ...m,
        role: normalizeMemberRole(m.role as unknown as string)
      }))
      return {
        success: true,
        data: normalized,
      }
    } catch (error) {
      return this.handleError(error)
    }
  }

  /**
   * Remove a member from an entity (creates tombstone)
   */
  async removeMember(request: RemoveMemberRequest): Promise<MemberOperationResult> {
    try {
      await invoke('member_remove', { request })
      return { success: true }
    } catch (error) {
      return this.handleError(error)
    }
  }

  /**
   * Update a member's role
   */
  async updateRole(request: UpdateRoleRequest): Promise<MemberOperationResult> {
    try {
      await invoke('member_update_role', { request })
      return { success: true }
    } catch (error) {
      return this.handleError(error)
    }
  }

  /**
   * Prune old tombstones from an entity
   * Returns the number of tombstones pruned
   */
  async pruneTombstones(
    entityType: MemberEntityType,
    entityId: string
  ): Promise<MemberOperationResult<number>> {
    try {
      const count = await invoke<number>('member_prune_tombstones', {
        entityType,
        entityId,
      })
      return {
        success: true,
        data: count,
      }
    } catch (error) {
      return this.handleError(error)
    }
  }

  /**
   * Get only active (non-deleted) members
   */
  async getActiveMembers(
    entityType: MemberEntityType,
    entityId: string
  ): Promise<MemberOperationResult<MemberInfo[]>> {
    const result = await this.listMembers(entityType, entityId)
    if (result.success && result.data) {
      return {
        success: true,
        data: result.data.filter((m) => !m.deleted),
      }
    }
    return result
  }

  /**
   * Check if a member exists in an entity
   */
  async isMember(
    entityType: MemberEntityType,
    entityId: string,
    memberId: string
  ): Promise<boolean> {
    const result = await this.getActiveMembers(entityType, entityId)
    if (result.success && result.data) {
      return result.data.some((m) => m.member_id === memberId)
    }
    return false
  }

  /**
   * Get member count for an entity
   */
  async getMemberCount(entityType: MemberEntityType, entityId: string): Promise<number> {
    const result = await this.getActiveMembers(entityType, entityId)
    return result.success && result.data ? result.data.length : 0
  }

  /**
   * Handle errors from Tauri backend
   */
  private handleError<T = void>(error: unknown): MemberOperationResult<T> {
    const errorString = String(error)

    // Parse error type from backend error message
    let errorType: MemberError = MemberError.Unknown

    if (errorString.includes('already exists')) {
      errorType = MemberError.AlreadyExists
    } else if (errorString.includes('not found')) {
      errorType = MemberError.NotFound
    } else if (errorString.includes('CRDT')) {
      errorType = MemberError.CrdtError
    }

    return {
      success: false,
      error: {
        type: errorType,
        message: errorString,
      },
    }
  }
}

// Export singleton instance
export const memberManagementService = new MemberManagementService()
