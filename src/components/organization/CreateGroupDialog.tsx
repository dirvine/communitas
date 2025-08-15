import React, { useState } from 'react'
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Button,
  Box,
  Typography,
  FormControlLabel,
  Switch,
  Chip
} from '@mui/material'
import { invoke } from '@tauri-apps/api/core'

interface CreateGroupDialogProps {
  open: boolean
  onClose: () => void
  onSubmit: (data: {
    name: string
    description?: string
    initial_members?: string[]
  }) => void
}

const CreateGroupDialog: React.FC<CreateGroupDialogProps> = ({
  open,
  onClose,
  onSubmit
}) => {
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    member_addresses: '',
    inherit_org_permissions: true
  })

  const [errors, setErrors] = useState<Record<string, string>>({})

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    // Validate form
    const newErrors: Record<string, string> = {}
    
    if (!formData.name.trim()) {
      newErrors.name = 'Group name is required'
    } else if (formData.name.trim().length < 2) {
      newErrors.name = 'Group name must be at least 2 characters'
    }

    // Validate member addresses via backend decode
    const addresses = formData.member_addresses
      .split(',')
      .map(a => a.trim().toLowerCase())
      .filter(a => a.length > 0)

    const valid: string[] = []
    const invalid: string[] = []
    for (const addr of addresses) {
      try {
        await invoke<string>('four_word_decode_address', { words: addr })
        valid.push(addr)
      } catch {
        invalid.push(addr)
      }
    }

    if (invalid.length > 0) {
      newErrors.member_addresses = `Invalid addresses: ${invalid.join(', ')}`
    }

    setErrors(newErrors)
    
    if (Object.keys(newErrors).length === 0) {
      onSubmit({
        name: formData.name.trim(),
        description: formData.description.trim() || undefined,
        initial_members: valid.length > 0 ? valid : undefined
      })
      handleClose()
    }
  }

  const handleClose = () => {
    setFormData({
      name: '',
      description: '',
      member_addresses: '',
      inherit_org_permissions: true
    })
    setErrors({})
    onClose()
  }

  const handleInputChange = (field: string) => (
    event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>
  ) => {
    const value = event.target.value
    setFormData(prev => ({ ...prev, [field]: value }))
    
    // Clear error when user starts typing
    if (errors[field]) {
      setErrors(prev => ({ ...prev, [field]: '' }))
    }
  }

  const getMemberAddresses = () => {
    return formData.member_addresses
      .split(',')
      .map(addr => addr.trim())
      .filter(addr => addr.length > 0)
  }

  const validAddresses: string[] = [] // Shown only after submit; live preview removed to avoid misleading regex
  const invalidAddresses: string[] = []

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
      <form onSubmit={handleSubmit}>
        <DialogTitle>Create New Group</DialogTitle>
        
        <DialogContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3, pt: 1 }}>
            <TextField
              label="Group Name"
              value={formData.name}
              onChange={handleInputChange('name')}
              error={!!errors.name}
              helperText={errors.name}
              required
              fullWidth
              placeholder="e.g., Development Team"
            />

            <TextField
              label="Description"
              value={formData.description}
              onChange={handleInputChange('description')}
              multiline
              rows={3}
              fullWidth
              placeholder="What is this group for?"
            />

            <Box>
              <TextField
                label="Initial Members (Optional)"
                value={formData.member_addresses}
                onChange={handleInputChange('member_addresses')}
                fullWidth
                placeholder="user.one.test.here, user.two.test.here"
                helperText="Enter four-word addresses separated by commas"
              />
              
              {/* Show parsed addresses */}
              {validAddresses.length > 0 && (
                <Box sx={{ mt: 1, display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
                  {validAddresses.map((addr, index) => (
                    <Chip 
                      key={index} 
                      label={addr} 
                      size="small" 
                      color="primary"
                      variant="outlined"
                    />
                  ))}
                </Box>
              )}
              
              {/* Show invalid addresses */}
              {invalidAddresses.length > 0 && (
                <Box sx={{ mt: 1, display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
                  {invalidAddresses.map((addr, index) => (
                    <Chip 
                      key={index} 
                      label={`${addr} (invalid format)`}
                      size="small" 
                      color="error"
                      variant="outlined"
                    />
                  ))}
                </Box>
              )}
            </Box>

            <FormControlLabel
              control={
                <Switch
                  checked={formData.inherit_org_permissions}
                  onChange={(e) => setFormData(prev => ({ 
                    ...prev, 
                    inherit_org_permissions: e.target.checked 
                  }))}
                />
              }
              label={
                <Box>
                  <Typography variant="body2">
                    Inherit Organization Permissions
                  </Typography>
                  <Typography variant="caption" color="text.secondary">
                    Organization members can access this group automatically
                  </Typography>
                </Box>
              }
            />

            <Box sx={{ bgcolor: 'info.main', color: 'info.contrastText', p: 2, borderRadius: 1 }}>
              <Typography variant="subtitle2" gutterBottom>
                Group Features:
              </Typography>
              <Typography variant="body2">
                • Chat-only functionality (no dedicated file system)
              </Typography>
              <Typography variant="body2">
                • Can share files using organization storage
              </Typography>
              <Typography variant="body2">
                • Voice and video calling support
              </Typography>
              <Typography variant="body2">
                • Message history and search
              </Typography>
            </Box>
          </Box>
        </DialogContent>

        <DialogActions>
          <Button onClick={handleClose}>
            Cancel
          </Button>
          <Button type="submit" variant="contained">
            Create Group
          </Button>
        </DialogActions>
      </form>
    </Dialog>
  )
}

export default CreateGroupDialog
