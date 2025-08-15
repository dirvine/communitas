import React, { useState, useEffect } from 'react'
import {
  Box,
  Drawer,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  ListItemButton,
  Collapse,
  IconButton,
  Typography,
  Breadcrumbs,
  Link,
  Chip,
  Badge,
  Tooltip,
  Divider,
  TextField,
  InputAdornment,
  Paper,
} from '@mui/material'
import {
  Dashboard,
  Message,
  Folder,
  NetworkCheck,
  Storage,
  BugReport,
  Person,
  People,
  Call,
  Web,
  ExpandLess,
  ExpandMore,
  Search,
  Settings,
  Help,
  KeyboardArrowRight,
  Home,
  Star,
  History,
  Speed,
  Description,
  Business,
} from '@mui/icons-material'

interface NavigationItem {
  id: string
  label: string
  icon: React.ReactElement
  path: string
  badge?: number
  children?: NavigationItem[]
  shortcut?: string
}

interface EnhancedNavigationProps {
  currentTab: number
  onTabChange: (index: number) => void
  onToggleSidebar?: () => void
  sidebarOpen?: boolean
}

const navigationItems: NavigationItem[] = [
  {
    id: 'organizations',
    label: 'Organizations',
    icon: <Business />,
    path: '/organizations',
    shortcut: 'Ctrl+O',
  },
  {
    id: 'groups',
    label: 'Groups',
    icon: <People />,
    path: '/groups',
    shortcut: 'Ctrl+G',
    badge: 2,
  },
]

const recentItems = [
  { label: 'Project Proposal.md', path: '/files/documents/project-proposal', icon: <Folder /> },
  { label: 'Video call with Alice', path: '/calls/history/alice', icon: <Call /> },
  { label: 'Network diagnostics', path: '/network-diagnostics', icon: <NetworkCheck /> },
]

const shortcuts = [
  { label: 'New Document', shortcut: 'Ctrl+N', action: 'new-document' },
  { label: 'Start Call', shortcut: 'Ctrl+Shift+C', action: 'start-call' },
  { label: 'Network Diagnostics', shortcut: 'Ctrl+4', action: 'network-diagnostics' },
  { label: 'Search', shortcut: 'Ctrl+K', action: 'search' },
  { label: 'Settings', shortcut: 'Ctrl+,', action: 'settings' },
]

export default function EnhancedNavigation({
  currentTab,
  onTabChange,
  sidebarOpen = true,
}: EnhancedNavigationProps) {
  const [expandedItems, setExpandedItems] = useState<string[]>(['files'])
  const [searchQuery, setSearchQuery] = useState('')
  const [quickMenuOpen, setQuickMenuOpen] = useState(false)
  const [breadcrumbs, setBreadcrumbs] = useState<Array<{ label: string, path: string }>>([])

  // Update breadcrumbs based on current tab
  useEffect(() => {
    const currentItem = navigationItems[currentTab]
    if (currentItem) {
      const newBreadcrumbs = [
        { label: 'Home', path: '/' },
        { label: currentItem.label, path: currentItem.path },
      ]
      setBreadcrumbs(newBreadcrumbs)
    }
  }, [currentTab])

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeydown = (event: KeyboardEvent) => {
      if (event.ctrlKey && !event.shiftKey) {
        const key = event.key
        if (key >= '1' && key <= '9') {
          const index = parseInt(key) - 1
          if (index < navigationItems.length) {
            event.preventDefault()
            onTabChange(index)
          }
        } else if (key === 'k') {
          event.preventDefault()
          setQuickMenuOpen(true)
        } else if (key === ',') {
          event.preventDefault()
          console.log('Open settings')
        } else if (key === 'o') {
          event.preventDefault()
          onTabChange(1) // Organizations tab
        }
      } else if (event.ctrlKey && event.shiftKey && event.key === 'c') {
        event.preventDefault()
        console.log('Start call')
      } else if (event.key === 'Escape') {
        setQuickMenuOpen(false)
      }
    }

    window.addEventListener('keydown', handleKeydown)
    return () => window.removeEventListener('keydown', handleKeydown)
  }, [onTabChange])

  const handleExpandClick = (itemId: string) => {
    setExpandedItems(prev =>
      prev.includes(itemId)
        ? prev.filter(id => id !== itemId)
        : [...prev, itemId]
    )
  }

  const handleNavigate = (_item: NavigationItem, index: number) => {
    onTabChange(index)
  }

  const filteredItems = navigationItems.filter(item =>
    item.label.toLowerCase().includes(searchQuery.toLowerCase())
  )

  const QuickActionsMenu = () => (
    <Paper
      sx={{
        position: 'fixed',
        top: '50%',
        left: '50%',
        transform: 'translate(-50%, -50%)',
        width: 400,
        maxHeight: 500,
        overflow: 'auto',
        zIndex: 9999,
        border: '1px solid',
        borderColor: 'divider',
      }}
    >
      <Box sx={{ p: 2 }}>
        <TextField
          fullWidth
          placeholder="Search actions, files, and more..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          InputProps={{
            startAdornment: (
              <InputAdornment position="start">
                <Search />
              </InputAdornment>
            ),
          }}
          autoFocus
        />
      </Box>

      <List>
        <Typography variant="subtitle2" sx={{ px: 2, color: 'text.secondary' }}>
          Quick Actions
        </Typography>
        {shortcuts.map((shortcut) => (
          <ListItemButton 
            key={shortcut.action}
            onClick={() => {
              if (shortcut.action === 'network-diagnostics') {
                onTabChange(5) // Network Diagnostics is at index 5
                setQuickMenuOpen(false)
              } else if (shortcut.action === 'search') {
                // Search action already handled
              } else if (shortcut.action === 'settings') {
                console.log('Open settings')
                setQuickMenuOpen(false)
              } else if (shortcut.action === 'new-document') {
                console.log('New document')
                setQuickMenuOpen(false)
              } else if (shortcut.action === 'start-call') {
                console.log('Start call')
                setQuickMenuOpen(false)
              }
            }}
          >
            <ListItemText
              primary={shortcut.label}
              secondary={shortcut.shortcut}
            />
          </ListItemButton>
        ))}

        <Divider sx={{ my: 1 }} />

        <Typography variant="subtitle2" sx={{ px: 2, color: 'text.secondary' }}>
          Recent
        </Typography>
        {recentItems.map((item, index) => (
          <ListItemButton key={index}>
            <ListItemIcon>{item.icon}</ListItemIcon>
            <ListItemText primary={item.label} />
          </ListItemButton>
        ))}

        <Divider sx={{ my: 1 }} />

        <Typography variant="subtitle2" sx={{ px: 2, color: 'text.secondary' }}>
          Navigation
        </Typography>
        {filteredItems.slice(0, 5).map((item, index) => (
          <ListItemButton key={item.id} onClick={() => handleNavigate(item, index)}>
            <ListItemIcon>{item.icon}</ListItemIcon>
            <ListItemText
              primary={item.label}
              secondary={item.shortcut}
            />
            {item.badge && (
              <Badge badgeContent={item.badge} color="primary" />
            )}
          </ListItemButton>
        ))}
      </List>
    </Paper>
  )

  return (
    <>
      {/* Enhanced Sidebar */}
      <Drawer
        variant="persistent"
        open={sidebarOpen}
        sx={{
          width: 280,
          flexShrink: 0,
          '& .MuiDrawer-paper': {
            width: 280,
            boxSizing: 'border-box',
            borderRight: '1px solid',
            borderColor: 'divider',
          },
        }}
      >
        {/* Sidebar Header */}
        <Box sx={{ p: 2, display: 'flex', alignItems: 'center', gap: 2 }}>
          <Typography variant="h6" sx={{ flexGrow: 1 }}>
            Communitas
          </Typography>
          <Tooltip title="Quick Search (Ctrl+K)">
            <IconButton
              size="small"
              onClick={() => {
                setQuickMenuOpen(true)
              }}
            >
              <Search />
            </IconButton>
          </Tooltip>
        </Box>

        <Divider />

        {/* Quick Actions */}
        <Box sx={{ p: 2 }}>
          <Typography variant="subtitle2" color="text.secondary" gutterBottom>
            Quick Links
          </Typography>
          <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap' }}>
            <Chip
              icon={<Dashboard />}
              label="Overview"
              size="small"
              variant="outlined"
              clickable
              onClick={() => onTabChange(2)} // Will trigger overview modal
            />
            <Chip
              icon={<Person />}
              label="Identity"
              size="small"
              variant="outlined"
              clickable
              onClick={() => onTabChange(3)} // Will trigger identity modal
            />
            <Chip
              icon={<Star />}
              label="Favorites"
              size="small"
              variant="outlined"
              clickable
            />
            <Chip
              icon={<History />}
              label="Recent"
              size="small"
              variant="outlined"
              clickable
            />
            <Chip
              icon={<Speed />}
              label="Quick Menu"
              size="small"
              variant="outlined"
              clickable
              onClick={() => setQuickMenuOpen(true)}
            />
          </Box>
        </Box>

        <Divider />

        {/* Navigation Items */}
        <List sx={{ flexGrow: 1 }}>
          {navigationItems.map((item, index) => (
            <Box key={item.id}>
              <ListItemButton
                selected={currentTab === index}
                onClick={() => {
                  if (item.children) {
                    handleExpandClick(item.id)
                  } else {
                    handleNavigate(item, index)
                  }
                }}
              >
                <ListItemIcon>{item.icon}</ListItemIcon>
                <ListItemText 
                  primary={item.label}
                  secondary={item.shortcut}
                />
                {item.badge && (
                  <Badge badgeContent={item.badge} color="error" />
                )}
                {item.children && (
                  expandedItems.includes(item.id) ? <ExpandLess /> : <ExpandMore />
                )}
              </ListItemButton>

              {item.children && (
                <Collapse in={expandedItems.includes(item.id)} timeout="auto" unmountOnExit>
                  <List component="div" disablePadding>
                    {item.children.map((child) => (
                      <ListItemButton key={child.id} sx={{ pl: 4 }}>
                        <ListItemIcon>{child.icon}</ListItemIcon>
                        <ListItemText primary={child.label} />
                      </ListItemButton>
                    ))}
                  </List>
                </Collapse>
              )}
            </Box>
          ))}
        </List>

        <Divider />

        {/* Footer actions */}
        <Box sx={{ p: 2 }}>
          <List dense>
            <ListItemButton>
              <ListItemIcon><Settings /></ListItemIcon>
              <ListItemText primary="Settings" />
              <Typography variant="caption" color="text.secondary">
                Ctrl+,
              </Typography>
            </ListItemButton>
            <ListItemButton>
              <ListItemIcon><Help /></ListItemIcon>
              <ListItemText primary="Help" />
            </ListItemButton>
          </List>
        </Box>
      </Drawer>

      {/* Breadcrumbs */}
      <Box sx={{ 
        p: 2, 
        borderBottom: 1, 
        borderColor: 'divider',
        backgroundColor: 'background.paper',
        display: 'flex',
        alignItems: 'center',
        gap: 2,
      }}>
        <Breadcrumbs separator={<KeyboardArrowRight />}>
          {breadcrumbs.map((crumb, index) => (
            <Link
              key={index}
              underline="hover"
              color={index === breadcrumbs.length - 1 ? 'text.primary' : 'inherit'}
              sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}
            >
              {index === 0 && <Home fontSize="small" />}
              {crumb.label}
            </Link>
          ))}
        </Breadcrumbs>

        {/* Status indicators */}
        <Box sx={{ ml: 'auto', display: 'flex', gap: 1 }}>
          <Chip label="Online" color="success" size="small" />
          <Chip label="5 peers" variant="outlined" size="small" />
        </Box>
      </Box>

      {/* Quick actions menu overlay */}
      {quickMenuOpen && (
        <>
          <Box
            sx={{
              position: 'fixed',
              top: 0,
              left: 0,
              right: 0,
              bottom: 0,
              backgroundColor: 'rgba(0, 0, 0, 0.5)',
              zIndex: 9998,
            }}
            onClick={() => setQuickMenuOpen(false)}
          />
          <QuickActionsMenu />
        </>
      )}
    </>
  )
}
