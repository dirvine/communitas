import React, { useState } from 'react'
import {
  Box,

  Typography,
  Button,
  Avatar,
  List,
  ListItem,
  ListItemAvatar,
  ListItemText,
  ListItemSecondaryAction,
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Chip,
  Tabs,
  Tab,
  InputAdornment,
  Badge,
} from '@mui/material'
import {
  Call,
  Videocam,
  Search,
  Person,
  Group,
  History,

  Phone,
  VideoCall as VideoCallIcon,
} from '@mui/icons-material'

interface Contact {
  id: string
  name: string
  avatar: string
  address: string
  isOnline: boolean
  lastSeen?: Date
}

interface CallHistory {
  id: string
  contact: Contact
  type: 'audio' | 'video'
  direction: 'incoming' | 'outgoing' | 'missed'
  timestamp: Date
  duration?: number
}

interface CallInterfaceProps {
  onStartCall: (contactIds: string[], isVideo: boolean) => void
}

const mockContacts: Contact[] = [
  {
    id: '1',
    name: 'Alice Johnson',
    avatar: '/avatar1.jpg',
    address: 'calm-river-mountain-dawn',
    isOnline: true,
  },
  {
    id: '2',
    name: 'Bob Smith',
    avatar: '/avatar2.jpg',
    address: 'bright-star-ocean-wind',
    isOnline: true,
  },
  {
    id: '3',
    name: 'Carol Williams',
    avatar: '/avatar3.jpg',
    address: 'gentle-forest-summer-light',
    isOnline: false,
    lastSeen: new Date(Date.now() - 1800000),
  },
]

const mockCallHistory: CallHistory[] = [
  {
    id: '1',
    contact: mockContacts[0],
    type: 'video',
    direction: 'outgoing',
    timestamp: new Date(Date.now() - 3600000),
    duration: 1245,
  },
  {
    id: '2',
    contact: mockContacts[1],
    type: 'audio',
    direction: 'incoming',
    timestamp: new Date(Date.now() - 7200000),
    duration: 892,
  },
]

export default function CallInterface({ onStartCall }: CallInterfaceProps) {
  const [contacts] = useState<Contact[]>(mockContacts)
  const [callHistory] = useState<CallHistory[]>(mockCallHistory)
  const [currentTab, setCurrentTab] = useState(0)
  const [searchQuery, setSearchQuery] = useState('')
  const [groupCallOpen, setGroupCallOpen] = useState(false)
  const [selectedContacts, setSelectedContacts] = useState<string[]>([])

  const filteredContacts = contacts.filter(contact =>
    contact.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    contact.address.toLowerCase().includes(searchQuery.toLowerCase())
  )

  const formatRelativeTime = (date: Date) => {
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffMinutes = Math.floor(diffMs / (1000 * 60))
    const diffHours = Math.floor(diffMinutes / 60)
    
    if (diffMinutes < 1) return 'Just now'
    if (diffMinutes < 60) return diffMinutes + 'm ago'
    if (diffHours < 24) return diffHours + 'h ago'
    return date.toLocaleDateString()
  }


  const handleContactSelect = (contactId: string) => {
    setSelectedContacts(prev => 
      prev.includes(contactId) 
        ? prev.filter(id => id !== contactId)
        : [...prev, contactId]
    )
  }

  const handleStartGroupCall = (isVideo: boolean) => {
    if (selectedContacts.length > 0) {
      onStartCall(selectedContacts, isVideo)
      setGroupCallOpen(false)
      setSelectedContacts([])
    }
  }

  const TabPanel = ({ children, value, index }: { children: React.ReactNode, value: number, index: number }) => (
    <div hidden={value !== index}>
      {value === index && <Box sx={{ pt: 2 }}>{children}</Box>}
    </div>
  )

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <Box sx={{ p: 3, borderBottom: 1, borderColor: 'divider' }}>
        <Typography variant="h5" component="h1" sx={{ fontWeight: 600, mb: 2 }}>
          Calls
        </Typography>
        
        <TextField
          fullWidth
          placeholder="Search contacts..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          InputProps={{
            startAdornment: (
              <InputAdornment position="start">
                <Search color="action" />
              </InputAdornment>
            ),
          }}
          sx={{ mb: 2 }}
        />

        <Button
          variant="contained"
          startIcon={<Group />}
          onClick={() => setGroupCallOpen(true)}
        >
          Group Call
        </Button>
      </Box>

      {/* Tabs */}
      <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
        <Tabs value={currentTab} onChange={(_e, newValue) => setCurrentTab(newValue)}>
          <Tab label="Contacts" icon={<Person />} />
          <Tab label="History" icon={<History />} />
        </Tabs>
      </Box>

      {/* Tab content */}
      <Box sx={{ flexGrow: 1, overflow: 'auto' }}>
        <TabPanel value={currentTab} index={0}>
          <List>
            {filteredContacts.map((contact) => (
              <ListItem key={contact.id}>
                <ListItemAvatar>
                  <Badge
                    color="success"
                    variant="dot"
                    invisible={!contact.isOnline}
                  >
                    <Avatar src={contact.avatar}>
                      {contact.name[0]}
                    </Avatar>
                  </Badge>
                </ListItemAvatar>
                <ListItemText
                  primary={contact.name}
                  secondary={contact.address}
                />
                <ListItemSecondaryAction>
                  <IconButton
                    edge="end"
                    onClick={() => onStartCall([contact.id], false)}
                    disabled={!contact.isOnline}
                  >
                    <Call />
                  </IconButton>
                  <IconButton
                    edge="end"
                    onClick={() => onStartCall([contact.id], true)}
                    disabled={!contact.isOnline}
                    sx={{ ml: 1 }}
                  >
                    <Videocam />
                  </IconButton>
                </ListItemSecondaryAction>
              </ListItem>
            ))}
          </List>
        </TabPanel>

        <TabPanel value={currentTab} index={1}>
          <List>
            {callHistory.map((call) => (
              <ListItem key={call.id}>
                <ListItemAvatar>
                  <Avatar src={call.contact.avatar}>
                    {call.contact.name[0]}
                  </Avatar>
                </ListItemAvatar>
                <ListItemText
                  primary={
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                      <Typography variant="body1">
                        {call.contact.name}
                      </Typography>
                      {call.type === 'video' ? <VideoCallIcon fontSize="small" /> : <Phone fontSize="small" />}
                      <Chip
                        label={call.direction}
                        size="small"
                        color={call.direction === 'missed' ? 'error' : 'default'}
                        variant="outlined"
                      />
                    </Box>
                  }
                  secondary={formatRelativeTime(call.timestamp)}
                />
                <ListItemSecondaryAction>
                  <IconButton
                    edge="end"
                    onClick={() => onStartCall([call.contact.id], call.type === 'video')}
                    disabled={!call.contact.isOnline}
                  >
                    {call.type === 'video' ? <Videocam /> : <Call />}
                  </IconButton>
                </ListItemSecondaryAction>
              </ListItem>
            ))}
          </List>
        </TabPanel>
      </Box>

      {/* Group call dialog */}
      <Dialog
        open={groupCallOpen}
        onClose={() => setGroupCallOpen(false)}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle>Start Group Call</DialogTitle>
        <DialogContent>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
            Select contacts to invite to the group call
          </Typography>
          
          <List>
            {contacts.filter(c => c.isOnline).map((contact) => (
              <ListItem 
                key={contact.id}
                button
                onClick={() => handleContactSelect(contact.id)}
                selected={selectedContacts.includes(contact.id)}
              >
                <ListItemAvatar>
                  <Badge color="success" variant="dot">
                    <Avatar src={contact.avatar}>
                      {contact.name[0]}
                    </Avatar>
                  </Badge>
                </ListItemAvatar>
                <ListItemText
                  primary={contact.name}
                  secondary={contact.address}
                />
              </ListItem>
            ))}
          </List>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setGroupCallOpen(false)}>Cancel</Button>
          <Button
            onClick={() => handleStartGroupCall(false)}
            disabled={selectedContacts.length === 0}
            startIcon={<Call />}
          >
            Audio Call
          </Button>
          <Button
            variant="contained"
            onClick={() => handleStartGroupCall(true)}
            disabled={selectedContacts.length === 0}
            startIcon={<Videocam />}
          >
            Video Call
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  )
}
