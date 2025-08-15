/**
 * Unified Context-Aware Navigation
 * Adapts based on Personal/Organization/Project context
 */

import React, { useState, useEffect } from 'react'
import {
  Box,
  Drawer,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Divider,
  Typography,
  IconButton,
  Avatar,
  Chip,
  Collapse,
  Badge,
  Tooltip,
  alpha,
  useTheme,
  useMediaQuery,
} from '@mui/material'

import {
  Home as HomeIcon,
  Message as MessageIcon,
  People as PeopleIcon,
  Phone as PhoneIcon,
  Folder as FolderIcon,
  Language as WebsiteIcon,
  Settings as SettingsIcon,
  Business as OrgIcon,
  Dashboard as DashboardIcon,
  Group as TeamIcon,
  Work as ProjectIcon,
  Tag as ChannelIcon,
  ExpandLess,
  ExpandMore,
  Add as AddIcon,
  ChevronLeft,
  ChevronRight,
  Notifications,
  Search,
  PersonOutline,
  GroupsOutlined,
  FolderOutlined,
  PublicOutlined,
  ChatBubbleOutline,
  VideoCallOutlined,
} from '@mui/icons-material'

import { designTokens, getFourWordGradient } from '../../theme/unified'

// Navigation modes
export type NavigationMode = 'personal' | 'organization' | 'project'

// Navigation context data
interface NavigationContext {
  mode: NavigationMode
  organizationId?: string
  organizationName?: string
  projectId?: string
  projectName?: string
  fourWords?: string
}

interface UnifiedNavigationProps {
  open: boolean
  onClose?: () => void
  onNavigate?: (path: string) => void
  context?: NavigationContext
  currentPath?: string
  variant?: 'permanent' | 'persistent' | 'temporary'
  width?: number
}

const DRAWER_WIDTH = 280
const DRAWER_WIDTH_COLLAPSED = 72

export default function UnifiedNavigation({
  open,
  onClose,
  onNavigate,
  context = { mode: 'personal' },
  currentPath = '/',
  variant = 'persistent',
  width = DRAWER_WIDTH,
}: UnifiedNavigationProps) {
  const theme = useTheme()
  const isMobile = useMediaQuery(theme.breakpoints.down('md'))
  const [collapsed, setCollapsed] = useState(false)
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set(['messages']))
  
  // Mock user data - will be replaced with real data
  const [user] = useState({
    name: 'Alice Johnson',
    fourWords: 'ocean-forest-moon-star',
    avatar: null,
    organizations: [
      { id: '1', name: 'Acme Corp', fourWords: 'acme-global-secure-network' },
      { id: '2', name: 'Tech Startup', fourWords: 'tech-innovate-build-future' },
    ],
    projects: [
      { id: '1', name: 'Project Alpha', fourWords: 'alpha-mission-space-explore' },
      { id: '2', name: 'Project Beta', fourWords: 'beta-test-improve-iterate' },
    ],
  })

  const toggleSection = (section: string) => {
    const newExpanded = new Set(expandedSections)
    if (newExpanded.has(section)) {
      newExpanded.delete(section)
    } else {
      newExpanded.add(section)
    }
    setExpandedSections(newExpanded)
  }

  const handleNavigate = (path: string) => {
    if (onNavigate) {
      onNavigate(path)
    }
  }

  const isActive = (path: string) => currentPath === path

  // Render different navigation based on context mode
  const renderPersonalNavigation = () => (
    <>
      <ListItem>
        <Box sx={{ width: '100%', p: 2 }}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            <Avatar
              sx={{
                width: 48,
                height: 48,
                background: getFourWordGradient(user.fourWords),
                fontWeight: 600,
              }}
            >
              {user.name.split(' ').map(n => n[0]).join('')}
            </Avatar>
            {!collapsed && (
              <Box>
                <Typography variant="subtitle1" fontWeight={600}>
                  {user.name}
                </Typography>
                <Typography variant="caption" color="text.secondary">
                  {user.fourWords}
                </Typography>
              </Box>
            )}
          </Box>
        </Box>
      </ListItem>
      
      <Divider />
      
      <List>
        <ListItemButton
          selected={isActive('/home')}
          onClick={() => handleNavigate('/home')}
        >
          <ListItemIcon>
            <HomeIcon />
          </ListItemIcon>
          {!collapsed && <ListItemText primary="Home" />}
        </ListItemButton>

        <ListItemButton onClick={() => toggleSection('messages')}>
          <ListItemIcon>
            <Badge badgeContent={3} color="error">
              <MessageIcon />
            </Badge>
          </ListItemIcon>
          {!collapsed && (
            <>
              <ListItemText primary="Messages" />
              {expandedSections.has('messages') ? <ExpandLess /> : <ExpandMore />}
            </>
          )}
        </ListItemButton>
        
        {!collapsed && (
          <Collapse in={expandedSections.has('messages')} timeout="auto" unmountOnExit>
            <List component="div" disablePadding>
              <ListItemButton
                sx={{ pl: 4 }}
                selected={isActive('/messages/direct')}
                onClick={() => handleNavigate('/messages/direct')}
              >
                <ListItemIcon>
                  <PersonOutline fontSize="small" />
                </ListItemIcon>
                <ListItemText primary="Direct Messages" />
                <Chip label="5" size="small" color="primary" />
              </ListItemButton>
              
              <ListItemButton
                sx={{ pl: 4 }}
                selected={isActive('/messages/groups')}
                onClick={() => handleNavigate('/messages/groups')}
              >
                <ListItemIcon>
                  <GroupsOutlined fontSize="small" />
                </ListItemIcon>
                <ListItemText primary="Group Chats" />
                <Chip label="2" size="small" />
              </ListItemButton>
            </List>
          </Collapse>
        )}

        <ListItemButton onClick={() => toggleSection('calls')}>
          <ListItemIcon>
            <PhoneIcon />
          </ListItemIcon>
          {!collapsed && (
            <>
              <ListItemText primary="Calls" />
              {expandedSections.has('calls') ? <ExpandLess /> : <ExpandMore />}
            </>
          )}
        </ListItemButton>
        
        {!collapsed && (
          <Collapse in={expandedSections.has('calls')} timeout="auto" unmountOnExit>
            <List component="div" disablePadding>
              <ListItemButton
                sx={{ pl: 4 }}
                selected={isActive('/calls/recent')}
                onClick={() => handleNavigate('/calls/recent')}
              >
                <ListItemIcon>
                  <PhoneIcon fontSize="small" />
                </ListItemIcon>
                <ListItemText primary="Recent" />
              </ListItemButton>
              
              <ListItemButton
                sx={{ pl: 4 }}
                selected={isActive('/calls/scheduled')}
                onClick={() => handleNavigate('/calls/scheduled')}
              >
                <ListItemIcon>
                  <VideoCallOutlined fontSize="small" />
                </ListItemIcon>
                <ListItemText primary="Scheduled" />
              </ListItemButton>
            </List>
          </Collapse>
        )}

        <ListItemButton
          selected={isActive('/files')}
          onClick={() => handleNavigate('/files')}
        >
          <ListItemIcon>
            <FolderIcon />
          </ListItemIcon>
          {!collapsed && <ListItemText primary="My Files" />}
        </ListItemButton>

        <ListItemButton
          selected={isActive('/website')}
          onClick={() => handleNavigate('/website')}
        >
          <ListItemIcon>
            <WebsiteIcon />
          </ListItemIcon>
          {!collapsed && <ListItemText primary="My Website" />}
        </ListItemButton>
      </List>

      <Divider />

      {!collapsed && (
        <>
          <List>
            <ListItem>
              <Typography variant="overline" color="text.secondary" sx={{ px: 2 }}>
                Organizations
              </Typography>
            </ListItem>
            {user.organizations.map(org => (
              <ListItemButton
                key={org.id}
                onClick={() => handleNavigate(`/org/${org.id}`)}
              >
                <ListItemIcon>
                  <Avatar
                    sx={{
                      width: 32,
                      height: 32,
                      background: getFourWordGradient(org.fourWords),
                      fontSize: '0.875rem',
                    }}
                  >
                    {org.name[0]}
                  </Avatar>
                </ListItemIcon>
                <ListItemText primary={org.name} />
              </ListItemButton>
            ))}
            <ListItemButton onClick={() => handleNavigate('/organizations/create')}>
              <ListItemIcon>
                <AddIcon />
              </ListItemIcon>
              <ListItemText primary="Create Organization" />
            </ListItemButton>
          </List>

          <Divider />
        </>
      )}
    </>
  )

  const renderOrganizationNavigation = () => (
    <>
      <ListItem>
        <Box sx={{ width: '100%', p: 2 }}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            <IconButton
              size="small"
              onClick={() => handleNavigate('/home')}
              sx={{ ml: -1 }}
            >
              <ChevronLeft />
            </IconButton>
            <Avatar
              sx={{
                width: 48,
                height: 48,
                background: getFourWordGradient(context.fourWords || ''),
                fontWeight: 600,
              }}
            >
              {context.organizationName?.[0] || 'O'}
            </Avatar>
            {!collapsed && (
              <Box sx={{ flex: 1 }}>
                <Typography variant="subtitle1" fontWeight={600}>
                  {context.organizationName}
                </Typography>
                <Typography variant="caption" color="text.secondary">
                  {context.fourWords}
                </Typography>
              </Box>
            )}
          </Box>
        </Box>
      </ListItem>

      <Divider />

      <List>
        <ListItemButton
          selected={isActive(`/org/${context.organizationId}/dashboard`)}
          onClick={() => handleNavigate(`/org/${context.organizationId}/dashboard`)}
        >
          <ListItemIcon>
            <DashboardIcon />
          </ListItemIcon>
          {!collapsed && <ListItemText primary="Dashboard" />}
        </ListItemButton>

        <ListItemButton onClick={() => toggleSection('teams')}>
          <ListItemIcon>
            <TeamIcon />
          </ListItemIcon>
          {!collapsed && (
            <>
              <ListItemText primary="Teams" />
              {expandedSections.has('teams') ? <ExpandLess /> : <ExpandMore />}
            </>
          )}
        </ListItemButton>

        {!collapsed && (
          <Collapse in={expandedSections.has('teams')} timeout="auto" unmountOnExit>
            <List component="div" disablePadding>
              <ListItemButton sx={{ pl: 4 }}>
                <ListItemText primary="Engineering" />
                <Chip label="12" size="small" />
              </ListItemButton>
              <ListItemButton sx={{ pl: 4 }}>
                <ListItemText primary="Design" />
                <Chip label="5" size="small" />
              </ListItemButton>
              <ListItemButton sx={{ pl: 4 }}>
                <ListItemText primary="Marketing" />
                <Chip label="8" size="small" />
              </ListItemButton>
            </List>
          </Collapse>
        )}

        <ListItemButton onClick={() => toggleSection('projects')}>
          <ListItemIcon>
            <ProjectIcon />
          </ListItemIcon>
          {!collapsed && (
            <>
              <ListItemText primary="Projects" />
              {expandedSections.has('projects') ? <ExpandLess /> : <ExpandMore />}
            </>
          )}
        </ListItemButton>

        <ListItemButton onClick={() => toggleSection('channels')}>
          <ListItemIcon>
            <ChannelIcon />
          </ListItemIcon>
          {!collapsed && (
            <>
              <ListItemText primary="Channels" />
              {expandedSections.has('channels') ? <ExpandLess /> : <ExpandMore />}
            </>
          )}
        </ListItemButton>

        {!collapsed && (
          <Collapse in={expandedSections.has('channels')} timeout="auto" unmountOnExit>
            <List component="div" disablePadding>
              <ListItemButton sx={{ pl: 4 }}>
                <ListItemText primary="#general" />
                <Badge variant="dot" color="error" />
              </ListItemButton>
              <ListItemButton sx={{ pl: 4 }}>
                <ListItemText primary="#announcements" />
              </ListItemButton>
              <ListItemButton sx={{ pl: 4 }}>
                <ListItemText primary="#random" />
              </ListItemButton>
            </List>
          </Collapse>
        )}

        <ListItemButton
          selected={isActive(`/org/${context.organizationId}/files`)}
          onClick={() => handleNavigate(`/org/${context.organizationId}/files`)}
        >
          <ListItemIcon>
            <FolderIcon />
          </ListItemIcon>
          {!collapsed && <ListItemText primary="Shared Files" />}
        </ListItemButton>

        <ListItemButton
          selected={isActive(`/org/${context.organizationId}/website`)}
          onClick={() => handleNavigate(`/org/${context.organizationId}/website`)}
        >
          <ListItemIcon>
            <WebsiteIcon />
          </ListItemIcon>
          {!collapsed && <ListItemText primary="Org Website" />}
        </ListItemButton>
      </List>
    </>
  )

  const renderProjectNavigation = () => (
    <>
      <ListItem>
        <Box sx={{ width: '100%', p: 2 }}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            <IconButton
              size="small"
              onClick={() => handleNavigate(`/org/${context.organizationId}`)}
              sx={{ ml: -1 }}
            >
              <ChevronLeft />
            </IconButton>
            <Avatar
              sx={{
                width: 48,
                height: 48,
                background: getFourWordGradient(context.fourWords || ''),
                fontWeight: 600,
              }}
              variant="rounded"
            >
              {context.projectName?.[0] || 'P'}
            </Avatar>
            {!collapsed && (
              <Box sx={{ flex: 1 }}>
                <Typography variant="subtitle1" fontWeight={600}>
                  {context.projectName}
                </Typography>
                <Typography variant="caption" color="text.secondary">
                  {context.fourWords}
                </Typography>
              </Box>
            )}
          </Box>
        </Box>
      </ListItem>

      <Divider />

      <List>
        {['Overview', 'Members', 'Discussion', 'Resources', 'Documents', 'Milestones', 'Project Site'].map(
          item => (
            <ListItemButton
              key={item}
              selected={isActive(`/project/${context.projectId}/${item.toLowerCase()}`)}
              onClick={() => handleNavigate(`/project/${context.projectId}/${item.toLowerCase()}`)}
            >
              <ListItemIcon>
                {item === 'Overview' && <DashboardIcon />}
                {item === 'Members' && <PeopleIcon />}
                {item === 'Discussion' && <ChatBubbleOutline />}
                {item === 'Resources' && <FolderIcon />}
                {item === 'Documents' && <FolderOutlined />}
                {item === 'Milestones' && <ProjectIcon />}
                {item === 'Project Site' && <PublicOutlined />}
              </ListItemIcon>
              {!collapsed && <ListItemText primary={item} />}
            </ListItemButton>
          )
        )}
      </List>
    </>
  )

  const drawerContent = (
    <Box
      sx={{
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        bgcolor: theme.palette.mode === 'dark' 
          ? alpha(theme.palette.background.paper, 0.8)
          : theme.palette.background.paper,
        backdropFilter: 'blur(10px)',
      }}
    >
      {/* Navigation Header */}
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          p: 2,
          borderBottom: 1,
          borderColor: 'divider',
        }}
      >
        {!collapsed && (
          <Typography variant="h6" fontWeight={600} color="primary">
            Communitas
          </Typography>
        )}
        {!isMobile && (
          <IconButton
            size="small"
            onClick={() => setCollapsed(!collapsed)}
            sx={{ ml: collapsed ? 'auto' : 0 }}
          >
            {collapsed ? <ChevronRight /> : <ChevronLeft />}
          </IconButton>
        )}
      </Box>

      {/* Navigation Content */}
      <Box sx={{ flex: 1, overflowY: 'auto', overflowX: 'hidden' }}>
        {context.mode === 'personal' && renderPersonalNavigation()}
        {context.mode === 'organization' && renderOrganizationNavigation()}
        {context.mode === 'project' && renderProjectNavigation()}
      </Box>

      {/* Navigation Footer */}
      <Divider />
      <List>
        <ListItemButton
          selected={isActive('/settings')}
          onClick={() => handleNavigate('/settings')}
        >
          <ListItemIcon>
            <SettingsIcon />
          </ListItemIcon>
          {!collapsed && <ListItemText primary="Settings" />}
        </ListItemButton>
      </List>
    </Box>
  )

  return (
    <Drawer
      variant={isMobile ? 'temporary' : variant}
      open={open}
      onClose={onClose}
      sx={{
        width: collapsed ? DRAWER_WIDTH_COLLAPSED : width,
        flexShrink: 0,
        '& .MuiDrawer-paper': {
          width: collapsed ? DRAWER_WIDTH_COLLAPSED : width,
          boxSizing: 'border-box',
          transition: theme.transitions.create('width', {
            easing: theme.transitions.easing.sharp,
            duration: theme.transitions.duration.enteringScreen,
          }),
          border: 'none',
          boxShadow: designTokens.shadows.lg,
        },
      }}
    >
      {drawerContent}
    </Drawer>
  )
}