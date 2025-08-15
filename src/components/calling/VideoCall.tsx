import { useState, useEffect, useRef } from 'react'
import {
  Box,
  Card,
  IconButton,
  Typography,
  Avatar,
  Button,
  Tooltip,
  Chip,
  Slider,
  Menu,
  MenuItem,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,




} from '@mui/material'
import {
  CallEnd,
  Mic,
  MicOff,
  Videocam,
  VideocamOff,
  ScreenShare,
  StopScreenShare,
  VolumeUp,
  Settings,
  Person,
  PersonAdd,
  Chat,
  MoreVert,
  Fullscreen,
  FullscreenExit,
} from '@mui/icons-material'

interface Participant {
  id: string
  name: string
  avatar: string
  isVideoEnabled: boolean
  isAudioEnabled: boolean
  isScreenSharing: boolean
  connectionQuality: 'excellent' | 'good' | 'poor'
}

interface VideoCallProps {
  callId: string
  participants: Participant[]
  onEndCall: () => void
  onToggleVideo: () => void
  onToggleAudio: () => void
  onToggleScreenShare: () => void
  isVideoEnabled: boolean
  isAudioEnabled: boolean
  isScreenSharing: boolean
}

const mockParticipants: Participant[] = [
  {
    id: '1',
    name: 'Alice Johnson',
    avatar: '/avatar1.jpg',
    isVideoEnabled: true,
    isAudioEnabled: true,
    isScreenSharing: false,
    connectionQuality: 'excellent',
  },
  {
    id: '2',
    name: 'Bob Smith',
    avatar: '/avatar2.jpg',
    isVideoEnabled: false,
    isAudioEnabled: true,
    isScreenSharing: false,
    connectionQuality: 'good',
  },
  {
    id: '3',
    name: 'Carol Williams',
    avatar: '/avatar3.jpg',
    isVideoEnabled: true,
    isAudioEnabled: false,
    isScreenSharing: true,
    connectionQuality: 'poor',
  },
]

export default function VideoCall({
  // callId,
  participants: propParticipants,
  onEndCall,
  onToggleVideo,
  onToggleAudio,
  onToggleScreenShare,
  isVideoEnabled,
  isAudioEnabled,
  isScreenSharing,
}: VideoCallProps) {
  const [participants] = useState<Participant[]>(propParticipants.length > 0 ? propParticipants : mockParticipants)
  const [volume, setVolume] = useState(70)
  const [isFullscreen, setIsFullscreen] = useState(false)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [menuAnchor, setMenuAnchor] = useState<HTMLElement | null>(null)
  // const [showParticipants, setShowParticipants] = useState(false)
  const [callDuration, setCallDuration] = useState(0)
  
  const localVideoRef = useRef<HTMLVideoElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const timer = setInterval(() => {
      setCallDuration(prev => prev + 1)
    }, 1000)
    
    return () => clearInterval(timer)
  }, [])

  const formatDuration = (seconds: number) => {
    const hours = Math.floor(seconds / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    const secs = seconds % 60
    
    const minStr = minutes.toString().padStart(2, '0')
    const secStr = secs.toString().padStart(2, '0')
    
    if (hours > 0) {
      return hours + ':' + minStr + ':' + secStr
    }
    return minutes + ':' + secStr
  }

  // getConnectionQualityColor function removed (unused)

  const handleFullscreen = () => {
    if (!isFullscreen && containerRef.current) {
      containerRef.current.requestFullscreen()
      setIsFullscreen(true)
    } else if (document.fullscreenElement) {
      document.exitFullscreen()
      setIsFullscreen(false)
    }
  }

  const handleVolumeChange = (_event: Event, newValue: number | number[]) => {
    setVolume(newValue as number)
  }

  const mainParticipant = participants.find(p => p.isScreenSharing) || participants[0]
  // otherParticipants variable removed (unused)
  return (
    <Box 
      ref={containerRef}
      sx={{ 
        height: '100vh',
        width: '100%',
        position: 'relative',
        background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      {/* Header */}
      <Box sx={{ 
        p: 2, 
        display: 'flex', 
        alignItems: 'center', 
        justifyContent: 'space-between',
        background: 'rgba(0,0,0,0.3)',
        backdropFilter: 'blur(10px)',
      }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
          <Typography variant="h6" color="white">
            Call with {participants.length} participants
          </Typography>
          <Chip 
            label={formatDuration(callDuration)}
            size="small"
            variant="outlined"
            sx={{ color: 'white', borderColor: 'white' }}
          />
        </Box>
        
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <Button
            variant="outlined"
            startIcon={<Person />}
            onClick={() => console.log("Show participants")}
            sx={{ color: 'white', borderColor: 'white' }}
          >
            {participants.length}
          </Button>
          
          <IconButton
            color="inherit"
            onClick={(e) => setMenuAnchor(e.currentTarget)}
            sx={{ color: 'white' }}
          >
            <MoreVert />
          </IconButton>
          
          <IconButton
            color="inherit"
            onClick={handleFullscreen}
            sx={{ color: 'white' }}
          >
            {isFullscreen ? <FullscreenExit /> : <Fullscreen />}
          </IconButton>
        </Box>
      </Box>

      {/* Main video area */}
      <Box sx={{ flexGrow: 1, position: 'relative', p: 2 }}>
        {/* Main participant video */}
        <Card sx={{ 
          width: '100%', 
          height: '100%',
          position: 'relative',
          overflow: 'hidden',
          borderRadius: 3,
        }}>
          {mainParticipant?.isVideoEnabled ? (
            <video
              style={{ 
                width: '100%', 
                height: '100%', 
                objectFit: 'cover',
                background: '#000',
              }}
              autoPlay
              muted
            />
          ) : (
            <Box sx={{ 
              width: '100%', 
              height: '100%',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              background: 'linear-gradient(45deg, #2196F3 30%, #21CBF3 90%)',
            }}>
              <Avatar 
                src={mainParticipant?.avatar}
                sx={{ width: 120, height: 120 }}
              >
                {mainParticipant?.name[0]}
              </Avatar>
            </Box>
          )}
        </Card>

        {/* Local video preview */}
        <Card sx={{ 
          position: 'absolute',
          bottom: 24,
          right: 24,
          width: 160,
          height: 120,
          overflow: 'hidden',
          border: '2px solid white',
        }}>
          <video
            ref={localVideoRef}
            style={{ 
              width: '100%', 
              height: '100%', 
              objectFit: 'cover',
              transform: 'scaleX(-1)',
            }}
            autoPlay
            muted
          />
          <Box sx={{ 
            position: 'absolute',
            bottom: 4,
            left: 4,
            right: 4,
            textAlign: 'center',
          }}>
            <Typography variant="caption" color="white" fontWeight={600}>
              You
            </Typography>
          </Box>
        </Card>
      </Box>

      {/* Call controls */}
      <Box sx={{ 
        p: 3,
        background: 'rgba(0,0,0,0.3)',
        backdropFilter: 'blur(10px)',
        display: 'flex',
        justifyContent: 'center',
        gap: 2,
      }}>
        <Tooltip title={isAudioEnabled ? 'Mute' : 'Unmute'}>
          <IconButton
            size="large"
            onClick={onToggleAudio}
            sx={{ 
              background: isAudioEnabled ? 'rgba(255,255,255,0.2)' : 'rgba(244, 67, 54, 0.8)',
              color: 'white',
              '&:hover': { 
                background: isAudioEnabled ? 'rgba(255,255,255,0.3)' : 'rgba(244, 67, 54, 0.9)'
              }
            }}
          >
            {isAudioEnabled ? <Mic /> : <MicOff />}
          </IconButton>
        </Tooltip>

        <Tooltip title={isVideoEnabled ? 'Turn off camera' : 'Turn on camera'}>
          <IconButton
            size="large"
            onClick={onToggleVideo}
            sx={{ 
              background: isVideoEnabled ? 'rgba(255,255,255,0.2)' : 'rgba(244, 67, 54, 0.8)',
              color: 'white',
              '&:hover': { 
                background: isVideoEnabled ? 'rgba(255,255,255,0.3)' : 'rgba(244, 67, 54, 0.9)'
              }
            }}
          >
            {isVideoEnabled ? <Videocam /> : <VideocamOff />}
          </IconButton>
        </Tooltip>

        <Tooltip title={isScreenSharing ? 'Stop sharing' : 'Share screen'}>
          <IconButton
            size="large"
            onClick={onToggleScreenShare}
            sx={{ 
              background: isScreenSharing ? 'rgba(76, 175, 80, 0.8)' : 'rgba(255,255,255,0.2)',
              color: 'white',
              '&:hover': { 
                background: isScreenSharing ? 'rgba(76, 175, 80, 0.9)' : 'rgba(255,255,255,0.3)'
              }
            }}
          >
            {isScreenSharing ? <StopScreenShare /> : <ScreenShare />}
          </IconButton>
        </Tooltip>

        <Tooltip title="Settings">
          <IconButton
            size="large"
            onClick={() => setSettingsOpen(true)}
            sx={{ 
              background: 'rgba(255,255,255,0.2)',
              color: 'white',
              '&:hover': { background: 'rgba(255,255,255,0.3)' }
            }}
          >
            <Settings />
          </IconButton>
        </Tooltip>

        <Tooltip title="End call">
          <IconButton
            size="large"
            onClick={onEndCall}
            sx={{ 
              background: 'rgba(244, 67, 54, 0.8)',
              color: 'white',
              '&:hover': { background: 'rgba(244, 67, 54, 0.9)' }
            }}
          >
            <CallEnd />
          </IconButton>
        </Tooltip>
      </Box>

      {/* Menu */}
      <Menu
        anchorEl={menuAnchor}
        open={Boolean(menuAnchor)}
        onClose={() => setMenuAnchor(null)}
      >
        <MenuItem onClick={() => console.log("Show participants")}>
          <Person sx={{ mr: 1 }} />
          View participants
        </MenuItem>
        <MenuItem>
          <PersonAdd sx={{ mr: 1 }} />
          Invite others
        </MenuItem>
        <MenuItem>
          <Chat sx={{ mr: 1 }} />
          Open chat
        </MenuItem>
        <MenuItem onClick={() => setSettingsOpen(true)}>
          <Settings sx={{ mr: 1 }} />
          Settings
        </MenuItem>
      </Menu>

      {/* Settings Dialog */}
      <Dialog 
        open={settingsOpen} 
        onClose={() => setSettingsOpen(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>Call Settings</DialogTitle>
        <DialogContent>
          <Box sx={{ py: 2 }}>
            <Typography gutterBottom>Volume</Typography>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
              <VolumeUp />
              <Slider
                value={volume}
                onChange={handleVolumeChange}
                valueLabelDisplay="auto"
                sx={{ flexGrow: 1 }}
              />
            </Box>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setSettingsOpen(false)}>Close</Button>
        </DialogActions>
      </Dialog>
    </Box>
  )
}
