import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Box,
  Card,
  CardContent,
  Typography,
  Button,
  TextField,
  List,
  ListItem,
  ListItemText,
  ListItemButton,
  Chip,
  Alert,
  CircularProgress,
  Stack,
  Switch,
  FormControlLabel,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  IconButton,
  Tooltip,
} from '@mui/material';
import {
  Security as SecurityIcon,
  Group as GroupIcon,
  Message as MessageIcon,
  Settings as SettingsIcon,
  Refresh as RefreshIcon,
  Add as AddIcon,
  Delete as DeleteIcon,
} from '@mui/icons-material';

interface MlsGroup {
  group_id: string;
  epoch: number;
  member_count: number;
  members: string[];
}

interface MlsStatus {
  initialized: boolean;
  group_count: number;
  config: {
    enable_pqc: boolean;
    max_epochs: number;
    key_rotation_interval: number;
  };
}



export const MlsInterface: React.FC = () => {
  const [mlsStatus, setMlsStatus] = useState<MlsStatus | null>(null);
  const [groups, setGroups] = useState<MlsGroup[]>([]);
  const [selectedGroup, setSelectedGroup] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // MLS Configuration
  const [enablePqc, setEnablePqc] = useState(true);
  const [maxEpochs, setMaxEpochs] = useState(1000);
  const [keyRotationInterval, setKeyRotationInterval] = useState(100);

  // Dialog states
  const [createGroupDialog, setCreateGroupDialog] = useState(false);
  const [joinGroupDialog, setJoinGroupDialog] = useState(false);
  const [groupName, setGroupName] = useState('');
  const [welcomeData, setWelcomeData] = useState('');

  // Initialize MLS client
  const initializeMls = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<boolean>('core_mls_initialize', {
        enable_pqc: enablePqc,
        max_epochs: maxEpochs,
        key_rotation_interval: keyRotationInterval,
      });

      if (result) {
        setSuccess('MLS client initialized successfully');
        await refreshStatus();
      }
    } catch (err) {
      setError(`Failed to initialize MLS: ${err}`);
    } finally {
      setLoading(false);
    }
  }, [enablePqc, maxEpochs, keyRotationInterval]);

  // Refresh MLS status
  const refreshStatus = useCallback(async () => {
    try {
      const status = await invoke<MlsStatus>('core_mls_get_status');
      setMlsStatus(status);

      const groupsList = await invoke<string[]>('core_mls_list_groups');
      const groupsData: MlsGroup[] = [];

      for (const groupId of groupsList) {
        try {
          const groupInfo = await invoke<any>('core_mls_get_group', { group_id: groupId });
          groupsData.push(groupInfo);
        } catch (err) {
          console.error(`Failed to get group info for ${groupId}:`, err);
        }
      }

      setGroups(groupsData);
    } catch (err) {
      console.error('Failed to refresh MLS status:', err);
    }
  }, []);

  // Create new group
  const createGroup = useCallback(async () => {
    if (!groupName.trim()) return;

    setLoading(true);
    setError(null);
    try {
      // Get current identity (this would need to be implemented based on your auth system)
      const identityJson = JSON.stringify({
        four_words: ['test', 'user', 'identity', 'one'],
        public_key: 'test_public_key',
        signature: 'test_signature'
      });

       await invoke<string>('core_mls_create_group', {
         group_name: groupName,
         identity_json: identityJson,
       });

      setSuccess(`Group "${groupName}" created successfully`);
      setCreateGroupDialog(false);
      setGroupName('');
      await refreshStatus();
    } catch (err) {
      setError(`Failed to create group: ${err}`);
    } finally {
      setLoading(false);
    }
  }, [groupName, refreshStatus]);

  // Join group
  const joinGroup = useCallback(async () => {
    if (!welcomeData.trim()) return;

    setLoading(true);
    setError(null);
    try {
      // Get current identity
      const identityJson = JSON.stringify({
        four_words: ['test', 'user', 'identity', 'one'],
        public_key: 'test_public_key',
        signature: 'test_signature'
      });

      await invoke<string>('core_mls_join_group', {
        welcome_data: Array.from(new TextEncoder().encode(welcomeData)),
        identity_json: identityJson,
      });

      setSuccess('Successfully joined group');
      setJoinGroupDialog(false);
      setWelcomeData('');
      await refreshStatus();
    } catch (err) {
      setError(`Failed to join group: ${err}`);
    } finally {
      setLoading(false);
    }
  }, [welcomeData, refreshStatus]);

  // Send message to group
  const sendMessage = useCallback(async (groupId: string, message: string) => {
    setLoading(true);
    setError(null);
    try {
      const identityJson = JSON.stringify({
        four_words: ['test', 'user', 'identity', 'one'],
        public_key: 'test_public_key',
        signature: 'test_signature'
      });

      await invoke<string>('core_mls_send_message', {
        group_id: groupId,
        content: Array.from(new TextEncoder().encode(message)),
        content_type: 'text/plain',
        identity_json: identityJson,
      });

      setSuccess('Message sent successfully');
      // In a real implementation, you'd refresh messages here
    } catch (err) {
      setError(`Failed to send message: ${err}`);
    } finally {
      setLoading(false);
    }
  }, []);

  // Leave group
  const leaveGroup = useCallback(async (groupId: string) => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<boolean>('core_mls_leave_group', {
        group_id: groupId,
      });

      if (result) {
        setSuccess('Successfully left group');
        await refreshStatus();
      }
    } catch (err) {
      setError(`Failed to leave group: ${err}`);
    } finally {
      setLoading(false);
    }
  }, [refreshStatus]);

  // Load messages for selected group
  const loadGroupMessages = useCallback(async (groupId: string) => {
    // This would be implemented when message storage/retrieval is available
    console.log(`Loading messages for group: ${groupId}`);
  }, []);

  // Initialize on component mount
  useEffect(() => {
    refreshStatus();
  }, [refreshStatus]);

  return (
    <Box sx={{ p: 3, maxWidth: 1200, mx: 'auto' }}>
      <Typography variant="h4" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
        <SecurityIcon />
        Message Layer Security (MLS)
      </Typography>

      {/* Status and Configuration */}
      <Card sx={{ mb: 3 }}>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            MLS Status & Configuration
          </Typography>

          {mlsStatus ? (
            <Stack spacing={2}>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                <Chip
                  label={mlsStatus.initialized ? 'Initialized' : 'Not Initialized'}
                  color={mlsStatus.initialized ? 'success' : 'error'}
                  variant="outlined"
                />
                <Typography variant="body2">
                  Groups: {mlsStatus.group_count}
                </Typography>
              </Box>

              <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap' }}>
                <FormControlLabel
                  control={
                    <Switch
                      checked={enablePqc}
                      onChange={(e) => setEnablePqc(e.target.checked)}
                      disabled={mlsStatus.initialized}
                    />
                  }
                  label="Enable Post-Quantum Cryptography"
                />
                <TextField
                  label="Max Epochs"
                  type="number"
                  value={maxEpochs}
                  onChange={(e) => setMaxEpochs(Number(e.target.value))}
                  disabled={mlsStatus.initialized}
                  size="small"
                  sx={{ width: 120 }}
                />
                <TextField
                  label="Key Rotation Interval"
                  type="number"
                  value={keyRotationInterval}
                  onChange={(e) => setKeyRotationInterval(Number(e.target.value))}
                  disabled={mlsStatus.initialized}
                  size="small"
                  sx={{ width: 150 }}
                />
              </Box>

              <Button
                variant="contained"
                onClick={initializeMls}
                disabled={mlsStatus.initialized || loading}
                startIcon={loading ? <CircularProgress size={20} /> : <SettingsIcon />}
              >
                {mlsStatus.initialized ? 'MLS Initialized' : 'Initialize MLS'}
              </Button>
            </Stack>
          ) : (
            <Typography variant="body2" color="text.secondary">
              MLS client not initialized
            </Typography>
          )}
        </CardContent>
      </Card>

      {/* Error/Success Messages */}
      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}
      {success && (
        <Alert severity="success" sx={{ mb: 2 }} onClose={() => setSuccess(null)}>
          {success}
        </Alert>
      )}

      {/* Group Management */}
      <Card sx={{ mb: 3 }}>
        <CardContent>
          <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
            <Typography variant="h6">
              <GroupIcon sx={{ mr: 1, verticalAlign: 'middle' }} />
              MLS Groups
            </Typography>
            <Box>
              <Button
                variant="outlined"
                startIcon={<RefreshIcon />}
                onClick={refreshStatus}
                disabled={loading}
                sx={{ mr: 1 }}
              >
                Refresh
              </Button>
              <Button
                variant="contained"
                startIcon={<AddIcon />}
                onClick={() => setCreateGroupDialog(true)}
                disabled={!mlsStatus?.initialized}
              >
                Create Group
              </Button>
            </Box>
          </Box>

          {groups.length === 0 ? (
            <Typography variant="body2" color="text.secondary">
              No groups found. Create your first group to get started.
            </Typography>
          ) : (
            <List>
              {groups.map((group) => (
                <ListItem key={group.group_id} divider>
                  <ListItemButton
                    selected={selectedGroup === group.group_id}
                    onClick={() => {
                      setSelectedGroup(group.group_id);
                      loadGroupMessages(group.group_id);
                    }}
                  >
                    <ListItemText
                      primary={
                        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                          <Typography variant="subtitle1">
                            Group {group.group_id.substring(0, 8)}...
                          </Typography>
                          <Chip
                            label={`${group.member_count} members`}
                            size="small"
                            variant="outlined"
                          />
                          <Chip
                            label={`Epoch ${group.epoch}`}
                            size="small"
                            color="primary"
                          />
                        </Box>
                      }
                      secondary={
                        <Typography variant="body2" color="text.secondary">
                          {group.members.length} members
                        </Typography>
                      }
                    />
                  </ListItemButton>
                  <Tooltip title="Leave Group">
                    <IconButton
                      color="error"
                      onClick={() => leaveGroup(group.group_id)}
                      disabled={loading}
                    >
                      <DeleteIcon />
                    </IconButton>
                  </Tooltip>
                </ListItem>
              ))}
            </List>
          )}
        </CardContent>
      </Card>

      {/* Message Interface */}
      {selectedGroup && (
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              <MessageIcon sx={{ mr: 1, verticalAlign: 'middle' }} />
              Group Messages
            </Typography>

            <Box sx={{ display: 'flex', gap: 1, mb: 2 }}>
              <TextField
                fullWidth
                placeholder="Type your message..."
                variant="outlined"
                size="small"
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    const target = e.target as HTMLInputElement;
                    sendMessage(selectedGroup, target.value);
                    target.value = '';
                  }
                }}
              />
              <Button
                variant="contained"
                onClick={() => {
                  const input = document.querySelector('input[placeholder="Type your message..."]') as HTMLInputElement;
                  if (input?.value) {
                    sendMessage(selectedGroup, input.value);
                    input.value = '';
                  }
                }}
                disabled={loading}
              >
                Send
              </Button>
            </Box>

            <Typography variant="body2" color="text.secondary">
              Messages will appear here once the message storage system is implemented.
            </Typography>
          </CardContent>
        </Card>
      )}

      {/* Create Group Dialog */}
      <Dialog open={createGroupDialog} onClose={() => setCreateGroupDialog(false)}>
        <DialogTitle>Create New MLS Group</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Group Name"
            fullWidth
            variant="outlined"
            value={groupName}
            onChange={(e) => setGroupName(e.target.value)}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateGroupDialog(false)}>Cancel</Button>
          <Button onClick={createGroup} disabled={!groupName.trim() || loading}>
            Create Group
          </Button>
        </DialogActions>
      </Dialog>

      {/* Join Group Dialog */}
      <Dialog open={joinGroupDialog} onClose={() => setJoinGroupDialog(false)}>
        <DialogTitle>Join MLS Group</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Welcome Data (Base64)"
            fullWidth
            variant="outlined"
            multiline
            rows={4}
            value={welcomeData}
            onChange={(e) => setWelcomeData(e.target.value)}
            placeholder="Paste the welcome message data here..."
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setJoinGroupDialog(false)}>Cancel</Button>
          <Button onClick={joinGroup} disabled={!welcomeData.trim() || loading}>
            Join Group
          </Button>
        </DialogActions>
      </Dialog>

      {/* Join Group Button */}
      <Box sx={{ mt: 2, textAlign: 'center' }}>
        <Button
          variant="outlined"
          onClick={() => setJoinGroupDialog(true)}
          disabled={!mlsStatus?.initialized}
        >
          Join Existing Group
        </Button>
      </Box>
    </Box>
  );
};