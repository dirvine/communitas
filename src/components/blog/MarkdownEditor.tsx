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
  Preview as PreviewIcon,
  Edit as EditIcon,
  Save as SaveIcon,
  Settings as SettingsIcon,
  Publish as PublishIcon,
  Schedule as ScheduleIcon,
} from '@mui/icons-material';
import { motion } from 'framer-motion';
import { useResponsive, ResponsiveContainer } from '../responsive';

// Blog post interface
export interface BlogPost {
  id: string;
  title: string;
  content: string;
  excerpt?: string;
  tags: string[];
  category: string;
  status: 'draft' | 'published' | 'scheduled';
  publishDate: string;
  author: string;
  featuredImage?: string;
  seoTitle?: string;
  seoDescription?: string;
  slug: string;
  createdAt: string;
  updatedAt: string;
}

// Markdown toolbar button interface
interface ToolbarButton {
  icon: React.ReactNode;
  label: string;
  action: () => void;
  shortcut?: string;
}

// Markdown editor props
interface MarkdownEditorProps {
  initialPost?: Partial<BlogPost>;
  onSave?: (post: BlogPost) => void;
  onPublish?: (post: BlogPost) => void;
  onPreview?: (post: BlogPost) => void;
  readonly?: boolean;
}

export const MarkdownEditor: React.FC<MarkdownEditorProps> = ({
  initialPost,
  onSave,
  onPublish,
  onPreview,
  readonly = false,
}) => {
  const theme = useTheme();
  const { isMobile } = useResponsive();
  const textAreaRef = useRef<HTMLTextAreaElement>(null);
  const [activeTab, setActiveTab] = useState<'edit' | 'preview'>('edit');
  const [settingsOpen, setSettingsOpen] = useState(false);

  // Post state
  const [post, setPost] = useState<BlogPost>({
    id: initialPost?.id || '',
    title: initialPost?.title || '',
    content: initialPost?.content || '',
    excerpt: initialPost?.excerpt || '',
    tags: initialPost?.tags || [],
    category: initialPost?.category || 'general',
    status: initialPost?.status || 'draft',
    publishDate: initialPost?.publishDate || new Date().toISOString(),
    author: initialPost?.author || 'Unknown',
    featuredImage: initialPost?.featuredImage || '',
    seoTitle: initialPost?.seoTitle || '',
    seoDescription: initialPost?.seoDescription || '',
    slug: initialPost?.slug || '',
    createdAt: initialPost?.createdAt || new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  });

  // Tag input state
  const [tagInput, setTagInput] = useState('');

  // Auto-generate slug from title
  useEffect(() => {
    if (post.title && !initialPost?.slug) {
      const slug = post.title
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/(^-|-$)/g, '');
      setPost(prev => ({ ...prev, slug }));
    }
  }, [post.title, initialPost?.slug]);

  // Insert text at cursor position
  const insertText = useCallback((text: string) => {
    if (!textAreaRef.current) return;

    const textarea = textAreaRef.current;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const content = post.content;

    const newContent = content.substring(0, start) + text + content.substring(end);
    setPost(prev => ({ ...prev, content: newContent }));

    // Move cursor after inserted text
    setTimeout(() => {
      textarea.setSelectionRange(start + text.length, start + text.length);
      textarea.focus();
    }, 0);
  }, [post.content]);

  // Wrap selected text
  const wrapText = useCallback((prefix: string, suffix?: string) => {
    if (!textAreaRef.current) return;

    const textarea = textAreaRef.current;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const selectedText = post.content.substring(start, end);
    const wrappedText = `${prefix}${selectedText}${suffix || prefix}`;

    insertText(wrappedText);
  }, [post.content, insertText]);

  // Toolbar actions
  const toolbarButtons: ToolbarButton[] = [
    {
      icon: <BoldIcon />,
      label: 'Bold',
      action: () => wrapText('**'),
      shortcut: 'Ctrl+B',
    },
    {
      icon: <ItalicIcon />,
      label: 'Italic',
      action: () => wrapText('*'),
      shortcut: 'Ctrl+I',
    },
    {
      icon: <CodeIcon />,
      label: 'Code',
      action: () => wrapText('`'),
      shortcut: 'Ctrl+`',
    },
    {
      icon: <LinkIcon />,
      label: 'Link',
      action: () => insertText('[Link Text](https://example.com)'),
    },
    {
      icon: <ImageIcon />,
      label: 'Image',
      action: () => insertText('![Alt text](image-url)'),
    },
    {
      icon: <ListIcon />,
      label: 'Bullet List',
      action: () => insertText('\n- List item'),
    },
    {
      icon: <NumberedListIcon />,
      label: 'Numbered List',
      action: () => insertText('\n1. List item'),
    },
    {
      icon: <QuoteIcon />,
      label: 'Quote',
      action: () => insertText('\n> Quote text'),
    },
  ];

  // Handle keyboard shortcuts
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
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
          onSave?.(post);
          break;
      }
    }
  }, [wrapText, onSave, post]);

  // Add tag
  const handleAddTag = () => {
    if (tagInput.trim() && !post.tags.includes(tagInput.trim())) {
      setPost(prev => ({
        ...prev,
        tags: [...prev.tags, tagInput.trim()],
      }));
      setTagInput('');
    }
  };

  // Remove tag
  const handleRemoveTag = (tagToRemove: string) => {
    setPost(prev => ({
      ...prev,
      tags: prev.tags.filter(tag => tag !== tagToRemove),
    }));
  };

  // Simple markdown preview renderer
  const renderPreview = (markdown: string) => {
    // This is a simple preview - in production you'd use a proper markdown parser
    return markdown
      .replace(/^### (.+$)/gim, '<h3>$1</h3>')
      .replace(/^## (.+$)/gim, '<h2>$1</h2>')
      .replace(/^# (.+$)/gim, '<h1>$1</h1>')
      .replace(/\*\*(.+)\*\*/gim, '<strong>$1</strong>')
      .replace(/\*(.+)\*/gim, '<em>$1</em>')
      .replace(/`(.+)`/gim, '<code>$1</code>')
      .replace(/\n/gim, '<br>');
  };

  return (
    <ResponsiveContainer maxWidth="lg">
      <Paper
        component={motion.div}
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        sx={{
          overflow: 'hidden',
          height: 'calc(100vh - 200px)',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        {/* Header */}
        <Box
          sx={{
            p: 2,
            borderBottom: `1px solid ${theme.palette.divider}`,
            background: alpha(theme.palette.primary.main, 0.02),
          }}
        >
          <Stack
            direction={isMobile ? 'column' : 'row'}
            alignItems={isMobile ? 'stretch' : 'center'}
            justifyContent="space-between"
            spacing={2}
          >
            <TextField
              fullWidth
              variant="outlined"
              placeholder="Post Title"
              value={post.title}
              onChange={(e) => setPost(prev => ({ ...prev, title: e.target.value }))}
              disabled={readonly}
              sx={{
                '& .MuiOutlinedInput-input': {
                  fontSize: isMobile ? '1.25rem' : '1.5rem',
                  fontWeight: 600,
                  padding: isMobile ? '12px' : '16px',
                },
              }}
            />

            <Stack direction="row" spacing={1}>
              <IconButton
                onClick={() => setSettingsOpen(true)}
                disabled={readonly}
                size="small"
                sx={{ color: theme.palette.text.secondary }}
              >
                <SettingsIcon />
              </IconButton>

              <Button
                variant="outlined"
                startIcon={<SaveIcon />}
                onClick={() => onSave?.(post)}
                disabled={readonly}
                size="small"
              >
                Save
              </Button>

              <Button
                variant="contained"
                startIcon={<PublishIcon />}
                onClick={() => onPublish?.(post)}
                disabled={readonly}
                size="small"
              >
                Publish
              </Button>
            </Stack>
          </Stack>
        </Box>

        {/* Tabs */}
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
            />
            <Tab
              label="Preview"
              value="preview"
              icon={<PreviewIcon />}
              iconPosition="start"
            />
          </Tabs>
        </Box>

        {/* Edit Tab */}
        {activeTab === 'edit' && (
          <>
            {/* Toolbar */}
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
              >
                {toolbarButtons.map((button, index) => (
                  <IconButton
                    key={index}
                    size="small"
                    onClick={button.action}
                    disabled={readonly}
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
              </Stack>
            </Box>

            {/* Content Editor */}
            <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
              <TextField
                multiline
                fullWidth
                placeholder="Start writing your post in Markdown..."
                value={post.content}
                onChange={(e) => setPost(prev => ({ ...prev, content: e.target.value }))}
                onKeyDown={handleKeyDown}
                inputRef={textAreaRef}
                disabled={readonly}
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

        {/* Preview Tab */}
        {activeTab === 'preview' && (
          <Box
            sx={{
              flex: 1,
              p: 3,
              overflow: 'auto',
              '& h1': {
                fontSize: '2rem',
                fontWeight: 700,
                mb: 2,
                color: theme.palette.text.primary,
              },
              '& h2': {
                fontSize: '1.5rem',
                fontWeight: 600,
                mb: 1.5,
                mt: 2,
                color: theme.palette.text.primary,
              },
              '& h3': {
                fontSize: '1.25rem',
                fontWeight: 600,
                mb: 1,
                mt: 1.5,
                color: theme.palette.text.primary,
              },
              '& p': {
                mb: 1.5,
                lineHeight: 1.7,
                color: theme.palette.text.primary,
              },
              '& code': {
                backgroundColor: alpha(theme.palette.primary.main, 0.1),
                color: theme.palette.primary.main,
                padding: '2px 4px',
                borderRadius: 4,
                fontFamily: 'Monaco, Consolas, monospace',
                fontSize: '0.875rem',
              },
              '& strong': {
                fontWeight: 600,
              },
              '& em': {
                fontStyle: 'italic',
              },
            }}
          >
            <Typography variant="h4" gutterBottom>
              {post.title || 'Untitled Post'}
            </Typography>
            
            <Box
              dangerouslySetInnerHTML={{
                __html: renderPreview(post.content || 'Start writing to see the preview...'),
              }}
            />
          </Box>
        )}
      </Paper>

      {/* Settings Dialog */}
      <Dialog
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        maxWidth="sm"
        fullWidth
        fullScreen={isMobile}
      >
        <DialogTitle>Post Settings</DialogTitle>
        <DialogContent>
          <Stack spacing={3} sx={{ mt: 1 }}>
            {/* Category */}
            <FormControl fullWidth>
              <InputLabel>Category</InputLabel>
              <Select
                value={post.category}
                onChange={(e) => setPost(prev => ({ ...prev, category: e.target.value }))}
                label="Category"
              >
                <MenuItem value="general">General</MenuItem>
                <MenuItem value="technology">Technology</MenuItem>
                <MenuItem value="business">Business</MenuItem>
                <MenuItem value="personal">Personal</MenuItem>
                <MenuItem value="tutorial">Tutorial</MenuItem>
              </Select>
            </FormControl>

            {/* Status */}
            <FormControl fullWidth>
              <InputLabel>Status</InputLabel>
              <Select
                value={post.status}
                onChange={(e) => setPost(prev => ({ ...prev, status: e.target.value as BlogPost['status'] }))}
                label="Status"
              >
                <MenuItem value="draft">Draft</MenuItem>
                <MenuItem value="published">Published</MenuItem>
                <MenuItem value="scheduled">Scheduled</MenuItem>
              </Select>
            </FormControl>

            {/* Excerpt */}
            <TextField
              fullWidth
              multiline
              rows={3}
              label="Excerpt"
              value={post.excerpt}
              onChange={(e) => setPost(prev => ({ ...prev, excerpt: e.target.value }))}
              helperText="Brief summary of your post"
            />

            {/* Tags */}
            <Box>
              <Typography variant="subtitle2" gutterBottom>
                Tags
              </Typography>
              <Stack direction="row" spacing={1} mb={1} flexWrap="wrap" gap={1}>
                {post.tags.map((tag) => (
                  <Chip
                    key={tag}
                    label={tag}
                    onDelete={() => handleRemoveTag(tag)}
                    size="small"
                    color="primary"
                    variant="outlined"
                  />
                ))}
              </Stack>
              <Stack direction="row" spacing={1}>
                <TextField
                  size="small"
                  placeholder="Add tag"
                  value={tagInput}
                  onChange={(e) => setTagInput(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      e.preventDefault();
                      handleAddTag();
                    }
                  }}
                  sx={{ flex: 1 }}
                />
                <Button
                  variant="outlined"
                  onClick={handleAddTag}
                  size="small"
                >
                  Add
                </Button>
              </Stack>
            </Box>

            {/* SEO */}
            <Typography variant="h6" gutterBottom>
              SEO Settings
            </Typography>
            <TextField
              fullWidth
              label="SEO Title"
              value={post.seoTitle}
              onChange={(e) => setPost(prev => ({ ...prev, seoTitle: e.target.value }))}
              helperText="Title for search engines"
            />
            <TextField
              fullWidth
              multiline
              rows={2}
              label="SEO Description"
              value={post.seoDescription}
              onChange={(e) => setPost(prev => ({ ...prev, seoDescription: e.target.value }))}
              helperText="Description for search engines"
            />
            <TextField
              fullWidth
              label="Slug"
              value={post.slug}
              onChange={(e) => setPost(prev => ({ ...prev, slug: e.target.value }))}
              helperText="URL-friendly version of the title"
            />
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setSettingsOpen(false)}>Cancel</Button>
          <Button
            onClick={() => setSettingsOpen(false)}
            variant="contained"
          >
            Save Settings
          </Button>
        </DialogActions>
      </Dialog>
    </ResponsiveContainer>
  );
};

export default MarkdownEditor;