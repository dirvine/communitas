import React, { useState } from 'react'
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
  Box,
  Typography,
  Alert
} from '@mui/material'
import { invoke } from '@tauri-apps/api/core'

interface InviteMemberDialogProps {
  open: boolean
  onClose: () => void
  onSubmit: (data: {
    address: string
    role: 'Admin' | 'Member' | 'Viewer' | 'Guest'
    message?: string
  }) => void
  entityType?: 'organization' | 'group' | 'project'
}

const InviteMemberDialog: React.FC<InviteMemberDialogProps> = ({
  open,
  onClose,
  onSubmit,
  entityType
}) => {
  const [formData, setFormData] = useState({
    address: '',
    role: 'Member' as 'Admin' | 'Member' | 'Viewer' | 'Guest',
    message: ''
  })

  const [errors, setErrors] = useState<Record<string, string>>({})

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    const newErrors: Record<string, string> = {}
    const address = formData.address.trim().toLowerCase()

    if (!address) {
      newErrors.address = 'Member address is required'
    } else {
      try {
        await invoke<string>('four_word_decode_address', { words: address })
      } catch (err) {
        newErrors.address = 'Invalid four-word address'
      }
    }

    setErrors(newErrors)
    if (Object.keys(newErrors).length === 0) {
      onSubmit({
        address,
        role: formData.role,
        message: formData.message.trim() || undefined
      })
      handleClose()
    }
  }

  const handleClose = () => {
    setFormData({
      address: '',
      role: 'Member',
      message: ''
    })
    setErrors({})
    onClose()
  }

  const handleInputChange = (field: string) => (
    event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement> | any
  ) => {
    const value = event.target.value
    setFormData(prev => ({ ...prev, [field]: value }))
    
    // Clear error when user starts typing
    if (errors[field]) {
      setErrors(prev => ({ ...prev, [field]: '' }))
    }
  }

  const getRoleDescription = (role: string) => {
    switch (role) {
      case 'Admin':
        return 'Can manage settings, members, and all content'
      case 'Member':
        return 'Can create content and invite other members'
      case 'Viewer':
        return 'Can view content but not make changes'
      case 'Guest':
        return 'Limited read-only access'
      default:
        return ''
    }
  }

  const getEntityTypeText = () => {
    switch (entityType) {
      case 'organization':
        return 'organization'
      case 'group':
        return 'group'
      case 'project':
        return 'project'
      default:
        return 'entity'
    }
  }

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
      <form onSubmit={handleSubmit}>
        <DialogTitle>
          Invite Member to {getEntityTypeText().charAt(0).toUpperCase() + getEntityTypeText().slice(1)}
        </DialogTitle>
        
        <DialogContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3, pt: 1 }}>
            <Alert severity="info" sx={{ mb: 1 }}>
              Members will receive an invitation to join this {getEntityTypeText()}. They can accept or decline the invitation.
            </Alert>

            <TextField
              label="Four-Word Address"
              value={formData.address}
              onChange={handleInputChange('address')}
              error={!!errors.address}
              helperText={errors.address || 'Enter the four-word address of the person you want to invite'}
              required
              fullWidth
              placeholder="user.name.test.here"
            />

            <FormControl fullWidth>
              <InputLabel>Role</InputLabel>
              <Select
                value={formData.role}
                label="Role"
                onChange={handleInputChange('role')}
              >
                <MenuItem value="Admin">
                  <Box>
                    <Typography variant="body1">Admin</Typography>
                    <Typography variant="caption" color="text.secondary">
                      {getRoleDescription('Admin')}
                    </Typography>
                  </Box>
                </MenuItem>
                <MenuItem value="Member">
                  <Box>
                    <Typography variant="body1">Member</Typography>
                    <Typography variant="caption" color="text.secondary">
                      {getRoleDescription('Member')}
                    </Typography>
                  </Box>
                </MenuItem>
                <MenuItem value="Viewer">
                  <Box>
                    <Typography variant="body1">Viewer</Typography>
                    <Typography variant="caption" color="text.secondary">
                      {getRoleDescription('Viewer')}
                    </Typography>
                  </Box>
                </MenuItem>
                <MenuItem value="Guest">
                  <Box>
                    <Typography variant="body1">Guest</Typography>
                    <Typography variant="caption" color="text.secondary">
                      {getRoleDescription('Guest')}
                    </Typography>
                  </Box>
                </MenuItem>
              </Select>
            </FormControl>

            <TextField
              label="Invitation Message (Optional)"
              value={formData.message}
              onChange={handleInputChange('message')}
              multiline
              rows={3}
              fullWidth
              placeholder="Add a personal message to your invitation..."
            />

            {entityType === 'group' && (
              <Alert severity="info" variant="outlined">
                <Typography variant="body2">
                  <strong>Group Invitation:</strong> This member will be able to participate in group discussions. 
                  File sharing uses the parent organization's storage.
                </Typography>
              </Alert>
            )}

            {entityType === 'project' && (
              <Alert severity="success" variant="outlined">
                <Typography variant="body2">
                  <strong>Project Invitation:</strong> This member will get access to the project's 
                  dedicated file system and collaboration tools.
                </Typography>
              </Alert>
            )}

            {entityType === 'organization' && (
              <Alert severity="warning" variant="outlined">
                <Typography variant="body2">
                  <strong>Organization Invitation:</strong> This member will be able to access all 
                  public groups and can be invited to projects within the organization.
                </Typography>
              </Alert>
            )}
          </Box>
        </DialogContent>

        <DialogActions>
          <Button onClick={handleClose}>
            Cancel
          </Button>
          <Button type="submit" variant="contained">
            Send Invitation
          </Button>
        </DialogActions>
      </form>
    </Dialog>
  )
}

export default InviteMemberDialog
