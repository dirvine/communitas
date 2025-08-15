import React, { useState } from 'react'
import {
  Box,
  Card,
  CardContent,
  CardActions,
  Typography,
  IconButton,
  Avatar,
  AvatarGroup,
  Grid,
  Chip,
  TextField,
  InputAdornment,
  Button,
  Stack,
  Tooltip,
  Paper,
  Badge,
} from '@mui/material'
import {
  Search,
  VideoCall,
  AttachFile,
  Call,
  Message,
  GroupAdd,
  MoreVert,
  People,
} from '@mui/icons-material'
import { motion } from 'framer-motion'

interface Group {
  id: string
  name: string
  description: string
  memberCount: number
  members: { id: string; name: string; avatar?: string }[]
  lastActivity: string
  unreadMessages?: number
  type: 'public' | 'private'
}

const GroupsTab: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('')
  const [groups] = useState<Group[]>([
    {
      id: '1',
      name: 'Engineering Team',
      description: 'Platform development and architecture discussions',
      memberCount: 12,
      members: [
        { id: '1', name: 'Alice Johnson' },
        { id: '2', name: 'Bob Smith' },
        { id: '3', name: 'Charlie Davis' },
      ],
      lastActivity: '10 minutes ago',
      unreadMessages: 5,
      type: 'private',
    },
    {
      id: '2',
      name: 'Design Squad',
      description: 'UI/UX design and creative discussions',
      memberCount: 8,
      members: [
        { id: '4', name: 'Diana Prince' },
        { id: '5', name: 'Eve Wilson' },
        { id: '6', name: 'Frank Miller' },
      ],
      lastActivity: '1 hour ago',
      type: 'public',
    },
    {
      id: '3',
      name: 'Marketing Team',
      description: 'Marketing strategies and campaign planning',
      memberCount: 6,
      members: [
        { id: '7', name: 'Grace Lee' },
        { id: '8', name: 'Henry Ford' },
      ],
      lastActivity: '3 hours ago',
      unreadMessages: 2,
      type: 'private',
    },
  ])

  const filteredGroups = groups.filter(group =>
    group.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    group.description.toLowerCase().includes(searchTerm.toLowerCase())
  )

  const handleVideoCall = (group: Group) => {
    console.log('Starting group video call with', group.name)
  }

  const handleVoiceCall = (group: Group) => {
    console.log('Starting group voice call with', group.name)
  }

  const handleFileShare = (group: Group) => {
    console.log('Sharing files with group', group.name)
  }

  const handleMessage = (group: Group) => {
    console.log('Opening group chat with', group.name)
  }

  return (
    <Box sx={{ p: 3 }}>
      {/* Header */}
      <Paper sx={{ p: 3, mb: 3 }}>
        <Stack direction="row" alignItems="center" justifyContent="space-between" mb={2}>
          <Typography variant="h4" fontWeight={600}>
            Groups
          </Typography>
          <Button
            variant="contained"
            startIcon={<GroupAdd />}
            color="primary"
          >
            Create Group
          </Button>
        </Stack>

        {/* Search */}
        <TextField
          fullWidth
          placeholder="Search groups by name or description..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          InputProps={{
            startAdornment: (
              <InputAdornment position="start">
                <Search />
              </InputAdornment>
            ),
          }}
        />
      </Paper>

      {/* Groups Grid */}
      <Grid container spacing={3}>
        {filteredGroups.map((group, index) => (
          <Grid item xs={12} sm={6} md={4} key={group.id}>
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: index * 0.1 }}
            >
              <Card 
                sx={{ 
                  height: '100%',
                  display: 'flex',
                  flexDirection: 'column',
                  '&:hover': {
                    boxShadow: 6,
                    transform: 'translateY(-4px)',
                  },
                  transition: 'all 0.3s ease',
                }}
              >
                <CardContent sx={{ flexGrow: 1 }}>
                  <Stack direction="row" alignItems="flex-start" justifyContent="space-between" mb={2}>
                    <Badge 
                      badgeContent={group.unreadMessages} 
                      color="primary"
                      max={99}
                    >
                      <Avatar sx={{ width: 56, height: 56, bgcolor: 'secondary.main' }}>
                        <People />
                      </Avatar>
                    </Badge>
                    <Stack direction="row" spacing={0.5}>
                      <Chip
                        label={group.type}
                        size="small"
                        color={group.type === 'private' ? 'default' : 'primary'}
                        variant="outlined"
                      />
                      <IconButton size="small">
                        <MoreVert />
                      </IconButton>
                    </Stack>
                  </Stack>

                  <Typography variant="h6" fontWeight={600} gutterBottom>
                    {group.name}
                  </Typography>
                  
                  <Typography 
                    variant="body2" 
                    color="text.secondary" 
                    gutterBottom
                    sx={{
                      overflow: 'hidden',
                      textOverflow: 'ellipsis',
                      display: '-webkit-box',
                      WebkitLineClamp: 2,
                      WebkitBoxOrient: 'vertical',
                    }}
                  >
                    {group.description}
                  </Typography>

                  <Stack direction="row" alignItems="center" spacing={1} mt={2}>
                    <AvatarGroup max={3} sx={{ '& .MuiAvatar-root': { width: 24, height: 24, fontSize: 12 } }}>
                      {group.members.map(member => (
                        <Avatar key={member.id} sx={{ bgcolor: 'primary.main' }}>
                          {member.name.charAt(0)}
                        </Avatar>
                      ))}
                    </AvatarGroup>
                    <Typography variant="caption" color="text.secondary">
                      {group.memberCount} members
                    </Typography>
                  </Stack>

                  <Typography variant="caption" color="text.secondary" display="block" mt={1}>
                    Active {group.lastActivity}
                  </Typography>
                </CardContent>

                <CardActions sx={{ justifyContent: 'space-around', borderTop: 1, borderColor: 'divider' }}>
                  <Tooltip title="Group Chat">
                    <IconButton 
                      color="primary"
                      onClick={() => handleMessage(group)}
                    >
                      <Badge badgeContent={group.unreadMessages} color="error">
                        <Message />
                      </Badge>
                    </IconButton>
                  </Tooltip>
                  
                  <Tooltip title="Group Voice Call">
                    <IconButton 
                      color="primary"
                      onClick={() => handleVoiceCall(group)}
                    >
                      <Call />
                    </IconButton>
                  </Tooltip>
                  
                  <Tooltip title="Group Video Call">
                    <IconButton 
                      color="primary"
                      onClick={() => handleVideoCall(group)}
                    >
                      <VideoCall />
                    </IconButton>
                  </Tooltip>
                  
                  <Tooltip title="Share Files">
                    <IconButton 
                      color="primary"
                      onClick={() => handleFileShare(group)}
                    >
                      <AttachFile />
                    </IconButton>
                  </Tooltip>
                </CardActions>
              </Card>
            </motion.div>
          </Grid>
        ))}
      </Grid>

      {filteredGroups.length === 0 && (
        <Box sx={{ textAlign: 'center', mt: 8 }}>
          <Typography variant="h6" color="text.secondary">
            No groups found
          </Typography>
          <Typography variant="body2" color="text.secondary" mt={1}>
            Try adjusting your search or create a new group
          </Typography>
        </Box>
      )}
    </Box>
  )
}

export default GroupsTab