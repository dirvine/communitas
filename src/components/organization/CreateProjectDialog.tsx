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
  Chip,
  Grid
} from '@mui/material'
import { invoke } from '@tauri-apps/api/core'

interface CreateProjectDialogProps {
  open: boolean
  onClose: () => void
  onSubmit: (data: {
    name: string
    description?: string
    deadline?: Date
    priority: 'low' | 'medium' | 'high' | 'critical'
    initial_storage_gb: number
    initial_members?: string[]
  }) => void
}

const CreateProjectDialog: React.FC<CreateProjectDialogProps> = ({
  open,
  onClose,
  onSubmit
}) => {
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    deadline: '',
    priority: 'medium' as 'low' | 'medium' | 'high' | 'critical',
    initial_storage_gb: 5,
    member_addresses: ''
  })

  const [errors, setErrors] = useState<Record<string, string>>({})

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    // Validate form
    const newErrors: Record<string, string> = {}
    
    if (!formData.name.trim()) {
      newErrors.name = 'Project name is required'
    } else if (formData.name.trim().length < 2) {
      newErrors.name = 'Project name must be at least 2 characters'
    }
    
    if (formData.initial_storage_gb < 1) {
      newErrors.initial_storage_gb = 'Storage must be at least 1 GB'
    } else if (formData.initial_storage_gb > 100) {
      newErrors.initial_storage_gb = 'Storage cannot exceed 100 GB per project'
    }

    // Validate deadline if provided
    if (formData.deadline) {
      const deadlineDate = new Date(formData.deadline)
      const now = new Date()
      if (deadlineDate <= now) {
        newErrors.deadline = 'Deadline must be in the future'
      }
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
        deadline: formData.deadline ? new Date(formData.deadline) : undefined,
        priority: formData.priority,
        initial_storage_gb: formData.initial_storage_gb,
        initial_members: valid.length > 0 ? valid : undefined
      })
      handleClose()
    }
  }

  const handleClose = () => {
    setFormData({
      name: '',
      description: '',
      deadline: '',
      priority: 'medium',
      initial_storage_gb: 5,
      member_addresses: ''
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

  const getMemberAddresses = () => {
    return formData.member_addresses
      .split(',')
      .map(addr => addr.trim())
      .filter(addr => addr.length > 0)
  }

  const validAddresses: string[] = []
  const invalidAddresses: string[] = []


  return (
    <Dialog open={open} onClose={handleClose} maxWidth="md" fullWidth>
      <form onSubmit={handleSubmit}>
        <DialogTitle>Create New Project</DialogTitle>
        
        <DialogContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3, pt: 1 }}>
            <Grid container spacing={2}>
              <Grid item xs={12} sm={8}>
                <TextField
                  label="Project Name"
                  value={formData.name}
                  onChange={handleInputChange('name')}
                  error={!!errors.name}
                  helperText={errors.name}
                  required
                  fullWidth
                  placeholder="e.g., Communitas v2 Development"
                />
              </Grid>
              <Grid item xs={12} sm={4}>
                <FormControl fullWidth>
                  <InputLabel>Priority</InputLabel>
                  <Select
                    value={formData.priority}
                    label="Priority"
                    onChange={handleInputChange('priority')}
                  >
                    <MenuItem value="low">
                      <Chip label="Low" size="small" color="default" />
                    </MenuItem>
                    <MenuItem value="medium">
                      <Chip label="Medium" size="small" color="info" />
                    </MenuItem>
                    <MenuItem value="high">
                      <Chip label="High" size="small" color="warning" />
                    </MenuItem>
                    <MenuItem value="critical">
                      <Chip label="Critical" size="small" color="error" />
                    </MenuItem>
                  </Select>
                </FormControl>
              </Grid>
            </Grid>

            <TextField
              label="Description"
              value={formData.description}
              onChange={handleInputChange('description')}
              multiline
              rows={3}
              fullWidth
              placeholder="Describe the project goals and scope..."
            />

            <Grid container spacing={2}>
              <Grid item xs={12} sm={6}>
                <TextField
                  label="Deadline (Optional)"
                  type="date"
                  value={formData.deadline}
                  onChange={handleInputChange('deadline')}
                  error={!!errors.deadline}
                  helperText={errors.deadline}
                  InputLabelProps={{
                    shrink: true,
                  }}
                  fullWidth
                />
              </Grid>
              <Grid item xs={12} sm={6}>
                <TextField
                  label="Storage Allocation (GB)"
                  type="number"
                  value={formData.initial_storage_gb}
                  onChange={handleInputChange('initial_storage_gb')}
                  error={!!errors.initial_storage_gb}
                  helperText={errors.initial_storage_gb || 'Dedicated storage for project files'}
                  inputProps={{ min: 1, max: 100, step: 1 }}
                  fullWidth
                />
              </Grid>
            </Grid>

            <Box>
              <TextField
                label="Initial Team Members (Optional)"
                value={formData.member_addresses}
                onChange={handleInputChange('member_addresses')}
                fullWidth
                placeholder="developer.one.team.here, designer.two.team.here"
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

            <Box sx={{ bgcolor: 'success.main', color: 'success.contrastText', p: 2, borderRadius: 1 }}>
              <Typography variant="subtitle2" gutterBottom>
                Project Features:
              </Typography>
              <Typography variant="body2">
                • Dedicated file system with {formData.initial_storage_gb} GB storage
              </Typography>
              <Typography variant="body2">
                • Version control and backup capabilities
              </Typography>
              <Typography variant="body2">
                • Task management and collaboration tools
              </Typography>
              <Typography variant="body2">
                • Team chat and file sharing
              </Typography>
              <Typography variant="body2">
                • Progress tracking and reporting
              </Typography>
            </Box>
          </Box>
        </DialogContent>

        <DialogActions>
          <Button onClick={handleClose}>
            Cancel
          </Button>
          <Button type="submit" variant="contained">
            Create Project
          </Button>
        </DialogActions>
      </form>
    </Dialog>
  )
}

export default CreateProjectDialog
