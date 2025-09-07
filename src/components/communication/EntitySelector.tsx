/**
 * Entity Selector Dialog
 * 
 * Shows a list of people, groups, and organizations to select from
 * when initiating communication actions without a pre-selected entity
 */

import React, { useState, useEffect } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  ListItemAvatar,
  Avatar,
  TextField,
  InputAdornment,
  Tabs,
  Tab,
  Box,
  Typography,
  Chip,
  Divider,
  IconButton,
  Stack,
  Badge,
} from '@mui/material';
import {
  Close as CloseIcon,
  Search as SearchIcon,
  Person as PersonIcon,
  Group as GroupIcon,
  Business as OrganizationIcon,
  Phone as PhoneIcon,
  VideoCall as VideoIcon,
  ScreenShare as ScreenIcon,
  Folder as StorageIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { mockOrganizations, mockPersonalGroups, mockPersonalUsers } from '../../data/mockCollaborationData';

interface EntitySelectorProps {
  open: boolean;
  onClose: () => void;
  onSelect: (entity: any, type: 'person' | 'group' | 'organization') => void;
  actionType: 'call' | 'video' | 'screen' | 'storage';
  title?: string;
}

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

export const EntitySelector: React.FC<EntitySelectorProps> = ({
  open,
  onClose,
  onSelect,
  actionType,
  title,
}) => {
  const [tabValue, setTabValue] = useState(0);
  const [searchTerm, setSearchTerm] = useState('');
  const [filteredPeople, setFilteredPeople] = useState(mockPersonalUsers);
  const [filteredGroups, setFilteredGroups] = useState(mockPersonalGroups);
  const [filteredOrgs, setFilteredOrgs] = useState(mockOrganizations);

  useEffect(() => {
    const lowerSearch = searchTerm.toLowerCase();
    
    setFilteredPeople(
      mockPersonalUsers.filter(person =>
        person.name.toLowerCase().includes(lowerSearch) ||
        person.networkIdentity?.fourWords?.toLowerCase().includes(lowerSearch)
      )
    );
    
    setFilteredGroups(
      mockPersonalGroups.filter(group =>
        group.name.toLowerCase().includes(lowerSearch) ||
        group.description?.toLowerCase().includes(lowerSearch)
      )
    );
    
    setFilteredOrgs(
      mockOrganizations.filter(org =>
        org.name.toLowerCase().includes(lowerSearch) ||
        org.description?.toLowerCase().includes(lowerSearch)
      )
    );
  }, [searchTerm]);

  const getActionIcon = () => {
    switch (actionType) {
      case 'call':
        return <PhoneIcon />;
      case 'video':
        return <VideoIcon />;
      case 'screen':
        return <ScreenIcon />;
      case 'storage':
        return <StorageIcon />;
      default:
        return <PhoneIcon />;
    }
  };

  const getActionText = () => {
    switch (actionType) {
      case 'call':
        return 'Start Voice Call';
      case 'video':
        return 'Start Video Call';
      case 'screen':
        return 'Share Screen';
      case 'storage':
        return 'Open Storage';
      default:
        return 'Select Action';
    }
  };

  const getDefaultTitle = () => {
    switch (actionType) {
      case 'call':
        return 'Who would you like to call?';
      case 'video':
        return 'Start a video call with...';
      case 'screen':
        return 'Share your screen with...';
      case 'storage':
        return 'Open storage for...';
      default:
        return 'Select a contact';
    }
  };

  const handleEntityClick = (entity: any, type: 'person' | 'group' | 'organization') => {
    onSelect(entity, type);
    onClose();
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      maxWidth="sm"
      fullWidth
      PaperProps={{
        component: motion.div,
        initial: { opacity: 0, scale: 0.95 },
        animate: { opacity: 1, scale: 1 },
        exit: { opacity: 0, scale: 0.95 },
        transition: { duration: 0.2 },
      }}
    >
      <DialogTitle>
        <Stack direction="row" alignItems="center" spacing={2}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, flex: 1 }}>
            {getActionIcon()}
            <Typography variant="h6">
              {title || getDefaultTitle()}
            </Typography>
          </Box>
          <IconButton onClick={onClose} size="small">
            <CloseIcon />
          </IconButton>
        </Stack>
      </DialogTitle>

      <DialogContent>
        <Stack spacing={2}>
          {/* Search Field */}
          <TextField
            fullWidth
            placeholder="Search people, groups, or organizations..."
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
            autoFocus
          />

          {/* Tabs */}
          <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
            <Tabs value={tabValue} onChange={(_, newValue) => setTabValue(newValue)}>
              <Tab 
                label={
                  <Stack direction="row" spacing={1} alignItems="center">
                    <PersonIcon fontSize="small" />
                    <span>People</span>
                    <Chip label={filteredPeople.length} size="small" />
                  </Stack>
                }
              />
              <Tab 
                label={
                  <Stack direction="row" spacing={1} alignItems="center">
                    <GroupIcon fontSize="small" />
                    <span>Groups</span>
                    <Chip label={filteredGroups.length} size="small" />
                  </Stack>
                }
              />
              <Tab 
                label={
                  <Stack direction="row" spacing={1} alignItems="center">
                    <OrganizationIcon fontSize="small" />
                    <span>Organizations</span>
                    <Chip label={filteredOrgs.length} size="small" />
                  </Stack>
                }
              />
            </Tabs>
          </Box>

          {/* People Tab */}
          <TabPanel value={tabValue} index={0}>
            <List sx={{ maxHeight: 300, overflow: 'auto' }}>
              <AnimatePresence>
                {filteredPeople.map((person) => (
                  <motion.div
                    key={person.id}
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ opacity: 1, x: 0 }}
                    exit={{ opacity: 0, x: 20 }}
                    transition={{ duration: 0.2 }}
                  >
                    <ListItem disablePadding>
                      <ListItemButton onClick={() => handleEntityClick(person, 'person')}>
                        <ListItemAvatar>
                          <Badge
                            overlap="circular"
                            anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                            variant="dot"
                            color={(person as any).status === 'online' ? 'success' : 'default'}
                          >
                            <Avatar src={person.avatar}>
                              {person.name.charAt(0)}
                            </Avatar>
                          </Badge>
                        </ListItemAvatar>
                        <ListItemText
                          primary={person.name}
                          secondary={
                            <Stack direction="row" spacing={1} alignItems="center">
                              <Typography variant="caption" color="text.secondary">
                                {person.networkIdentity?.fourWords}
                              </Typography>
                              {(person as any).status === 'online' && (
                                <Chip label="Online" size="small" color="success" sx={{ height: 16 }} />
                              )}
                            </Stack>
                          }
                        />
                      </ListItemButton>
                    </ListItem>
                  </motion.div>
                ))}
              </AnimatePresence>
              {filteredPeople.length === 0 && (
                <ListItem>
                  <ListItemText
                    primary="No people found"
                    secondary="Try a different search term"
                    sx={{ textAlign: 'center', color: 'text.secondary' }}
                  />
                </ListItem>
              )}
            </List>
          </TabPanel>

          {/* Groups Tab */}
          <TabPanel value={tabValue} index={1}>
            <List sx={{ maxHeight: 300, overflow: 'auto' }}>
              <AnimatePresence>
                {filteredGroups.map((group) => (
                  <motion.div
                    key={group.id}
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ opacity: 1, x: 0 }}
                    exit={{ opacity: 0, x: 20 }}
                    transition={{ duration: 0.2 }}
                  >
                    <ListItem disablePadding>
                      <ListItemButton onClick={() => handleEntityClick(group, 'group')}>
                        <ListItemIcon>
                          <GroupIcon />
                        </ListItemIcon>
                        <ListItemText
                          primary={group.name}
                          secondary={
                            <Stack direction="row" spacing={1} alignItems="center">
                              <Typography variant="caption" color="text.secondary">
                                {group.members?.length ?? 0} members
                              </Typography>
                              {group.type && (
                                <Chip label={group.type} size="small" sx={{ height: 16 }} />
                              )}
                            </Stack>
                          }
                        />
                      </ListItemButton>
                    </ListItem>
                  </motion.div>
                ))}
              </AnimatePresence>
              {filteredGroups.length === 0 && (
                <ListItem>
                  <ListItemText
                    primary="No groups found"
                    secondary="Try a different search term"
                    sx={{ textAlign: 'center', color: 'text.secondary' }}
                  />
                </ListItem>
              )}
            </List>
          </TabPanel>

          {/* Organizations Tab */}
          <TabPanel value={tabValue} index={2}>
            <List sx={{ maxHeight: 300, overflow: 'auto' }}>
              <AnimatePresence>
                {filteredOrgs.map((org) => (
                  <motion.div
                    key={org.id}
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ opacity: 1, x: 0 }}
                    exit={{ opacity: 0, x: 20 }}
                    transition={{ duration: 0.2 }}
                  >
                    <ListItem disablePadding>
                      <ListItemButton onClick={() => handleEntityClick(org, 'organization')}>
                        <ListItemIcon>
                          <OrganizationIcon />
                        </ListItemIcon>
                        <ListItemText
                          primary={org.name}
                          secondary={
                            <Stack direction="row" spacing={1} alignItems="center">
                              <Typography variant="caption" color="text.secondary">
                                {org.users?.length || 0} members
                              </Typography>
                              {org.type && (
                                <Chip label={org.type} size="small" sx={{ height: 16 }} />
                              )}
                            </Stack>
                          }
                        />
                      </ListItemButton>
                    </ListItem>
                  </motion.div>
                ))}
              </AnimatePresence>
              {filteredOrgs.length === 0 && (
                <ListItem>
                  <ListItemText
                    primary="No organizations found"
                    secondary="Try a different search term"
                    sx={{ textAlign: 'center', color: 'text.secondary' }}
                  />
                </ListItem>
              )}
            </List>
          </TabPanel>
        </Stack>
      </DialogContent>

      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
      </DialogActions>
    </Dialog>
  );
};

export default EntitySelector;
