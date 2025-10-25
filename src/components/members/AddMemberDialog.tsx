import { useState, useEffect } from 'react'
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Button,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  CircularProgress,
  Alert
} from '@mui/material'
import type { MemberEntityType, MemberRole } from '@/types/memberManagement'
import { memberManagementService } from '@/services/MemberManagementService'

interface AddMemberDialogProps {
  open: boolean
  onClose: () => void
  entityType: MemberEntityType
  entityId: string
  onMemberAdded: () => void
}

export function AddMemberDialog({
  open,
  onClose,
  entityType,
  entityId,
  onMemberAdded
}: AddMemberDialogProps) {
  const [fourWords, setFourWords] = useState('')
  const [role, setRole] = useState<MemberRole>('member')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Reset form when dialog opens/closes
  useEffect(() => {
    if (!open) {
      setFourWords('')
      setRole('member')
      setError(null)
    }
  }, [open])

  const validateFourWords = (input: string): boolean => {
    // Four-word format: word-word-word-word
    const pattern = /^[a-z]+-[a-z]+-[a-z]+-[a-z]+$/
    return pattern.test(input)
  }

  const handleAdd = async () => {
    setLoading(true)
    setError(null)

    try {
      // Validate four-word format
      if (!validateFourWords(fourWords)) {
        setError('Invalid four-word address format. Expected: word-word-word-word')
        setLoading(false)
        return
      }

      // Get current user ID (in production, get from auth context)
      const currentUserId = 'current-user' // TODO: Get from auth context

      // Call backend
      const result = await memberManagementService.addMember({
        entity_type: entityType,
        entity_id: entityId,
        member_id: fourWords,
        role,
        added_by: currentUserId
      })

      if (result.success) {
        onMemberAdded()
        onClose()
      } else {
        setError(result.error?.message || 'Failed to add member')
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An error occurred')
    } finally {
      setLoading(false)
    }
  }

  return (
    <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
      <DialogTitle>Add Member</DialogTitle>
      <DialogContent>
        {error && (
          <Alert severity="error" sx={{ mb: 2 }}>
            {error}
          </Alert>
        )}

        <TextField
          label="Four-Word Address"
          placeholder="ocean-blue-eagle-star"
          value={fourWords}
          onChange={(e) => setFourWords(e.target.value.toLowerCase())}
          fullWidth
          margin="normal"
          autoFocus
          helperText="Enter the four-word address of the member to add"
        />

        <FormControl fullWidth margin="normal">
          <InputLabel>Role</InputLabel>
          <Select
            value={role}
            label="Role"
            onChange={(e) => setRole(e.target.value as MemberRole)}
          >
            <MenuItem value="guest">Guest - View only access</MenuItem>
            <MenuItem value="member">Member - Standard access</MenuItem>
            <MenuItem value="admin">Admin - Can manage members</MenuItem>
          </Select>
        </FormControl>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose} disabled={loading}>
          Cancel
        </Button>
        <Button
          onClick={handleAdd}
          disabled={loading || !fourWords}
          variant="contained"
        >
          {loading ? <CircularProgress size={20} /> : 'Add'}
        </Button>
      </DialogActions>
    </Dialog>
  )
}
