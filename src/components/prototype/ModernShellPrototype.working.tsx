import React, { useMemo, useState, useEffect } from 'react'
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
  KeyboardArrowDown,
  PersonOutline,
  HomeOutlined,
  LanguageOutlined,
  ArchiveOutlined,
} from '@mui/icons-material'
import { styled } from '@mui/material/styles'

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
  org?: string
  membersOnline?: number
  online?: boolean
  hasWebsite?: boolean
  description?: string
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
  reactions?: { emoji: string; count: number }[]
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

const ConversationPrototype: React.FC = () => {
  const isCompact = useMediaQuery((theme: Theme) => theme.breakpoints.down('lg'))
  const [activeFilter, setActiveFilter] = useState('all')
  const [drawerOpen, setDrawerOpen] = useState(true)
  const [activeDrawerTab, setActiveDrawerTab] = useState<DrawerTab>('Overview')
  const [hoveredMessageId, setHoveredMessageId] = useState<string | null>(null)
  const [messageMenu, setMessageMenu] = useState<{ anchorEl: HTMLElement | null; message?: Message }>({ anchorEl: null })
  const [channelViewMode, setChannelViewMode] = useState<ChannelMode>('chat')
  const [projectViewMode, setProjectViewMode] = useState<ProjectMode>('chat')

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
        snippet: 'That’s OK Lauren, no worries',
        time: 'Yesterday',
        status: 'read',
        org: 'Direct messages',
        online: false,
      },
    ],
    []
  )

  const [selectedConversationId, setSelectedConversationId] = useState(() => conversations[0]?.id ?? '')

  const messages = useMemo<Message[]>(
    () => [
      {
        id: '1',
        author: 'System',
        text: 'You created this group',
        time: 'Today',
        self: false,
        system: true,
      },
      {
        id: '2',
        author: 'System',
        text: 'Messages and calls are end-to-end encrypted.',
        time: 'Today',
        self: false,
        system: true,
      },
      {
        id: '3',
        author: 'David Allan',
        text: 'Thanks for today guys, it’s a lot to go through, but very interesting meeting. We have a lot to think about here',
        time: '21:35',
        self: true,
        status: 'read',
        threadCount: 3,
        latestReplyBy: 'Lauren',
        reactions: [
          { emoji: '👍', count: 4 },
          { emoji: '🎉', count: 2 },
        ],
      },
      {
        id: '4',
        author: 'Lauren',
        text: 'Good to see you all. Will reflect and come back with thoughts. Using Communitas until something better is available? 😄',
        time: '21:37',
        self: false,
        status: 'delivered',
        reactions: [{ emoji: '😄', count: 3 }],
      },
    ],
    []
  )

  const defaultConversationId = useMemo(
    () => conversations.find(c => c.type !== 'organisation')?.id ?? conversations[0]?.id ?? '',
    [conversations]
  )

  const selectedConversation = useMemo(
    () => conversations.find(c => c.id === selectedConversationId) ?? conversations[0],
    [conversations, selectedConversationId]
  )

  const isOrganisationView = selectedConversation.type === 'organisation'
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
      <Box sx={{ bgcolor: alpha('#FFFFFF', 0.04), borderRadius: 3, p: 3, display: 'flex', flexDirection: 'column', gap: 1 }}>
        <Typography variant="subtitle1" fontWeight={600}>Members</Typography>
        <Typography variant="body2" sx={{ color: TOKENS.textSecondary }}>Hover to see details or remove participants.</Typography>
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
              <Tooltip title="Message privately">
                <IconButton size="small" sx={{ color: TOKENS.textSecondary }}><ReplyOutlined fontSize="inherit" /></IconButton>
              </Tooltip>
              <Tooltip title="Remove">
                <IconButton size="small" sx={{ color: TOKENS.textSecondary }}><CloseIcon fontSize="inherit" /></IconButton>
              </Tooltip>
            </Stack>
          ))}
        </Stack>
      </Box>

      <Box sx={{ bgcolor: alpha('#FFFFFF', 0.04), borderRadius: 3, p: 3, display: 'flex', flexDirection: 'column', gap: 1 }}>
        <Typography variant="subtitle1" fontWeight={600}>Projects</Typography>
        <Typography variant="body2" sx={{ color: TOKENS.textSecondary }}>Hover to open project workspace.</Typography>
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
              <Typography variant="body2" fontWeight={500}>{project}</Typography>
            </Box>
          ))}
        </Stack>
      </Box>

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
                    '& .MuiLinearProgress-bar': { backgroundColor: container.status === 'Healthy' ? TOKENS.accent : '#F5B759' },
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

  const renderChannelMode = () => (
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

  const renderProjectMode = () => (
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
                          <KeyboardArrowDown fontSize="inherit" />
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
