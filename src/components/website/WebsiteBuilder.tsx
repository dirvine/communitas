import React, { useState } from 'react';
import {
  Box,
  Typography,
  Button,
  Paper,
  Stack,
  Grid,
  Card,
  CardContent,
  CardActions,
  TextField,
  Switch,
  FormControlLabel,
  Tab,
  Tabs,
  Chip,
  IconButton,
  useTheme,
  alpha,
} from '@mui/material';
import {
  Web as WebIcon,
  Article as ArticleIcon,
  Settings as SettingsIcon,
  Preview as PreviewIcon,
  Publish as PublishIcon,
  Edit as EditIcon,
  Add as AddIcon,
  Delete as DeleteIcon,
  Home as HomeIcon,
  Menu as MenuIcon,
  Style as StyleIcon,
} from '@mui/icons-material';
import { motion } from 'framer-motion';
import { useResponsive, ResponsiveContainer, ResponsiveGrid } from '../responsive';
import { BlogManager } from '../blog';

// Website configuration interface
interface WebsiteConfig {
  id: string;
  name: string;
  description: string;
  domain: string;
  theme: string;
  logo?: string;
  favicon?: string;
  googleAnalytics?: string;
  socialMedia: {
    twitter?: string;
    facebook?: string;
    linkedin?: string;
    github?: string;
  };
  navigation: NavItem[];
  footerText: string;
  isPublished: boolean;
  createdAt: string;
  updatedAt: string;
}

// Navigation item interface
interface NavItem {
  id: string;
  label: string;
  url: string;
  isExternal: boolean;
  order: number;
}

// Website template interface
interface WebsiteTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  preview: string;
  features: string[];
}

export default function WebsiteBuilder() {
  const theme = useTheme();
  const { isMobile } = useResponsive();
  const [activeTab, setActiveTab] = useState<'overview' | 'content' | 'design' | 'settings'>('overview');

  // Mock website configuration
  const [websiteConfig, setWebsiteConfig] = useState<WebsiteConfig>({
    id: '1',
    name: 'My P2P Website',
    description: 'A decentralized website built on P2P technology',
    domain: 'my-site.p2p',
    theme: 'modern',
    socialMedia: {},
    navigation: [
      { id: '1', label: 'Home', url: '/', isExternal: false, order: 0 },
      { id: '2', label: 'Blog', url: '/blog', isExternal: false, order: 1 },
      { id: '3', label: 'About', url: '/about', isExternal: false, order: 2 },
      { id: '4', label: 'Contact', url: '/contact', isExternal: false, order: 3 },
    ],
    footerText: 'Built with Communitas P2P Platform',
    isPublished: false,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  });

  // Mock website templates
  const templates: WebsiteTemplate[] = [
    {
      id: 'modern',
      name: 'Modern Blog',
      description: 'Clean, responsive design perfect for blogging',
      category: 'Blog',
      preview: '/templates/modern.jpg',
      features: ['Responsive Design', 'Dark Mode', 'SEO Optimized', 'Fast Loading'],
    },
    {
      id: 'business',
      name: 'Business Pro',
      description: 'Professional template for business websites',
      category: 'Business',
      preview: '/templates/business.jpg',
      features: ['Contact Forms', 'Service Pages', 'Team Section', 'Testimonials'],
    },
    {
      id: 'portfolio',
      name: 'Creative Portfolio',
      description: 'Showcase your work with style',
      category: 'Portfolio',
      preview: '/templates/portfolio.jpg',
      features: ['Gallery', 'Project Showcase', 'Skills Section', 'Resume Download'],
    },
    {
      id: 'minimal',
      name: 'Minimal Clean',
      description: 'Simple, elegant design focusing on content',
      category: 'Minimal',
      preview: '/templates/minimal.jpg',
      features: ['Typography Focused', 'Fast Loading', 'Mobile First', 'Accessibility'],
    },
  ];

  // Mock statistics
  const stats = {
    totalVisitors: 1247,
    pageViews: 3891,
    uniqueVisitors: 892,
    averageSessionDuration: '2m 34s',
  };

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
          <Box>
            <Typography variant={isMobile ? 'h5' : 'h4'} fontWeight={700} gutterBottom>
              Website Builder
            </Typography>
            <Typography variant="body1" color="text.secondary">
              Create and manage your decentralized website
            </Typography>
          </Box>
          
          <Stack direction="row" spacing={1}>
            <Button
              variant="outlined"
              startIcon={<PreviewIcon />}
              href={`https://${websiteConfig.domain}`}
              target="_blank"
            >
              Preview
            </Button>
            <Button
              variant="contained"
              startIcon={<PublishIcon />}
              color={websiteConfig.isPublished ? 'success' : 'primary'}
            >
              {websiteConfig.isPublished ? 'Published' : 'Publish'}
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
            <Tab label="Overview" value="overview" icon={<WebIcon />} iconPosition="start" />
            <Tab label="Content" value="content" icon={<ArticleIcon />} iconPosition="start" />
            <Tab label="Design" value="design" icon={<StyleIcon />} iconPosition="start" />
            <Tab label="Settings" value="settings" icon={<SettingsIcon />} iconPosition="start" />
          </Tabs>
        </Paper>

        {/* Overview Tab */}
        {activeTab === 'overview' && (
          <>
            {/* Website Status Card */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
            >
              <Paper
                sx={{
                  p: 3,
                  mb: 3,
                  background: `linear-gradient(135deg, ${alpha(theme.palette.primary.main, 0.1)} 0%, ${alpha(theme.palette.secondary.main, 0.05)} 100%)`,
                  border: `1px solid ${alpha(theme.palette.primary.main, 0.2)}`,
                }}
              >
                <Stack direction={isMobile ? 'column' : 'row'} alignItems="center" spacing={3}>
                  <Box
                    sx={{
                      width: 80,
                      height: 80,
                      borderRadius: '50%',
                      background: theme.gradients?.primary,
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      color: 'white',
                    }}
                  >
                    <WebIcon fontSize="large" />
                  </Box>
                  
                  <Box flex={1}>
                    <Typography variant="h5" fontWeight={600} gutterBottom>
                      {websiteConfig.name}
                    </Typography>
                    <Typography variant="body1" color="text.secondary" gutterBottom>
                      {websiteConfig.description}
                    </Typography>
                    <Stack direction="row" spacing={1} flexWrap="wrap" gap={0.5}>
                      <Chip
                        label={websiteConfig.isPublished ? 'Published' : 'Draft'}
                        color={websiteConfig.isPublished ? 'success' : 'warning'}
                        size="small"
                      />
                      <Chip
                        label={`${websiteConfig.domain}`}
                        variant="outlined"
                        size="small"
                      />
                      <Chip
                        label={`Theme: ${websiteConfig.theme}`}
                        variant="outlined"
                        size="small"
                      />
                    </Stack>
                  </Box>

                  <Box textAlign="center">
                    <Typography variant="h6" fontWeight={600}>
                      P2P Ready
                    </Typography>
                    <Typography variant="caption" color="text.secondary">
                      Decentralized Hosting
                    </Typography>
                  </Box>
                </Stack>
              </Paper>
            </motion.div>

            {/* Statistics */}
            <Typography variant="h6" fontWeight={600} gutterBottom>
              Website Analytics
            </Typography>
            <ResponsiveGrid columns={{ xs: 2, sm: 4 }} spacing={2} sx={{ mb: 3 }}>
              {Object.entries(stats).map(([key, value], index) => (
                <motion.div
                  key={key}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: index * 0.1 }}
                >
                  <Card>
                    <CardContent sx={{ textAlign: 'center', py: 2 }}>
                      <Typography variant="h5" fontWeight={700} color="primary">
                        {value}
                      </Typography>
                      <Typography variant="caption" color="text.secondary" textTransform="capitalize">
                        {key.replace(/([A-Z])/g, ' $1').trim()}
                      </Typography>
                    </CardContent>
                  </Card>
                </motion.div>
              ))}
            </ResponsiveGrid>

            {/* Quick Actions */}
            <Typography variant="h6" fontWeight={600} gutterBottom>
              Quick Actions
            </Typography>
            <ResponsiveGrid columns={{ xs: 1, sm: 2, lg: 4 }} spacing={2}>
              <Card sx={{ cursor: 'pointer', '&:hover': { transform: 'translateY(-2px)' } }}>
                <CardContent sx={{ textAlign: 'center', py: 3 }}>
                  <EditIcon fontSize="large" color="primary" sx={{ mb: 1 }} />
                  <Typography variant="h6" gutterBottom>
                    Edit Content
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Manage your pages and blog posts
                  </Typography>
                </CardContent>
              </Card>

              <Card sx={{ cursor: 'pointer', '&:hover': { transform: 'translateY(-2px)' } }}>
                <CardContent sx={{ textAlign: 'center', py: 3 }}>
                  <StyleIcon fontSize="large" color="primary" sx={{ mb: 1 }} />
                  <Typography variant="h6" gutterBottom>
                    Customize Design
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Change themes and styling
                  </Typography>
                </CardContent>
              </Card>

              <Card sx={{ cursor: 'pointer', '&:hover': { transform: 'translateY(-2px)' } }}>
                <CardContent sx={{ textAlign: 'center', py: 3 }}>
                  <SettingsIcon fontSize="large" color="primary" sx={{ mb: 1 }} />
                  <Typography variant="h6" gutterBottom>
                    Site Settings
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Configure domain and SEO
                  </Typography>
                </CardContent>
              </Card>

              <Card sx={{ cursor: 'pointer', '&:hover': { transform: 'translateY(-2px)' } }}>
                <CardContent sx={{ textAlign: 'center', py: 3 }}>
                  <PublishIcon fontSize="large" color="primary" sx={{ mb: 1 }} />
                  <Typography variant="h6" gutterBottom>
                    Publish Changes
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Deploy to P2P network
                  </Typography>
                </CardContent>
              </Card>
            </ResponsiveGrid>
          </>
        )}

        {/* Content Tab */}
        {activeTab === 'content' && (
          <BlogManager />
        )}

        {/* Design Tab */}
        {activeTab === 'design' && (
          <>
            <Typography variant="h6" fontWeight={600} gutterBottom>
              Website Templates
            </Typography>
            <ResponsiveGrid columns={{ xs: 1, sm: 2, lg: 3 }} spacing={3}>
              {templates.map((template) => (
                <motion.div
                  key={template.id}
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={{ opacity: 1, scale: 1 }}
                  whileHover={{ scale: 1.02 }}
                >
                  <Card
                    sx={{
                      cursor: 'pointer',
                      border: websiteConfig.theme === template.id ? 
                        `2px solid ${theme.palette.primary.main}` : 
                        `1px solid ${theme.palette.divider}`,
                      transition: 'all 0.2s ease',
                    }}
                  >
                    {/* Template Preview */}
                    <Box
                      sx={{
                        height: 200,
                        background: `linear-gradient(135deg, ${theme.palette.primary.light} 0%, ${theme.palette.secondary.light} 100%)`,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        position: 'relative',
                        overflow: 'hidden',
                      }}
                    >
                      <WebIcon fontSize="large" sx={{ opacity: 0.3 }} />
                      {websiteConfig.theme === template.id && (
                        <Chip
                          label="Active"
                          color="primary"
                          sx={{
                            position: 'absolute',
                            top: 8,
                            right: 8,
                          }}
                        />
                      )}
                    </Box>

                    <CardContent>
                      <Typography variant="h6" gutterBottom>
                        {template.name}
                      </Typography>
                      <Typography variant="body2" color="text.secondary" gutterBottom>
                        {template.description}
                      </Typography>
                      
                      <Box sx={{ mt: 2 }}>
                        <Stack direction="row" spacing={0.5} flexWrap="wrap" gap={0.5}>
                          {template.features.slice(0, 2).map((feature) => (
                            <Chip
                              key={feature}
                              label={feature}
                              size="small"
                              variant="outlined"
                            />
                          ))}
                          {template.features.length > 2 && (
                            <Chip
                              label={`+${template.features.length - 2} more`}
                              size="small"
                              variant="outlined"
                            />
                          )}
                        </Stack>
                      </Box>
                    </CardContent>

                    <CardActions>
                      <Button
                        size="small"
                        fullWidth
                        variant={websiteConfig.theme === template.id ? 'contained' : 'outlined'}
                        onClick={() => setWebsiteConfig(prev => ({ ...prev, theme: template.id }))}
                      >
                        {websiteConfig.theme === template.id ? 'Active' : 'Use Template'}
                      </Button>
                    </CardActions>
                  </Card>
                </motion.div>
              ))}
            </ResponsiveGrid>
          </>
        )}

        {/* Settings Tab */}
        {activeTab === 'settings' && (
          <Stack spacing={3}>
            <Paper sx={{ p: 3 }}>
              <Typography variant="h6" gutterBottom>
                Basic Information
              </Typography>
              <Stack spacing={2}>
                <TextField
                  fullWidth
                  label="Website Name"
                  value={websiteConfig.name}
                  onChange={(e) => setWebsiteConfig(prev => ({ ...prev, name: e.target.value }))}
                />
                <TextField
                  fullWidth
                  multiline
                  rows={2}
                  label="Description"
                  value={websiteConfig.description}
                  onChange={(e) => setWebsiteConfig(prev => ({ ...prev, description: e.target.value }))}
                />
                <TextField
                  fullWidth
                  label="Domain"
                  value={websiteConfig.domain}
                  onChange={(e) => setWebsiteConfig(prev => ({ ...prev, domain: e.target.value }))}
                  helperText="Your P2P domain address"
                />
              </Stack>
            </Paper>

            <Paper sx={{ p: 3 }}>
              <Typography variant="h6" gutterBottom>
                Publishing
              </Typography>
              <FormControlLabel
                control={
                  <Switch
                    checked={websiteConfig.isPublished}
                    onChange={(e) => setWebsiteConfig(prev => ({ ...prev, isPublished: e.target.checked }))}
                  />
                }
                label="Publish website to P2P network"
              />
              <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                When enabled, your website will be available on the decentralized network
              </Typography>
            </Paper>

            <Paper sx={{ p: 3 }}>
              <Typography variant="h6" gutterBottom>
                Social Media
              </Typography>
              <Stack spacing={2}>
                <TextField
                  fullWidth
                  label="Twitter"
                  value={websiteConfig.socialMedia.twitter || ''}
                  onChange={(e) => setWebsiteConfig(prev => ({
                    ...prev,
                    socialMedia: { ...prev.socialMedia, twitter: e.target.value }
                  }))}
                />
                <TextField
                  fullWidth
                  label="GitHub"
                  value={websiteConfig.socialMedia.github || ''}
                  onChange={(e) => setWebsiteConfig(prev => ({
                    ...prev,
                    socialMedia: { ...prev.socialMedia, github: e.target.value }
                  }))}
                />
              </Stack>
            </Paper>
          </Stack>
        )}
      </Box>
    </ResponsiveContainer>
  );
}
