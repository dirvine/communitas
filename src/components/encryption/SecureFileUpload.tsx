import React, { useState, useCallback, useRef } from 'react';
import {
  Box,
  Paper,
  Typography,
  Button,
  LinearProgress,
  Alert,
  Chip,
  IconButton,
  Stack,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  FormControlLabel,
  Switch,
  TextField,
} from '@mui/material';
import {
  CloudUpload as UploadIcon,
  Security as SecurityIcon,
  Delete as DeleteIcon,
  Visibility as VisibilityIcon,
  FilePresent as FileIcon,
  Lock as LockIcon,
  Key as KeyIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { useEncryption } from '../../contexts/EncryptionContext';
import { EncryptedData } from '../../utils/crypto';
import { invoke } from '@tauri-apps/api/core';

export interface SecureFileInfo {
  id: string;
  name: string;
  size: number;
  type: string;
  encrypted: boolean;
  encryptedData?: EncryptedData;
  keyId?: string;
  uploadedAt: string;
  hash?: string;
}

export interface SecureFileUploadProps {
  onFilesUploaded?: (files: SecureFileInfo[]) => void;
  onError?: (error: string) => void;
  maxFiles?: number;
  maxSizeBytes?: number;
  acceptedTypes?: string[];
  encryptByDefault?: boolean;
  organizationId?: string;
  projectId?: string;
  allowMultiple?: boolean;
  showEncryptionOptions?: boolean;
}

export const SecureFileUpload: React.FC<SecureFileUploadProps> = ({
  onFilesUploaded,
  onError,
  maxFiles = 10,
  maxSizeBytes = 100 * 1024 * 1024, // 100MB
  acceptedTypes,
  encryptByDefault = true,
  organizationId,
  projectId,
  allowMultiple = true,
  showEncryptionOptions = true,
}) => {
  const { state: encryptionState, encryptFile, getOrCreateKey, generateSecureHash } = useEncryption();
  const [files, setFiles] = useState<SecureFileInfo[]>([]);
  const [uploading, setUploading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0);
  const [dragOver, setDragOver] = useState(false);
  const [encryptFiles, setEncryptFiles] = useState(encryptByDefault);
  const [customKeyScope, setCustomKeyScope] = useState('');
  const [showKeyDialog, setShowKeyDialog] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
  }, []);

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    
    const droppedFiles = Array.from(e.dataTransfer.files);
    await processFiles(droppedFiles);
  }, [encryptFiles]);

  const handleFileSelect = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFiles = Array.from(e.target.files || []);
    await processFiles(selectedFiles);
    
    // Clear the input
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  }, [encryptFiles]);

  const processFiles = useCallback(async (fileList: File[]) => {
    if (fileList.length === 0) return;

    // Validate files
    const validFiles = fileList.filter(file => {
      if (acceptedTypes && !acceptedTypes.some(type => file.type.includes(type))) {
        onError?.(`File type ${file.type} not accepted`);
        return false;
      }
      if (file.size > maxSizeBytes) {
        onError?.(`File ${file.name} is too large (max ${maxSizeBytes / 1024 / 1024}MB)`);
        return false;
      }
      return true;
    });

    if (files.length + validFiles.length > maxFiles) {
      onError?.(`Cannot upload more than ${maxFiles} files`);
      return;
    }

    setUploading(true);
    setUploadProgress(0);

    try {
      const processedFiles: SecureFileInfo[] = [];
      
      for (let i = 0; i < validFiles.length; i++) {
        const file = validFiles[i];
        setUploadProgress((i / validFiles.length) * 100);

        const fileId = crypto.randomUUID();
        let encryptedData: EncryptedData | undefined;
        let keyId: string | undefined;
        let hash: string | undefined;

        if (encryptFiles && encryptionState.isInitialized) {
          // Get or create encryption key for the current scope
          const scope = projectId ? `project:${projectId}` : 
                      organizationId ? `organization:${organizationId}` :
                      customKeyScope || undefined;
          
          const encryptionKey = await getOrCreateKey('file', scope);
          keyId = encryptionKey.id;
          
          // Encrypt the file
          encryptedData = await encryptFile(file, keyId);
          hash = await generateSecureHash(encryptedData.data);
        } else {
          // Generate hash for unencrypted file
          const buffer = await file.arrayBuffer();
          hash = await generateSecureHash(buffer);
        }

        // Store the file (encrypted or unencrypted) via Tauri
        await invoke('store_file', {
          fileId,
          fileName: file.name,
          fileType: file.type,
          fileSize: file.size,
          encryptedData: encryptedData ? {
            data: Array.from(new Uint8Array(encryptedData.data)),
            iv: Array.from(new Uint8Array(encryptedData.iv)),
            algorithm: encryptedData.algorithm,
            keyId: encryptedData.keyId,
            timestamp: encryptedData.timestamp,
            version: encryptedData.version,
          } : null,
          keyId,
          hash,
          organizationId,
          projectId,
        });

        const secureFileInfo: SecureFileInfo = {
          id: fileId,
          name: file.name,
          size: file.size,
          type: file.type,
          encrypted: !!encryptedData,
          encryptedData,
          keyId,
          uploadedAt: new Date().toISOString(),
          hash,
        };

        processedFiles.push(secureFileInfo);
      }

      setFiles(prev => [...prev, ...processedFiles]);
      onFilesUploaded?.(processedFiles);
      setUploadProgress(100);

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to upload files';
      onError?.(errorMessage);
      console.error('File upload failed:', error);
    } finally {
      setUploading(false);
      setTimeout(() => setUploadProgress(0), 1000);
    }
  }, [files, encryptFiles, encryptionState.isInitialized, maxFiles, maxSizeBytes, acceptedTypes]);

  const removeFile = useCallback((fileId: string) => {
    setFiles(prev => prev.filter(f => f.id !== fileId));
  }, []);

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
    <Box>
      {/* Upload Area */}
      <Paper
        sx={{
          p: 3,
          border: dragOver ? '2px dashed' : '2px solid',
          borderColor: dragOver ? 'primary.main' : 'grey.300',
          bgcolor: dragOver ? 'primary.50' : 'background.paper',
          transition: 'all 0.2s ease',
          cursor: 'pointer',
          position: 'relative',
          overflow: 'hidden',
        }}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        onClick={() => fileInputRef.current?.click()}
      >
        <input
          ref={fileInputRef}
          type="file"
          hidden
          multiple={allowMultiple}
          accept={acceptedTypes?.join(',')}
          onChange={handleFileSelect}
        />

        <Stack alignItems="center" spacing={2}>
          <motion.div
            animate={{
              scale: dragOver ? 1.1 : 1,
              rotate: dragOver ? 5 : 0,
            }}
            transition={{ type: "spring", stiffness: 300, damping: 25 }}
          >
            <UploadIcon 
              sx={{ 
                fontSize: 48, 
                color: dragOver ? 'primary.main' : 'text.secondary' 
              }} 
            />
          </motion.div>

          <Typography variant="h6" textAlign="center">
            {dragOver ? 'Drop files here' : 'Upload Files'}
          </Typography>

          <Typography variant="body2" color="text.secondary" textAlign="center">
            Drag & drop files here, or click to select
          </Typography>

          <Stack direction="row" spacing={2} alignItems="center">
            {encryptionState.isInitialized && (
              <Chip
                icon={<SecurityIcon />}
                label={encryptFiles ? "Encrypted" : "Unencrypted"}
                color={encryptFiles ? "success" : "warning"}
                size="small"
              />
            )}
            
            <Typography variant="caption" color="text.secondary">
              Max {formatFileSize(maxSizeBytes)} per file
            </Typography>
          </Stack>
        </Stack>

        {uploading && (
          <Box sx={{ position: 'absolute', bottom: 0, left: 0, right: 0 }}>
            <LinearProgress 
              variant="determinate" 
              value={uploadProgress}
              sx={{ height: 6 }}
            />
          </Box>
        )}
      </Paper>

      {/* Encryption Options */}
      {showEncryptionOptions && encryptionState.isInitialized && (
        <Box sx={{ mt: 2 }}>
          <Stack direction="row" spacing={2} alignItems="center">
            <FormControlLabel
              control={
                <Switch
                  checked={encryptFiles}
                  onChange={(e) => setEncryptFiles(e.target.checked)}
                  disabled={uploading}
                />
              }
              label="Encrypt files"
            />

            <Button
              size="small"
              startIcon={<KeyIcon />}
              onClick={() => setShowKeyDialog(true)}
              disabled={!encryptFiles || uploading}
            >
              Key Settings
            </Button>
          </Stack>
        </Box>
      )}

      {/* Uploaded Files List */}
      <AnimatePresence>
        {files.length > 0 && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -20 }}
          >
            <Box sx={{ mt: 3 }}>
              <Typography variant="h6" gutterBottom>
                Uploaded Files ({files.length})
              </Typography>
              
              <Stack spacing={1}>
                {files.map((file) => (
                  <Paper key={file.id} variant="outlined" sx={{ p: 2 }}>
                    <Stack direction="row" alignItems="center" spacing={2}>
                      <FileIcon color="primary" />
                      
                      <Box sx={{ flex: 1 }}>
                        <Typography variant="body1" fontWeight={500}>
                          {file.name}
                        </Typography>
                        <Typography variant="body2" color="text.secondary">
                          {formatFileSize(file.size)} • {file.type}
                        </Typography>
                      </Box>

                      <Stack direction="row" spacing={1}>
                        {file.encrypted && (
                          <Chip
                            icon={<LockIcon />}
                            label="Encrypted"
                            color="success"
                            size="small"
                          />
                        )}

                        <IconButton
                          size="small"
                          onClick={() => removeFile(file.id)}
                          color="error"
                        >
                          <DeleteIcon />
                        </IconButton>
                      </Stack>
                    </Stack>
                  </Paper>
                ))}
              </Stack>
            </Box>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Key Settings Dialog */}
      <Dialog open={showKeyDialog} onClose={() => setShowKeyDialog(false)}>
        <DialogTitle>Encryption Key Settings</DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ mt: 1 }}>
            <Alert severity="info">
              Choose the scope for file encryption keys. Files encrypted with the same scope can be decrypted by users with access to that scope.
            </Alert>

            <TextField
              fullWidth
              label="Custom Key Scope"
              value={customKeyScope}
              onChange={(e) => setCustomKeyScope(e.target.value)}
              placeholder="e.g., confidential-project"
              helperText="Leave empty to use default organization/project scope"
            />

            <Box>
              <Typography variant="body2" color="text.secondary">
                Current scope: {
                  projectId ? `project:${projectId}` :
                  organizationId ? `organization:${organizationId}` :
                  customKeyScope || 'user-default'
                }
              </Typography>
            </Box>
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShowKeyDialog(false)}>
            Close
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default SecureFileUpload;