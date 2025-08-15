import React, { useState, useCallback, useRef, useEffect } from 'react';
import {
  Box,
  Paper,
  Typography,
  TextField,
  Button,
  IconButton,
  Divider,
  Chip,
  Stack,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Switch,
  FormControlLabel,
  Tabs,
  Tab,
  useTheme,
  alpha,
  Avatar,
  Badge,
  Tooltip,
  CircularProgress,
  Alert,
  Snackbar,
  Menu,
  ListItemIcon,
  ListItemText,
} from '@mui/material';
import {
  FormatBold as BoldIcon,
  FormatItalic as ItalicIcon,
  FormatListBulleted as ListIcon,
  FormatListNumbered as NumberedListIcon,
  FormatQuote as QuoteIcon,
  Code as CodeIcon,
  Link as LinkIcon,
  Image as ImageIcon,
  VideoFile as VideoIcon,
  AttachFile as FileIcon,
  Preview as PreviewIcon,
  Edit as EditIcon,
  Save as SaveIcon,
  CloudSync as SyncIcon,
  People as CollabIcon,
  History as HistoryIcon,
  Share as ShareIcon,
  Home as HomeIcon,
  FolderOpen as FolderIcon,
  Add as AddIcon,
  MoreVert as MoreIcon,
  Security as SecurityIcon,
  CloudDownload as DownloadIcon,
  CloudUpload as UploadIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { useResponsive, ResponsiveContainer } from '../responsive';

// Enhanced universal content types
export interface UniversalContent {
  id: string;
  path: string; // Full path like "ocean-forest-mountain-star/projects/readme.md"
  filename: string; // Just the filename like "readme.md"
  title: string;
  content: string;
  contentType: 'markdown' | 'text' | 'image' | 'video' | 'document';
  tags: string[];
  category: string;
  status: 'draft' | 'published' | 'private' | 'collaborative';
  owner: FourWordIdentity;
  collaborators: CollaboratorInfo[];
  version: number;
  encryption: EncryptionInfo;
  storage: StorageInfo;
  metadata: ContentMetadata;
  createdAt: string;
  updatedAt: string;
  lastSyncAt?: string;
}

// Four-word identity for human-readable addresses
export interface FourWordIdentity {
  words: [string, string, string, string];
  address: string; // Combined four words: "ocean-forest-mountain-star"
  publicKey: string;
  displayName?: string;
  avatar?: string;
  isOnline?: boolean;
}

// Collaborator information for shared editing
export interface CollaboratorInfo {
  identity: FourWordIdentity;
  role: 'owner' | 'editor' | 'viewer';
  lastActive: string;
  cursorPosition?: number;
  isTyping?: boolean;
  permissions: CollaborationPermissions;
}

export interface CollaborationPermissions {
  canEdit: boolean;
  canInvite: boolean;
  canDelete: boolean;
  canShare: boolean;
  canViewHistory: boolean;
}

// Encryption and storage information
export interface EncryptionInfo {
  isEncrypted: boolean;
  algorithm: 'AES-256-GCM' | 'none';
  keyDerivation: 'file-hash' | 'user-key';
  integrityHash: string;
}

export interface StorageInfo {
  distributionStrategy: 'local-only' | 'group-redundant' | 'dht-backup';
  reedSolomon: {
    dataShards: number;
    parityShards: number;
    availabilityRatio: number;
  };
  shardSize: number; // 1MB blocks as specified
  replicationFactor: number;
  witnessCount: number;
}

export interface ContentMetadata {
  size: number;
  mimeType: string;
  checksum: string;
  compressionRatio?: number;
  lastBackup?: string;
  syncStatus: 'synced' | 'pending' | 'conflict' | 'error';
  conflictResolution?: 'manual' | 'auto' | 'latest-wins';
}

// Enhanced toolbar button interface with collaboration support
interface ToolbarButton {
  icon: React.ReactNode;
  label: string;
  action: () => void;
  shortcut?: string;
  disabled?: boolean;
  requiresPermission?: keyof CollaborationPermissions;
}

// Universal Markdown Editor props
interface UniversalMarkdownEditorProps {
  initialContent?: Partial<UniversalContent>;
  currentUser: FourWordIdentity;
  onSave?: (content: UniversalContent) => Promise<void>;
  onShare?: (content: UniversalContent, recipients: FourWordIdentity[]) => Promise<void>;
  onSync?: (content: UniversalContent) => Promise<void>;
  onUploadFile?: (file: File) => Promise<string>; // Returns file URL/reference
  onInviteCollaborator?: (contentId: string, identity: FourWordIdentity) => Promise<void>;
  readonly?: boolean;
  collaborativeMode?: boolean;
  homeDirectory?: boolean; // Whether this is editing home.md (acts as index.html)
  entityType?: 'person' | 'organization' | 'project' | 'group' | 'channel';
}

export const UniversalMarkdownEditor: React.FC<UniversalMarkdownEditorProps> = ({
  initialContent,
  currentUser,
  onSave,
  onShare,
  onSync,
  onUploadFile,
  onInviteCollaborator,
  readonly = false,
  collaborativeMode = false,
  homeDirectory = false,
  entityType = 'person',
}) => {
  const theme = useTheme();
  const { isMobile } = useResponsive();
  const textAreaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  
  // State management
  const [activeTab, setActiveTab] = useState<'edit' | 'preview' | 'collaborate'>('edit');
  const [shareDialogOpen, setShareDialogOpen] = useState(false);
  const [collaboratorMenuAnchor, setCollaboratorMenuAnchor] = useState<null | HTMLElement>(null);
  const [syncStatus, setSyncStatus] = useState<'idle' | 'syncing' | 'error'>('idle');
  const [snackbar, setSnackbar] = useState<{ open: boolean; message: string; severity: 'success' | 'error' | 'info' }>({
    open: false,
    message: '',
    severity: 'info',
  });

  // Content state with enhanced metadata
  const [content, setContent] = useState<UniversalContent>({
    id: initialContent?.id || crypto.randomUUID(),
    path: initialContent?.path || `${currentUser.address}/${homeDirectory ? 'home.md' : 'untitled.md'}`,
    filename: initialContent?.filename || (homeDirectory ? 'home.md' : 'untitled.md'),
    title: initialContent?.title || (homeDirectory ? `Welcome to ${currentUser.displayName || currentUser.address}` : 'Untitled Document'),
    content: initialContent?.content || (homeDirectory ? generateHomeTemplate(currentUser, entityType) : ''),
    contentType: 'markdown',
    tags: initialContent?.tags || [],
    category: initialContent?.category || 'general',
    status: initialContent?.status || 'draft',
    owner: currentUser,
    collaborators: initialContent?.collaborators || [],
    version: initialContent?.version || 1,
    encryption: initialContent?.encryption || {
      isEncrypted: true,
      algorithm: 'AES-256-GCM',
      keyDerivation: 'file-hash',
      integrityHash: '',
    },
    storage: initialContent?.storage || {
      distributionStrategy: 'group-redundant',
      reedSolomon: {
        dataShards: 8,
        parityShards: 4,
        availabilityRatio: 0.6, // 60% availability requirement
      },
      shardSize: 1024 * 1024, // 1MB blocks as specified
      replicationFactor: 3,
      witnessCount: 5,
    },
    metadata: initialContent?.metadata || {
      size: 0,
      mimeType: 'text/markdown',
      checksum: '',
      syncStatus: 'synced',
    },
    createdAt: initialContent?.createdAt || new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  });

  // Current user permissions
  const userPermissions: CollaborationPermissions = content.owner.address === currentUser.address 
    ? { canEdit: true, canInvite: true, canDelete: true, canShare: true, canViewHistory: true }
    : content.collaborators.find(c => c.identity.address === currentUser.address)?.permissions || 
      { canEdit: false, canInvite: false, canDelete: false, canShare: false, canViewHistory: false };

  // Auto-save functionality
  useEffect(() => {
    if (content.content && userPermissions.canEdit) {
      const autoSaveTimer = setTimeout(async () => {
        try {
          setSyncStatus('syncing');
          await onSave?.(content);
          setSyncStatus('idle');
          setContent(prev => ({ ...prev, lastSyncAt: new Date().toISOString() }));
        } catch (error) {
          setSyncStatus('error');
          console.error('Auto-save failed:', error);
        }
      }, 2000); // Auto-save after 2 seconds of inactivity

      return () => clearTimeout(autoSaveTimer);
    }
  }, [content.content, content.title, userPermissions.canEdit, onSave]);

  // Insert text at cursor position with collaboration awareness
  const insertText = useCallback((text: string, notifyCollaborators = true) => {
    if (!textAreaRef.current || !userPermissions.canEdit) return;

    const textarea = textAreaRef.current;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const currentContent = content.content;

    const newContent = currentContent.substring(0, start) + text + currentContent.substring(end);
    
    setContent(prev => ({
      ...prev,
      content: newContent,
      updatedAt: new Date().toISOString(),
      version: prev.version + 1,
      metadata: {
        ...prev.metadata,
        size: newContent.length,
        syncStatus: 'pending',
      },
    }));

    // Move cursor after inserted text
    setTimeout(() => {
      textarea.setSelectionRange(start + text.length, start + text.length);
      textarea.focus();
    }, 0);

    // TODO: Notify collaborators of change in real implementation
    if (notifyCollaborators && collaborativeMode) {
      // This would send real-time updates to other collaborators
      console.log('Notifying collaborators of change at position:', start);
    }
  }, [content.content, userPermissions.canEdit, collaborativeMode]);

  // Wrap selected text with markdown formatting
  const wrapText = useCallback((prefix: string, suffix?: string) => {
    if (!textAreaRef.current || !userPermissions.canEdit) return;

    const textarea = textAreaRef.current;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const selectedText = content.content.substring(start, end);
    const wrappedText = `${prefix}${selectedText}${suffix || prefix}`;

    insertText(wrappedText);
  }, [content.content, insertText, userPermissions.canEdit]);

  // Handle file upload for images, videos, and documents
  const handleFileUpload = useCallback(async (file: File) => {
    if (!onUploadFile || !userPermissions.canEdit) return;

    try {
      const fileUrl = await onUploadFile(file);
      const isImage = file.type.startsWith('image/');
      const isVideo = file.type.startsWith('video/');
      
      let markdownText = '';
      if (isImage) {
        markdownText = `![${file.name}](${fileUrl})`;
      } else if (isVideo) {
        markdownText = `<video controls>\n  <source src="${fileUrl}" type="${file.type}">\n  Your browser does not support the video tag.\n</video>`;
      } else {
        markdownText = `[${file.name}](${fileUrl})`;
      }
      
      insertText(`\n${markdownText}\n`);
      setSnackbar({ open: true, message: `${file.name} uploaded successfully`, severity: 'success' });
    } catch (error) {
      console.error('File upload failed:', error);
      setSnackbar({ open: true, message: 'File upload failed', severity: 'error' });
    }
  }, [onUploadFile, insertText, userPermissions.canEdit]);

  // Enhanced toolbar buttons with collaboration features
  const toolbarButtons: ToolbarButton[] = [
    {
      icon: <BoldIcon />,
      label: 'Bold',
      action: () => wrapText('**'),
      shortcut: 'Ctrl+B',
      requiresPermission: 'canEdit',
    },
    {
      icon: <ItalicIcon />,
      label: 'Italic',
      action: () => wrapText('*'),
      shortcut: 'Ctrl+I',
      requiresPermission: 'canEdit',
    },
    {
      icon: <CodeIcon />,
      label: 'Code',
      action: () => wrapText('`'),
      shortcut: 'Ctrl+`',
      requiresPermission: 'canEdit',
    },
    {
      icon: <LinkIcon />,
      label: 'Link',
      action: () => insertText('[Link Text](ocean-forest-mountain-star)'), // Four-word address example
      requiresPermission: 'canEdit',
    },
    {
      icon: <ImageIcon />,
      label: 'Image',
      action: () => fileInputRef.current?.click(),
      requiresPermission: 'canEdit',
    },
    {
      icon: <VideoIcon />,
      label: 'Video',
      action: () => insertText('<video controls>\n  <source src="video-url" type="video/mp4">\n</video>'),
      requiresPermission: 'canEdit',
    },
    {
      icon: <FileIcon />,
      label: 'Attach File',
      action: () => fileInputRef.current?.click(),
      requiresPermission: 'canEdit',
    },
    {
      icon: <ListIcon />,
      label: 'Bullet List',
      action: () => insertText('\n- List item'),
      requiresPermission: 'canEdit',
    },
    {
      icon: <NumberedListIcon />,
      label: 'Numbered List',
      action: () => insertText('\n1. List item'),
      requiresPermission: 'canEdit',
    },
    {
      icon: <QuoteIcon />,
      label: 'Quote',
      action: () => insertText('\n> Quote text'),
      requiresPermission: 'canEdit',
    },
  ];

  // Handle keyboard shortcuts with collaboration awareness
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (!userPermissions.canEdit) return;

    if (e.ctrlKey || e.metaKey) {
      switch (e.key) {
        case 'b':
          e.preventDefault();
          wrapText('**');
          break;
        case 'i':
          e.preventDefault();
          wrapText('*');
          break;
        case '`':
          e.preventDefault();
          wrapText('`');
          break;
        case 's':
          e.preventDefault();
          handleSave();
          break;
      }
    }
  }, [wrapText, userPermissions.canEdit]);

  // Save content with enhanced metadata
  const handleSave = useCallback(async () => {
    if (!onSave || !userPermissions.canEdit) return;

    try {
      setSyncStatus('syncing');
      const updatedContent = {
        ...content,
        metadata: {
          ...content.metadata,
          size: content.content.length,
          checksum: await generateChecksum(content.content),
          syncStatus: 'synced' as const,
        },
      };
      
      await onSave(updatedContent);
      setContent(updatedContent);
      setSyncStatus('idle');
      setSnackbar({ open: true, message: 'Content saved successfully', severity: 'success' });
    } catch (error) {
      setSyncStatus('error');
      setSnackbar({ open: true, message: 'Save failed', severity: 'error' });
    }
  }, [content, onSave, userPermissions.canEdit]);

  // Share content with other four-word identities
  const handleShare = useCallback(async (recipients: FourWordIdentity[]) => {
    if (!onShare || !userPermissions.canShare) return;

    try {
      await onShare(content, recipients);
      setShareDialogOpen(false);
      setSnackbar({ open: true, message: 'Content shared successfully', severity: 'success' });
    } catch (error) {
      setSnackbar({ open: true, message: 'Sharing failed', severity: 'error' });
    }
  }, [content, onShare, userPermissions.canShare]);

  // Enhanced markdown preview with four-word address linking
  const renderPreview = useCallback((markdown: string) => {
    return markdown
      // Headers
      .replace(/^### (.+$)/gim, '<h3>$1</h3>')
      .replace(/^## (.+$)/gim, '<h2>$1</h2>')
      .replace(/^# (.+$)/gim, '<h1>$1</h1>')
      // Text formatting
      .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
      .replace(/\*(.+?)\*/g, '<em>$1</em>')
      .replace(/`(.+?)`/g, '<code>$1</code>')
      // Four-word address links (special handling)
      .replace(/\[([^\]]+)\]\(([a-z]+-[a-z]+-[a-z]+-[a-z]+(?:\/[^)]*)?)\)/g, 
               '<a href="communitas://$2" class="four-word-link">$1</a>')
      // Regular links
      .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>')
      // Images
      .replace(/!\[([^\]]*)\]\(([^)]+)\)/g, '<img src="$2" alt="$1" style="max-width: 100%; height: auto;" />')
      // Line breaks
      .replace(/\n/g, '<br>');
  }, []);

  return (
    <ResponsiveContainer maxWidth="xl">
      <Paper
        component={motion.div}
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        sx={{
          overflow: 'hidden',
          height: 'calc(100vh - 100px)',
          display: 'flex',
          flexDirection: 'column',
          border: homeDirectory ? `2px solid ${theme.palette.primary.main}` : undefined,
        }}
      >
        {/* Enhanced Header with Four-Word Address */}
        <Box
          sx={{
            p: 2,
            borderBottom: `1px solid ${theme.palette.divider}`,
            background: homeDirectory 
              ? `linear-gradient(45deg, ${alpha(theme.palette.primary.main, 0.1)}, ${alpha(theme.palette.secondary.main, 0.1)})`
              : alpha(theme.palette.primary.main, 0.02),
          }}
        >
          <Stack spacing={2}>
            {/* Path and Identity */}
            <Stack direction="row" alignItems="center" spacing={2}>
              {homeDirectory && <HomeIcon color="primary" />}
              <Typography variant="body2" color="text.secondary" sx={{ fontFamily: 'monospace' }}>
                {content.path}
              </Typography>
              <Chip 
                size="small" 
                label={content.status} 
                color={content.status === 'published' ? 'success' : 'default'}
              />
              {content.encryption.isEncrypted && (
                <Tooltip title="End-to-end encrypted">
                  <SecurityIcon fontSize="small" color="primary" />
                </Tooltip>
              )}
            </Stack>

            {/* Title and Actions */}
            <Stack
              direction={isMobile ? 'column' : 'row'}
              alignItems={isMobile ? 'stretch' : 'center'}
              justifyContent="space-between"
              spacing={2}
            >
              <TextField
                fullWidth
                variant="outlined"
                placeholder={homeDirectory ? "Your Digital Identity Title" : "Document Title"}
                value={content.title}
                onChange={(e) => setContent(prev => ({ ...prev, title: e.target.value }))}
                disabled={readonly || !userPermissions.canEdit}
                sx={{
                  '& .MuiOutlinedInput-input': {
                    fontSize: isMobile ? '1.25rem' : '1.5rem',
                    fontWeight: 600,
                    padding: isMobile ? '12px' : '16px',
                  },
                }}
              />

              <Stack direction="row" spacing={1} alignItems="center">
                {/* Collaborators */}
                {collaborativeMode && content.collaborators.length > 0 && (
                  <Stack direction="row" spacing={0.5}>
                    {content.collaborators.slice(0, 3).map((collab) => (
                      <Tooltip key={collab.identity.address} title={`${collab.identity.displayName || collab.identity.address} (${collab.role})`}>
                        <Badge
                          variant="dot"
                          color={collab.identity.isOnline ? 'success' : 'default'}
                          overlap="circular"
                          anchorOrigin={{
                            vertical: 'bottom',
                            horizontal: 'right',
                          }}
                        >
                          <Avatar 
                            src={collab.identity.avatar}
                            sx={{ width: 24, height: 24, fontSize: '0.75rem' }}
                          >
                            {collab.identity.displayName?.[0] || collab.identity.words[0][0].toUpperCase()}
                          </Avatar>
                        </Badge>
                      </Tooltip>
                    ))}
                    {content.collaborators.length > 3 && (
                      <Typography variant="caption" color="text.secondary">
                        +{content.collaborators.length - 3}
                      </Typography>
                    )}
                  </Stack>
                )}

                {/* Sync Status */}
                <Tooltip title={`Last synced: ${content.lastSyncAt ? new Date(content.lastSyncAt).toLocaleTimeString() : 'Never'}`}>
                  <IconButton size="small">
                    {syncStatus === 'syncing' ? (
                      <CircularProgress size={16} />
                    ) : (
                      <SyncIcon 
                        fontSize="small" 
                        color={content.metadata.syncStatus === 'synced' ? 'success' : 'warning'}
                      />
                    )}
                  </IconButton>
                </Tooltip>

                {/* Action Buttons */}
                {userPermissions.canShare && (
                  <Button
                    variant="outlined"
                    startIcon={<ShareIcon />}
                    onClick={() => setShareDialogOpen(true)}
                    size="small"
                    disabled={readonly}
                  >
                    Share
                  </Button>
                )}

                <Button
                  variant={userPermissions.canEdit ? "contained" : "outlined"}
                  startIcon={<SaveIcon />}
                  onClick={handleSave}
                  disabled={readonly || !userPermissions.canEdit || syncStatus === 'syncing'}
                  size="small"
                >
                  {syncStatus === 'syncing' ? 'Saving...' : 'Save'}
                </Button>
              </Stack>
            </Stack>

            {homeDirectory && (
              <Alert severity="info" variant="outlined">
                <Typography variant="body2">
                  This is your home page - it serves as your digital identity's front page, like index.html for websites. 
                  People visiting <strong>{currentUser.address}</strong> will see this content first.
                </Typography>
              </Alert>
            )}
          </Stack>
        </Box>

        {/* Enhanced Tabs */}
        <Box sx={{ borderBottom: `1px solid ${theme.palette.divider}` }}>
          <Tabs
            value={activeTab}
            onChange={(_, newValue) => setActiveTab(newValue)}
            variant={isMobile ? 'fullWidth' : 'standard'}
          >
            <Tab
              label="Edit"
              value="edit"
              icon={<EditIcon />}
              iconPosition="start"
              disabled={!userPermissions.canEdit}
            />
            <Tab
              label="Preview"
              value="preview"
              icon={<PreviewIcon />}
              iconPosition="start"
            />
            {collaborativeMode && (
              <Tab
                label="Collaborate"
                value="collaborate"
                icon={<CollabIcon />}
                iconPosition="start"
              />
            )}
          </Tabs>
        </Box>

        {/* Edit Tab */}
        {activeTab === 'edit' && (
          <>
            {/* Enhanced Toolbar with File Support */}
            <Box
              sx={{
                p: 1,
                borderBottom: `1px solid ${theme.palette.divider}`,
                background: theme.palette.background.paper,
              }}
            >
              <Stack
                direction="row"
                spacing={0.5}
                flexWrap="wrap"
                gap={0.5}
                alignItems="center"
              >
                {toolbarButtons.map((button, index) => (
                  <IconButton
                    key={index}
                    size="small"
                    onClick={button.action}
                    disabled={readonly || (button.requiresPermission && !userPermissions[button.requiresPermission])}
                    title={`${button.label} ${button.shortcut ? `(${button.shortcut})` : ''}`}
                    sx={{
                      color: theme.palette.text.secondary,
                      '&:hover': {
                        color: theme.palette.primary.main,
                        backgroundColor: alpha(theme.palette.primary.main, 0.1),
                      },
                    }}
                  >
                    {button.icon}
                  </IconButton>
                ))}

                <Divider orientation="vertical" sx={{ height: 24, mx: 1 }} />
                
                {/* Quick Four-Word Address Insert */}
                <Tooltip title="Insert four-word address link">
                  <Button
                    size="small"
                    variant="outlined"
                    onClick={() => insertText('[Link Text](ocean-forest-mountain-star)')}
                    disabled={!userPermissions.canEdit}
                    sx={{ minWidth: 'auto', px: 1, fontSize: '0.75rem' }}
                  >
                    Four-Word Link
                  </Button>
                </Tooltip>
              </Stack>
            </Box>

            {/* Content Editor */}
            <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
              <TextField
                multiline
                fullWidth
                placeholder={homeDirectory 
                  ? "Welcome to your digital space! This is your home page - write about yourself, your projects, or whatever represents you best..."
                  : "Start writing your content in Markdown..."
                }
                value={content.content}
                onChange={(e) => setContent(prev => ({ 
                  ...prev, 
                  content: e.target.value,
                  updatedAt: new Date().toISOString(),
                  version: prev.version + 1,
                }))}
                onKeyDown={handleKeyDown}
                inputRef={textAreaRef}
                disabled={readonly || !userPermissions.canEdit}
                InputProps={{
                  sx: {
                    height: '100%',
                    alignItems: 'stretch',
                    '& textarea': {
                      height: '100% !important',
                      resize: 'none',
                      fontFamily: 'Monaco, Consolas, monospace',
                      fontSize: '14px',
                      lineHeight: 1.6,
                    },
                  },
                }}
                sx={{
                  height: '100%',
                  '& .MuiOutlinedInput-root': {
                    height: '100%',
                    borderRadius: 0,
                    border: 'none',
                    '& fieldset': {
                      border: 'none',
                    },
                  },
                }}
              />
            </Box>
          </>
        )}

        {/* Enhanced Preview Tab */}
        {activeTab === 'preview' && (
          <Box
            sx={{
              flex: 1,
              p: 3,
              overflow: 'auto',
              '& h1': {
                fontSize: '2.5rem',
                fontWeight: 700,
                mb: 3,
                color: theme.palette.text.primary,
                borderBottom: `2px solid ${theme.palette.primary.main}`,
                pb: 1,
              },
              '& h2': {
                fontSize: '2rem',
                fontWeight: 600,
                mb: 2,
                mt: 3,
                color: theme.palette.text.primary,
              },
              '& h3': {
                fontSize: '1.5rem',
                fontWeight: 600,
                mb: 1.5,
                mt: 2,
                color: theme.palette.text.primary,
              },
              '& p': {
                mb: 2,
                lineHeight: 1.8,
                color: theme.palette.text.primary,
                fontSize: '1.1rem',
              },
              '& code': {
                backgroundColor: alpha(theme.palette.primary.main, 0.1),
                color: theme.palette.primary.main,
                padding: '2px 6px',
                borderRadius: 4,
                fontFamily: 'Monaco, Consolas, monospace',
                fontSize: '0.9rem',
              },
              '& .four-word-link': {
                color: theme.palette.secondary.main,
                textDecoration: 'underline',
                fontWeight: 600,
                '&:hover': {
                  color: theme.palette.secondary.dark,
                },
              },
              '& img, & video': {
                maxWidth: '100%',
                height: 'auto',
                borderRadius: theme.shape.borderRadius,
                boxShadow: theme.shadows[2],
                mb: 2,
              },
              '& strong': {
                fontWeight: 600,
                color: theme.palette.text.primary,
              },
              '& em': {
                fontStyle: 'italic',
                color: theme.palette.text.secondary,
              },
            }}
          >
            <Typography variant="h3" gutterBottom>
              {content.title}
            </Typography>
            
            <Stack direction="row" spacing={2} mb={3}>
              <Typography variant="body2" color="text.secondary">
                By {content.owner.displayName || content.owner.address}
              </Typography>
              <Typography variant="body2" color="text.secondary">
                Last updated: {new Date(content.updatedAt).toLocaleDateString()}
              </Typography>
              {homeDirectory && (
                <Chip 
                  size="small" 
                  label={`${entityType} identity page`}
                  color="primary"
                  variant="outlined"
                />
              )}
            </Stack>
            
            <Box
              dangerouslySetInnerHTML={{
                __html: renderPreview(content.content || 'Start writing to see the preview...'),
              }}
            />
          </Box>
        )}

        {/* Collaborate Tab */}
        {activeTab === 'collaborate' && collaborativeMode && (
          <Box sx={{ flex: 1, p: 3, overflow: 'auto' }}>
            <Typography variant="h5" gutterBottom>
              Collaboration
            </Typography>
            
            <Stack spacing={3}>
              {/* Current Collaborators */}
              <Box>
                <Typography variant="h6" gutterBottom>
                  Collaborators ({content.collaborators.length})
                </Typography>
                <Stack spacing={2}>
                  {content.collaborators.map((collab) => (
                    <Stack key={collab.identity.address} direction="row" alignItems="center" spacing={2}>
                      <Badge
                        variant="dot"
                        color={collab.identity.isOnline ? 'success' : 'default'}
                        overlap="circular"
                        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                      >
                        <Avatar src={collab.identity.avatar}>
                          {collab.identity.displayName?.[0] || collab.identity.words[0][0].toUpperCase()}
                        </Avatar>
                      </Badge>
                      <Box sx={{ flex: 1 }}>
                        <Typography variant="subtitle1">
                          {collab.identity.displayName || collab.identity.address}
                        </Typography>
                        <Typography variant="body2" color="text.secondary">
                          {collab.identity.address} • {collab.role} • Last active: {new Date(collab.lastActive).toLocaleTimeString()}
                        </Typography>
                      </Box>
                      {collab.isTyping && (
                        <Chip size="small" label="Typing..." color="primary" variant="outlined" />
                      )}
                    </Stack>
                  ))}
                </Stack>
              </Box>

              {/* Invite New Collaborator */}
              {userPermissions.canInvite && (
                <Box>
                  <Typography variant="h6" gutterBottom>
                    Invite Collaborator
                  </Typography>
                  <TextField
                    fullWidth
                    placeholder="Enter four-word address (e.g., mountain-river-sun-cloud)"
                    variant="outlined"
                    InputProps={{
                      endAdornment: (
                        <Button variant="contained" size="small">
                          Invite
                        </Button>
                      ),
                    }}
                  />
                </Box>
              )}

              {/* Storage Information */}
              <Box>
                <Typography variant="h6" gutterBottom>
                  Storage & Security
                </Typography>
                <Stack spacing={1}>
                  <Typography variant="body2">
                    <strong>Distribution:</strong> {content.storage.distributionStrategy}
                  </Typography>
                  <Typography variant="body2">
                    <strong>Reed-Solomon:</strong> {content.storage.reedSolomon.dataShards} data + {content.storage.reedSolomon.parityShards} parity shards 
                    ({Math.round(content.storage.reedSolomon.availabilityRatio * 100)}% availability)
                  </Typography>
                  <Typography variant="body2">
                    <strong>Encryption:</strong> {content.encryption.algorithm} with {content.encryption.keyDerivation}
                  </Typography>
                  <Typography variant="body2">
                    <strong>Size:</strong> {(content.metadata.size / 1024).toFixed(2)} KB across {Math.ceil(content.metadata.size / content.storage.shardSize)} shards
                  </Typography>
                </Stack>
              </Box>
            </Stack>
          </Box>
        )}

        {/* Hidden File Input */}
        <input
          ref={fileInputRef}
          type="file"
          accept="image/*,video/*,.pdf,.doc,.docx,.txt"
          style={{ display: 'none' }}
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (file) handleFileUpload(file);
          }}
        />
      </Paper>

      {/* Share Dialog */}
      <Dialog
        open={shareDialogOpen}
        onClose={() => setShareDialogOpen(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>Share Content</DialogTitle>
        <DialogContent>
          <Typography variant="body2" color="text.secondary" gutterBottom>
            Enter four-word addresses to share this content with:
          </Typography>
          <TextField
            fullWidth
            placeholder="mountain-river-sun-cloud, forest-lake-bird-wind"
            multiline
            rows={3}
            sx={{ mt: 2 }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShareDialogOpen(false)}>Cancel</Button>
          <Button variant="contained" onClick={() => handleShare([])}>
            Share
          </Button>
        </DialogActions>
      </Dialog>

      {/* Snackbar for Notifications */}
      <Snackbar
        open={snackbar.open}
        autoHideDuration={4000}
        onClose={() => setSnackbar(prev => ({ ...prev, open: false }))}
      >
        <Alert severity={snackbar.severity} onClose={() => setSnackbar(prev => ({ ...prev, open: false }))}>
          {snackbar.message}
        </Alert>
      </Snackbar>
    </ResponsiveContainer>
  );
};

// Helper function to generate home page template based on entity type
function generateHomeTemplate(identity: FourWordIdentity, entityType: string): string {
  const displayName = identity.displayName || identity.address;
  
  const templates = {
    person: `# Welcome to ${displayName}'s Digital Space
Address: ${identity.address}

## About Me
I'm building my presence on the decentralized web. This is my personal space where I share thoughts, projects, and connect with others.

## My Spaces
- [Personal Projects](${identity.address}/projects/)
- [Blog Posts](${identity.address}/blog/)
- [Photo Gallery](${identity.address}/photos/)

## Connect With Me
Feel free to reach out directly at [${identity.address}](${identity.address}/contact) or explore my content above.

---
*Powered by Communitas - Your Digital Identity, Your Way*`,
    
    organization: `# ${displayName}
Address: ${identity.address}

## About Our Organization
We're building the future of decentralized collaboration. Our organization provides innovative solutions and creates value for our community.

## Our Projects
- [Current Initiatives](${identity.address}/projects/)
- [Team Directory](${identity.address}/team/)
- [Resources](${identity.address}/resources/)

## Get Involved
- [Join Our Team](${identity.address}/careers/)
- [Partner With Us](${identity.address}/partners/)
- [Contact Us](${identity.address}/contact/)

---
*${displayName} - Decentralized Organization*`,
    
    project: `# ${displayName}
Address: ${identity.address}

## Project Overview
This is the home page for our project. Here you'll find all the information, documentation, and resources related to our work.

## Quick Links
- [Documentation](${identity.address}/docs/)
- [Source Code](${identity.address}/code/)
- [Community](${identity.address}/community/)
- [Updates](${identity.address}/updates/)

## Contributors
See our [contributors page](${identity.address}/contributors/) for everyone involved in this project.

## Getting Started
Check out our [getting started guide](${identity.address}/docs/getting-started) to begin contributing or using our project.

---
*Open Source • Decentralized • Community Driven*`,
    
    group: `# ${displayName}
Address: ${identity.address}

## Welcome to Our Group
This is our group's home page where we coordinate, share resources, and build community together.

## Group Activities
- [Discussions](${identity.address}/discussions/)
- [Shared Files](${identity.address}/files/)
- [Events](${identity.address}/events/)
- [Members](${identity.address}/members/)

## How to Participate
1. Join our discussions
2. Share resources
3. Participate in group activities
4. Help newcomers get started

---
*Collaborative • Inclusive • Decentralized*`,
    
    channel: `# ${displayName}
Address: ${identity.address}

## Channel Information
This channel is dedicated to focused discussions and content sharing around our specific topic or interest.

## Channel Resources
- [Recent Discussions](${identity.address}/recent/)
- [Archive](${identity.address}/archive/)
- [Guidelines](${identity.address}/guidelines/)
- [Moderators](${identity.address}/moderators/)

## Participation Guidelines
Please follow our community guidelines to ensure productive and respectful discussions.

---
*Focused Discussion • Community Moderated*`
  };
  
  return templates[entityType as keyof typeof templates] || templates.person;
}

// Helper function to generate checksum (simplified for demo)
async function generateChecksum(content: string): Promise<string> {
  // In a real implementation, this would use BLAKE3 or similar
  // For demo purposes, using a simple hash
  const encoder = new TextEncoder();
  const data = encoder.encode(content);
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
}

export default UniversalMarkdownEditor;