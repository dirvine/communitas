import React, { useState, useEffect, useCallback, useRef } from 'react';
import {
  Box,
  Paper,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  ListItemSecondaryAction,
  IconButton,
  Typography,
  Breadcrumbs,
  Link,
  Button,
  Stack,
  Chip,
  CircularProgress,
  LinearProgress,
  Alert,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Menu,
  MenuItem,
  Divider,
  Tooltip,
  Grid,
  Card,
  CardContent,
  CardActions,
  Select,
  FormControl,
  InputLabel,
  Fab,
  Tabs,
  Tab,
  Badge,
} from '@mui/material';
import {
  Folder as FolderIcon,
  InsertDriveFile as FileIcon,
  CloudUpload as UploadIcon,
  CloudDownload as DownloadIcon,
  CreateNewFolder as NewFolderIcon,
  Delete as DeleteIcon,
  Edit as EditIcon,
  Share as ShareIcon,
  MoreVert as MoreIcon,
  Lock as LockIcon,
  LockOpen as UnlockIcon,
  Security as SecurityIcon,
  Warning as WarningIcon,
  CheckCircle as HealthyIcon,
  Error as CriticalIcon,
  Home as HomeIcon,
  NavigateNext as NavigateNextIcon,
  ViewList as ListViewIcon,
  ViewModule as GridViewIcon,
  Search as SearchIcon,
  FilterList as FilterIcon,
  Sort as SortIcon,
  Refresh as RefreshIcon,
  Add as AddIcon,
  Code as CodeIcon,
  Image as ImageIcon,
  VideoFile as VideoIcon,
  AudioFile as AudioIcon,
  PictureAsPdf as PdfIcon,
  Description as DocumentIcon,
  Archive as ArchiveIcon,
  CloudOff as OfflineIcon,
  Language as WebsiteIcon,
  Storage as DataIcon,
  Public as PublicIcon,
  Group as GroupIcon,
  Language as LanguageOutlined,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { format } from 'date-fns';
import { useLocalStorage } from '../../contexts/LocalStorageProvider';
import { networkService } from '../../services/network/NetworkConnectionService';
import { offlineStorage } from '../../services/storage/OfflineStorageService';
import { GlassCard, GlassCardContent } from '../ui/GlassCard';
import { ModernButton } from '../ui/ModernButton';
import { designTokens } from '../../styles/theme';
import { motion } from 'framer-motion';
import { alpha, styled, useTheme } from '@mui/material/styles';

interface StorageItem {
  name: string;
  path: string;
  type: 'file' | 'folder';
  size?: number;
  modified?: string;
  encrypted: boolean;
  shared: boolean;
  permissions: string[];
  mime_type?: string;
  shard_status?: {
    available: number;
    total: number;
    health: 'healthy' | 'degraded' | 'critical';
  };
}

interface StorageStats {
  total_size: number;
  used_size: number;
  file_count: number;
  folder_count: number;
  encrypted_count: number;
  shared_count: number;
}

interface EncryptionStatus {
  enabled: boolean;
  threshold: { k: number; m: number };
  available_shards: number;
  health_status: 'healthy' | 'degraded' | 'critical';
}

type StorageArea = 'website' | 'data' | 'shared';

// Styled components for glassmorphism
const MotionCard = motion(GlassCard);
const StorageSection = styled(GlassCard)(({ theme }) => ({
  marginBottom: theme.spacing(2),
  padding: theme.spacing(2),
  background: alpha(theme.palette.background.paper, 0.6),
  backdropFilter: 'blur(10px)',
}));

const StyledFab = styled(Fab)(({ theme }) => ({
  background: designTokens.colors.primary.gradient,
  color: '#ffffff',
  boxShadow: designTokens.shadows.xl,
  '&:hover': {
    background: designTokens.colors.primary.gradient,
    transform: 'scale(1.1)',
  },
}));

interface StoragePanelProps {
  entityType: 'individual' | 'group' | 'channel' | 'project';
  entityId: string;
  entityName: string;
  fourWords: string;
  permissions: string[];
  encryptionStatus: EncryptionStatus | null;
  initialArea?: StorageArea;
}

const StoragePanel: React.FC<StoragePanelProps> = ({
  entityType,
  entityId,
  entityName,
  fourWords,
  permissions,
  encryptionStatus,
  initialArea,
}) => {
  // Try to use LocalStorage context, fall back to direct service if not available
  let localStorage: any = null;
  try {
    localStorage = useLocalStorage();
  } catch (error) {
    console.warn('[StoragePanel] LocalStorageProvider not available, using fallback');
    // Provide fallback implementation using offlineStorage directly
    localStorage = {
      list: async (entityId: string, path: string) => {
        const cached = await offlineStorage.get(`storage:${entityId}:${path}`) || [];
        return cached;
      },
      read: async (entityId: string, path: string) => {
        return await offlineStorage.get(`file:${entityId}:${path}`);
      },
      write: async (entityId: string, path: string, content: any) => {
        await offlineStorage.store(`file:${entityId}:${path}`, content, {
          encrypt: true,
          syncOnline: true
        });
      },
      delete: async (entityId: string, path: string) => {
        await offlineStorage.remove(`file:${entityId}:${path}`);
      },
      mkdir: async (entityId: string, path: string) => {
        // Store folder metadata
        await offlineStorage.store(`folder:${entityId}:${path}`, {
          created: new Date().toISOString()
        });
      },
      isOnline: false,
      syncInProgress: false,
      lastSyncError: null,
      getSyncQueue: () => [],
      forceSyncNow: async () => {},
      clearSyncQueue: () => {}
    };
  }

  // Storage area state
  const [currentArea, setCurrentArea] = useState<StorageArea>(initialArea || 'website');
  
  // Separate state for each storage area
  const [websiteItems, setWebsiteItems] = useState<StorageItem[]>([]);
  const [dataItems, setDataItems] = useState<StorageItem[]>([]);
  const [currentPath, setCurrentPath] = useState('/');
  const [loading, setLoading] = useState(true);
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedItems, setSelectedItems] = useState<Set<string>>(new Set());
  const [viewMode, setViewMode] = useState<'list' | 'grid'>('list');
  const [sortBy, setSortBy] = useState<'name' | 'size' | 'modified'>('name');
  const [filterType, setFilterType] = useState<'all' | 'files' | 'folders'>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [websiteStats, setWebsiteStats] = useState<StorageStats | null>(null);
  const [dataStats, setDataStats] = useState<StorageStats | null>(null);
  const [isOffline, setIsOffline] = useState(false);
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; item: StorageItem } | null>(null);
  const [renameDialog, setRenameDialog] = useState<{ open: boolean; item: StorageItem | null }>({ open: false, item: null });
  const [newFolderDialog, setNewFolderDialog] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');
  const [newItemName, setNewItemName] = useState('');
  const fileInputRef = useRef<HTMLInputElement>(null);
  
  // Get current items based on active area
  const items = currentArea === 'website' ? websiteItems : dataItems;
  const stats = currentArea === 'website' ? websiteStats : dataStats;

  const canWrite = permissions.includes('write') || permissions.includes('admin');
  const canDelete = permissions.includes('admin');
  const canShare = permissions.includes('share') || permissions.includes('admin');

  // Monitor network status
  useEffect(() => {
    const unsubscribe = networkService.subscribe((state) => {
      setIsOffline(state.status !== 'connected');
    });

    // Get initial state
    const currentState = networkService.getState();
    setIsOffline(currentState.status !== 'connected');

    return unsubscribe;
  }, []);

  // Get storage root path based on current area
  const getStorageRoot = (area: StorageArea) => {
    return area === 'website' ? '/web' : '/personal';
  };

  // Load storage items for specific area
  const loadItems = useCallback(async (area?: StorageArea) => {
    const targetArea = area || currentArea;
    const rootPath = getStorageRoot(targetArea);
    const fullPath = rootPath + currentPath;
    
    try {
      setLoading(true);
      setError(null);

      // Use local-first storage with area-specific path
      const files = await localStorage.list(entityId, fullPath);

      // Convert to StorageItem format
      const storageItems: StorageItem[] = files.map(file => ({
        name: file.name,
        path: file.path,
        type: file.isDirectory ? 'folder' : 'file',
        size: file.size,
        modified: file.modifiedAt,
        mime_type: file.contentType,
        encrypted: targetArea === 'data', // Data area is always encrypted
        shared: targetArea === 'data' && entityType !== 'individual', // Shared if data storage and not individual
        permissions: permissions,
      }));

      // Update the appropriate state
      if (targetArea === 'website') {
        setWebsiteItems(storageItems);
      } else {
        setDataItems(storageItems);
      }

      // Try to load stats if online (non-blocking)
      if (localStorage.isOnline) {
        try {
          const storageStats = await invoke('core_storage_stats', {
            entityId,
            path: rootPath,
          });
          if (targetArea === 'website') {
            setWebsiteStats(storageStats as StorageStats);
          } else {
            setDataStats(storageStats as StorageStats);
          }
        } catch (err) {
          // Stats are optional, don't fail the whole operation
          console.warn('Could not load storage stats:', err);
        }
      }
    } catch (err) {
      console.error('Failed to load storage items:', err);
      setError('Failed to load storage items. Working in offline mode.');
      // Still show empty list rather than crashing
      if (targetArea === 'website') {
        setWebsiteItems([]);
      } else {
        setDataItems([]);
      }
    } finally {
      setLoading(false);
    }
  }, [entityId, currentPath, localStorage, encryptionStatus, permissions, currentArea, entityType]);

  // Handle area change
  const handleAreaChange = (event: React.SyntheticEvent, newArea: StorageArea) => {
    setCurrentArea(newArea);
    setCurrentPath('/'); // Reset path when switching areas
    setSelectedItems(new Set()); // Clear selection
    loadItems(newArea); // Load items for new area
  };

  // Upload file with appropriate encryption based on storage area
  const handleFileUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = event.target.files;
    if (!files || files.length === 0 || !canWrite) return;

    setUploading(true);
    try {
      const rootPath = getStorageRoot(currentArea);
      
      for (const file of files) {
        const arrayBuffer = await file.arrayBuffer();
        const content = new Uint8Array(arrayBuffer);
        const fullPath = `${rootPath}${currentPath}/${file.name}`;

        if (currentArea === 'website') {
          // Website storage: use regular file operations (no encryption)
          await invoke('core_entity_write_file', {
            entityId,
            entityType,
            path: fullPath,
            content: Array.from(content),
          });
        } else {
          // Data storage: use threshold encryption
          const recipients = await invoke('core_entity_get_members', {
            entityId,
          }) as string[];

          await invoke('core_entity_write_file_sealed', {
            entityId,
            entityType,
            path: fullPath,
            content: Array.from(content),
            recipients,
          });
        }
      }

      await loadItems();
    } catch (err) {
      console.error('Failed to upload files:', err);
      setError('Failed to upload files');
    } finally {
      setUploading(false);
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    }
  };

  // Download file with appropriate decryption based on storage area
  const handleDownload = async (item: StorageItem) => {
    try {
      let content: number[];
      
      if (currentArea === 'website') {
        // Website storage: use regular file operations (no decryption)
        content = await invoke('core_entity_read_file', {
          entityId,
          path: item.path,
        }) as number[];
      } else {
        // Data storage: use threshold decryption
        content = await invoke('core_entity_read_file_sealed', {
          entityId,
          path: item.path,
        }) as number[];
      }

      // Convert to blob and download
      const blob = new Blob([new Uint8Array(content)], { type: item.mime_type || 'application/octet-stream' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = item.name;
      a.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error('Failed to download file:', err);
      setError('Failed to download file');
    }
  };

  // Create new folder
  const handleCreateFolder = async () => {
    if (!newFolderName.trim() || !canWrite) return;

    try {
      const rootPath = getStorageRoot(currentArea);
      const fullPath = `${rootPath}${currentPath}/${newFolderName.trim()}`;
      
      await invoke('core_storage_mkdir', {
        entityId,
        path: fullPath,
      });

      setNewFolderDialog(false);
      setNewFolderName('');
      await loadItems();
    } catch (err) {
      console.error('Failed to create folder:', err);
      setError('Failed to create folder');
    }
  };

  // Delete items
  const handleDelete = async (itemsToDelete: StorageItem[]) => {
    if (!canDelete) return;

    try {
      for (const item of itemsToDelete) {
        await invoke('core_storage_delete', {
          entityId,
          path: item.path, // Path should already include the correct root from loadItems
        });
      }

      setSelectedItems(new Set());
      await loadItems();
    } catch (err) {
      console.error('Failed to delete items:', err);
      setError('Failed to delete items');
    }
  };

  // Rename item
  const handleRename = async () => {
    if (!renameDialog.item || !newItemName.trim() || !canWrite) return;

    try {
      const rootPath = getStorageRoot(currentArea);
      const newPath = `${rootPath}${currentPath}/${newItemName.trim()}`;
      await invoke('core_storage_rename', {
        entityId,
        oldPath: renameDialog.item.path,
        newPath,
      });

      setRenameDialog({ open: false, item: null });
      setNewItemName('');
      await loadItems();
    } catch (err) {
      console.error('Failed to rename item:', err);
      setError('Failed to rename item');
    }
  };

  // Navigate to folder
  const navigateToFolder = (path: string) => {
    setCurrentPath(path);
    setSelectedItems(new Set());
  };

  // Get file icon
  const getFileIcon = (item: StorageItem) => {
    if (item.type === 'folder') return <FolderIcon />;

    const ext = item.name.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'jpg':
      case 'jpeg':
      case 'png':
      case 'gif':
      case 'svg':
        return <ImageIcon />;
      case 'mp4':
      case 'avi':
      case 'mov':
      case 'webm':
        return <VideoIcon />;
      case 'mp3':
      case 'wav':
      case 'ogg':
      case 'flac':
        return <AudioIcon />;
      case 'pdf':
        return <PdfIcon />;
      case 'doc':
      case 'docx':
      case 'txt':
      case 'md':
        return <DocumentIcon />;
      case 'zip':
      case 'tar':
      case 'gz':
      case '7z':
        return <ArchiveIcon />;
      case 'js':
      case 'ts':
      case 'jsx':
      case 'tsx':
      case 'py':
      case 'rs':
      case 'go':
        return <CodeIcon />;
      default:
        return <FileIcon />;
    }
  };

  // Format file size
  const formatFileSize = (bytes?: number) => {
    if (!bytes) return '-';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return `${size.toFixed(1)} ${units[unitIndex]}`;
  };

  // Get health color
  const getHealthColor = (health?: string) => {
    switch (health) {
      case 'healthy': return 'success';
      case 'degraded': return 'warning';
      case 'critical': return 'error';
      default: return 'default';
    }
  };

  // Filter and sort items
  const processedItems = items
    .filter(item => {
      if (filterType === 'files' && item.type !== 'file') return false;
      if (filterType === 'folders' && item.type !== 'folder') return false;
      if (searchQuery && !item.name.toLowerCase().includes(searchQuery.toLowerCase())) return false;
      return true;
    })
    .sort((a, b) => {
      switch (sortBy) {
        case 'size':
          return (b.size || 0) - (a.size || 0);
        case 'modified':
          return new Date(b.modified || 0).getTime() - new Date(a.modified || 0).getTime();
        default:
          return a.name.localeCompare(b.name);
      }
    });

  useEffect(() => {
    // Load items for both areas on mount
    loadItems('website');
    loadItems('data');
  }, []);

  // Reload current area when switching
  useEffect(() => {
    loadItems(currentArea);
  }, [currentArea, loadItems]);

  // Refresh on entity:refresh event
  useEffect(() => {
    const handleRefresh = (event: CustomEvent) => {
      if (event.detail.entityId === entityId) {
        loadItems();
      }
    };
    window.addEventListener('entity:refresh', handleRefresh as EventListener);
    return () => {
      window.removeEventListener('entity:refresh', handleRefresh as EventListener);
    };
  }, [entityId, loadItems]);

  if (loading && items.length === 0) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%' }}>
        <CircularProgress />
      </Box>
    );
  }

  const theme = useTheme();

  // For individual contacts, show a different layout with shared storage emphasized
  if (entityType === 'individual') {
    return (
      <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column', p: 2 }}>
        <Grid container spacing={2}>
          {/* Shared Storage Section */}
          <Grid item xs={12} md={6}>
            <MotionCard
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.5 }}
              variant="light"
              elevation={0}
            >
              <CardContent>
                <Stack direction="row" alignItems="center" spacing={2} sx={{ mb: 2 }}>
                  <ShareIcon color="primary" />
                  <Typography variant="h6" fontWeight={600}>
                    Shared with {entityName}
                  </Typography>
                  <Badge badgeContent={3} color="primary">
                    <Box />
                  </Badge>
                </Stack>

                <List dense>
                  <ListItem>
                    <ListItemIcon><FileIcon /></ListItemIcon>
                    <ListItemText
                      primary="Project Proposal.pdf"
                      secondary="Shared 2 days ago • 2.4 MB"
                    />
                    <IconButton size="small"><DownloadIcon /></IconButton>
                  </ListItem>
                  <ListItem>
                    <ListItemIcon><FolderIcon /></ListItemIcon>
                    <ListItemText
                      primary="Design Assets"
                      secondary="Shared 1 week ago • 12 files"
                    />
                    <IconButton size="small"><MoreIcon /></IconButton>
                  </ListItem>
                </List>

                <ModernButton
                  variant="contained"
                  gradient={true}
                  startIcon={<AddIcon />}
                  fullWidth
                  sx={{ mt: 2 }}
                  onClick={() => fileInputRef.current?.click()}
                >
                  Share New Files
                </ModernButton>
              </CardContent>
            </MotionCard>
          </Grid>

          {/* Their Website Section */}
          <Grid item xs={12} md={6}>
            <MotionCard
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.5, delay: 0.1 }}
              variant="colored"
              elevation={0}
            >
              <CardContent>
                <Stack direction="row" alignItems="center" spacing={2} sx={{ mb: 2 }}>
                  <WebsiteIcon color="info" />
                  <Typography variant="h6" fontWeight={600}>
                    {entityName}'s Website
                  </Typography>
                  <Chip label="Public" size="small" color="info" />
                </Stack>

                <Box sx={{ mb: 2 }}>
                  <Typography variant="body2" color="text.secondary" gutterBottom>
                    {fourWords}
                  </Typography>
                  <Typography variant="caption" color="text.secondary">
                    Last updated 3 hours ago
                  </Typography>
                </Box>

                <List dense>
                  <ListItem>
                    <ListItemIcon><FileIcon /></ListItemIcon>
                    <ListItemText
                      primary="index.md"
                      secondary="Home page"
                    />
                  </ListItem>
                  <ListItem>
                    <ListItemIcon><FileIcon /></ListItemIcon>
                    <ListItemText
                      primary="portfolio.md"
                      secondary="Work samples"
                    />
                  </ListItem>
                </List>

                <ModernButton
                  variant="contained"
                  gradient={true}
                  startIcon={<LanguageOutlined />}
                  fullWidth
                  sx={{ mt: 2 }}
                >
                  Visit Website
                </ModernButton>
              </CardContent>
            </MotionCard>
          </Grid>

          {/* Files They Shared Section */}
          <Grid item xs={12}>
            <MotionCard
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.2 }}
              variant="light"
              elevation={0}
            >
              <CardContent>
                <Stack direction="row" alignItems="center" spacing={2} sx={{ mb: 2 }}>
                  <FolderIcon color="secondary" />
                  <Typography variant="h6" fontWeight={600}>
                    Files from {entityName}
                  </Typography>
                  <Badge badgeContent={5} color="secondary">
                    <Box />
                  </Badge>
                </Stack>

                <Grid container spacing={2}>
                  {[1, 2, 3, 4, 5].map((i) => (
                    <Grid item xs={12} sm={6} md={4} key={i}>
                      <Paper
                        sx={{
                          p: 2,
                          cursor: 'pointer',
                          background: alpha(theme.palette.background.paper, 0.6),
                          backdropFilter: 'blur(10px)',
                          '&:hover': {
                            background: alpha(theme.palette.primary.main, 0.1),
                          },
                        }}
                      >
                        <Stack direction="row" spacing={2} alignItems="center">
                          <FileIcon color="action" />
                          <Box flex={1}>
                            <Typography variant="body2">Document{i}.pdf</Typography>
                            <Typography variant="caption" color="text.secondary">
                              1.2 MB • 2 days ago
                            </Typography>
                          </Box>
                          <IconButton size="small">
                            <DownloadIcon fontSize="small" />
                          </IconButton>
                        </Stack>
                      </Paper>
                    </Grid>
                  ))}
                </Grid>
              </CardContent>
            </MotionCard>
          </Grid>
        </Grid>
      </Box>
    );
  }

  // Original layout for groups/channels/projects
  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Storage Header */}
      <Paper elevation={0} sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
        {/* Storage Area Tabs */}
        <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 2 }}>
          <Tabs value={currentArea} onChange={handleAreaChange} aria-label="storage areas">
            <Tab 
              icon={<WebsiteIcon />}
              label={
                <Stack direction="row" alignItems="center" spacing={1}>
                  <Typography variant="body2">Website Storage</Typography>
                  <Badge 
                    badgeContent={websiteStats?.file_count || 0} 
                    color="primary"
                    variant="standard"
                  />
                </Stack>
              } 
              value="website" 
              iconPosition="start"
            />
            <Tab 
              icon={currentArea === 'data' ? <LockIcon /> : <DataIcon />}
              label={
                <Stack direction="row" alignItems="center" spacing={1}>
                  <Typography variant="body2">Data Storage</Typography>
                  <Badge 
                    badgeContent={dataStats?.file_count || 0} 
                    color="secondary"
                    variant="standard"
                  />
                  {currentArea === 'data' && (
                    <Chip 
                      label="Encrypted" 
                      size="small" 
                      color="secondary" 
                      variant="outlined"
                    />
                  )}
                </Stack>
              }
              value="data" 
              iconPosition="start"
            />
          </Tabs>
        </Box>

        {/* Storage Area Description */}
        <Alert 
          severity={currentArea === 'website' ? 'info' : 'warning'}
          icon={currentArea === 'website' ? <PublicIcon /> : <LockIcon />}
          sx={{ mb: 2 }}
        >
          <Typography variant="body2">
            {currentArea === 'website' 
              ? `Website Storage: Plain markdown files for public website content. Files stored here are not encrypted and can be published as a decentralized website.`
              : `Data Storage: Threshold encrypted files shared with ${entityType} members. All files are automatically encrypted using ${encryptionStatus?.threshold.k || 2}-of-${(encryptionStatus?.threshold.k || 2) + (encryptionStatus?.threshold.m || 1)} threshold encryption.`
            }
          </Typography>
        </Alert>

        {/* Encryption Status for Data Storage */}
        {currentArea === 'data' && encryptionStatus && (
          <Alert
            severity={getHealthColor(encryptionStatus.health_status) as any}
            icon={
              encryptionStatus.health_status === 'healthy' ? <HealthyIcon /> :
              encryptionStatus.health_status === 'degraded' ? <WarningIcon /> :
              <CriticalIcon />
            }
            sx={{ mb: 2 }}
          >
            <Stack direction="row" alignItems="center" spacing={2}>
              <Typography variant="body2">
                Threshold Encryption: {encryptionStatus.threshold.k}-of-{encryptionStatus.threshold.k + encryptionStatus.threshold.m}
              </Typography>
              <Chip
                label={`${encryptionStatus.available_shards} shards available`}
                size="small"
                color={getHealthColor(encryptionStatus.health_status) as any}
              />
            </Stack>
          </Alert>
        )}

        {/* Breadcrumbs */}
        <Breadcrumbs separator={<NavigateNextIcon fontSize="small" />} sx={{ mb: 2 }}>
          <Link
            component="button"
            variant="body1"
            onClick={() => navigateToFolder('/')}
            underline="hover"
            color="inherit"
          >
            <HomeIcon sx={{ mr: 0.5, verticalAlign: 'bottom' }} fontSize="small" />
            {entityName}
          </Link>
          {currentPath !== '/' && currentPath.split('/').filter(Boolean).map((folder, index, arr) => (
            <Link
              key={index}
              component="button"
              variant="body1"
              onClick={() => navigateToFolder('/' + arr.slice(0, index + 1).join('/'))}
              underline="hover"
              color={index === arr.length - 1 ? 'text.primary' : 'inherit'}
            >
              {folder}
            </Link>
          ))}
        </Breadcrumbs>

        {/* Toolbar */}
        <Stack direction="row" alignItems="center" justifyContent="space-between">
          <Stack direction="row" spacing={1}>
            {canWrite && (
              <>
                <input
                  ref={fileInputRef}
                  type="file"
                  multiple
                  hidden
                  onChange={handleFileUpload}
                />
                <Button
                  variant="contained"
                  startIcon={uploading ? <CircularProgress size={16} /> : <UploadIcon />}
                  onClick={() => fileInputRef.current?.click()}
                  disabled={uploading}
                >
                  Upload
                </Button>
                <Button
                  variant="outlined"
                  startIcon={<NewFolderIcon />}
                  onClick={() => setNewFolderDialog(true)}
                >
                  New Folder
                </Button>
              </>
            )}

            {selectedItems.size > 0 && (
              <>
                <Button
                  startIcon={<DownloadIcon />}
                  onClick={() => {
                    processedItems
                      .filter(item => selectedItems.has(item.path))
                      .forEach(item => handleDownload(item));
                  }}
                >
                  Download ({selectedItems.size})
                </Button>

                {canDelete && (
                  <Button
                    color="error"
                    startIcon={<DeleteIcon />}
                    onClick={() => {
                      const itemsToDelete = processedItems.filter(item => selectedItems.has(item.path));
                      handleDelete(itemsToDelete);
                    }}
                  >
                    Delete ({selectedItems.size})
                  </Button>
                )}
              </>
            )}
          </Stack>

          <Stack direction="row" spacing={1} alignItems="center">
            {/* Search */}
            <TextField
              size="small"
              placeholder="Search..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              InputProps={{
                startAdornment: <SearchIcon fontSize="small" sx={{ mr: 1, color: 'text.secondary' }} />,
              }}
            />

            {/* Filter */}
            <FormControl size="small" sx={{ minWidth: 100 }}>
              <Select
                value={filterType}
                onChange={(e) => setFilterType(e.target.value as any)}
                displayEmpty
              >
                <MenuItem value="all">All</MenuItem>
                <MenuItem value="files">Files</MenuItem>
                <MenuItem value="folders">Folders</MenuItem>
              </Select>
            </FormControl>

            {/* Sort */}
            <FormControl size="small" sx={{ minWidth: 100 }}>
              <Select
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value as any)}
                displayEmpty
              >
                <MenuItem value="name">Name</MenuItem>
                <MenuItem value="size">Size</MenuItem>
                <MenuItem value="modified">Modified</MenuItem>
              </Select>
            </FormControl>

            {/* View Mode */}
            <IconButton onClick={() => setViewMode(viewMode === 'list' ? 'grid' : 'list')}>
              {viewMode === 'list' ? <GridViewIcon /> : <ListViewIcon />}
            </IconButton>

            {/* Refresh */}
            <IconButton onClick={() => { void loadItems(); }}>
              <RefreshIcon />
            </IconButton>
          </Stack>
        </Stack>

        {/* Storage Stats */}
        {stats && (
          <Stack direction="row" spacing={2} sx={{ mt: 2 }}>
            <Chip label={`${stats.file_count} files`} size="small" />
            <Chip label={`${stats.folder_count} folders`} size="small" />
            <Chip label={formatFileSize(stats.used_size)} size="small" />
            {stats.encrypted_count > 0 && (
              <Chip
                icon={<LockIcon />}
                label={`${stats.encrypted_count} encrypted`}
                size="small"
                color="primary"
              />
            )}
            {stats.shared_count > 0 && (
              <Chip
                icon={<ShareIcon />}
                label={`${stats.shared_count} shared`}
                size="small"
                color="secondary"
              />
            )}
          </Stack>
        )}
      </Paper>

      {/* Storage Content */}
      <Box sx={{ flex: 1, overflow: 'auto', p: 2 }}>
        {error && (
          <Alert severity="error" onClose={() => setError(null)} sx={{ mb: 2 }}>
            {error}
          </Alert>
        )}

        {uploading && (
          <LinearProgress sx={{ mb: 2 }} />
        )}

        {viewMode === 'list' ? (
          <List>
            {processedItems.map(item => (
              <ListItem
                key={item.path}
                button
                selected={selectedItems.has(item.path)}
                onClick={() => {
                  if (item.type === 'folder') {
                    navigateToFolder(item.path);
                  } else {
                    const newSelected = new Set(selectedItems);
                    if (newSelected.has(item.path)) {
                      newSelected.delete(item.path);
                    } else {
                      newSelected.add(item.path);
                    }
                    setSelectedItems(newSelected);
                  }
                }}
                onContextMenu={(e) => {
                  e.preventDefault();
                  setContextMenu({ x: e.clientX, y: e.clientY, item });
                }}
              >
                <ListItemIcon>
                  {getFileIcon(item)}
                </ListItemIcon>

                <ListItemText
                  primary={
                    <Stack direction="row" alignItems="center" spacing={1}>
                      <Typography variant="body2">{item.name}</Typography>
                      {item.encrypted && (
                        <Tooltip title="Encrypted">
                          <LockIcon fontSize="small" color="action" />
                        </Tooltip>
                      )}
                      {item.shared && (
                        <Tooltip title="Shared">
                          <ShareIcon fontSize="small" color="action" />
                        </Tooltip>
                      )}
                      {item.shard_status && (
                        <Chip
                          label={`${item.shard_status.available}/${item.shard_status.total}`}
                          size="small"
                          color={getHealthColor(item.shard_status.health) as any}
                          sx={{ height: 20 }}
                        />
                      )}
                    </Stack>
                  }
                  secondary={
                    <Stack direction="row" spacing={2}>
                      {item.size !== undefined && (
                        <Typography variant="caption" color="text.secondary">
                          {formatFileSize(item.size)}
                        </Typography>
                      )}
                      {item.modified && (
                        <Typography variant="caption" color="text.secondary">
                          {format(new Date(item.modified), 'dd/MM/yyyy HH:mm')}
                        </Typography>
                      )}
                    </Stack>
                  }
                />

                <ListItemSecondaryAction>
                  <IconButton
                    edge="end"
                    onClick={(e) => {
                      e.stopPropagation();
                      setContextMenu({ x: e.currentTarget.getBoundingClientRect().left, y: e.currentTarget.getBoundingClientRect().bottom, item });
                    }}
                  >
                    <MoreIcon />
                  </IconButton>
                </ListItemSecondaryAction>
              </ListItem>
            ))}
          </List>
        ) : (
          <Grid container spacing={2}>
            {processedItems.map(item => (
              <Grid item xs={12} sm={6} md={4} lg={3} key={item.path}>
                <Card
                  sx={{
                    cursor: 'pointer',
                    border: selectedItems.has(item.path) ? 2 : 0,
                    borderColor: 'primary.main',
                  }}
                  onClick={() => {
                    if (item.type === 'folder') {
                      navigateToFolder(item.path);
                    } else {
                      const newSelected = new Set(selectedItems);
                      if (newSelected.has(item.path)) {
                        newSelected.delete(item.path);
                      } else {
                        newSelected.add(item.path);
                      }
                      setSelectedItems(newSelected);
                    }
                  }}
                  onContextMenu={(e) => {
                    e.preventDefault();
                    setContextMenu({ x: e.clientX, y: e.clientY, item });
                  }}
                >
                  <CardContent>
                    <Stack alignItems="center" spacing={2}>
                      <Box sx={{ fontSize: 48 }}>
                        {getFileIcon(item)}
                      </Box>
                      <Typography variant="body2" noWrap sx={{ width: '100%', textAlign: 'center' }}>
                        {item.name}
                      </Typography>
                      <Stack direction="row" spacing={1}>
                        {item.encrypted && <LockIcon fontSize="small" color="action" />}
                        {item.shared && <ShareIcon fontSize="small" color="action" />}
                      </Stack>
                      {item.size !== undefined && (
                        <Typography variant="caption" color="text.secondary">
                          {formatFileSize(item.size)}
                        </Typography>
                      )}
                    </Stack>
                  </CardContent>
                </Card>
              </Grid>
            ))}
          </Grid>
        )}
      </Box>

      {/* Context Menu */}
      <Menu
        open={contextMenu !== null}
        onClose={() => setContextMenu(null)}
        anchorReference="anchorPosition"
        anchorPosition={
          contextMenu !== null
            ? { top: contextMenu.y, left: contextMenu.x }
            : undefined
        }
      >
        {contextMenu?.item.type === 'file' && (
          <MenuItem onClick={() => { handleDownload(contextMenu.item); setContextMenu(null); }}>
            <ListItemIcon><DownloadIcon fontSize="small" /></ListItemIcon>
            <ListItemText>Download</ListItemText>
          </MenuItem>
        )}

        {canShare && (
          <MenuItem onClick={() => setContextMenu(null)}>
            <ListItemIcon><ShareIcon fontSize="small" /></ListItemIcon>
            <ListItemText>Share</ListItemText>
          </MenuItem>
        )}

        {canWrite && (
          <MenuItem onClick={() => {
            if (contextMenu) {
              setRenameDialog({ open: true, item: contextMenu.item });
              setNewItemName(contextMenu.item.name);
            }
            setContextMenu(null);
          }}>
            <ListItemIcon><EditIcon fontSize="small" /></ListItemIcon>
            <ListItemText>Rename</ListItemText>
          </MenuItem>
        )}

        <Divider />

        {canDelete && (
          <MenuItem onClick={() => {
            if (contextMenu) {
              handleDelete([contextMenu.item]);
            }
            setContextMenu(null);
          }}>
            <ListItemIcon><DeleteIcon fontSize="small" color="error" /></ListItemIcon>
            <ListItemText>Delete</ListItemText>
          </MenuItem>
        )}
      </Menu>

      {/* New Folder Dialog */}
      <Dialog open={newFolderDialog} onClose={() => setNewFolderDialog(false)}>
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
            onKeyPress={(e) => {
              if (e.key === 'Enter') {
                handleCreateFolder();
              }
            }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setNewFolderDialog(false)}>Cancel</Button>
          <Button onClick={handleCreateFolder} variant="contained">Create</Button>
        </DialogActions>
      </Dialog>

      {/* Rename Dialog */}
      <Dialog open={renameDialog.open} onClose={() => setRenameDialog({ open: false, item: null })}>
        <DialogTitle>Rename {renameDialog.item?.type === 'folder' ? 'Folder' : 'File'}</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="New Name"
            fullWidth
            variant="outlined"
            value={newItemName}
            onChange={(e) => setNewItemName(e.target.value)}
            onKeyPress={(e) => {
              if (e.key === 'Enter') {
                handleRename();
              }
            }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setRenameDialog({ open: false, item: null })}>Cancel</Button>
          <Button onClick={handleRename} variant="contained">Rename</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default StoragePanel;
