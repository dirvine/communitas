import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, waitFor, cleanup } from '@testing-library/react'
import { MemberListPanel } from '../MemberListPanel'
import { memberManagementService } from '@/services/MemberManagementService'
import { MemberError } from '@/types/memberManagement'
import type { MemberInfo } from '@/types/memberManagement'

describe('MemberListPanel', () => {
  const mockMembers: MemberInfo[] = [
    {
      member_id: 'owner-four-words',
      role: 'owner',
      joined_at: Date.now() - 86400000 * 4,
      deleted: false
    },
    {
      member_id: 'admin-four-words',
      role: 'admin',
      joined_at: Date.now() - 86400000 * 3,
      deleted: false
    },
    {
      member_id: 'member-four-words',
      role: 'member',
      joined_at: Date.now() - 86400000 * 2,
      deleted: false
    }
  ]

  beforeEach(() => {
    vi.clearAllMocks()
    vi.spyOn(memberManagementService, 'listMembers')
    vi.spyOn(memberManagementService, 'addMember')
    vi.spyOn(memberManagementService, 'removeMember')
    vi.spyOn(memberManagementService, 'updateRole')
  })

  afterEach(() => {
    cleanup()
  })

  it('loads and displays member list on mount', async () => {
    vi.mocked(memberManagementService.listMembers).mockResolvedValue({
      success: true,
      data: mockMembers
    })

    render(
      <MemberListPanel
        entityType="group"
        entityId="group-123"
        currentUserId="admin-four-words"
        currentUserRole="admin"
      />
    )

    await waitFor(() => {
      expect(screen.getByText('owner-four-words')).toBeInTheDocument()
      expect(screen.getByText('admin-four-words')).toBeInTheDocument()
      expect(screen.getByText('member-four-words')).toBeInTheDocument()
    })
  })

  it('displays member count in header', async () => {
    vi.mocked(memberManagementService.listMembers).mockResolvedValue({
      success: true,
      data: mockMembers
    })

    render(
      <MemberListPanel
        entityType="group"
        entityId="group-123"
        currentUserId="admin-four-words"
        currentUserRole="admin"
      />
    )

    await waitFor(() => {
      expect(screen.getByText('Members (3)')).toBeInTheDocument()
    })
  })

  it('shows loading state while fetching members', () => {
    vi.mocked(memberManagementService.listMembers).mockImplementation(() =>
      new Promise(resolve => setTimeout(resolve, 1000))
    )

    render(
      <MemberListPanel
        entityType="group"
        entityId="group-123"
        currentUserId="admin-four-words"
        currentUserRole="admin"
      />
    )

    expect(screen.getByRole('progressbar')).toBeInTheDocument()
  })

  it('shows Add Member button for admins', async () => {
    vi.mocked(memberManagementService.listMembers).mockResolvedValue({
      success: true,
      data: []
    })

    render(
      <MemberListPanel
        entityType="group"
        entityId="group-123"
        currentUserId="admin-four-words"
        currentUserRole="admin"
      />
    )

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /add member/i })).toBeInTheDocument()
    })
  })

  it('shows Add Member button for owners', async () => {
    vi.mocked(memberManagementService.listMembers).mockResolvedValue({
      success: true,
      data: []
    })

    render(
      <MemberListPanel
        entityType="group"
        entityId="group-123"
        currentUserId="owner-four-words"
        currentUserRole="owner"
      />
    )

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /add member/i })).toBeInTheDocument()
    })
  })

  it('does not show Add Member button for regular members', async () => {
    vi.mocked(memberManagementService.listMembers).mockResolvedValue({
      success: true,
      data: mockMembers
    })

    render(
      <MemberListPanel
        entityType="group"
        entityId="group-123"
        currentUserId="member-four-words"
        currentUserRole="member"
      />
    )

    await waitFor(() => {
      expect(screen.queryByRole('button', { name: /add member/i })).not.toBeInTheDocument()
    })
  })

  it('does not show Add Member button for guests', async () => {
    vi.mocked(memberManagementService.listMembers).mockResolvedValue({
      success: true,
      data: mockMembers
    })

    render(
      <MemberListPanel
        entityType="group"
        entityId="group-123"
        currentUserId="guest-four-words"
        currentUserRole="guest"
      />
    )

    await waitFor(() => {
      expect(screen.queryByRole('button', { name: /add member/i })).not.toBeInTheDocument()
    })
  })

  it('opens AddMemberDialog when Add Member clicked', async () => {
    vi.mocked(memberManagementService.listMembers).mockResolvedValue({
      success: true,
      data: []
    })

    render(
      <MemberListPanel
        entityType="group"
        entityId="group-123"
        currentUserId="admin-four-words"
        currentUserRole="admin"
      />
    )

    await waitFor(() => {
      const addButton = screen.getByRole('button', { name: /add member/i })
      fireEvent.click(addButton)
    })

    // Dialog should appear
    expect(screen.getByRole('dialog')).toBeInTheDocument()
    expect(screen.getByLabelText('Four-Word Address')).toBeInTheDocument()
  })

  it('reloads member list after member added', async () => {
    vi.mocked(memberManagementService.listMembers)
      .mockResolvedValueOnce({ success: true, data: [] })
      .mockResolvedValueOnce({ success: true, data: [mockMembers[0]] })

    vi.mocked(memberManagementService.addMember).mockResolvedValue({
      success: true
    })

    render(
      <MemberListPanel
        entityType="group"
        entityId="group-123"
        currentUserId="admin-four-words"
        currentUserRole="admin"
      />
    )

    // Initial load
    await waitFor(() => {
      expect(screen.getByText('Members (0)')).toBeInTheDocument()
    })

    // Open dialog
    const addButton = screen.getByRole('button', { name: /add member/i })
    fireEvent.click(addButton)

    // Add member with valid format
    await waitFor(() => {
      const input = screen.getByLabelText('Four-Word Address')
      fireEvent.change(input, { target: { value: 'ocean-blue-eagle-star' } })
    })
    
    const submitButton = screen.getByRole('button', { name: /^add$/i })
    fireEvent.click(submitButton)

    // Should reload list
    await waitFor(() => {
      expect(memberManagementService.listMembers).toHaveBeenCalledTimes(2)
    }, { timeout: 3000 })
  })

  it('displays empty state when no members', async () => {
    vi.mocked(memberManagementService.listMembers).mockResolvedValue({
      success: true,
      data: []
    })

    render(
      <MemberListPanel
        entityType="group"
        entityId="group-123"
        currentUserId="owner-four-words"
        currentUserRole="owner"
      />
    )

    await waitFor(() => {
      expect(screen.getByText('Members (0)')).toBeInTheDocument()
    })
  })

  it('handles error loading members gracefully', async () => {
    vi.mocked(memberManagementService.listMembers).mockResolvedValue({
      success: false,
      error: {
        type: MemberError.Unknown,
        message: 'Failed to load members'
      }
    })

    render(
      <MemberListPanel
        entityType="group"
        entityId="group-123"
        currentUserId="admin-four-words"
        currentUserRole="admin"
      />
    )

    await waitFor(() => {
      // Should show error or empty state gracefully
      expect(screen.getByText('Members (0)')).toBeInTheDocument()
    })
  })

  it('reloads members when entityId changes', async () => {
    vi.mocked(memberManagementService.listMembers)
      .mockResolvedValue({ success: true, data: mockMembers })

    const { rerender } = render(
      <MemberListPanel
        entityType="group"
        entityId="group-123"
        currentUserId="admin-four-words"
        currentUserRole="admin"
      />
    )

    await waitFor(() => {
      expect(memberManagementService.listMembers).toHaveBeenCalledWith('group-123', 'group')
    })

    // Change entity
    rerender(
      <MemberListPanel
        entityType="group"
        entityId="group-456"
        currentUserId="admin-four-words"
        currentUserRole="admin"
      />
    )

    await waitFor(() => {
      expect(memberManagementService.listMembers).toHaveBeenCalledWith('group-456', 'group')
      expect(memberManagementService.listMembers).toHaveBeenCalledTimes(2)
    })
  })
})
