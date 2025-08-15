import React, { useState } from 'react';
import {
  Box,
  Paper,
  IconButton,
  Typography,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  ListItemSecondaryAction,
  Breadcrumbs,
  Link,
  Menu,
  MenuItem,
  LinearProgress,
  Chip,
  Button,
  TextField,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Grid,
  Card,
  CardContent,
  CardActions,
  Avatar,
  Tooltip,
  InputAdornment,
} from '@mui/material';
import {
  Folder as FolderIcon,
  InsertDriveFile as FileIcon,
  CloudUpload as UploadIcon,
  CreateNewFolder as NewFolderIcon,
  Delete as DeleteIcon,
  Share as ShareIcon,
  GetApp as DownloadIcon,
  MoreVert as MoreIcon,
  GridView as GridViewIcon,
  ViewList as ListViewIcon,
  Search as SearchIcon,
  History as HistoryIcon,
  CloudDone as SyncedIcon,
  CloudSync as SyncingIcon,
  CloudOff as OfflineIcon,
  Image as ImageIcon,
  VideoLibrary as VideoIcon,
  AudioFile as AudioIcon,
  Description as DocIcon,
  Code as CodeIcon,
  Archive as ZipIcon,
  Star as StarIcon,
  StarBorder as StarBorderIcon,
} from '@mui/icons-material';

interface FileItem {
  id: string;
  name: string;
  type: 'folder' | 'file';
  size?: number;
  modified: Date;
  owner: string;
  shared?: string[];
  syncStatus: 'synced' | 'syncing' | 'offline';
  starred?: boolean;
  fileType?: 'image' | 'video' | 'audio' | 'document' | 'code' | 'archive' | 'other';
  preview?: string;
  version?: number;
}

interface FileManagerProps {
  organizationId?: string;
  folderId?: string;
  onFileSelect?: (file: FileItem) => void;
  onFileUpload?: (files: File[]) => void;
}

export const FileManager: React.FC<FileManagerProps> = ({
  organizationId,
  folderId,
  onFileSelect,
  onFileUpload,
}) => {
  const [viewMode, setViewMode] = useState<'list' | 'grid'>('list');
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [createFolderOpen, setCreateFolderOpen] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const [selectedFile, setSelectedFile] = useState<FileItem | null>(null);

  const [files] = useState<FileItem[]>([
    {
      id: '1',
      name: 'Project Documents',
      type: 'folder',
      modified: new Date(Date.now() - 86400000),
      owner: 'You',
      shared: ['Alice Johnson', 'Bob Chen'],
      syncStatus: 'synced',
      starred: true,
    },
    {
      id: '2',
      name: 'Architecture.pdf',
      type: 'file',
      size: 2457600,
      modified: new Date(Date.now() - 3600000),
      owner: 'Alice Johnson',
      syncStatus: 'synced',
      fileType: 'document',
      version: 3,
    },
    {
      id: '3',
      name: 'Demo Video.mp4',
      type: 'file',
      size: 45678900,
      modified: new Date(Date.now() - 7200000),
      owner: 'Bob Chen',
      syncStatus: 'syncing',
      fileType: 'video',
    },
    {
      id: '4',
      name: 'Source Code',
      type: 'folder',
      modified: new Date(Date.now() - 172800000),
      owner: 'You',
      syncStatus: 'synced',
    },
    {
      id: '5',
      name: 'app.tsx',
      type: 'file',
      size: 34567,
      modified: new Date(Date.now() - 1800000),
      owner: 'You',
      syncStatus: 'synced',
      fileType: 'code',
      starred: true,
    },
  ]);

  const [storageUsed] = useState(3.2); // GB
  const [storageTotal] = useState(10); // GB

  const formatFileSize = (bytes?: number): string => {
    if (!bytes) return '--';
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`;
  };

  const getFileIcon = (item: FileItem) => {
    if (item.type === 'folder') return <FolderIcon />;
    switch (item.fileType) {
      case 'image': return <ImageIcon />;
      case 'video': return <VideoIcon />;
      case 'audio': return <AudioIcon />;
      case 'document': return <DocIcon />;
      case 'code': return <CodeIcon />;
      case 'archive': return <ZipIcon />;
      default: return <FileIcon />;
    }
  };

  const getSyncIcon = (status: FileItem['syncStatus']) => {
    switch (status) {
      case 'synced': return <SyncedIcon color="success" fontSize="small" />;
      case 'syncing': return <SyncingIcon color="info" fontSize="small" />;
      case 'offline': return <OfflineIcon color="disabled" fontSize="small" />;
    }
  };

  const handleFileClick = (file: FileItem) => {
    if (file.type === 'folder') {
      // Navigate to folder
      onFileSelect?.(file);
    } else {
      // Open file preview or download
      onFileSelect?.(file);
    }
  };

  const handleCreateFolder = () => {
    if (newFolderName.trim()) {
      // Create folder logic
      setCreateFolderOpen(false);
      setNewFolderName('');
    }
  };

  const handleFileUpload = (event: React.ChangeEvent<HTMLInputElement>) => {
    if (event.target.files) {
      const filesArray = Array.from(event.target.files);
      onFileUpload?.(filesArray);
    }
  };

  const filteredFiles = files.filter(file =>
    file.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <Paper elevation={0} sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
          <Breadcrumbs>
            <Link href="#" underline="hover" color="inherit">
              My Files
            </Link>
            <Link href="#" underline="hover" color="inherit">
              Projects
            </Link>
            <Typography color="text.primary">Current Folder</Typography>
          </Breadcrumbs>
          
          <Box sx={{ display: 'flex', gap: 1 }}>
            <Button
              variant="outlined"
              startIcon={<NewFolderIcon />}
              onClick={() => setCreateFolderOpen(true)}
            >
              New Folder
            </Button>
            <Button
              variant="contained"
              startIcon={<UploadIcon />}
              component="label"
            >
              Upload
              <input
                type="file"
                hidden
                multiple
                onChange={handleFileUpload}
              />
            </Button>
          </Box>
        </Box>

        <Box sx={{ display: 'flex', gap: 2, alignItems: 'center' }}>
          <TextField
            size="small"
            placeholder="Search files..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            InputProps={{
              startAdornment: (
                <InputAdornment position="start">
                  <SearchIcon />
                </InputAdornment>
              ),
            }}
            sx={{ flex: 1, maxWidth: 400 }}
          />
          
          <Box sx={{ display: 'flex', gap: 1 }}>
            <IconButton
              onClick={() => setViewMode('list')}
              color={viewMode === 'list' ? 'primary' : 'default'}
            >
              <ListViewIcon />
            </IconButton>
            <IconButton
              onClick={() => setViewMode('grid')}
              color={viewMode === 'grid' ? 'primary' : 'default'}
            >
              <GridViewIcon />
            </IconButton>
          </Box>
        </Box>
      </Paper>

      {/* Storage Info */}
      <Paper elevation={0} sx={{ mx: 2, mt: 2, p: 2 }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 1 }}>
          <Typography variant="body2">Storage Used</Typography>
          <Typography variant="body2">
            {storageUsed} GB of {storageTotal} GB
          </Typography>
        </Box>
        <LinearProgress
          variant="determinate"
          value={(storageUsed / storageTotal) * 100}
          sx={{ height: 8, borderRadius: 1 }}
        />
      </Paper>

      {/* Files List/Grid */}
      <Box sx={{ flex: 1, overflow: 'auto', p: 2 }}>
        {viewMode === 'list' ? (
          <List>
            {filteredFiles.map((file) => (
              <ListItem
                key={file.id}
                button
                selected={selectedFiles.includes(file.id)}
                onClick={() => handleFileClick(file)}
                sx={{
                  borderRadius: 1,
                  mb: 0.5,
                  '&:hover': {
                    backgroundColor: 'action.hover',
                  },
                }}
              >
                <ListItemIcon>{getFileIcon(file)}</ListItemIcon>
                <ListItemText
                  primary={
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                      <Typography variant="body1">{file.name}</Typography>
                      {file.starred && <StarIcon color="warning" fontSize="small" />}
                      {file.shared && (
                        <Chip
                          label={`Shared with ${file.shared.length}`}
                          size="small"
                          variant="outlined"
                        />
                      )}
                      {file.version && (
                        <Chip label={`v${file.version}`} size="small" />
                      )}
                    </Box>
                  }
                  secondary={
                    <Box sx={{ display: 'flex', gap: 2 }}>
                      <Typography variant="caption">
                        {file.type === 'file' ? formatFileSize(file.size) : '--'}
                      </Typography>
                      <Typography variant="caption">
                        Modified {file.modified.toLocaleDateString()}
                      </Typography>
                      <Typography variant="caption">
                        by {file.owner}
                      </Typography>
                    </Box>
                  }
                />
                <ListItemSecondaryAction>
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                    {getSyncIcon(file.syncStatus)}
                    <IconButton
                      edge="end"
                      onClick={(e) => {
                        e.stopPropagation();
                        setAnchorEl(e.currentTarget);
                        setSelectedFile(file);
                      }}
                    >
                      <MoreIcon />
                    </IconButton>
                  </Box>
                </ListItemSecondaryAction>
              </ListItem>
            ))}
          </List>
        ) : (
          <Grid container spacing={2}>
            {filteredFiles.map((file) => (
              <Grid item xs={12} sm={6} md={4} lg={3} key={file.id}>
                <Card
                  sx={{
                    cursor: 'pointer',
                    '&:hover': {
                      boxShadow: 3,
                    },
                  }}
                  onClick={() => handleFileClick(file)}
                >
                  <CardContent sx={{ textAlign: 'center', py: 3 }}>
                    <Box sx={{ fontSize: 48, color: 'primary.main', mb: 1 }}>
                      {getFileIcon(file)}
                    </Box>
                    <Typography variant="body1" noWrap>
                      {file.name}
                    </Typography>
                    <Typography variant="caption" color="text.secondary">
                      {file.type === 'file' ? formatFileSize(file.size) : `${Math.floor(Math.random() * 10 + 1)} items`}
                    </Typography>
                  </CardContent>
                  <CardActions sx={{ justifyContent: 'space-between' }}>
                    <Box sx={{ display: 'flex', gap: 0.5 }}>
                      {getSyncIcon(file.syncStatus)}
                      {file.shared && <ShareIcon fontSize="small" />}
                    </Box>
                    <IconButton
                      size="small"
                      onClick={(e) => {
                        e.stopPropagation();
                        setAnchorEl(e.currentTarget);
                        setSelectedFile(file);
                      }}
                    >
                      <MoreIcon />
                    </IconButton>
                  </CardActions>
                </Card>
              </Grid>
            ))}
          </Grid>
        )}
      </Box>

      {/* File Actions Menu */}
      <Menu
        anchorEl={anchorEl}
        open={Boolean(anchorEl)}
        onClose={() => setAnchorEl(null)}
      >
        <MenuItem>
          <ListItemIcon><ShareIcon fontSize="small" /></ListItemIcon>
          <ListItemText>Share</ListItemText>
        </MenuItem>
        <MenuItem>
          <ListItemIcon><DownloadIcon fontSize="small" /></ListItemIcon>
          <ListItemText>Download</ListItemText>
        </MenuItem>
        <MenuItem>
          <ListItemIcon><HistoryIcon fontSize="small" /></ListItemIcon>
          <ListItemText>Version History</ListItemText>
        </MenuItem>
        <MenuItem>
          <ListItemIcon>
            {selectedFile?.starred ? <StarBorderIcon fontSize="small" /> : <StarIcon fontSize="small" />}
          </ListItemIcon>
          <ListItemText>{selectedFile?.starred ? 'Unstar' : 'Star'}</ListItemText>
        </MenuItem>
        <MenuItem>
          <ListItemIcon><DeleteIcon fontSize="small" /></ListItemIcon>
          <ListItemText>Delete</ListItemText>
        </MenuItem>
      </Menu>

      {/* Create Folder Dialog */}
      <Dialog open={createFolderOpen} onClose={() => setCreateFolderOpen(false)}>
        <DialogTitle>Create New Folder</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Folder Name"
            fullWidth
            variant="outlined"
            value={newFolderName}
            onChange={(e) => setNewFolderName(e.target.value)}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateFolderOpen(false)}>Cancel</Button>
          <Button onClick={handleCreateFolder} variant="contained">Create</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};