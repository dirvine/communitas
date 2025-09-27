import React, { useState, useEffect, useRef, useCallback } from 'react';
import {
  Box,
  Paper,
  IconButton,
  Button,
  Stack,
  Typography,
  TextField,
  Alert,
  CircularProgress,
  Tooltip,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Tabs,
  Tab,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  ListItemSecondaryAction,
  Divider,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Chip,
  Switch,
  FormControlLabel,
} from '@mui/material';
import {
  Edit as EditIcon,
  Preview as PreviewIcon,
  Code as CodeIcon,
  Save as SaveIcon,
  Publish as PublishIcon,
  Refresh as RefreshIcon,
  OpenInNew as OpenInNewIcon,
  Settings as SettingsIcon,
  CloudUpload as UploadIcon,
  Image as ImageIcon,
  VideoFile as VideoIcon,
  AudioFile as AudioIcon,
  Description as DocumentIcon,
  Add as AddIcon,
  Link as LinkIcon,
  FormatBold as BoldIcon,
  FormatItalic as ItalicIcon,
  FormatUnderlined as UnderlineIcon,
  FormatListBulleted as ListIcon,
  FormatListNumbered as NumberedListIcon,
  FormatQuote as QuoteIcon,
  Code as CodeBlockIcon,
  Title as HeadingIcon,
  InsertLink as InsertLinkIcon,
  InsertPhoto as InsertImageIcon,
  Undo as UndoIcon,
  Redo as RedoIcon,
  ContentCopy as CopyIcon,
  ContentPaste as PasteIcon,
  ContentCut as CutIcon,
  SelectAll as SelectAllIcon,
  Public as PublicIcon,
  Lock as PrivateIcon,
  History as HistoryIcon,
  CompareArrows as DiffIcon,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { materialDark } from 'react-syntax-highlighter/dist/esm/styles/prism';

interface WebsitePage {
  path: string;
  title: string;
  content: string;
  modified: string;
  published: boolean;
  public: boolean;
}

interface WebsiteSettings {
  site_title: string;
  site_description: string;
  theme: 'light' | 'dark' | 'auto';
  custom_css?: string;
  custom_js?: string;
  analytics_id?: string;
  favicon?: string;
  logo?: string;
}

interface WebsitePanelProps {
  entityType: 'individual' | 'group' | 'channel' | 'project';
  entityId: string;
  entityName: string;
  fourWords: string;
  permissions: string[];
}

const WebsitePanel: React.FC<WebsitePanelProps> = ({
  entityType,
  entityId,
  entityName,
  fourWords,
  permissions,
}) => {
  const [mode, setMode] = useState<'edit' | 'preview' | 'split'>('split');
  const [activeTab, setActiveTab] = useState(0);
  const [pages, setPages] = useState<WebsitePage[]>([]);
  const [currentPage, setCurrentPage] = useState<WebsitePage | null>(null);
  const [content, setContent] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [publishing, setPublishing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [settings, setSettings] = useState<WebsiteSettings | null>(null);
  const [settingsDialog, setSettingsDialog] = useState(false);
  const [newPageDialog, setNewPageDialog] = useState(false);
  const [newPagePath, setNewPagePath] = useState('');
  const [newPageTitle, setNewPageTitle] = useState('');
  const [history, setHistory] = useState<any[]>([]);
  const [historyDialog, setHistoryDialog] = useState(false);
  const [websiteUrl, setWebsiteUrl] = useState<string | null>(null);
  const editorRef = useRef<HTMLTextAreaElement>(null);
  const previewRef = useRef<HTMLDivElement>(null);

  const canEdit = permissions.includes('write') || permissions.includes('admin');
  const canPublish = permissions.includes('publish') || permissions.includes('admin');

  // Load website pages
  const loadPages = useCallback(async () => {
    try {
      setLoading(true);
      const result = await invoke('core_website_list_pages', {
        entityId,
      });
      setPages(result as WebsitePage[]);

      // Load settings
      const websiteSettings = await invoke('core_website_get_settings', {
        entityId,
      });
      setSettings(websiteSettings as WebsiteSettings);

      // Get website URL
      const url = await invoke('core_website_get_url', {
        entityId,
        fourWords,
      });
      setWebsiteUrl(url as string);

      // Load index page by default
      const indexPage = (result as WebsitePage[]).find(p => p.path === '/index.md');
      if (indexPage) {
        setCurrentPage(indexPage);
        setContent(indexPage.content);
      }

      setError(null);
    } catch (err) {
      console.error('Failed to load website pages:', err);
      setError('Failed to load website pages');
    } finally {
      setLoading(false);
    }
  }, [entityId, fourWords]);

  // Save page
  const savePage = async () => {
    if (!currentPage || !canEdit) return;

    setSaving(true);
    try {
      await invoke('core_website_save_page', {
        entityId,
        path: currentPage.path,
        content,
        title: currentPage.title,
      });

      // Update local state
      setCurrentPage({ ...currentPage, content, modified: new Date().toISOString() });
      setPages(pages.map(p =>
        p.path === currentPage.path
          ? { ...p, content, modified: new Date().toISOString() }
          : p
      ));

      setError(null);
    } catch (err) {
      console.error('Failed to save page:', err);
      setError('Failed to save page');
    } finally {
      setSaving(false);
    }
  };

  // Publish website
  const publishWebsite = async () => {
    if (!canPublish) return;

    setPublishing(true);
    try {
      // Build website
      const rootHash = await invoke('core_website_build', {
        entityId,
      });

      // Publish to network
      await invoke('core_website_publish', {
        entityId,
        rootHash,
      });

      // Update all pages as published
      setPages(pages.map(p => ({ ...p, published: true })));

      setError(null);
    } catch (err) {
      console.error('Failed to publish website:', err);
      setError('Failed to publish website');
    } finally {
      setPublishing(false);
    }
  };

  // Create new page
  const createPage = async () => {
    if (!newPagePath.trim() || !newPageTitle.trim() || !canEdit) return;

    try {
      const path = newPagePath.startsWith('/') ? newPagePath : `/${newPagePath}`;
      const fullPath = path.endsWith('.md') ? path : `${path}.md`;

      await invoke('core_website_create_page', {
        entityId,
        path: fullPath,
        title: newPageTitle.trim(),
        content: `# ${newPageTitle}\n\n`,
      });

      setNewPageDialog(false);
      setNewPagePath('');
      setNewPageTitle('');
      await loadPages();
    } catch (err) {
      console.error('Failed to create page:', err);
      setError('Failed to create page');
    }
  };

  // Insert formatting
  const insertFormatting = (prefix: string, suffix: string = '') => {
    if (!editorRef.current) return;

    const start = editorRef.current.selectionStart;
    const end = editorRef.current.selectionEnd;
    const selectedText = content.substring(start, end);
    const newContent =
      content.substring(0, start) +
      prefix +
      selectedText +
      suffix +
      content.substring(end);

    setContent(newContent);
    editorRef.current.focus();
    setTimeout(() => {
      if (editorRef.current) {
        editorRef.current.selectionStart = start + prefix.length;
        editorRef.current.selectionEnd = start + prefix.length + selectedText.length;
      }
    }, 0);
  };

  // Markdown components for preview
  const markdownComponents = {
    code({ node, inline, className, children, ...props }: any) {
      const match = /language-(\w+)/.exec(className || '');
      return !inline && match ? (
        <SyntaxHighlighter
          style={materialDark}
          language={match[1]}
          PreTag="div"
          {...props}
        >
          {String(children).replace(/\n$/, '')}
        </SyntaxHighlighter>
      ) : (
        <code className={className} {...props}>
          {children}
        </code>
      );
    },
  };

  useEffect(() => {
    loadPages();
  }, [loadPages]);

  if (loading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%' }}>
        <CircularProgress />
      </Box>
    );
  }

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Toolbar */}
      <Paper elevation={0} sx={{ p: 1, borderBottom: 1, borderColor: 'divider' }}>
        <Stack direction="row" alignItems="center" justifyContent="space-between">
          <Stack direction="row" spacing={1} alignItems="center">
            {/* Mode Toggle */}
            <Tabs value={mode} onChange={(e, v) => setMode(v)} sx={{ minHeight: 36 }}>
              <Tab value="edit" icon={<EditIcon />} label="Edit" sx={{ minHeight: 36, py: 0 }} />
              <Tab value="preview" icon={<PreviewIcon />} label="Preview" sx={{ minHeight: 36, py: 0 }} />
              <Tab value="split" icon={<CodeIcon />} label="Split" sx={{ minHeight: 36, py: 0 }} />
            </Tabs>

            <Divider orientation="vertical" flexItem />

            {/* Editor Tools */}
            {(mode === 'edit' || mode === 'split') && canEdit && (
              <>
                <Tooltip title="Bold">
                  <IconButton size="small" onClick={() => insertFormatting('**', '**')}>
                    <BoldIcon />
                  </IconButton>
                </Tooltip>
                <Tooltip title="Italic">
                  <IconButton size="small" onClick={() => insertFormatting('*', '*')}>
                    <ItalicIcon />
                  </IconButton>
                </Tooltip>
                <Tooltip title="Heading">
                  <IconButton size="small" onClick={() => insertFormatting('## ', '\n')}>
                    <HeadingIcon />
                  </IconButton>
                </Tooltip>
                <Tooltip title="Link">
                  <IconButton size="small" onClick={() => insertFormatting('[', '](url)')}>
                    <InsertLinkIcon />
                  </IconButton>
                </Tooltip>
                <Tooltip title="Image">
                  <IconButton size="small" onClick={() => insertFormatting('![', '](image-url)')}>
                    <InsertImageIcon />
                  </IconButton>
                </Tooltip>
                <Tooltip title="Code Block">
                  <IconButton size="small" onClick={() => insertFormatting('```\n', '\n```')}>
                    <CodeBlockIcon />
                  </IconButton>
                </Tooltip>
                <Tooltip title="Bullet List">
                  <IconButton size="small" onClick={() => insertFormatting('- ', '')}>
                    <ListIcon />
                  </IconButton>
                </Tooltip>
                <Tooltip title="Quote">
                  <IconButton size="small" onClick={() => insertFormatting('> ', '')}>
                    <QuoteIcon />
                  </IconButton>
                </Tooltip>

                <Divider orientation="vertical" flexItem />
              </>
            )}
          </Stack>

          <Stack direction="row" spacing={1}>
            {/* Website URL */}
            {websiteUrl && (
              <Chip
                icon={<PublicIcon />}
                label={websiteUrl}
                onClick={() => window.open(websiteUrl, '_blank')}
                deleteIcon={<OpenInNewIcon />}
                onDelete={() => window.open(websiteUrl, '_blank')}
              />
            )}

            {/* Actions */}
            {canEdit && (
              <>
                <Button
                  startIcon={saving ? <CircularProgress size={16} /> : <SaveIcon />}
                  onClick={savePage}
                  disabled={saving || !currentPage}
                  variant="outlined"
                >
                  Save
                </Button>

                {canPublish && (
                  <Button
                    startIcon={publishing ? <CircularProgress size={16} /> : <PublishIcon />}
                    onClick={publishWebsite}
                    disabled={publishing}
                    variant="contained"
                    color="primary"
                  >
                    Publish
                  </Button>
                )}

                <IconButton onClick={() => setSettingsDialog(true)}>
                  <SettingsIcon />
                </IconButton>

                <IconButton onClick={() => setHistoryDialog(true)}>
                  <HistoryIcon />
                </IconButton>
              </>
            )}

            <IconButton onClick={loadPages}>
              <RefreshIcon />
            </IconButton>
          </Stack>
        </Stack>
      </Paper>

      {/* Error Alert */}
      {error && (
        <Alert severity="error" onClose={() => setError(null)} sx={{ m: 1 }}>
          {error}
        </Alert>
      )}

      {/* Main Content */}
      <Box sx={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        {/* Pages Sidebar */}
        <Paper
          elevation={0}
          sx={{
            width: 240,
            borderRight: 1,
            borderColor: 'divider',
            overflow: 'auto',
          }}
        >
          <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
            <Stack direction="row" alignItems="center" justifyContent="space-between">
              <Typography variant="subtitle2">Pages</Typography>
              {canEdit && (
                <IconButton size="small" onClick={() => setNewPageDialog(true)}>
                  <AddIcon />
                </IconButton>
              )}
            </Stack>
          </Box>

          <List dense>
            {pages.map(page => (
              <ListItem
                key={page.path}
                button
                selected={currentPage?.path === page.path}
                onClick={() => {
                  setCurrentPage(page);
                  setContent(page.content);
                }}
              >
                <ListItemIcon>
                  <DocumentIcon />
                </ListItemIcon>
                <ListItemText
                  primary={page.title}
                  secondary={page.path}
                />
                <ListItemSecondaryAction>
                  <Stack direction="row" spacing={0.5}>
                    {page.published && (
                      <Tooltip title="Published">
                        <PublicIcon fontSize="small" color="success" />
                      </Tooltip>
                    )}
                    {page.public ? (
                      <Tooltip title="Public">
                        <PublicIcon fontSize="small" />
                      </Tooltip>
                    ) : (
                      <Tooltip title="Private">
                        <PrivateIcon fontSize="small" />
                      </Tooltip>
                    )}
                  </Stack>
                </ListItemSecondaryAction>
              </ListItem>
            ))}
          </List>
        </Paper>

        {/* Editor/Preview */}
        <Box sx={{ flex: 1, display: 'flex' }}>
          {/* Editor */}
          {(mode === 'edit' || mode === 'split') && (
            <Box
              sx={{
                flex: mode === 'split' ? 1 : undefined,
                width: mode === 'edit' ? '100%' : undefined,
                display: 'flex',
                flexDirection: 'column',
              }}
            >
              <TextField
                inputRef={editorRef}
                multiline
                fullWidth
                value={content}
                onChange={(e) => setContent(e.target.value)}
                disabled={!canEdit}
                sx={{
                  flex: 1,
                  '& .MuiInputBase-root': {
                    fontFamily: 'monospace',
                    fontSize: '14px',
                    height: '100%',
                  },
                  '& .MuiInputBase-input': {
                    height: '100% !important',
                    overflow: 'auto !important',
                  },
                }}
                InputProps={{
                  sx: {
                    p: 2,
                    alignItems: 'flex-start',
                  },
                }}
              />
            </Box>
          )}

          {/* Divider */}
          {mode === 'split' && (
            <Divider orientation="vertical" flexItem />
          )}

          {/* Preview */}
          {(mode === 'preview' || mode === 'split') && (
            <Box
              ref={previewRef}
              sx={{
                flex: mode === 'split' ? 1 : undefined,
                width: mode === 'preview' ? '100%' : undefined,
                overflow: 'auto',
                p: 3,
                bgcolor: 'background.default',
              }}
            >
              <Paper
                elevation={0}
                sx={{
                  p: 4,
                  maxWidth: 800,
                  mx: 'auto',
                  minHeight: '100%',
                  '& h1': { mt: 0 },
                  '& img': { maxWidth: '100%', height: 'auto' },
                  '& pre': {
                    borderRadius: 1,
                    overflow: 'auto',
                  },
                  '& code': {
                    bgcolor: 'action.hover',
                    px: 0.5,
                    py: 0.25,
                    borderRadius: 0.5,
                    fontSize: '0.875em',
                  },
                  '& blockquote': {
                    borderLeft: '4px solid',
                    borderColor: 'primary.main',
                    pl: 2,
                    ml: 0,
                    color: 'text.secondary',
                  },
                  '& table': {
                    width: '100%',
                    borderCollapse: 'collapse',
                    '& th, & td': {
                      border: '1px solid',
                      borderColor: 'divider',
                      p: 1,
                    },
                    '& th': {
                      bgcolor: 'action.hover',
                      fontWeight: 'bold',
                    },
                  },
                }}
              >
                <ReactMarkdown
                  remarkPlugins={[remarkGfm]}
                  components={markdownComponents}
                >
                  {content}
                </ReactMarkdown>
              </Paper>
            </Box>
          )}
        </Box>
      </Box>

      {/* Settings Dialog */}
      <Dialog open={settingsDialog} onClose={() => setSettingsDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Website Settings</DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ mt: 1 }}>
            <TextField
              label="Site Title"
              fullWidth
              value={settings?.site_title || ''}
              onChange={(e) => setSettings(settings ? { ...settings, site_title: e.target.value } : null)}
            />
            <TextField
              label="Site Description"
              fullWidth
              multiline
              rows={3}
              value={settings?.site_description || ''}
              onChange={(e) => setSettings(settings ? { ...settings, site_description: e.target.value } : null)}
            />
            <FormControl fullWidth>
              <InputLabel>Theme</InputLabel>
              <Select
                value={settings?.theme || 'auto'}
                onChange={(e) => setSettings(settings ? { ...settings, theme: e.target.value as any } : null)}
              >
                <MenuItem value="light">Light</MenuItem>
                <MenuItem value="dark">Dark</MenuItem>
                <MenuItem value="auto">Auto</MenuItem>
              </Select>
            </FormControl>
            <TextField
              label="Analytics ID"
              fullWidth
              value={settings?.analytics_id || ''}
              onChange={(e) => setSettings(settings ? { ...settings, analytics_id: e.target.value } : null)}
            />
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setSettingsDialog(false)}>Cancel</Button>
          <Button
            variant="contained"
            onClick={async () => {
              if (settings) {
                try {
                  await invoke('core_website_save_settings', {
                    entityId,
                    settings,
                  });
                  setSettingsDialog(false);
                } catch (err) {
                  console.error('Failed to save settings:', err);
                  setError('Failed to save settings');
                }
              }
            }}
          >
            Save Settings
          </Button>
        </DialogActions>
      </Dialog>

      {/* New Page Dialog */}
      <Dialog open={newPageDialog} onClose={() => setNewPageDialog(false)}>
        <DialogTitle>Create New Page</DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ mt: 1 }}>
            <TextField
              label="Page Path"
              fullWidth
              value={newPagePath}
              onChange={(e) => setNewPagePath(e.target.value)}
              placeholder="/about.md"
              helperText="Path must start with / and end with .md"
            />
            <TextField
              label="Page Title"
              fullWidth
              value={newPageTitle}
              onChange={(e) => setNewPageTitle(e.target.value)}
              placeholder="About Us"
            />
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setNewPageDialog(false)}>Cancel</Button>
          <Button variant="contained" onClick={createPage}>Create</Button>
        </DialogActions>
      </Dialog>

      {/* History Dialog */}
      <Dialog open={historyDialog} onClose={() => setHistoryDialog(false)} maxWidth="md" fullWidth>
        <DialogTitle>Page History</DialogTitle>
        <DialogContent>
          <List>
            {history.map((entry, index) => (
              <ListItem key={index}>
                <ListItemText
                  primary={entry.message}
                  secondary={new Date(entry.timestamp).toLocaleString()}
                />
                <ListItemSecondaryAction>
                  <Button size="small" onClick={() => setContent(entry.content)}>
                    Restore
                  </Button>
                </ListItemSecondaryAction>
              </ListItem>
            ))}
          </List>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setHistoryDialog(false)}>Close</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default WebsitePanel;
