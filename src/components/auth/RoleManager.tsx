import React, { useState, useEffect } from 'react';
import {
  Box,
  Paper,
  Typography,
  Button,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  IconButton,
  Chip,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Stack,
  Alert,
  Autocomplete,
  Card,
  CardContent,
  Grid,
  Avatar,
  Tooltip,
  CircularProgress,
} from '@mui/material';
import {
  Add as AddIcon,
  Edit as EditIcon,
  Delete as DeleteIcon,
  Person as PersonIcon,
  Group as GroupIcon,
  Shield as ShieldIcon,
  AdminPanelSettings as AdminIcon,
  SupervisedUserCircle as SupervisedIcon,
  Search as SearchIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { useAuth, UserIdentity } from '../../contexts/AuthContext';
import { useRBAC } from '../../hooks/useRBAC';
import { 
  ResourceType, 
  Action, 
  SystemRole, 
  OrganizationRole, 
  ProjectRole 
} from '../../utils/rbac';
import { ResponsiveContainer, ResponsiveGrid } from '../responsive';

// Role assignment interface
interface RoleAssignment {
  id: string;
  userId: string;
  userName: string;
  userAvatar?: string;
  role: string;
  scope?: string;
  scopeType: 'system' | 'organization' | 'project';
  scopeId?: string;
  scopeName?: string;
  assignedAt: string;
  assignedBy: string;
  expiresAt?: string;
}

// Props for role manager
interface RoleManagerProps {
  organizationId?: string;
  projectId?: string;
  onClose?: () => void;
}

export const RoleManager: React.FC<RoleManagerProps> = ({
  organizationId,
  projectId,
  onClose,
}) => {
  const { authState } = useAuth();
  const rbac = useRBAC();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [roleAssignments, setRoleAssignments] = useState<RoleAssignment[]>([]);
  const [availableUsers, setAvailableUsers] = useState<UserIdentity[]>([]);
  const [addDialogOpen, setAddDialogOpen] = useState(false);
  const [editingAssignment, setEditingAssignment] = useState<RoleAssignment | null>(null);
  const [searchTerm, setSearchTerm] = useState('');

  const [newAssignment, setNewAssignment] = useState({
    userId: '',
    role: '',
    expiresAt: '',
  });

  // Determine context and available roles
  const contextType = projectId ? 'project' : organizationId ? 'organization' : 'system';
  const contextId = projectId || organizationId;

  const availableRoles = React.useMemo(() => {
    switch (contextType) {
      case 'system':
        return Object.values(SystemRole);
      case 'organization':
        return Object.values(OrganizationRole);
      case 'project':
        return Object.values(ProjectRole);
      default:
        return [];
    }
  }, [contextType]);

  // Check if current user can manage roles in this context
  const canManageRoles = React.useMemo(() => {
    if (contextType === 'system') {
      return rbac.canManage(ResourceType.SYSTEM);
    } else if (contextType === 'organization') {
      return rbac.canManage(ResourceType.ORGANIZATION, contextId, organizationId);
    } else if (contextType === 'project') {
      return rbac.canManage(ResourceType.PROJECT, contextId, organizationId, projectId);
    }
    return false;
  }, [rbac, contextType, contextId, organizationId, projectId]);

  // Load role assignments
  useEffect(() => {
    loadRoleAssignments();
    loadAvailableUsers();
  }, [contextType, contextId]);

  const loadRoleAssignments = async () => {
    setLoading(true);
    try {
      // Mock data - in real app, fetch from backend
      const mockAssignments: RoleAssignment[] = [
        {
          id: '1',
          userId: 'user1',
          userName: 'Alice Smith',
          role: 'admin',
          scopeType: contextType as any,
          scopeId: contextId,
          scopeName: contextType === 'organization' ? 'My Organization' : 'My Project',
          assignedAt: new Date().toISOString(),
          assignedBy: 'system',
        },
        {
          id: '2',
          userId: 'user2',
          userName: 'Bob Johnson',
          role: 'member',
          scopeType: contextType as any,
          scopeId: contextId,
          scopeName: contextType === 'organization' ? 'My Organization' : 'My Project',
          assignedAt: new Date().toISOString(),
          assignedBy: authState.user?.id || 'system',
        },
      ];
      setRoleAssignments(mockAssignments);
    } catch (err) {
      setError('Failed to load role assignments');
      console.error('Failed to load role assignments:', err);
    } finally {
      setLoading(false);
    }
  };

  const loadAvailableUsers = async () => {
    try {
      // Mock data - in real app, fetch from backend
      const mockUsers: UserIdentity[] = [
        {
          id: 'user3',
          publicKey: 'mock-key-3',
          fourWordAddress: 'swift-river-calm-peak',
          name: 'Charlie Brown',
          profile: {},
          permissions: [],
          createdAt: new Date().toISOString(),
          lastActive: new Date().toISOString(),
        },
        {
          id: 'user4',
          publicKey: 'mock-key-4',
          fourWordAddress: 'bright-moon-soft-cloud',
          name: 'Diana Prince',
          profile: {},
          permissions: [],
          createdAt: new Date().toISOString(),
          lastActive: new Date().toISOString(),
        },
      ];
      setAvailableUsers(mockUsers);
    } catch (err) {
      console.error('Failed to load available users:', err);
    }
  };

  const handleAddRole = async () => {
    if (!newAssignment.userId || !newAssignment.role) {
      setError('Please select a user and role');
      return;
    }

    setLoading(true);
    try {
      const user = availableUsers.find(u => u.id === newAssignment.userId);
      if (!user) throw new Error('User not found');

      const assignment: RoleAssignment = {
        id: Date.now().toString(),
        userId: user.id,
        userName: user.name,
        role: newAssignment.role,
        scopeType: contextType as any,
        scopeId: contextId,
        scopeName: contextType === 'organization' ? 'My Organization' : 'My Project',
        assignedAt: new Date().toISOString(),
        assignedBy: authState.user?.id || 'system',
        expiresAt: newAssignment.expiresAt || undefined,
      };

      // In real app, save to backend
      setRoleAssignments(prev => [...prev, assignment]);
      setAddDialogOpen(false);
      setNewAssignment({ userId: '', role: '', expiresAt: '' });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add role');
    } finally {
      setLoading(false);
    }
  };

  const handleRemoveRole = async (assignmentId: string) => {
    setLoading(true);
    try {
      // In real app, remove from backend
      setRoleAssignments(prev => prev.filter(a => a.id !== assignmentId));
    } catch (err) {
      setError('Failed to remove role assignment');
    } finally {
      setLoading(false);
    }
  };

  const getRoleIcon = (role: string) => {
    if (role.includes('admin') || role.includes('owner')) {
      return <AdminIcon />;
    } else if (role.includes('moderator') || role.includes('manager')) {
      return <SupervisedIcon />;
    } else {
      return <PersonIcon />;
    }
  };

  const getRoleColor = (role: string): "default" | "primary" | "secondary" | "error" | "info" | "success" | "warning" => {
    if (role.includes('owner')) return 'error';
    if (role.includes('admin')) return 'primary';
    if (role.includes('moderator') || role.includes('manager')) return 'warning';
    if (role.includes('member') || role.includes('contributor')) return 'success';
    return 'default';
  };

  const filteredAssignments = roleAssignments.filter(assignment =>
    assignment.userName.toLowerCase().includes(searchTerm.toLowerCase()) ||
    assignment.role.toLowerCase().includes(searchTerm.toLowerCase())
  );

  if (!canManageRoles) {
    return (
      <Paper sx={{ p: 3 }}>
        <Alert severity="warning">
          You don't have permission to manage roles in this context.
        </Alert>
      </Paper>
    );
  }

  return (
    <ResponsiveContainer maxWidth="lg">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.3 }}
      >
        <Stack spacing={3}>
          {/* Header */}
          <Paper sx={{ p: 3 }}>
            <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={2}>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                <ShieldIcon color="primary" fontSize="large" />
                <Box>
                  <Typography variant="h5" fontWeight={600}>
                    Role Management
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Manage user roles and permissions for{' '}
                    {contextType === 'system' ? 'the system' : 
                     contextType === 'organization' ? 'this organization' : 'this project'}
                  </Typography>
                </Box>
              </Box>

              <Button
                variant="contained"
                startIcon={<AddIcon />}
                onClick={() => setAddDialogOpen(true)}
                disabled={loading}
              >
                Add Role
              </Button>
            </Stack>

            {error && (
              <Alert severity="error" sx={{ mt: 2 }} onClose={() => setError(null)}>
                {error}
              </Alert>
            )}
          </Paper>

          {/* Search and Filters */}
          <Paper sx={{ p: 2 }}>
            <TextField
              fullWidth
              placeholder="Search users or roles..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              InputProps={{
                startAdornment: <SearchIcon sx={{ mr: 1, color: 'text.secondary' }} />,
              }}
            />
          </Paper>

          {/* Role Assignments Table */}
          <Paper>
            <TableContainer>
              <Table>
                <TableHead>
                  <TableRow>
                    <TableCell>User</TableCell>
                    <TableCell>Role</TableCell>
                    <TableCell>Assigned</TableCell>
                    <TableCell>Expires</TableCell>
                    <TableCell align="right">Actions</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  <AnimatePresence>
                    {filteredAssignments.map((assignment) => (
                      <motion.tr
                        key={assignment.id}
                        component={TableRow}
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        transition={{ duration: 0.2 }}
                      >
                        <TableCell>
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                            <Avatar
                              sx={{ width: 32, height: 32, bgcolor: 'primary.main' }}
                            >
                              {assignment.userName.charAt(0)}
                            </Avatar>
                            <Typography variant="body2" fontWeight={500}>
                              {assignment.userName}
                            </Typography>
                          </Box>
                        </TableCell>
                        
                        <TableCell>
                          <Chip
                            icon={getRoleIcon(assignment.role)}
                            label={assignment.role}
                            color={getRoleColor(assignment.role)}
                            variant="outlined"
                            size="small"
                          />
                        </TableCell>
                        
                        <TableCell>
                          <Typography variant="body2" color="text.secondary">
                            {new Date(assignment.assignedAt).toLocaleDateString()}
                          </Typography>
                        </TableCell>
                        
                        <TableCell>
                          <Typography variant="body2" color="text.secondary">
                            {assignment.expiresAt 
                              ? new Date(assignment.expiresAt).toLocaleDateString()
                              : 'Never'
                            }
                          </Typography>
                        </TableCell>
                        
                        <TableCell align="right">
                          <Stack direction="row" spacing={1}>
                            <Tooltip title="Edit role">
                              <IconButton
                                size="small"
                                onClick={() => setEditingAssignment(assignment)}
                              >
                                <EditIcon />
                              </IconButton>
                            </Tooltip>
                            
                            <Tooltip title="Remove role">
                              <IconButton
                                size="small"
                                color="error"
                                onClick={() => handleRemoveRole(assignment.id)}
                              >
                                <DeleteIcon />
                              </IconButton>
                            </Tooltip>
                          </Stack>
                        </TableCell>
                      </motion.tr>
                    ))}
                  </AnimatePresence>
                  
                  {filteredAssignments.length === 0 && (
                    <TableRow>
                      <TableCell colSpan={5} align="center" sx={{ py: 4 }}>
                        <Typography variant="body2" color="text.secondary">
                          {searchTerm ? 'No matching role assignments found' : 'No role assignments yet'}
                        </Typography>
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>
            </TableContainer>
            
            {loading && (
              <Box sx={{ display: 'flex', justifyContent: 'center', p: 2 }}>
                <CircularProgress />
              </Box>
            )}
          </Paper>

          {/* Role Hierarchy Info */}
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              Role Hierarchy
            </Typography>
            <Grid container spacing={2}>
              {availableRoles.map((role) => (
                <Grid item xs={12} sm={6} md={4} key={role}>
                  <Card variant="outlined">
                    <CardContent sx={{ textAlign: 'center', py: 2 }}>
                      {getRoleIcon(role)}
                      <Typography variant="body2" fontWeight={500} sx={{ mt: 1 }}>
                        {role.charAt(0).toUpperCase() + role.slice(1)}
                      </Typography>
                      <Chip
                        size="small"
                        label={getRoleColor(role)}
                        color={getRoleColor(role)}
                        sx={{ mt: 0.5 }}
                      />
                    </CardContent>
                  </Card>
                </Grid>
              ))}
            </Grid>
          </Paper>
        </Stack>
      </motion.div>

      {/* Add Role Dialog */}
      <Dialog
        open={addDialogOpen}
        onClose={() => setAddDialogOpen(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>Add Role Assignment</DialogTitle>
        <DialogContent>
          <Stack spacing={3} sx={{ mt: 1 }}>
            <Autocomplete
              options={availableUsers}
              getOptionLabel={(option) => `${option.name} (${option.fourWordAddress})`}
              value={availableUsers.find(u => u.id === newAssignment.userId) || null}
              onChange={(_, value) => setNewAssignment(prev => ({ 
                ...prev, 
                userId: value?.id || '' 
              }))}
              renderInput={(params) => (
                <TextField {...params} label="Select User" />
              )}
            />

            <FormControl fullWidth>
              <InputLabel>Role</InputLabel>
              <Select
                value={newAssignment.role}
                onChange={(e) => setNewAssignment(prev => ({ 
                  ...prev, 
                  role: e.target.value 
                }))}
                label="Role"
              >
                {availableRoles.map((role) => (
                  <MenuItem key={role} value={role}>
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                      {getRoleIcon(role)}
                      {role.charAt(0).toUpperCase() + role.slice(1)}
                    </Box>
                  </MenuItem>
                ))}
              </Select>
            </FormControl>

            <TextField
              fullWidth
              label="Expiration Date (Optional)"
              type="date"
              value={newAssignment.expiresAt}
              onChange={(e) => setNewAssignment(prev => ({ 
                ...prev, 
                expiresAt: e.target.value 
              }))}
              InputLabelProps={{
                shrink: true,
              }}
              helperText="Leave empty for permanent role assignment"
            />
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setAddDialogOpen(false)}>Cancel</Button>
          <Button
            variant="contained"
            onClick={handleAddRole}
            disabled={loading || !newAssignment.userId || !newAssignment.role}
          >
            Add Role
          </Button>
        </DialogActions>
      </Dialog>
    </ResponsiveContainer>
  );
};

export default RoleManager;