import React, { useState } from 'react'
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  TextField,
  Box,
  Typography,
  IconButton,
  Alert,
} from '@mui/material'
import { Close as CloseIcon } from '@mui/icons-material'

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
}

export const AddContactDialog: React.FC<AddContactDialogProps> = ({ open, onClose, onSave }) => {
  const [name, setName] = useState('')
  const [fourWords, setFourWords] = useState('')
  const [error, setError] = useState('')

  const validateFourWords = (input: string): boolean => {
    const pattern = /^[a-z]+-[a-z]+-[a-z]+-[a-z]+$/
    return pattern.test(input.trim().toLowerCase())
  }

  const handleSave = () => {
    if (!name.trim()) {
      setError('Name is required')
      return
    }
    if (!validateFourWords(fourWords)) {
      setError('Invalid four-word address format (e.g., ocean-forest-moon-star)')
      return
    }

    onSave({
      name: name.trim(),
      fourWords: fourWords.trim().toLowerCase(),
      snippet: 'No messages yet',
      time: 'Never',
      online: false,
      starred: false,
    })

    // Reset form
    setName('')
    setFourWords('')
    setError('')
    onClose()
  }

  const handleClose = () => {
    setName('')
    setFourWords('')
    setError('')
    onClose()
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
          placeholder="ocean-forest-moon-star"
          helperText="Enter the contact's four-word network identity"
        />
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClose}>Cancel</Button>
        <Button onClick={handleSave} variant="contained">
          Add Contact
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
