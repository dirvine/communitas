import React, { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
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
  Typography
} from '@mui/material'

interface CreateOrganizationDialogProps {
  open: boolean
  onClose: () => void
  onSubmit: (data: {
    name: string
    description?: string
    visibility: 'public' | 'private' | 'invite_only'
    initial_storage_gb: number
  }) => void
}

const CreateOrganizationDialog: React.FC<CreateOrganizationDialogProps> = ({
  open,
  onClose,
  onSubmit
}) => {
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    visibility: 'private' as 'public' | 'private' | 'invite_only',
    initial_storage_gb: 10
  })

  const [errors, setErrors] = useState<Record<string, string>>({})

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    // Validate form
    const newErrors: Record<string, string> = {}
    
    if (!formData.name.trim()) {
      newErrors.name = 'Organization name is required'
    } else if (formData.name.trim().length < 3) {
      newErrors.name = 'Organization name must be at least 3 characters'
    }
    
    if (formData.initial_storage_gb < 1) {
      newErrors.initial_storage_gb = 'Storage must be at least 1 GB'
    } else if (formData.initial_storage_gb > 1000) {
      newErrors.initial_storage_gb = 'Storage cannot exceed 1000 GB'
    }

    setErrors(newErrors)
    
    if (Object.keys(newErrors).length === 0) {
      try {
        await invoke('create_organization', { name: formData.name.trim() })
      } catch (_) {}
      onSubmit({
        name: formData.name.trim(),
        description: formData.description.trim() || undefined,
        visibility: formData.visibility,
        initial_storage_gb: formData.initial_storage_gb
      })
      handleClose()
    }
  }

  const handleClose = () => {
    setFormData({
      name: '',
      description: '',
      visibility: 'private',
      initial_storage_gb: 10
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

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
      <form onSubmit={handleSubmit}>
        <DialogTitle>Create New Organization</DialogTitle>
        
        <DialogContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3, pt: 1 }}>
            <TextField
              label="Organization Name"
              value={formData.name}
              onChange={handleInputChange('name')}
              error={!!errors.name}
              helperText={errors.name}
              required
              fullWidth
              placeholder="e.g., MaidSafe Foundation"
            />

            <TextField
              label="Description"
              value={formData.description}
              onChange={handleInputChange('description')}
              multiline
              rows={3}
              fullWidth
              placeholder="Brief description of your organization..."
            />

            <FormControl fullWidth>
              <InputLabel>Visibility</InputLabel>
              <Select
                value={formData.visibility}
                label="Visibility"
                onChange={handleInputChange('visibility')}
              >
                <MenuItem value="public">
                  <Box>
                    <Typography variant="body1">Public</Typography>
                    <Typography variant="caption" color="text.secondary">
                      Anyone can discover and request to join
                    </Typography>
                  </Box>
                </MenuItem>
                <MenuItem value="private">
                  <Box>
                    <Typography variant="body1">Private</Typography>
                    <Typography variant="caption" color="text.secondary">
                      Only visible to members, invite-only
                    </Typography>
                  </Box>
                </MenuItem>
                <MenuItem value="invite_only">
                  <Box>
                    <Typography variant="body1">Invite Only</Typography>
                    <Typography variant="caption" color="text.secondary">
                      Members can only be added by invitation
                    </Typography>
                  </Box>
                </MenuItem>
              </Select>
            </FormControl>

            <TextField
              label="Initial Storage (GB)"
              type="number"
              value={formData.initial_storage_gb}
              onChange={handleInputChange('initial_storage_gb')}
              error={!!errors.initial_storage_gb}
              helperText={errors.initial_storage_gb || 'Storage allocation for organization and projects'}
              inputProps={{ min: 1, max: 1000, step: 1 }}
              fullWidth
            />

            <Box sx={{ bgcolor: 'background.paper', p: 2, borderRadius: 1, border: 1, borderColor: 'divider' }}>
              <Typography variant="subtitle2" gutterBottom>
                What you'll get:
              </Typography>
              <Typography variant="body2" color="text.secondary">
                • Organization-wide file system with {formData.initial_storage_gb} GB storage
              </Typography>
              <Typography variant="body2" color="text.secondary">
                • Unlimited groups for team discussions
              </Typography>
              <Typography variant="body2" color="text.secondary">
                • Projects with dedicated file systems
              </Typography>
              <Typography variant="body2" color="text.secondary">
                • Role-based permissions and member management
              </Typography>
            </Box>
          </Box>
        </DialogContent>

        <DialogActions>
          <Button onClick={handleClose}>
            Cancel
          </Button>
          <Button type="submit" variant="contained">
            Create Organization
          </Button>
        </DialogActions>
      </form>
    </Dialog>
  )
}

export default CreateOrganizationDialog
