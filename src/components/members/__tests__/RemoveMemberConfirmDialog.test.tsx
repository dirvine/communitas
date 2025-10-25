import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { RemoveMemberConfirmDialog } from '../RemoveMemberConfirmDialog'

describe('RemoveMemberConfirmDialog', () => {
  const mockOnConfirm = vi.fn()
  const mockOnCancel = vi.fn()

  it('renders when open is true', () => {
    render(
      <RemoveMemberConfirmDialog
        open={true}
        memberName="ocean-blue-eagle-star"
        onConfirm={mockOnConfirm}
        onCancel={mockOnCancel}
      />
    )

    expect(screen.getByText('Remove Member')).toBeInTheDocument()
    expect(screen.getByText(/Are you sure/)).toBeInTheDocument()
    expect(screen.getByText('ocean-blue-eagle-star')).toBeInTheDocument()
  })

  it('does not render when open is false', () => {
    render(
      <RemoveMemberConfirmDialog
        open={false}
        memberName="ocean-blue-eagle-star"
        onConfirm={mockOnConfirm}
        onCancel={mockOnCancel}
      />
    )

    expect(screen.queryByText('Remove Member')).not.toBeInTheDocument()
  })

  it('calls onCancel when Cancel clicked', () => {
    render(
      <RemoveMemberConfirmDialog
        open={true}
        memberName="test-member"
        onConfirm={mockOnConfirm}
        onCancel={mockOnCancel}
      />
    )

    const cancelButton = screen.getByRole('button', { name: /cancel/i })
    fireEvent.click(cancelButton)

    expect(mockOnCancel).toHaveBeenCalled()
    expect(mockOnConfirm).not.toHaveBeenCalled()
  })

  it('calls onConfirm when Remove clicked', () => {
    const onConfirm = vi.fn()
    const onCancel = vi.fn()
    
    render(
      <RemoveMemberConfirmDialog
        open={true}
        memberName="test-member"
        onConfirm={onConfirm}
        onCancel={onCancel}
      />
    )

    // Get all buttons and find the Remove button (not Cancel)
    const buttons = screen.getAllByRole('button')
    const removeButton = buttons.find(btn => btn.textContent === 'Remove')
    expect(removeButton).toBeDefined()
    
    fireEvent.click(removeButton!)

    expect(onConfirm).toHaveBeenCalled()
    expect(onCancel).not.toHaveBeenCalled()
  })

  it('shows warning color on confirm button', () => {
    render(
      <RemoveMemberConfirmDialog
        open={true}
        memberName="test-member"
        onConfirm={mockOnConfirm}
        onCancel={mockOnCancel}
      />
    )

    const removeButton = screen.getByRole('button', { name: /remove/i })
    expect(removeButton).toHaveClass('MuiButton-colorError')
  })

  it('displays member name in warning message', () => {
    render(
      <RemoveMemberConfirmDialog
        open={true}
        memberName="specific-user-name"
        onConfirm={mockOnConfirm}
        onCancel={mockOnCancel}
      />
    )

    expect(screen.getByText(/specific-user-name/)).toBeInTheDocument()
  })
})
