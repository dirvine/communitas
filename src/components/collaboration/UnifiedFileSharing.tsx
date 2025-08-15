import React, { useState } from 'react';
import {
  Box,
  Paper,
  Typography,
  Button,
  IconButton,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  ListItemSecondaryAction,
  Chip,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Tab,
  Tabs,
  Grid,
  Card,
  CardContent,
  CardActions,
  Avatar,
  LinearProgress,
  Alert,
  Stack,
  Divider,
  Tooltip,
  Menu,
  MenuItem,
  Switch,
  FormControlLabel,
  InputAdornment,
  alpha,
} from '@mui/material';
import {
  CloudUpload as UploadIcon,
  Folder as FolderIcon,
  InsertDriveFile as FileIcon,
  Language as WebsiteIcon,
  Public as PublishIcon,
  Lock as PrivateIcon,
  Share as ShareIcon,
  Download as DownloadIcon,
  Delete as DeleteIcon,
  MoreVert as MoreIcon,
  ContentCopy as CopyIcon,
  Visibility as PreviewIcon,
  Edit as EditIcon,
  Code as CodeIcon,
  Description as MarkdownIcon,
  Image as ImageIcon,
  VideoLibrary as VideoIcon,
  AudioFile as AudioIcon,
  Archive as ZipIcon,
  CheckCircle as PublishedIcon,
  RadioButtonUnchecked as UnpublishedIcon,
  Dns as DnsIcon,
  VpnKey as KeyIcon,
  ForwardToInbox as ForwardIcon,
} from '@mui/icons-material';
import { SharedFile, PublishedWebsite, NetworkIdentity } from '../../types/collaboration';

interface UnifiedFileSharingProps {
  entityId: string;
  entityType: string;
  entityName: string;
  files: SharedFile[];
  publishedWebsite?: PublishedWebsite;
  onFileUpload?: (files: File[]) => void;
  onFileDelete?: (fileId: string) => void;
  onFileShare?: (fileId: string, users: string[]) => void;
  onPublishWebsite?: (config: WebsitePublishConfig) => void;
  onUnpublishWebsite?: () => void;
  onGenerateIdentity?: (type: 'file' | 'website') => NetworkIdentity;
}

interface WebsitePublishConfig {
  name: string;
  domain: string;
  indexFile: string;
  enableAnalytics: boolean;
  customCSS?: string;
  theme?: 'light' | 'dark' | 'auto';
}

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props;
  return (
    <div hidden={value !== index} {...other}>
      {value === index && <Box sx={{ pt: 2 }}>{children}</Box>}
    </div>
  );
}

export const UnifiedFileSharing: React.FC<UnifiedFileSharingProps> = ({
  entityId,
  entityType,
  entityName,
  files,
  publishedWebsite,
  onFileUpload,
  onFileDelete,
  onFileShare,
  onPublishWebsite,
  onUnpublishWebsite,
  onGenerateIdentity,
}) => {
  const [activeTab, setActiveTab] = useState(0);
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [publishDialog, setPublishDialog] = useState(false);
  const [identityDialog, setIdentityDialog] = useState(false);
  const [shareDialog, setShareDialog] = useState(false);
  const [selectedFile, setSelectedFile] = useState<SharedFile | null>(null);
  const [websiteConfig, setWebsiteConfig] = useState<WebsitePublishConfig>({
    name: `${entityName} Website`,
    domain: '',
    indexFile: 'index.html',
    enableAnalytics: true,
    theme: 'auto',
  });
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);

  const getFileIcon = (file: SharedFile) => {
    const type = file.mimeType.split('/')[0];
    switch (type) {
      case 'image': return <ImageIcon />;
      case 'video': return <VideoIcon />;
      case 'audio': return <AudioIcon />;
      case 'text':
        if (file.name.endsWith('.md')) return <MarkdownIcon />;
        if (file.name.endsWith('.html')) return <CodeIcon />;
        return <FileIcon />;
      case 'application':
        if (file.name.endsWith('.zip')) return <ZipIcon />;
        return <FileIcon />;
      default: return <FileIcon />;
    }
  };

  const formatFileSize = (bytes: number): string => {
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`;
  };

  const handleFileUpload = (event: React.ChangeEvent<HTMLInputElement>) => {
    if (event.target.files) {
      const filesArray = Array.from(event.target.files);
      onFileUpload?.(filesArray);
    }
  };

  const handlePublishWebsite = () => {
    // Generate a four-word domain if not provided
    if (!websiteConfig.domain) {
      const identity = onGenerateIdentity?.('website');
      if (identity) {
        websiteConfig.domain = identity.fourWords;
      }
    }
    onPublishWebsite?.(websiteConfig);
    setPublishDialog(false);
  };

  const markdownFiles = files.filter(f => f.name.endsWith('.md'));
  const htmlFiles = files.filter(f => f.name.endsWith('.html'));
  const websiteFiles = [...markdownFiles, ...htmlFiles];

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <Paper elevation={0} sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
        <Stack direction="row" alignItems="center" justifyContent="space-between">
          <Box>
            <Typography variant="h6" fontWeight={600}>
              File Sharing & Website
            </Typography>
            <Typography variant="body2" color="text.secondary">
              {entityName} • {files.length} files • {formatFileSize(files.reduce((acc, f) => acc + f.size, 0))}
            </Typography>
          </Box>
          <Stack direction="row" spacing={1}>
            <Button
              variant="outlined"
              startIcon={<DnsIcon />}
              onClick={() => setIdentityDialog(true)}
            >
              Network Identity
            </Button>
            <Button
              variant="contained"
              startIcon={<UploadIcon />}
              component="label"
            >
              Upload Files
              <input
                type="file"
                hidden
                multiple
                onChange={handleFileUpload}
              />
            </Button>
          </Stack>
        </Stack>
      </Paper>

      {/* Tabs */}
      <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
        <Tabs value={activeTab} onChange={(e, v) => setActiveTab(v)}>
          <Tab label="Files" icon={<FolderIcon />} iconPosition="start" />
          <Tab 
            label="Website" 
            icon={<WebsiteIcon />} 
            iconPosition="start"
            sx={{
              '& .MuiTab-iconWrapper': {
                color: publishedWebsite ? 'success.main' : 'inherit',
              },
            }}
          />
          <Tab label="Sharing" icon={<ShareIcon />} iconPosition="start" />
        </Tabs>
      </Box>

      {/* Files Tab */}
      <TabPanel value={activeTab} index={0}>
        <Box sx={{ p: 2 }}>
          <List>
            {files.map(file => (
              <ListItem
                key={file.id}
                sx={{
                  border: 1,
                  borderColor: 'divider',
                  borderRadius: 1,
                  mb: 1,
                  '&:hover': {
                    bgcolor: 'action.hover',
                  },
                }}
              >
                <ListItemIcon>{getFileIcon(file)}</ListItemIcon>
                <ListItemText
                  primary={
                    <Stack direction="row" alignItems="center" spacing={1}>
                      <Typography variant="body1">{file.name}</Typography>
                      {file.networkIdentity && (
                        <Chip
                          size="small"
                          icon={<KeyIcon />}
                          label={file.networkIdentity.fourWords}
                          variant="outlined"
                        />
                      )}
                      {file.forwardIdentity && (
                        <Chip
                          size="small"
                          icon={<ForwardIcon />}
                          label="Forward"
                          color="primary"
                          variant="outlined"
                        />
                      )}
                    </Stack>
                  }
                  secondary={
                    <Stack direction="row" spacing={2}>
                      <Typography variant="caption">{formatFileSize(file.size)}</Typography>
                      <Typography variant="caption">v{file.version}</Typography>
                      <Typography variant="caption">
                        Modified {new Date(file.modifiedAt).toLocaleDateString()}
                      </Typography>
                      {file.sharedWith.length > 0 && (
                        <Chip
                          size="small"
                          label={`Shared with ${file.sharedWith.length}`}
                          sx={{ height: 16 }}
                        />
                      )}
                    </Stack>
                  }
                />
                <ListItemSecondaryAction>
                  <IconButton onClick={() => setSelectedFile(file)}>
                    <PreviewIcon />
                  </IconButton>
                  <IconButton onClick={() => {
                    setSelectedFile(file);
                    setShareDialog(true);
                  }}>
                    <ShareIcon />
                  </IconButton>
                  <IconButton>
                    <DownloadIcon />
                  </IconButton>
                  <IconButton
                    onClick={(e) => {
                      setAnchorEl(e.currentTarget);
                      setSelectedFile(file);
                    }}
                  >
                    <MoreIcon />
                  </IconButton>
                </ListItemSecondaryAction>
              </ListItem>
            ))}
          </List>
        </Box>
      </TabPanel>

      {/* Website Tab */}
      <TabPanel value={activeTab} index={1}>
        <Box sx={{ p: 2 }}>
          {publishedWebsite ? (
            <Card sx={{
              background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
              color: 'white',
            }}>
              <CardContent>
                <Stack direction="row" alignItems="center" spacing={1} sx={{ mb: 2 }}>
                  <PublishedIcon />
                  <Typography variant="h6">Website Published</Typography>
                </Stack>
                <Grid container spacing={2}>
                  <Grid item xs={12} md={6}>
                    <Typography variant="body2" sx={{ opacity: 0.9 }}>Domain</Typography>
                    <Stack direction="row" alignItems="center" spacing={1}>
                      <Typography variant="h6">{publishedWebsite.domain}</Typography>
                      <IconButton size="small" sx={{ color: 'inherit' }}>
                        <CopyIcon fontSize="small" />
                      </IconButton>
                    </Stack>
                  </Grid>
                  <Grid item xs={12} md={6}>
                    <Typography variant="body2" sx={{ opacity: 0.9 }}>Network Identity</Typography>
                    <Typography variant="body1">{publishedWebsite.networkIdentity.fourWords}</Typography>
                  </Grid>
                  <Grid item xs={12} md={4}>
                    <Typography variant="body2" sx={{ opacity: 0.9 }}>Files</Typography>
                    <Typography variant="h6">{publishedWebsite.files.length}</Typography>
                  </Grid>
                  <Grid item xs={12} md={4}>
                    <Typography variant="body2" sx={{ opacity: 0.9 }}>Views</Typography>
                    <Typography variant="h6">{publishedWebsite.analytics?.views || 0}</Typography>
                  </Grid>
                  <Grid item xs={12} md={4}>
                    <Typography variant="body2" sx={{ opacity: 0.9 }}>Bandwidth</Typography>
                    <Typography variant="h6">
                      {formatFileSize(publishedWebsite.analytics?.bandwidth || 0)}
                    </Typography>
                  </Grid>
                </Grid>
              </CardContent>
              <CardActions>
                <Button sx={{ color: 'inherit' }} startIcon={<PreviewIcon />}>
                  Preview Site
                </Button>
                <Button sx={{ color: 'inherit' }} startIcon={<EditIcon />}>
                  Edit Settings
                </Button>
                <Button 
                  sx={{ color: 'inherit' }} 
                  onClick={() => onUnpublishWebsite?.()}
                >
                  Unpublish
                </Button>
              </CardActions>
            </Card>
          ) : (
            <Card>
              <CardContent sx={{ textAlign: 'center', py: 4 }}>
                <UnpublishedIcon sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
                <Typography variant="h6" gutterBottom>
                  No Website Published
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
                  Publish a website from your markdown or HTML files
                </Typography>
                
                {websiteFiles.length > 0 ? (
                  <Box>
                    <Typography variant="body2" sx={{ mb: 2 }}>
                      Available files for website:
                    </Typography>
                    <Stack direction="row" spacing={1} justifyContent="center" flexWrap="wrap">
                      {websiteFiles.slice(0, 5).map(file => (
                        <Chip
                          key={file.id}
                          icon={file.name.endsWith('.md') ? <MarkdownIcon /> : <CodeIcon />}
                          label={file.name}
                          variant="outlined"
                        />
                      ))}
                      {websiteFiles.length > 5 && (
                        <Chip label={`+${websiteFiles.length - 5} more`} variant="outlined" />
                      )}
                    </Stack>
                  </Box>
                ) : (
                  <Alert severity="info" sx={{ mt: 2 }}>
                    Upload HTML or Markdown files to create a website
                  </Alert>
                )}
                
                <Button
                  variant="contained"
                  startIcon={<PublishIcon />}
                  onClick={() => setPublishDialog(true)}
                  disabled={websiteFiles.length === 0}
                  sx={{ mt: 3 }}
                >
                  Publish Website
                </Button>
              </CardContent>
            </Card>
          )}

          {/* Website Features */}
          <Box sx={{ mt: 3 }}>
            <Typography variant="h6" gutterBottom>
              Website Features
            </Typography>
            <Grid container spacing={2}>
              <Grid item xs={12} md={6}>
                <Paper sx={{ p: 2 }}>
                  <Stack direction="row" alignItems="center" spacing={2}>
                    <Avatar sx={{ bgcolor: 'primary.main' }}>
                      <MarkdownIcon />
                    </Avatar>
                    <Box>
                      <Typography variant="subtitle1">Markdown Support</Typography>
                      <Typography variant="body2" color="text.secondary">
                        Write content in Markdown, automatically converted to HTML
                      </Typography>
                    </Box>
                  </Stack>
                </Paper>
              </Grid>
              <Grid item xs={12} md={6}>
                <Paper sx={{ p: 2 }}>
                  <Stack direction="row" alignItems="center" spacing={2}>
                    <Avatar sx={{ bgcolor: 'secondary.main' }}>
                      <DnsIcon />
                    </Avatar>
                    <Box>
                      <Typography variant="subtitle1">Four-Word Domain</Typography>
                      <Typography variant="body2" color="text.secondary">
                        Memorable, unique domain like "ocean-forest-moon-star"
                      </Typography>
                    </Box>
                  </Stack>
                </Paper>
              </Grid>
              <Grid item xs={12} md={6}>
                <Paper sx={{ p: 2 }}>
                  <Stack direction="row" alignItems="center" spacing={2}>
                    <Avatar sx={{ bgcolor: 'success.main' }}>
                      <KeyIcon />
                    </Avatar>
                    <Box>
                      <Typography variant="subtitle1">Network Identity</Typography>
                      <Typography variant="body2" color="text.secondary">
                        Each file and website has its own P2P network identity
                      </Typography>
                    </Box>
                  </Stack>
                </Paper>
              </Grid>
              <Grid item xs={12} md={6}>
                <Paper sx={{ p: 2 }}>
                  <Stack direction="row" alignItems="center" spacing={2}>
                    <Avatar sx={{ bgcolor: 'warning.main' }}>
                      <ForwardIcon />
                    </Avatar>
                    <Box>
                      <Typography variant="subtitle1">Forward Identity</Typography>
                      <Typography variant="body2" color="text.secondary">
                        Optional forwarding for enhanced privacy and redundancy
                      </Typography>
                    </Box>
                  </Stack>
                </Paper>
              </Grid>
            </Grid>
          </Box>
        </Box>
      </TabPanel>

      {/* Sharing Tab */}
      <TabPanel value={activeTab} index={2}>
        <Box sx={{ p: 2 }}>
          <Typography variant="h6" gutterBottom>
            Sharing Settings
          </Typography>
          <List>
            {files.filter(f => f.sharedWith.length > 0).map(file => (
              <ListItem key={file.id}>
                <ListItemIcon>{getFileIcon(file)}</ListItemIcon>
                <ListItemText
                  primary={file.name}
                  secondary={`Shared with ${file.sharedWith.join(', ')}`}
                />
                <ListItemSecondaryAction>
                  <Button size="small" onClick={() => {
                    setSelectedFile(file);
                    setShareDialog(true);
                  }}>
                    Manage
                  </Button>
                </ListItemSecondaryAction>
              </ListItem>
            ))}
          </List>
        </Box>
      </TabPanel>

      {/* Publish Website Dialog */}
      <Dialog open={publishDialog} onClose={() => setPublishDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Publish Website</DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ mt: 2 }}>
            <TextField
              label="Website Name"
              value={websiteConfig.name}
              onChange={(e) => setWebsiteConfig({ ...websiteConfig, name: e.target.value })}
              fullWidth
            />
            <TextField
              label="Four-Word Domain"
              value={websiteConfig.domain}
              onChange={(e) => setWebsiteConfig({ ...websiteConfig, domain: e.target.value })}
              placeholder="Leave empty to auto-generate"
              helperText="e.g., ocean-forest-moon-star"
              fullWidth
              InputProps={{
                endAdornment: (
                  <InputAdornment position="end">
                    <Button
                      size="small"
                      onClick={() => {
                        const identity = onGenerateIdentity?.('website');
                        if (identity) {
                          setWebsiteConfig({ ...websiteConfig, domain: identity.fourWords });
                        }
                      }}
                    >
                      Generate
                    </Button>
                  </InputAdornment>
                ),
              }}
            />
            <TextField
              select
              label="Index File"
              value={websiteConfig.indexFile}
              onChange={(e) => setWebsiteConfig({ ...websiteConfig, indexFile: e.target.value })}
              fullWidth
            >
              {websiteFiles.map(file => (
                <MenuItem key={file.id} value={file.name}>
                  {file.name}
                </MenuItem>
              ))}
            </TextField>
            <FormControlLabel
              control={
                <Switch
                  checked={websiteConfig.enableAnalytics}
                  onChange={(e) => setWebsiteConfig({ ...websiteConfig, enableAnalytics: e.target.checked })}
                />
              }
              label="Enable Analytics"
            />
            <TextField
              select
              label="Theme"
              value={websiteConfig.theme}
              onChange={(e) => setWebsiteConfig({ ...websiteConfig, theme: e.target.value as any })}
              fullWidth
            >
              <MenuItem value="light">Light</MenuItem>
              <MenuItem value="dark">Dark</MenuItem>
              <MenuItem value="auto">Auto</MenuItem>
            </TextField>
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setPublishDialog(false)}>Cancel</Button>
          <Button variant="contained" onClick={handlePublishWebsite}>
            Publish
          </Button>
        </DialogActions>
      </Dialog>

      {/* Network Identity Dialog */}
      <Dialog open={identityDialog} onClose={() => setIdentityDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Network Identity</DialogTitle>
        <DialogContent>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
            Each file and website can have its own network identity for P2P access
          </Typography>
          <List>
            {files.filter(f => f.networkIdentity).map(file => (
              <ListItem key={file.id}>
                <ListItemIcon>{getFileIcon(file)}</ListItemIcon>
                <ListItemText
                  primary={file.name}
                  secondary={
                    <Stack spacing={1}>
                      <Chip
                        size="small"
                        icon={<KeyIcon />}
                        label={file.networkIdentity.fourWords}
                      />
                      {file.forwardIdentity && (
                        <Chip
                          size="small"
                          icon={<ForwardIcon />}
                          label={file.forwardIdentity.fourWords}
                          color="primary"
                        />
                      )}
                    </Stack>
                  }
                />
              </ListItem>
            ))}
          </List>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setIdentityDialog(false)}>Close</Button>
        </DialogActions>
      </Dialog>

      {/* File Actions Menu */}
      <Menu
        anchorEl={anchorEl}
        open={Boolean(anchorEl)}
        onClose={() => setAnchorEl(null)}
      >
        <MenuItem onClick={() => {
          const identity = onGenerateIdentity?.('file');
          setAnchorEl(null);
        }}>
          <ListItemIcon><KeyIcon fontSize="small" /></ListItemIcon>
          <ListItemText>Generate Identity</ListItemText>
        </MenuItem>
        <MenuItem onClick={() => {
          const identity = onGenerateIdentity?.('file');
          setAnchorEl(null);
        }}>
          <ListItemIcon><ForwardIcon fontSize="small" /></ListItemIcon>
          <ListItemText>Add Forward Identity</ListItemText>
        </MenuItem>
        <Divider />
        <MenuItem onClick={() => {
          if (selectedFile) onFileDelete?.(selectedFile.id);
          setAnchorEl(null);
        }}>
          <ListItemIcon><DeleteIcon fontSize="small" /></ListItemIcon>
          <ListItemText>Delete</ListItemText>
        </MenuItem>
      </Menu>
    </Box>
  );
};