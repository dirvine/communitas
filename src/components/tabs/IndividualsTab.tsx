import React, { useState } from 'react'
import {
  Box,
  Card,
  CardContent,
  CardActions,
  Typography,
  IconButton,
  Avatar,
  Grid,
  Chip,
  TextField,
  InputAdornment,
  Button,
  Stack,
  Tooltip,
  Badge,
  Paper,
} from '@mui/material'
import {
  Search,
  VideoCall,
  AttachFile,
  Call,
  Message,
  PersonAdd,
  MoreVert,
} from '@mui/icons-material'
import { motion } from 'framer-motion'

interface Individual {
  id: string
  name: string
  fourWordAddress: string
  avatar?: string
  status: 'online' | 'offline' | 'busy'
  lastSeen?: string
  bio?: string
}

const IndividualsTab: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('')
  const [individuals] = useState<Individual[]>([
    {
      id: '1',
      name: 'Alice Johnson',
      fourWordAddress: 'swift-river-calm-peak',
      status: 'online',
      bio: 'Software Engineer',
    },
    {
      id: '2',
      name: 'Bob Smith',
      fourWordAddress: 'bright-moon-soft-cloud',
      status: 'offline',
      lastSeen: '2 hours ago',
      bio: 'Product Manager',
    },
    {
      id: '3',
      name: 'Charlie Davis',
      fourWordAddress: 'gentle-wind-warm-lake',
      status: 'busy',
      bio: 'UX Designer',
    },
  ])

  const filteredIndividuals = individuals.filter(person =>
    person.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    person.fourWordAddress.includes(searchTerm.toLowerCase())
  )

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'online': return 'success'
      case 'offline': return 'default'
      case 'busy': return 'warning'
      default: return 'default'
    }
  }

  const handleVideoCall = (individual: Individual) => {
    console.log('Starting video call with', individual.name)
  }

  const handleVoiceCall = (individual: Individual) => {
    console.log('Starting voice call with', individual.name)
  }

  const handleFileShare = (individual: Individual) => {
    console.log('Sharing files with', individual.name)
  }

  const handleMessage = (individual: Individual) => {
    console.log('Opening chat with', individual.name)
  }

  return (
    <Box sx={{ p: 3 }}>
      {/* Header */}
      <Paper sx={{ p: 3, mb: 3 }}>
        <Stack direction="row" alignItems="center" justifyContent="space-between" mb={2}>
          <Typography variant="h4" fontWeight={600}>
            Individuals
          </Typography>
          <Button
            variant="contained"
            startIcon={<PersonAdd />}
            color="primary"
          >
            Add Contact
          </Button>
        </Stack>

        {/* Search */}
        <TextField
          fullWidth
          placeholder="Search individuals by name or address..."
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

      {/* Individuals Grid */}
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
                  <Stack direction="row" alignItems="flex-start" justifyContent="space-between" mb={2}>
                    <Badge
                      overlap="circular"
                      anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                      variant="dot"
                      sx={{
                        '& .MuiBadge-badge': {
                          backgroundColor: theme => 
                            individual.status === 'online' ? theme.palette.success.main :
                            individual.status === 'busy' ? theme.palette.warning.main :
                            theme.palette.grey[400],
                          color: theme => 
                            individual.status === 'online' ? theme.palette.success.main :
                            individual.status === 'busy' ? theme.palette.warning.main :
                            theme.palette.grey[400],
                          boxShadow: theme => `0 0 0 2px ${theme.palette.background.paper}`,
                          '&::after': {
                            position: 'absolute',
                            top: 0,
                            left: 0,
                            width: '100%',
                            height: '100%',
                            borderRadius: '50%',
                            animation: individual.status === 'online' ? 'ripple 1.2s infinite ease-in-out' : 'none',
                            border: '1px solid currentColor',
                            content: '""',
                          },
                        },
                        '@keyframes ripple': {
                          '0%': {
                            transform: 'scale(.8)',
                            opacity: 1,
                          },
                          '100%': {
                            transform: 'scale(2.4)',
                            opacity: 0,
                          },
                        },
                      }}
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
                  
                  <Typography variant="body2" color="text.secondary" gutterBottom>
                    {individual.bio}
                  </Typography>

                  <Chip
                    label={individual.fourWordAddress}
                    size="small"
                    variant="outlined"
                    sx={{ mt: 1 }}
                  />

                  {individual.status === 'offline' && individual.lastSeen && (
                    <Typography variant="caption" color="text.secondary" display="block" mt={1}>
                      Last seen: {individual.lastSeen}
                    </Typography>
                  )}
                </CardContent>

                <CardActions sx={{ justifyContent: 'space-around', borderTop: 1, borderColor: 'divider' }}>
                  <Tooltip title="Message">
                    <IconButton 
                      color="primary"
                      onClick={() => handleMessage(individual)}
                    >
                      <Message />
                    </IconButton>
                  </Tooltip>
                  
                  <Tooltip title="Voice Call">
                    <IconButton 
                      color="primary"
                      onClick={() => handleVoiceCall(individual)}
                      disabled={individual.status === 'offline'}
                    >
                      <Call />
                    </IconButton>
                  </Tooltip>
                  
                  <Tooltip title="Video Call">
                    <IconButton 
                      color="primary"
                      onClick={() => handleVideoCall(individual)}
                      disabled={individual.status === 'offline'}
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

      {filteredIndividuals.length === 0 && (
        <Box sx={{ textAlign: 'center', mt: 8 }}>
          <Typography variant="h6" color="text.secondary">
            No individuals found
          </Typography>
          <Typography variant="body2" color="text.secondary" mt={1}>
            Try adjusting your search or add new contacts
          </Typography>
        </Box>
      )}
    </Box>
  )
}

export default IndividualsTab