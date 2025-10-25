import { useState, useEffect } from 'react'
import {
  Box,
  Button,
  CircularProgress,
  List,
  Typography
} from '@mui/material'
import { PersonAdd } from '@mui/icons-material'
import { MemberCard } from './MemberCard'
import { AddMemberDialog } from './AddMemberDialog'
import type { MemberInfo, MemberEntityType, MemberRole } from '@/types/memberManagement'
import { memberManagementService } from '@/services/MemberManagementService'

interface MemberListPanelProps {
  entityType: MemberEntityType
  entityId: string
  currentUserId: string
  currentUserRole: MemberRole
}

export function MemberListPanel({
  entityType,
  entityId,
  currentUserId,
  currentUserRole
}: MemberListPanelProps) {
  const [members, setMembers] = useState<MemberInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [addDialogOpen, setAddDialogOpen] = useState(false)

  useEffect(() => {
    loadMembers()
  }, [entityId, entityType])

  const loadMembers = async () => {
    setLoading(true)
    try {
      const result = await memberManagementService.listMembers(entityId, entityType)
      if (result.success && result.data) {
        setMembers(result.data)
      } else {
        console.error('Failed to load members:', result.error)
        setMembers([])
      }
    } catch (err) {
      console.error('Error loading members:', err)
      setMembers([])
    } finally {
      setLoading(false)
    }
  }

  const handleRemove = async (memberId: string) => {
    try {
      const result = await memberManagementService.removeMember({
        entity_type: entityType,
        entity_id: entityId,
        member_id: memberId,
        deleted_by: currentUserId
      })

      if (result.success) {
        // Reload member list
        loadMembers()
      } else {
        console.error('Failed to remove member:', result.error)
      }
    } catch (err) {
      console.error('Error removing member:', err)
    }
  }

  const handleRoleChange = async (memberId: string, newRole: MemberRole) => {
    try {
      const result = await memberManagementService.updateRole({
        entity_type: entityType,
        entity_id: entityId,
        member_id: memberId,
        new_role: newRole,
        updated_by: currentUserId
      })

      if (result.success) {
        // Reload member list
        loadMembers()
      } else {
        console.error('Failed to update role:', result.error)
      }
    } catch (err) {
      console.error('Error updating role:', err)
    }
  }

  const canManageMembers = currentUserRole === 'owner' || currentUserRole === 'admin'

  return (
    <Box>
      <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
        <Typography variant="h6">
          Members ({members.length})
        </Typography>
        {canManageMembers && (
          <Button
            startIcon={<PersonAdd />}
            onClick={() => setAddDialogOpen(true)}
            variant="outlined"
            size="small"
          >
            Add Member
          </Button>
        )}
      </Box>

      {loading ? (
        <Box display="flex" justifyContent="center" p={4}>
          <CircularProgress />
        </Box>
      ) : members.length === 0 ? (
        <Box textAlign="center" p={4}>
          <Typography variant="body2" color="textSecondary">
            No members yet
          </Typography>
          {canManageMembers && (
            <Typography variant="caption" color="textSecondary" display="block" mt={1}>
              Click "Add Member" to invite people
            </Typography>
          )}
        </Box>
      ) : (
        <List>
          {members.map((member) => (
            <MemberCard
              key={member.member_id}
              member={member}
              canManage={canManageMembers && member.member_id !== currentUserId}
              onRemove={() => handleRemove(member.member_id)}
              onRoleChange={(newRole) => handleRoleChange(member.member_id, newRole)}
            />
          ))}
        </List>
      )}

      <AddMemberDialog
        open={addDialogOpen}
        onClose={() => setAddDialogOpen(false)}
        entityType={entityType}
        entityId={entityId}
        onMemberAdded={loadMembers}
      />
    </Box>
  )
}
