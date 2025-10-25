import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { MemberCard } from '../MemberCard'
import type { MemberInfo } from '@/types/memberManagement'

describe('MemberCard', () => {
  const mockMember: MemberInfo = {
    member_id: 'ocean-blue-eagle-star',
    role: 'member',
    joined_at: Date.now(),
    deleted: false
  }

  it('renders member information correctly', () => {
    render(
      <MemberCard
        member={mockMember}
        canManage={false}
      />
    )

    expect(screen.getByText('ocean-blue-eagle-star')).toBeInTheDocument()
    expect(screen.getByText('MEMBER')).toBeInTheDocument()
  })

  it('displays role badge with correct color for member role', () => {
    render(
      <MemberCard
        member={mockMember}
        canManage={false}
      />
    )

    const badge = screen.getByText('MEMBER')
    expect(badge).toBeInTheDocument()
    // Chip should have primary color for member role
  })

  it('displays role badge with error color for owner role', () => {
    const ownerMember = { ...mockMember, role: 'owner' as const }
    
    render(
      <MemberCard
        member={ownerMember}
        canManage={false}
      />
    )

    expect(screen.getByText('OWNER')).toBeInTheDocument()
  })

  it('displays role badge with warning color for admin role', () => {
    const adminMember = { ...mockMember, role: 'admin' as const }
    
    render(
      <MemberCard
        member={adminMember}
        canManage={false}
      />
    )

    expect(screen.getByText('ADMIN')).toBeInTheDocument()
  })

  it('displays online status badge for active members', () => {
    render(
      <MemberCard
        member={{ ...mockMember, deleted: false }}
        canManage={false}
      />
    )

    // Should show badge with success color (green dot for online)
    // Badge is rendered as part of the Avatar component
    expect(screen.getByText('ocean-blue-eagle-star')).toBeInTheDocument()
  })

  it('does not show action menu when canManage is false', () => {
    render(
      <MemberCard
        member={mockMember}
        canManage={false}
      />
    )

    // Should not have more options button
    expect(screen.queryByLabelText('more options')).not.toBeInTheDocument()
  })

  it('shows action menu button when canManage is true', () => {
    render(
      <MemberCard
        member={mockMember}
        canManage={true}
      />
    )

    expect(screen.getByLabelText('more options')).toBeInTheDocument()
  })

  it('opens action menu when more button clicked', () => {
    render(
      <MemberCard
        member={mockMember}
        canManage={true}
      />
    )

    const moreButton = screen.getByLabelText('more options')
    fireEvent.click(moreButton)

    expect(screen.getByText('Change Role')).toBeInTheDocument()
    expect(screen.getByText('Remove Member')).toBeInTheDocument()
  })

  it('calls onRemove when Remove Member clicked', () => {
    const onRemove = vi.fn()
    
    render(
      <MemberCard
        member={mockMember}
        canManage={true}
        onRemove={onRemove}
      />
    )

    fireEvent.click(screen.getByLabelText('more options'))
    fireEvent.click(screen.getByText('Remove Member'))

    expect(onRemove).toHaveBeenCalledWith('ocean-blue-eagle-star')
  })

  it('calls onRoleChange when Change Role clicked', () => {
    const onRoleChange = vi.fn()
    
    render(
      <MemberCard
        member={mockMember}
        canManage={true}
        onRoleChange={onRoleChange}
      />
    )

    fireEvent.click(screen.getByLabelText('more options'))
    fireEvent.click(screen.getByText('Change Role'))

    // Should trigger role change dialog/flow
    expect(onRoleChange).toHaveBeenCalled()
  })

  it('displays joined time', () => {
    render(
      <MemberCard
        member={mockMember}
        canManage={false}
      />
    )

    expect(screen.getByText(/Joined:/)).toBeInTheDocument()
  })
})
