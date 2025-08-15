import React, { useState, useEffect, useRef } from 'react';
import {
  Box,
  Paper,
  Typography,
  IconButton,
  Avatar,
  Stack,
  Chip,
  Button,
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  Divider,
  Card,
  CardContent,
  CardActions,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Grid,
  List,
  ListItem,
  ListItemButton,
  ListItemAvatar,
  Badge,
  Tooltip,
  InputAdornment,
  Tabs,
  Tab,
  Alert,
  LinearProgress,
  Breadcrumbs,
  Link,
  Fab,
  Collapse,
  Table,
  TableHead,
  TableRow,
  TableCell,
  TableBody,
  TableContainer,
  Checkbox,
  FormControlLabel,
} from '@mui/material';
import {
  Folder as FolderIcon,
  InsertDriveFile as FileIcon,
  Image as ImageIcon,
  VideoFile as VideoIcon,
  AudioFile as AudioIcon,
  PictureAsPdf as PdfIcon,
  Description as DocIcon,
  Archive as ArchiveIcon,
  Code as CodeIcon,
  Search as SearchIcon,
  Upload as UploadIcon,
  Download as DownloadIcon,
  Share as ShareIcon,
  Delete as DeleteIcon,
  Edit as EditIcon,
  MoreVert as MoreVertIcon,
  ViewList as ListViewIcon,
  ViewModule as GridViewIcon,
  Sort as SortIcon,
  FilterList as FilterIcon,
  CloudUpload as CloudUploadIcon,
  CreateNewFolder as CreateFolderIcon,
  Star as StarIcon,
  StarBorder as StarBorderIcon,
  History as HistoryIcon,
  Info as InfoIcon,
  Security as SecurityIcon,
  Group as GroupIcon,
  Person as PersonIcon,
  Visibility as VisibilityIcon,
  VisibilityOff as VisibilityOffIcon,
  NavigateNext as NavigateNextIcon,
  ArrowBack as ArrowBackIcon,
  Refresh as RefreshIcon,
  Link as LinkIcon,
  ContentCopy as CopyIcon,
  Launch as LaunchIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { Member } from '../../types/organization';

interface SharedResource {
  id: string;
  name: string;
  type: 'file' | 'folder';
  size: number;
  mime_type?: string;
  created_at: string;
  updated_at: string;
  created_by: Member;
  path: string;
  parent_id?: string;
  is_starred: boolean;
  is_shared: boolean;
  permissions: ResourcePermission[];
  version: number;
  download_count: number;
  tags: string[];
  description?: string;
  thumbnail?: string;
  checksum?: string;
}

interface ResourcePermission {
  user_id: string;
  permission: 'read' | 'write' | 'admin';
  granted_by: string;
  granted_at: string;
}

interface ResourceVersion {
  version: number;
  created_at: string;
  created_by: Member;
  size: number;
  checksum: string;
  comment?: string;
}

interface SharedResourcesViewProps {
  entityId: string;
  entityType: 'organization' | 'project' | 'group';
  entityName: string;
  currentUser: Member;
  onUploadFiles: (files: File[], targetPath: string) => Promise<void>;
  onCreateFolder: (name: string, parentPath: string) => Promise<void>;
  onDeleteResource: (resourceId: string) => Promise<void>;
  onShareResource: (resourceId: string, permissions: ResourcePermission[]) => Promise<void>;
}

export const SharedResourcesView: React.FC<SharedResourcesViewProps> = ({
  entityId,
  entityType,
  entityName,
  currentUser,
  onUploadFiles,
  onCreateFolder,
  onDeleteResource,
  onShareResource,
}) => {
  const [resources, setResources] = useState<SharedResource[]>([]);
  const [currentPath, setCurrentPath] = useState<string>('/');
  const [selectedResources, setSelectedResources] = useState<Set<string>>(new Set());
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [sortBy, setSortBy] = useState<'name' | 'date' | 'size' | 'type'>('name');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc');
  const [searchQuery, setSearchQuery] = useState('');
  const [filterType, setFilterType] = useState<string>('all');
  const [tabValue, setTabValue] = useState(0);
  
  // Dialog states
  const [showUploadDialog, setShowUploadDialog] = useState(false);
  const [showCreateFolderDialog, setShowCreateFolderDialog] = useState(false);
  const [showShareDialog, setShowShareDialog] = useState(false);
  const [showDetailsDialog, setShowDetailsDialog] = useState(false);
  const [showVersionDialog, setShowVersionDialog] = useState(false);
  const [selectedResource, setSelectedResource] = useState<SharedResource | null>(null);
  
  // Upload states
  const [dragOver, setDragOver] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0);
  
  // Menu states
  const [menuAnchor, setMenuAnchor] = useState<null | HTMLElement>(null);
  const [contextMenuResource, setContextMenuResource] = useState<SharedResource | null>(null);
  
  const fileInputRef = useRef<HTMLInputElement>(null);
  const dropZoneRef = useRef<HTMLDivElement>(null);

  // Mock data - replace with actual API calls
  useEffect(() => {
    loadResources();
  }, [entityId, currentPath]);

  const loadResources = async () => {
    // Mock resources for demo
    const mockResources: SharedResource[] = [
      {
        id: '1',
        name: 'Project Documentation',
        type: 'folder',
        size: 0,
        created_at: new Date(Date.now() - 86400000 * 7).toISOString(),
        updated_at: new Date(Date.now() - 86400000 * 2).toISOString(),
        created_by: currentUser,
        path: '/Project Documentation',
        is_starred: true,
        is_shared: true,
        permissions: [],
        version: 1,
        download_count: 0,
        tags: ['documentation', 'important'],
      },
      {
        id: '2',
        name: 'Design Assets.zip',
        type: 'file',
        size: 25600000, // 25.6 MB
        mime_type: 'application/zip',
        created_at: new Date(Date.now() - 86400000 * 3).toISOString(),
        updated_at: new Date(Date.now() - 86400000 * 1).toISOString(),
        created_by: { ...currentUser, display_name: 'John Designer' },
        path: '/Design Assets.zip',
        is_starred: false,
        is_shared: true,
        permissions: [],
        version: 2,
        download_count: 15,
        tags: ['design', 'assets'],
        description: 'Complete design system and UI assets for the project',
      },
      {
        id: '3',
        name: 'Meeting Recording - Sprint Review.mp4',
        type: 'file',
        size: 157286400, // 150 MB
        mime_type: 'video/mp4',
        created_at: new Date(Date.now() - 86400000 * 1).toISOString(),
        updated_at: new Date(Date.now() - 86400000 * 1).toISOString(),
        created_by: { ...currentUser, display_name: 'Sarah Manager' },
        path: '/Meeting Recording - Sprint Review.mp4',
        is_starred: true,
        is_shared: false,
        permissions: [],
        version: 1,
        download_count: 8,
        tags: ['meeting', 'sprint', 'video'],
      },
      {
        id: '4',
        name: 'API Specification.pdf',
        type: 'file',
        size: 2048000, // 2 MB
        mime_type: 'application/pdf',
        created_at: new Date(Date.now() - 86400000 * 5).toISOString(),
        updated_at: new Date(Date.now() - 86400000 * 3).toISOString(),
        created_by: { ...currentUser, display_name: 'Mike Developer' },
        path: '/API Specification.pdf',
        is_starred: false,
        is_shared: true,
        permissions: [],
        version: 3,
        download_count: 42,
        tags: ['api', 'documentation', 'technical'],
        description: 'Complete API documentation with examples and authentication details',
      },
    ];
    
    setResources(mockResources);
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const getFileIcon = (resource: SharedResource) => {
    if (resource.type === 'folder') return <FolderIcon color="primary" />;
    
    const mimeType = resource.mime_type || '';
    if (mimeType.startsWith('image/')) return <ImageIcon color="info" />;
    if (mimeType.startsWith('video/')) return <VideoIcon color="secondary" />;
    if (mimeType.startsWith('audio/')) return <AudioIcon color="warning" />;
    if (mimeType === 'application/pdf') return <PdfIcon color="error" />;
    if (mimeType.includes('document') || mimeType.includes('text')) return <DocIcon color="info" />;
    if (mimeType.includes('zip') || mimeType.includes('archive')) return <ArchiveIcon color="action" />;
    if (mimeType.includes('code') || resource.name.includes('.js') || resource.name.includes('.ts')) return <CodeIcon color="success" />;
    
    return <FileIcon color="action" />;
  };

  const handleFileUpload = async (files: File[]) => {
    if (files.length === 0) return;
    
    setUploading(true);
    setUploadProgress(0);
    
    try {
      // Simulate upload progress
      for (let i = 0; i <= 100; i += 10) {
        setUploadProgress(i);
        await new Promise(resolve => setTimeout(resolve, 100));
      }
      
      await onUploadFiles(files, currentPath);
      await loadResources();
      setShowUploadDialog(false);
    } catch (error) {
      console.error('Upload failed:', error);
    } finally {
      setUploading(false);
      setUploadProgress(0);
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    const files = Array.from(e.dataTransfer.files);
    if (files.length > 0) {
      handleFileUpload(files);
    }
  };

  const handleResourceClick = (resource: SharedResource) => {
    if (resource.type === 'folder') {
      setCurrentPath(resource.path);
    } else {
      // Open file preview/download
      setSelectedResource(resource);
      setShowDetailsDialog(true);
    }
  };

  const handleContextMenu = (e: React.MouseEvent, resource: SharedResource) => {
    e.preventDefault();
    setMenuAnchor(e.currentTarget as HTMLElement);
    setContextMenuResource(resource);
  };

  const filteredAndSortedResources = resources
    .filter(resource => {
      if (searchQuery && !resource.name.toLowerCase().includes(searchQuery.toLowerCase())) {
        return false;
      }
      if (filterType !== 'all') {
        if (filterType === 'folders' && resource.type !== 'folder') return false;
        if (filterType === 'files' && resource.type !== 'file') return false;
        if (filterType === 'starred' && !resource.is_starred) return false;
        if (filterType === 'shared' && !resource.is_shared) return false;
      }
      return true;
    })
    .sort((a, b) => {
      let comparison = 0;
      switch (sortBy) {
        case 'name':
          comparison = a.name.localeCompare(b.name);
          break;
        case 'date':
          comparison = new Date(a.updated_at).getTime() - new Date(b.updated_at).getTime();
          break;
        case 'size':
          comparison = a.size - b.size;
          break;
        case 'type':
          comparison = a.type.localeCompare(b.type);
          break;
      }
      return sortOrder === 'asc' ? comparison : -comparison;
    });

  const renderBreadcrumbs = () => {
    const pathParts = currentPath.split('/').filter(part => part);
    const breadcrumbs = [{ name: 'Root', path: '/' }];
    
    let currentBreadcrumbPath = '';
    pathParts.forEach(part => {
      currentBreadcrumbPath += '/' + part;
      breadcrumbs.push({ name: part, path: currentBreadcrumbPath });
    });

    return (
      <Breadcrumbs separator={<NavigateNextIcon fontSize="small" />} sx={{ mb: 2 }}>
        {breadcrumbs.map((crumb, index) => {
          const isLast = index === breadcrumbs.length - 1;
          return isLast ? (
            <Typography key={crumb.path} color="text.primary">{crumb.name}</Typography>
          ) : (
            <Link
              key={crumb.path}
              color="inherit"
              href="#"
              onClick={() => setCurrentPath(crumb.path)}
              sx={{ cursor: 'pointer' }}
            >
              {crumb.name}
            </Link>
          );
        })}
      </Breadcrumbs>
    );
  };

  const renderGridView = () => (
    <Grid container spacing={2}>
      {filteredAndSortedResources.map((resource) => (
        <Grid item xs={12} sm={6} md={4} lg={3} key={resource.id}>
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.3 }}
          >
            <Card
              sx={{
                cursor: 'pointer',
                transition: 'all 0.2s ease',
                '&:hover': {
                  transform: 'translateY(-4px)',
                  boxShadow: 3,
                },
                border: selectedResources.has(resource.id) ? 2 : 0,
                borderColor: 'primary.main',
              }}
              onClick={() => handleResourceClick(resource)}
              onContextMenu={(e) => handleContextMenu(e, resource)}
            >
              <CardContent sx={{ textAlign: 'center', py: 3 }}>
                <Box sx={{ position: 'relative', mb: 2 }}>
                  <Box sx={{ fontSize: '3rem', color: 'primary.main' }}>
                    {getFileIcon(resource)}
                  </Box>
                  {resource.is_starred && (
                    <StarIcon
                      sx={{
                        position: 'absolute',
                        top: -8,
                        right: -8,
                        color: 'warning.main',
                        fontSize: '1.2rem',
                      }}
                    />
                  )}
                  {resource.is_shared && (
                    <ShareIcon
                      sx={{
                        position: 'absolute',
                        bottom: -8,
                        right: -8,
                        color: 'info.main',
                        fontSize: '1rem',
                      }}
                    />
                  )}
                </Box>
                
                <Typography 
                  variant="subtitle2" 
                  noWrap 
                  sx={{ fontWeight: 600, mb: 1 }}
                  title={resource.name}
                >
                  {resource.name}
                </Typography>
                
                <Stack direction="row" spacing={1} justifyContent="center" sx={{ mb: 1 }}>
                  {resource.type === 'file' && (
                    <Chip 
                      label={formatFileSize(resource.size)} 
                      size="small" 
                      variant="outlined" 
                    />
                  )}
                  {resource.version > 1 && (
                    <Chip 
                      label={`v${resource.version}`} 
                      size="small" 
                      color="info"
                    />
                  )}
                </Stack>
                
                <Typography variant="caption" color="text.secondary">
                  {new Date(resource.updated_at).toLocaleDateString()}
                </Typography>
              </CardContent>
            </Card>
          </motion.div>
        </Grid>
      ))}
    </Grid>
  );

  const renderListView = () => (
    <TableContainer component={Paper}>
      <Table>
        <TableHead>
          <TableRow>
            <TableCell padding="checkbox">
              <Checkbox
                indeterminate={selectedResources.size > 0 && selectedResources.size < resources.length}
                checked={resources.length > 0 && selectedResources.size === resources.length}
                onChange={(e) => {
                  if (e.target.checked) {
                    setSelectedResources(new Set(resources.map(r => r.id)));
                  } else {
                    setSelectedResources(new Set());
                  }
                }}
              />
            </TableCell>
            <TableCell>Name</TableCell>
            <TableCell>Type</TableCell>
            <TableCell>Size</TableCell>
            <TableCell>Modified</TableCell>
            <TableCell>Created By</TableCell>
            <TableCell>Actions</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {filteredAndSortedResources.map((resource) => (
            <TableRow
              key={resource.id}
              hover
              selected={selectedResources.has(resource.id)}
              onClick={() => handleResourceClick(resource)}
              sx={{ cursor: 'pointer' }}
            >
              <TableCell padding="checkbox">
                <Checkbox
                  checked={selectedResources.has(resource.id)}
                  onClick={(e) => e.stopPropagation()}
                  onChange={(e) => {
                    const newSelected = new Set(selectedResources);
                    if (e.target.checked) {
                      newSelected.add(resource.id);
                    } else {
                      newSelected.delete(resource.id);
                    }
                    setSelectedResources(newSelected);
                  }}
                />
              </TableCell>
              <TableCell>
                <Stack direction="row" alignItems="center" spacing={2}>
                  {getFileIcon(resource)}
                  <Box>
                    <Typography variant="body2" sx={{ fontWeight: 500 }}>
                      {resource.name}
                    </Typography>
                    {resource.description && (
                      <Typography variant="caption" color="text.secondary" noWrap>
                        {resource.description}
                      </Typography>
                    )}
                  </Box>
                  {resource.is_starred && <StarIcon color="warning" sx={{ fontSize: '1rem' }} />}
                  {resource.is_shared && <ShareIcon color="info" sx={{ fontSize: '1rem' }} />}
                </Stack>
              </TableCell>
              <TableCell>
                <Chip 
                  label={resource.type} 
                  size="small" 
                  color={resource.type === 'folder' ? 'primary' : 'default'}
                />
              </TableCell>
              <TableCell>
                {resource.type === 'file' ? formatFileSize(resource.size) : '—'}
              </TableCell>
              <TableCell>
                <Typography variant="body2">
                  {new Date(resource.updated_at).toLocaleDateString()}
                </Typography>
              </TableCell>
              <TableCell>
                <Stack direction="row" alignItems="center" spacing={1}>
                  <Avatar sx={{ width: 24, height: 24, fontSize: '0.75rem' }}>
                    {resource.created_by.display_name[0]}
                  </Avatar>
                  <Typography variant="body2">
                    {resource.created_by.display_name}
                  </Typography>
                </Stack>
              </TableCell>
              <TableCell>
                <IconButton 
                  size="small"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleContextMenu(e, resource);
                  }}
                >
                  <MoreVertIcon />
                </IconButton>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <Paper elevation={1} sx={{ p: 2, mb: 2 }}>
        <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ mb: 2 }}>
          <Typography variant="h5" sx={{ fontWeight: 600 }}>
            {entityName} - Shared Resources
          </Typography>
          
          <Stack direction="row" spacing={1}>
            <Button
              variant="outlined"
              startIcon={<CreateFolderIcon />}
              onClick={() => setShowCreateFolderDialog(true)}
            >
              New Folder
            </Button>
            <Button
              variant="contained"
              startIcon={<CloudUploadIcon />}
              onClick={() => setShowUploadDialog(true)}
            >
              Upload Files
            </Button>
          </Stack>
        </Stack>

        {renderBreadcrumbs()}

        {/* Search and Filters */}
        <Stack direction="row" spacing={2} alignItems="center">
          <TextField
            size="small"
            placeholder="Search resources..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            InputProps={{
              startAdornment: (
                <InputAdornment position="start">
                  <SearchIcon />
                </InputAdornment>
              ),
            }}
            sx={{ flexGrow: 1 }}
          />
          
          <Button
            variant="outlined"
            startIcon={<FilterIcon />}
            onClick={() => {}} // Filter menu implementation
          >
            Filter
          </Button>
          
          <Button
            variant="outlined"
            startIcon={<SortIcon />}
            onClick={() => {}} // Sort menu implementation
          >
            Sort
          </Button>
          
          <Tooltip title={viewMode === 'grid' ? 'Switch to List View' : 'Switch to Grid View'}>
            <IconButton onClick={() => setViewMode(viewMode === 'grid' ? 'list' : 'grid')}>
              {viewMode === 'grid' ? <ListViewIcon /> : <GridViewIcon />}
            </IconButton>
          </Tooltip>
        </Stack>
      </Paper>

      {/* Tabs */}
      <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 2 }}>
        <Tabs value={tabValue} onChange={(_, newValue) => setTabValue(newValue)}>
          <Tab label="All Files" />
          <Tab label="Recent" />
          <Tab label="Starred" />
          <Tab label="Shared" />
          <Tab label="Trash" />
        </Tabs>
      </Box>

      {/* Drop Zone */}
      <Box
        ref={dropZoneRef}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        sx={{
          flexGrow: 1,
          overflow: 'auto',
          border: dragOver ? '2px dashed' : '1px solid transparent',
          borderColor: dragOver ? 'primary.main' : 'transparent',
          borderRadius: 2,
          backgroundColor: dragOver ? 'action.hover' : 'transparent',
          transition: 'all 0.2s ease',
          position: 'relative',
        }}
      >
        {dragOver && (
          <Box
            sx={{
              position: 'absolute',
              top: 0,
              left: 0,
              right: 0,
              bottom: 0,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              backgroundColor: 'rgba(0, 0, 0, 0.5)',
              zIndex: 10,
              borderRadius: 2,
            }}
          >
            <Paper elevation={4} sx={{ p: 4, textAlign: 'center' }}>
              <CloudUploadIcon sx={{ fontSize: '4rem', color: 'primary.main', mb: 2 }} />
              <Typography variant="h5" gutterBottom>
                Drop files here to upload
              </Typography>
              <Typography variant="body2" color="text.secondary">
                Files will be uploaded to {currentPath}
              </Typography>
            </Paper>
          </Box>
        )}

        {/* Content */}
        {uploading && (
          <Box sx={{ mb: 2 }}>
            <Alert severity="info" sx={{ mb: 1 }}>
              Uploading files... {uploadProgress}%
            </Alert>
            <LinearProgress variant="determinate" value={uploadProgress} />
          </Box>
        )}

        {filteredAndSortedResources.length === 0 ? (
          <Box sx={{ textAlign: 'center', py: 8 }}>
            <FolderIcon sx={{ fontSize: '4rem', color: 'text.secondary', mb: 2 }} />
            <Typography variant="h6" color="text.secondary" gutterBottom>
              No files in this location
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
              Upload files or create folders to get started
            </Typography>
            <Stack direction="row" spacing={2} justifyContent="center">
              <Button
                variant="outlined"
                startIcon={<CreateFolderIcon />}
                onClick={() => setShowCreateFolderDialog(true)}
              >
                Create Folder
              </Button>
              <Button
                variant="contained"
                startIcon={<CloudUploadIcon />}
                onClick={() => setShowUploadDialog(true)}
              >
                Upload Files
              </Button>
            </Stack>
          </Box>
        ) : viewMode === 'grid' ? renderGridView() : renderListView()}
      </Box>

      {/* Floating Action Button */}
      <Fab
        color="primary"
        sx={{ position: 'fixed', bottom: 24, right: 24 }}
        onClick={() => setShowUploadDialog(true)}
      >
        <UploadIcon />
      </Fab>

      {/* Context Menu */}
      <Menu
        anchorEl={menuAnchor}
        open={Boolean(menuAnchor)}
        onClose={() => setMenuAnchor(null)}
      >
        <MenuItem onClick={() => {
          if (contextMenuResource) {
            setSelectedResource(contextMenuResource);
            setShowDetailsDialog(true);
          }
          setMenuAnchor(null);
        }}>
          <ListItemIcon><InfoIcon /></ListItemIcon>
          <ListItemText>Details</ListItemText>
        </MenuItem>
        <MenuItem onClick={() => setMenuAnchor(null)}>
          <ListItemIcon><DownloadIcon /></ListItemIcon>
          <ListItemText>Download</ListItemText>
        </MenuItem>
        <MenuItem onClick={() => {
          if (contextMenuResource) {
            setSelectedResource(contextMenuResource);
            setShowShareDialog(true);
          }
          setMenuAnchor(null);
        }}>
          <ListItemIcon><ShareIcon /></ListItemIcon>
          <ListItemText>Share</ListItemText>
        </MenuItem>
        <MenuItem onClick={() => setMenuAnchor(null)}>
          <ListItemIcon><EditIcon /></ListItemIcon>
          <ListItemText>Rename</ListItemText>
        </MenuItem>
        <Divider />
        <MenuItem 
          onClick={() => setMenuAnchor(null)}
          sx={{ color: 'error.main' }}
        >
          <ListItemIcon><DeleteIcon color="error" /></ListItemIcon>
          <ListItemText>Delete</ListItemText>
        </MenuItem>
      </Menu>

      {/* Upload Dialog */}
      <Dialog
        open={showUploadDialog}
        onClose={() => setShowUploadDialog(false)}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle>Upload Files</DialogTitle>
        <DialogContent>
          <input
            type="file"
            multiple
            onChange={(e) => {
              if (e.target.files) {
                handleFileUpload(Array.from(e.target.files));
              }
            }}
            style={{ display: 'none' }}
            ref={fileInputRef}
          />
          <Box
            sx={{
              border: '2px dashed',
              borderColor: 'divider',
              borderRadius: 2,
              p: 4,
              textAlign: 'center',
              cursor: 'pointer',
            }}
            onClick={() => fileInputRef.current?.click()}
          >
            <CloudUploadIcon sx={{ fontSize: '3rem', color: 'text.secondary', mb: 2 }} />
            <Typography variant="h6" gutterBottom>
              Drag files here or click to browse
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Upload to: {currentPath}
            </Typography>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShowUploadDialog(false)}>Cancel</Button>
        </DialogActions>
      </Dialog>

      {/* Create Folder Dialog */}
      <Dialog
        open={showCreateFolderDialog}
        onClose={() => setShowCreateFolderDialog(false)}
      >
        <DialogTitle>Create New Folder</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Folder Name"
            fullWidth
            variant="outlined"
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShowCreateFolderDialog(false)}>Cancel</Button>
          <Button variant="contained">Create</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default SharedResourcesView;