import { Close as CloseIcon, PersonAdd, PersonAddAlt, PersonRemove } from '@mui/icons-material'
import {
    Alert, Box, Button, Chip, Dialog, DialogActions, DialogContent, DialogTitle, Divider, FormControl, IconButton, InputLabel, List,
    ListItem, ListItemSecondaryAction, ListItemText, MenuItem, Select, Tab, Tabs, TextField, Typography
} from '@mui/material'
import React, { useState } from 'react'

export interface Contact {
  id: string
  name: string
  fourWords: string
  snippet: string
  time: string
  online?: boolean
  starred?: boolean
  lastMessageTime?: number // Unix timestamp for MRU sorting
}

interface AddContactDialogProps {
  open: boolean
  onClose: () => void
  onSave: (contact: Omit<Contact, 'id' | 'lastMessageTime'>) => void
  onGenerateIdentity?: () => Promise<string> // Callback to generate new four-word identity
}

export const AddContactDialog: React.FC<AddContactDialogProps> = ({ open, onClose, onSave, onGenerateIdentity }) => {
  const [mode, setMode] = useState<'existing' | 'new'>('existing')
  const [name, setName] = useState('')
  const [fourWords, setFourWords] = useState('')
  const [error, setError] = useState('')
  const [generatingIdentity, setGeneratingIdentity] = useState(false)

  // Normalize four words - accept spaces or dashes
  const normalizeFourWords = (input: string): string => {
    return input.trim().toLowerCase().replace(/\s+/g, '-')
  }

  // Validate against actual four-word dictionary format
  const validateFourWords = async (input: string): Promise<boolean> => {
    const normalized = normalizeFourWords(input)
    // Accept spaces or dashes between words
    const pattern = /^[a-z]+[\s-][a-z]+[\s-][a-z]+[\s-][a-z]+$/

    if (!pattern.test(input.trim().toLowerCase())) {
      return false
    }

    // Check if each word is valid using Tauri backend
    try {
      if (typeof window !== 'undefined' && '_TAURI_' in window) {
        const { invoke } = await import('@tauri-apps/api/core')
        const isValid = await invoke<boolean>('validate_four_words', { fourWords: normalized })
        return isValid
      }
      // In browser mode, just check format
      return true
    } catch (error) {
      console.error('Validation error:', error)
      return false
    }
  }

  const handleGenerateIdentity = async () => {
    if (!onGenerateIdentity) return

    setGeneratingIdentity(true)
    setError('')
    try {
      const newIdentity = await onGenerateIdentity()
      setFourWords(newIdentity)
    } catch (err) {
      setError(`Failed to generate identity: ${err}`)
    } finally {
      setGeneratingIdentity(false)
    }
  }

  const handleSave = async () => {
    if (!name.trim()) {
      setError('Name is required')
      return
    }

    if (mode === 'existing') {
      if (!fourWords.trim()) {
        setError('Four-word address is required')
        return
      }

      const isValid = await validateFourWords(fourWords)
      if (!isValid) {
        setError('Invalid four-word address. Each word must be from the dictionary.')
        return
      }
    } else {
      // New contact mode - must have generated identity
      if (!fourWords.trim()) {
        setError('Please generate a four-word identity first')
        return
      }
    }

    onSave({
      name: name.trim(),
      fourWords: normalizeFourWords(fourWords),
      snippet: 'No messages yet',
      time: 'Never',
      online: false,
      starred: false,
    })

    // Reset form
    setName('')
    setFourWords('')
    setError('')
    setMode('existing')
    onClose()
  }

  const handleClose = () => {
    setName('')
    setFourWords('')
    setError('')
    setMode('existing')
    onClose()
  }

  const handleModeChange = (_: React.SyntheticEvent, newMode: 'existing' | 'new') => {
    setMode(newMode)
    setFourWords('') // Clear four-words when switching modes
    setError('')
  }

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
      <DialogTitle>
        <Box display="flex" alignItems="center" justifyContent="space-between">
          <Typography variant="h6">Add Contact</Typography>
          <IconButton onClick={handleClose} size="small">
            <CloseIcon />
          </IconButton>
        </Box>
        <Tabs value={mode} onChange={handleModeChange} sx={{ mt: 1 }}>
          <Tab
            icon={<PersonAdd />}
            iconPosition="start"
            label="Add Existing"
            value="existing"
          />
          <Tab
            icon={<PersonAddAlt />}
            iconPosition="start"
            label="Create New"
            value="new"
          />
        </Tabs>
      </DialogTitle>
      <DialogContent>
        {error && (
          <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError('')}>
            {error}
          </Alert>
        )}

        {mode === 'existing' ? (
          <>
            <Alert severity="info" sx={{ mb: 2 }}>
              Add someone you already know by entering their four-word network identity
            </Alert>
            <TextField
              autoFocus
              margin="dense"
              label="Display Name"
              type="text"
              fullWidth
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="John Doe"
              sx={{ mb: 2 }}
            />
            <TextField
              margin="dense"
              label="Four-Word Address"
              type="text"
              fullWidth
              value={fourWords}
              onChange={(e) => setFourWords(e.target.value)}
              placeholder="ocean forest moon star"
              helperText="Enter their existing four-word network identity (spaces or dashes)"
            />
          </>
        ) : (
          <>
            <Alert severity="info" sx={{ mb: 2 }}>
              Create a new contact with a generated four-word identity
            </Alert>
            <TextField
              autoFocus
              margin="dense"
              label="Display Name"
              type="text"
              fullWidth
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="John Doe"
              sx={{ mb: 2 }}
            />
            <Box sx={{ mb: 2 }}>
              <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
                Four-Word Identity:
              </Typography>
              {fourWords ? (
                <Chip
                  label={fourWords}
                  color="primary"
                  onDelete={() => setFourWords('')}
                  sx={{ fontFamily: 'monospace' }}
                />
              ) : (
                <Button
                  variant="outlined"
                  onClick={handleGenerateIdentity}
                  disabled={generatingIdentity}
                  fullWidth
                >
                  {generatingIdentity ? 'Generating...' : 'Generate Identity'}
                </Button>
              )}
            </Box>
            <Alert severity="warning" sx={{ mt: 2 }}>
              <Typography variant="caption">
                <strong>Note:</strong> The generated identity will be created when you save.
                Make sure to share it with the contact so they can accept your invitation.
              </Typography>
            </Alert>
          </>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClose}>Cancel</Button>
        <Button onClick={handleSave} variant="contained" disabled={generatingIdentity}>
          {mode === 'existing' ? 'Add Contact' : 'Create Contact'}
        </Button>
      </DialogActions>
    </Dialog>
  )
}

interface EditContactDialogProps {
  open: boolean
  contact: Contact | null
  onClose: () => void
  onSave: (id: string, updates: Partial<Contact>) => void
}

export const EditContactDialog: React.FC<EditContactDialogProps> = ({ open, contact, onClose, onSave }) => {
  const [name, setName] = useState(contact?.name ?? '')
  const [fourWords, setFourWords] = useState(contact?.fourWords ?? '')
  const [error, setError] = useState('')

  React.useEffect(() => {
    if (contact) {
      setName(contact.name)
      setFourWords(contact.fourWords)
    }
  }, [contact])

  const validateFourWords = (input: string): boolean => {
    const pattern = /^[a-z]+-[a-z]+-[a-z]+-[a-z]+$/
    return pattern.test(input.trim().toLowerCase())
  }

  const handleSave = () => {
    if (!contact) return

    if (!name.trim()) {
      setError('Name is required')
      return
    }
    if (!validateFourWords(fourWords)) {
      setError('Invalid four-word address format')
      return
    }

    onSave(contact.id, {
      name: name.trim(),
      fourWords: fourWords.trim().toLowerCase(),
    })

    setError('')
    onClose()
  }

  const handleClose = () => {
    setError('')
    onClose()
  }

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
      <DialogTitle>
        <Box display="flex" alignItems="center" justifyContent="space-between">
          <Typography variant="h6">Edit Contact</Typography>
          <IconButton onClick={handleClose} size="small">
            <CloseIcon />
          </IconButton>
        </Box>
      </DialogTitle>
      <DialogContent>
        {error && (
          <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError('')}>
            {error}
          </Alert>
        )}
        <TextField
          autoFocus
          margin="dense"
          label="Name"
          type="text"
          fullWidth
          value={name}
          onChange={(e) => setName(e.target.value)}
          sx={{ mb: 2 }}
        />
        <TextField
          margin="dense"
          label="Four-Word Address"
          type="text"
          fullWidth
          value={fourWords}
          onChange={(e) => setFourWords(e.target.value)}
          helperText="The contact's four-word network identity"
        />
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClose}>Cancel</Button>
        <Button onClick={handleSave} variant="contained">
          Save Changes
        </Button>
      </DialogActions>
    </Dialog>
  )
}

interface DeleteContactDialogProps {
  open: boolean
  contact: Contact | null
  onClose: () => void
  onConfirm: (id: string) => void
}

export const DeleteContactDialog: React.FC<DeleteContactDialogProps> = ({ open, contact, onClose, onConfirm }) => {
  const handleConfirm = () => {
    if (contact) {
      onConfirm(contact.id)
      onClose()
    }
  }

  return (
    <Dialog open={open} onClose={onClose} maxWidth="xs" fullWidth>
      <DialogTitle>Delete Contact</DialogTitle>
      <DialogContent>
        <Typography>
          Are you sure you want to delete <strong>{contact?.name}</strong>?
        </Typography>
        <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
          This will remove the contact from your list. Message history will be preserved.
        </Typography>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <Button onClick={handleConfirm} color="error" variant="contained">
          Delete
        </Button>
      </DialogActions>
    </Dialog>
  )
}

// Group interface matching collaboration.ts
export interface Group {
  id: string
  name: string
  fourWords: string
  members: string[]
  admins: string[]
  isPersonal: boolean
  organizationId?: string
}

interface EditGroupDialogProps {
  open: boolean
  group: Group | null
  onClose: () => void
  onSave: (id: string, updates: Partial<Group>) => void
  onAddMember: (groupId: string, userId: string, role: string) => Promise<void>
  onRemoveMember: (groupId: string, userId: string) => Promise<void>
  availableUsers: Array<{ id: string; name: string; fourWords: string }>
}

export const EditGroupDialog: React.FC<EditGroupDialogProps> = ({
  open,
  group,
  onClose,
  onSave,
  onAddMember,
  onRemoveMember,
  availableUsers,
}) => {
  const [name, setName] = useState(group?.name ?? '')
  const [fourWords, setFourWords] = useState(group?.fourWords ?? '')
  const [error, setError] = useState('')
  const [selectedUserId, setSelectedUserId] = useState('')
  const [selectedRole, setSelectedRole] = useState<'member' | 'admin'>('member')
  const [adding, setAdding] = useState(false)

  React.useEffect(() => {
    if (group) {
      setName(group.name)
      setFourWords(group.fourWords)
    }
  }, [group])

  const validateFourWords = (input: string): boolean => {
    const pattern = /^[a-z]+-[a-z]+-[a-z]+-[a-z]+$/
    return pattern.test(input.trim().toLowerCase())
  }

  const handleSave = () => {
    if (!group) return

    if (!name.trim()) {
      setError('Name is required')
      return
    }
    if (!validateFourWords(fourWords)) {
      setError('Invalid four-word address format')
      return
    }

    onSave(group.id, {
      name: name.trim(),
      fourWords: fourWords.trim().toLowerCase(),
    })

    setError('')
    onClose()
  }

  const handleAddMember = async () => {
    if (!group || !selectedUserId) {
      setError('Please select a user to add')
      return
    }

    setAdding(true)
    setError('')
    try {
      await onAddMember(group.id, selectedUserId, selectedRole)
      setSelectedUserId('')
      setSelectedRole('member')
    } catch (err) {
      setError(`Failed to add member: ${err}`)
    } finally {
      setAdding(false)
    }
  }

  const handleRemoveMember = async (userId: string) => {
    if (!group) return

    try {
      await onRemoveMember(group.id, userId)
    } catch (err) {
      setError(`Failed to remove member: ${err}`)
    }
  }

  const handleClose = () => {
    setError('')
    setSelectedUserId('')
    setSelectedRole('member')
    onClose()
  }

  // Get user name from availableUsers list
  const getUserName = (userId: string): string => {
    const user = availableUsers.find((u) => u.id === userId)
    return user ? user.name : userId
  }

  // Filter out users that are already members
  const nonMemberUsers = availableUsers.filter(
    (user) => !group?.members.includes(user.id)
  )

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="md" fullWidth>
      <DialogTitle>
        <Box display="flex" alignItems="center" justifyContent="space-between">
          <Typography variant="h6">Edit Group</Typography>
          <IconButton onClick={handleClose} size="small">
            <CloseIcon />
          </IconButton>
        </Box>
      </DialogTitle>
      <DialogContent>
        {error && (
          <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError('')}>
            {error}
          </Alert>
        )}

        {/* Group Details Section */}
        <Box sx={{ mb: 3 }}>
          <Typography variant="subtitle1" sx={{ mb: 2, fontWeight: 'bold' }}>
            Group Details
          </Typography>
          <TextField
            autoFocus
            margin="dense"
            label="Group Name"
            type="text"
            fullWidth
            value={name}
            onChange={(e) => setName(e.target.value)}
            sx={{ mb: 2 }}
          />
          <TextField
            margin="dense"
            label="Four-Word Address"
            type="text"
            fullWidth
            value={fourWords}
            onChange={(e) => setFourWords(e.target.value)}
            helperText="The group's four-word network identity"
          />
        </Box>

        <Divider sx={{ my: 2 }} />

        {/* Members Section */}
        <Box>
          <Typography variant="subtitle1" sx={{ mb: 2, fontWeight: 'bold' }}>
            Members ({group?.members.length ?? 0})
          </Typography>

          {/* Add Member Section */}
          <Box sx={{ mb: 2, p: 2, bgcolor: 'action.hover', borderRadius: 1 }}>
            <Typography variant="body2" sx={{ mb: 1 }}>
              Add Member
            </Typography>
            <Box sx={{ display: 'flex', gap: 1, alignItems: 'flex-start' }}>
              <FormControl sx={{ flex: 1 }}>
                <InputLabel>Select User</InputLabel>
                <Select
                  value={selectedUserId}
                  onChange={(e) => setSelectedUserId(e.target.value)}
                  label="Select User"
                  size="small"
                  disabled={adding || nonMemberUsers.length === 0}
                >
                  {nonMemberUsers.map((user) => (
                    <MenuItem key={user.id} value={user.id}>
                      {user.name} ({user.fourWords})
                    </MenuItem>
                  ))}
                  {nonMemberUsers.length === 0 && (
                    <MenuItem value="" disabled>
                      No available users
                    </MenuItem>
                  )}
                </Select>
              </FormControl>
              <FormControl sx={{ minWidth: 120 }}>
                <InputLabel>Role</InputLabel>
                <Select
                  value={selectedRole}
                  onChange={(e) => setSelectedRole(e.target.value as 'member' | 'admin')}
                  label="Role"
                  size="small"
                  disabled={adding}
                >
                  <MenuItem value="member">Member</MenuItem>
                  <MenuItem value="admin">Admin</MenuItem>
                </Select>
              </FormControl>
              <Button
                variant="contained"
                onClick={handleAddMember}
                disabled={adding || !selectedUserId}
                startIcon={<PersonAdd />}
                sx={{ minWidth: 100 }}
              >
                {adding ? 'Adding...' : 'Add'}
              </Button>
            </Box>
          </Box>

          {/* Members List */}
          {group && group.members.length > 0 ? (
            <List sx={{ bgcolor: 'background.paper', borderRadius: 1 }}>
              {group.members.map((userId, index) => {
                const isAdmin = group.admins.includes(userId)
                return (
                  <React.Fragment key={userId}>
                    {index > 0 && <Divider />}
                    <ListItem>
                      <ListItemText
                        primary={getUserName(userId)}
                        secondary={
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                            <Typography variant="caption" color="text.secondary">
                              {userId}
                            </Typography>
                            {isAdmin && (
                              <Chip label="Admin" size="small" color="primary" />
                            )}
                          </Box>
                        }
                      />
                      <ListItemSecondaryAction>
                        <IconButton
                          edge="end"
                          aria-label="remove"
                          onClick={() => handleRemoveMember(userId)}
                          color="error"
                        >
                          <PersonRemove />
                        </IconButton>
                      </ListItemSecondaryAction>
                    </ListItem>
                  </React.Fragment>
                )
              })}
            </List>
          ) : (
            <Alert severity="info">No members yet. Add members to get started.</Alert>
          )}
        </Box>
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClose}>Cancel</Button>
        <Button onClick={handleSave} variant="contained">
          Save Changes
        </Button>
      </DialogActions>
    </Dialog>
  )
}
