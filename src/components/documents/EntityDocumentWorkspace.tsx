/**
 * EntityDocumentWorkspace - Collaborative markdown document workspace for entities
 *
 * This component provides the core document collaboration interface where:
 * - Each entity (org, channel, project, group, contact) has its own document space
 * - Documents are stored in two modes: 'files' (encrypted, member-only) or 'web' (public)
 * - Real-time collaborative editing via Yrs CRDT
 * - Markdown rendering with live preview
 * - Shared automatically with all entity members
 *
 * This is the killer feature that gives Communitas an edge over Slack/Discord/Teams.
 */

import {
    Add as AddIcon, Delete as DeleteIcon, Description as DocumentIcon, DriveFileRenameOutline as RenameIcon, Edit as EditIcon, FileCopy as DuplicateIcon, Home as HomeIcon, Lock as LockIcon, MoreVert as MoreIcon, Public as PublicIcon, Refresh as RefreshIcon,
    Search as SearchIcon, ViewList as ListViewIcon,
    ViewModule as GridViewIcon, Visibility as PreviewIcon
} from '@mui/icons-material';
import {
    Alert, Box, Breadcrumbs, Button, Card, CardActions, CardContent, Chip, CircularProgress, Dialog, DialogActions, DialogContent, DialogTitle, Divider, Grid, IconButton, Link, List,
    ListItem, ListItemIcon,
    ListItemSecondaryAction, ListItemText, Menu,
    MenuItem, Paper, Stack, TextField, ToggleButton, ToggleButtonGroup, Tooltip, Typography
} from '@mui/material';
import { alpha, useTheme } from '@mui/material/styles';
import React, { useCallback, useEffect, useMemo, useState } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { documentService } from '../../services/DocumentService';
import type {
    Document,
    DocumentStorageMode,
    DocumentWithState
} from '../../types/documents';
import { ModernButton } from '../ui/ModernButton';
import { RenameDocumentDialog } from './RenameDocumentDialog';

interface EntityDocumentWorkspaceProps {
  entityId: string;
  entityName: string;
  storageMode: DocumentStorageMode;
  permissions: string[];
}

type ViewMode = 'list' | 'editor' | 'preview';
type LayoutMode = 'grid' | 'list';

export const EntityDocumentWorkspace: React.FC<EntityDocumentWorkspaceProps> = ({
  entityId,
  entityName,
  storageMode,
  permissions,
}) => {
  const theme = useTheme();

  // State
  const [documents, setDocuments] = useState<Document[]>([]);
  const [selectedDoc, setSelectedDoc] = useState<DocumentWithState | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>('list');
  const [layoutMode, setLayoutMode] = useState<LayoutMode>('grid');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [isCreating, setIsCreating] = useState(false);

  // Dialog state
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [docToDelete, setDocToDelete] = useState<Document | null>(null);
  const [renameDialogOpen, setRenameDialogOpen] = useState(false);
  const [docToRename, setDocToRename] = useState<Document | null>(null);
  const [newDocName, setNewDocName] = useState('');

  // Context menu
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    doc: Document;
  } | null>(null);

  // Permissions
  const canWrite = permissions.includes('write') || permissions.includes('admin');
  const canDelete = permissions.includes('admin');

  // Storage mode label and icon
  const storageModeInfo = useMemo(() => {
    return storageMode === 'web'
      ? { label: 'Public Website', icon: <PublicIcon />, color: 'info' }
      : { label: 'Private Files', icon: <LockIcon />, color: 'success' };
  }, [storageMode]);

  // Load documents for entity
  const loadDocuments = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const docs = await documentService.listDocuments(entityId, storageMode);
      setDocuments(docs);
    } catch (err) {
      console.error('Failed to load documents:', err);
      setError('Failed to load documents');
    } finally {
      setLoading(false);
    }
  }, [entityId, storageMode]);

  // Initial load
  useEffect(() => {
    loadDocuments();
  }, [loadDocuments]);

  // Filter documents by search query
  const filteredDocuments = useMemo(() => {
    if (!searchQuery.trim()) return documents;
    const query = searchQuery.toLowerCase();
    return documents.filter((doc) => doc.name.toLowerCase().includes(query));
  }, [documents, searchQuery]);

  // Create new document
  const handleCreateDocument = useCallback(async () => {
    if (!newDocName.trim() || isCreating) return;

    setIsCreating(true);
    setError(null);

    try {
      // Auto-append .md extension if not present
      let docName = newDocName.trim();
      if (!docName.endsWith('.md')) {
        docName = `${docName}.md`;
      }

      console.log('Creating document:', docName);
      const doc = await documentService.createDocument(entityId, docName, storageMode);
      console.log('Document created:', doc);

      setDocuments((prev) => [...prev, doc]);
      setCreateDialogOpen(false);
      setNewDocName('');

      // Open the new document in editor
      const docWithContent = await documentService.getDocumentWithContent(doc.docId);
      setSelectedDoc(docWithContent);
      setViewMode('editor');
    } catch (err) {
      console.error('Failed to create document:', err);
      setError('Failed to create document');
    } finally {
      setIsCreating(false);
    }
  }, [newDocName, entityId, storageMode, isCreating]);

  // Open document in editor/preview
  const handleOpenDocument = useCallback(
    async (doc: Document, mode: 'editor' | 'preview' = 'preview') => {
      try {
        const docWithContent = await documentService.getDocumentWithContent(doc.docId);
        setSelectedDoc(docWithContent);
        setViewMode(mode);
      } catch (err) {
        console.error('Failed to open document:', err);
        setError('Failed to open document');
      }
    },
    []
  );

  // Delete document
  const handleDeleteDocument = useCallback(async () => {
    if (!docToDelete) return;

    try {
      await documentService.deleteDocument(docToDelete.docId);
      setDocuments((prev) => prev.filter((d) => d.docId !== docToDelete.docId));
      if (selectedDoc?.docId === docToDelete.docId) {
        setSelectedDoc(null);
        setViewMode('list');
      }
      setDeleteDialogOpen(false);
      setDocToDelete(null);
    } catch (err) {
      console.error('Failed to delete document:', err);
      setError('Failed to delete document');
    }
  }, [docToDelete, selectedDoc]);

  // Rename document handler
  const handleRenameDocument = useCallback(async (document: Document, newName: string) => {
    try {
      setError(null);
      await documentService.renameDocument(document.docId, newName);
      await loadDocuments(); // Refresh list
    } catch (err) {
      console.error('Failed to rename document:', err);
      setError('Failed to rename document');
      throw err; // Re-throw so dialog can handle
    }
  }, [loadDocuments]);

  // Duplicate document handler
  const handleDuplicateDocument = useCallback(async (doc: Document) => {
    try {
      setError(null);
      const newName = `${doc.name} (Copy)`;
      const content = await documentService.getText(doc.docId);
      const newDoc = await documentService.createDocument(
        doc.entityId,
        newName,
        doc.storageMode
      );
      if (content) {
        await documentService.insertText(newDoc.docId, 0, content);
      }
      await loadDocuments(); // Refresh list
    } catch (err) {
      console.error('Failed to duplicate document:', err);
      setError('Failed to duplicate document');
    }
  }, [loadDocuments]);

  // Back to list
  const handleBackToList = useCallback(() => {
    setSelectedDoc(null);
    setViewMode('list');
  }, []);

  // Context menu handlers
  const handleContextMenu = useCallback(
    (event: React.MouseEvent, doc: Document) => {
      event.preventDefault();
      setContextMenu({ x: event.clientX, y: event.clientY, doc });
    },
    []
  );

  const handleCloseContextMenu = useCallback(() => {
    setContextMenu(null);
  }, []);

  const handleContextEdit = useCallback(() => {
    if (contextMenu) {
      handleOpenDocument(contextMenu.doc, 'editor');
    }
    handleCloseContextMenu();
  }, [contextMenu, handleOpenDocument, handleCloseContextMenu]);

  const handleContextPreview = useCallback(() => {
    if (contextMenu) {
      handleOpenDocument(contextMenu.doc, 'preview');
    }
    handleCloseContextMenu();
  }, [contextMenu, handleOpenDocument, handleCloseContextMenu]);

  const handleContextDelete = useCallback(() => {
    if (contextMenu) {
      setDocToDelete(contextMenu.doc);
      setDeleteDialogOpen(true);
    }
    handleCloseContextMenu();
  }, [contextMenu, handleCloseContextMenu]);

  // Render: Document List View
  const renderListView = () => (
    <Box>
      {/* Header */}
      <Box sx={{ mb: 3 }}>
        <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 2 }}>
          <Box>
            <Typography variant="h5" fontWeight={600} gutterBottom>
              Documents shared with {entityName}
            </Typography>
            <Stack direction="row" spacing={1} alignItems="center">
              <Chip
                icon={storageModeInfo.icon}
                label={storageModeInfo.label}
                color={storageModeInfo.color as any}
                size="small"
              />
              <Chip label={`${filteredDocuments.length} documents`} size="small" variant="outlined" />
            </Stack>
          </Box>
          <Stack direction="row" spacing={1}>
            <ToggleButtonGroup
              value={layoutMode}
              exclusive
              onChange={(_, newMode) => newMode && setLayoutMode(newMode)}
              size="small"
            >
              <ToggleButton value="grid">
                <Tooltip title="Grid view">
                  <GridViewIcon fontSize="small" />
                </Tooltip>
              </ToggleButton>
              <ToggleButton value="list">
                <Tooltip title="List view">
                  <ListViewIcon fontSize="small" />
                </Tooltip>
              </ToggleButton>
            </ToggleButtonGroup>
            <IconButton onClick={loadDocuments} size="small">
              <RefreshIcon />
            </IconButton>
            {canWrite && (
              <ModernButton
                variant="contained"
                gradient={true}
                startIcon={<AddIcon />}
                onClick={() => setCreateDialogOpen(true)}
              >
                New Document
              </ModernButton>
            )}
          </Stack>
        </Stack>

        {/* Search */}
        <TextField
          fullWidth
          size="small"
          placeholder="Search documents..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          InputProps={{
            startAdornment: <SearchIcon sx={{ mr: 1, color: 'text.secondary' }} />,
          }}
        />
      </Box>

      {/* Loading */}
      {loading && (
        <Box sx={{ display: 'flex', justifyContent: 'center', py: 8 }}>
          <CircularProgress />
        </Box>
      )}

      {/* Error */}
      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {/* Empty state */}
      {!loading && filteredDocuments.length === 0 && (
        <Paper
          sx={{
            p: 8,
            textAlign: 'center',
            background: alpha(theme.palette.background.paper, 0.6),
          }}
        >
          <DocumentIcon sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
          <Typography variant="h6" color="text.secondary" gutterBottom>
            {searchQuery ? 'No documents found' : 'No documents yet'}
          </Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
            {searchQuery
              ? 'Try a different search term'
              : `Create your first ${storageMode === 'web' ? 'public' : 'private'} document`}
          </Typography>
          {canWrite && !searchQuery && (
            <ModernButton
              variant="contained"
              gradient={true}
              startIcon={<AddIcon />}
              onClick={() => setCreateDialogOpen(true)}
            >
              Create Document
            </ModernButton>
          )}
        </Paper>
      )}

      {/* Document Grid */}
      {!loading && layoutMode === 'grid' && filteredDocuments.length > 0 && (
        <Grid container spacing={2}>
          {filteredDocuments.map((doc) => (
            <Grid item xs={12} sm={6} md={4} key={doc.docId}>
              <Card
                sx={{
                  cursor: 'pointer',
                  transition: 'all 0.2s',
                  '&:hover': {
                    transform: 'translateY(-4px)',
                    boxShadow: theme.shadows[8],
                  },
                }}
                onContextMenu={(e) => handleContextMenu(e, doc)}
              >
                <CardContent>
                  <Stack direction="row" alignItems="center" spacing={1} sx={{ mb: 1 }}>
                    <DocumentIcon color="primary" />
                    <Typography variant="h6" noWrap sx={{ flex: 1 }}>
                      {doc.name}
                    </Typography>
                    <IconButton
                      size="small"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleContextMenu(e as any, doc);
                      }}
                    >
                      <MoreIcon fontSize="small" />
                    </IconButton>
                  </Stack>
                  <Typography variant="caption" color="text.secondary">
                    {doc.storageMode === 'web' ? 'Public' : 'Private'}
                  </Typography>
                </CardContent>
                <CardActions>
                  <Button
                    size="small"
                    startIcon={<PreviewIcon />}
                    onClick={() => handleOpenDocument(doc, 'preview')}
                  >
                    View
                  </Button>
                  {canWrite && (
                    <Button
                      size="small"
                      startIcon={<EditIcon />}
                      onClick={() => handleOpenDocument(doc, 'editor')}
                    >
                      Edit
                    </Button>
                  )}
                </CardActions>
              </Card>
            </Grid>
          ))}
        </Grid>
      )}

      {/* Document List */}
      {!loading && layoutMode === 'list' && filteredDocuments.length > 0 && (
        <Paper>
          <List>
            {filteredDocuments.map((doc, index) => (
              <React.Fragment key={doc.docId}>
                {index > 0 && <Divider />}
                <ListItem
                  button
                  onClick={() => handleOpenDocument(doc, 'preview')}
                  onContextMenu={(e) => handleContextMenu(e, doc)}
                >
                  <ListItemIcon>
                    <DocumentIcon />
                  </ListItemIcon>
                  <ListItemText
                    primary={doc.name}
                    secondary={doc.storageMode === 'web' ? 'Public Website' : 'Private Files'}
                  />
                  <ListItemSecondaryAction>
                    <Stack direction="row" spacing={1}>
                      {canWrite && (
                        <IconButton
                          edge="end"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleOpenDocument(doc, 'editor');
                          }}
                        >
                          <EditIcon />
                        </IconButton>
                      )}
                      <IconButton
                        edge="end"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleContextMenu(e as any, doc);
                        }}
                      >
                        <MoreIcon />
                      </IconButton>
                    </Stack>
                  </ListItemSecondaryAction>
                </ListItem>
              </React.Fragment>
            ))}
          </List>
        </Paper>
      )}
    </Box>
  );

  // Render: Document Editor View
  const renderEditorView = () => (
    <Box>
      {/* Header */}
      <Box sx={{ mb: 2 }}>
        <Breadcrumbs>
          <Link
            component="button"
            variant="body2"
            onClick={handleBackToList}
            sx={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}
          >
            <HomeIcon sx={{ mr: 0.5 }} fontSize="small" />
            Documents
          </Link>
          <Typography color="text.primary">{selectedDoc?.name}</Typography>
        </Breadcrumbs>
      </Box>

      {/* TODO: Implement Monaco editor integration */}
      <Paper sx={{ p: 3, minHeight: 600 }}>
        <Alert severity="info">
          Editor view coming in Phase 2 with CRDT real-time collaboration
        </Alert>
        <Typography variant="body2" sx={{ mt: 2 }}>
          Document: {selectedDoc?.docId}
        </Typography>
        <Typography variant="body2">Content length: {selectedDoc?.content?.length || 0}</Typography>
      </Paper>
    </Box>
  );

  // Render: Document Preview View
  const renderPreviewView = () => (
    <Box>
      {/* Header */}
      <Box sx={{ mb: 2 }}>
        <Stack direction="row" justifyContent="space-between" alignItems="center">
          <Breadcrumbs>
            <Link
              component="button"
              variant="body2"
              onClick={handleBackToList}
              sx={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}
            >
              <HomeIcon sx={{ mr: 0.5 }} fontSize="small" />
              Documents
            </Link>
            <Typography color="text.primary">{selectedDoc?.name}</Typography>
          </Breadcrumbs>
          {canWrite && (
            <ModernButton
              variant="contained"
              startIcon={<EditIcon />}
              onClick={() => setViewMode('editor')}
            >
              Edit
            </ModernButton>
          )}
        </Stack>
      </Box>

      {/* Markdown Preview */}
      <Paper sx={{ p: 4, minHeight: 600 }}>
        {selectedDoc?.content ? (
          <ReactMarkdown remarkPlugins={[remarkGfm]}>
            {selectedDoc.content}
          </ReactMarkdown>
        ) : (
          <Typography variant="body2" color="text.secondary">
            This document is empty. Click Edit to add content.
          </Typography>
        )}
      </Paper>
    </Box>
  );

  // Render main component
  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column', p: 3 }}>
      {/* Render current view */}
      {viewMode === 'list' && renderListView()}
      {viewMode === 'editor' && selectedDoc && renderEditorView()}
      {viewMode === 'preview' && selectedDoc && renderPreviewView()}

      {/* Create Document Dialog */}
      <Dialog open={createDialogOpen} onClose={() => setCreateDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create New Document</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            fullWidth
            label="Document Name"
            placeholder="meeting-notes"
            value={newDocName}
            onChange={(e) => setNewDocName(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleCreateDocument()}
            helperText={`Will be created in ${storageModeInfo.label} storage (extension .md will be added automatically)`}
            sx={{ mt: 2 }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateDialogOpen(false)}>Cancel</Button>
          <ModernButton
            variant="contained"
            gradient={true}
            onClick={handleCreateDocument}
            disabled={!newDocName.trim() || isCreating}
          >
            {isCreating ? 'Creating...' : 'Create'}
          </ModernButton>
        </DialogActions>
      </Dialog>

      {/* Delete Document Dialog */}
      <Dialog open={deleteDialogOpen} onClose={() => setDeleteDialogOpen(false)} maxWidth="sm">
        <DialogTitle>Delete Document?</DialogTitle>
        <DialogContent>
          <Typography>
            Are you sure you want to delete <strong>{docToDelete?.name}</strong>? This action cannot
            be undone.
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteDialogOpen(false)}>Cancel</Button>
          <Button color="error" variant="contained" onClick={handleDeleteDocument}>
            Delete
          </Button>
        </DialogActions>
      </Dialog>

      {/* Rename Document Dialog */}
      <RenameDocumentDialog
        open={renameDialogOpen}
        document={docToRename}
        onClose={() => {
          setRenameDialogOpen(false);
          setDocToRename(null);
        }}
        onRename={handleRenameDocument}
      />

      {/* Context Menu */}
      <Menu
        open={Boolean(contextMenu)}
        onClose={handleCloseContextMenu}
        anchorReference="anchorPosition"
        anchorPosition={
          contextMenu ? { top: contextMenu.y, left: contextMenu.x } : undefined
        }
      >
        <MenuItem onClick={handleContextPreview}>
          <ListItemIcon>
            <PreviewIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText>Preview</ListItemText>
        </MenuItem>

        {canWrite && (
          <MenuItem onClick={handleContextEdit}>
            <ListItemIcon>
              <EditIcon fontSize="small" />
            </ListItemIcon>
            <ListItemText>Edit</ListItemText>
          </MenuItem>
        )}

        {canWrite && (
          <MenuItem onClick={() => {
            if (contextMenu) {
              setDocToRename(contextMenu.doc);
              setRenameDialogOpen(true);
            }
            handleCloseContextMenu();
          }}>
            <ListItemIcon>
              <RenameIcon fontSize="small" />
            </ListItemIcon>
            <ListItemText>Rename</ListItemText>
          </MenuItem>
        )}

        <MenuItem onClick={() => {
          if (contextMenu) {
            handleDuplicateDocument(contextMenu.doc);
          }
          handleCloseContextMenu();
        }}>
          <ListItemIcon>
            <DuplicateIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText>Duplicate</ListItemText>
        </MenuItem>

        {canDelete && [
          <Divider key="divider" />,
          <MenuItem key="delete" onClick={handleContextDelete}>
            <ListItemIcon>
              <DeleteIcon fontSize="small" color="error" />
            </ListItemIcon>
            <ListItemText>Delete</ListItemText>
          </MenuItem>,
        ]}
      </Menu>
    </Box>
  );
};
