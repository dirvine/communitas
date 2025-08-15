import { useState, useCallback } from 'react'
import CallInterface from './CallInterface'
import VideoCall from './VideoCall'

interface Participant {
  id: string
  name: string
  avatar: string
  isVideoEnabled: boolean
  isAudioEnabled: boolean
  isScreenSharing: boolean
  connectionQuality: 'excellent' | 'good' | 'poor'
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
]

export default function CallingInterface() {
  const [isInCall, setIsInCall] = useState(false)
  const [currentCall, setCurrentCall] = useState<{
    id: string
    participants: Participant[]
    isVideo: boolean
  } | null>(null)
  const [isVideoEnabled, setIsVideoEnabled] = useState(true)
  const [isAudioEnabled, setIsAudioEnabled] = useState(true)
  const [isScreenSharing, setIsScreenSharing] = useState(false)

  const handleStartCall = useCallback((contactIds: string[], isVideo: boolean) => {
    const callId = Date.now().toString()
    const participants = mockParticipants.filter(p => contactIds.includes(p.id))
    
    setCurrentCall({
      id: callId,
      participants,
      isVideo,
    })
    setIsVideoEnabled(isVideo)
    setIsAudioEnabled(true)
    setIsScreenSharing(false)
    setIsInCall(true)
    
    console.log('Starting call with participants:', participants.map(p => p.name).join(', '))
  }, [])

  const handleEndCall = useCallback(() => {
    setIsInCall(false)
    setCurrentCall(null)
    console.log('Call ended')
  }, [])

  const handleToggleVideo = useCallback(() => {
    setIsVideoEnabled(prev => !prev)
    console.log('Video toggled:', !isVideoEnabled)
  }, [isVideoEnabled])

  const handleToggleAudio = useCallback(() => {
    setIsAudioEnabled(prev => !prev)
    console.log('Audio toggled:', !isAudioEnabled)
  }, [isAudioEnabled])

  const handleToggleScreenShare = useCallback(() => {
    setIsScreenSharing(prev => !prev)
    console.log('Screen sharing toggled:', !isScreenSharing)
  }, [isScreenSharing])

  if (isInCall && currentCall) {
    return (
      <VideoCall
        callId={currentCall.id}
        participants={currentCall.participants}
        onEndCall={handleEndCall}
        onToggleVideo={handleToggleVideo}
        onToggleAudio={handleToggleAudio}
        onToggleScreenShare={handleToggleScreenShare}
        isVideoEnabled={isVideoEnabled}
        isAudioEnabled={isAudioEnabled}
        isScreenSharing={isScreenSharing}
      />
    )
  }

  return <CallInterface onStartCall={handleStartCall} />
}