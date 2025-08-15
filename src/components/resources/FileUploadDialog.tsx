import React, { useState, useRef, useCallback } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Box,
  Typography,
  LinearProgress,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  ListItemSecondaryAction,
  IconButton,
  Alert,
  Stack,
  Chip,
  Paper,
  Divider,
  TextField,
  Switch,
  FormControlLabel,
  Collapse,
  Card,
  CardContent,
  Grid,
} from '@mui/material';
import {
  CloudUpload as CloudUploadIcon,
  InsertDriveFile as FileIcon,
  Image as ImageIcon,
  VideoFile as VideoIcon,
  AudioFile as AudioIcon,
  Delete as DeleteIcon,
  CheckCircle as CheckCircleIcon,
  Error as ErrorIcon,
  Warning as WarningIcon,
  Info as InfoIcon,
  Security as SecurityIcon,
  Schedule as ScheduleIcon,
  Storage as StorageIcon,
  Visibility as VisibilityIcon,
  Group as GroupIcon,
  ExpandMore as ExpandMoreIcon,
  ExpandLess as ExpandLessIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';

interface FileUploadItem {
  id: string;
  file: File;
  progress: number;
  status: 'pending' | 'uploading' | 'completed' | 'error' | 'cancelled';
  error?: string;
  preview?: string;
}

interface UploadSettings {
  compress_images: boolean;
  generate_thumbnails: boolean;
  virus_scan: boolean;
  auto_tag: boolean;
  notify_members: boolean;
  public_access: boolean;
  max_file_size_mb: number;
  allowed_extensions: string[];
}

interface FileUploadDialogProps {
  open: boolean;
  onClose: () => void;
  onUpload: (files: FileUploadItem[], settings: UploadSettings) => Promise<void>;
  targetPath: string;
  entityName: string;
  maxFileSize?: number; // in MB
  allowedTypes?: string[];
  currentStorageUsed?: number; // in GB
  storageQuota?: number; // in GB
}

const DEFAULT_SETTINGS: UploadSettings = {
  compress_images: true,
  generate_thumbnails: true,
  virus_scan: true,
  auto_tag: true,
  notify_members: false,
  public_access: false,
  max_file_size_mb: 100,
  allowed_extensions: ['jpg', 'jpeg', 'png', 'gif', 'pdf', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'txt', 'zip', 'rar', 'mp4', 'avi', 'mov', 'mp3', 'wav'],
};

export const FileUploadDialog: React.FC<FileUploadDialogProps> = ({
  open,
  onClose,
  onUpload,
  targetPath,
  entityName,
  maxFileSize = 100,
  allowedTypes,
  currentStorageUsed = 0,
  storageQuota = 50,
}) => {
  const [files, setFiles] = useState<FileUploadItem[]>([]);
  const [settings, setSettings] = useState<UploadSettings>(DEFAULT_SETTINGS);
  const [uploading, setUploading] = useState(false);
  const [dragOver, setDragOver] = useState(false);
  const [showAdvancedSettings, setShowAdvancedSettings] = useState(false);
  const [totalProgress, setTotalProgress] = useState(0);

  const fileInputRef = useRef<HTMLInputElement>(null);
  const dropZoneRef = useRef<HTMLDivElement>(null);

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const getFileIcon = (file: File) => {
    const type = file.type;
    if (type.startsWith('image/')) return <ImageIcon color="info" />;
    if (type.startsWith('video/')) return <VideoIcon color="secondary" />;
    if (type.startsWith('audio/')) return <AudioIcon color="warning" />;
    return <FileIcon color="action" />;
  };

  const validateFile = (file: File): { valid: boolean; error?: string } => {
    // Check file size
    if (file.size > settings.max_file_size_mb * 1024 * 1024) {
      return { valid: false, error: `File size exceeds ${settings.max_file_size_mb}MB limit` };
    }

    // Check file extension
    const extension = file.name.split('.').pop()?.toLowerCase();
    if (extension && !settings.allowed_extensions.includes(extension)) {
      return { valid: false, error: `File type .${extension} is not allowed` };
    }

    // Check storage quota
    const totalFileSize = files.reduce((acc, f) => acc + f.file.size, 0) + file.size;
    const totalSizeGB = totalFileSize / (1024 * 1024 * 1024);
    if (currentStorageUsed + totalSizeGB > storageQuota) {
      return { valid: false, error: 'Insufficient storage space' };
    }

    return { valid: true };
  };

  const generatePreview = async (file: File): Promise<string | undefined> => {
    if (!file.type.startsWith('image/')) return undefined;
    
    return new Promise((resolve) => {
      const reader = new FileReader();
      reader.onload = (e) => resolve(e.target?.result as string);
      reader.onerror = () => resolve(undefined);
      reader.readAsDataURL(file);
    });
  };

  const addFiles = async (newFiles: File[]) => {
    const uploadItems: FileUploadItem[] = [];

    for (const file of newFiles) {
      const validation = validateFile(file);
      const preview = await generatePreview(file);

      uploadItems.push({
        id: Math.random().toString(36).substring(7),
        file,
        progress: 0,
        status: validation.valid ? 'pending' : 'error',
        error: validation.error,
        preview,
      });
    }

    setFiles(prev => [...prev, ...uploadItems]);
  };

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    if (event.target.files) {
      addFiles(Array.from(event.target.files));
    }
  };

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    const droppedFiles = Array.from(e.dataTransfer.files);
    addFiles(droppedFiles);
  }, [files, settings]);

  const removeFile = (id: string) => {
    setFiles(prev => prev.filter(f => f.id !== id));
  };

  const clearAll = () => {
    setFiles([]);
  };

  const handleUpload = async () => {
    const validFiles = files.filter(f => f.status !== 'error');
    if (validFiles.length === 0) return;

    setUploading(true);
    
    try {
      // Update files to uploading status
      setFiles(prev => prev.map(f => 
        f.status === 'pending' ? { ...f, status: 'uploading' as const } : f
      ));

      // Simulate upload progress
      const progressInterval = setInterval(() => {
        setFiles(prev => prev.map(f => {
          if (f.status === 'uploading' && f.progress < 100) {
            const newProgress = Math.min(f.progress + Math.random() * 15, 100);
            return { 
              ...f, 
              progress: newProgress,
              status: newProgress === 100 ? 'completed' : 'uploading'
            };
          }
          return f;
        }));
        
        // Calculate total progress
        setFiles(prev => {
          const totalProgress = prev.reduce((acc, f) => acc + f.progress, 0) / prev.length;
          setTotalProgress(totalProgress);
          return prev;
        });
      }, 200);

      // Wait for all files to complete
      await new Promise(resolve => {
        const checkCompletion = () => {
          setFiles(prev => {
            const allCompleted = prev.every(f => f.status === 'completed' || f.status === 'error');
            if (allCompleted) {
              clearInterval(progressInterval);
              resolve(true);
            }
            return prev;
          });
        };
        
        setTimeout(() => {
          checkCompletion();
          const completionChecker = setInterval(() => {
            checkCompletion();
          }, 100);
          
          setTimeout(() => {
            clearInterval(completionChecker);
            clearInterval(progressInterval);
            resolve(true);
          }, 5000); // Timeout after 5 seconds
        }, 1000);
      });

      await onUpload(validFiles, settings);
      
      // Close dialog after successful upload
      setTimeout(() => {
        onClose();
        setFiles([]);
        setUploading(false);
        setTotalProgress(0);
      }, 1000);

    } catch (error) {
      console.error('Upload failed:', error);
      setFiles(prev => prev.map(f => 
        f.status === 'uploading' ? { ...f, status: 'error', error: 'Upload failed' } : f
      ));
      setUploading(false);
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed': return 'success';
      case 'error': return 'error';
      case 'uploading': return 'info';
      default: return 'default';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed': return <CheckCircleIcon color="success" />;
      case 'error': return <ErrorIcon color="error" />;
      case 'uploading': return <InfoIcon color="info" />;
      default: return <ScheduleIcon color="action" />;
    }
  };

  const totalSize = files.reduce((acc, f) => acc + f.file.size, 0);
  const validFiles = files.filter(f => f.status !== 'error');
  const storageUsagePercent = ((currentStorageUsed + totalSize / (1024 * 1024 * 1024)) / storageQuota) * 100;

  return (
    <Dialog
      open={open}
      onClose={onClose}
      maxWidth="md"
      fullWidth
      PaperProps={{
        sx: { height: '80vh', display: 'flex', flexDirection: 'column' }
      }}
    >
      <DialogTitle>
        <Stack direction="row" alignItems="center" spacing={2}>
          <CloudUploadIcon color="primary" />
          <Box>
            <Typography variant="h6">Upload Files</Typography>
            <Typography variant="body2" color="text.secondary">
              Upload to: {entityName}{targetPath !== '/' ? ` → ${targetPath}` : ''}
            </Typography>
          </Box>
        </Stack>
      </DialogTitle>

      <DialogContent sx={{ display: 'flex', flexDirection: 'column', gap: 2, overflow: 'hidden' }}>
        {/* Storage Usage */}
        <Card variant="outlined">
          <CardContent sx={{ py: 2 }}>
            <Stack direction="row" alignItems="center" spacing={2}>
              <StorageIcon color="action" />
              <Box sx={{ flexGrow: 1 }}>
                <Typography variant="body2" color="text.secondary">
                  Storage Usage: {currentStorageUsed.toFixed(2)}GB / {storageQuota}GB
                </Typography>
                <LinearProgress
                  variant="determinate"
                  value={Math.min(storageUsagePercent, 100)}
                  color={storageUsagePercent > 90 ? 'error' : storageUsagePercent > 75 ? 'warning' : 'primary'}
                  sx={{ mt: 0.5 }}
                />
              </Box>
              <Typography variant="body2" color={storageUsagePercent > 90 ? 'error.main' : 'text.secondary'}>
                {storageUsagePercent.toFixed(1)}%
              </Typography>
            </Stack>
          </CardContent>
        </Card>

        {/* Drop Zone */}
        <Box
          ref={dropZoneRef}
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
          onClick={() => fileInputRef.current?.click()}
          sx={{
            border: '2px dashed',
            borderColor: dragOver ? 'primary.main' : 'divider',
            borderRadius: 2,
            p: 4,
            textAlign: 'center',
            cursor: 'pointer',
            backgroundColor: dragOver ? 'action.hover' : 'transparent',
            transition: 'all 0.2s ease',
          }}
        >
          <input
            type="file"
            multiple
            onChange={handleFileSelect}
            style={{ display: 'none' }}
            ref={fileInputRef}
            accept={allowedTypes?.join(',') || '*/*'}
          />
          
          <CloudUploadIcon sx={{ fontSize: '3rem', color: 'text.secondary', mb: 2 }} />
          <Typography variant="h6" gutterBottom>
            {dragOver ? 'Drop files here' : 'Choose files or drag them here'}
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Maximum file size: {maxFileSize}MB • Supported formats: {settings.allowed_extensions.slice(0, 5).join(', ')}
            {settings.allowed_extensions.length > 5 && ` +${settings.allowed_extensions.length - 5} more`}
          </Typography>
        </Box>

        {/* File List */}
        {files.length > 0 && (
          <Box sx={{ flexGrow: 1, overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
            <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ mb: 1 }}>
              <Typography variant="subtitle2">
                Files ({files.length}) • Total: {formatFileSize(totalSize)}
              </Typography>
              {!uploading && (
                <Button size="small" onClick={clearAll} color="error">
                  Clear All
                </Button>
              )}
            </Stack>

            {uploading && (
              <Box sx={{ mb: 2 }}>
                <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ mb: 1 }}>
                  <Typography variant="body2">
                    Uploading {validFiles.length} files...
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    {Math.round(totalProgress)}%
                  </Typography>
                </Stack>
                <LinearProgress variant="determinate" value={totalProgress} />
              </Box>
            )}

            <Box sx={{ flexGrow: 1, overflow: 'auto' }}>
              <List dense>
                <AnimatePresence>
                  {files.map((item) => (
                    <motion.div
                      key={item.id}
                      initial={{ opacity: 0, height: 0 }}
                      animate={{ opacity: 1, height: 'auto' }}
                      exit={{ opacity: 0, height: 0 }}
                      transition={{ duration: 0.2 }}
                    >
                      <ListItem
                        sx={{
                          border: 1,
                          borderColor: item.status === 'error' ? 'error.main' : 'divider',
                          borderRadius: 1,
                          mb: 1,
                        }}
                      >
                        <ListItemIcon>
                          {item.preview ? (
                            <Box
                              component="img"
                              src={item.preview}
                              sx={{
                                width: 32,
                                height: 32,
                                objectFit: 'cover',
                                borderRadius: 1,
                              }}
                            />
                          ) : (
                            getFileIcon(item.file)
                          )}
                        </ListItemIcon>
                        <ListItemText
                          primary={
                            <Stack direction="row" alignItems="center" spacing={1}>
                              <Typography variant="body2" noWrap sx={{ maxWidth: 200 }}>
                                {item.file.name}
                              </Typography>
                              <Chip
                                label={item.status}
                                size="small"
                                color={getStatusColor(item.status) as any}
                                icon={getStatusIcon(item.status)}
                              />
                            </Stack>
                          }
                          secondary={
                            <Stack spacing={0.5}>
                              <Typography variant="caption" color="text.secondary">
                                {formatFileSize(item.file.size)} • {item.file.type}
                              </Typography>
                              {item.error && (
                                <Typography variant="caption" color="error.main">
                                  {item.error}
                                </Typography>
                              )}
                              {item.status === 'uploading' && (
                                <LinearProgress
                                  variant="determinate"
                                  value={item.progress}
                                  size="small"
                                />
                              )}
                            </Stack>
                          }
                        />
                        <ListItemSecondaryAction>
                          {!uploading && (
                            <IconButton
                              edge="end"
                              onClick={() => removeFile(item.id)}
                              size="small"
                            >
                              <DeleteIcon />
                            </IconButton>
                          )}
                        </ListItemSecondaryAction>
                      </ListItem>
                    </motion.div>
                  ))}
                </AnimatePresence>
              </List>
            </Box>
          </Box>
        )}

        {/* Advanced Settings */}
        <Divider />
        <Box>
          <Button
            startIcon={showAdvancedSettings ? <ExpandLessIcon /> : <ExpandMoreIcon />}
            onClick={() => setShowAdvancedSettings(!showAdvancedSettings)}
            size="small"
          >
            Advanced Settings
          </Button>
          
          <Collapse in={showAdvancedSettings}>
            <Grid container spacing={2} sx={{ mt: 1 }}>
              <Grid item xs={12} sm={6}>
                <FormControlLabel
                  control={
                    <Switch
                      checked={settings.compress_images}
                      onChange={(e) => setSettings(prev => ({ ...prev, compress_images: e.target.checked }))}
                    />
                  }
                  label="Compress Images"
                />
              </Grid>
              <Grid item xs={12} sm={6}>
                <FormControlLabel
                  control={
                    <Switch
                      checked={settings.generate_thumbnails}
                      onChange={(e) => setSettings(prev => ({ ...prev, generate_thumbnails: e.target.checked }))}
                    />
                  }
                  label="Generate Thumbnails"
                />
              </Grid>
              <Grid item xs={12} sm={6}>
                <FormControlLabel
                  control={
                    <Switch
                      checked={settings.virus_scan}
                      onChange={(e) => setSettings(prev => ({ ...prev, virus_scan: e.target.checked }))}
                    />
                  }
                  label="Virus Scan"
                />
              </Grid>
              <Grid item xs={12} sm={6}>
                <FormControlLabel
                  control={
                    <Switch
                      checked={settings.notify_members}
                      onChange={(e) => setSettings(prev => ({ ...prev, notify_members: e.target.checked }))}
                    />
                  }
                  label="Notify Members"
                />
              </Grid>
            </Grid>
          </Collapse>
        </Box>
      </DialogContent>

      <DialogActions sx={{ px: 3, py: 2 }}>
        <Button onClick={onClose} disabled={uploading}>
          Cancel
        </Button>
        <Button
          onClick={handleUpload}
          variant="contained"
          disabled={validFiles.length === 0 || uploading}
          startIcon={uploading ? undefined : <CloudUploadIcon />}
        >
          {uploading ? `Uploading ${validFiles.length} files...` : `Upload ${validFiles.length} files`}
        </Button>
      </DialogActions>
    </Dialog>
  );
};

export default FileUploadDialog;