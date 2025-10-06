import React, { useMemo, useState, useEffect, useCallback, useRef } from 'react'
import {
  Box,
  Typography,
  IconButton,
  Avatar,
  Stack,
  Chip,
  Tooltip,
  alpha,
  useMediaQuery,
  Theme,
  Button,
  LinearProgress,
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  InputBase,
  Divider,
  Badge,
} from '@mui/material'
import { getMessageSyncService } from '../../services/MessageSyncService.browser'
import type { CRDTMessage } from '../../types/crdt'
import {
  ChatBubbleOutline,
  PeopleOutline,
  Apartment,
  ExploreOutlined,
  SettingsOutlined,
  StorageOutlined,
  PhoneOutlined,
  VideocamOutlined,
  InfoOutlined,
  Search as SearchIcon,
  PushPinOutlined,
  NotificationsOffOutlined,
  Check,
  CheckCircle,
  Call,
  GridView,
  Add,
  MoreHoriz,
  LibraryBooksOutlined,
  FolderOutlined,
  EmojiEmotionsOutlined,
  AttachFileOutlined,
  ReplyOutlined,
  LockOutlined,
  ForumOutlined,
  CloudOutlined,
  LinkOutlined,
  StorageRounded,
  SendRounded,
  Close as CloseIcon,
  StarBorder,
  ForwardOutlined,
  ContentCopyOutlined,
  ReportProblemOutlined,
  DeleteOutline,
  CheckBoxOutlineBlankOutlined,
  PersonOutline,
  HomeOutlined,
  LanguageOutlined,
  ArchiveOutlined,
  MoreVert,
  KeyboardArrowDown,
  EditOutlined,
} from '@mui/icons-material'
import { styled } from '@mui/material/styles'
import {
  AddContactDialog,
  EditContactDialog,
  DeleteContactDialog,
  type Contact,
} from './ContactManagementDialogs'
import { MessageReactionPicker, MessageReactionsDisplay } from './MessageReactionPicker'
import { Star, StarBorder as StarOutlineIcon } from '@mui/icons-material'
import { ConnectionStatus } from '../ConnectionStatus'

const TOKENS = {
  bgBase: '#101518',
  bgRaised: '#161C20',
  surfaceActive: '#1E252B',
  borderSubtle: '#1F262C',
  textPrimary: '#F4F6F8',
  textSecondary: '#9AA2AB',
  accent: '#2EB67D',
  accentMuted: 'rgba(46, 182, 125, 0.15)',
  danger: '#E25555',
  warning: '#F5B759',
}

type ConversationType = 'person' | 'group' | 'project' | 'channel' | 'organisation' | 'storage'

type Conversation = {
  id: string
  name: string
  type: ConversationType
  snippet: string
  time: string
  unread?: number
  status?: 'sent' | 'delivered' | 'read'
  pinned?: boolean
  muted?: boolean
  starred?: boolean // For favourites
  lastMessageTime?: number // Unix timestamp for MRU sorting
  org?: string
  membersOnline?: number
  online?: boolean
  hasWebsite?: boolean
  description?: string
  fourWords?: string // Four-word network address for contacts
  projectMeta?: {
    status: 'Active' | 'Planning' | 'Blocked'
    completion: number
    owner: string
  }
  channelMeta?: {
    topic: string
    members: number
    integrations?: string[]
  }
}

type Message = {
  id: string
  author: string
  text: string
  time: string
  self: boolean
  status?: 'sent' | 'delivered' | 'read'
  system?: boolean
  threadCount?: number
  latestReplyBy?: string
  reactions?: { emoji: string; count: number; userReacted?: boolean }[]
}

type DrawerTab = 'Overview' | 'Members' | 'Files' | 'Tasks' | 'Timeline' | 'Storage'

type ChannelMode = 'chat' | 'threads' | 'files' | 'integrations'
type ProjectMode = 'chat' | 'board' | 'tasks' | 'timeline'

const filters: { key: string; label: string }[] = [
  { key: 'all', label: 'All' },
  { key: 'unread', label: 'Unread' },
  { key: 'favourites', label: 'Favourites' },
  { key: 'groups', label: 'Groups' },
  { key: 'projects', label: 'Projects' },
  { key: 'people', label: 'People' },
]

const channelModes: { key: ChannelMode; label: string }[] = [
  { key: 'chat', label: 'Chat' },
  { key: 'threads', label: 'Threads' },
  { key: 'files', label: 'Files' },
  { key: 'integrations', label: 'Integrations' },
]

const projectModes: { key: ProjectMode; label: string }[] = [
  { key: 'chat', label: 'Chat' },
  { key: 'board', label: 'Board' },
  { key: 'tasks', label: 'Tasks' },
  { key: 'timeline', label: 'Timeline' },
]

const SystemRailButton = styled(IconButton)(({ theme }) => ({
  width: '100%',
  color: TOKENS.textSecondary,
  borderRadius: 12,
  padding: 12,
  marginBottom: 6,
  transition: 'background 120ms ease',
  '&:hover': {
    background: alpha(TOKENS.accent, 0.12),
    color: TOKENS.textPrimary,
  },
}))

const FilterChip = styled(Chip)(() => ({
  borderRadius: 12,
  height: 28,
  fontWeight: 500,
  background: 'transparent',
  color: TOKENS.textSecondary,
  '&.MuiChip-filled': {
    background: TOKENS.accentMuted,
    color: TOKENS.accent,
  },
}))

const DrawerTabChip = styled(Chip)(() => ({
  borderRadius: 10,
  height: 26,
  fontSize: 12,
  background: 'transparent',
  color: TOKENS.textSecondary,
  '&.MuiChip-filled': {
    background: alpha(TOKENS.accent, 0.16),
    color: TOKENS.accent,
  },
}))

const ViewChip = styled(Chip)(() => ({
  borderRadius: 999,
  height: 28,
  fontSize: 12,
  background: 'transparent',
  color: TOKENS.textSecondary,
  '&.MuiChip-filled': {
    background: alpha(TOKENS.accent, 0.18),
    color: TOKENS.accent,
  },
}))

const ConversationListItem = styled(Box)<{ selected?: boolean }>(({ selected }) => ({
  display: 'flex',
  alignItems: 'center',
  padding: '12px 16px',
  gap: 12,
  cursor: 'pointer',
  borderRadius: 16,
  transition: 'background 120ms ease, transform 120ms ease',
  background: selected ? TOKENS.surfaceActive : 'transparent',
  '&:hover': {
    background: selected ? TOKENS.surfaceActive : 'rgba(255,255,255,0.04)',
    transform: 'translateY(-1px)',
  },
}))

const Bubble = styled(Box)<{ self?: boolean }>(({ self }) => ({
  maxWidth: '75%',
  alignSelf: self ? 'flex-end' : 'flex-start',
  background: self ? alpha(TOKENS.accent, 0.16) : TOKENS.surfaceActive,
  color: TOKENS.textPrimary,
  padding: '12px 14px',
  borderRadius: 16,
  borderTopRightRadius: self ? 4 : 16,
  borderTopLeftRadius: self ? 16 : 4,
}))

const avatarShapeStyles: Record<ConversationType, React.CSSProperties> = {
  person: { borderRadius: '50%' },
  group: { borderRadius: 16 },
  project: { borderRadius: '20% 20% 12% 12%' },
  channel: { borderRadius: 14, border: `1px solid ${alpha('#FFFFFF', 0.16)}` },
  storage: { borderRadius: 12, border: `1px solid ${alpha('#FFFFFF', 0.12)}` },
  organisation: { borderRadius: 12, border: `1px solid ${alpha(TOKENS.accent, 0.4)}` },
}

const presenceBadge = (conversation: Conversation) => {
  switch (conversation.type) {
    case 'storage':
      return { icon: <StorageOutlined sx={{ fontSize: 15, color: TOKENS.accent }} />, bg: alpha(TOKENS.accent, 0.18) }
    case 'organisation':
      return { icon: <Apartment sx={{ fontSize: 15, color: TOKENS.accent }} />, bg: alpha(TOKENS.accent, 0.18) }
    case 'project':
      return { icon: <FolderOutlined sx={{ fontSize: 15, color: TOKENS.accent }} />, bg: alpha(TOKENS.accent, 0.12) }
    case 'channel':
      return { icon: <GridView sx={{ fontSize: 15, color: TOKENS.textSecondary }} />, bg: alpha('#FFFFFF', 0.08) }
    case 'group':
      return { icon: <PeopleOutline sx={{ fontSize: 15, color: TOKENS.textSecondary }} />, bg: alpha('#FFFFFF', 0.08) }
    default: {
      const online = conversation.online ?? (conversation.membersOnline ?? 0) > 0
      return { icon: null, bg: online ? TOKENS.accent : alpha('#FFFFFF', 0.25) }
    }
  }
}

// Mock data from spec
const channelThreadPreviews = [
  {
    id: 'thread-1',
    title: 'Storage alert follow-up',
    summary: 'Thread with 8 replies · Latest from Ben 4m ago',
  },
  {
    id: 'thread-2',
    title: 'Marketing microsite check-in',
    summary: 'Thread with 3 replies · Latest from Lauren 12m ago',
  },
]

const channelFiles = ['Roadmap.pdf', 'launch-brief.md', 'storage-playbook.xlsx']
const channelIntegrations = ['Storage Bot', 'Calendar', 'Incident Pager']

const projectBoard = [
  { title: 'Backlog', items: ['Document onboarding flow', 'Design storage health widget'] },
  { title: 'In Progress', items: ['Implement PQC sync patch', 'Bootstrap GA checklist'] },
  { title: 'Review', items: ['Marketing microsite QA'] },
]

const projectTasks = [
  { title: 'Prepare launch comms', assignee: 'Lauren', due: 'Tomorrow', status: 'In progress' },
  { title: 'Validate storage failover', assignee: 'Ben', due: 'Friday', status: 'Blocked' },
]

const projectTimeline = [
  'Sep 29 · Storage GA milestone marked complete by Lauren',
  'Sep 28 · PQC sync patch deployed to FRA1',
  'Sep 27 · Marketing microsite preview published',
]

const organisationOverviewData = {
  members: [
    { name: 'David Allan', role: 'Owner', status: 'Online' },
    { name: 'Lauren McFadyen', role: 'Product', status: 'Online' },
    { name: 'Ben Thomson', role: 'Engineering', status: 'Away' },
    { name: 'Storage Bot', role: 'Automation', status: 'Reports only' },
  ],
  projects: ['Project Lumos', 'Bootstrap Hardening', 'Marketing Microsite'],
  channels: ['#general', '#storage', '#marketing', '#engineering'],
}

const storageContainers = [
  {
    id: 'vault',
    name: 'Org Vault',
    description: 'End-to-end encrypted vault replicated across bootstrap nodes.',
    usagePercent: 42,
    usageText: '420 GB / 1 TB',
    status: 'Healthy',
    icon: <StorageRounded sx={{ color: TOKENS.accent }} />,
    actions: ['Open', 'Manage'],
  },
  {
    id: 'web-disk',
    name: 'Web Storage (Virtual Disk)',
    description: 'S3-compatible virtual disk for org web apps.',
    usagePercent: 68,
    usageText: '340 GB / 500 GB',
    status: 'Syncing',
    icon: <CloudOutlined sx={{ color: '#5CC9FF' }} />,
    actions: ['Open', 'Mount'],
  },
  {
    id: 'personal',
    name: 'Personal Web Space',
    description: 'Private workspace linked to ocean-forest-moon-star.',
    usagePercent: 18,
    usageText: '18 GB / 100 GB',
    status: 'Healthy',
    icon: <CloudOutlined sx={{ color: TOKENS.textSecondary }} />,
    actions: ['Open', 'Share link'],
  },
]

export const ModernShellPrototypeScreen: React.FC = () => {
  const isCompact = useMediaQuery((theme: Theme) => theme.breakpoints.down('lg'))
  const [activeFilter, setActiveFilter] = useState('all')
  const [drawerOpen, setDrawerOpen] = useState(true)
  const [activeDrawerTab, setActiveDrawerTab] = useState<DrawerTab>('Overview')
  const [hoveredMessageId, setHoveredMessageId] = useState<string | null>(null)
  const [messageMenu, setMessageMenu] = useState<{ anchorEl: HTMLElement | null; message?: Message }>({ anchorEl: null })
  const [channelViewMode, setChannelViewMode] = useState<ChannelMode>('chat')
  const [projectViewMode, setProjectViewMode] = useState<ProjectMode>('chat')

  // Contact management state
  const [contactDialogMode, setContactDialogMode] = useState<'add' | 'edit' | 'delete' | null>(null)
  const [selectedContact, setSelectedContact] = useState<Conversation | null>(null)

  // Entity menu state
  const [entityMenuAnchor, setEntityMenuAnchor] = useState<HTMLElement | null>(null)
  const [sidebarMenuState, setSidebarMenuState] = useState<{ anchorEl: HTMLElement | null; conversation: Conversation | null }>({
    anchorEl: null,
    conversation: null,
  })

  const conversations = useMemo<Conversation[]>(
    () => [
      {
        id: 'saorsa-labs',
        name: 'Saorsa Labs (Group)',
        type: 'group',
        snippet: 'David: Thanks for today guys, lots to think about…',
        time: '21:37',
        unread: 3,
        status: 'read',
        pinned: true,
        membersOnline: 3,
        org: 'Saorsa Labs',
        hasWebsite: true,
      },
      {
        id: 'saorsa-org',
        name: 'Saorsa Labs (Org)',
        type: 'organisation',
        snippet: 'Org overview · 12 members · 6 projects',
        time: '21:30',
        status: 'read',
        pinned: true,
        org: 'Saorsa Labs',
        hasWebsite: true,
      },
      {
        id: 'saorsa-general',
        name: 'General (Channel)',
        type: 'channel',
        snippet: 'Pinned note: Launch plan checkpoint tomorrow 10:00',
        time: '20:12',
        unread: 5,
        status: 'read',
        org: 'Saorsa Labs',
        channelMeta: {
          topic: 'Company-wide updates and announcements.',
          members: 42,
          integrations: ['Calendar', 'Storage Bot'],
        },
      },
      {
        id: 'project-lumos',
        name: 'Project Lumos',
        type: 'project',
        snippet: 'Storage pipeline deployed to region FRA1',
        time: '17:05',
        unread: 2,
        status: 'delivered',
        org: 'Saorsa Labs',
        hasWebsite: true,
        projectMeta: {
          status: 'Active',
          completion: 72,
          owner: 'Lauren McFadyen',
        },
      },
      {
        id: 'storage-ops',
        name: 'Storage Ops',
        type: 'storage',
        snippet: '🔔 Backup failed on lon1-seed-02',
        time: '16:40',
        unread: 1,
        status: 'sent',
        org: 'Systems',
      },
      {
        id: 'ben-thomson',
        name: 'Ben Thomson',
        type: 'person',
        snippet: 'Ok I will put the kilt away then 😄😄',
        time: '18:21',
        status: 'delivered',
        org: 'Direct messages',
        online: true,
      },
      {
        id: 'lauren',
        name: 'Lauren McFadyen',
        type: 'person',
        snippet: "That's OK Lauren, no worries",
        time: 'Yesterday',
        status: 'read',
        org: 'Direct messages',
        online: false,
      },
    ],
    []
  )

  const [selectedConversationId, setSelectedConversationId] = useState(() => conversations[0]?.id ?? '')

  // CRDT Message State
  const [messages, setMessages] = useState<Message[]>([])
  const [messageInputValue, setMessageInputValue] = useState('')
  const [ourPeerId, setOurPeerId] = useState<string>('')
  const messageSyncService = useRef(getMessageSyncService())
  const syncIntervalRef = useRef<NodeJS.Timeout | null>(null)

  // Convert CRDT message to UI Message format
  const convertCRDTToUIMessage = useCallback((crdtMsg: CRDTMessage, ourPeerId: string): Message => {
    const isOurMessage = crdtMsg.metadata.author_peer_id === ourPeerId

    return {
      id: crdtMsg.metadata.id,
      author: crdtMsg.content.author,
      text: crdtMsg.content.text,
      time: new Date(crdtMsg.metadata.timestamp).toLocaleTimeString('en-US', {
        hour: '2-digit',
        minute: '2-digit'
      }),
      self: isOurMessage,
      status: crdtMsg.local_state?.status?.toLowerCase() as 'sent' | 'delivered' | 'read' | undefined,
      threadCount: crdtMsg.local_state?.thread_count,
      latestReplyBy: crdtMsg.local_state?.latest_reply_by,
      reactions: crdtMsg.local_state?.reactions?.map(r => ({
        emoji: r.emoji,
        count: r.count,
        userReacted: r.user_reacted,
      })),
    }
  }, [])

  // Load messages for current conversation
  const loadMessages = useCallback(async (entityId: string) => {
    try {
      const crdtMessages = await messageSyncService.current.getMessages(entityId)
      const uiMessages = crdtMessages.map(msg => convertCRDTToUIMessage(msg, ourPeerId))
      setMessages(uiMessages)
      console.log(`📨 Loaded ${uiMessages.length} messages for entity ${entityId}`)
    } catch (error) {
      console.error('❌ Failed to load messages:', error)
      // Show system messages on error
      setMessages([
        {
          id: 'system-1',
          author: 'System',
          text: 'Messages are end-to-end encrypted.',
          time: 'Today',
          self: false,
          system: true,
        },
      ])
    }
  }, [ourPeerId, convertCRDTToUIMessage])

  // Initialize MessageSyncService with peer ID
  useEffect(() => {
    const initializeMessaging = async () => {
      // Generate or retrieve four-word peer ID
      // Priority: URL param > localStorage > default
      const urlParams = new URLSearchParams(window.location.search)
      const peerIdFromUrl = urlParams.get('peerId')
      const peerIdFromStorage = localStorage.getItem('testPeerId')
      const testPeerId = peerIdFromUrl || peerIdFromStorage || 'ocean-forest-moon-star'

      // Save to localStorage for persistence
      localStorage.setItem('testPeerId', testPeerId)
      setOurPeerId(testPeerId)

      try {
        await messageSyncService.current.initialize(testPeerId)
        console.log('✅ MessageSyncService initialized with peer:', testPeerId)

        // Load existing messages for selected conversation
        if (selectedConversationId) {
          const crdtMessages = await messageSyncService.current.getMessages(selectedConversationId)
          const uiMessages = crdtMessages.map(msg => convertCRDTToUIMessage(msg, testPeerId))
          setMessages(uiMessages)
        }
      } catch (error) {
        console.error('❌ Failed to initialize MessageSyncService:', error)
      }
    }

    initializeMessaging()

    // Cleanup on unmount
    return () => {
      if (syncIntervalRef.current) {
        clearInterval(syncIntervalRef.current)
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // Load messages when conversation changes
  useEffect(() => {
    if (selectedConversationId && ourPeerId) {
      loadMessages(selectedConversationId)
    }
  }, [selectedConversationId, ourPeerId, loadMessages])

  // Periodic sync for receiving new messages
  useEffect(() => {
    if (!selectedConversationId || !ourPeerId) return

    // Poll for new messages every 2 seconds
    const syncInterval = setInterval(async () => {
      try {
        const crdtMessages = await messageSyncService.current.getMessages(selectedConversationId)

        // Only update if message count changed
        setMessages(currentMessages => {
          if (crdtMessages.length !== currentMessages.length) {
            const uiMessages = crdtMessages.map(msg => convertCRDTToUIMessage(msg, ourPeerId))
            console.log(`🔄 Synced: ${uiMessages.length} messages (was ${currentMessages.length})`)
            return uiMessages
          }
          return currentMessages
        })
      } catch (error) {
        console.error('❌ Sync failed:', error)
      }
    }, 2000)

    syncIntervalRef.current = syncInterval

    return () => {
      clearInterval(syncInterval)
      syncIntervalRef.current = null
    }
  }, [selectedConversationId, ourPeerId, convertCRDTToUIMessage])

  // Send message handler
  const handleSendMessage = useCallback(async () => {
    if (!messageInputValue.trim() || !selectedConversationId || !ourPeerId) return

    const text = messageInputValue.trim()
    setMessageInputValue('') // Clear input immediately for better UX

    try {
      const conversation = conversations.find(c => c.id === selectedConversationId)
      if (!conversation) return

      // Determine entity type
      const entityType = conversation.type === 'person' ? 'person' :
                        conversation.type === 'group' ? 'group' :
                        conversation.type === 'project' ? 'project' :
                        conversation.type === 'channel' ? 'channel' : 'organisation'

      // Send via CRDT service
      const crdtMessage = await messageSyncService.current.sendMessage(
        selectedConversationId,
        entityType,
        text,
        conversation.name, // Use conversation name as author display name
        undefined // No reply-to for now
      )

      // Optimistically add to UI
      const uiMessage = convertCRDTToUIMessage(crdtMessage, ourPeerId)
      setMessages(prev => [...prev, uiMessage])

      console.log('✅ Message sent:', crdtMessage.metadata.id)
    } catch (error) {
      console.error('❌ Failed to send message:', error)
      // Re-add text to input on failure
      setMessageInputValue(text)
    }
  }, [messageInputValue, selectedConversationId, ourPeerId, conversations, convertCRDTToUIMessage])

  // Handle Enter key to send
  const handleKeyPress = useCallback((event: React.KeyboardEvent) => {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault()
      handleSendMessage()
    }
  }, [handleSendMessage])

  const defaultConversationId = useMemo(
    () => conversations.find(c => c.type !== 'organisation')?.id ?? conversations[0]?.id ?? '',
    [conversations]
  )

  const selectedConversation = useMemo(
    () => conversations.find(c => c.id === selectedConversationId) ?? conversations[0],
    [conversations, selectedConversationId]
  )

  const isOrganisationView = selectedConversation.type === 'organisation'
  const isChannelView = selectedConversation.type === 'channel'
  const isProjectView = selectedConversation.type === 'project'
  const isGroupConversation = selectedConversation.type !== 'person'

  useEffect(() => {
    setChannelViewMode('chat')
    setProjectViewMode('chat')
  }, [selectedConversationId])

  const headerSubtitle = useMemo(() => {
    switch (selectedConversation.type) {
      case 'project':
        return selectedConversation.projectMeta
          ? `${selectedConversation.projectMeta.status} · ${selectedConversation.projectMeta.owner} · ${selectedConversation.projectMeta.completion}%`
          : 'Project overview'
      case 'channel':
        return selectedConversation.channelMeta
          ? `${selectedConversation.channelMeta.topic} · ${selectedConversation.channelMeta.members} members`
          : 'Channel overview'
      case 'organisation':
        return 'Organisation overview'
      case 'storage':
        return 'Storage operations'
      case 'group':
        return `${selectedConversation.membersOnline ?? 3} online`
      default:
        return 'Direct message'
    }
  }, [selectedConversation])

  const filteredConversations = useMemo(() => {
    switch (activeFilter) {
      case 'unread':
        return conversations.filter(c => (c.unread ?? 0) > 0)
      case 'favourites':
        return conversations.filter(c => c.pinned)
      case 'groups':
        return conversations.filter(c => c.type === 'group')
      case 'projects':
        return conversations.filter(c => c.type === 'project')
      case 'people':
        return conversations.filter(c => c.type === 'person')
      default:
        return conversations
    }
  }, [conversations, activeFilter])

  const handleMessageMenuOpen = (event: React.MouseEvent<HTMLElement>, message: Message) => {
    setMessageMenu({ anchorEl: event.currentTarget, message })
  }

  const handleMessageMenuClose = () => setMessageMenu({ anchorEl: null })

  const handleHome = () => {
    if (defaultConversationId) {
      setSelectedConversationId(defaultConversationId)
      setChannelViewMode('chat')
      setProjectViewMode('chat')
    }
  }

  const handleWebsiteOpen = () => {
    if (selectedConversation.hasWebsite) {
      console.log(`Open website for ${selectedConversation.name}`)
    }
  }

  // Entity menu handlers
  const handleEntityMenuOpen = (event: React.MouseEvent<HTMLElement>) => {
    event.stopPropagation()
    setEntityMenuAnchor(event.currentTarget)
  }

  const handleEntityMenuClose = () => {
    setEntityMenuAnchor(null)
  }

  const handleSidebarMenuOpen = (event: React.MouseEvent<HTMLElement>, conversation: Conversation) => {
    event.stopPropagation()
    setSidebarMenuState({ anchorEl: event.currentTarget, conversation })
  }

  const handleSidebarMenuClose = () => {
    setSidebarMenuState({ anchorEl: null, conversation: null })
  }

  const handleEntityEdit = () => {
    setEntityMenuAnchor(null)
    setSelectedContact(selectedConversation)
    setContactDialogMode('edit')
  }

  const handleEntityDelete = () => {
    setEntityMenuAnchor(null)
    setSelectedContact(selectedConversation)
    setContactDialogMode('delete')
  }

  const handleSidebarEntityEdit = () => {
    if (sidebarMenuState.conversation) {
      setSidebarMenuState({ anchorEl: null, conversation: null })
      setSelectedContact(sidebarMenuState.conversation)
      setContactDialogMode('edit')
    }
  }

  const handleSidebarEntityDelete = () => {
    if (sidebarMenuState.conversation) {
      const conv = sidebarMenuState.conversation
      setSidebarMenuState({ anchorEl: null, conversation: null })
      setSelectedContact(conv)
      setContactDialogMode('delete')
    }
  }

  // Dialog callbacks that call Tauri backend
  const handleSaveEntityEdit = async (id: string, updates: Partial<Conversation>) => {
    try {
      if (typeof window !== 'undefined' && '__TAURI__' in window) {
        const { invoke } = await import('@tauri-apps/api/core')

        // Find the entity type from the conversation
        const entityType = selectedContact?.type || 'contact'

        await invoke('core_entity_update', {
          entityId: id,
          entityType,
          name: updates.name,
          description: updates.snippet,
        })

        console.log(`✅ Updated ${entityType}: ${updates.name}`)
      } else {
        console.log(`🔶 Browser mode: Would update entity ${id}`, updates)
      }

      setContactDialogMode(null)
      setSelectedContact(null)
    } catch (error) {
      console.error('Failed to update entity:', error)
      alert(`Failed to update: ${error}`)
    }
  }

  const handleConfirmEntityDelete = async (id: string) => {
    try {
      if (typeof window !== 'undefined' && '__TAURI__' in window) {
        const { invoke } = await import('@tauri-apps/api/core')

        // Find the entity type from the conversation
        const entityType = selectedContact?.type || 'contact'

        await invoke('core_entity_delete', {
          entityId: id,
          entityType,
        })

        console.log(`✅ Deleted ${entityType}: ${id}`)
      } else {
        console.log(`🔶 Browser mode: Would delete entity ${id}`)
      }

      setContactDialogMode(null)
      setSelectedContact(null)
    } catch (error) {
      console.error('Failed to delete entity:', error)
      alert(`Failed to delete: ${error}`)
    }
  }

  // Render functions for different view modes
  const renderOrganisationOverview = () => (
    <Box
      sx={{
        flexGrow: 1,
        px: 4,
        py: 4,
        overflowY: 'auto',
        display: 'grid',
        gridTemplateColumns: { xs: '1fr', md: 'repeat(2, minmax(0, 1fr))' },
        gridAutoRows: 'minmax(140px, auto)',
        gap: 3,
        backgroundImage: 'radial-gradient(circle at 0% 0%, rgba(46,182,125,0.08), transparent 60%)',
      }}
    >
      {/* Members Card with Remove/Edit actions */}
      <Box sx={{ bgcolor: alpha('#FFFFFF', 0.04), borderRadius: 3, p: 3, display: 'flex', flexDirection: 'column', gap: 1 }}>
        <Typography variant="subtitle1" fontWeight={600}>Members</Typography>
        <Typography variant="body2" sx={{ color: TOKENS.textSecondary }}>Hover to see details or manage participants.</Typography>
        <Stack spacing={1.2} sx={{ mt: 1 }}>
          {organisationOverviewData.members.map(member => (
            <Stack key={member.name} direction="row" spacing={1.5} alignItems="center" sx={{
              p: 1,
              borderRadius: 2,
              bgcolor: alpha('#FFFFFF', 0.02),
              '&:hover': { bgcolor: alpha(TOKENS.accent, 0.12) },
            }}>
              <Avatar sx={{ width: 32, height: 32, bgcolor: alpha(TOKENS.accent, 0.12) }}>
                {member.name.split(' ').map(n => n[0]).join('')}
              </Avatar>
              <Box sx={{ flexGrow: 1 }}>
                <Typography variant="body2" fontWeight={500}>{member.name}</Typography>
                <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>{member.role} · {member.status}</Typography>
              </Box>
              <Tooltip title="Edit role">
                <IconButton size="small" sx={{ color: TOKENS.textSecondary }} onClick={() => console.log('Edit member:', member.name)}>
                  <EditOutlined fontSize="inherit" />
                </IconButton>
              </Tooltip>
              <Tooltip title="Message privately">
                <IconButton size="small" sx={{ color: TOKENS.textSecondary }}>
                  <ReplyOutlined fontSize="inherit" />
                </IconButton>
              </Tooltip>
              <Tooltip title="Remove from org">
                <IconButton size="small" sx={{ color: TOKENS.danger }} onClick={() => console.log('Remove member:', member.name)}>
                  <DeleteOutline fontSize="inherit" />
                </IconButton>
              </Tooltip>
            </Stack>
          ))}
        </Stack>
      </Box>

      {/* Projects Card with Archive action */}
      <Box sx={{ bgcolor: alpha('#FFFFFF', 0.04), borderRadius: 3, p: 3, display: 'flex', flexDirection: 'column', gap: 1 }}>
        <Typography variant="subtitle1" fontWeight={600}>Projects</Typography>
        <Typography variant="body2" sx={{ color: TOKENS.textSecondary }}>Hover to open project or archive.</Typography>
        <Stack spacing={1.2} sx={{ mt: 1 }}>
          {organisationOverviewData.projects.map(project => (
            <Box
              key={project}
              sx={{
                display: 'flex',
                alignItems: 'center',
                gap: 1.2,
                p: 1,
                borderRadius: 2,
                bgcolor: alpha('#FFFFFF', 0.02),
                '&:hover': { bgcolor: alpha(TOKENS.accent, 0.12) },
              }}
            >
              <FolderOutlined sx={{ color: TOKENS.accent }} />
              <Typography variant="body2" fontWeight={500} sx={{ flexGrow: 1 }}>{project}</Typography>
              <Tooltip title="Archive project">
                <IconButton size="small" sx={{ color: TOKENS.textSecondary }} onClick={() => console.log('Archive project:', project)}>
                  <ArchiveOutlined fontSize="inherit" />
                </IconButton>
              </Tooltip>
            </Box>
          ))}
        </Stack>
      </Box>

      {/* Channels Card */}
      <Box sx={{ bgcolor: alpha('#FFFFFF', 0.04), borderRadius: 3, p: 3, display: 'flex', flexDirection: 'column', gap: 1 }}>
        <Typography variant="subtitle1" fontWeight={600}>Channels</Typography>
        <Typography variant="body2" sx={{ color: TOKENS.textSecondary }}>Hover to preview members.</Typography>
        <Stack spacing={1.2} sx={{ mt: 1 }}>
          {organisationOverviewData.channels.map(channel => (
            <Box
              key={channel}
              sx={{
                display: 'flex',
                alignItems: 'center',
                gap: 1.2,
                p: 1,
                borderRadius: 2,
                bgcolor: alpha('#FFFFFF', 0.02),
                '&:hover': { bgcolor: alpha('#FFFFFF', 0.08) },
              }}
            >
              <GridView sx={{ color: TOKENS.textSecondary }} />
              <Typography variant="body2" fontWeight={500}>{channel}</Typography>
            </Box>
          ))}
        </Stack>
      </Box>

      {/* Storage Card */}
      <Box sx={{ bgcolor: alpha('#FFFFFF', 0.04), borderRadius: 3, p: 3, display: 'flex', flexDirection: 'column', gap: 1 }}>
        <Typography variant="subtitle1" fontWeight={600}>Storage</Typography>
        <Typography variant="body2" sx={{ color: TOKENS.textSecondary }}>Manage encrypted vaults and virtual disks.</Typography>
        <Stack spacing={1.5} sx={{ mt: 1 }}>
          {storageContainers.map(container => (
            <Box key={container.id} sx={{ bgcolor: alpha('#FFFFFF', 0.02), borderRadius: 2, p: 1.5, display: 'flex', flexDirection: 'column', gap: 0.75 }}>
              <Stack direction="row" spacing={1.2} alignItems="center">
                {container.icon}
                <Box>
                  <Typography variant="body2" fontWeight={600}>{container.name}</Typography>
                  <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>{container.description}</Typography>
                </Box>
              </Stack>
              <Stack spacing={0.5}>
                <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>{container.usageText}</Typography>
                <LinearProgress
                  variant="determinate"
                  value={container.usagePercent}
                  sx={{
                    height: 6,
                    borderRadius: 999,
                    bgcolor: alpha('#FFFFFF', 0.1),
                    '& .MuiLinearProgress-bar': { backgroundColor: container.status === 'Healthy' ? TOKENS.accent : TOKENS.warning },
                  }}
                />
              </Stack>
              <Stack direction="row" spacing={1}>
                {container.actions.map(action => (
                  <Button
                    key={action}
                    size="small"
                    variant="outlined"
                    sx={{
                      textTransform: 'none',
                      borderRadius: 999,
                      color: TOKENS.accent,
                      borderColor: alpha(TOKENS.accent, 0.4),
                      '&:hover': { borderColor: TOKENS.accent },
                    }}
                  >
                    {action}
                  </Button>
                ))}
              </Stack>
            </Box>
          ))}
        </Stack>
      </Box>
    </Box>
  )

  const renderChannelMode = () => {
    // Only show non-chat modes when channel is selected
    if (!isChannelView || channelViewMode === 'chat') {
      return null
    }

    return (
      <Box sx={{ flexGrow: 1, px: 4, py: 4, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: 2 }}>
        {channelViewMode === 'threads' && (
          <Stack spacing={2}>
            {channelThreadPreviews.map(thread => (
              <Box key={thread.id} sx={{ bgcolor: alpha('#FFFFFF', 0.04), borderRadius: 3, p: 2.5 }}>
                <Typography variant="subtitle1" fontWeight={600}>{thread.title}</Typography>
                <Typography variant="body2" sx={{ color: TOKENS.textSecondary, mt: 0.5 }}>{thread.summary}</Typography>
                <Stack direction="row" spacing={1} sx={{ mt: 1 }}>
                  <Button size="small" variant="outlined" sx={{ textTransform: 'none', borderRadius: 999, color: TOKENS.accent, borderColor: alpha(TOKENS.accent, 0.4) }}>Open thread</Button>
                  <Button size="small" variant="outlined" sx={{ textTransform: 'none', borderRadius: 999, color: TOKENS.textSecondary, borderColor: alpha('#FFFFFF', 0.2) }}>Reply privately</Button>
                </Stack>
              </Box>
            ))}
          </Stack>
        )}
        {channelViewMode === 'files' && (
          <Stack spacing={1.2}>
            {channelFiles.map(file => (
              <Box key={file} sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', bgcolor: alpha('#FFFFFF', 0.04), borderRadius: 2, px: 2, py: 1.2 }}>
                <Typography variant="body2" fontWeight={500}>{file}</Typography>
                <Stack direction="row" spacing={1}>
                  <Button size="small" variant="outlined" sx={{ textTransform: 'none', borderRadius: 999, color: TOKENS.accent, borderColor: alpha(TOKENS.accent, 0.4) }}>Open</Button>
                  <Button size="small" variant="outlined" sx={{ textTransform: 'none', borderRadius: 999, color: TOKENS.textSecondary, borderColor: alpha('#FFFFFF', 0.2) }}>Share</Button>
                </Stack>
              </Box>
            ))}
          </Stack>
        )}
        {channelViewMode === 'integrations' && (
          <Stack spacing={1.2}>
            {channelIntegrations.map(integration => (
              <Box key={integration} sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', bgcolor: alpha('#FFFFFF', 0.04), borderRadius: 2, px: 2, py: 1.2 }}>
                <Box>
                  <Typography variant="body2" fontWeight={500}>{integration}</Typography>
                  <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>Connected · click to configure</Typography>
                </Box>
                <Button size="small" variant="outlined" sx={{ textTransform: 'none', borderRadius: 999, color: TOKENS.accent, borderColor: alpha(TOKENS.accent, 0.4) }}>Manage</Button>
              </Box>
            ))}
          </Stack>
        )}
      </Box>
    )
  }

  const renderProjectMode = () => {
    // Only show non-chat modes when project is selected
    if (!isProjectView || projectViewMode === 'chat') {
      return null
    }

    return (
      <Box sx={{ flexGrow: 1, px: 4, py: 4, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: 3 }}>
        {projectViewMode === 'board' && (
          <Box sx={{ display: 'grid', gridTemplateColumns: { xs: '1fr', lg: 'repeat(3, 1fr)' }, gap: 2 }}>
            {projectBoard.map(column => (
              <Box key={column.title} sx={{ bgcolor: alpha('#FFFFFF', 0.04), borderRadius: 2, p: 2, display: 'flex', flexDirection: 'column', gap: 1.2 }}>
                <Typography variant="subtitle2" fontWeight={600}>{column.title}</Typography>
                {column.items.map(item => (
                  <Box key={item} sx={{ bgcolor: alpha('#FFFFFF', 0.06), borderRadius: 2, p: 1.5 }}>
                    <Typography variant="body2" fontWeight={500}>{item}</Typography>
                    <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>Assignee: Lauren</Typography>
                  </Box>
                ))}
                <Button size="small" variant="outlined" sx={{ textTransform: 'none', borderRadius: 999, color: TOKENS.textSecondary, borderColor: alpha('#FFFFFF', 0.2) }}>Add card</Button>
              </Box>
            ))}
          </Box>
        )}
        {projectViewMode === 'tasks' && (
          <Stack spacing={1.2}>
            {projectTasks.map(task => (
              <Box key={task.title} sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', bgcolor: alpha('#FFFFFF', 0.04), borderRadius: 2, px: 2, py: 1.2 }}>
                <Box>
                  <Typography variant="body2" fontWeight={500}>{task.title}</Typography>
                  <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>{task.assignee} · {task.status} · Due {task.due}</Typography>
                </Box>
                <Stack direction="row" spacing={1}>
                  <Button size="small" variant="outlined" sx={{ textTransform: 'none', borderRadius: 999, color: TOKENS.accent, borderColor: alpha(TOKENS.accent, 0.4) }}>Update</Button>
                  <Button size="small" variant="outlined" sx={{ textTransform: 'none', borderRadius: 999, color: TOKENS.textSecondary, borderColor: alpha('#FFFFFF', 0.2) }}>Comment</Button>
                </Stack>
              </Box>
            ))}
          </Stack>
        )}
        {projectViewMode === 'timeline' && (
          <Stack spacing={1.2}>
            {projectTimeline.map(event => (
              <Typography key={event} variant="body2" sx={{ color: TOKENS.textSecondary }}>{event}</Typography>
            ))}
          </Stack>
        )}
      </Box>
    )
  }

  const renderChatTimeline = () => (
    <Box
      sx={{
        flexGrow: 1,
        backgroundImage: 'radial-gradient(circle at 0% 0%, rgba(46,182,125,0.08), transparent 60%)',
        px: 4,
        py: 4,
        display: 'flex',
        flexDirection: 'column',
        gap: 2,
        overflowY: 'auto',
      }}
    >
      <Stack spacing={2}>
        {messages.map(msg => (
          <Box
            key={msg.id}
            onMouseEnter={() => setHoveredMessageId(msg.id)}
            onMouseLeave={() => setHoveredMessageId(prev => (prev === msg.id ? null : prev))}
            sx={{ display: 'flex', flexDirection: 'column', alignItems: msg.self ? 'flex-end' : 'flex-start', gap: 0.75 }}
          >
            {msg.system ? (
              <Box
                sx={{
                  alignSelf: 'center',
                  px: 2,
                  py: 0.5,
                  borderRadius: 999,
                  bgcolor: alpha('#FFFFFF', 0.05),
                  color: TOKENS.textSecondary,
                  fontSize: 12,
                  textTransform: 'uppercase',
                  letterSpacing: 0.6,
                }}
              >
                {msg.text}
              </Box>
            ) : (
              <>
                <Bubble self={msg.self}>
                  <Typography variant="body2" sx={{ whiteSpace: 'pre-wrap' }}>{msg.text}</Typography>
                </Bubble>
                <Stack direction="row" spacing={0.5} alignItems="center" sx={{ color: TOKENS.textSecondary, fontSize: 11 }}>
                  <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>{msg.time}</Typography>
                  {msg.self && msg.status === 'read' && <CheckCircle sx={{ fontSize: 14, color: TOKENS.accent }} />}
                  {msg.self && msg.status === 'delivered' && <>
                    <Check sx={{ fontSize: 14, color: TOKENS.textSecondary }} />
                    <Check sx={{ fontSize: 14, color: TOKENS.textSecondary, ml: '-6px' }} />
                  </>}
                </Stack>
                {msg.reactions && msg.reactions.length > 0 && (
                  <Stack direction="row" spacing={0.75} sx={{ mt: 0.5 }}>
                    {msg.reactions.map(reaction => (
                      <Box
                        key={reaction.emoji}
                        sx={{
                          display: 'flex',
                          alignItems: 'center',
                          gap: 0.5,
                          bgcolor: alpha('#FFFFFF', 0.06),
                          borderRadius: 999,
                          px: 1,
                          py: 0.2,
                          fontSize: 12,
                          color: TOKENS.textSecondary,
                        }}
                      >
                        <span>{reaction.emoji}</span>
                        <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>{reaction.count}</Typography>
                      </Box>
                    ))}
                  </Stack>
                )}
                {msg.threadCount && (
                  <Button
                    size="small"
                    startIcon={<ForumOutlined sx={{ fontSize: 16 }} />}
                    sx={{
                      mt: 0.75,
                      alignSelf: msg.self ? 'flex-end' : 'flex-start',
                      textTransform: 'none',
                      borderRadius: 999,
                      bgcolor: alpha('#FFFFFF', 0.05),
                      color: TOKENS.textSecondary,
                      '&:hover': { bgcolor: alpha('#FFFFFF', 0.08) },
                    }}
                  >
                    View thread · {msg.threadCount} · Latest from {msg.latestReplyBy ?? 'you'}
                  </Button>
                )}
                {hoveredMessageId === msg.id && !msg.system && (
                  <Stack
                    direction="row"
                    spacing={1}
                    sx={{
                      mt: 1,
                      alignSelf: msg.self ? 'flex-end' : 'flex-start',
                      bgcolor: alpha('#FFFFFF', 0.05),
                      borderRadius: 999,
                      px: 1,
                      py: 0.5,
                    }}
                  >
                    {isGroupConversation && (
                      <Tooltip title="More options">
                        <IconButton size="small" sx={{ color: TOKENS.textSecondary }} onClick={event => handleMessageMenuOpen(event, msg)}>
                          <MoreVert fontSize="inherit" />
                        </IconButton>
                      </Tooltip>
                    )}
                    <Tooltip title="React">
                      <IconButton size="small" sx={{ color: TOKENS.textSecondary }}>
                        <EmojiEmotionsOutlined fontSize="inherit" />
                      </IconButton>
                    </Tooltip>
                    <Tooltip title="Reply">
                      <IconButton size="small" sx={{ color: TOKENS.textSecondary }}>
                        <ReplyOutlined fontSize="inherit" />
                      </IconButton>
                    </Tooltip>
                    <Tooltip title="Reply privately">
                      <IconButton size="small" sx={{ color: TOKENS.textSecondary }}>
                        <LockOutlined fontSize="inherit" />
                      </IconButton>
                    </Tooltip>
                    <Tooltip title="Start new thread">
                      <IconButton size="small" sx={{ color: TOKENS.textSecondary }}>
                        <ForumOutlined fontSize="inherit" />
                      </IconButton>
                    </Tooltip>
                    <Tooltip title="Share link">
                      <IconButton size="small" sx={{ color: TOKENS.textSecondary }}>
                        <LinkOutlined fontSize="inherit" />
                      </IconButton>
                    </Tooltip>
                  </Stack>
                )}
              </>
            )}
          </Box>
        ))}
      </Stack>
    </Box>
  )

  // Main component render
  return (
    <Box sx={{ display: 'flex', height: '100vh', bgcolor: TOKENS.bgBase, color: TOKENS.textPrimary }}>
      {/* A. System Rail (52px) */}
      <Box
        sx={{
          width: 52,
          bgcolor: TOKENS.bgRaised,
          borderRight: `1px solid ${TOKENS.borderSubtle}`,
          display: 'flex',
          flexDirection: 'column',
          p: 1,
          gap: 0.5,
        }}
      >
        <Stack spacing={0.5} sx={{ flexGrow: 1 }}>
          <Tooltip title="Home (⌘+1)" placement="right">
            <SystemRailButton onClick={handleHome}>
              <HomeOutlined />
            </SystemRailButton>
          </Tooltip>
          <Tooltip title="Chats" placement="right">
            <SystemRailButton>
              <ChatBubbleOutline />
            </SystemRailButton>
          </Tooltip>
          <Tooltip title="Organizations" placement="right">
            <SystemRailButton>
              <Apartment />
            </SystemRailButton>
          </Tooltip>
          <Tooltip title="Discover" placement="right">
            <SystemRailButton>
              <ExploreOutlined />
            </SystemRailButton>
          </Tooltip>
          <Tooltip title="Storage" placement="right">
            <SystemRailButton>
              <StorageOutlined />
            </SystemRailButton>
          </Tooltip>
          {selectedConversation.hasWebsite && (
            <Tooltip title="Website" placement="right">
              <SystemRailButton onClick={handleWebsiteOpen} sx={{ color: TOKENS.accent }}>
                <LanguageOutlined />
              </SystemRailButton>
            </Tooltip>
          )}
          <Tooltip title="Calls" placement="right">
            <SystemRailButton>
              <PhoneOutlined />
            </SystemRailButton>
          </Tooltip>
        </Stack>
        <Tooltip title="Settings" placement="right">
          <SystemRailButton>
            <SettingsOutlined />
          </SystemRailButton>
        </Tooltip>
        <Avatar sx={{ width: 40, height: 40, cursor: 'pointer', mx: 'auto' }}>DA</Avatar>
      </Box>

      {/* B. Conversation List (320-360px) */}
      <Box
        sx={{
          width: isCompact ? 280 : 360,
          bgcolor: TOKENS.bgRaised,
          borderRight: `1px solid ${TOKENS.borderSubtle}`,
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        {/* B1. Header */}
        <Box sx={{ p: 2, borderBottom: `1px solid ${TOKENS.borderSubtle}` }}>
          <Stack direction="row" spacing={1} alignItems="center" justifyContent="space-between">
            <Typography variant="h6" fontWeight={600}>Chats</Typography>
            <Stack direction="row" spacing={0.5}>
              <IconButton size="small" sx={{ color: TOKENS.textSecondary }}><Add /></IconButton>
              <IconButton size="small" sx={{ color: TOKENS.textSecondary }}><MoreHoriz /></IconButton>
            </Stack>
          </Stack>
        </Box>

        {/* B2. Filters */}
        <Box sx={{ px: 2, py: 1.5, borderBottom: `1px solid ${TOKENS.borderSubtle}` }}>
          <Stack direction="row" spacing={0.75} sx={{ overflowX: 'auto' }}>
            {filters.map(f => (
              <FilterChip
                key={f.key}
                label={f.label}
                variant={activeFilter === f.key ? 'filled' : 'outlined'}
                onClick={() => setActiveFilter(f.key)}
              />
            ))}
          </Stack>
        </Box>

        {/* B3. Search */}
        <Box sx={{ px: 2, py: 1.5 }}>
          <InputBase
            placeholder="Search..."
            startAdornment={<SearchIcon sx={{ mr: 1, color: TOKENS.textSecondary }} />}
            sx={{
              width: '100%',
              bgcolor: alpha('#FFFFFF', 0.05),
              borderRadius: 2,
              px: 1.5,
              py: 0.75,
              color: TOKENS.textPrimary,
              fontSize: 14,
            }}
          />
        </Box>

        {/* B4. List Items */}
        <Box sx={{ flexGrow: 1, overflowY: 'auto', px: 1 }}>
          {filteredConversations.map(conv => {
            const badge = presenceBadge(conv)
            return (
              <ConversationListItem
                key={conv.id}
                selected={conv.id === selectedConversationId}
                onClick={() => setSelectedConversationId(conv.id)}
                sx={{
                  position: 'relative',
                  '&:hover .entity-menu-button': {
                    opacity: 1,
                  },
                }}
              >
                <Box sx={{ position: 'relative' }}>
                  <Avatar sx={{ width: 48, height: 48, ...avatarShapeStyles[conv.type] }}>
                    {conv.name.substring(0, 2).toUpperCase()}
                  </Avatar>
                  <Box
                    sx={{
                      position: 'absolute',
                      bottom: -2,
                      right: -2,
                      width: 20,
                      height: 20,
                      borderRadius: '50%',
                      bgcolor: badge.bg,
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      border: `2px solid ${TOKENS.bgRaised}`,
                    }}
                  >
                    {badge.icon}
                  </Box>
                </Box>
                <Box sx={{ flexGrow: 1, minWidth: 0 }}>
                  <Stack direction="row" spacing={1} alignItems="center" justifyContent="space-between">
                    <Stack direction="row" spacing={0.5} alignItems="center" sx={{ flexGrow: 1, minWidth: 0 }}>
                      <Typography variant="body2" fontWeight={600} noWrap>{conv.name}</Typography>
                      <IconButton
                        className="entity-menu-button"
                        size="small"
                        onClick={(e) => handleSidebarMenuOpen(e, conv)}
                        sx={{
                          opacity: 0,
                          transition: 'opacity 0.2s',
                          color: TOKENS.textSecondary,
                          padding: 0.25,
                          '&:hover': {
                            bgcolor: alpha('#FFFFFF', 0.08),
                          },
                        }}
                      >
                        <KeyboardArrowDown fontSize="small" />
                      </IconButton>
                    </Stack>
                    <Typography variant="caption" sx={{ color: TOKENS.textSecondary, flexShrink: 0 }}>{conv.time}</Typography>
                  </Stack>
                  <Stack direction="row" spacing={0.5} alignItems="center">
                    <Typography variant="body2" sx={{ color: TOKENS.textSecondary, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                      {conv.snippet}
                    </Typography>
                    {conv.status === 'read' && <CheckCircle sx={{ fontSize: 14, color: TOKENS.accent, flexShrink: 0 }} />}
                  </Stack>
                </Box>
                {conv.unread && (
                  <Badge badgeContent={conv.unread} sx={{ '& .MuiBadge-badge': { bgcolor: TOKENS.accent, color: '#000' } }} />
                )}
              </ConversationListItem>
            )
          })}
        </Box>

        {/* B5. Connection Status */}
        <Box sx={{ borderTop: `1px solid ${TOKENS.borderSubtle}` }}>
          <ConnectionStatus compact={true} refreshInterval={15000} />
        </Box>
      </Box>

      {/* C. Conversation Pane */}
      <Box sx={{ flexGrow: 1, display: 'flex', flexDirection: 'column', position: 'relative' }}>
        {/* C1. Header */}
        <Box
          sx={{
            height: 64,
            bgcolor: TOKENS.bgRaised,
            borderBottom: `1px solid ${TOKENS.borderSubtle}`,
            px: 3,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
          }}
        >
          <Stack direction="row" spacing={1.5} alignItems="center">
            <Avatar sx={{ width: 40, height: 40 }}>{selectedConversation.name.substring(0, 2)}</Avatar>
            <Box
              onClick={handleEntityMenuOpen}
              sx={{
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: 0.5,
                '&:hover': {
                  '& .entity-dropdown-arrow': {
                    opacity: 1,
                  },
                },
              }}
            >
              <Box>
                <Typography variant="subtitle1" fontWeight={600}>{selectedConversation.name}</Typography>
                <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>{headerSubtitle}</Typography>
              </Box>
              <KeyboardArrowDown
                className="entity-dropdown-arrow"
                sx={{
                  fontSize: 18,
                  color: TOKENS.textSecondary,
                  opacity: 0.5,
                  transition: 'opacity 0.2s',
                }}
              />
            </Box>
          </Stack>
          <Stack direction="row" spacing={1}>
            <Tooltip title="Call">
              <IconButton sx={{ color: TOKENS.textSecondary }}>
                <Call />
              </IconButton>
            </Tooltip>
            <Tooltip title="Video">
              <IconButton sx={{ color: TOKENS.textSecondary }}>
                <VideocamOutlined />
              </IconButton>
            </Tooltip>
            <Tooltip title={drawerOpen ? 'Close info' : 'Open info'}>
              <IconButton sx={{ color: TOKENS.textSecondary }} onClick={() => setDrawerOpen(o => !o)}>
                <InfoOutlined />
              </IconButton>
            </Tooltip>
          </Stack>
        </Box>

        {/* View Mode Chips for Channels */}
        {isChannelView && (
          <Box sx={{ px: 3, py: 1.5, borderBottom: `1px solid ${TOKENS.borderSubtle}`, bgcolor: TOKENS.bgRaised }}>
            <Stack direction="row" spacing={1}>
              {channelModes.map(mode => (
                <ViewChip
                  key={mode.key}
                  label={mode.label}
                  variant={channelViewMode === mode.key ? 'filled' : 'outlined'}
                  onClick={() => setChannelViewMode(mode.key)}
                />
              ))}
            </Stack>
          </Box>
        )}

        {/* View Mode Chips for Projects */}
        {isProjectView && (
          <Box sx={{ px: 3, py: 1.5, borderBottom: `1px solid ${TOKENS.borderSubtle}`, bgcolor: TOKENS.bgRaised }}>
            <Stack direction="row" spacing={1}>
              {projectModes.map(mode => (
                <ViewChip
                  key={mode.key}
                  label={mode.label}
                  variant={projectViewMode === mode.key ? 'filled' : 'outlined'}
                  onClick={() => setProjectViewMode(mode.key)}
                />
              ))}
            </Stack>
          </Box>
        )}

        {/* C2. Content Area - renders different views based on mode */}
        {isOrganisationView ? renderOrganisationOverview() :
         isChannelView && channelViewMode !== 'chat' ? renderChannelMode() :
         isProjectView && projectViewMode !== 'chat' ? renderProjectMode() :
         renderChatTimeline()}

        {/* C3. Composer (only show for chat modes) */}
        {!isOrganisationView &&
         ((!isChannelView && !isProjectView) ||
          (isChannelView && channelViewMode === 'chat') ||
          (isProjectView && projectViewMode === 'chat')) && (
          <Box
            sx={{
              p: 2,
              bgcolor: alpha(TOKENS.bgRaised, 0.95),
              backdropFilter: 'blur(10px)',
              borderTop: `1px solid ${TOKENS.borderSubtle}`,
            }}
          >
            <Box
              sx={{
                bgcolor: alpha('#FFFFFF', 0.05),
                borderRadius: 3,
                px: 2,
                py: 1.5,
                display: 'flex',
                alignItems: 'center',
                gap: 1.5,
              }}
            >
              <IconButton size="small" sx={{ color: TOKENS.textSecondary }}><EmojiEmotionsOutlined /></IconButton>
              <IconButton size="small" sx={{ color: TOKENS.textSecondary }}><AttachFileOutlined /></IconButton>
              <InputBase
                placeholder="Message..."
                value={messageInputValue}
                onChange={(e) => setMessageInputValue(e.target.value)}
                onKeyPress={handleKeyPress}
                sx={{ flexGrow: 1, color: TOKENS.textPrimary, fontSize: 14 }}
              />
              <IconButton
                size="small"
                sx={{ color: TOKENS.accent }}
                onClick={handleSendMessage}
                disabled={!messageInputValue.trim()}
              >
                <SendRounded />
              </IconButton>
            </Box>
          </Box>
        )}

        {/* C4. Context Drawer */}
        {drawerOpen && (
          <Box
            sx={{
              position: 'absolute',
              top: 64,
              right: 0,
              bottom: 0,
              width: 320,
              bgcolor: TOKENS.bgRaised,
              borderLeft: `1px solid ${TOKENS.borderSubtle}`,
              display: 'flex',
              flexDirection: 'column',
              zIndex: 10,
            }}
          >
            {/* Drawer Tabs */}
            <Box sx={{ px: 2, py: 1.5, borderBottom: `1px solid ${TOKENS.borderSubtle}` }}>
              <Stack direction="row" spacing={0.75} sx={{ overflowX: 'auto', flexWrap: 'wrap', gap: 0.75 }}>
                {(['Overview', 'Members', 'Files', 'Tasks', 'Timeline', 'Storage'] as DrawerTab[]).map(tab => (
                  <DrawerTabChip
                    key={tab}
                    label={tab}
                    size="small"
                    variant={activeDrawerTab === tab ? 'filled' : 'outlined'}
                    onClick={() => setActiveDrawerTab(tab)}
                  />
                ))}
              </Stack>
            </Box>

            {/* Drawer Content */}
            <Box sx={{ flexGrow: 1, overflowY: 'auto', p: 2 }}>
              <Typography variant="body2" sx={{ color: TOKENS.textSecondary }}>
                {activeDrawerTab} content goes here
              </Typography>
            </Box>
          </Box>
        )}
      </Box>

      {/* Message Context Menu */}
      <Menu
        anchorEl={messageMenu.anchorEl}
        open={Boolean(messageMenu.anchorEl)}
        onClose={handleMessageMenuClose}
        sx={{ '& .MuiPaper-root': { bgcolor: TOKENS.bgRaised, color: TOKENS.textPrimary } }}
      >
        <MenuItem onClick={handleMessageMenuClose}>
          <ListItemIcon><ReplyOutlined sx={{ color: TOKENS.textPrimary }} /></ListItemIcon>
          <ListItemText>Reply</ListItemText>
        </MenuItem>
        <MenuItem onClick={handleMessageMenuClose}>
          <ListItemIcon><ForwardOutlined sx={{ color: TOKENS.textPrimary }} /></ListItemIcon>
          <ListItemText>Forward</ListItemText>
        </MenuItem>
        <MenuItem onClick={handleMessageMenuClose}>
          <ListItemIcon><ContentCopyOutlined sx={{ color: TOKENS.textPrimary }} /></ListItemIcon>
          <ListItemText>Copy</ListItemText>
        </MenuItem>
        <MenuItem onClick={handleMessageMenuClose}>
          <ListItemIcon><StarBorder sx={{ color: TOKENS.textPrimary }} /></ListItemIcon>
          <ListItemText>Star</ListItemText>
        </MenuItem>
        <Divider />
        <MenuItem onClick={handleMessageMenuClose}>
          <ListItemIcon><ReportProblemOutlined sx={{ color: TOKENS.danger }} /></ListItemIcon>
          <ListItemText sx={{ color: TOKENS.danger }}>Report</ListItemText>
        </MenuItem>
        <MenuItem onClick={handleMessageMenuClose}>
          <ListItemIcon><DeleteOutline sx={{ color: TOKENS.danger }} /></ListItemIcon>
          <ListItemText sx={{ color: TOKENS.danger }}>Delete</ListItemText>
        </MenuItem>
      </Menu>

      {/* Entity Menu (Header) */}
      <Menu
        anchorEl={entityMenuAnchor}
        open={Boolean(entityMenuAnchor)}
        onClose={handleEntityMenuClose}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'left' }}
        transformOrigin={{ vertical: 'top', horizontal: 'left' }}
        PaperProps={{
          sx: {
            bgcolor: TOKENS.bgRaised,
            border: `1px solid ${TOKENS.borderSubtle}`,
            borderRadius: 2,
            minWidth: 200,
            mt: 0.5,
          },
        }}
      >
        <MenuItem onClick={handleEntityEdit}>
          <ListItemIcon>
            <EditOutlined fontSize="small" sx={{ color: TOKENS.textPrimary }} />
          </ListItemIcon>
          <ListItemText>Edit {selectedConversation.type === 'person' ? 'Contact' : 'Entity'}</ListItemText>
        </MenuItem>
        <Divider sx={{ my: 0.5, borderColor: TOKENS.borderSubtle }} />
        <MenuItem onClick={handleEntityDelete}>
          <ListItemIcon>
            <DeleteOutline fontSize="small" sx={{ color: TOKENS.danger }} />
          </ListItemIcon>
          <ListItemText sx={{ color: TOKENS.danger }}>
            Delete {selectedConversation.type === 'person' ? 'Contact' : 'Entity'}
          </ListItemText>
        </MenuItem>
      </Menu>

      {/* Entity Menu (Sidebar) */}
      <Menu
        anchorEl={sidebarMenuState.anchorEl}
        open={Boolean(sidebarMenuState.anchorEl)}
        onClose={handleSidebarMenuClose}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'left' }}
        transformOrigin={{ vertical: 'top', horizontal: 'left' }}
        PaperProps={{
          sx: {
            bgcolor: TOKENS.bgRaised,
            border: `1px solid ${TOKENS.borderSubtle}`,
            borderRadius: 2,
            minWidth: 200,
            mt: 0.5,
          },
        }}
      >
        <MenuItem onClick={handleSidebarEntityEdit}>
          <ListItemIcon>
            <EditOutlined fontSize="small" sx={{ color: TOKENS.textPrimary }} />
          </ListItemIcon>
          <ListItemText>
            Edit {sidebarMenuState.conversation?.type === 'person' ? 'Contact' : 'Entity'}
          </ListItemText>
        </MenuItem>
        <Divider sx={{ my: 0.5, borderColor: TOKENS.borderSubtle }} />
        <MenuItem onClick={handleSidebarEntityDelete}>
          <ListItemIcon>
            <DeleteOutline fontSize="small" sx={{ color: TOKENS.danger }} />
          </ListItemIcon>
          <ListItemText sx={{ color: TOKENS.danger }}>
            Delete {sidebarMenuState.conversation?.type === 'person' ? 'Contact' : 'Entity'}
          </ListItemText>
        </MenuItem>
      </Menu>

      {/* Entity Management Dialogs */}
      <EditContactDialog
        open={contactDialogMode === 'edit'}
        contact={selectedContact}
        onClose={() => {
          setContactDialogMode(null)
          setSelectedContact(null)
        }}
        onSave={handleSaveEntityEdit}
      />

      <DeleteContactDialog
        open={contactDialogMode === 'delete'}
        contact={selectedContact}
        onClose={() => {
          setContactDialogMode(null)
          setSelectedContact(null)
        }}
        onConfirm={handleConfirmEntityDelete}
      />
    </Box>
  )
}
