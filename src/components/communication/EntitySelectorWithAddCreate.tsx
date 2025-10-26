/**
 * Enhanced Entity Selector Dialog with Add/Create Distinction
 *
 * Clearly separates:
 * - Creating new entities (generates Four-Words)
 * - Adding existing entities (requires Four-Words input)
 * - Selecting from already-known entities
 */

import {
    Add as AddIcon, Business as OrganizationIcon, Check as CheckIcon, Close as CloseIcon, Create as CreateIcon, Group as GroupIcon, Info as InfoIcon, Link as LinkIcon, Person as PersonIcon, Search as SearchIcon
} from '@mui/icons-material';
import {
    Alert, Avatar, Box, Button, Chip, Dialog, DialogActions, DialogContent, DialogTitle, FormControl, FormControlLabel, FormLabel, IconButton, InputAdornment, List,
    ListItem, ListItemAvatar, ListItemButton,
    ListItemIcon,
    ListItemText, Paper, Radio, RadioGroup, Stack, Tab, Tabs, TextField, ToggleButton,
    ToggleButtonGroup, Typography
} from '@mui/material';
import React, { useEffect, useState } from 'react';
import { useEntityDirectory } from '../../contexts/EntityDirectoryContext';
import { validateFourWordIdentity } from '../../utils/identity';

interface EntitySelectorWithAddCreateProps {
  open: boolean;
  onClose: () => void;
  onSelect?: (entity: any, type: 'person' | 'group' | 'organization') => void;
  onCreateNew?: (type: 'person' | 'group' | 'organization', data: any) => void;
  onAddExisting?: (type: 'person' | 'group' | 'organization', fourWords: string, data: any) => void;
  actionType?: 'call' | 'video' | 'screen' | 'storage' | 'manage';
  title?: string;
  allowCreate?: boolean;
  allowAdd?: boolean;
}

type OperationMode = 'select' | 'create' | 'add';
type EntityType = 'person' | 'group' | 'organization';

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props;
  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      {...other}
    >
      {value === index && <Box sx={{ py: 2 }}>{children}</Box>}
    </div>
  );
}

export const EntitySelectorWithAddCreate: React.FC<EntitySelectorWithAddCreateProps> = ({
  open,
  onClose,
  onSelect,
  onCreateNew,
  onAddExisting, actionType: _actionType = 'manage',
  title,
  allowCreate = true,
  allowAdd = true,
}) => {
  const {
    personalUsers,
    personalGroups,
    organizations,
    createContact,
    addExistingContact,
    createGroup,
    addExistingGroup,
    createOrganization,
    addExistingOrganization,
  } = useEntityDirectory();

  const [tabValue, setTabValue] = useState(0);
  const [searchTerm, setSearchTerm] = useState('');
  const [operationMode, setOperationMode] = useState<OperationMode>('select');
  const [selectedType, setSelectedType] = useState<EntityType>('person');

  // Form states for creating new entities
  const [newEntityName, setNewEntityName] = useState('');
  const [newEntityDescription, setNewEntityDescription] = useState('');

  // Form states for adding existing entities
  const [existingFourWords, setExistingFourWords] = useState('');
  const [existingAlias, setExistingAlias] = useState('');
  const [fourWordsValid, setFourWordsValid] = useState<boolean | null>(null);
  const [validationMessage, setValidationMessage] = useState('');

  const [filteredPeople, setFilteredPeople] = useState(personalUsers);
  const [filteredGroups, setFilteredGroups] = useState(personalGroups);
  const [filteredOrgs, setFilteredOrgs] = useState(organizations);

  useEffect(() => {
    const lowerSearch = searchTerm.toLowerCase();

    setFilteredPeople(
      personalUsers.filter(person =>
        person.name.toLowerCase().includes(lowerSearch) ||
        person.networkIdentity?.fourWords?.toLowerCase().includes(lowerSearch)
      )
    );

    setFilteredGroups(
      personalGroups.filter(group =>
        group.name.toLowerCase().includes(lowerSearch) ||
        group.description?.toLowerCase().includes(lowerSearch)
      )
    );

    setFilteredOrgs(
      organizations.filter(org =>
        org.name.toLowerCase().includes(lowerSearch) ||
        org.description?.toLowerCase().includes(lowerSearch)
      )
    );
  }, [searchTerm, personalUsers, personalGroups, organizations]);

  const validateFourWords = async (fourWords: string) => {
    if (!fourWords.trim()) {
      setFourWordsValid(null);
      setValidationMessage('');
      return;
    }

    const isValid = await validateFourWordIdentity(fourWords);
    setFourWordsValid(isValid);

    if (!isValid) {
      setValidationMessage('Invalid Four-Word format. Should be like: ocean-forest-moon-star');
    } else {
      setValidationMessage('Four-Word identity format is valid');
    }
  };

  useEffect(() => {
    const debounceTimer = setTimeout(() => {
      if (existingFourWords) {
        validateFourWords(existingFourWords);
      }
    }, 500);

    return () => clearTimeout(debounceTimer);
  }, [existingFourWords]);

  const handleEntityClick = (entity: any, type: EntityType) => {
    if (onSelect) {
      onSelect(entity, type);
    }
    onClose();
  };

  const handleCreateNew = async () => {
    if (!newEntityName.trim()) return;

    const data = {
      displayName: newEntityName,
      description: newEntityDescription,
    };

    try {
      let result;
      switch (selectedType) {
        case 'person':
          result = await createContact(data);
          break;
        case 'group':
          result = await createGroup(data);
          break;
        case 'organization':
          result = await createOrganization(data);
          break;
      }

      if (result && onCreateNew) {
        onCreateNew(selectedType, result);
      }

      // Reset form
      setNewEntityName('');
      setNewEntityDescription('');
      setOperationMode('select');
      onClose();
    } catch (error) {
      console.error('Failed to create entity:', error);
    }
  };

  const handleAddExisting = async () => {
    if (!existingFourWords.trim() || !fourWordsValid) return;

    const normalizedFourWords = existingFourWords.trim().toLowerCase().replace(/\s+/g, '-');
    const data = {
      fourWords: normalizedFourWords,
      displayName: existingAlias || undefined,
    };

    try {
      let result;
      switch (selectedType) {
        case 'person':
          result = await addExistingContact(data);
          break;
        case 'group':
          result = await addExistingGroup(data);
          break;
        case 'organization':
          result = await addExistingOrganization(data);
          break;
      }

      if (result && onAddExisting) {
        onAddExisting(selectedType, normalizedFourWords, result);
      }

      // Reset form
      setExistingFourWords('');
      setExistingAlias('');
      setOperationMode('select');
      onClose();
    } catch (error) {
      console.error('Failed to add entity:', error);
    }
  };

  const _getEntityIcon = (type: EntityType) => {
    switch (type) {
      case 'person':
        return <PersonIcon />;
      case 'group':
        return <GroupIcon />;
      case 'organization':
        return <OrganizationIcon />;
    }
  };

  const getEntityLabel = (type: EntityType) => {
    switch (type) {
      case 'person':
        return 'Person/Contact';
      case 'group':
        return 'Group';
      case 'organization':
        return 'Organization';
    }
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      maxWidth="sm"
      fullWidth
      PaperProps={{
        sx: { minHeight: 500 }
      }}
    >
      <DialogTitle>
        <Stack direction="row" alignItems="center" justifyContent="space-between">
          <Typography variant="h6">
            {title || 'Select or Add Entity'}
          </Typography>
          <IconButton onClick={onClose} size="small">
            <CloseIcon />
          </IconButton>
        </Stack>
      </DialogTitle>

      <DialogContent>
        <Stack spacing={2}>
          {/* Operation Mode Toggle */}
          <ToggleButtonGroup
            value={operationMode}
            exclusive
            onChange={(_, value) => value && setOperationMode(value)}
            fullWidth
            size="small"
          >
            <ToggleButton value="select">
              <Stack direction="row" spacing={1} alignItems="center">
                <SearchIcon />
                <span>Select Existing</span>
              </Stack>
            </ToggleButton>
            {allowCreate && (
              <ToggleButton value="create">
                <Stack direction="row" spacing={1} alignItems="center">
                  <CreateIcon />
                  <span>Create New</span>
                </Stack>
              </ToggleButton>
            )}
            {allowAdd && (
              <ToggleButton value="add">
                <Stack direction="row" spacing={1} alignItems="center">
                  <AddIcon />
                  <span>Add by Four-Words</span>
                </Stack>
              </ToggleButton>
            )}
          </ToggleButtonGroup>

          {/* Mode-specific content */}
          {operationMode === 'select' && (
            <>
              {/* Search Field */}
              <TextField
                fullWidth
                placeholder="Search by name or Four-Words..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                InputProps={{
                  startAdornment: (
                    <InputAdornment position="start">
                      <SearchIcon />
                    </InputAdornment>
                  ),
                }}
                size="small"
              />

              {/* Entity Type Tabs */}
              <Tabs value={tabValue} onChange={(_, value) => setTabValue(value)}>
                <Tab icon={<PersonIcon />} label="People" />
                <Tab icon={<GroupIcon />} label="Groups" />
                <Tab icon={<OrganizationIcon />} label="Organizations" />
              </Tabs>

              {/* People Tab */}
              <TabPanel value={tabValue} index={0}>
                <List sx={{ maxHeight: 250, overflow: 'auto' }}>
                  {filteredPeople.map((person) => (
                    <ListItem key={person.id} disablePadding>
                      <ListItemButton onClick={() => handleEntityClick(person, 'person')}>
                        <ListItemAvatar>
                          <Avatar src={person.avatar}>
                            {person.name.charAt(0)}
                          </Avatar>
                        </ListItemAvatar>
                        <ListItemText
                          primary={person.name}
                          secondary={person.networkIdentity?.fourWords}
                        />
                        {person.networkIdentity?.isOwned && (
                          <Chip label="You" size="small" color="primary" />
                        )}
                      </ListItemButton>
                    </ListItem>
                  ))}
                  {filteredPeople.length === 0 && (
                    <ListItem>
                      <ListItemText
                        primary="No people found"
                        secondary="Try creating or adding one"
                        sx={{ textAlign: 'center', color: 'text.secondary' }}
                      />
                    </ListItem>
                  )}
                </List>
              </TabPanel>

              {/* Groups Tab */}
              <TabPanel value={tabValue} index={1}>
                <List sx={{ maxHeight: 250, overflow: 'auto' }}>
                  {filteredGroups.map((group) => (
                    <ListItem key={group.id} disablePadding>
                      <ListItemButton onClick={() => handleEntityClick(group, 'group')}>
                        <ListItemIcon>
                          <GroupIcon />
                        </ListItemIcon>
                        <ListItemText
                          primary={group.name}
                          secondary={`${group.members?.length || 0} members`}
                        />
                        {group.networkIdentity?.isOwned && (
                          <Chip label="Admin" size="small" color="primary" />
                        )}
                      </ListItemButton>
                    </ListItem>
                  ))}
                  {filteredGroups.length === 0 && (
                    <ListItem>
                      <ListItemText
                        primary="No groups found"
                        secondary="Try creating or adding one"
                        sx={{ textAlign: 'center', color: 'text.secondary' }}
                      />
                    </ListItem>
                  )}
                </List>
              </TabPanel>

              {/* Organizations Tab */}
              <TabPanel value={tabValue} index={2}>
                <List sx={{ maxHeight: 250, overflow: 'auto' }}>
                  {filteredOrgs.map((org) => (
                    <ListItem key={org.id} disablePadding>
                      <ListItemButton onClick={() => handleEntityClick(org, 'organization')}>
                        <ListItemIcon>
                          <OrganizationIcon />
                        </ListItemIcon>
                        <ListItemText
                          primary={org.name}
                          secondary={`${org.users?.length || 0} members`}
                        />
                        {org.networkIdentity?.isOwned && (
                          <Chip label="Owner" size="small" color="primary" />
                        )}
                      </ListItemButton>
                    </ListItem>
                  ))}
                  {filteredOrgs.length === 0 && (
                    <ListItem>
                      <ListItemText
                        primary="No organizations found"
                        secondary="Try creating or adding one"
                        sx={{ textAlign: 'center', color: 'text.secondary' }}
                      />
                    </ListItem>
                  )}
                </List>
              </TabPanel>
            </>
          )}

          {operationMode === 'create' && (
            <>
              <Alert severity="info" icon={<InfoIcon />}>
                <Typography variant="body2">
                  <strong>Creating a new entity:</strong> A unique Four-Word identity will be automatically generated for you.
                  You will own and control this entity.
                </Typography>
              </Alert>

              {/* Entity Type Selection */}
              <FormControl component="fieldset">
                <FormLabel component="legend">Entity Type</FormLabel>
                <RadioGroup
                  row
                  value={selectedType}
                  onChange={(e) => setSelectedType(e.target.value as EntityType)}
                >
                  <FormControlLabel value="person" control={<Radio />} label="Person" />
                  <FormControlLabel value="group" control={<Radio />} label="Group" />
                  <FormControlLabel value="organization" control={<Radio />} label="Organization" />
                </RadioGroup>
              </FormControl>

              {/* Create Form */}
              <TextField
                fullWidth
                label="Display Name"
                value={newEntityName}
                onChange={(e) => setNewEntityName(e.target.value)}
                placeholder={`Enter ${selectedType} name...`}
                required
              />

              <TextField
                fullWidth
                label="Description (optional)"
                value={newEntityDescription}
                onChange={(e) => setNewEntityDescription(e.target.value)}
                placeholder={`Brief description of the ${selectedType}...`}
                multiline
                rows={2}
              />

              <Paper sx={{ p: 2, bgcolor: 'background.default' }}>
                <Typography variant="caption" color="text.secondary">
                  Four-Words will be generated: <strong>????-????-????-????</strong>
                </Typography>
              </Paper>
            </>
          )}

          {operationMode === 'add' && (
            <>
              <Alert severity="warning" icon={<LinkIcon />}>
                <Typography variant="body2">
                  <strong>Adding an existing entity:</strong> You need the Four-Word identity of the {selectedType} you want to add.
                  You won't own this entity, just connect to it.
                </Typography>
              </Alert>

              {/* Entity Type Selection */}
              <FormControl component="fieldset">
                <FormLabel component="legend">Entity Type</FormLabel>
                <RadioGroup
                  row
                  value={selectedType}
                  onChange={(e) => setSelectedType(e.target.value as EntityType)}
                >
                  <FormControlLabel value="person" control={<Radio />} label="Person" />
                  <FormControlLabel value="group" control={<Radio />} label="Group" />
                  <FormControlLabel value="organization" control={<Radio />} label="Organization" />
                </RadioGroup>
              </FormControl>

              {/* Add Form */}
              <TextField
                fullWidth
                label="Four-Word Identity"
                value={existingFourWords}
                onChange={(e) => setExistingFourWords(e.target.value)}
                placeholder="e.g., ocean-forest-moon-star"
                required
                error={fourWordsValid === false}
                helperText={validationMessage}
                InputProps={{
                  endAdornment: fourWordsValid === true && (
                    <InputAdornment position="end">
                      <CheckIcon color="success" />
                    </InputAdornment>
                  ),
                }}
              />

              <TextField
                fullWidth
                label="Local Alias (optional)"
                value={existingAlias}
                onChange={(e) => setExistingAlias(e.target.value)}
                placeholder={`What to call this ${selectedType} locally...`}
                helperText="A friendly name for your reference"
              />
            </>
          )}
        </Stack>
      </DialogContent>

      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>

        {operationMode === 'create' && (
          <Button
            variant="contained"
            startIcon={<CreateIcon />}
            onClick={handleCreateNew}
            disabled={!newEntityName.trim()}
          >
            Create {getEntityLabel(selectedType)}
          </Button>
        )}

        {operationMode === 'add' && (
          <Button
            variant="contained"
            startIcon={<AddIcon />}
            onClick={handleAddExisting}
            disabled={!existingFourWords.trim() || fourWordsValid !== true}
          >
            Add {getEntityLabel(selectedType)}
          </Button>
        )}
      </DialogActions>
    </Dialog>
  );
};

export default EntitySelectorWithAddCreate;