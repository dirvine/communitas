import React, { useState, useEffect } from 'react';
import {
  Box,
  Paper,
  Typography,
  Button,
  IconButton,
  Stack,
  Grid,
  Card,
  CardContent,
  CardActions,
  Chip,
  TextField,
  InputAdornment,
  Menu,
  MenuItem,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  FormControl,
  InputLabel,
  Select,
  Tab,
  Tabs,
  Fab,
  useTheme,
  alpha,
} from '@mui/material';
import {
  Add as AddIcon,
  Edit as EditIcon,
  Delete as DeleteIcon,
  Preview as PreviewIcon,
  Publish as PublishIcon,
  Schedule as ScheduleIcon,
  Search as SearchIcon,
  FilterList as FilterIcon,
  MoreVert as MoreVertIcon,
  Article as ArticleIcon,
  Web as WebIcon,
  Dashboard as DashboardIcon,
  Settings as SettingsIcon,
  Analytics as AnalyticsIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { useResponsive, ResponsiveContainer, ResponsiveGrid, ResponsiveCard } from '../responsive';
import { BlogPost, MarkdownEditor } from './MarkdownEditor';

// Blog statistics interface
interface BlogStats {
  totalPosts: number;
  publishedPosts: number;
  draftPosts: number;
  scheduledPosts: number;
  totalViews: number;
  monthlyViews: number;
}

// Website page interface
interface WebsitePage {
  id: string;
  title: string;
  content: string;
  slug: string;
  status: 'published' | 'draft';
  isHomePage: boolean;
  menuOrder: number;
  createdAt: string;
  updatedAt: string;
}

// Blog manager props
interface BlogManagerProps {
  initialPosts?: BlogPost[];
  initialPages?: WebsitePage[];
  onSavePost?: (post: BlogPost) => void;
  onDeletePost?: (postId: string) => void;
  onSavePage?: (page: WebsitePage) => void;
  onDeletePage?: (pageId: string) => void;
}

export const BlogManager: React.FC<BlogManagerProps> = ({
  initialPosts = [],
  initialPages = [],
  onSavePost,
  onDeletePost,
  onSavePage,
  onDeletePage,
}) => {
  const theme = useTheme();
  const { isMobile, isTablet } = useResponsive();
  
  // State management
  const [activeTab, setActiveTab] = useState<'posts' | 'pages' | 'analytics' | 'settings'>('posts');
  const [posts, setPosts] = useState<BlogPost[]>(initialPosts);
  const [pages, setPages] = useState<WebsitePage[]>(initialPages);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterStatus, setFilterStatus] = useState<string>('all');
  const [filterCategory, setFilterCategory] = useState<string>('all');
  const [selectedPost, setSelectedPost] = useState<BlogPost | null>(null);
  const [selectedPage, setSelectedPage] = useState<WebsitePage | null>(null);
  const [editorOpen, setEditorOpen] = useState(false);
  const [deleteConfirmOpen, setDeleteConfirmOpen] = useState(false);
  const [itemToDelete, setItemToDelete] = useState<{ type: 'post' | 'page'; id: string } | null>(null);
  const [menuAnchor, setMenuAnchor] = useState<null | HTMLElement>(null);
  
  // Mock statistics - in real app, fetch from backend
  const [stats, setStats] = useState<BlogStats>({
    totalPosts: posts.length,
    publishedPosts: posts.filter(p => p.status === 'published').length,
    draftPosts: posts.filter(p => p.status === 'draft').length,
    scheduledPosts: posts.filter(p => p.status === 'scheduled').length,
    totalViews: 12543,
    monthlyViews: 2156,
  });

  // Update stats when posts change
  useEffect(() => {
    setStats({
      totalPosts: posts.length,
      publishedPosts: posts.filter(p => p.status === 'published').length,
      draftPosts: posts.filter(p => p.status === 'draft').length,
      scheduledPosts: posts.filter(p => p.status === 'scheduled').length,
      totalViews: 12543, // Mock data
      monthlyViews: 2156, // Mock data
    });
  }, [posts]);

  // Filter posts based on search and filters
  const filteredPosts = posts.filter(post => {
    const matchesSearch = post.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         post.content.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesStatus = filterStatus === 'all' || post.status === filterStatus;
    const matchesCategory = filterCategory === 'all' || post.category === filterCategory;
    
    return matchesSearch && matchesStatus && matchesCategory;
  });

  // Get unique categories
  const categories = Array.from(new Set(posts.map(post => post.category)));

  // Handle create new post
  const handleCreatePost = () => {
    setSelectedPost(null);
    setEditorOpen(true);
  };

  // Handle create new page
  const handleCreatePage = () => {
    const newPage: WebsitePage = {
      id: Date.now().toString(),
      title: '',
      content: '',
      slug: '',
      status: 'draft',
      isHomePage: false,
      menuOrder: pages.length,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    setSelectedPage(newPage);
    setEditorOpen(true);
  };

  // Handle edit post
  const handleEditPost = (post: BlogPost) => {
    setSelectedPost(post);
    setEditorOpen(true);
  };

  // Handle edit page
  const handleEditPage = (page: WebsitePage) => {
    setSelectedPage(page);
    setEditorOpen(true);
  };

  // Handle save post
  const handleSavePost = (post: BlogPost) => {
    const updatedPost = {
      ...post,
      id: post.id || Date.now().toString(),
      updatedAt: new Date().toISOString(),
    };

    if (selectedPost) {
      // Update existing post
      setPosts(prev => prev.map(p => p.id === post.id ? updatedPost : p));
    } else {
      // Create new post
      updatedPost.id = Date.now().toString();
      updatedPost.createdAt = new Date().toISOString();
      setPosts(prev => [...prev, updatedPost]);
    }

    onSavePost?.(updatedPost);
    setEditorOpen(false);
    setSelectedPost(null);
  };

  // Handle save page
  const handleSavePage = (pageData: Partial<WebsitePage>) => {
    const updatedPage: WebsitePage = {
      ...selectedPage!,
      ...pageData,
      updatedAt: new Date().toISOString(),
    };

    if (pages.find(p => p.id === updatedPage.id)) {
      // Update existing page
      setPages(prev => prev.map(p => p.id === updatedPage.id ? updatedPage : p));
    } else {
      // Create new page
      setPages(prev => [...prev, updatedPage]);
    }

    onSavePage?.(updatedPage);
    setEditorOpen(false);
    setSelectedPage(null);
  };

  // Handle delete confirmation
  const handleDeleteClick = (type: 'post' | 'page', id: string) => {
    setItemToDelete({ type, id });
    setDeleteConfirmOpen(true);
    setMenuAnchor(null);
  };

  // Handle confirmed delete
  const handleConfirmDelete = () => {
    if (!itemToDelete) return;

    if (itemToDelete.type === 'post') {
      setPosts(prev => prev.filter(p => p.id !== itemToDelete.id));
      onDeletePost?.(itemToDelete.id);
    } else {
      setPages(prev => prev.filter(p => p.id !== itemToDelete.id));
      onDeletePage?.(itemToDelete.id);
    }

    setDeleteConfirmOpen(false);
    setItemToDelete(null);
  };

  // Statistics cards
  const statsCards = [
    { label: 'Total Posts', value: stats.totalPosts, icon: <ArticleIcon />, color: theme.palette.primary.main },
    { label: 'Published', value: stats.publishedPosts, icon: <PublishIcon />, color: theme.palette.success.main },
    { label: 'Drafts', value: stats.draftPosts, icon: <EditIcon />, color: theme.palette.warning.main },
    { label: 'Scheduled', value: stats.scheduledPosts, icon: <ScheduleIcon />, color: theme.palette.info.main },
  ];

  return (
    <ResponsiveContainer maxWidth="xl">
      <Box>
        {/* Header */}
        <Stack
          direction={isMobile ? 'column' : 'row'}
          alignItems={isMobile ? 'stretch' : 'center'}
          justifyContent="space-between"
          spacing={2}
          mb={3}
        >
          <Typography variant={isMobile ? 'h5' : 'h4'} fontWeight={700}>
            Content Management
          </Typography>
          
          {/* Action buttons */}
          <Stack direction="row" spacing={1}>
            <Button
              variant="outlined"
              startIcon={<WebIcon />}
              onClick={handleCreatePage}
            >
              New Page
            </Button>
            <Button
              variant="contained"
              startIcon={<AddIcon />}
              onClick={handleCreatePost}
            >
              New Post
            </Button>
          </Stack>
        </Stack>

        {/* Tabs */}
        <Paper sx={{ mb: 3 }}>
          <Tabs
            value={activeTab}
            onChange={(_, newValue) => setActiveTab(newValue)}
            variant={isMobile ? 'scrollable' : 'standard'}
            scrollButtons="auto"
          >
            <Tab label="Blog Posts" value="posts" icon={<ArticleIcon />} iconPosition="start" />
            <Tab label="Website Pages" value="pages" icon={<WebIcon />} iconPosition="start" />
            <Tab label="Analytics" value="analytics" icon={<AnalyticsIcon />} iconPosition="start" />
            <Tab label="Settings" value="settings" icon={<SettingsIcon />} iconPosition="start" />
          </Tabs>
        </Paper>

        {/* Posts Tab */}
        {activeTab === 'posts' && (
          <>
            {/* Statistics */}
            <ResponsiveGrid 
              columns={{ xs: 2, sm: 4 }} 
              spacing={2}
              sx={{ mb: 3 }}
            >
              {statsCards.map((stat, index) => (
                <motion.div
                  key={index}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: index * 0.1 }}
                >
                  <Card>
                    <CardContent sx={{ textAlign: 'center', py: 2 }}>
                      <Box sx={{ color: stat.color, mb: 1 }}>
                        {stat.icon}
                      </Box>
                      <Typography variant="h4" fontWeight={700}>
                        {stat.value}
                      </Typography>
                      <Typography variant="body2" color="text.secondary">
                        {stat.label}
                      </Typography>
                    </CardContent>
                  </Card>
                </motion.div>
              ))}
            </ResponsiveGrid>

            {/* Filters */}
            <Paper sx={{ p: 2, mb: 3 }}>
              <Stack
                direction={isMobile ? 'column' : 'row'}
                spacing={2}
                alignItems={isMobile ? 'stretch' : 'center'}
              >
                <TextField
                  placeholder="Search posts..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  InputProps={{
                    startAdornment: (
                      <InputAdornment position="start">
                        <SearchIcon />
                      </InputAdornment>
                    ),
                  }}
                  sx={{ flex: 1 }}
                />

                <FormControl sx={{ minWidth: 120 }}>
                  <InputLabel>Status</InputLabel>
                  <Select
                    value={filterStatus}
                    onChange={(e) => setFilterStatus(e.target.value)}
                    label="Status"
                  >
                    <MenuItem value="all">All Status</MenuItem>
                    <MenuItem value="published">Published</MenuItem>
                    <MenuItem value="draft">Draft</MenuItem>
                    <MenuItem value="scheduled">Scheduled</MenuItem>
                  </Select>
                </FormControl>

                <FormControl sx={{ minWidth: 120 }}>
                  <InputLabel>Category</InputLabel>
                  <Select
                    value={filterCategory}
                    onChange={(e) => setFilterCategory(e.target.value)}
                    label="Category"
                  >
                    <MenuItem value="all">All Categories</MenuItem>
                    {categories.map(category => (
                      <MenuItem key={category} value={category}>
                        {category}
                      </MenuItem>
                    ))}
                  </Select>
                </FormControl>
              </Stack>
            </Paper>

            {/* Posts Grid */}
            <ResponsiveGrid columns={{ xs: 1, sm: 2, lg: 3 }} spacing={3}>
              <AnimatePresence>
                {filteredPosts.map((post) => (
                  <motion.div
                    key={post.id}
                    initial={{ opacity: 0, scale: 0.9 }}
                    animate={{ opacity: 1, scale: 1 }}
                    exit={{ opacity: 0, scale: 0.9 }}
                    transition={{ duration: 0.2 }}
                  >
                    <ResponsiveCard
                      title={post.title || 'Untitled Post'}
                      subtitle={new Date(post.updatedAt).toLocaleDateString()}
                      content={
                        <Box>
                          <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
                            {post.excerpt || post.content.substring(0, 150) + '...'}
                          </Typography>
                          <Stack direction="row" spacing={1} flexWrap="wrap" gap={0.5}>
                            <Chip
                              size="small"
                              label={post.status}
                              color={
                                post.status === 'published' ? 'success' :
                                post.status === 'draft' ? 'warning' : 'info'
                              }
                              variant="outlined"
                            />
                            <Chip
                              size="small"
                              label={post.category}
                              variant="outlined"
                            />
                          </Stack>
                        </Box>
                      }
                      actions={
                        <Stack direction="row" spacing={1} width="100%">
                          <Button
                            size="small"
                            startIcon={<EditIcon />}
                            onClick={() => handleEditPost(post)}
                            fullWidth={isMobile}
                          >
                            Edit
                          </Button>
                          <Button
                            size="small"
                            startIcon={<PreviewIcon />}
                            fullWidth={isMobile}
                          >
                            Preview
                          </Button>
                          <IconButton
                            size="small"
                            onClick={(e) => {
                              setMenuAnchor(e.currentTarget);
                              setSelectedPost(post);
                            }}
                          >
                            <MoreVertIcon />
                          </IconButton>
                        </Stack>
                      }
                    />
                  </motion.div>
                ))}
              </AnimatePresence>
            </ResponsiveGrid>
          </>
        )}

        {/* Pages Tab */}
        {activeTab === 'pages' && (
          <ResponsiveGrid columns={{ xs: 1, sm: 2, lg: 3 }} spacing={3}>
            <AnimatePresence>
              {pages.map((page) => (
                <motion.div
                  key={page.id}
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.9 }}
                >
                  <ResponsiveCard
                    title={page.title || 'Untitled Page'}
                    subtitle={`/${page.slug}`}
                    content={
                      <Stack direction="row" spacing={1}>
                        <Chip
                          size="small"
                          label={page.status}
                          color={page.status === 'published' ? 'success' : 'warning'}
                          variant="outlined"
                        />
                        {page.isHomePage && (
                          <Chip
                            size="small"
                            label="Home Page"
                            color="primary"
                            variant="outlined"
                          />
                        )}
                      </Stack>
                    }
                    actions={
                      <Stack direction="row" spacing={1} width="100%">
                        <Button
                          size="small"
                          startIcon={<EditIcon />}
                          onClick={() => handleEditPage(page)}
                          fullWidth={isMobile}
                        >
                          Edit
                        </Button>
                        <IconButton
                          size="small"
                          onClick={() => handleDeleteClick('page', page.id)}
                        >
                          <DeleteIcon />
                        </IconButton>
                      </Stack>
                    }
                  />
                </motion.div>
              ))}
            </AnimatePresence>
          </ResponsiveGrid>
        )}

        {/* Analytics Tab */}
        {activeTab === 'analytics' && (
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              Analytics Dashboard
            </Typography>
            <Typography color="text.secondary">
              Analytics features coming soon. This will show detailed statistics about your blog performance.
            </Typography>
          </Paper>
        )}

        {/* Settings Tab */}
        {activeTab === 'settings' && (
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              Blog Settings
            </Typography>
            <Typography color="text.secondary">
              Blog configuration options will be available here.
            </Typography>
          </Paper>
        )}

        {/* Floating Action Button for mobile */}
        {isMobile && (
          <Fab
            color="primary"
            sx={{ position: 'fixed', bottom: 16, right: 16 }}
            onClick={handleCreatePost}
          >
            <AddIcon />
          </Fab>
        )}
      </Box>

      {/* Context Menu */}
      <Menu
        anchorEl={menuAnchor}
        open={Boolean(menuAnchor)}
        onClose={() => setMenuAnchor(null)}
      >
        <MenuItem onClick={() => {
          if (selectedPost) handleEditPost(selectedPost);
          setMenuAnchor(null);
        }}>
          <EditIcon sx={{ mr: 1 }} /> Edit
        </MenuItem>
        <MenuItem onClick={() => {
          if (selectedPost) handleDeleteClick('post', selectedPost.id);
        }}>
          <DeleteIcon sx={{ mr: 1 }} /> Delete
        </MenuItem>
      </Menu>

      {/* Editor Dialog */}
      <Dialog
        open={editorOpen}
        onClose={() => {
          setEditorOpen(false);
          setSelectedPost(null);
          setSelectedPage(null);
        }}
        maxWidth={false}
        fullScreen
        sx={{
          '& .MuiDialog-paper': {
            m: 0,
            maxHeight: '100vh',
          },
        }}
      >
        {selectedPost !== null && (
          <MarkdownEditor
            initialPost={selectedPost}
            onSave={handleSavePost}
          />
        )}
        {selectedPage !== null && (
          <MarkdownEditor
            initialPost={{
              ...selectedPage,
              content: selectedPage.content,
              tags: [],
              category: 'page',
              status: selectedPage.status === 'published' ? 'published' : 'draft',
              author: 'Admin',
              publishDate: new Date().toISOString(),
            }}
            onSave={(post) => handleSavePage({
              title: post.title,
              content: post.content,
              slug: post.slug,
              status: post.status === 'published' ? 'published' : 'draft',
            })}
          />
        )}
      </Dialog>

      {/* Delete Confirmation */}
      <Dialog
        open={deleteConfirmOpen}
        onClose={() => {
          setDeleteConfirmOpen(false);
          setItemToDelete(null);
        }}
      >
        <DialogTitle>Confirm Delete</DialogTitle>
        <DialogContent>
          <Typography>
            Are you sure you want to delete this {itemToDelete?.type}? This action cannot be undone.
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteConfirmOpen(false)}>Cancel</Button>
          <Button
            onClick={handleConfirmDelete}
            color="error"
            variant="contained"
          >
            Delete
          </Button>
        </DialogActions>
      </Dialog>
    </ResponsiveContainer>
  );
};

export default BlogManager;