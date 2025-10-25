import { Add, Close as CloseIcon, GroupAdd } from '@mui/icons-material'
import {
    Alert, Box, Button, Checkbox, Chip, Dialog, DialogActions, DialogContent, DialogTitle, FormControl, IconButton, InputLabel, ListItemText, MenuItem, OutlinedInput, Select, Tab, Tabs, TextField, Typography
} from '@mui/material'
import React, { useState } from 'react'
import type { Contact } from './ContactManagementDialogs'

export type EntityType = 'group' | 'organization' | 'channel' | 'project'
export type EntityScope = 'personal' | 'organization'

export interface EntityCreationResult {
  name: string
  description?: string
  fourWords?: string
  memberIds?: string[]
  scope?: EntityScope
}

interface EntityCreationDialogProps {
  open: boolean
  onClose: () => void
  onSave: (entity: EntityCreationResult) => void
  entityType: EntityType
  scope?: EntityScope
  availableMembers?: Contact[] // Contacts that can be added as members
  onGenerateIdentity?: () => Promise<string>
}

const EntityLabels = {
  group: { singular: 'Group', plural: 'Groups', article: 'a' },
  organization: { singular: 'Organization', plural: 'Organizations', article: 'an' },
  channel: { singular: 'Channel', plural: 'Channels', article: 'a' },
  project: { singular: 'Project', plural: 'Projects', article: 'a' },
}

export const EntityCreationDialog: React.FC<EntityCreationDialogProps> = ({
  open,
  onClose,
  onSave,
  entityType,
  scope,
  availableMembers = [],
  onGenerateIdentity,
}) => {
  const [mode, setMode] = useState<'existing' | 'new'>('new')
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [fourWords, setFourWords] = useState('')
  const [selectedMemberIds, setSelectedMemberIds] = useState<string[]>([])
  const [error, setError] = useState('')
  const [generatingIdentity, setGeneratingIdentity] = useState(false)

  // Only access labels if entityType is defined
  if (!entityType || !open) {
    return null
  }

  const labels = EntityLabels[entityType]

  // Normalize four words - accept spaces or dashes
  const normalizeFourWords = (input: string): string => {
    return input.trim().toLowerCase().replace(/\s+/g, '-')
  }

  // Validate against actual four-word dictionary format
  const validateFourWords = async (input: string): Promise<boolean> => {
    const normalized = normalizeFourWords(input)
    const pattern = /^[a-z]+[\s-][a-z]+[\s-][a-z]+[\s-][a-z]+$/

    if (!pattern.test(input.trim().toLowerCase())) {
      return false
    }

    try {
      if (typeof window !== 'undefined' && '__TAURI__' in window) {
        const { invoke } = await import('@tauri-apps/api/core')
        const isValid = await invoke<boolean>('validate_four_words', { fourWords: normalized })
        return isValid
      }
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

      onSave({
        name: name.trim(),
        description: description.trim() || undefined,
        fourWords: normalizeFourWords(fourWords),
        scope,
      })
    } else {
      // New entity mode
      if (entityType !== 'organization' && !fourWords.trim()) {
        setError('Please generate a four-word identity first')
        return
      }

      onSave({
        name: name.trim(),
        description: description.trim() || undefined,
        fourWords: fourWords ? normalizeFourWords(fourWords) : undefined,
        memberIds: selectedMemberIds.length > 0 ? selectedMemberIds : undefined,
        scope,
      })
    }

    // Reset form
    setName('')
    setDescription('')
    setFourWords('')
    setSelectedMemberIds([])
    setError('')
    setMode('new')
    onClose()
  }

  const handleClose = () => {
    setName('')
    setDescription('')
    setFourWords('')
    setSelectedMemberIds([])
    setError('')
    setMode('new')
    onClose()
  }

  const handleModeChange = (_: React.SyntheticEvent, newMode: 'existing' | 'new') => {
    setMode(newMode)
    setFourWords('')
    setError('')
  }

  const handleMemberChange = (event: any) => {
    const value = event.target.value
    setSelectedMemberIds(typeof value === 'string' ? value.split(',') : value)
  }

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
      <DialogTitle>
        <Box display="flex" alignItems="center" justifyContent="space-between">
          <Typography variant="h6">
            {mode === 'existing' ? `Add ${labels.singular}` : `Create ${labels.singular}`}
          </Typography>
          <IconButton onClick={handleClose} size="small">
            <CloseIcon />
          </IconButton>
        </Box>
        <Tabs value={mode} onChange={handleModeChange} sx={{ mt: 1 }}>
          <Tab icon={<Add />} iconPosition="start" label="Create New" value="new" />
          <Tab icon={<GroupAdd />} iconPosition="start" label="Add Existing" value="existing" />
        </Tabs>
      </DialogTitle>
      <DialogContent>
        {error && (
          <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError('')}>
            {error}
          </Alert>
        )}

        {mode === 'new' ? (
          <>
            <Alert severity="info" sx={{ mb: 2 }}>
              Create {labels.article} new {entityType} {scope ? `(${scope})` : ''}
            </Alert>
            <TextField
              autoFocus
              margin="dense"
              label="Name"
              type="text"
              fullWidth
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={`My ${labels.singular}`}
              sx={{ mb: 2 }}
            />
            <TextField
              margin="dense"
              label="Description (optional)"
              type="text"
              fullWidth
              multiline
              rows={2}
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder={`Description of this ${entityType}`}
              sx={{ mb: 2 }}
            />

            {/* Four-word identity for groups/channels/projects (not orgs) */}
            {entityType !== 'organization' && (
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
                    disabled={generatingIdentity || !onGenerateIdentity}
                    fullWidth
                  >
                    {generatingIdentity ? 'Generating...' : 'Generate Identity'}
                  </Button>
                )}
              </Box>
            )}

            {/* Member selection */}
            {availableMembers.length > 0 && (
              <FormControl fullWidth sx={{ mb: 2 }}>
                <InputLabel id="members-label">Add Members (optional)</InputLabel>
                <Select
                  labelId="members-label"
                  multiple
                  value={selectedMemberIds}
                  onChange={handleMemberChange}
                  input={<OutlinedInput label="Add Members (optional)" />}
                  renderValue={(selected) => (
                    <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
                      {selected.map((id) => {
                        const member = availableMembers.find((m) => m.id === id)
                        return <Chip key={id} label={member?.name || id} size="small" />
                      })}
                    </Box>
                  )}
                >
                  {availableMembers.map((member) => (
                    <MenuItem key={member.id} value={member.id}>
                      <Checkbox checked={selectedMemberIds.indexOf(member.id) > -1} />
                      <ListItemText primary={member.name} secondary={member.fourWords} />
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
            )}

            <Alert severity="warning" sx={{ mt: 2 }}>
              <Typography variant="caption">
                <strong>Note:</strong> The {entityType} will be created when you save.
                {selectedMemberIds.length > 0 && ' Selected members will be added automatically.'}
              </Typography>
            </Alert>
          </>
        ) : (
          <>
            <Alert severity="info" sx={{ mb: 2 }}>
              Add an existing {entityType} by entering its four-word identity
            </Alert>
            <TextField
              autoFocus
              margin="dense"
              label="Display Name"
              type="text"
              fullWidth
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={`My ${labels.singular}`}
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
              helperText={`Enter the existing ${entityType}'s four-word identity (spaces or dashes)`}
            />
          </>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClose}>Cancel</Button>
        <Button onClick={handleSave} variant="contained" disabled={generatingIdentity}>
          {mode === 'existing' ? `Add ${labels.singular}` : `Create ${labels.singular}`}
        </Button>
      </DialogActions>
    </Dialog>
  )
}
