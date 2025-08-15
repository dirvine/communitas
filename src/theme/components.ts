import { Components, Theme } from '@mui/material/styles';

export const components: Components<Omit<Theme, 'components'>> = {
  // Button component styling
  MuiButton: {
    styleOverrides: {
      root: ({ theme }) => ({
        borderRadius: theme.shape.borderRadius,
        textTransform: 'none',
        fontWeight: 500,
        boxShadow: 'none',
        '&:hover': {
          boxShadow: theme.customShadows?.card || theme.shadows[2],
        },
        '&.Mui-disabled': {
          backgroundColor: theme.palette.action.disabledBackground,
          color: theme.palette.action.disabled,
        },
      }),
      sizeSmall: ({ theme }) => ({
        padding: theme.spacing(0.5, 1.5),
        fontSize: '0.8125rem',
        minHeight: 32,
      }),
      sizeMedium: ({ theme }) => ({
        padding: theme.spacing(1, 2.5),
        fontSize: '0.875rem',
        minHeight: 40,
      }),
      sizeLarge: ({ theme }) => ({
        padding: theme.spacing(1.5, 3),
        fontSize: '0.9375rem',
        minHeight: 48,
      }),
      contained: ({ theme }) => ({
        background: theme.gradients?.primary || theme.palette.primary.main,
        '&:hover': {
          background: theme.palette.primary.dark,
          transform: 'translateY(-1px)',
        },
        '&:active': {
          transform: 'translateY(0)',
        },
      }),
      outlined: ({ theme }) => ({
        borderWidth: 1.5,
        borderColor: theme.palette.divider,
        '&:hover': {
          borderWidth: 1.5,
          backgroundColor: theme.palette.action.hover,
        },
      }),
    },
  },

  // Card component styling
  MuiCard: {
    styleOverrides: {
      root: ({ theme }) => ({
        borderRadius: Number(theme.shape.borderRadius) * 1.5,
        boxShadow: theme.customShadows?.card || theme.shadows[2],
        border: `1px solid ${theme.palette.divider}`,
        transition: theme.transitions.create(['box-shadow', 'transform'], {
          duration: theme.transitions.duration.short,
        }),
        '&:hover': {
          boxShadow: theme.customShadows?.dropdown || theme.shadows[4],
        },
      }),
    },
  },

  // Paper component styling
  MuiPaper: {
    styleOverrides: {
      root: ({ theme }) => ({
        borderRadius: theme.shape.borderRadius,
        backgroundImage: 'none',
      }),
      elevation1: ({ theme }) => ({
        boxShadow: theme.customShadows?.navigation || theme.shadows[1],
      }),
      elevation2: ({ theme }) => ({
        boxShadow: theme.customShadows?.card || theme.shadows[2],
      }),
      elevation4: ({ theme }) => ({
        boxShadow: theme.customShadows?.dropdown || theme.shadows[4],
      }),
    },
  },

  // App Bar styling
  MuiAppBar: {
    styleOverrides: {
      root: ({ theme }) => ({
        boxShadow: theme.customShadows?.navigation || theme.shadows[1],
        backdropFilter: 'blur(20px)',
        backgroundColor: theme.palette.mode === 'light' 
          ? 'rgba(255, 255, 255, 0.8)'
          : 'rgba(30, 41, 59, 0.8)',
      }),
    },
  },

  // Drawer styling
  MuiDrawer: {
    styleOverrides: {
      paper: ({ theme }) => ({
        borderRadius: 0,
        borderRight: `1px solid ${theme.palette.divider}`,
        backgroundColor: theme.palette.background.paper,
      }),
    },
  },

  // Dialog styling
  MuiDialog: {
    styleOverrides: {
      paper: ({ theme }) => ({
        borderRadius: Number(theme.shape.borderRadius) * 2,
        boxShadow: theme.customShadows?.modal || theme.shadows[8],
      }),
    },
  },

  // Menu styling
  MuiMenu: {
    styleOverrides: {
      paper: ({ theme }) => ({
        borderRadius: theme.shape.borderRadius,
        border: `1px solid ${theme.palette.divider}`,
        boxShadow: theme.customShadows?.dropdown || theme.shadows[4],
        marginTop: theme.spacing(0.5),
      }),
    },
  },

  // List Item styling
  MuiListItemButton: {
    styleOverrides: {
      root: ({ theme }) => ({
        borderRadius: theme.shape.borderRadius,
        margin: theme.spacing(0, 1),
        '&:hover': {
          backgroundColor: theme.palette.action.hover,
        },
        '&.Mui-selected': {
          backgroundColor: theme.palette.action.selected,
          '&:hover': {
            backgroundColor: theme.palette.action.selected,
          },
        },
      }),
    },
  },

  // TextField styling
  MuiTextField: {
    styleOverrides: {
      root: ({ theme }) => ({
        '& .MuiOutlinedInput-root': {
          borderRadius: theme.shape.borderRadius,
          transition: theme.transitions.create(['border-color', 'box-shadow'], {
            duration: theme.transitions.duration.short,
          }),
          '&:hover': {
            '& .MuiOutlinedInput-notchedOutline': {
              borderColor: theme.palette.primary.light,
            },
          },
          '&.Mui-focused': {
            '& .MuiOutlinedInput-notchedOutline': {
              borderColor: theme.palette.primary.main,
              borderWidth: 2,
            },
          },
        },
      }),
    },
  },

  // Chip styling
  MuiChip: {
    styleOverrides: {
      root: ({ theme }) => ({
        borderRadius: Number(theme.shape.borderRadius) * 0.75,
        fontWeight: 500,
        fontSize: '0.8125rem',
      }),
      filled: ({ theme }) => ({
        backgroundColor: theme.palette.grey[theme.palette.mode === 'light' ? 100 : 800],
        color: theme.palette.text.primary,
        '&.MuiChip-colorPrimary': {
          background: theme.gradients?.primary || theme.palette.primary.main,
          color: theme.palette.primary.contrastText,
        },
      }),
    },
  },

  // Tab styling
  MuiTab: {
    styleOverrides: {
      root: ({ theme }) => ({
        textTransform: 'none',
        fontWeight: 500,
        fontSize: '0.875rem',
        minHeight: 48,
        '&.Mui-selected': {
          color: theme.palette.primary.main,
        },
      }),
    },
  },

  // FAB styling
  MuiFab: {
    styleOverrides: {
      root: ({ theme }) => ({
        boxShadow: theme.customShadows?.fab || theme.shadows[8],
        '&:hover': {
          transform: 'scale(1.05)',
          boxShadow: theme.customShadows?.fab || theme.shadows[12],
        },
      }),
    },
  },

  // Table styling
  MuiTableContainer: {
    styleOverrides: {
      root: ({ theme }) => ({
        borderRadius: theme.shape.borderRadius,
        border: `1px solid ${theme.palette.divider}`,
      }),
    },
  },

  MuiTableHead: {
    styleOverrides: {
      root: ({ theme }) => ({
        backgroundColor: theme.palette.grey[theme.palette.mode === 'light' ? 50 : 900],
      }),
    },
  },

  MuiTableRow: {
    styleOverrides: {
      root: ({ theme }) => ({
        '&:hover': {
          backgroundColor: theme.palette.action.hover,
        },
        '&.Mui-selected': {
          backgroundColor: theme.palette.action.selected,
          '&:hover': {
            backgroundColor: theme.palette.action.selected,
          },
        },
      }),
    },
  },

  // Alert styling
  MuiAlert: {
    styleOverrides: {
      root: ({ theme }) => ({
        borderRadius: theme.shape.borderRadius,
        border: `1px solid`,
      }),
      filled: ({ theme }) => ({
        fontWeight: 500,
      }),
      outlined: ({ theme }) => ({
        backgroundColor: 'transparent',
      }),
    },
  },

  // Progress styling
  MuiLinearProgress: {
    styleOverrides: {
      root: ({ theme }) => ({
        borderRadius: theme.shape.borderRadius,
        height: 6,
      }),
      bar: ({ theme }) => ({
        borderRadius: theme.shape.borderRadius,
      }),
    },
  },

  // Tooltip styling
  MuiTooltip: {
    styleOverrides: {
      tooltip: ({ theme }) => ({
        backgroundColor: theme.palette.grey[theme.palette.mode === 'light' ? 800 : 200],
        color: theme.palette.grey[theme.palette.mode === 'light' ? 50 : 800],
        fontSize: '0.75rem',
        fontWeight: 500,
        borderRadius: Number(theme.shape.borderRadius) * 0.75,
        boxShadow: theme.customShadows?.dropdown || theme.shadows[4],
      }),
      arrow: ({ theme }) => ({
        color: theme.palette.grey[theme.palette.mode === 'light' ? 800 : 200],
      }),
    },
  },

  // Badge styling
  MuiBadge: {
    styleOverrides: {
      badge: ({ theme }) => ({
        fontWeight: 600,
        fontSize: '0.75rem',
        minWidth: 20,
        height: 20,
        borderRadius: 10,
      }),
    },
  },

  // Breadcrumbs styling
  MuiBreadcrumbs: {
    styleOverrides: {
      root: ({ theme }) => ({
        fontSize: '0.875rem',
      }),
      separator: ({ theme }) => ({
        color: theme.palette.text.secondary,
      }),
    },
  },

  // Switch styling
  MuiSwitch: {
    styleOverrides: {
      root: ({ theme }) => ({
        width: 42,
        height: 26,
        padding: 0,
        '& .MuiSwitch-switchBase': {
          padding: 0,
          margin: 2,
          transitionDuration: '300ms',
          '&.Mui-checked': {
            transform: 'translateX(16px)',
            color: '#fff',
            '& + .MuiSwitch-track': {
              backgroundColor: theme.palette.primary.main,
              opacity: 1,
              border: 0,
            },
          },
        },
        '& .MuiSwitch-thumb': {
          boxSizing: 'border-box',
          width: 22,
          height: 22,
        },
        '& .MuiSwitch-track': {
          borderRadius: 26 / 2,
          backgroundColor: theme.palette.mode === 'light' ? '#E9E9EA' : '#39393D',
          opacity: 1,
          transition: theme.transitions.create(['background-color'], {
            duration: 500,
          }),
        },
      }),
    },
  },
};