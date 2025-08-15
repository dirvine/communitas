/**
 * Unified Home Page
 * The main landing page for the unified platform
 */

import React from 'react'
import {
  Box,
  Grid,
  Card,
  CardContent,
  Typography,
  Button,
  Avatar,
  Chip,
  Stack,
  Paper,
  IconButton,
  List,
  ListItem,
  ListItemAvatar,
  ListItemText,
  ListItemSecondaryAction,
  Divider,
  alpha,
  useTheme,
} from '@mui/material'

import {
  Message as MessageIcon,
  People as PeopleIcon,
  Folder as FolderIcon,
  Language as WebsiteIcon,
  VideoCall as VideoCallIcon,
  Add as AddIcon,
  MoreVert as MoreIcon,
  Circle as OnlineIcon,
  Schedule as ScheduleIcon,
  TrendingUp as TrendingIcon,
} from '@mui/icons-material'

import { getFourWordGradient } from '../../theme/unified'

interface UnifiedHomeProps {
  userId?: string
  fourWords?: string
  userName?: string
}

export default function UnifiedHome({
  userId = 'user_owner_123',
  fourWords = 'ocean-forest-moon-star',
  userName = 'Alice Johnson',
}: UnifiedHomeProps) {
  const theme = useTheme()

  // Mock data - will be replaced with real data
  const recentMessages = [
    { id: 1, from: 'Bob Smith', message: 'Hey, check out the new design!', time: '2 min ago', unread: true },
    { id: 2, from: 'Carol White', message: 'Meeting at 3pm?', time: '15 min ago', unread: true },
    { id: 3, from: 'Tech Team', message: 'Deployment complete ✅', time: '1 hour ago', unread: false },
  ]

  const organizations = [
    { id: 1, name: 'Acme Corp', members: 45, active: true },
    { id: 2, name: 'Tech Startup', members: 12, active: true },
    { id: 3, name: 'Open Source Project', members: 128, active: false },
  ]

  const quickActions = [
    { icon: <MessageIcon />, label: 'New Message', color: 'primary' },
    { icon: <VideoCallIcon />, label: 'Start Call', color: 'secondary' },
    { icon: <PeopleIcon />, label: 'Create Group', color: 'success' },
    { icon: <FolderIcon />, label: 'Upload Files', color: 'warning' },
  ]

  return (
    <Box sx={{ p: 3 }}>
      {/* Welcome Header */}
      <Box sx={{ mb: 4 }}>
        <Typography variant="h4" fontWeight={600} gutterBottom>
          Welcome back, {userName}
        </Typography>
        <Stack direction="row" spacing={2} alignItems="center">
          <Chip
            label={fourWords}
            size="small"
            sx={{
              background: getFourWordGradient(fourWords),
              color: 'white',
              fontWeight: 500,
            }}
          />
          <Typography variant="body2" color="text.secondary">
            Your unique four-word identity
          </Typography>
        </Stack>
      </Box>

      {/* Quick Actions */}
      <Grid container spacing={3} sx={{ mb: 4 }}>
        {quickActions.map((action, index) => (
          <Grid item xs={6} sm={3} key={index}>
            <Card
              sx={{
                cursor: 'pointer',
                transition: 'all 0.3s ease',
                '&:hover': {
                  transform: 'translateY(-4px)',
                  boxShadow: theme.shadows[8],
                },
              }}
            >
              <CardContent sx={{ textAlign: 'center' }}>
                <Avatar
                  sx={{
                    width: 56,
                    height: 56,
                    margin: '0 auto 12px',
                    bgcolor: `${action.color}.main`,
                  }}
                >
                  {action.icon}
                </Avatar>
                <Typography variant="body1" fontWeight={500}>
                  {action.label}
                </Typography>
              </CardContent>
            </Card>
          </Grid>
        ))}
      </Grid>

      <Grid container spacing={3}>
        {/* Recent Messages */}
        <Grid item xs={12} md={6}>
          <Paper
            sx={{
              p: 2,
              background: theme.palette.mode === 'dark'
                ? alpha(theme.palette.background.paper, 0.8)
                : theme.palette.background.paper,
              backdropFilter: 'blur(10px)',
            }}
          >
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
              <Typography variant="h6" fontWeight={600}>
                Recent Messages
              </Typography>
              <Button size="small" startIcon={<AddIcon />}>
                New
              </Button>
            </Box>
            <List>
              {recentMessages.map((msg, index) => (
                <React.Fragment key={msg.id}>
                  <ListItem alignItems="flex-start">
                    <ListItemAvatar>
                      <Avatar sx={{ background: getFourWordGradient(msg.from) }}>
                        {msg.from[0]}
                      </Avatar>
                    </ListItemAvatar>
                    <ListItemText
                      primary={
                        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                          <Typography variant="subtitle2" fontWeight={600}>
                            {msg.from}
                          </Typography>
                          {msg.unread && (
                            <OnlineIcon sx={{ fontSize: 8, color: 'primary.main' }} />
                          )}
                        </Box>
                      }
                      secondary={
                        <>
                          <Typography variant="body2" color="text.primary">
                            {msg.message}
                          </Typography>
                          <Typography variant="caption" color="text.secondary">
                            {msg.time}
                          </Typography>
                        </>
                      }
                    />
                    <ListItemSecondaryAction>
                      <IconButton edge="end" size="small">
                        <MoreIcon />
                      </IconButton>
                    </ListItemSecondaryAction>
                  </ListItem>
                  {index < recentMessages.length - 1 && <Divider variant="inset" component="li" />}
                </React.Fragment>
              ))}
            </List>
          </Paper>
        </Grid>

        {/* Organizations */}
        <Grid item xs={12} md={6}>
          <Paper
            sx={{
              p: 2,
              background: theme.palette.mode === 'dark'
                ? alpha(theme.palette.background.paper, 0.8)
                : theme.palette.background.paper,
              backdropFilter: 'blur(10px)',
            }}
          >
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
              <Typography variant="h6" fontWeight={600}>
                Your Organizations
              </Typography>
              <Button size="small" startIcon={<AddIcon />}>
                Join
              </Button>
            </Box>
            <Stack spacing={2}>
              {organizations.map(org => (
                <Card
                  key={org.id}
                  variant="outlined"
                  sx={{
                    p: 2,
                    cursor: 'pointer',
                    transition: 'all 0.2s ease',
                    '&:hover': {
                      bgcolor: alpha(theme.palette.primary.main, 0.05),
                      borderColor: 'primary.main',
                    },
                  }}
                >
                  <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                      <Avatar
                        sx={{
                          background: getFourWordGradient(org.name),
                          fontWeight: 600,
                        }}
                      >
                        {org.name[0]}
                      </Avatar>
                      <Box>
                        <Typography variant="subtitle1" fontWeight={600}>
                          {org.name}
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                          {org.members} members
                        </Typography>
                      </Box>
                    </Box>
                    {org.active && (
                      <Chip
                        label="Active"
                        size="small"
                        color="success"
                        variant="outlined"
                      />
                    )}
                  </Box>
                </Card>
              ))}
            </Stack>
          </Paper>
        </Grid>

        {/* Activity Stats */}
        <Grid item xs={12}>
          <Paper
            sx={{
              p: 3,
              background: `linear-gradient(135deg, ${alpha(theme.palette.primary.main, 0.1)} 0%, ${alpha(theme.palette.secondary.main, 0.1)} 100%)`,
              backdropFilter: 'blur(10px)',
            }}
          >
            <Typography variant="h6" fontWeight={600} gutterBottom>
              Today's Activity
            </Typography>
            <Grid container spacing={3}>
              <Grid item xs={6} sm={3}>
                <Box sx={{ textAlign: 'center' }}>
                  <Typography variant="h4" fontWeight={600} color="primary.main">
                    12
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Messages Sent
                  </Typography>
                </Box>
              </Grid>
              <Grid item xs={6} sm={3}>
                <Box sx={{ textAlign: 'center' }}>
                  <Typography variant="h4" fontWeight={600} color="secondary.main">
                    3
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Files Shared
                  </Typography>
                </Box>
              </Grid>
              <Grid item xs={6} sm={3}>
                <Box sx={{ textAlign: 'center' }}>
                  <Typography variant="h4" fontWeight={600} color="success.main">
                    2
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Calls Made
                  </Typography>
                </Box>
              </Grid>
              <Grid item xs={6} sm={3}>
                <Box sx={{ textAlign: 'center' }}>
                  <Typography variant="h4" fontWeight={600} color="warning.main">
                    5
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Team Members Online
                  </Typography>
                </Box>
              </Grid>
            </Grid>
          </Paper>
        </Grid>
      </Grid>
    </Box>
  )
}