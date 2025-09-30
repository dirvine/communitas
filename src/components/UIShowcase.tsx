import React, { useState } from 'react';
import {
  Box,
  Container,
  Typography,
  Grid,
  Stack,
  Divider,
  Paper,
  Switch,
  FormControlLabel,
  IconButton,
} from '@mui/material';
import { ThemeProvider } from '@mui/material/styles';
import {
  Send as SendIcon,
  Search as SearchIcon,
  Favorite as FavoriteIcon,
  Download as DownloadIcon,
  Upload as UploadIcon,
  Settings as SettingsIcon,
  Notifications as NotificationIcon,
  LightMode as LightIcon,
  DarkMode as DarkIcon,
} from '@mui/icons-material';
import { theme, darkTheme, designTokens } from '../styles/theme';
import {
  ModernButton,
  GlassCard,
  GlassCardContent,
  ModernInput,
  SearchInput,
  ModernLoader,
  SkeletonBox,
} from './ui';

export const UIShowcase: React.FC = () => {
  const [isDarkMode, setIsDarkMode] = useState(false);
  const [inputValue, setInputValue] = useState('');
  const [searchValue, setSearchValue] = useState('');
  const [loadingStates, setLoadingStates] = useState<Record<string, boolean>>({});

  const currentTheme = isDarkMode ? darkTheme : theme;

  const handleButtonClick = (id: string) => {
    setLoadingStates(prev => ({ ...prev, [id]: true }));
    setTimeout(() => {
      setLoadingStates(prev => ({ ...prev, [id]: false }));
    }, 2000);
  };

  return (
    <ThemeProvider theme={currentTheme}>
      <Box
        sx={{
          minHeight: '100vh',
          background: isDarkMode
            ? 'linear-gradient(135deg, #1a1a2e 0%, #0f0f1e 100%)'
            : 'linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%)',
          py: 4,
        }}
      >
        <Container maxWidth="lg">
          {/* Header */}
          <Stack direction="row" justifyContent="space-between" alignItems="center" mb={4}>
            <Typography variant="h3" fontWeight={700} gutterBottom>
              Ultra-Modern UI Components
            </Typography>
            <FormControlLabel
              control={
                <Switch
                  checked={isDarkMode}
                  onChange={(e) => setIsDarkMode(e.target.checked)}
                  icon={<LightIcon />}
                  checkedIcon={<DarkIcon />}
                />
              }
              label={isDarkMode ? 'Dark Mode' : 'Light Mode'}
            />
          </Stack>

          <Grid container spacing={4}>
            {/* Buttons Section */}
            <Grid item xs={12}>
              <GlassCard variant={isDarkMode ? 'dark' : 'light'} glow>
                <GlassCardContent>
                  <Box p={3}>
                    <Typography variant="h5" fontWeight={600} gutterBottom>
                      Modern Buttons
                    </Typography>
                    <Divider sx={{ my: 2 }} />

                    <Stack spacing={3}>
                      {/* Contained Buttons */}
                      <Box>
                        <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                          Contained Variants
                        </Typography>
                        <Stack direction="row" spacing={2} flexWrap="wrap">
                          <ModernButton
                            variant="contained"
                            startIcon={<SendIcon />}
                            loading={loadingStates['send']}
                            onClick={() => handleButtonClick('send')}
                          >
                            Send Message
                          </ModernButton>
                          <ModernButton
                            variant="contained"
                            gradient={false}
                            startIcon={<FavoriteIcon />}
                          >
                            Like
                          </ModernButton>
                          <ModernButton
                            variant="contained"
                            glow
                            startIcon={<DownloadIcon />}
                          >
                            Download
                          </ModernButton>
                        </Stack>
                      </Box>

                      {/* Outlined Buttons */}
                      <Box>
                        <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                          Outlined Variants
                        </Typography>
                        <Stack direction="row" spacing={2} flexWrap="wrap">
                          <ModernButton
                            variant="outlined"
                            startIcon={<UploadIcon />}
                            loading={loadingStates['upload']}
                            onClick={() => handleButtonClick('upload')}
                          >
                            Upload File
                          </ModernButton>
                          <ModernButton
                            variant="outlined"
                            startIcon={<SettingsIcon />}
                          >
                            Settings
                          </ModernButton>
                        </Stack>
                      </Box>

                      {/* Text Buttons */}
                      <Box>
                        <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                          Text Variants
                        </Typography>
                        <Stack direction="row" spacing={2} flexWrap="wrap">
                          <ModernButton variant="text">Learn More</ModernButton>
                          <ModernButton variant="text" startIcon={<NotificationIcon />}>
                            View All
                          </ModernButton>
                        </Stack>
                      </Box>
                    </Stack>
                  </Box>
                </GlassCardContent>
              </GlassCard>
            </Grid>

            {/* Cards Section */}
            <Grid item xs={12} md={6}>
              <Stack spacing={3}>
                <GlassCard variant="colored" hover glow>
                  <GlassCardContent particles>
                    <Box p={3}>
                      <Typography variant="h6" fontWeight={600} gutterBottom>
                        Colored Glass Card
                      </Typography>
                      <Typography variant="body2" color="text.secondary">
                        This card features a colored gradient background with glassmorphism effects
                        and animated particles for an ultra-modern look.
                      </Typography>
                    </Box>
                  </GlassCardContent>
                </GlassCard>

                <GlassCard variant="gradient" hover>
                  <Box p={3}>
                    <Typography variant="h6" fontWeight={600} color="white" gutterBottom>
                      Gradient Card
                    </Typography>
                    <Typography variant="body2" color="rgba(255,255,255,0.9)">
                      A vibrant gradient card with smooth hover animations and elegant design.
                    </Typography>
                  </Box>
                </GlassCard>

                <GlassCard variant={isDarkMode ? 'dark' : 'light'} hover>
                  <Box p={3}>
                    <Typography variant="h6" fontWeight={600} gutterBottom>
                      Adaptive Glass Card
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      This card adapts to the current theme with perfect glassmorphism effects.
                    </Typography>
                  </Box>
                </GlassCard>
              </Stack>
            </Grid>

            {/* Inputs and Loaders Section */}
            <Grid item xs={12} md={6}>
              <Stack spacing={3}>
                {/* Input Fields */}
                <GlassCard variant={isDarkMode ? 'dark' : 'light'}>
                  <Box p={3}>
                    <Typography variant="h6" fontWeight={600} gutterBottom>
                      Modern Inputs
                    </Typography>
                    <Stack spacing={2} mt={2}>
                      <ModernInput
                        label="Email Address"
                        placeholder="Enter your email"
                        value={inputValue}
                        onChange={(e) => setInputValue(e.target.value)}
                        icon={<SendIcon />}
                        iconPosition="end"
                      />
                      <ModernInput
                        label="Password"
                        type="password"
                        placeholder="Enter password"
                        helperText="Must be at least 8 characters"
                      />
                      <SearchInput
                        value={searchValue}
                        onChange={(e) => setSearchValue(e.target.value)}
                        placeholder="Search anything..."
                        icon={<SearchIcon />}
                      />
                    </Stack>
                  </Box>
                </GlassCard>

                {/* Loaders */}
                <GlassCard variant={isDarkMode ? 'dark' : 'light'}>
                  <Box p={3}>
                    <Typography variant="h6" fontWeight={600} gutterBottom>
                      Loading Animations
                    </Typography>
                    <Grid container spacing={2} mt={1}>
                      <Grid item xs={4}>
                        <Stack alignItems="center" spacing={1}>
                          <ModernLoader variant="pulse" size="small" />
                          <Typography variant="caption">Pulse</Typography>
                        </Stack>
                      </Grid>
                      <Grid item xs={4}>
                        <Stack alignItems="center" spacing={1}>
                          <ModernLoader variant="wave" size="small" />
                          <Typography variant="caption">Wave</Typography>
                        </Stack>
                      </Grid>
                      <Grid item xs={4}>
                        <Stack alignItems="center" spacing={1}>
                          <ModernLoader variant="orbit" size="small" />
                          <Typography variant="caption">Orbit</Typography>
                        </Stack>
                      </Grid>
                      <Grid item xs={4}>
                        <Stack alignItems="center" spacing={1}>
                          <ModernLoader variant="dots" size="small" />
                          <Typography variant="caption">Dots</Typography>
                        </Stack>
                      </Grid>
                      <Grid item xs={4}>
                        <Stack alignItems="center" spacing={1}>
                          <ModernLoader variant="gradient" size="small" />
                          <Typography variant="caption">Gradient</Typography>
                        </Stack>
                      </Grid>
                      <Grid item xs={4}>
                        <Stack alignItems="center" spacing={1}>
                          <ModernLoader variant="spinner" size="small" />
                          <Typography variant="caption">Spinner</Typography>
                        </Stack>
                      </Grid>
                    </Grid>
                  </Box>
                </GlassCard>

                {/* Skeleton Loading */}
                <GlassCard variant={isDarkMode ? 'dark' : 'light'}>
                  <Box p={3}>
                    <Typography variant="h6" fontWeight={600} gutterBottom>
                      Skeleton Loading
                    </Typography>
                    <Stack spacing={2} mt={2}>
                      <SkeletonBox height={40} />
                      <SkeletonBox height={60} />
                      <Stack direction="row" spacing={2}>
                        <SkeletonBox width={100} height={30} />
                        <SkeletonBox width={100} height={30} />
                        <SkeletonBox width={100} height={30} />
                      </Stack>
                    </Stack>
                  </Box>
                </GlassCard>
              </Stack>
            </Grid>

            {/* Design Tokens Display */}
            <Grid item xs={12}>
              <GlassCard variant={isDarkMode ? 'dark' : 'light'}>
                <Box p={3}>
                  <Typography variant="h6" fontWeight={600} gutterBottom>
                    Design System Colors
                  </Typography>
                  <Grid container spacing={2} mt={1}>
                    {Object.entries(designTokens.colors).map(([key, value]) => (
                      <Grid item xs={6} sm={4} md={2} key={key}>
                        <Stack spacing={1}>
                          <Box
                            sx={{
                              height: 60,
                              borderRadius: designTokens.borderRadius.md,
                              background: typeof value === 'object' && 'gradient' in value
                                ? value.gradient
                                : typeof value === 'object' && 'main' in value
                                ? value.main
                                : '#ccc',
                              boxShadow: designTokens.shadows.sm,
                            }}
                          />
                          <Typography variant="caption" textAlign="center">
                            {key}
                          </Typography>
                        </Stack>
                      </Grid>
                    ))}
                  </Grid>
                </Box>
              </GlassCard>
            </Grid>
          </Grid>
        </Container>
      </Box>
    </ThemeProvider>
  );
};