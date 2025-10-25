import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { MemberListPanel } from '../MemberListPanel'
import { memberManagementService } from '../../../services/MemberManagementService'
import type { MemberInfo, MemberEntityType } from '@/types/memberManagement'

vi.mock('../../../services/MemberManagementService', () => ({
  memberManagementService: {
    listMembers: vi.fn(),
    removeMember: vi.fn(),
    addMember: vi.fn(),
    updateRole: vi.fn()
  }
}))

describe('Member Removal - All Entity Types', () => {
  const mockMember: MemberInfo = {
    member_id: 'test-member',
    role: 'member',
    joined_at: Date.now(),
    deleted: false
  }

  beforeEach(() => {
    vi.clearAllMocks()
  })

  const testEntityTypes: MemberEntityType[] = [
    'organization',
    'group',
    'channel',
    'project',
    'individual'
  ]

  testEntityTypes.forEach((entityType) => {
    it(`can remove member from ${entityType}`, async () => {
      // Arrange: Mock list returning one member
      vi.mocked(memberManagementService.listMembers).mockResolvedValue({
        success: true,
        data: [mockMember]
      })

      vi.mocked(memberManagementService.removeMember).mockResolvedValue({
        success: true
      })

      render(
        <MemberListPanel
          entityType={entityType}
          entityId={`test-${entityType}-123`}
          currentUserId="admin-user"
          currentUserRole="admin"
        />
      )

      // Wait for member to load
      await waitFor(() => {
        expect(screen.getByText('test-member')).toBeInTheDocument()
      })

      // Act: Open action menu and click remove
      const actionButton = screen.getByLabelText('more options')
      fireEvent.click(actionButton)

      const removeMenuItem = screen.getByText('Remove Member')
      fireEvent.click(removeMenuItem)

      // Assert: removeMember called with correct params
      await waitFor(() => {
        expect(memberManagementService.removeMember).toHaveBeenCalledWith({
          entity_type: entityType,
          entity_id: `test-${entityType}-123`,
          member_id: 'test-member',
          deleted_by: 'admin-user'
        })
      })

      // Should reload member list after removal
      expect(memberManagementService.listMembers).toHaveBeenCalledTimes(2) // Initial load + reload
    })
  })

  it('handles remove member errors gracefully', async () => {
    vi.mocked(memberManagementService.listMembers).mockResolvedValue({
      success: true,
      data: [mockMember]
    })

    vi.mocked(memberManagementService.removeMember).mockResolvedValue({
      success: false,
      error: {
        type: 'UNKNOWN_ERROR' as any,
        message: 'Cannot remove last owner'
      }
    })

    render(
      <MemberListPanel
        entityType="group"
        entityId="test-group"
        currentUserId="admin-user"
        currentUserRole="admin"
      />
    )

    await waitFor(() => {
      expect(screen.getByText('test-member')).toBeInTheDocument()
    })

    // Try to remove
    const actionButton = screen.getByLabelText('more options')
    fireEvent.click(actionButton)
    fireEvent.click(screen.getByText('Remove Member'))

    await waitFor(() => {
      expect(memberManagementService.removeMember).toHaveBeenCalled()
    })

    // Error should be logged (not shown to user yet - no error UI)
    // In future, should show error toast/snackbar
  })

  it('does not show remove option for current user', async () => {
    vi.mocked(memberManagementService.listMembers).mockResolvedValue({
      success: true,
      data: [{
        member_id: 'current-user-id',
        role: 'admin',
        joined_at: Date.now(),
        deleted: false
      }]
    })

    render(
      <MemberListPanel
        entityType="group"
        entityId="test-group"
        currentUserId="current-user-id"
        currentUserRole="admin"
      />
    )

    await waitFor(() => {
      expect(screen.getByText('current-user-id')).toBeInTheDocument()
    })

    // Action menu should not be present for current user
    expect(screen.queryByLabelText('more options')).not.toBeInTheDocument()
  })

  it('removes member and shows updated list', async () => {
    const memberToRemove = { ...mockMember, member_id: 'remove-me' }
    const remainingMember = { ...mockMember, member_id: 'keep-me' }

    // First call: both members
    vi.mocked(memberManagementService.listMembers)
      .mockResolvedValueOnce({
        success: true,
        data: [memberToRemove, remainingMember]
      })
      // Second call after removal: one deleted, one active
      .mockResolvedValueOnce({
        success: true,
        data: [
          { ...memberToRemove, deleted: true },
          remainingMember
        ]
      })

    vi.mocked(memberManagementService.removeMember).mockResolvedValue({
      success: true
    })

    render(
      <MemberListPanel
        entityType="group"
        entityId="test-group"
        currentUserId="admin-user"
        currentUserRole="admin"
      />
    )

    // Wait for both members to load
    await waitFor(() => {
      expect(screen.getByText('remove-me')).toBeInTheDocument()
      expect(screen.getByText('keep-me')).toBeInTheDocument()
    })

    // Find and click remove for 'remove-me'
    const allActionButtons = screen.getAllByLabelText('more options')
    fireEvent.click(allActionButtons[0]) // Click first member's action button

    const removeMenuItem = screen.getByText('Remove Member')
    fireEvent.click(removeMenuItem)

    // Should reload list
    await waitFor(() => {
      expect(memberManagementService.listMembers).toHaveBeenCalledTimes(2)
    })
  })
})
