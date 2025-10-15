import React, { useState, useEffect, useMemo } from 'react'
import {
  Box,
  Typography,
  Card,
  CardContent,
  Grid,
  LinearProgress,
  Chip,
  Stack,
  Avatar,
  IconButton,
  Tooltip,
} from '@mui/material'
import {
  ChatBubbleOutline,
  Description,
  People,
  Storage,
  TrendingUp,
  TrendingDown,
  NotificationsOff,
  Settings,
} from '@mui/icons-material'
import { useEntityDirectory } from '../../contexts/EntityDirectoryContext'
import { useAuth } from '../../contexts/AuthContext'

interface ActivityMetric {
  label: string
  value: string | number
  trend?: {
    direction: 'up' | 'down' | 'neutral'
    amount: string
    period: string
  }
  icon: React.ReactNode
  color: string
}

interface ActivityItem {
  id: string
  type: 'message' | 'document' | 'member' | 'system'
  title: string
  description: string
  timestamp: string
  entity: string
  avatar?: string
  actionUrl?: string
}

export const ActivityDashboard: React.FC = () => {
  const { organizations, personalGroups, personalUsers } = useEntityDirectory()
  const { user } = useAuth()
  const [activityItems, setActivityItems] = useState<ActivityItem[]>([])
  const [isLoading, setIsLoading] = useState(true)

  // Calculate real metrics from entity data
  const metrics = useMemo<ActivityMetric[]>(() => {
    // Count unread messages across all entities
    const totalEntities = organizations.length + personalGroups.length + personalUsers.length
    const unreadMessages = Math.floor(Math.random() * 50) // TODO: Calculate from actual message data

    // Count active members (simulate activity)
    const activeMembers = organizations.reduce((acc, org) => acc + (org.members?.length || 0), 0) +
                         personalGroups.reduce((acc, group) => acc + (group.members?.length || 0), 0) +
                         personalUsers.length

    // Simulate storage usage
    const storageUsed = totalEntities * 25 + Math.floor(Math.random() * 200)

    // Simulate document updates
    const documentUpdates = Math.floor(Math.random() * 15)

    return [
      {
        label: 'Unread Messages',
        value: unreadMessages,
        trend: {
          direction: unreadMessages > 20 ? 'up' : 'down',
          amount: `${Math.abs(unreadMessages - 20)}`,
          period: 'since yesterday'
        },
        icon: <ChatBubbleOutline />,
        color: '#2EB67D'
      },
      {
        label: 'Documents Updated',
        value: documentUpdates,
        trend: {
          direction: documentUpdates > 5 ? 'up' : 'neutral',
          amount: documentUpdates > 5 ? '3' : '0',
          period: 'need review'
        },
        icon: <Description />,
        color: '#F5B759'
      },
      {
        label: 'Active Members',
        value: `${activeMembers}/${totalEntities * 2}`,
        trend: {
          direction: 'neutral',
          amount: '',
          period: 'online now'
        },
        icon: <People />,
        color: '#1E88E5'
      },
      {
        label: 'Storage Used',
        value: `${storageUsed} GB`,
        trend: {
          direction: 'neutral',
          amount: '',
          period: `of ${(totalEntities * 100)} GB total`
        },
        icon: <Storage />,
        color: '#9C27B0'
      }
    ]
  }, [organizations, personalGroups, personalUsers])

  // Generate activity feed items
  useEffect(() => {
    const generateActivityItems = (): ActivityItem[] => {
      const items: ActivityItem[] = []

      // Add recent entity activities
      organizations.slice(0, 3).forEach(org => {
        items.push({
          id: `org-${org.id}`,
          type: 'member',
          title: `${user?.name || 'You'} joined ${org.name}`,
          description: `Welcome to the ${org.name} organization`,
          timestamp: '2 hours ago',
          entity: org.name,
          avatar: user?.avatar || `https://api.dicebear.com/7.x/avataaars/svg?seed=${user?.name || 'user'}`
        })
      })

      // Add mock activity items
      items.push(
        {
          id: 'activity-1',
          type: 'document',
          title: 'Lauren McFadyen edited Q4 Roadmap.md',
          description: 'Updated project timeline and deliverables',
          timestamp: '5 minutes ago',
          entity: '#engineering',
          avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=lauren'
        },
        {
          id: 'activity-2',
          type: 'message',
          title: 'Ben Thomson sent a message',
          description: '"Hey team, just pushed the CRDT implementation..."',
          timestamp: '12 minutes ago',
          entity: '#general',
          avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=ben'
        },
        {
          id: 'activity-3',
          type: 'system',
          title: 'New member joined',
          description: 'Sarah joined the Design Team project',
          timestamp: '1 hour ago',
          entity: 'Design Team',
          avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=sarah'
        }
      )

      return items.slice(0, 6) // Limit to 6 items
    }

    // Simulate loading
    setTimeout(() => {
      setActivityItems(generateActivityItems())
      setIsLoading(false)
    }, 1000)
  }, [organizations, user])

  const getActivityIcon = (type: ActivityItem['type']) => {
    switch (type) {
      case 'message': return <ChatBubbleOutline sx={{ fontSize: 16, color: '#2EB67D' }} />
      case 'document': return <Description sx={{ fontSize: 16, color: '#F5B759' }} />
      case 'member': return <People sx={{ fontSize: 16, color: '#1E88E5' }} />
      case 'system': return <Storage sx={{ fontSize: 16, color: '#9C27B0' }} />
      default: return <ChatBubbleOutline sx={{ fontSize: 16 }} />
    }
  }

  const getTrendIcon = (direction: 'up' | 'down' | 'neutral') => {
    switch (direction) {
      case 'up': return <TrendingUp sx={{ fontSize: 14, color: '#E25555' }} />
      case 'down': return <TrendingDown sx={{ fontSize: 14, color: '#2EB67D' }} />
      default: return null
    }
  }

  if (isLoading) {
    return (
      <Box sx={{ p: 3 }}>
        <LinearProgress />
        <Typography sx={{ mt: 2, textAlign: 'center', color: 'text.secondary' }}>
          Loading activity dashboard...
        </Typography>
      </Box>
    )
  }

  return (
    <Box sx={{ p: 3 }}>
      {/* Header */}
      <Box sx={{ mb: 3, display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <Box>
          <Typography variant="h4" sx={{ fontWeight: 600, mb: 1 }}>
            Good afternoon, {user?.name?.split(' ')[0] || 'User'} 👋
          </Typography>
          <Typography variant="body1" sx={{ color: 'text.secondary' }}>
            Here's what's happening across your spaces
          </Typography>
        </Box>
        <Stack direction="row" spacing={2} alignItems="center">
          <Chip
            label="🔔 All Caught Up"
            variant="outlined"
            sx={{
              borderColor: 'success.main',
              color: 'success.main',
              '&:hover': { bgcolor: 'success.main', color: 'white' }
            }}
          />
          <Tooltip title="Dashboard Settings">
            <IconButton>
              <Settings />
            </IconButton>
          </Tooltip>
        </Stack>
      </Box>

      {/* Metrics Grid */}
      <Grid container spacing={3} sx={{ mb: 4 }}>
        {metrics.map((metric, index) => (
          <Grid item xs={12} sm={6} md={3} key={index}>
            <Card sx={{ height: '100%', borderRadius: 2 }}>
              <CardContent sx={{ p: 3 }}>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
                  <Box
                    sx={{
                      p: 1,
                      borderRadius: 2,
                      bgcolor: `${metric.color}20`,
                      mr: 2
                    }}
                  >
                    {React.cloneElement(metric.icon as React.ReactElement, {
                      sx: { color: metric.color }
                    })}
                  </Box>
                  <Typography variant="body2" sx={{ color: 'text.secondary', fontWeight: 500 }}>
                    {metric.label}
                  </Typography>
                </Box>

                <Typography variant="h4" sx={{ fontWeight: 700, mb: 1 }}>
                  {metric.value}
                </Typography>

                {metric.trend && (
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
                    {getTrendIcon(metric.trend.direction)}
                    <Typography variant="caption" sx={{ color: 'text.secondary' }}>
                      {metric.trend.amount && `${metric.trend.amount} `}
                      {metric.trend.period}
                    </Typography>
                  </Box>
                )}
              </CardContent>
            </Card>
          </Grid>
        ))}
      </Grid>

      {/* Activity Feed */}
      <Grid container spacing={3}>
        <Grid item xs={12} md={8}>
          <Card sx={{ borderRadius: 2 }}>
            <CardContent sx={{ p: 3 }}>
              <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 3 }}>
                <Typography variant="h6" sx={{ fontWeight: 600 }}>
                  🔥 Recent Activity
                </Typography>
                <Typography
                  variant="body2"
                  sx={{ color: 'primary.main', cursor: 'pointer', '&:hover': { textDecoration: 'underline' } }}
                >
                  View All →
                </Typography>
              </Box>

              <Stack spacing={2}>
                {activityItems.map((item) => (
                  <Box
                    key={item.id}
                    sx={{
                      display: 'flex',
                      gap: 3,
                      p: 2,
                      borderRadius: 2,
                      bgcolor: 'rgba(255, 255, 255, 0.02)',
                      border: '1px solid',
                      borderColor: 'divider',
                      cursor: 'pointer',
                      transition: 'all 0.2s',
                      '&:hover': {
                        bgcolor: 'rgba(255, 255, 255, 0.04)',
                        transform: 'translateY(-1px)'
                      }
                    }}
                  >
                    <Avatar
                      src={item.avatar}
                      sx={{ width: 40, height: 40 }}
                    >
                      {item.title.charAt(0).toUpperCase()}
                    </Avatar>

                    <Box sx={{ flex: 1 }}>
                      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 0.5 }}>
                        {getActivityIcon(item.type)}
                        <Typography variant="body2" sx={{ fontWeight: 600 }}>
                          {item.title}
                        </Typography>
                      </Box>

                      <Typography variant="body2" sx={{ color: 'text.secondary', mb: 1 }}>
                        {item.description}
                      </Typography>

                      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                        <Chip
                          label={item.entity}
                          size="small"
                          variant="outlined"
                          sx={{ fontSize: '0.7rem', height: 20 }}
                        />
                        <Typography variant="caption" sx={{ color: 'text.secondary' }}>
                          {item.timestamp}
                        </Typography>
                      </Box>
                    </Box>
                  </Box>
                ))}
              </Stack>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} md={4}>
          <Card sx={{ borderRadius: 2 }}>
            <CardContent sx={{ p: 3 }}>
              <Typography variant="h6" sx={{ fontWeight: 600, mb: 3 }}>
                📋 Quick Actions
              </Typography>

              <Stack spacing={2}>
                <Box
                  sx={{
                    p: 2,
                    borderRadius: 2,
                    bgcolor: 'rgba(46, 182, 125, 0.1)',
                    border: '1px solid rgba(46, 182, 125, 0.2)',
                    cursor: 'pointer',
                    transition: 'all 0.2s',
                    '&:hover': { bgcolor: 'rgba(46, 182, 125, 0.15)' }
                  }}
                >
                  <Typography variant="body2" sx={{ fontWeight: 600, color: 'success.main' }}>
                    + New Message
                  </Typography>
                  <Typography variant="caption" sx={{ color: 'text.secondary' }}>
                    Start a conversation
                  </Typography>
                </Box>

                <Box
                  sx={{
                    p: 2,
                    borderRadius: 2,
                    bgcolor: 'rgba(245, 183, 89, 0.1)',
                    border: '1px solid rgba(245, 183, 89, 0.2)',
                    cursor: 'pointer',
                    transition: 'all 0.2s',
                    '&:hover': { bgcolor: 'rgba(245, 183, 89, 0.15)' }
                  }}
                >
                  <Typography variant="body2" sx={{ fontWeight: 600, color: 'warning.main' }}>
                    📄 Create Document
                  </Typography>
                  <Typography variant="caption" sx={{ color: 'text.secondary' }}>
                    Collaborative editing
                  </Typography>
                </Box>

                <Box
                  sx={{
                    p: 2,
                    borderRadius: 2,
                    bgcolor: 'rgba(30, 136, 229, 0.1)',
                    border: '1px solid rgba(30, 136, 229, 0.2)',
                    cursor: 'pointer',
                    transition: 'all 0.2s',
                    '&:hover': { bgcolor: 'rgba(30, 136, 229, 0.15)' }
                  }}
                >
                  <Typography variant="body2" sx={{ fontWeight: 600, color: 'info.main' }}>
                    👥 Invite Member
                  </Typography>
                  <Typography variant="caption" sx={{ color: 'text.secondary' }}>
                    Grow your team
                  </Typography>
                </Box>
              </Stack>
            </CardContent>
          </Card>
        </Grid>
      </Grid>
    </Box>
  )
}
