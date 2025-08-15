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
  Divider,
} from '@mui/material'
import {
  Search,
  VideoCall,
  AttachFile,
  Call,
  Message,
  GroupAdd,
  PersonAdd,
  MoreVert,
  People,
  Person,
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

interface Individual {
  id: string
  name: string
  status: 'online' | 'offline' | 'busy' | 'away'
  lastSeen?: string
  avatar?: string
  role?: string
  unreadMessages?: number
}

const GroupsAndPeopleTab: React.FC = () => {
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

  const [individuals] = useState<Individual[]>([
    {
      id: '1',
      name: 'Alice Johnson',
      status: 'online',
      role: 'Lead Developer',
      unreadMessages: 3,
    },
    {
      id: '2',
      name: 'Bob Smith',
      status: 'busy',
      role: 'Product Manager',
      lastSeen: '5 minutes ago',
    },
    {
      id: '3',
      name: 'Charlie Davis',
      status: 'away',
      role: 'UX Designer',
      lastSeen: '1 hour ago',
      unreadMessages: 1,
    },
    {
      id: '4',
      name: 'Diana Prince',
      status: 'offline',
      role: 'Marketing Lead',
      lastSeen: '2 hours ago',
    },
    {
      id: '5',
      name: 'Eve Wilson',
      status: 'online',
      role: 'Data Scientist',
    },
    {
      id: '6',
      name: 'Frank Miller',
      status: 'online',
      role: 'DevOps Engineer',
      unreadMessages: 2,
    },
  ])

  const filteredGroups = groups.filter(group =>
    group.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    group.description.toLowerCase().includes(searchTerm.toLowerCase())
  )

  const filteredIndividuals = individuals.filter(individual =>
    individual.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    individual.role?.toLowerCase().includes(searchTerm.toLowerCase())
  )

  const handleVideoCall = (entity: Group | Individual) => {
    console.log('Starting video call with', entity.name)
  }

  const handleVoiceCall = (entity: Group | Individual) => {
    console.log('Starting voice call with', entity.name)
  }

  const handleFileShare = (entity: Group | Individual) => {
    console.log('Sharing files with', entity.name)
  }

  const handleMessage = (entity: Group | Individual) => {
    console.log('Opening chat with', entity.name)
  }

  const getStatusColor = (status: Individual['status']) => {
    switch (status) {
      case 'online': return 'success'
      case 'busy': return 'error'
      case 'away': return 'warning'
      case 'offline': return 'default'
      default: return 'default'
    }
  }

  return (
    <Box sx={{ p: 3 }}>
      {/* Header */}
      <Paper sx={{ p: 3, mb: 3 }}>
        <Stack direction="row" alignItems="center" justifyContent="space-between" mb={2}>
          <Typography variant="h4" fontWeight={600}>
            Groups & People
          </Typography>
          <Stack direction="row" spacing={1}>
            <Button
              variant="outlined"
              startIcon={<PersonAdd />}
              color="primary"
            >
              Add Contact
            </Button>
            <Button
              variant="contained"
              startIcon={<GroupAdd />}
              color="primary"
            >
              Create Group
            </Button>
          </Stack>
        </Stack>

        {/* Search */}
        <TextField
          fullWidth
          placeholder="Search groups and people..."
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

      {/* Groups Section */}
      <Typography variant="h5" fontWeight={600} gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
        <People color="primary" />
        Groups
      </Typography>
      <Grid container spacing={3} mb={4}>
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

      <Divider sx={{ my: 4 }} />

      {/* Individuals Section */}
      <Typography variant="h5" fontWeight={600} gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
        <Person color="primary" />
        People
      </Typography>
      <Grid container spacing={3}>
        {filteredIndividuals.map((individual, index) => (
          <Grid item xs={12} sm={6} md={4} lg={3} key={individual.id}>
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
                  <Stack direction="row" alignItems="center" justifyContent="space-between" mb={2}>
                    <Badge
                      overlap="circular"
                      anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                      badgeContent={
                        <Box
                          sx={{
                            width: 12,
                            height: 12,
                            borderRadius: '50%',
                            bgcolor: `${getStatusColor(individual.status)}.main`,
                            border: '2px solid white',
                          }}
                        />
                      }
                    >
                      <Avatar sx={{ width: 56, height: 56, bgcolor: 'primary.main' }}>
                        {individual.name.charAt(0)}
                      </Avatar>
                    </Badge>
                    <IconButton size="small">
                      <MoreVert />
                    </IconButton>
                  </Stack>

                  <Typography variant="h6" fontWeight={600} gutterBottom>
                    {individual.name}
                  </Typography>
                  
                  {individual.role && (
                    <Typography variant="body2" color="text.secondary" gutterBottom>
                      {individual.role}
                    </Typography>
                  )}

                  <Chip
                    label={individual.status}
                    size="small"
                    color={getStatusColor(individual.status)}
                    variant="outlined"
                    sx={{ mt: 1 }}
                  />

                  {individual.lastSeen && (
                    <Typography variant="caption" color="text.secondary" display="block" mt={1}>
                      Last seen {individual.lastSeen}
                    </Typography>
                  )}
                </CardContent>

                <CardActions sx={{ justifyContent: 'space-around', borderTop: 1, borderColor: 'divider' }}>
                  <Tooltip title="Message">
                    <IconButton 
                      color="primary"
                      onClick={() => handleMessage(individual)}
                    >
                      <Badge badgeContent={individual.unreadMessages} color="error">
                        <Message />
                      </Badge>
                    </IconButton>
                  </Tooltip>
                  
                  <Tooltip title="Voice Call">
                    <IconButton 
                      color="primary"
                      onClick={() => handleVoiceCall(individual)}
                    >
                      <Call />
                    </IconButton>
                  </Tooltip>
                  
                  <Tooltip title="Video Call">
                    <IconButton 
                      color="primary"
                      onClick={() => handleVideoCall(individual)}
                    >
                      <VideoCall />
                    </IconButton>
                  </Tooltip>
                  
                  <Tooltip title="Share Files">
                    <IconButton 
                      color="primary"
                      onClick={() => handleFileShare(individual)}
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

      {filteredGroups.length === 0 && filteredIndividuals.length === 0 && (
        <Box sx={{ textAlign: 'center', mt: 8 }}>
          <Typography variant="h6" color="text.secondary">
            No groups or people found
          </Typography>
          <Typography variant="body2" color="text.secondary" mt={1}>
            Try adjusting your search or add new contacts
          </Typography>
        </Box>
      )}
    </Box>
  )
}

export default GroupsAndPeopleTab