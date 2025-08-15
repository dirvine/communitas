import React, { useState, useEffect } from 'react'
import {
  Box,
  Drawer,
  AppBar,
  Toolbar,
  Typography,
  IconButton,
  useTheme,
  useMediaQuery,
  Fab,
  Backdrop,
  SwipeableDrawer,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Badge,
  Avatar,
  Divider,
} from '@mui/material'
import {
  Menu as MenuIcon,
  ArrowBack,
  MoreVert,
  Add,
  Group,
  Person,
  Settings,
  Search,
  // Notifications,
} from '@mui/icons-material'
import GroupChatInterface from './GroupChatInterface'
import { GroupPresencePanel, useUserPresence } from './UserPresenceIndicator'

interface MobileChatLayoutProps {
  initialGroupId?: string
  standalone?: boolean // When true, shows its own AppBar
}

interface GroupInfo {
  id: string
  name: string
  description?: string
  member_count: number
  unread_count: number
  last_message?: {
    content: string
    timestamp: string
    sender: string
  }
}

const MobileChatLayout: React.FC<MobileChatLayoutProps> = ({ initialGroupId, standalone = false }) => {
  const theme = useTheme()
  const isMobile = useMediaQuery(theme.breakpoints.down('md'))
  const isTablet = useMediaQuery(theme.breakpoints.between('md', 'lg'))
  
  // State management
  const [currentGroupId, setCurrentGroupId] = useState<string | undefined>(initialGroupId)
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [presencePanelOpen, setPresencePanelOpen] = useState(false)
  const [groups, setGroups] = useState<GroupInfo[]>([])
  
  // User presence hook
  const { users: presenceUsers } = useUserPresence(currentGroupId)

  // Mock groups data
  useEffect(() => {
    const mockGroups: GroupInfo[] = [
      {
        id: 'general',
        name: 'General',
        description: 'General discussion',
        member_count: 12,
        unread_count: 3,
        last_message: {
          content: 'Hey everyone! How is the P2P development going?',
          timestamp: new Date(Date.now() - 300000).toISOString(),
          sender: 'Alice'
        }
      },
      {
        id: 'tech-talk',
        name: 'Tech Talk',
        description: 'Technical discussions',
        member_count: 8,
        unread_count: 0,
        last_message: {
          content: 'The new DHT implementation looks great!',
          timestamp: new Date(Date.now() - 1800000).toISOString(),
          sender: 'Bob'
        }
      },
      {
        id: 'random',
        name: 'Random',
        description: 'Off-topic conversations',
        member_count: 15,
        unread_count: 1,
        last_message: {
          content: 'Anyone want to grab coffee later?',
          timestamp: new Date(Date.now() - 600000).toISOString(),
          sender: 'Charlie'
        }
      }
    ]
    setGroups(mockGroups)
  }, [])

  // Handle group selection
  const handleGroupSelect = (groupId: string) => {
    setCurrentGroupId(groupId)
    if (isMobile) {
      setDrawerOpen(false)
    }
  }

  // Format timestamp for last message
  const formatLastMessageTime = (timestamp: string) => {
    const date = new Date(timestamp)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    
    if (diff < 60000) return 'now'
    if (diff < 3600000) return Math.floor(diff / 60000) + 'm'
    if (diff < 86400000) return Math.floor(diff / 3600000) + 'h'
    return Math.floor(diff / 86400000) + 'd'
  }

  // Drawer content component
  const DrawerContent = () => (
    <Box sx={{ width: isMobile ? '80vw' : 280, height: '100%' }}>
      {/* Drawer Header */}
      <Box sx={{ 
        p: 2, 
        borderBottom: 1, 
        borderColor: 'divider',
        backgroundColor: 'primary.main',
        color: 'primary.contrastText'
      }}>
        <Typography variant="h6" noWrap>
          Communitas Groups
        </Typography>
        <Typography variant="body2" sx={{ opacity: 0.8 }}>
          {groups.length} groups available
        </Typography>
      </Box>

      {/* Groups List */}
      <List sx={{ p: 0 }}>
        {groups.map((group) => (
          <ListItem key={group.id} disablePadding>
            <ListItemButton
              onClick={() => handleGroupSelect(group.id)}
              selected={currentGroupId === group.id}
              sx={{
                py: 1.5,
                px: 2,
                '&.Mui-selected': {
                  backgroundColor: 'primary.light',
                  '&:hover': {
                    backgroundColor: 'primary.light',
                  },
                },
              }}
            >
              <ListItemIcon>
                <Badge 
                  badgeContent={group.unread_count} 
                  color="error"
                  invisible={group.unread_count === 0}
                >
                  <Avatar sx={{ width: 40, height: 40 }}>
                    <Group />
                  </Avatar>
                </Badge>
              </ListItemIcon>
              
              <ListItemText
                primary={
                  <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <Typography variant="subtitle1" fontWeight="bold">
                      {group.name}
                    </Typography>
                    {group.last_message && (
                      <Typography variant="caption" color="textSecondary">
                        {formatLastMessageTime(group.last_message.timestamp)}
                      </Typography>
                    )}
                  </Box>
                }
                secondary={
                  <Box>
                    <Typography variant="body2" color="textSecondary" noWrap>
                      {group.member_count} members
                    </Typography>
                    {group.last_message && (
                      <Typography 
                        variant="caption" 
                        color="textSecondary" 
                        noWrap
                        sx={{ display: 'block', mt: 0.5 }}
                      >
                        {group.last_message.sender}: {group.last_message.content}
                      </Typography>
                    )}
                  </Box>
                }
              />
            </ListItemButton>
          </ListItem>
        ))}
      </List>

      <Divider sx={{ my: 2 }} />

      {/* Quick Actions */}
      <List>
        <ListItem disablePadding>
          <ListItemButton>
            <ListItemIcon>
              <Add />
            </ListItemIcon>
            <ListItemText primary="Create Group" />
          </ListItemButton>
        </ListItem>
        
        <ListItem disablePadding>
          <ListItemButton>
            <ListItemIcon>
              <Search />
            </ListItemIcon>
            <ListItemText primary="Find Groups" />
          </ListItemButton>
        </ListItem>
        
        <ListItem disablePadding>
          <ListItemButton>
            <ListItemIcon>
              <Settings />
            </ListItemIcon>
            <ListItemText primary="Settings" />
          </ListItemButton>
        </ListItem>
      </List>
    </Box>
  )

  // Mobile app bar
  const MobileAppBar = () => (
    <AppBar position="static" sx={{ zIndex: theme.zIndex.drawer + 1 }}>
      <Toolbar>
        {currentGroupId ? (
          <IconButton
            edge="start"
            color="inherit"
            onClick={() => setCurrentGroupId(undefined)}
            sx={{ mr: 2 }}
          >
            <ArrowBack />
          </IconButton>
        ) : (
          <IconButton
            edge="start"
            color="inherit"
            onClick={() => setDrawerOpen(true)}
            sx={{ mr: 2 }}
          >
            <MenuIcon />
          </IconButton>
        )}
        
        <Typography variant="h6" noWrap sx={{ flexGrow: 1 }}>
          {currentGroupId 
            ? groups.find(g => g.id === currentGroupId)?.name || 'Chat'
            : 'Communitas'
          }
        </Typography>
        
        {currentGroupId && (
          <>
            <IconButton 
              color="inherit"
              onClick={() => setPresencePanelOpen(true)}
            >
              <Badge badgeContent={presenceUsers.filter(u => u.status === 'online').length} color="success">
                <Person />
              </Badge>
            </IconButton>
            
            <IconButton color="inherit">
              <MoreVert />
            </IconButton>
          </>
        )}
      </Toolbar>
    </AppBar>
  )

  // Desktop layout
  if (!isMobile && !isTablet) {
    return (
      <Box sx={{ display: 'flex', height: '100vh' }}>
        {/* Permanent drawer for desktop */}
        <Drawer
          variant="permanent"
          sx={{
            width: 280,
            flexShrink: 0,
            '& .MuiDrawer-paper': {
              width: 280,
              boxSizing: 'border-box',
            },
          }}
        >
          <DrawerContent />
        </Drawer>
        
        {/* Main content */}
        <Box sx={{ flexGrow: 1, display: 'flex', flexDirection: 'column' }}>
          {currentGroupId ? (
            <GroupChatInterface 
              groupId={currentGroupId}
              onGroupChange={setCurrentGroupId}
            />
          ) : (
            <Box sx={{ 
              display: 'flex', 
              alignItems: 'center', 
              justifyContent: 'center',
              height: '100%',
              backgroundColor: 'background.default'
            }}>
              <Typography variant="h5" color="textSecondary">
                Select a group to start chatting
              </Typography>
            </Box>
          )}
        </Box>
        
        {/* Presence panel */}
        {currentGroupId && (
          <Drawer
            anchor="right"
            open={presencePanelOpen}
            onClose={() => setPresencePanelOpen(false)}
            sx={{
              '& .MuiDrawer-paper': {
                width: 300,
                boxSizing: 'border-box',
              },
            }}
          >
            <Box sx={{ mt: 8 }}>
              <GroupPresencePanel 
                users={presenceUsers}
                groupId={currentGroupId}
              />
            </Box>
          </Drawer>
        )}
      </Box>
    )
  }

  // Mobile/Tablet layout
  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Mobile App Bar - Only show in standalone mode */}
      {standalone && <MobileAppBar />}
      
      {/* Simplified header when not standalone */}
      {!standalone && (
        <Box sx={{ 
          p: 2, 
          borderBottom: 1, 
          borderColor: 'divider',
          display: 'flex',
          alignItems: 'center',
          gap: 2
        }}>
          <IconButton onClick={() => setDrawerOpen(true)}>
            <MenuIcon />
          </IconButton>
          <Typography variant="h6">
            {currentGroupId ? groups.find(g => g.id === currentGroupId)?.name || 'Messages' : 'Messages'}
          </Typography>
          {currentGroupId && (
            <IconButton 
              sx={{ ml: 'auto' }}
              onClick={() => setPresencePanelOpen(true)}
            >
              <Badge badgeContent={presenceUsers.filter(u => u.status === 'online').length} color="success">
                <Group />
              </Badge>
            </IconButton>
          )}
        </Box>
      )}
      
      {/* Main Content */}
      <Box sx={{ flexGrow: 1, overflow: 'hidden' }}>
        {currentGroupId ? (
          <GroupChatInterface 
            groupId={currentGroupId}
            onGroupChange={setCurrentGroupId}
          />
        ) : (
          <Box sx={{ 
            display: 'flex', 
            flexDirection: 'column',
            alignItems: 'center', 
            justifyContent: 'center',
            height: '100%',
            p: 3,
            textAlign: 'center'
          }}>
            <Group sx={{ fontSize: 64, color: 'primary.main', mb: 2 }} />
            <Typography variant="h5" gutterBottom>
              Welcome to Communitas
            </Typography>
            <Typography variant="body1" color="textSecondary" sx={{ mb: 3 }}>
              Select a group from the menu to start chatting with your peers in the P2P network.
            </Typography>
            <Fab 
              variant="extended" 
              color="primary"
              onClick={() => setDrawerOpen(true)}
            >
              <MenuIcon sx={{ mr: 1 }} />
              Browse Groups
            </Fab>
          </Box>
        )}
      </Box>

      {/* Mobile Navigation Drawer */}
      <SwipeableDrawer
        anchor="left"
        open={drawerOpen}
        onClose={() => setDrawerOpen(false)}
        onOpen={() => setDrawerOpen(true)}
        sx={{
          '& .MuiDrawer-paper': {
            boxSizing: 'border-box',
          },
        }}
      >
        <DrawerContent />
      </SwipeableDrawer>

      {/* Mobile Presence Panel */}
      <SwipeableDrawer
        anchor="bottom"
        open={presencePanelOpen}
        onClose={() => setPresencePanelOpen(false)}
        onOpen={() => setPresencePanelOpen(true)}
        sx={{
          '& .MuiDrawer-paper': {
            maxHeight: '70vh',
            borderTopLeftRadius: 16,
            borderTopRightRadius: 16,
          },
        }}
      >
        <Box sx={{ p: 2 }}>
          <Box sx={{ 
            width: 40, 
            height: 4, 
            backgroundColor: 'grey.300', 
            borderRadius: 2, 
            mx: 'auto', 
            mb: 2 
          }} />
          <GroupPresencePanel 
            users={presenceUsers}
            groupId={currentGroupId || ''}
          />
        </Box>
      </SwipeableDrawer>
      
      {/* Backdrop for mobile drawer */}
      <Backdrop
        open={drawerOpen}
        onClick={() => setDrawerOpen(false)}
        sx={{ zIndex: theme.zIndex.drawer - 1 }}
      />
    </Box>
  )
}

export default MobileChatLayout
