import React, { useState, useEffect, useRef } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Box,
  Typography,
  IconButton,
  Avatar,
  Stack,
  Chip,
  Toolbar,
  AppBar,
  Tooltip,
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  Divider,
  Paper,
  TextField,
  Card,
  CardContent,
  Badge,
  Slider,
  ToggleButton,
  ToggleButtonGroup,
  Alert,
  Collapse,
  List,
  ListItem,
  ListItemAvatar,
  Tab,
  Tabs,
  Grid,
  Accordion,
  AccordionSummary,
  AccordionDetails,
} from '@mui/material';
import {
  Close as CloseIcon,
  Download as DownloadIcon,
  Share as ShareIcon,
  Print as PrintIcon,
  Edit as EditIcon,
  Save as SaveIcon,
  Undo as UndoIcon,
  Redo as RedoIcon,
  ZoomIn as ZoomInIcon,
  ZoomOut as ZoomOutIcon,
  Fullscreen as FullscreenIcon,
  FullscreenExit as FullscreenExitIcon,
  RotateRight as RotateRightIcon,
  FitScreen as FitScreenIcon,
  Comment as CommentIcon,
  Visibility as VisibilityIcon,
  VisibilityOff as VisibilityOffIcon,
  History as HistoryIcon,
  Info as InfoIcon,
  Security as SecurityIcon,
  Group as GroupIcon,
  Person as PersonIcon,
  AccessTime as AccessTimeIcon,
  InsertDriveFile as FileIcon,
  Image as ImageIcon,
  VideoFile as VideoIcon,
  AudioFile as AudioIcon,
  PictureAsPdf as PdfIcon,
  Description as DocIcon,
  Code as CodeIcon,
  ExpandMore as ExpandMoreIcon,
  ChatBubble as ChatBubbleIcon,
  Send as SendIcon,
  MoreVert as MoreVertIcon,
  Launch as LaunchIcon,
  ContentCopy as CopyIcon,
  Flag as FlagIcon,
  StarBorder as StarBorderIcon,
  Star as StarIcon,
  Bookmark as BookmarkIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { Member } from '../../types/organization';

interface DocumentComment {
  id: string;
  user: Member;
  content: string;
  timestamp: string;
  position?: { x: number; y: number };
  resolved: boolean;
  replies: DocumentComment[];
}

interface DocumentVersion {
  version: number;
  created_at: string;
  created_by: Member;
  size: number;
  comment?: string;
  changes_summary?: string;
}

interface SharedResource {
  id: string;
  name: string;
  type: string;
  mime_type: string;
  size: number;
  url: string;
  created_by: Member;
  created_at: string;
  updated_at: string;
  version: number;
  download_count: number;
  is_starred: boolean;
  description?: string;
  tags: string[];
}

interface DocumentViewerProps {
  open: boolean;
  onClose: () => void;
  resource: SharedResource;
  currentUser: Member;
  onSave?: (content: string) => Promise<void>;
  onComment?: (comment: Omit<DocumentComment, 'id' | 'timestamp' | 'replies'>) => Promise<void>;
  readOnly?: boolean;
  collaborators?: Member[];
}

export const DocumentViewer: React.FC<DocumentViewerProps> = ({
  open,
  onClose,
  resource,
  currentUser,
  onSave,
  onComment,
  readOnly = false,
  collaborators = [],
}) => {
  const [zoom, setZoom] = useState(100);
  const [fullscreen, setFullscreen] = useState(false);
  const [showComments, setShowComments] = useState(false);
  const [showVersions, setShowVersions] = useState(false);
  const [showInfo, setShowInfo] = useState(false);
  const [editMode, setEditMode] = useState(false);
  const [content, setContent] = useState('');
  const [comments, setComments] = useState<DocumentComment[]>([]);
  const [versions, setVersions] = useState<DocumentVersion[]>([]);
  const [newComment, setNewComment] = useState('');
  const [selectedComment, setSelectedComment] = useState<DocumentComment | null>(null);
  const [tabValue, setTabValue] = useState(0);
  const [menuAnchor, setMenuAnchor] = useState<null | HTMLElement>(null);

  const viewerRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);

  // Mock data
  useEffect(() => {
    if (open) {
      // Load mock comments
      setComments([
        {
          id: '1',
          user: { ...currentUser, display_name: 'Alice Johnson' },
          content: 'This section needs more detail about the implementation approach.',
          timestamp: new Date(Date.now() - 3600000).toISOString(),
          position: { x: 300, y: 150 },
          resolved: false,
          replies: [
            {
              id: '2',
              user: currentUser,
              content: 'I agree. I\'ll add more technical specifications.',
              timestamp: new Date(Date.now() - 1800000).toISOString(),
              resolved: false,
              replies: [],
            },
          ],
        },
        {
          id: '3',
          user: { ...currentUser, display_name: 'Bob Wilson' },
          content: 'Great work on the architecture diagram!',
          timestamp: new Date(Date.now() - 7200000).toISOString(),
          resolved: true,
          replies: [],
        },
      ]);

      // Load mock versions
      setVersions([
        {
          version: 3,
          created_at: new Date().toISOString(),
          created_by: currentUser,
          size: resource.size,
          comment: 'Updated API documentation with new endpoints',
          changes_summary: 'Added 3 new sections, modified 2 existing',
        },
        {
          version: 2,
          created_at: new Date(Date.now() - 86400000).toISOString(),
          created_by: { ...currentUser, display_name: 'Alice Johnson' },
          size: resource.size - 1024,
          comment: 'Fixed typos and formatting issues',
          changes_summary: 'Minor text corrections',
        },
        {
          version: 1,
          created_at: new Date(Date.now() - 172800000).toISOString(),
          created_by: resource.created_by,
          size: resource.size - 2048,
          comment: 'Initial version',
          changes_summary: 'Created document',
        },
      ]);

      // Load mock content based on file type
      if (resource.mime_type.includes('text') || resource.mime_type.includes('document')) {
        setContent(`# ${resource.name}

This is a sample document content that would be loaded from the actual file.

## Introduction
This document contains important information about our project specifications and requirements.

## Key Features
- Feature 1: Advanced user authentication
- Feature 2: Real-time collaboration
- Feature 3: Secure file sharing
- Feature 4: Cross-platform compatibility

## Technical Specifications
The system is built using modern technologies including React, TypeScript, and WebRTC for real-time communication.

## Implementation Notes
Please refer to the attached diagrams for detailed implementation guidelines.`);
      }
    }
  }, [open, resource]);

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const getFileIcon = (mimeType: string) => {
    if (mimeType.startsWith('image/')) return <ImageIcon color="info" />;
    if (mimeType.startsWith('video/')) return <VideoIcon color="secondary" />;
    if (mimeType.startsWith('audio/')) return <AudioIcon color="warning" />;
    if (mimeType === 'application/pdf') return <PdfIcon color="error" />;
    if (mimeType.includes('document') || mimeType.includes('text')) return <DocIcon color="info" />;
    if (mimeType.includes('code')) return <CodeIcon color="success" />;
    return <FileIcon color="action" />;
  };

  const handleAddComment = async () => {
    if (!newComment.trim() || !onComment) return;

    const comment: Omit<DocumentComment, 'id' | 'timestamp' | 'replies'> = {
      user: currentUser,
      content: newComment,
      resolved: false,
      position: { x: Math.random() * 400 + 100, y: Math.random() * 300 + 100 },
    };

    try {
      await onComment(comment);
      setNewComment('');
      // Refresh comments
    } catch (error) {
      console.error('Failed to add comment:', error);
    }
  };

  const handleSave = async () => {
    if (!onSave) return;
    
    try {
      await onSave(content);
      setEditMode(false);
    } catch (error) {
      console.error('Failed to save:', error);
    }
  };

  const renderContent = () => {
    const mimeType = resource.mime_type;

    if (mimeType.startsWith('image/')) {
      return (
        <Box
          sx={{
            display: 'flex',
            justifyContent: 'center',
            alignItems: 'center',
            minHeight: 400,
            position: 'relative',
          }}
        >
          <img
            src={resource.url}
            alt={resource.name}
            style={{
              maxWidth: '100%',
              maxHeight: '100%',
              transform: `scale(${zoom / 100})`,
              transition: 'transform 0.2s ease',
            }}
          />
          {/* Comment markers for images */}
          {comments.map((comment) => (
            comment.position && (
              <Box
                key={comment.id}
                sx={{
                  position: 'absolute',
                  left: comment.position.x,
                  top: comment.position.y,
                  transform: 'translate(-50%, -50%)',
                  cursor: 'pointer',
                }}
                onClick={() => setSelectedComment(comment)}
              >
                <Avatar
                  sx={{
                    width: 24,
                    height: 24,
                    bgcolor: comment.resolved ? 'success.main' : 'warning.main',
                    fontSize: '0.75rem',
                  }}
                >
                  <ChatBubbleIcon sx={{ fontSize: 12 }} />
                </Avatar>
              </Box>
            )
          ))}
        </Box>
      );
    }

    if (mimeType === 'application/pdf') {
      return (
        <Box sx={{ height: 600, border: 1, borderColor: 'divider', borderRadius: 1 }}>
          <iframe
            src={`${resource.url}#zoom=${zoom}`}
            width="100%"
            height="100%"
            style={{ border: 'none' }}
            title={resource.name}
          />
        </Box>
      );
    }

    if (mimeType.startsWith('video/')) {
      return (
        <Box sx={{ textAlign: 'center' }}>
          <video
            controls
            style={{
              maxWidth: '100%',
              height: 'auto',
              transform: `scale(${zoom / 100})`,
            }}
          >
            <source src={resource.url} type={mimeType} />
            Your browser does not support the video tag.
          </video>
        </Box>
      );
    }

    if (mimeType.startsWith('audio/')) {
      return (
        <Box sx={{ p: 4, textAlign: 'center' }}>
          <AudioIcon sx={{ fontSize: '4rem', color: 'text.secondary', mb: 2 }} />
          <audio controls style={{ width: '100%', maxWidth: 400 }}>
            <source src={resource.url} type={mimeType} />
            Your browser does not support the audio tag.
          </audio>
        </Box>
      );
    }

    if (mimeType.includes('text') || mimeType.includes('document')) {
      return (
        <Box sx={{ position: 'relative' }}>
          {editMode ? (
            <TextField
              fullWidth
              multiline
              rows={20}
              value={content}
              onChange={(e) => setContent(e.target.value)}
              variant="outlined"
              sx={{
                '& .MuiOutlinedInput-root': {
                  fontFamily: 'monospace',
                  fontSize: `${zoom / 100}rem`,
                },
              }}
            />
          ) : (
            <Paper
              variant="outlined"
              sx={{
                p: 3,
                minHeight: 400,
                fontFamily: 'monospace',
                fontSize: `${zoom / 100}rem`,
                lineHeight: 1.6,
                whiteSpace: 'pre-wrap',
                position: 'relative',
              }}
              ref={contentRef}
            >
              {content}
            </Paper>
          )}
        </Box>
      );
    }

    // Default view for unsupported file types
    return (
      <Box sx={{ textAlign: 'center', py: 8 }}>
        {getFileIcon(mimeType)}
        <Typography variant="h6" sx={{ mt: 2, mb: 1 }}>
          {resource.name}
        </Typography>
        <Typography variant="body2" color="text.secondary" gutterBottom>
          {formatFileSize(resource.size)} • {mimeType}
        </Typography>
        <Button
          variant="contained"
          startIcon={<DownloadIcon />}
          href={resource.url}
          download={resource.name}
          sx={{ mt: 2 }}
        >
          Download to View
        </Button>
      </Box>
    );
  };

  const renderSidebar = () => {
    return (
      <Box sx={{ width: 350, borderLeft: 1, borderColor: 'divider', display: 'flex', flexDirection: 'column' }}>
        <Tabs value={tabValue} onChange={(_, newValue) => setTabValue(newValue)}>
          <Tab label="Comments" />
          <Tab label="Versions" />
          <Tab label="Info" />
        </Tabs>

        <Box sx={{ flexGrow: 1, overflow: 'auto', p: 2 }}>
          {/* Comments Tab */}
          {tabValue === 0 && (
            <Box>
              <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ mb: 2 }}>
                <Typography variant="subtitle2">
                  Comments ({comments.length})
                </Typography>
                <Chip label={`${comments.filter(c => !c.resolved).length} active`} size="small" />
              </Stack>

              <Box sx={{ mb: 2 }}>
                <TextField
                  fullWidth
                  size="small"
                  placeholder="Add a comment..."
                  value={newComment}
                  onChange={(e) => setNewComment(e.target.value)}
                  multiline
                  rows={2}
                />
                <Button
                  fullWidth
                  variant="contained"
                  size="small"
                  startIcon={<SendIcon />}
                  onClick={handleAddComment}
                  disabled={!newComment.trim()}
                  sx={{ mt: 1 }}
                >
                  Add Comment
                </Button>
              </Box>

              <List dense>
                {comments.map((comment) => (
                  <ListItem
                    key={comment.id}
                    alignItems="flex-start"
                    sx={{
                      border: 1,
                      borderColor: comment.resolved ? 'success.light' : 'divider',
                      borderRadius: 1,
                      mb: 1,
                      bgcolor: comment.resolved ? 'success.light' : 'transparent',
                    }}
                  >
                    <ListItemAvatar>
                      <Avatar sx={{ width: 32, height: 32 }}>
                        {comment.user.display_name[0]}
                      </Avatar>
                    </ListItemAvatar>
                    <Box sx={{ flexGrow: 1 }}>
                      <Stack direction="row" alignItems="center" spacing={1}>
                        <Typography variant="caption" sx={{ fontWeight: 600 }}>
                          {comment.user.display_name}
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                          {new Date(comment.timestamp).toLocaleString()}
                        </Typography>
                        {comment.resolved && (
                          <Chip label="Resolved" size="small" color="success" />
                        )}
                      </Stack>
                      <Typography variant="body2" sx={{ mt: 0.5 }}>
                        {comment.content}
                      </Typography>
                      {comment.replies.length > 0 && (
                        <Box sx={{ mt: 1, pl: 2, borderLeft: 2, borderColor: 'divider' }}>
                          {comment.replies.map((reply) => (
                            <Box key={reply.id} sx={{ mb: 1 }}>
                              <Stack direction="row" alignItems="center" spacing={1}>
                                <Avatar sx={{ width: 20, height: 20, fontSize: '0.7rem' }}>
                                  {reply.user.display_name[0]}
                                </Avatar>
                                <Typography variant="caption" sx={{ fontWeight: 600 }}>
                                  {reply.user.display_name}
                                </Typography>
                                <Typography variant="caption" color="text.secondary">
                                  {new Date(reply.timestamp).toLocaleString()}
                                </Typography>
                              </Stack>
                              <Typography variant="caption" sx={{ mt: 0.5, display: 'block' }}>
                                {reply.content}
                              </Typography>
                            </Box>
                          ))}
                        </Box>
                      )}
                    </Box>
                  </ListItem>
                ))}
              </List>
            </Box>
          )}

          {/* Versions Tab */}
          {tabValue === 1 && (
            <Box>
              <Typography variant="subtitle2" gutterBottom>
                Version History
              </Typography>
              <List dense>
                {versions.map((version) => (
                  <ListItem key={version.version} sx={{ px: 0 }}>
                    <Card variant="outlined" sx={{ width: '100%' }}>
                      <CardContent sx={{ py: 1.5 }}>
                        <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ mb: 1 }}>
                          <Chip
                            label={`v${version.version}`}
                            size="small"
                            color={version.version === resource.version ? 'primary' : 'default'}
                          />
                          <Typography variant="caption" color="text.secondary">
                            {formatFileSize(version.size)}
                          </Typography>
                        </Stack>
                        <Typography variant="body2" gutterBottom>
                          {version.comment || 'No comment'}
                        </Typography>
                        <Stack direction="row" alignItems="center" spacing={1}>
                          <Avatar sx={{ width: 20, height: 20, fontSize: '0.7rem' }}>
                            {version.created_by.display_name[0]}
                          </Avatar>
                          <Typography variant="caption">
                            {version.created_by.display_name}
                          </Typography>
                          <Typography variant="caption" color="text.secondary">
                            {new Date(version.created_at).toLocaleDateString()}
                          </Typography>
                        </Stack>
                        {version.changes_summary && (
                          <Typography variant="caption" color="text.secondary" sx={{ mt: 1, display: 'block' }}>
                            {version.changes_summary}
                          </Typography>
                        )}
                      </CardContent>
                    </Card>
                  </ListItem>
                ))}
              </List>
            </Box>
          )}

          {/* Info Tab */}
          {tabValue === 2 && (
            <Box>
              <Typography variant="subtitle2" gutterBottom>
                File Information
              </Typography>
              <Stack spacing={2}>
                <Card variant="outlined">
                  <CardContent sx={{ py: 2 }}>
                    <Grid container spacing={1}>
                      <Grid item xs={4}>
                        <Typography variant="caption" color="text.secondary">Name:</Typography>
                      </Grid>
                      <Grid item xs={8}>
                        <Typography variant="body2">{resource.name}</Typography>
                      </Grid>
                      <Grid item xs={4}>
                        <Typography variant="caption" color="text.secondary">Size:</Typography>
                      </Grid>
                      <Grid item xs={8}>
                        <Typography variant="body2">{formatFileSize(resource.size)}</Typography>
                      </Grid>
                      <Grid item xs={4}>
                        <Typography variant="caption" color="text.secondary">Type:</Typography>
                      </Grid>
                      <Grid item xs={8}>
                        <Typography variant="body2">{resource.mime_type}</Typography>
                      </Grid>
                      <Grid item xs={4}>
                        <Typography variant="caption" color="text.secondary">Created:</Typography>
                      </Grid>
                      <Grid item xs={8}>
                        <Typography variant="body2">
                          {new Date(resource.created_at).toLocaleDateString()}
                        </Typography>
                      </Grid>
                      <Grid item xs={4}>
                        <Typography variant="caption" color="text.secondary">Modified:</Typography>
                      </Grid>
                      <Grid item xs={8}>
                        <Typography variant="body2">
                          {new Date(resource.updated_at).toLocaleDateString()}
                        </Typography>
                      </Grid>
                      <Grid item xs={4}>
                        <Typography variant="caption" color="text.secondary">Downloads:</Typography>
                      </Grid>
                      <Grid item xs={8}>
                        <Typography variant="body2">{resource.download_count}</Typography>
                      </Grid>
                    </Grid>
                  </CardContent>
                </Card>

                {resource.tags.length > 0 && (
                  <Box>
                    <Typography variant="caption" color="text.secondary" gutterBottom>
                      Tags:
                    </Typography>
                    <Stack direction="row" spacing={0.5} flexWrap="wrap" useFlexGap>
                      {resource.tags.map((tag) => (
                        <Chip key={tag} label={tag} size="small" variant="outlined" />
                      ))}
                    </Stack>
                  </Box>
                )}

                {resource.description && (
                  <Box>
                    <Typography variant="caption" color="text.secondary" gutterBottom>
                      Description:
                    </Typography>
                    <Typography variant="body2">{resource.description}</Typography>
                  </Box>
                )}
              </Stack>
            </Box>
          )}
        </Box>
      </Box>
    );
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      maxWidth={false}
      fullWidth
      PaperProps={{
        sx: {
          width: '95vw',
          height: '90vh',
          maxWidth: 'none',
          display: 'flex',
          flexDirection: 'column',
        },
      }}
    >
      {/* Header */}
      <AppBar position="static" color="default" elevation={1}>
        <Toolbar variant="dense">
          <Stack direction="row" alignItems="center" spacing={2} sx={{ flexGrow: 1 }}>
            {getFileIcon(resource.mime_type)}
            <Box>
              <Typography variant="subtitle1" noWrap>
                {resource.name}
              </Typography>
              <Typography variant="caption" color="text.secondary">
                {formatFileSize(resource.size)} • Version {resource.version}
              </Typography>
            </Box>
            {resource.is_starred && <StarIcon color="warning" />}
          </Stack>

          <Stack direction="row" spacing={1}>
            {/* Zoom Controls */}
            <Tooltip title="Zoom Out">
              <IconButton
                size="small"
                onClick={() => setZoom(Math.max(25, zoom - 25))}
                disabled={zoom <= 25}
              >
                <ZoomOutIcon />
              </IconButton>
            </Tooltip>
            <Typography variant="body2" sx={{ minWidth: 40, textAlign: 'center', alignSelf: 'center' }}>
              {zoom}%
            </Typography>
            <Tooltip title="Zoom In">
              <IconButton
                size="small"
                onClick={() => setZoom(Math.min(200, zoom + 25))}
                disabled={zoom >= 200}
              >
                <ZoomInIcon />
              </IconButton>
            </Tooltip>

            <Divider orientation="vertical" flexItem />

            {/* Edit Mode */}
            {!readOnly && (resource.mime_type.includes('text') || resource.mime_type.includes('document')) && (
              <>
                {editMode ? (
                  <Tooltip title="Save">
                    <IconButton size="small" onClick={handleSave} color="primary">
                      <SaveIcon />
                    </IconButton>
                  </Tooltip>
                ) : (
                  <Tooltip title="Edit">
                    <IconButton size="small" onClick={() => setEditMode(true)}>
                      <EditIcon />
                    </IconButton>
                  </Tooltip>
                )}
              </>
            )}

            <Tooltip title="Comments">
              <IconButton
                size="small"
                onClick={() => setShowComments(!showComments)}
                color={showComments ? 'primary' : 'default'}
              >
                <Badge badgeContent={comments.filter(c => !c.resolved).length} color="error">
                  <CommentIcon />
                </Badge>
              </IconButton>
            </Tooltip>

            <Tooltip title="Download">
              <IconButton size="small" href={resource.url} download={resource.name}>
                <DownloadIcon />
              </IconButton>
            </Tooltip>

            <Tooltip title="Share">
              <IconButton size="small">
                <ShareIcon />
              </IconButton>
            </Tooltip>

            <IconButton size="small" onClick={(e) => setMenuAnchor(e.currentTarget)}>
              <MoreVertIcon />
            </IconButton>

            <Tooltip title="Close">
              <IconButton size="small" onClick={onClose}>
                <CloseIcon />
              </IconButton>
            </Tooltip>
          </Stack>
        </Toolbar>
      </AppBar>

      {/* Content */}
      <Box sx={{ display: 'flex', flexGrow: 1, overflow: 'hidden' }}>
        <Box
          ref={viewerRef}
          sx={{
            flexGrow: 1,
            overflow: 'auto',
            p: 2,
            display: 'flex',
            flexDirection: 'column',
          }}
        >
          {renderContent()}
        </Box>

        {/* Sidebar */}
        {showComments && renderSidebar()}
      </Box>

      {/* Menu */}
      <Menu
        anchorEl={menuAnchor}
        open={Boolean(menuAnchor)}
        onClose={() => setMenuAnchor(null)}
      >
        <MenuItem onClick={() => setMenuAnchor(null)}>
          <ListItemIcon><PrintIcon /></ListItemIcon>
          <ListItemText>Print</ListItemText>
        </MenuItem>
        <MenuItem onClick={() => setMenuAnchor(null)}>
          <ListItemIcon><CopyIcon /></ListItemIcon>
          <ListItemText>Copy Link</ListItemText>
        </MenuItem>
        <MenuItem onClick={() => setMenuAnchor(null)}>
          <ListItemIcon><LaunchIcon /></ListItemIcon>
          <ListItemText>Open in New Tab</ListItemText>
        </MenuItem>
        <Divider />
        <MenuItem onClick={() => setMenuAnchor(null)}>
          <ListItemIcon>{resource.is_starred ? <StarIcon /> : <StarBorderIcon />}</ListItemIcon>
          <ListItemText>{resource.is_starred ? 'Remove Star' : 'Add Star'}</ListItemText>
        </MenuItem>
        <MenuItem onClick={() => setMenuAnchor(null)}>
          <ListItemIcon><FlagIcon /></ListItemIcon>
          <ListItemText>Report Issue</ListItemText>
        </MenuItem>
      </Menu>
    </Dialog>
  );
};

export default DocumentViewer;