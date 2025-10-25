import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, waitFor, cleanup } from '@testing-library/react'
import { AddMemberDialog } from '../AddMemberDialog'
import { memberManagementService } from '@/services/MemberManagementService'
import { MemberError } from '@/types/memberManagement'

describe('AddMemberDialog', () => {
  const mockOnClose = vi.fn()
  const mockOnMemberAdded = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
    vi.spyOn(memberManagementService, 'addMember')
  })

  afterEach(() => {
    cleanup()
  })

  it('renders when open is true', () => {
    render(
      <AddMemberDialog
        open={true}
        onClose={mockOnClose}
        entityType="group"
        entityId="group-123"
        onMemberAdded={mockOnMemberAdded}
      />
    )

    expect(screen.getByText('Add Member')).toBeInTheDocument()
    expect(screen.getByLabelText('Four-Word Address')).toBeInTheDocument()
    expect(screen.getByRole('combobox')).toBeInTheDocument()
  })

  it('does not render when open is false', () => {
    render(
      <AddMemberDialog
        open={false}
        onClose={mockOnClose}
        entityType="group"
        entityId="group-123"
        onMemberAdded={mockOnMemberAdded}
      />
    )

    expect(screen.queryByText('Add Member')).not.toBeInTheDocument()
  })

  it('validates four-word address format', async () => {
    render(
      <AddMemberDialog
        open={true}
        onClose={mockOnClose}
        entityType="group"
        entityId="group-123"
        onMemberAdded={mockOnMemberAdded}
      />
    )

    const input = screen.getByLabelText('Four-Word Address')
    const submitButton = screen.getByRole('button', { name: /add/i })

    // Enter invalid format
    fireEvent.change(input, { target: { value: 'invalid' } })
    fireEvent.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText(/Invalid four-word address format/i)).toBeInTheDocument()
    })

    expect(mockOnMemberAdded).not.toHaveBeenCalled()
  })

  it('accepts valid four-word address format', async () => {
    vi.mocked(memberManagementService.addMember).mockResolvedValue({
      success: true
    })

    render(
      <AddMemberDialog
        open={true}
        onClose={mockOnClose}
        entityType="group"
        entityId="group-123"
        onMemberAdded={mockOnMemberAdded}
      />
    )

    const input = screen.getByLabelText('Four-Word Address')
    const submitButton = screen.getByRole('button', { name: /add/i })

    // Enter valid format
    fireEvent.change(input, { target: { value: 'ocean-blue-eagle-star' } })
    fireEvent.click(submitButton)

    await waitFor(() => {
      expect(memberManagementService.addMember).toHaveBeenCalledWith({
        entity_type: 'group',
        entity_id: 'group-123',
        member_id: 'ocean-blue-eagle-star',
        role: 'member',
        added_by: expect.any(String)
      })
    })

    expect(mockOnMemberAdded).toHaveBeenCalled()
    expect(mockOnClose).toHaveBeenCalled()
  })

  it('allows role selection', () => {
    render(
      <AddMemberDialog
        open={true}
        onClose={mockOnClose}
        entityType="group"
        entityId="group-123"
        onMemberAdded={mockOnMemberAdded}
      />
    )

    // Role select should exist with default value
    const roleInput = screen.getByRole('combobox')
    expect(roleInput).toBeInTheDocument()
    expect(roleInput).toHaveTextContent('Member - Standard access')
  })

  it('calls memberManagementService with selected role', async () => {
    vi.mocked(memberManagementService.addMember).mockResolvedValue({
      success: true
    })

    render(
      <AddMemberDialog
        open={true}
        onClose={mockOnClose}
        entityType="organization"
        entityId="org-456"
        onMemberAdded={mockOnMemberAdded}
      />
    )

    // Enter four-word address
    const input = screen.getByLabelText('Four-Word Address')
    fireEvent.change(input, { target: { value: 'apple-banana-cherry-date' } })

    // Submit with default member role
    const submitButton = screen.getByRole('button', { name: /add/i })
    fireEvent.click(submitButton)

    await waitFor(() => {
      expect(memberManagementService.addMember).toHaveBeenCalledWith(
        expect.objectContaining({
          entity_type: 'organization',
          entity_id: 'org-456',
          member_id: 'apple-banana-cherry-date',
          role: expect.any(String)
        })
      )
    })
  })

  it('displays error message when backend fails', async () => {
    vi.mocked(memberManagementService.addMember).mockResolvedValue({
      success: false,
      error: {
        type: MemberError.AlreadyExists,
        message: 'Member already exists in this group'
      }
    })

    render(
      <AddMemberDialog
        open={true}
        onClose={mockOnClose}
        entityType="group"
        entityId="group-123"
        onMemberAdded={mockOnMemberAdded}
      />
    )

    const input = screen.getByLabelText('Four-Word Address')
    fireEvent.change(input, { target: { value: 'ocean-blue-eagle-star' } })

    const submitButton = screen.getByRole('button', { name: /add/i })
    fireEvent.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText('Member already exists in this group')).toBeInTheDocument()
    })

    expect(mockOnMemberAdded).not.toHaveBeenCalled()
    expect(mockOnClose).not.toHaveBeenCalled()
  })

  it('disables submit button while loading', async () => {
    vi.mocked(memberManagementService.addMember).mockImplementation(() => 
      new Promise(resolve => setTimeout(resolve, 1000))
    )

    render(
      <AddMemberDialog
        open={true}
        onClose={mockOnClose}
        entityType="group"
        entityId="group-123"
        onMemberAdded={mockOnMemberAdded}
      />
    )

    const input = screen.getByLabelText('Four-Word Address')
    fireEvent.change(input, { target: { value: 'ocean-blue-eagle-star' } })

    const submitButton = screen.getByRole('button', { name: /add/i })
    fireEvent.click(submitButton)

    // Button should be disabled during loading
    expect(submitButton).toBeDisabled()
  })

  it('closes dialog on cancel button click', () => {
    render(
      <AddMemberDialog
        open={true}
        onClose={mockOnClose}
        entityType="group"
        entityId="group-123"
        onMemberAdded={mockOnMemberAdded}
      />
    )

    const cancelButton = screen.getByRole('button', { name: /cancel/i })
    fireEvent.click(cancelButton)

    expect(mockOnClose).toHaveBeenCalled()
  })

  it('resets form when dialog is closed and reopened', async () => {
    const { rerender } = render(
      <AddMemberDialog
        open={true}
        onClose={mockOnClose}
        entityType="group"
        entityId="group-123"
        onMemberAdded={mockOnMemberAdded}
      />
    )

    // Enter some data
    const input = screen.getByLabelText('Four-Word Address')
    fireEvent.change(input, { target: { value: 'test-words' } })

    // Close dialog
    rerender(
      <AddMemberDialog
        open={false}
        onClose={mockOnClose}
        entityType="group"
        entityId="group-123"
        onMemberAdded={mockOnMemberAdded}
      />
    )

    // Reopen dialog
    rerender(
      <AddMemberDialog
        open={true}
        onClose={mockOnClose}
        entityType="group"
        entityId="group-123"
        onMemberAdded={mockOnMemberAdded}
      />
    )

    // Input should be reset
    const reopenedInput = screen.getByLabelText('Four-Word Address') as HTMLInputElement
    expect(reopenedInput.value).toBe('')
  })
})
