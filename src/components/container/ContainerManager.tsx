// Copyright (c) 2025 Saorsa Labs Limited
//
// Container management component

import React, { useState, useEffect } from 'react';
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
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Alert,
  Snackbar,
  Chip,
  LinearProgress,
  Tabs,
  Tab,
  Paper,
  Grid,
  Divider,
} from '@mui/material';
import {
  CloudUpload as UploadIcon,
  CloudDownload as DownloadIcon,
  Add as AddIcon,
  Delete as DeleteIcon,
  Visibility as ViewIcon,
  Refresh as RefreshIcon,
  Storage as StorageIcon,
  PostAdd as PostIcon,
  Info as InfoIcon,
} from '@mui/icons-material';
import { ContainerService, ObjectInfo, TipInfo, ContainerStats } from '../../services/ContainerService';

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
      id={`container-tabpanel-${index}`}
      aria-labelledby={`container-tab-${index}`}
      {...other}
    >
      {value === index && <Box sx={{ p: 3 }}>{children}</Box>}
    </div>
  );
}

export const ContainerManager: React.FC = () => {
  const [tabValue, setTabValue] = useState(0);
  const [initialized, setInitialized] = useState(false);
  const [loading, setLoading] = useState(false);
  const [objects, setObjects] = useState<ObjectInfo[]>([]);
  const [stats, setStats] = useState<ContainerStats | null>(null);
  const [currentTip, setCurrentTip] = useState<TipInfo | null>(null);
  
  // Dialog states
  const [uploadDialogOpen, setUploadDialogOpen] = useState(false);
  const [viewDialogOpen, setViewDialogOpen] = useState(false);
  const [postDialogOpen, setPostDialogOpen] = useState(false);
  const [selectedObject, setSelectedObject] = useState<ObjectInfo | null>(null);
  const [objectContent, setObjectContent] = useState<string>('');
  
  // Form states
  const [uploadText, setUploadText] = useState('');
  const [postContent, setPostContent] = useState('');
  
  // Notification state
  const [notification, setNotification] = useState<{
    open: boolean;
    message: string;
    severity: 'success' | 'error' | 'info' | 'warning';
  }>({
    open: false,
    message: '',
    severity: 'info',
  });

  // Initialize container on mount
  useEffect(() => {
    initializeContainer();
  }, []);

  const showNotification = (message: string, severity: 'success' | 'error' | 'info' | 'warning' = 'info') => {
    setNotification({ open: true, message, severity });
  };

  const hideNotification = () => {
    setNotification(prev => ({ ...prev, open: false }));
  };

  const initializeContainer = async () => {
    setLoading(true);
    try {
      const success = await ContainerService.init();
      if (success) {
        setInitialized(true);
        showNotification('Container initialized successfully', 'success');
        await loadData();
      } else {
        showNotification('Failed to initialize container', 'error');
      }
    } catch (error) {
      console.error('Container initialization error:', error);
      showNotification(`Container initialization failed: ${error}`, 'error');
    } finally {
      setLoading(false);
    }
  };

  const loadData = async () => {
    try {
      const [objectsData, statsData, tipData] = await Promise.all([
        ContainerService.listObjects(),
        ContainerService.getStats(),
        ContainerService.getCurrentTip(),
      ]);
      
      setObjects(objectsData);
      setStats(statsData);
      setCurrentTip(tipData);
    } catch (error) {
      console.error('Failed to load container data:', error);
      showNotification(`Failed to load data: ${error}`, 'error');
    }
  };

  const handleUploadObject = async () => {
    if (!uploadText.trim()) {
      showNotification('Please enter some content to upload', 'warning');
      return;
    }

    setLoading(true);
    try {
      const objectInfo = await ContainerService.putText(uploadText);
      showNotification(`Object stored successfully: ${objectInfo.oid_hex}`, 'success');
      setUploadText('');
      setUploadDialogOpen(false);
      await loadData();
    } catch (error) {
      console.error('Upload error:', error);
      showNotification(`Upload failed: ${error}`, 'error');
    } finally {
      setLoading(false);
    }
  };

  const handleViewObject = async (object: ObjectInfo) => {
    setLoading(true);
    try {
      const content = await ContainerService.getText(object.oid_hex);
      setObjectContent(content);
      setSelectedObject(object);
      setViewDialogOpen(true);
    } catch (error) {
      console.error('View object error:', error);
      showNotification(`Failed to view object: ${error}`, 'error');
    } finally {
      setLoading(false);
    }
  };

  const handleCreatePost = async () => {
    if (!postContent.trim()) {
      showNotification('Please enter post content', 'warning');
      return;
    }

    setLoading(true);
    try {
      const tipRoot = await ContainerService.createPost(postContent);
      showNotification(`Post created successfully. New tip: ${tipRoot}`, 'success');
      setPostContent('');
      setPostDialogOpen(false);
      await loadData();
    } catch (error) {
      console.error('Create post error:', error);
      showNotification(`Failed to create post: ${error}`, 'error');
    } finally {
      setLoading(false);
    }
  };

  const handleTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setTabValue(newValue);
  };

  return (
    <Box sx={{ width: '100%', p: 2 }}>
      <Typography variant="h4" gutterBottom>
        Container Management
      </Typography>

      {loading && <LinearProgress sx={{ mb: 2 }} />}

      {/* Status Cards */}
      <Grid container spacing={2} sx={{ mb: 3 }}>
        <Grid item xs={12} md={4}>
          <Card>
            <CardContent>
              <Box display="flex" alignItems="center">
                <StorageIcon sx={{ mr: 2, color: 'primary.main' }} />
                <Box>
                  <Typography variant="h6">Status</Typography>
                  <Chip 
                    label={initialized ? 'Initialized' : 'Not Initialized'} 
                    color={initialized ? 'success' : 'warning'}
                    size="small"
                  />
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} md={4}>
          <Card>
            <CardContent>
              <Typography variant="h6">Objects</Typography>
              <Typography variant="h4">{objects.length}</Typography>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} md={4}>
          <Card>
            <CardContent>
              <Typography variant="h6">Operations</Typography>
              <Typography variant="h4">{currentTip?.count || 0}</Typography>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Action Buttons */}
      <Box sx={{ mb: 2 }}>
        <Button
          variant="contained"
          startIcon={<UploadIcon />}
          onClick={() => setUploadDialogOpen(true)}
          disabled={!initialized}
          sx={{ mr: 1 }}
        >
          Upload Object
        </Button>
        <Button
          variant="contained"
          startIcon={<PostIcon />}
          onClick={() => setPostDialogOpen(true)}
          disabled={!initialized}
          sx={{ mr: 1 }}
        >
          Create Post
        </Button>
        <Button
          variant="outlined"
          startIcon={<RefreshIcon />}
          onClick={loadData}
          disabled={!initialized}
        >
          Refresh
        </Button>
      </Box>

      {/* Tabs */}
      <Paper sx={{ width: '100%' }}>
        <Tabs
          value={tabValue}
          onChange={handleTabChange}
          aria-label="Container management tabs"
        >
          <Tab label="Objects" />
          <Tab label="Statistics" />
          <Tab label="Current Tip" />
        </Tabs>

        {/* Objects Tab */}
        <TabPanel value={tabValue} index={0}>
          <Typography variant="h6" gutterBottom>
            Stored Objects
          </Typography>
          <List>
            {objects.length === 0 ? (
              <ListItem>
                <ListItemText primary="No objects stored" />
              </ListItem>
            ) : (
              objects.map((object) => (
                <ListItem
                  key={object.oid_hex}
                  divider
                  secondaryAction={
                    <IconButton
                      edge="end"
                      onClick={() => handleViewObject(object)}
                      title="View Object"
                    >
                      <ViewIcon />
                    </IconButton>
                  }
                >
                  <ListItemText
                    primary={object.oid_hex}
                    secondary={
                      <>
                        <Typography variant="body2" color="textSecondary">
                          Size: {ContainerService.formatFileSize(object.size_bytes)}
                        </Typography>
                        <Typography variant="body2" color="textSecondary">
                          Created: {ContainerService.formatTimestamp(object.timestamp)}
                        </Typography>
                      </>
                    }
                  />
                </ListItem>
              ))
            )}
          </List>
        </TabPanel>

        {/* Statistics Tab */}
        <TabPanel value={tabValue} index={1}>
          <Typography variant="h6" gutterBottom>
            Container Statistics
          </Typography>
          {stats ? (
            <Box>
              <Typography>Initialized: {stats.initialized ? 'Yes' : 'No'}</Typography>
              <Typography>Current Tip Root: {stats.current_tip.root_hex}</Typography>
              <Typography>Operation Count: {stats.current_tip.count}</Typography>
              <Typography>Last Updated: {ContainerService.formatTimestamp(stats.timestamp)}</Typography>
            </Box>
          ) : (
            <Typography>No statistics available</Typography>
          )}
        </TabPanel>

        {/* Current Tip Tab */}
        <TabPanel value={tabValue} index={2}>
          <Typography variant="h6" gutterBottom>
            Current CRDT Tip
          </Typography>
          {currentTip ? (
            <Box>
              <Typography>Root: {currentTip.root_hex}</Typography>
              <Typography>Count: {currentTip.count}</Typography>
              <Typography>Signature: {currentTip.signature_hex}</Typography>
              <Typography>Timestamp: {ContainerService.formatTimestamp(currentTip.timestamp)}</Typography>
            </Box>
          ) : (
            <Typography>No tip information available</Typography>
          )}
        </TabPanel>
      </Paper>

      {/* Upload Dialog */}
      <Dialog open={uploadDialogOpen} onClose={() => setUploadDialogOpen(false)} maxWidth="md" fullWidth>
        <DialogTitle>Upload Object</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Content"
            multiline
            rows={6}
            fullWidth
            variant="outlined"
            value={uploadText}
            onChange={(e) => setUploadText(e.target.value)}
            placeholder="Enter content to store in the container..."
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setUploadDialogOpen(false)}>Cancel</Button>
          <Button onClick={handleUploadObject} variant="contained" disabled={loading}>
            Upload
          </Button>
        </DialogActions>
      </Dialog>

      {/* View Object Dialog */}
      <Dialog open={viewDialogOpen} onClose={() => setViewDialogOpen(false)} maxWidth="md" fullWidth>
        <DialogTitle>Object Content</DialogTitle>
        <DialogContent>
          {selectedObject && (
            <Box>
              <Typography variant="body2" color="textSecondary" gutterBottom>
                OID: {selectedObject.oid_hex}
              </Typography>
              <Typography variant="body2" color="textSecondary" gutterBottom>
                Size: {ContainerService.formatFileSize(selectedObject.size_bytes)}
              </Typography>
              <Divider sx={{ my: 2 }} />
              <TextField
                multiline
                rows={10}
                fullWidth
                variant="outlined"
                value={objectContent}
                InputProps={{ readOnly: true }}
              />
            </Box>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setViewDialogOpen(false)}>Close</Button>
        </DialogActions>
      </Dialog>

      {/* Create Post Dialog */}
      <Dialog open={postDialogOpen} onClose={() => setPostDialogOpen(false)} maxWidth="md" fullWidth>
        <DialogTitle>Create Post</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Post Content (Markdown)"
            multiline
            rows={6}
            fullWidth
            variant="outlined"
            value={postContent}
            onChange={(e) => setPostContent(e.target.value)}
            placeholder="Enter your post content in Markdown format..."
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setPostDialogOpen(false)}>Cancel</Button>
          <Button onClick={handleCreatePost} variant="contained" disabled={loading}>
            Create Post
          </Button>
        </DialogActions>
      </Dialog>

      {/* Notification */}
      <Snackbar
        open={notification.open}
        autoHideDuration={6000}
        onClose={hideNotification}
      >
        <Alert onClose={hideNotification} severity={notification.severity}>
          {notification.message}
        </Alert>
      </Snackbar>
    </Box>
  );
};