import React, { useState } from 'react';
import {
  Box,
  Button,
  Card,
  CardContent,
  Chip,
  Divider,
  Fade,
  IconButton,
  Menu,
  MenuItem,
  Paper,
  Stack,
  Switch,
  Tooltip,
  Typography,
  useTheme as useMuiTheme,
} from '@mui/material';
import {
  Palette as PaletteIcon,
  DarkMode as DarkModeIcon,
  LightMode as LightModeIcon,
  AutoAwesome as AutoIcon,
  Brush as BrushIcon,
  Computer as TechIcon,
  LocalFireDepartment as WarmIcon,
  Check as CheckIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { useTheme } from './ThemeProvider';
import { ColorPreset } from '../../theme';

// Color preset configurations
const COLOR_PRESETS: { [key in ColorPreset]: { 
  label: string; 
  icon: React.ElementType; 
  description: string;
  colors: string[];
}} = {
  professional: {
    label: 'Professional',
    icon: AutoIcon,
    description: 'Clean, business-focused design',
    colors: ['#2563EB', '#7C3AED', '#059669', '#DC2626'],
  },
  creative: {
    label: 'Creative',
    icon: BrushIcon,
    description: 'Vibrant, artistic expression',
    colors: ['#F59E0B', '#EF4444', '#8B5CF6', '#10B981'],
  },
  tech: {
    label: 'Tech',
    icon: TechIcon,
    description: 'Modern, high-tech aesthetic',
    colors: ['#06B6D4', '#3B82F6', '#6366F1', '#8B5CF6'],
  },
  warm: {
    label: 'Warm',
    icon: WarmIcon,
    description: 'Cozy, welcoming atmosphere',
    colors: ['#F59E0B', '#EF4444', '#EC4899', '#F97316'],
  },
};

// Theme switcher props
interface ThemeSwitcherProps {
  compact?: boolean;
  showPresets?: boolean;
}

export const ThemeSwitcher: React.FC<ThemeSwitcherProps> = ({
  compact = false,
  showPresets = true,
}) => {
  const muiTheme = useMuiTheme();
  const { mode, colorPreset, toggleMode, setColorPreset, isDarkMode } = useTheme();
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const [presetMenuAnchor, setPresetMenuAnchor] = useState<null | HTMLElement>(null);

  const handleMenuOpen = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const handleMenuClose = () => {
    setAnchorEl(null);
  };

  const handlePresetMenuOpen = (event: React.MouseEvent<HTMLElement>) => {
    setPresetMenuAnchor(event.currentTarget);
  };

  const handlePresetMenuClose = () => {
    setPresetMenuAnchor(null);
  };

  const handlePresetChange = (preset: ColorPreset) => {
    setColorPreset(preset);
    handlePresetMenuClose();
  };

  // Compact version for toolbar/header
  if (compact) {
    return (
      <Stack direction="row" spacing={1} alignItems="center">
        <Tooltip title={`Switch to ${isDarkMode ? 'light' : 'dark'} mode`}>
          <IconButton
            onClick={toggleMode}
            size="small"
            sx={{
              color: muiTheme.palette.text.secondary,
              '&:hover': {
                backgroundColor: muiTheme.palette.action.hover,
                color: muiTheme.palette.primary.main,
              },
            }}
          >
            {isDarkMode ? <LightModeIcon /> : <DarkModeIcon />}
          </IconButton>
        </Tooltip>

        {showPresets && (
          <Tooltip title="Change color preset">
            <IconButton
              onClick={handlePresetMenuOpen}
              size="small"
              sx={{
                color: muiTheme.palette.text.secondary,
                '&:hover': {
                  backgroundColor: muiTheme.palette.action.hover,
                  color: muiTheme.palette.primary.main,
                },
              }}
            >
              <PaletteIcon />
            </IconButton>
          </Tooltip>
        )}

        <Menu
          anchorEl={presetMenuAnchor}
          open={Boolean(presetMenuAnchor)}
          onClose={handlePresetMenuClose}
          transformOrigin={{ horizontal: 'right', vertical: 'top' }}
          anchorOrigin={{ horizontal: 'right', vertical: 'bottom' }}
        >
          {Object.entries(COLOR_PRESETS).map(([key, preset]) => {
            const IconComponent = preset.icon;
            const isSelected = colorPreset === key;
            
            return (
              <MenuItem
                key={key}
                onClick={() => handlePresetChange(key as ColorPreset)}
                selected={isSelected}
              >
                <Stack direction="row" spacing={2} alignItems="center" width="100%">
                  <IconComponent fontSize="small" />
                  <Box flex={1}>
                    <Typography variant="body2">{preset.label}</Typography>
                  </Box>
                  {isSelected && <CheckIcon fontSize="small" color="primary" />}
                </Stack>
              </MenuItem>
            );
          })}
        </Menu>
      </Stack>
    );
  }

  // Full theme switcher panel
  return (
    <Card
      component={motion.div}
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
      sx={{
        maxWidth: 320,
        boxShadow: muiTheme.customShadows?.dropdown,
      }}
    >
      <CardContent>
        <Typography variant="h6" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <PaletteIcon color="primary" />
          Theme Settings
        </Typography>

        {/* Dark/Light Mode Toggle */}
        <Box mb={3}>
          <Typography variant="subtitle2" gutterBottom color="text.secondary">
            Appearance
          </Typography>
          <Paper
            variant="outlined"
            sx={{
              p: 2,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
            }}
          >
            <Stack direction="row" alignItems="center" spacing={2}>
              <Box
                sx={{
                  width: 40,
                  height: 40,
                  borderRadius: 1,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  backgroundColor: muiTheme.palette.primary.main,
                  color: muiTheme.palette.primary.contrastText,
                }}
              >
                {isDarkMode ? <DarkModeIcon /> : <LightModeIcon />}
              </Box>
              <Box>
                <Typography variant="body1" fontWeight={500}>
                  {isDarkMode ? 'Dark Mode' : 'Light Mode'}
                </Typography>
                <Typography variant="caption" color="text.secondary">
                  {isDarkMode ? 'Dark theme active' : 'Light theme active'}
                </Typography>
              </Box>
            </Stack>
            <Switch
              checked={isDarkMode}
              onChange={toggleMode}
              color="primary"
            />
          </Paper>
        </Box>

        {/* Color Presets */}
        {showPresets && (
          <>
            <Divider sx={{ mb: 2 }} />
            <Typography variant="subtitle2" gutterBottom color="text.secondary">
              Color Presets
            </Typography>
            <Stack spacing={1.5}>
              {Object.entries(COLOR_PRESETS).map(([key, preset]) => {
                const IconComponent = preset.icon;
                const isSelected = colorPreset === key;
                
                return (
                  <motion.div
                    key={key}
                    whileHover={{ scale: 1.02 }}
                    whileTap={{ scale: 0.98 }}
                  >
                    <Paper
                      variant="outlined"
                      sx={{
                        p: 2,
                        cursor: 'pointer',
                        border: isSelected 
                          ? `2px solid ${muiTheme.palette.primary.main}`
                          : `1px solid ${muiTheme.palette.divider}`,
                        backgroundColor: isSelected 
                          ? muiTheme.palette.action.selected
                          : 'transparent',
                        transition: 'all 0.2s ease',
                        '&:hover': {
                          backgroundColor: muiTheme.palette.action.hover,
                        },
                      }}
                      onClick={() => handlePresetChange(key as ColorPreset)}
                    >
                      <Stack direction="row" spacing={2} alignItems="center">
                        <Box
                          sx={{
                            width: 40,
                            height: 40,
                            borderRadius: 1,
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            background: `linear-gradient(135deg, ${preset.colors[0]} 0%, ${preset.colors[1]} 100%)`,
                            color: 'white',
                          }}
                        >
                          <IconComponent />
                        </Box>
                        <Box flex={1}>
                          <Stack direction="row" alignItems="center" spacing={1}>
                            <Typography variant="body1" fontWeight={500}>
                              {preset.label}
                            </Typography>
                            {isSelected && (
                              <CheckIcon fontSize="small" color="primary" />
                            )}
                          </Stack>
                          <Typography variant="caption" color="text.secondary">
                            {preset.description}
                          </Typography>
                          <Stack direction="row" spacing={0.5} mt={1}>
                            {preset.colors.map((color, index) => (
                              <Box
                                key={index}
                                sx={{
                                  width: 16,
                                  height: 16,
                                  borderRadius: 0.5,
                                  backgroundColor: color,
                                  border: `1px solid ${muiTheme.palette.divider}`,
                                }}
                              />
                            ))}
                          </Stack>
                        </Box>
                      </Stack>
                    </Paper>
                  </motion.div>
                );
              })}
            </Stack>
          </>
        )}

        {/* Current Theme Info */}
        <Box mt={3} pt={2} borderTop={`1px solid ${muiTheme.palette.divider}`}>
          <Stack direction="row" spacing={1} flexWrap="wrap">
            <Chip 
              label={`${mode} mode`}
              size="small" 
              color="primary" 
              variant="outlined"
            />
            <Chip 
              label={COLOR_PRESETS[colorPreset].label}
              size="small" 
              color="secondary" 
              variant="outlined"
            />
          </Stack>
        </Box>
      </CardContent>
    </Card>
  );
};

export default ThemeSwitcher;