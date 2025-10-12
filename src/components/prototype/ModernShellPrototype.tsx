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
  Modal,
  Paper,
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
  ExpandLess,
  ExpandMore,
  Tag,
  Groups,
  WorkOutline,
  Message,
  AddCircleOutline,
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
import { useEntityDirectory } from '../../contexts/EntityDirectoryContext'
import { EntityDocumentWorkspace } from '../documents/EntityDocumentWorkspace'
import { useSnackbar } from 'notistack'
import { AnimatePresence, motion } from 'framer-motion'
import { useAuth } from '../../contexts/AuthContext'
import { IdentityPicker } from '../auth/IdentityPicker'
import { UnifiedAuthFlow } from '../auth/UnifiedAuthFlow'
import { FirstLaunchWelcome } from '../auth/FirstLaunchWelcome'
import { Dialog, DialogTitle, DialogContent, DialogActions, TextField, InputAdornment, Alert } from '@mui/material'
import { LockOutlined as LockIcon, Visibility as VisibilityIcon, VisibilityOff as VisibilityOffIcon } from '@mui/icons-material'

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

const LIST_ITEM_TRANSITION = { duration: 0.18, ease: [0.4, 0, 0.2, 1] as [number, number, number, number] }

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
  orgId?: string
  membersOnline?: number
  online?: boolean
  hasWebsite?: boolean
  description?: string
  fourWords?: string // Four-word network address for contacts
  projectMeta?: {
    status: 'Active' | 'Planning' | 'Blocked' | 'Completed' | 'Archived'
    completion: number
    owner: string
  }
  channelMeta?: {
    topic: string
    members: number
    integrations?: string[]
  }
  scope?: 'organization' | 'personal'
}

type CommandPaletteEntityItem = {
  id: string
  type: 'entity'
  conversation: Conversation
  label: string
  subtitle?: string
}

type CommandPaletteActionItem = {
  id: string
  type: 'action'
  label: string
  subtitle?: string
  run: () => Promise<void> | void
}

type CommandPaletteItem = CommandPaletteEntityItem | CommandPaletteActionItem

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

type ChannelMode = 'chat' | 'threads' | 'files' | 'website' | 'integrations'
type ProjectMode = 'chat' | 'threads' | 'files' | 'website' | 'board' | 'tasks' | 'timeline'
type GroupMode = 'chat' | 'threads' | 'files' | 'website'
type PersonMode = 'chat' | 'threads' | 'files' | 'website'

const scopeFilters: { key: 'all' | 'organization' | 'personal'; label: string }[] = [
  { key: 'all', label: 'All Spaces' },
  { key: 'organization', label: 'Organisations' },
  { key: 'personal', label: 'Personal' },
]

const typeFilters: { key: 'all' | 'channels' | 'projects' | 'groups' | 'people'; label: string }[] = [
  { key: 'all', label: 'All Types' },
  { key: 'channels', label: 'Channels' },
  { key: 'projects', label: 'Projects' },
  { key: 'groups', label: 'Groups' },
  { key: 'people', label: 'People' },
]

const channelModes: { key: ChannelMode; label: string }[] = [
  { key: 'chat', label: 'Chat' },
  { key: 'threads', label: 'Threads' },
  { key: 'files', label: 'Files' },
  { key: 'website', label: 'Website' },
  { key: 'integrations', label: 'Integrations' },
]

const projectModes: { key: ProjectMode; label: string }[] = [
  { key: 'chat', label: 'Chat' },
  { key: 'threads', label: 'Threads' },
  { key: 'files', label: 'Files' },
  { key: 'website', label: 'Website' },
  { key: 'board', label: 'Board' },
  { key: 'tasks', label: 'Tasks' },
  { key: 'timeline', label: 'Timeline' },
]

const groupModes: { key: GroupMode; label: string }[] = [
  { key: 'chat', label: 'Chat' },
  { key: 'threads', label: 'Threads' },
  { key: 'files', label: 'Files' },
  { key: 'website', label: 'Website' },
]

const personModes: { key: PersonMode; label: string }[] = [
  { key: 'chat', label: 'Chat' },
  { key: 'threads', label: 'Threads' },
  { key: 'files', label: 'Files' },
  { key: 'website', label: 'Website' },
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

const conversationTypeLabel: Record<ConversationType, string> = {
  person: 'Direct message',
  group: 'Group',
  project: 'Project',
  channel: 'Channel',
  organisation: 'Organisation',
  storage: 'Storage',
}

const getEntityColor = (conversation: Conversation): string => {
  const isOrgScoped = !!conversation.org

  switch (conversation.type) {
    case 'storage': return TOKENS.accent
    case 'organisation': return TOKENS.accent
    case 'project': return '#00BFA5'
    case 'channel': return '#42A5F5'
    case 'group': return isOrgScoped ? '#AB47BC' : '#FF7043'
    case 'person':
      if (isOrgScoped) return '#26C6DA'
      return conversation.online ? TOKENS.accent : alpha('#FFFFFF', 0.25)
    default: return alpha('#FFFFFF', 0.25)
  }
}

const getEntityIcon = (conversation: Conversation) => {
  const isOrgScoped = !!conversation.org
  const color = getEntityColor(conversation)

  switch (conversation.type) {
    case 'storage':
      return <StorageOutlined sx={{ fontSize: 16, color: alpha(color, 0.8) }} />
    case 'organisation':
      return <Apartment sx={{ fontSize: 16, color: alpha(color, 0.8) }} />
    case 'project':
      return <WorkOutline sx={{ fontSize: 16, color: alpha(color, 0.8) }} />
    case 'channel':
      return <Tag sx={{ fontSize: 16, color: alpha(color, 0.8) }} />
    case 'group':
      if (isOrgScoped) {
        return <Groups sx={{ fontSize: 16, color: alpha(color, 0.8) }} />
      } else {
        return <PeopleOutline sx={{ fontSize: 16, color: alpha(color, 0.8) }} />
      }
    case 'person':
      if (isOrgScoped) {
        return <Message sx={{ fontSize: 16, color: alpha(color, 0.8) }} />
      } else {
        return null
      }
    default:
      return null
  }
}

const presenceBadge = (conversation: Conversation) => {
  const isOrgScoped = !!conversation.org
  const color = getEntityColor(conversation)

  switch (conversation.type) {
    case 'storage':
      return { icon: <StorageOutlined sx={{ fontSize: 18, color: '#FFFFFF' }} />, bg: color, borderColor: color }
    case 'organisation':
      return { icon: <Apartment sx={{ fontSize: 18, color: '#FFFFFF' }} />, bg: color, borderColor: color }
    case 'project':
      return { icon: <WorkOutline sx={{ fontSize: 18, color: '#FFFFFF' }} />, bg: color, borderColor: color }
    case 'channel':
      return { icon: <Tag sx={{ fontSize: 18, color: '#FFFFFF' }} />, bg: color, borderColor: color }
    case 'group':
      if (isOrgScoped) {
        return { icon: <Groups sx={{ fontSize: 18, color: '#FFFFFF' }} />, bg: color, borderColor: color }
      } else {
        return { icon: <PeopleOutline sx={{ fontSize: 18, color: '#FFFFFF' }} />, bg: color, borderColor: color }
      }
    case 'person':
      if (isOrgScoped) {
        return { icon: <Message sx={{ fontSize: 18, color: '#FFFFFF' }} />, bg: color, borderColor: color }
      } else {
        const online = conversation.online ?? false
        return { icon: null, bg: online ? TOKENS.accent : alpha('#FFFFFF', 0.25), borderColor: color }
      }
    default: {
      const online = conversation.online ?? (conversation.membersOnline ?? 0) > 0
      return { icon: null, bg: online ? TOKENS.accent : alpha('#FFFFFF', 0.25), borderColor: color }
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

// Hardcoded mock data removed - now using real organization data from backend

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
  const {
    organizations,
    personalGroups,
    personalUsers,
    createOrganization,
    createChannel,
    createProject,
    createGroup,
    createContact,
  } = useEntityDirectory()
  const { enqueueSnackbar } = useSnackbar()

  // Authentication state
  const { authState, login, logout, signInWithPasskey, isFirstLaunch, getRecentIdentities } = useAuth()
  const [showIdentityPicker, setShowIdentityPicker] = useState(false)
  const [showFirstLaunchWelcome, setShowFirstLaunchWelcome] = useState(false)
  const [showUnifiedAuthFlow, setShowUnifiedAuthFlow] = useState(false)
  const [showPasswordDialog, setShowPasswordDialog] = useState(false)
  const [selectedIdentity, setSelectedIdentity] = useState<string>('')
  const [password, setPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [authError, setAuthError] = useState<string | null>(null)
  const [isAuthenticating, setIsAuthenticating] = useState(false)

  const [scopeFilter, setScopeFilter] = useState<'all' | 'organization' | 'personal'>('all')
  const [typeFilter, setTypeFilter] = useState<'all' | 'channels' | 'projects' | 'groups' | 'people'>('all')
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [activeDrawerTab, setActiveDrawerTab] = useState<DrawerTab>('Overview')
  const [hoveredMessageId, setHoveredMessageId] = useState<string | null>(null)
  const [messageMenu, setMessageMenu] = useState<{ anchorEl: HTMLElement | null; message?: Message }>({ anchorEl: null })
  const [channelViewMode, setChannelViewMode] = useState<ChannelMode>('chat')
  const [projectViewMode, setProjectViewMode] = useState<ProjectMode>('chat')
  const [groupViewMode, setGroupViewMode] = useState<GroupMode>('chat')
  const [personViewMode, setPersonViewMode] = useState<PersonMode>('chat')
  const [preferencesLoaded, setPreferencesLoaded] = useState(false)
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false)
  const [commandQuery, setCommandQuery] = useState('')
  const commandInputRef = useRef<HTMLInputElement | null>(null)
  const searchInputRef = useRef<HTMLInputElement | null>(null)
  const scopeChipRefs = useRef<(HTMLElement | null)[]>([])
  const typeChipRefs = useRef<(HTMLElement | null)[]>([])

  // Organization expansion state
  const [expandedOrgs, setExpandedOrgs] = useState<Set<string>>(new Set())

  // Contact management state
  const [contactDialogMode, setContactDialogMode] = useState<'add' | 'edit' | 'delete' | null>(null)
  const [selectedContact, setSelectedContact] = useState<Conversation | null>(null)

  // Organization and group management state
  const [showOrgManagementDialog, setShowOrgManagementDialog] = useState(false)
  const [showGroupManagementDialog, setShowGroupManagementDialog] = useState(false)

  const conversationToContact = useCallback((conversation: Conversation): Contact => ({
    id: conversation.id,
    name: conversation.name,
    fourWords: conversation.fourWords ?? conversation.id,
    snippet: conversation.snippet,
    time: conversation.time,
    online: conversation.online,
    starred: conversation.starred ?? false,
    lastMessageTime: conversation.lastMessageTime,
  }), [])

  const dialogContact = useMemo(() => (
    selectedContact ? conversationToContact(selectedContact) : null
  ), [conversationToContact, selectedContact])

  // Entity menu state
  const [entityMenuAnchor, setEntityMenuAnchor] = useState<HTMLElement | null>(null)
  const [sidebarMenuState, setSidebarMenuState] = useState<{ anchorEl: HTMLElement | null; conversation: Conversation | null }>({
    anchorEl: null,
    conversation: null,
  })

  const conversations = useMemo<Conversation[]>(() => {
    const formatTimestamp = (value?: Date | string) => {
      if (!value) {
        return ''
      }

      const date = value instanceof Date ? value : new Date(value)
      if (Number.isNaN(date.getTime())) {
        return ''
      }

      const diffMs = Date.now() - date.getTime()
      const minuteMs = 60_000
      const hourMs = 60 * minuteMs
      const dayMs = 24 * hourMs

      if (diffMs < hourMs) {
        const minutes = Math.max(1, Math.round(diffMs / minuteMs))
        return `${minutes}m`
      }

      if (diffMs < dayMs) {
        return date.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' })
      }

      return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' })
    }

    const result: Conversation[] = []

    organizations.forEach((org) => {
      const orgName = org.name
      result.push({
        id: org.id,
        name: orgName,
        type: 'organisation',
        snippet: org.description ?? 'Organisation overview',
        time: formatTimestamp(org.updatedAt),
        status: 'read',
        pinned: true,
        org: orgName,
        orgId: org.id,
        membersOnline: org.users?.length ?? 0,
        hasWebsite: org.settings?.websitePublishingEnabled ?? false,
        scope: 'organization',
        fourWords: org.networkIdentity?.fourWords,
      })

      org.channels?.forEach((channel) => {
        result.push({
          id: channel.id,
          name: channel.name.startsWith('#') ? channel.name : `#${channel.name}`,
          type: 'channel',
          snippet: channel.topic ?? 'Channel updates',
          time: formatTimestamp(channel.updatedAt),
          status: 'read',
          org: orgName,
          orgId: org.id,
          channelMeta: {
            topic: channel.topic ?? 'General updates',
            members: channel.members?.length ?? 0,
          },
          scope: 'organization',
          fourWords: channel.networkIdentity?.fourWords,
        })
      })

      org.projects?.forEach((project) => {
        const ownerName = org.users?.find((user) => project.leads.includes(user.userId))?.name ?? 'Unassigned'
        const completedMilestones = project.milestones?.filter((milestone) => milestone.completed).length ?? 0
        const totalMilestones = project.milestones?.length ?? 0
        const completion = totalMilestones > 0 ? Math.round((completedMilestones / totalMilestones) * 100) : 0
        const statusLabel = project.status === 'planning'
          ? 'Planning'
          : project.status === 'active'
          ? 'Active'
          : project.status === 'completed'
          ? 'Completed'
          : 'Archived'

        result.push({
          id: project.id,
          name: project.name,
          type: 'project',
          snippet: `${statusLabel} · ${project.members.length} members`,
          time: formatTimestamp(project.updatedAt ?? project.createdAt),
          status: 'read',
          org: orgName,
          orgId: org.id,
          projectMeta: {
            status: statusLabel as NonNullable<Conversation['projectMeta']>['status'],
            completion,
            owner: ownerName,
          },
          scope: 'organization',
          fourWords: project.networkIdentity?.fourWords,
        })
      })

      org.groups?.forEach((group) => {
        result.push({
          id: group.id,
          name: group.name,
          type: 'group',
          snippet: `${group.members.length} members · ${group.admins.length} admins`,
          time: formatTimestamp(group.updatedAt ?? group.createdAt),
          status: 'read',
          org: orgName,
          orgId: org.id,
          membersOnline: group.members.length,
          scope: 'organization',
          fourWords: group.networkIdentity?.fourWords,
        })
      })
    })

    personalGroups.forEach((group) => {
      result.push({
        id: group.id,
        name: group.name,
        type: 'group',
        snippet: `${group.members.length} members`,
        time: formatTimestamp(group.updatedAt ?? group.createdAt),
        status: 'read',
        org: 'Personal',
        orgId: undefined,
        membersOnline: group.members.length,
        scope: 'personal',
        fourWords: group.networkIdentity?.fourWords,
      })
    })

    personalUsers.forEach((user) => {
      result.push({
        id: user.id,
        name: user.name,
        type: 'person',
        snippet: user.relationship ? `Relationship: ${user.relationship}` : 'Direct message',
        time: formatTimestamp(user.lastContact ?? user.updatedAt ?? user.createdAt),
        status: 'read',
        org: 'Personal',
        orgId: undefined,
        scope: 'personal',
        online: false,
        fourWords: user.networkIdentity?.fourWords,
      })
    })

    return result
  }, [organizations, personalGroups, personalUsers])

  const [selectedConversationId, setSelectedConversationId] = useState(() => conversations[0]?.id ?? '')

  // Check authentication status on mount and when auth state changes
  useEffect(() => {
    const checkAuthStatus = async () => {
      if (!authState.loading && !authState.isAuthenticated) {
        // Check if this is first launch (no vaults exist)
        const firstLaunch = await isFirstLaunch()
        if (firstLaunch) {
          console.log('🎉 First launch detected - showing welcome screen')
          setShowFirstLaunchWelcome(true)
        } else {
          console.log('📝 Existing identities found - showing identity picker')
          setShowIdentityPicker(true)
        }
      }
    }
    checkAuthStatus()
  }, [authState.isAuthenticated, authState.loading, isFirstLaunch])

  // Authentication handlers
  const handleSelectIdentity = useCallback(async (fourWords: string, usePasskey: boolean) => {
    try {
      setIsAuthenticating(true)
      setAuthError(null)

      if (usePasskey) {
        // Use passkey authentication
        const success = await signInWithPasskey(fourWords)
        if (success) {
          setShowIdentityPicker(false)
          enqueueSnackbar('Signed in successfully', { variant: 'success' })
        } else {
          setAuthError('Passkey authentication failed')
        }
      } else {
        // Show password dialog
        setSelectedIdentity(fourWords)
        setShowPasswordDialog(true)
      }
    } catch (err) {
      setAuthError(err instanceof Error ? err.message : 'Authentication failed')
      enqueueSnackbar('Authentication failed', { variant: 'error' })
    } finally {
      setIsAuthenticating(false)
    }
  }, [signInWithPasskey, enqueueSnackbar])

  const handlePasswordLogin = useCallback(async () => {
    try {
      setIsAuthenticating(true)
      setAuthError(null)

      const success = await login(selectedIdentity, password)
      if (success) {
        setShowPasswordDialog(false)
        setShowIdentityPicker(false)
        setPassword('')
        enqueueSnackbar('Signed in successfully', { variant: 'success' })
      } else {
        setAuthError('Invalid password')
      }
    } catch (err) {
      setAuthError(err instanceof Error ? err.message : 'Login failed')
      enqueueSnackbar('Login failed', { variant: 'error' })
    } finally {
      setIsAuthenticating(false)
    }
  }, [selectedIdentity, password, login, enqueueSnackbar])

  // Handle identity switching
  const handleSwitchIdentity = useCallback(async (fourWords: string) => {
    try {
      console.log('🔄 Switching to identity:', fourWords)
      setConnectionMenuAnchor(null)

      // Logout current identity
      await logout()
      console.log('✅ Logged out current identity')

      // Login with selected identity using passkey (auto-login)
      const success = await signInWithPasskey(fourWords)
      if (success) {
        enqueueSnackbar(`Switched to ${fourWords}`, { variant: 'success' })
        console.log('✅ Switched to new identity successfully')
      } else {
        // If passkey fails, show identity picker for manual login
        setShowIdentityPicker(true)
        enqueueSnackbar('Auto-login failed, please sign in manually', { variant: 'warning' })
      }
    } catch (error) {
      console.error('Failed to switch identity:', error)
      enqueueSnackbar('Failed to switch identity', { variant: 'error' })
      setShowIdentityPicker(true)
    }
  }, [logout, signInWithPasskey, enqueueSnackbar])

  // Handle creating new identity from switcher menu
  const handleCreateNewIdentity = useCallback(() => {
    setConnectionMenuAnchor(null)
    logout().then(() => {
      setShowFirstLaunchWelcome(true)
      console.log('🎉 Creating new identity')
    })
  }, [logout])

  useEffect(() => {
    if (conversations.length === 0) {
      return
    }

    const hasSelection = conversations.some((conversation) => conversation.id === selectedConversationId)
    if (!selectedConversationId || !hasSelection) {
      const fallback = conversations.find((conversation) => conversation.type !== 'organisation') ?? conversations[0]
      if (fallback) {
        setSelectedConversationId(fallback.id)
      }
    }
  }, [conversations, selectedConversationId])

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault()
        setIsCommandPaletteOpen(prev => !prev)
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  useEffect(() => {
    if (!isCommandPaletteOpen) {
      return
    }

    setCommandQuery('')
    const frame = window.requestAnimationFrame(() => {
      commandInputRef.current?.focus()
      commandInputRef.current?.select()
    })

    return () => window.cancelAnimationFrame(frame)
  }, [isCommandPaletteOpen])

  // CRDT Message State
  const [messages, setMessages] = useState<Message[]>([])
  const [messageInputValue, setMessageInputValue] = useState('')
  const [ourPeerId, setOurPeerId] = useState<string>('')
  const [ourDisplayName, setOurDisplayName] = useState<string>('')
  const [connectionMenuAnchor, setConnectionMenuAnchor] = useState<null | HTMLElement>(null)
  const [recentIdentities, setRecentIdentities] = useState<any[]>([])
  const [addConnectionDialogOpen, setAddConnectionDialogOpen] = useState(false)
  const [connectionWordsInput, setConnectionWordsInput] = useState('')
  const [editDisplayNameDialogOpen, setEditDisplayNameDialogOpen] = useState(false)
  const [displayNameInput, setDisplayNameInput] = useState('')
  const messageSyncService = useRef(getMessageSyncService())
  const syncIntervalRef = useRef<NodeJS.Timeout | null>(null)

  // Compute storage key after ourPeerId state is declared
  const preferenceStorageKey = `modern-shell-preferences-${ourPeerId || 'default'}`

  // Load recent identities when connection menu opens
  useEffect(() => {
    const loadRecentIdentities = async () => {
      if (connectionMenuAnchor && authState.isAuthenticated) {
        try {
          const identities = await getRecentIdentities()
          console.log('📋 Loaded recent identities:', identities)
          setRecentIdentities(identities)
        } catch (error) {
          console.error('Failed to load recent identities:', error)
        }
      }
    }
    loadRecentIdentities()
  }, [connectionMenuAnchor, authState.isAuthenticated, getRecentIdentities])

  useEffect(() => {
    if (preferencesLoaded) {
      return
    }

    if (typeof window === 'undefined') {
      return
    }

    try {
      const raw = window.localStorage.getItem(preferenceStorageKey)

      if (!raw) {
        if (
          organizations.length === 0 &&
          personalGroups.length === 0 &&
          personalUsers.length === 0
        ) {
          return
        }

        const defaultExpanded = new Set<string>()
        organizations.forEach((organization) => defaultExpanded.add(organization.name))
        if ((personalGroups.length > 0) || (personalUsers.length > 0)) {
          defaultExpanded.add('Personal Space')
        }
        if (defaultExpanded.size > 0) {
          setExpandedOrgs(defaultExpanded)
        }
      } else {
        const parsed = JSON.parse(raw) as {
          scopeFilter?: typeof scopeFilter
          typeFilter?: typeof typeFilter
          expandedOrgs?: string[]
        }

        const validScopeFilters: Array<typeof scopeFilter> = ['all', 'organization', 'personal']
        const validTypeFilters: Array<typeof typeFilter> = ['all', 'channels', 'projects', 'groups', 'people']

        if (parsed.scopeFilter && validScopeFilters.includes(parsed.scopeFilter)) {
          setScopeFilter(parsed.scopeFilter)
        }
        if (parsed.typeFilter && validTypeFilters.includes(parsed.typeFilter)) {
          setTypeFilter(parsed.typeFilter)
        }
        if (Array.isArray(parsed.expandedOrgs)) {
          setExpandedOrgs(new Set(parsed.expandedOrgs))
        }
      }
    } catch (error) {
      console.warn('Failed to load shell preferences', error)
    } finally {
      setPreferencesLoaded(true)
    }
  }, [
    preferencesLoaded,
    preferenceStorageKey,
    organizations,
    personalGroups,
    personalUsers,
  ])

  useEffect(() => {
    if (!preferencesLoaded || typeof window === 'undefined') {
      return
    }

    const payload = {
      scopeFilter,
      typeFilter,
      expandedOrgs: Array.from(expandedOrgs),
    }

    try {
      window.localStorage.setItem(preferenceStorageKey, JSON.stringify(payload))
    } catch (error) {
      console.warn('Failed to persist shell preferences', error)
    }
  }, [expandedOrgs, preferenceStorageKey, preferencesLoaded, scopeFilter, typeFilter])

  // Convert CRDT message to UI Message format
  const convertCRDTToUIMessage = useCallback((crdtMsg: CRDTMessage, ourPeerId: string): Message => {
    const isOurMessage = crdtMsg.metadata.authorPeerId === ourPeerId

    return {
      id: crdtMsg.metadata.id,
      author: crdtMsg.content.author,
      text: crdtMsg.content.text,
      time: new Date(crdtMsg.metadata.timestamp).toLocaleTimeString('en-US', {
        hour: '2-digit',
        minute: '2-digit'
      }),
      self: isOurMessage,
      status: crdtMsg.localState?.status?.toLowerCase() as 'sent' | 'delivered' | 'read' | undefined,
      threadCount: crdtMsg.localState?.threadCount,
      latestReplyBy: crdtMsg.localState?.latestReplyBy,
      reactions: crdtMsg.localState?.reactions?.map(r => ({
        emoji: r.emoji,
        count: r.count,
        userReacted: r.userReacted,
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
      let testPeerId = 'ocean-forest-moon-star' // default fallback
      let testDisplayName = '' // default empty

      // Check if running in Tauri - try to get user info from backend with retry
      if ((window as any).__TAURI__?.tauri?.invoke) {
        let retries = 5
        let userInfoRetrieved = false

        while (retries > 0 && !userInfoRetrieved) {
          try {
            const userInfo = await (window as any).__TAURI__.tauri.invoke('core_get_user_info') as { peerId: string; displayName: string }
            if (userInfo && userInfo.peerId) {
              testPeerId = userInfo.peerId
              testDisplayName = userInfo.displayName || ''
              console.log('✅ Got user info from Tauri backend:', { peerId: testPeerId, displayName: testDisplayName })
              userInfoRetrieved = true
            }
          } catch (err) {
            retries--
            if (retries > 0) {
              console.log(`⚠️  Core not initialized yet, retrying... (${retries} attempts left)`)
              await new Promise(resolve => setTimeout(resolve, 200))
            } else {
              console.log('⚠️  Core not initialized after retries, will use fallback values')
            }
          }
        }
      } else {
        // Browser mode - use URL param or localStorage
        const urlParams = new URLSearchParams(window.location.search)
        const peerIdFromUrl = urlParams.get('peerId')
        const peerIdFromStorage = localStorage.getItem('testPeerId')
        testPeerId = peerIdFromUrl || peerIdFromStorage || testPeerId
        testDisplayName = localStorage.getItem('testDisplayName') || ''
      }

      // Save to localStorage for persistence and set state
      localStorage.setItem('testPeerId', testPeerId)
      localStorage.setItem('testDisplayName', testDisplayName)
      setOurPeerId(testPeerId)
      setOurDisplayName(testDisplayName)

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

  // Connection menu handlers
  const handleCopyConnectionWords = useCallback(() => {
    if (ourPeerId) {
      navigator.clipboard.writeText(ourPeerId)
      console.log('✅ Copied connection words:', ourPeerId)
      setConnectionMenuAnchor(null)
    }
  }, [ourPeerId])

  const handleAddConnection = useCallback(async () => {
    if (!connectionWordsInput.trim()) {
      console.warn('⚠️  Connection words cannot be empty')
      return
    }

    if ((window as any).__TAURI__?.tauri?.invoke) {
      try {
        await (window as any).__TAURI__.tauri.invoke('core_add_bootstrap_node', {
          node: connectionWordsInput.trim()
        })
        console.log('✅ Added bootstrap node:', connectionWordsInput.trim())
        setConnectionWordsInput('')
        setAddConnectionDialogOpen(false)
      } catch (err) {
        console.error('❌ Failed to add bootstrap node:', err)
      }
    } else {
      console.log('📝 Would add connection:', connectionWordsInput.trim())
      setConnectionWordsInput('')
      setAddConnectionDialogOpen(false)
    }
  }, [connectionWordsInput])

  const handleSaveDisplayName = useCallback(async () => {
    if (!displayNameInput.trim()) {
      console.warn('⚠️  Display name cannot be empty')
      return
    }

    if ((window as any).__TAURI__?.tauri?.invoke) {
      try {
        await (window as any).__TAURI__.tauri.invoke('core_set_display_name', {
          displayName: displayNameInput.trim()
        })
        console.log('✅ Display name updated:', displayNameInput.trim())
        setOurDisplayName(displayNameInput.trim())
        localStorage.setItem('testDisplayName', displayNameInput.trim())
        setDisplayNameInput('')
        setEditDisplayNameDialogOpen(false)
      } catch (err) {
        console.error('❌ Failed to update display name:', err)
      }
    } else {
      console.log('📝 Would update display name:', displayNameInput.trim())
      setOurDisplayName(displayNameInput.trim())
      localStorage.setItem('testDisplayName', displayNameInput.trim())
      setDisplayNameInput('')
      setEditDisplayNameDialogOpen(false)
    }
  }, [displayNameInput])

  const defaultConversationId = useMemo(
    () => conversations.find(c => c.type !== 'organisation')?.id ?? conversations[0]?.id ?? '',
    [conversations]
  )

  const selectedConversation = useMemo(
    () => conversations.find(c => c.id === selectedConversationId) ?? conversations[0],
    [conversations, selectedConversationId]
  )

  const activeOrg = useMemo(() => {
    if (!selectedConversation) {
      return organizations[0] ?? null
    }

    if (selectedConversation.type === 'organisation') {
      return organizations.find(org => org.id === selectedConversation.id) ?? null
    }

    if (selectedConversation.orgId) {
      return organizations.find(org => org.id === selectedConversation.orgId) ?? null
    }

    return organizations[0] ?? null
  }, [organizations, selectedConversation])

  const isOrganisationView = selectedConversation?.type === 'organisation'
  const isGroupView = selectedConversation?.type === 'group'
  const isPersonView = selectedConversation?.type === 'person'
  const isChannelView = selectedConversation?.type === 'channel'
  const isProjectView = selectedConversation?.type === 'project'
  const isGroupConversation = selectedConversation?.type !== 'person'

  const focusConversationById = useCallback((conversationId: string, orgNameToExpand?: string) => {
    setSelectedConversationId(conversationId)
    if (orgNameToExpand) {
      setExpandedOrgs(prev => {
        const next = new Set(prev)
        next.add(orgNameToExpand)
        return next
      })
    }
  }, [])

  const closeCommandPalette = useCallback(() => {
    setIsCommandPaletteOpen(false)
    setCommandQuery('')
  }, [])

  const runCreateOrganizationCommand = useCallback(async (nameHint?: string) => {
    const displayName = nameHint?.trim() || 'New Organization'
    try {
      const result = await createOrganization({ displayName, description: undefined })
      if (!result.success) {
        enqueueSnackbar(result.error ?? 'Failed to create organisation', { variant: 'error' })
        return
      }
      enqueueSnackbar(`Created organisation “${displayName}”`, { variant: 'success' })
      focusConversationById(result.entityId, displayName)
      closeCommandPalette()
    } catch (error) {
      console.error('Create organisation failed', error)
      enqueueSnackbar('Failed to create organisation', { variant: 'error' })
    }
  }, [closeCommandPalette, createOrganization, enqueueSnackbar, focusConversationById])

  const runCreateChannelCommand = useCallback(async (nameHint?: string) => {
    const org = activeOrg
    if (!org) {
      enqueueSnackbar('Select an organisation to create a channel.', { variant: 'warning' })
      return
    }

    const displayName = nameHint?.trim() || 'new-channel'

    try {
      const result = await createChannel({
        organizationId: org.id,
        displayName,
        description: undefined,
        isPrivate: false,
      })

      if (!result.success) {
        enqueueSnackbar(result.error ?? 'Failed to create channel', { variant: 'error' })
        return
      }

      enqueueSnackbar(`Created #${displayName}`, { variant: 'success' })
      focusConversationById(result.entityId, org.name)
      closeCommandPalette()
    } catch (error) {
      console.error('Create channel failed', error)
      enqueueSnackbar('Failed to create channel', { variant: 'error' })
    }
  }, [activeOrg, closeCommandPalette, createChannel, enqueueSnackbar, focusConversationById])

  const runCreateProjectCommand = useCallback(async (nameHint?: string) => {
    const org = activeOrg
    if (!org) {
      enqueueSnackbar('Select an organisation to create a project.', { variant: 'warning' })
      return
    }

    const displayName = nameHint?.trim() || 'New Project'

    try {
      const result = await createProject({
        organizationId: org.id,
        displayName,
        description: undefined,
      })

      if (!result.success) {
        enqueueSnackbar(result.error ?? 'Failed to create project', { variant: 'error' })
        return
      }

      enqueueSnackbar(`Created project “${displayName}”`, { variant: 'success' })
      focusConversationById(result.entityId, org.name)
      closeCommandPalette()
    } catch (error) {
      console.error('Create project failed', error)
      enqueueSnackbar('Failed to create project', { variant: 'error' })
    }
  }, [activeOrg, closeCommandPalette, createProject, enqueueSnackbar, focusConversationById])

  const runCreateGroupCommand = useCallback(async (nameHint?: string, scope: 'organization' | 'personal' = 'organization') => {
    if (scope === 'organization' && !activeOrg) {
      enqueueSnackbar('Select an organisation to create a group.', { variant: 'warning' })
      return
    }

    const displayName = nameHint?.trim() || (scope === 'organization' ? 'New Team Group' : 'New Personal Group')

    try {
      const result = await createGroup({
        displayName,
        description: undefined,
        organizationId: scope === 'organization' ? activeOrg?.id : undefined,
      })

      if (!result.success) {
        enqueueSnackbar(result.error ?? 'Failed to create group', { variant: 'error' })
        return
      }

      const orgName = scope === 'organization' ? activeOrg?.name : 'Personal Space'
      enqueueSnackbar(`Created group “${displayName}”`, { variant: 'success' })
      focusConversationById(result.entityId, orgName)
      closeCommandPalette()
    } catch (error) {
      console.error('Create group failed', error)
      enqueueSnackbar('Failed to create group', { variant: 'error' })
    }
  }, [activeOrg, closeCommandPalette, createGroup, enqueueSnackbar, focusConversationById])

  const runCreateContactCommand = useCallback(async (nameHint?: string) => {
    const displayName = nameHint?.trim() || 'New Contact'

    try {
      const result = await createContact({
        displayName,
        relationship: 'colleague',
      })

      if (!result.success) {
        enqueueSnackbar(result.error ?? 'Failed to create contact', { variant: 'error' })
        return
      }

      enqueueSnackbar(`Created contact “${displayName}”`, { variant: 'success' })
      focusConversationById(result.entityId, 'Personal Space')
      closeCommandPalette()
    } catch (error) {
      console.error('Create contact failed', error)
      enqueueSnackbar('Failed to create contact', { variant: 'error' })
    }
  }, [closeCommandPalette, createContact, enqueueSnackbar, focusConversationById])

  const focusFirstScopeChip = useCallback(() => {
    scopeChipRefs.current.find(Boolean)?.focus()
  }, [])

  const focusLastScopeChip = useCallback(() => {
    const chips = scopeChipRefs.current.filter(Boolean)
    chips[chips.length - 1]?.focus()
  }, [])

  const focusFirstTypeChip = useCallback(() => {
    typeChipRefs.current.find(Boolean)?.focus()
  }, [])

  const focusLastTypeChip = useCallback(() => {
    const chips = typeChipRefs.current.filter(Boolean)
    chips[chips.length - 1]?.focus()
  }, [])

  const handleSearchKeyDown = useCallback((event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'ArrowDown') {
      event.preventDefault()
      if (scopeChipRefs.current.length > 0) {
        focusFirstScopeChip()
      } else if (typeChipRefs.current.length > 0) {
        focusFirstTypeChip()
      }
    } else if (event.key === 'ArrowUp') {
      event.preventDefault()
      if (typeChipRefs.current.length > 0) {
        focusLastTypeChip()
      } else if (scopeChipRefs.current.length > 0) {
        focusLastScopeChip()
      }
    }
  }, [focusFirstScopeChip, focusFirstTypeChip, focusLastScopeChip, focusLastTypeChip])

  const handleScopeChipKeyDown = useCallback((event: React.KeyboardEvent<HTMLDivElement>, index: number) => {
    if (event.key === 'ArrowRight') {
      event.preventDefault()
      const next = scopeChipRefs.current[index + 1]
      if (next) {
        next.focus()
      } else if (typeChipRefs.current.length > 0) {
        focusFirstTypeChip()
      } else {
        searchInputRef.current?.focus()
      }
    } else if (event.key === 'ArrowLeft') {
      event.preventDefault()
      const prev = scopeChipRefs.current[index - 1]
      if (prev) {
        prev.focus()
      } else {
        searchInputRef.current?.focus()
      }
    } else if (event.key === 'ArrowDown') {
      event.preventDefault()
      if (typeChipRefs.current.length > 0) {
        focusFirstTypeChip()
      } else {
        searchInputRef.current?.focus()
      }
    } else if (event.key === 'ArrowUp') {
      event.preventDefault()
      searchInputRef.current?.focus()
    }
  }, [focusFirstTypeChip])

  const handleTypeChipKeyDown = useCallback((event: React.KeyboardEvent<HTMLDivElement>, index: number) => {
    if (event.key === 'ArrowRight') {
      event.preventDefault()
      const next = typeChipRefs.current[index + 1]
      if (next) {
        next.focus()
      } else {
        searchInputRef.current?.focus()
      }
    } else if (event.key === 'ArrowLeft') {
      event.preventDefault()
      const prev = typeChipRefs.current[index - 1]
      if (prev) {
        prev.focus()
      } else {
        focusLastScopeChip()
      }
    } else if (event.key === 'ArrowUp') {
      event.preventDefault()
      if (scopeChipRefs.current.length > 0) {
        focusLastScopeChip()
      } else {
        searchInputRef.current?.focus()
      }
    } else if (event.key === 'ArrowDown') {
      event.preventDefault()
      searchInputRef.current?.focus()
    }
  }, [focusLastScopeChip])

  useEffect(() => {
    setChannelViewMode('chat')
    setProjectViewMode('chat')
    setGroupViewMode('chat')
    setPersonViewMode('chat')
  }, [selectedConversationId])

  const headerSubtitle = useMemo(() => {
    if (!selectedConversation) return ''

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

  // Toggle organization expansion
  const toggleOrgExpansion = (orgName: string) => {
    setExpandedOrgs(prev => {
      const next = new Set(prev)
      if (next.has(orgName)) {
        next.delete(orgName)
      } else {
        next.add(orgName)
      }
      return next
    })
  }

  // Group conversations by organization
  const organizedConversations = useMemo(() => {
    // Get all orgs
    const orgs = conversations.filter(c => c.type === 'organisation')

    // Get standalone items (not belonging to any org)
    const personalItems = conversations.filter(c => c.scope === 'personal')

    const standalone = conversations.filter(c =>
      c.type !== 'organisation' &&
      c.type !== 'storage' &&
      c.scope !== 'personal' &&
      !orgs.some(org => c.org === org.name.replace(' (Org)', ''))
    )

    // Build hierarchical structure
    const result: Array<Conversation & { children?: Conversation[] }> = []

    orgs.forEach(org => {
      const orgName = org.name.replace(' (Org)', '')
      const children = conversations.filter(c =>
        c.org === orgName &&
        c.id !== org.id &&
        c.type !== 'storage' // Exclude storage from sidebar
      )
      result.push({ ...org, children })
    })

    if (personalItems.length > 0) {
      result.push({
        id: 'personal-space',
        name: 'Personal Space',
        type: 'organisation',
        snippet: 'Direct messages and personal groups',
        time: '',
        status: 'read',
        org: 'Personal Space',
        scope: 'personal',
        children: personalItems,
      })
    }

    // Add standalone items
    result.push(...standalone)

    return result
  }, [conversations])

  const filteredConversations = useMemo(() => {
    const matchesScope = (conversation: Conversation) => {
      if (scopeFilter === 'all') {
        return true
      }
      if (scopeFilter === 'organization') {
        return conversation.scope !== 'personal'
      }
      return conversation.scope === 'personal'
    }

    const matchesType = (conversation: Conversation) => {
      if (typeFilter === 'all') {
        return true
      }

      switch (typeFilter) {
        case 'channels':
          return conversation.type === 'channel'
        case 'projects':
          return conversation.type === 'project'
        case 'groups':
          return conversation.type === 'group'
        case 'people':
          return conversation.type === 'person'
        default:
          return true
      }
    }

    return organizedConversations.reduce<Array<Conversation & { children?: Conversation[] }>>((acc, conversation) => {
      if (conversation.type === 'organisation') {
        const conversationScope = conversation.scope === 'personal' ? 'personal' : 'organization'
        if (scopeFilter !== 'all' && scopeFilter !== conversationScope) {
          return acc
        }

        if (!conversation.children) {
          if (typeFilter === 'all') {
            acc.push(conversation)
          }
          return acc
        }

        const scopedChildren = conversation.children.filter(child => matchesScope(child) && matchesType(child))

        if (typeFilter === 'all') {
          acc.push({ ...conversation, children: scopedChildren })
        } else if (scopedChildren.length > 0) {
          acc.push({ ...conversation, children: scopedChildren })
        }

        return acc
      }

      if (!matchesScope(conversation) || !matchesType(conversation)) {
        return acc
      }

      acc.push(conversation)
      return acc
    }, [])
  }, [organizedConversations, scopeFilter, typeFilter])

  const normalizedCommandQuery = commandQuery.trim().toLowerCase()

  const entityCommandItems = useMemo<CommandPaletteEntityItem[]>(() => {
    const matches = normalizedCommandQuery
      ? conversations.filter(conversation => {
          const haystack = [
            conversation.name,
            conversation.snippet,
            conversation.org ?? '',
            conversation.scope === 'organization' ? 'organisation' : conversation.scope ?? '',
            conversation.type,
          ]
            .join(' ')
            .toLowerCase()
          return haystack.includes(normalizedCommandQuery)
        })
      : conversations

    const unique = new Map<string, Conversation>()
    matches.forEach(match => {
      if (!unique.has(match.id)) {
        unique.set(match.id, match)
      }
    })

    return Array.from(unique.values())
      .slice(0, 10)
      .map<CommandPaletteEntityItem>(conversation => ({
        id: `entity-${conversation.id}`,
        type: 'entity',
        conversation,
        label: conversation.name,
        subtitle: conversation.org ? `${conversation.org} · ${conversation.type}` : conversation.type,
      }))
  }, [conversations, normalizedCommandQuery])

  const quickCommandItems = useMemo<CommandPaletteActionItem[]>(() => {
    const trimmedQuery = commandQuery.trim()
    const normalized = trimmedQuery.toLowerCase()
    const matchesQuery = (label: string, subtitle?: string) => {
      if (!normalized) return true
      const haystack = `${label} ${subtitle ?? ''}`.toLowerCase()
      return haystack.includes(normalized)
    }

    const actions: CommandPaletteActionItem[] = []

    const orgNameSuggestion = trimmedQuery || undefined

    const orgActionLabel = trimmedQuery
      ? `Create organisation “${trimmedQuery}”`
      : 'Create new organisation'
    const orgActionSubtitle = 'Generate a workspace with starter channels and projects'
    if (matchesQuery(orgActionLabel, orgActionSubtitle)) {
      actions.push({
        id: 'action-create-organisation',
        type: 'action',
        label: orgActionLabel,
        subtitle: orgActionSubtitle,
        run: () => runCreateOrganizationCommand(orgNameSuggestion),
      })
    }

    if (activeOrg) {
      const channelLabel = trimmedQuery
        ? `Create channel “${trimmedQuery}” in ${activeOrg.name}`
        : `Create channel in ${activeOrg.name}`
      const channelSubtitle = '#channel · Visible to everyone in the organisation'
      if (matchesQuery(channelLabel, channelSubtitle)) {
        actions.push({
          id: 'action-create-channel',
          type: 'action',
          label: channelLabel,
          subtitle: channelSubtitle,
          run: () => runCreateChannelCommand(trimmedQuery || undefined),
        })
      }

      const projectLabel = trimmedQuery
        ? `Create project “${trimmedQuery}” in ${activeOrg.name}`
        : `Create project in ${activeOrg.name}`
      const projectSubtitle = 'Track milestones, owners, and progress'
      if (matchesQuery(projectLabel, projectSubtitle)) {
        actions.push({
          id: 'action-create-project',
          type: 'action',
          label: projectLabel,
          subtitle: projectSubtitle,
          run: () => runCreateProjectCommand(trimmedQuery || undefined),
        })
      }

      const groupLabel = trimmedQuery
        ? `Create group “${trimmedQuery}” in ${activeOrg.name}`
        : `Create group in ${activeOrg.name}`
      const groupSubtitle = 'Smaller teams with dedicated permissions'
      if (matchesQuery(groupLabel, groupSubtitle)) {
        actions.push({
          id: 'action-create-org-group',
          type: 'action',
          label: groupLabel,
          subtitle: groupSubtitle,
          run: () => runCreateGroupCommand(trimmedQuery || undefined, 'organization'),
        })
      }
    }

    const personalGroupLabel = trimmedQuery
      ? `Create personal group “${trimmedQuery}”`
      : 'Create personal group'
    const personalGroupSubtitle = 'Coordinate with friends or family in your private space'
    if (matchesQuery(personalGroupLabel, personalGroupSubtitle)) {
      actions.push({
        id: 'action-create-personal-group',
        type: 'action',
        label: personalGroupLabel,
        subtitle: personalGroupSubtitle,
        run: () => runCreateGroupCommand(trimmedQuery || undefined, 'personal'),
      })
    }

    const contactLabel = trimmedQuery ? `Add contact “${trimmedQuery}”` : 'Add contact'
    const contactSubtitle = 'Start a private thread with a new contact'
    if (matchesQuery(contactLabel, contactSubtitle)) {
      actions.push({
        id: 'action-create-contact',
        type: 'action',
        label: contactLabel,
        subtitle: contactSubtitle,
        run: () => runCreateContactCommand(trimmedQuery || undefined),
      })
    }

    return actions
  }, [activeOrg, commandQuery, runCreateChannelCommand, runCreateContactCommand, runCreateGroupCommand, runCreateOrganizationCommand, runCreateProjectCommand])

  const commandItems = useMemo<CommandPaletteItem[]>(() => {
    const combined = [...quickCommandItems, ...entityCommandItems]
    return combined.slice(0, 12)
  }, [entityCommandItems, quickCommandItems])

  const handleMessageMenuOpen = (event: React.MouseEvent<HTMLElement>, message: Message) => {
    setMessageMenu({ anchorEl: event.currentTarget, message })
  }

  const handleMessageMenuClose = () => setMessageMenu({ anchorEl: null })

  const handleHome = () => {
    if (defaultConversationId) {
      setSelectedConversationId(defaultConversationId)
      setChannelViewMode('chat')
      setProjectViewMode('chat')
      setGroupViewMode('chat')
      setPersonViewMode('chat')
    }
  }

  const handleWebsiteOpen = () => {
    if (!selectedConversation) return

    // Open website storage page for the selected entity
    // This will connect to saorsa-sites using the entity's four-word identity
    const fourWords = selectedConversation.fourWords || selectedConversation.id
    console.log(`🌐 Opening website for ${selectedConversation.name}`)
    console.log(`   Entity: ${selectedConversation.type}`)
    console.log(`   Four Words: ${fourWords}`)
    console.log(`   ID: ${selectedConversation.id}`)
    // TODO: Navigate to website storage page or open saorsa-sites interface
    // This would typically call: navigate(`/website/${fourWords}`) or invoke Tauri command
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

  const handleCommandItemSelect = useCallback(async (item: CommandPaletteItem) => {
    if (item.type === 'entity') {
      const orgName = item.conversation.scope === 'personal'
        ? 'Personal Space'
        : item.conversation.type === 'organisation'
          ? item.conversation.name
          : item.conversation.org
      focusConversationById(item.conversation.id, orgName ?? undefined)
      closeCommandPalette()
      return
    }

    try {
      await item.run()
    } catch (error) {
      console.error('Command action failed', error)
      enqueueSnackbar('Command failed', { variant: 'error' })
    }
  }, [closeCommandPalette, enqueueSnackbar, focusConversationById])

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
  const renderOrganisationOverview = () => {
    // Get first organization (or show empty state)
    const selectedOrg = organizations[0];

    // Show empty state if no organizations
    if (!selectedOrg) {
      return (
        <Box
          sx={{
            flexGrow: 1,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            px: 4,
            py: 8,
            textAlign: 'center',
          }}
        >
          <Typography variant="h5" fontWeight={600} gutterBottom>
            Welcome! 👋
          </Typography>
          <Typography variant="body1" color="text.secondary" paragraph sx={{ maxWidth: 600 }}>
            You don't have any organizations yet. Organizations are optional - you can start messaging contacts directly, or create an organization to collaborate with teams.
          </Typography>
          <Box sx={{ display: 'flex', gap: 2, mt: 3, flexWrap: 'wrap', justifyContent: 'center' }}>
            <Button
              variant="contained"
              startIcon={<AddCircleOutline />}
              onClick={() => setShowOrgManagementDialog(true)}
            >
              Create Organization
            </Button>
            <Button
              variant="outlined"
              startIcon={<AddCircleOutline />}
              onClick={() => setShowGroupManagementDialog(true)}
            >
              Create Group
            </Button>
            <Button
              variant="outlined"
              startIcon={<AddCircleOutline />}
              onClick={() => setContactDialogMode('add')}
            >
              Add Contact
            </Button>
          </Box>
        </Box>
      );
    }

    return (
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
          {(!selectedOrg?.users || selectedOrg.users.length === 0) ? (
            <Typography variant="body2" color="text.secondary" sx={{ py: 2 }}>No members yet</Typography>
          ) : selectedOrg.users.map(user => (
            <Stack key={user.userId} direction="row" spacing={1.5} alignItems="center" sx={{
              p: 1,
              borderRadius: 2,
              bgcolor: alpha('#FFFFFF', 0.02),
              '&:hover': { bgcolor: alpha(TOKENS.accent, 0.12) },
            }}>
              <Avatar sx={{ width: 32, height: 32, bgcolor: alpha(TOKENS.accent, 0.12) }}>
                {user.name.split(' ').map((n: string) => n[0]).join('')}
              </Avatar>
              <Box sx={{ flexGrow: 1 }}>
                <Typography variant="body2" fontWeight={500}>{user.name}</Typography>
                <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>
                  {user.role || 'member'}
                </Typography>
              </Box>
              <Tooltip title="Edit role">
                <IconButton size="small" sx={{ color: TOKENS.textSecondary }} onClick={() => console.log('Edit member:', user.name)}>
                  <EditOutlined fontSize="inherit" />
                </IconButton>
              </Tooltip>
              <Tooltip title="Message privately">
                <IconButton size="small" sx={{ color: TOKENS.textSecondary }}>
                  <ReplyOutlined fontSize="inherit" />
                </IconButton>
              </Tooltip>
              <Tooltip title="Remove from org">
                <IconButton size="small" sx={{ color: TOKENS.danger }} onClick={() => console.log('Remove member:', user.name)}>
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
          {(!selectedOrg?.projects || selectedOrg.projects.length === 0) ? (
            <Typography variant="body2" color="text.secondary" sx={{ py: 2 }}>No projects yet</Typography>
          ) : selectedOrg.projects.map(project => (
            <Box
              key={project.id}
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
              <Typography variant="body2" fontWeight={500} sx={{ flexGrow: 1 }}>{project.name}</Typography>
              <Tooltip title="Archive project">
                <IconButton size="small" sx={{ color: TOKENS.textSecondary }} onClick={() => console.log('Archive project:', project.name)}>
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
          {(!selectedOrg?.channels || selectedOrg.channels.length === 0) ? (
            <Typography variant="body2" color="text.secondary" sx={{ py: 2 }}>No channels yet</Typography>
          ) : selectedOrg.channels.map(channel => (
            <Box
              key={channel.id}
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
              <Typography variant="body2" fontWeight={500}>{channel.name}</Typography>
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
  }

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

  const renderGroupMode = () => {
    // Only show non-chat modes when group is selected
    if (!isGroupView || groupViewMode === 'chat') {
      return null
    }

    return (
      <Box sx={{ flexGrow: 1, px: 4, py: 4, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: 2 }}>
        {groupViewMode === 'threads' && (
          <Typography variant="body2" sx={{ color: TOKENS.textSecondary }}>
            Threads view for groups - Coming soon
          </Typography>
        )}
        {groupViewMode === 'files' && selectedConversation && (
          <EntityDocumentWorkspace
            entityId={selectedConversationId}
            entityName={selectedConversation.name}
            storageMode="files"
            permissions={['read', 'write']}
          />
        )}
        {groupViewMode === 'website' && (
          <Typography variant="body2" sx={{ color: TOKENS.textSecondary }}>
            Website management for groups - Coming soon
          </Typography>
        )}
      </Box>
    )
  }

  const renderPersonMode = () => {
    // Only show non-chat modes when person is selected
    if (!isPersonView || personViewMode === 'chat') {
      return null
    }

    return (
      <Box sx={{ flexGrow: 1, px: 4, py: 4, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: 2 }}>
        {personViewMode === 'threads' && (
          <Typography variant="body2" sx={{ color: TOKENS.textSecondary }}>
            Threads view for people - Coming soon
          </Typography>
        )}
        {personViewMode === 'files' && selectedConversation && (
          <EntityDocumentWorkspace
            entityId={selectedConversationId}
            entityName={selectedConversation.name}
            storageMode="files"
            permissions={['read', 'write']}
          />
        )}
        {personViewMode === 'website' && (
          <Typography variant="body2" sx={{ color: TOKENS.textSecondary }}>
            Website management for people - Coming soon
          </Typography>
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
  // Show first launch welcome if this is the first time
  if (showFirstLaunchWelcome && !authState.isAuthenticated) {
    return (
      <FirstLaunchWelcome
        open={showFirstLaunchWelcome}
        onClose={() => {
          setShowFirstLaunchWelcome(false)
          // After first launch setup, user is auto-logged in
          console.log('🎉 First launch complete!')
        }}
      />
    )
  }

  // Show authentication UI if not authenticated
  if (showIdentityPicker && !authState.isAuthenticated) {
    return (
      <>
        <IdentityPicker
          onSelectIdentity={handleSelectIdentity}
          onCreateNew={() => {
            setShowIdentityPicker(false)
            setShowUnifiedAuthFlow(true)
          }}
          onManualEntry={(fourWords) => {
            // Handle manual entry - prompt for password
            setSelectedIdentity(fourWords)
            setShowPasswordDialog(true)
          }}
        />

        {/* Password Dialog for non-passkey auth */}
        <Dialog
          open={showPasswordDialog}
          onClose={() => !isAuthenticating && setShowPasswordDialog(false)}
          maxWidth="xs"
          fullWidth
        >
          <DialogTitle>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <LockIcon color="primary" />
              <Typography variant="h6" fontWeight={600}>
                Enter Password
              </Typography>
            </Box>
          </DialogTitle>
          <DialogContent>
            <Stack spacing={2} sx={{ mt: 1 }}>
              {authError && (
                <Alert severity="error" onClose={() => setAuthError(null)}>
                  {authError}
                </Alert>
              )}
              <Typography variant="body2" color="text.secondary">
                Enter your password for <strong>{selectedIdentity}</strong>
              </Typography>
              <TextField
                fullWidth
                label="Password"
                type={showPassword ? 'text' : 'password'}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                onKeyPress={(e) => {
                  if (e.key === 'Enter' && password) {
                    handlePasswordLogin()
                  }
                }}
                disabled={isAuthenticating}
                autoFocus
                InputProps={{
                  startAdornment: (
                    <InputAdornment position="start">
                      <LockIcon fontSize="small" />
                    </InputAdornment>
                  ),
                  endAdornment: (
                    <InputAdornment position="end">
                      <IconButton
                        onClick={() => setShowPassword(!showPassword)}
                        edge="end"
                        size="small"
                      >
                        {showPassword ? <VisibilityOffIcon /> : <VisibilityIcon />}
                      </IconButton>
                    </InputAdornment>
                  ),
                }}
              />
            </Stack>
          </DialogContent>
          <DialogActions sx={{ px: 3, pb: 3 }}>
            <Button
              onClick={() => {
                setShowPasswordDialog(false)
                setPassword('')
                setAuthError(null)
              }}
              disabled={isAuthenticating}
            >
              Cancel
            </Button>
            <Button
              variant="contained"
              onClick={handlePasswordLogin}
              disabled={!password || isAuthenticating}
            >
              {isAuthenticating ? 'Signing in...' : 'Sign In'}
            </Button>
          </DialogActions>
        </Dialog>
      </>
    )
  }

  // Show UnifiedAuthFlow for creating new identity
  if (showUnifiedAuthFlow) {
    return (
      <UnifiedAuthFlow
        initialMode="register"
        onSuccess={() => {
          setShowUnifiedAuthFlow(false)
          enqueueSnackbar('Identity created successfully', { variant: 'success' })
        }}
        onCancel={() => {
          setShowUnifiedAuthFlow(false)
          setShowIdentityPicker(true)
        }}
      />
    )
  }

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
          {selectedConversation?.hasWebsite && (
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
          {/* Current User Indicator */}
          {ourPeerId && (
            <>
              <Box
                onClick={(e) => setConnectionMenuAnchor(e.currentTarget)}
                sx={{
                  mt: 1.5,
                  p: 1.5,
                  bgcolor: alpha(TOKENS.accent, 0.1),
                  borderRadius: 2,
                  border: `1px solid ${alpha(TOKENS.accent, 0.3)}`,
                  cursor: 'pointer',
                  '&:hover': {
                    bgcolor: alpha(TOKENS.accent, 0.15),
                  }
                }}
              >
                <Stack direction="row" spacing={1} alignItems="center">
                  <Avatar
                    sx={{
                      width: 32,
                      height: 32,
                      bgcolor: TOKENS.accent,
                      fontSize: 14,
                      fontWeight: 600
                    }}
                  >
                    {ourDisplayName
                      ? ourDisplayName.substring(0, 2).toUpperCase()
                      : ourPeerId.split('-')[0].substring(0, 2).toUpperCase()
                    }
                  </Avatar>
                  <Box sx={{ flexGrow: 1, minWidth: 0 }}>
                    <Typography
                      variant="body2"
                      fontWeight={600}
                      sx={{ color: TOKENS.textPrimary }}
                    >
                      {ourDisplayName || ourPeerId.split('-').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
                    </Typography>
                    <Typography
                      variant="caption"
                      sx={{
                        color: TOKENS.textSecondary,
                        fontSize: 11,
                        fontFamily: 'monospace'
                      }}
                    >
                      {ourPeerId}
                    </Typography>
                  </Box>
                  <KeyboardArrowDown sx={{ color: TOKENS.textSecondary, fontSize: 18 }} />
                </Stack>
              </Box>

              {/* Connection Menu */}
              <Menu
                anchorEl={connectionMenuAnchor}
                open={Boolean(connectionMenuAnchor)}
                onClose={() => setConnectionMenuAnchor(null)}
                PaperProps={{
                  sx: {
                    bgcolor: TOKENS.bgRaised,
                    border: `1px solid ${TOKENS.borderSubtle}`,
                    borderRadius: 2,
                    minWidth: 280,
                  }
                }}
              >
                <Box sx={{ px: 2, py: 1.5, borderBottom: `1px solid ${TOKENS.borderSubtle}` }}>
                  <Typography variant="caption" sx={{ color: TOKENS.textSecondary, textTransform: 'uppercase', fontWeight: 600 }}>
                    Your Connection Words
                  </Typography>
                  <Box sx={{ mt: 1, p: 1.5, bgcolor: alpha(TOKENS.accent, 0.05), borderRadius: 1, border: `1px solid ${alpha(TOKENS.accent, 0.2)}` }}>
                    <Typography variant="body2" sx={{ color: TOKENS.textPrimary, fontFamily: 'monospace', fontSize: 13, wordBreak: 'break-all' }}>
                      {ourPeerId}
                    </Typography>
                  </Box>
                </Box>

                {/* Recent Identities Section */}
                {recentIdentities.length > 1 && (
                  <>
                    <Box sx={{ px: 2, py: 1.5, borderBottom: `1px solid ${TOKENS.borderSubtle}` }}>
                      <Typography variant="caption" sx={{ color: TOKENS.textSecondary, textTransform: 'uppercase', fontWeight: 600, mb: 1, display: 'block' }}>
                        Switch Identity
                      </Typography>
                      <Stack spacing={0.5}>
                        {recentIdentities
                          .filter(identity => identity.four_words !== ourPeerId)
                          .slice(0, 4)
                          .map((identity) => (
                            <Box
                              key={identity.four_words}
                              onClick={() => handleSwitchIdentity(identity.four_words)}
                              sx={{
                                p: 1,
                                borderRadius: 1,
                                cursor: 'pointer',
                                bgcolor: alpha(TOKENS.textSecondary, 0.03),
                                border: `1px solid ${TOKENS.borderSubtle}`,
                                '&:hover': {
                                  bgcolor: alpha(TOKENS.accent, 0.08),
                                  borderColor: alpha(TOKENS.accent, 0.3),
                                }
                              }}
                            >
                              <Stack direction="row" spacing={1} alignItems="center">
                                <Avatar sx={{ width: 24, height: 24, fontSize: 11, bgcolor: alpha(TOKENS.accent, 0.2) }}>
                                  {identity.display_name.substring(0, 2).toUpperCase()}
                                </Avatar>
                                <Box sx={{ flexGrow: 1, minWidth: 0 }}>
                                  <Typography variant="body2" sx={{ color: TOKENS.textPrimary, fontSize: 13, fontWeight: 500 }}>
                                    {identity.display_name}
                                  </Typography>
                                  <Typography variant="caption" sx={{ color: TOKENS.textSecondary, fontSize: 10, fontFamily: 'monospace' }}>
                                    {identity.four_words}
                                  </Typography>
                                </Box>
                                {identity.has_passkey && (
                                  <CheckCircle sx={{ fontSize: 14, color: TOKENS.accent }} />
                                )}
                              </Stack>
                            </Box>
                          ))}
                      </Stack>
                    </Box>
                    <MenuItem onClick={handleCreateNewIdentity} sx={{ color: TOKENS.textPrimary, '&:hover': { bgcolor: TOKENS.surfaceActive } }}>
                      <ListItemIcon>
                        <PersonOutline fontSize="small" sx={{ color: TOKENS.accent }} />
                      </ListItemIcon>
                      <ListItemText>Create New Identity</ListItemText>
                    </MenuItem>
                    <Divider sx={{ my: 0.5, borderColor: TOKENS.borderSubtle }} />
                  </>
                )}

                <MenuItem onClick={handleCopyConnectionWords} sx={{ color: TOKENS.textPrimary, '&:hover': { bgcolor: TOKENS.surfaceActive } }}>
                  <ListItemIcon>
                    <ContentCopyOutlined fontSize="small" sx={{ color: TOKENS.textSecondary }} />
                  </ListItemIcon>
                  <ListItemText>Copy Connection Words</ListItemText>
                </MenuItem>
                <MenuItem onClick={() => { setConnectionMenuAnchor(null); setDisplayNameInput(ourDisplayName); setEditDisplayNameDialogOpen(true); }} sx={{ color: TOKENS.textPrimary, '&:hover': { bgcolor: TOKENS.surfaceActive } }}>
                  <ListItemIcon>
                    <EditOutlined fontSize="small" sx={{ color: TOKENS.textSecondary }} />
                  </ListItemIcon>
                  <ListItemText>Edit Display Name</ListItemText>
                </MenuItem>
                <Divider sx={{ my: 0.5, borderColor: TOKENS.borderSubtle }} />
                <MenuItem onClick={() => { setConnectionMenuAnchor(null); setAddConnectionDialogOpen(true); }} sx={{ color: TOKENS.textPrimary, '&:hover': { bgcolor: TOKENS.surfaceActive } }}>
                  <ListItemIcon>
                    <Add fontSize="small" sx={{ color: TOKENS.accent }} />
                  </ListItemIcon>
                  <ListItemText>Add Connection</ListItemText>
                </MenuItem>
              </Menu>

              {/* Add Connection Dialog */}
              <Modal
                open={addConnectionDialogOpen}
                onClose={() => setAddConnectionDialogOpen(false)}
              >
                <Paper
                  sx={{
                    position: 'absolute',
                    top: '50%',
                    left: '50%',
                    transform: 'translate(-50%, -50%)',
                    width: 450,
                    bgcolor: TOKENS.bgRaised,
                    border: `1px solid ${TOKENS.borderSubtle}`,
                    borderRadius: 3,
                    p: 3,
                  }}
                >
                  <Typography variant="h6" sx={{ color: TOKENS.textPrimary, mb: 1 }}>
                    Add Connection
                  </Typography>
                  <Typography variant="body2" sx={{ color: TOKENS.textSecondary, mb: 3 }}>
                    Paste the connection words shared by another user to connect to them.
                  </Typography>
                  <InputBase
                    fullWidth
                    multiline
                    rows={3}
                    value={connectionWordsInput}
                    onChange={(e) => setConnectionWordsInput(e.target.value)}
                    placeholder="ocean-forest-moon-star"
                    sx={{
                      bgcolor: TOKENS.bgBase,
                      border: `1px solid ${TOKENS.borderSubtle}`,
                      borderRadius: 2,
                      p: 1.5,
                      color: TOKENS.textPrimary,
                      fontFamily: 'monospace',
                      fontSize: 13,
                      '& ::placeholder': {
                        color: TOKENS.textSecondary,
                        opacity: 0.5,
                      }
                    }}
                  />
                  <Stack direction="row" spacing={2} sx={{ mt: 3 }}>
                    <Button
                      onClick={() => setAddConnectionDialogOpen(false)}
                      sx={{ color: TOKENS.textSecondary }}
                    >
                      Cancel
                    </Button>
                    <Button
                      variant="contained"
                      onClick={handleAddConnection}
                      disabled={!connectionWordsInput.trim()}
                      sx={{
                        bgcolor: TOKENS.accent,
                        color: '#fff',
                        '&:hover': { bgcolor: alpha(TOKENS.accent, 0.8) },
                        '&:disabled': { bgcolor: alpha(TOKENS.accent, 0.3) }
                      }}
                    >
                      Add Connection
                    </Button>
                  </Stack>
                </Paper>
              </Modal>

              {/* Edit Display Name Dialog */}
              <Modal
                open={editDisplayNameDialogOpen}
                onClose={() => setEditDisplayNameDialogOpen(false)}
              >
                <Paper
                  sx={{
                    position: 'absolute',
                    top: '50%',
                    left: '50%',
                    transform: 'translate(-50%, -50%)',
                    width: 450,
                    bgcolor: TOKENS.bgRaised,
                    border: `1px solid ${TOKENS.borderSubtle}`,
                    borderRadius: 3,
                    p: 3,
                  }}
                >
                  <Typography variant="h6" sx={{ color: TOKENS.textPrimary, mb: 1 }}>
                    Edit Display Name
                  </Typography>
                  <Typography variant="body2" sx={{ color: TOKENS.textSecondary, mb: 3 }}>
                    Choose how you appear to others in the network.
                  </Typography>
                  <InputBase
                    fullWidth
                    value={displayNameInput}
                    onChange={(e) => setDisplayNameInput(e.target.value)}
                    placeholder="Your Name"
                    sx={{
                      bgcolor: TOKENS.bgBase,
                      border: `1px solid ${TOKENS.borderSubtle}`,
                      borderRadius: 2,
                      p: 1.5,
                      color: TOKENS.textPrimary,
                      fontSize: 14,
                    }}
                  />
                  <Stack direction="row" spacing={2} sx={{ mt: 3 }}>
                    <Button onClick={() => setEditDisplayNameDialogOpen(false)} sx={{ color: TOKENS.textSecondary }}>
                      Cancel
                    </Button>
                    <Button
                      variant="contained"
                      onClick={handleSaveDisplayName}
                      disabled={!displayNameInput.trim()}
                      sx={{
                        bgcolor: TOKENS.accent,
                        color: '#fff',
                        '&:hover': { bgcolor: alpha(TOKENS.accent, 0.8) },
                        '&:disabled': { bgcolor: alpha(TOKENS.accent, 0.3) }
                      }}
                    >
                      Save
                    </Button>
                  </Stack>
                </Paper>
              </Modal>
            </>
          )}
        </Box>

        {/* B2. Filters */}
        <Box sx={{ px: 2, py: 1.5, borderBottom: `1px solid ${TOKENS.borderSubtle}`, display: 'grid', gap: 1.25 }}>
          <Stack direction="row" spacing={0.75} sx={{ overflowX: 'auto' }}>
            {scopeFilters.map((filter, index) => (
              <FilterChip
                key={filter.key}
                label={filter.label}
                variant={scopeFilter === filter.key ? 'filled' : 'outlined'}
                onClick={() => setScopeFilter(filter.key)}
                ref={el => {
                  scopeChipRefs.current[index] = el as HTMLElement | null
                }}
                onKeyDown={(event) => handleScopeChipKeyDown(event as React.KeyboardEvent<HTMLDivElement>, index)}
              />
            ))}
          </Stack>
          <Stack direction="row" spacing={0.75} sx={{ overflowX: 'auto' }}>
            {typeFilters.map((filter, index) => (
              <FilterChip
                key={filter.key}
                label={filter.label}
                variant={typeFilter === filter.key ? 'filled' : 'outlined'}
                onClick={() => setTypeFilter(filter.key)}
                ref={el => {
                  typeChipRefs.current[index] = el as HTMLElement | null
                }}
                onKeyDown={(event) => handleTypeChipKeyDown(event as React.KeyboardEvent<HTMLDivElement>, index)}
              />
            ))}
          </Stack>
        </Box>

        {/* B3. Search */}
        <Box sx={{ px: 2, py: 1.5 }}>
          <InputBase
            placeholder="Search or jump (⌘K)"
            inputRef={searchInputRef}
            onKeyDown={handleSearchKeyDown}
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
          <AnimatePresence initial={false}>
            {filteredConversations.map(conv => {
              const badge = presenceBadge(conv)
              const entityIcon = getEntityIcon(conv)
              const isOrg = conv.type === 'organisation'
              const orgName = isOrg ? conv.name.replace(' (Org)', '') : null
              const isExpanded = orgName ? expandedOrgs.has(orgName) : false
              const hasChildren = (conv as any).children?.length > 0

              if (isOrg) {
                return (
                  <motion.div
                    key={conv.id}
                    layout
                    initial={{ opacity: 0, y: 8 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -8 }}
                    transition={LIST_ITEM_TRANSITION}
                  >
                    <Box
                      onClick={() => {
                        if (hasChildren) {
                          toggleOrgExpansion(orgName!)
                        }
                        setSelectedConversationId(conv.id)
                      }}
                      sx={{
                        px: 2,
                        py: 1.5,
                        mt: 2,
                        mb: 0.5,
                        display: 'flex',
                        alignItems: 'center',
                        gap: 1,
                        cursor: 'pointer',
                        bgcolor: conv.id === selectedConversationId ? alpha('#FFFFFF', 0.08) : 'transparent',
                        borderRadius: 2,
                        '&:hover': {
                          bgcolor: alpha('#FFFFFF', 0.05),
                        },
                      }}
                    >
                      {hasChildren && (
                        <IconButton
                          size="small"
                          onClick={(e) => {
                            e.stopPropagation()
                            toggleOrgExpansion(orgName!)
                          }}
                          sx={{ color: TOKENS.textSecondary, padding: 0.5 }}
                        >
                          {isExpanded ? <ExpandLess fontSize="small" /> : <ExpandMore fontSize="small" />}
                        </IconButton>
                      )}
                      {entityIcon}
                      <Typography
                        variant="caption"
                        fontWeight={700}
                        sx={{
                          color: TOKENS.textPrimary,
                          textTransform: 'uppercase',
                          letterSpacing: 0.5,
                          flexGrow: 1,
                        }}
                      >
                        {conv.name.replace(' (Org)', '')}
                      </Typography>
                      {conv.membersOnline && conv.membersOnline > 0 && (
                        <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>
                          {conv.membersOnline} online
                        </Typography>
                      )}
                    </Box>

                    <AnimatePresence initial={false}>
                      {isExpanded && (conv as any).children?.map((child: Conversation) => {
                        const childBadge = presenceBadge(child)
                        const childIcon = getEntityIcon(child)
                        return (
                          <motion.div
                            key={child.id}
                            layout
                            initial={{ opacity: 0, y: 6 }}
                            animate={{ opacity: 1, y: 0 }}
                            exit={{ opacity: 0, y: -6 }}
                            transition={LIST_ITEM_TRANSITION}
                          >
                            <ConversationListItem
                              selected={child.id === selectedConversationId}
                              onClick={() => setSelectedConversationId(child.id)}
                              sx={{
                                position: 'relative',
                                pl: 4,
                                '&:hover .entity-menu-button': {
                                  opacity: 1,
                                },
                              }}
                            >
                              <Box sx={{ position: 'relative' }}>
                                <Avatar sx={{ width: 40, height: 40, ...(child.type ? avatarShapeStyles[child.type] : {}) }}>
                                  {child.name?.substring(0, 2).toUpperCase() || '??'}
                                </Avatar>
                                <Box
                                  sx={{
                                    position: 'absolute',
                                    bottom: -2,
                                    right: -2,
                                    width: 18,
                                    height: 18,
                                    borderRadius: '50%',
                                    bgcolor: childBadge.bg,
                                    display: 'flex',
                                    alignItems: 'center',
                                    justifyContent: 'center',
                                    border: `2px solid ${TOKENS.bgRaised}`,
                                  }}
                                >
                                  {React.isValidElement(childBadge.icon) && React.cloneElement(childBadge.icon, { sx: { fontSize: 14, color: '#FFFFFF' } })}
                                </Box>
                              </Box>
                              <Box sx={{ flexGrow: 1, minWidth: 0 }}>
                                <Stack direction="row" spacing={1} alignItems="center" justifyContent="space-between">
                                  <Stack direction="row" spacing={0.5} alignItems="center" sx={{ flexGrow: 1, minWidth: 0 }}>
                                    {childIcon}
                                    <Typography variant="body2" fontWeight={500} noWrap>{child.name}</Typography>
                                    <IconButton
                                      className="entity-menu-button"
                                      size="small"
                                      onClick={(e) => handleSidebarMenuOpen(e, child)}
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
                                  <Typography variant="caption" sx={{ color: TOKENS.textSecondary, flexShrink: 0 }}>{child.time}</Typography>
                                </Stack>
                                <Stack direction="row" spacing={0.5} alignItems="center">
                                  <Typography variant="caption" sx={{ color: TOKENS.textSecondary, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                                    {child.snippet}
                                  </Typography>
                                  {child.status === 'read' && <CheckCircle sx={{ fontSize: 12, color: TOKENS.accent, flexShrink: 0 }} />}
                                </Stack>
                              </Box>
                              {child.unread && (
                                <Badge badgeContent={child.unread} sx={{ '& .MuiBadge-badge': { bgcolor: TOKENS.accent, color: '#000', fontSize: 10, minWidth: 16, height: 16 } }} />
                              )}
                            </ConversationListItem>
                          </motion.div>
                        )
                      })}
                    </AnimatePresence>
                  </motion.div>
                )
              }

              return (
                <motion.div
                  key={conv.id}
                  layout
                  initial={{ opacity: 0, y: 8 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -8 }}
                  transition={LIST_ITEM_TRANSITION}
                >
                  <ConversationListItem
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
                      <Avatar sx={{ width: 48, height: 48, ...(conv.type ? avatarShapeStyles[conv.type] : {}) }}>
                        {conv.name?.substring(0, 2).toUpperCase() || '??'}
                      </Avatar>
                      {!entityIcon && React.isValidElement(badge.icon) && (
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
                          {React.cloneElement(badge.icon, { sx: { fontSize: 14, color: '#FFFFFF' } })}
                        </Box>
                      )}
                    </Box>
                    <Box sx={{ flexGrow: 1, minWidth: 0 }}>
                      <Stack direction="row" spacing={1} alignItems="center" justifyContent="space-between">
                        <Stack direction="row" spacing={0.5} alignItems="center" sx={{ flexGrow: 1, minWidth: 0 }}>
                          {entityIcon}
                          <Typography variant="body2" fontWeight={500} noWrap>{conv.name}</Typography>
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
                        <Typography variant="caption" sx={{ color: TOKENS.textSecondary, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                          {conv.snippet}
                        </Typography>
                        {conv.status === 'read' && <CheckCircle sx={{ fontSize: 12, color: TOKENS.accent, flexShrink: 0 }} />}
                      </Stack>
                    </Box>
                    {conv.unread && (
                      <Badge badgeContent={conv.unread} sx={{ '& .MuiBadge-badge': { bgcolor: TOKENS.accent, color: '#000', fontSize: 10, minWidth: 20, height: 20 } }} />
                    )}
                  </ConversationListItem>
                </motion.div>
              )
            })}
          </AnimatePresence>
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
            <Avatar sx={{ width: 40, height: 40 }}>{selectedConversation?.name?.substring(0, 2) || '??'}</Avatar>
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
                <Typography variant="subtitle1" fontWeight={600}>{selectedConversation?.name || 'Unknown'}</Typography>
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
            <Tooltip title="Files">
              <IconButton sx={{ color: TOKENS.textSecondary }} onClick={() => console.log('Open files for:', selectedConversation?.name)}>
                <FolderOutlined />
              </IconButton>
            </Tooltip>
            <Tooltip title="Website">
              <IconButton sx={{ color: TOKENS.textSecondary }} onClick={handleWebsiteOpen}>
                <LanguageOutlined />
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

        {/* View Mode Chips for Groups */}
        {isGroupView && (
          <Box sx={{ px: 3, py: 1.5, borderBottom: `1px solid ${TOKENS.borderSubtle}`, bgcolor: TOKENS.bgRaised }}>
            <Stack direction="row" spacing={1}>
              {groupModes.map(mode => (
                <ViewChip
                  key={mode.key}
                  label={mode.label}
                  variant={groupViewMode === mode.key ? 'filled' : 'outlined'}
                  onClick={() => setGroupViewMode(mode.key)}
                />
              ))}
            </Stack>
          </Box>
        )}

        {/* View Mode Chips for People */}
        {isPersonView && (
          <Box sx={{ px: 3, py: 1.5, borderBottom: `1px solid ${TOKENS.borderSubtle}`, bgcolor: TOKENS.bgRaised }}>
            <Stack direction="row" spacing={1}>
              {personModes.map(mode => (
                <ViewChip
                  key={mode.key}
                  label={mode.label}
                  variant={personViewMode === mode.key ? 'filled' : 'outlined'}
                  onClick={() => setPersonViewMode(mode.key)}
                />
              ))}
            </Stack>
          </Box>
        )}

        {/* C2. Content Area - renders different views based on mode */}
        {isOrganisationView ? renderOrganisationOverview() :
         isChannelView && channelViewMode !== 'chat' ? renderChannelMode() :
         isProjectView && projectViewMode !== 'chat' ? renderProjectMode() :
         isGroupView && groupViewMode !== 'chat' ? renderGroupMode() :
         isPersonView && personViewMode !== 'chat' ? renderPersonMode() :
         renderChatTimeline()}

        {/* C3. Composer (only show for chat modes) */}
        {!isOrganisationView &&
         ((!isChannelView && !isProjectView && !isGroupView && !isPersonView) ||
          (isChannelView && channelViewMode === 'chat') ||
          (isProjectView && projectViewMode === 'chat') ||
          (isGroupView && groupViewMode === 'chat') ||
          (isPersonView && personViewMode === 'chat')) && (
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
          <ListItemText>Edit {selectedConversation?.type === 'person' ? 'Contact' : 'Entity'}</ListItemText>
        </MenuItem>
        {selectedConversation?.type === 'organisation' && (
          <MenuItem onClick={() => {
            setActiveDrawerTab('Storage')
            setDrawerOpen(true)
            handleEntityMenuClose()
          }}>
            <ListItemIcon>
              <StorageOutlined fontSize="small" sx={{ color: TOKENS.textPrimary }} />
            </ListItemIcon>
            <ListItemText>Storage</ListItemText>
          </MenuItem>
        )}
        <Divider sx={{ my: 0.5, borderColor: TOKENS.borderSubtle }} />
        <MenuItem onClick={handleEntityDelete}>
          <ListItemIcon>
            <DeleteOutline fontSize="small" sx={{ color: TOKENS.danger }} />
          </ListItemIcon>
          <ListItemText sx={{ color: TOKENS.danger }}>
            Delete {selectedConversation?.type === 'person' ? 'Contact' : 'Entity'}
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
        {sidebarMenuState.conversation?.type === 'organisation' && (
          <MenuItem onClick={() => {
            setSelectedConversationId(sidebarMenuState.conversation!.id)
            setActiveDrawerTab('Storage')
            setDrawerOpen(true)
            handleSidebarMenuClose()
          }}>
            <ListItemIcon>
              <StorageOutlined fontSize="small" sx={{ color: TOKENS.textPrimary }} />
            </ListItemIcon>
            <ListItemText>Storage</ListItemText>
          </MenuItem>
        )}
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

      <Modal
        open={isCommandPaletteOpen}
        onClose={() => setIsCommandPaletteOpen(false)}
        closeAfterTransition
      >
        <Box
          sx={{
            minHeight: '100vh',
            display: 'flex',
            alignItems: 'flex-start',
            justifyContent: 'center',
            pt: 12,
          }}
        >
          <Paper
            elevation={8}
            sx={{
              width: 'min(640px, 90vw)',
              bgcolor: TOKENS.bgRaised,
              borderRadius: 3,
              border: `1px solid ${TOKENS.borderSubtle}`,
              boxShadow: '0px 24px 48px rgba(0,0,0,0.35)',
              overflow: 'hidden',
            }}
          >
            <Box sx={{ px: 3, py: 2, borderBottom: `1px solid ${TOKENS.borderSubtle}` }}>
              <InputBase
                inputRef={commandInputRef}
                value={commandQuery}
                onChange={(event) => setCommandQuery(event.target.value)}
                placeholder="Jump to organisation, channel, project…"
                startAdornment={<SearchIcon sx={{ mr: 1.5, color: TOKENS.textSecondary }} />}
                sx={{
                  width: '100%',
                  fontSize: 16,
                  fontWeight: 500,
                  color: TOKENS.textPrimary,
                }}
              />
            </Box>
            <Box sx={{ maxHeight: 320, overflowY: 'auto', p: 1.5, display: 'flex', flexDirection: 'column', gap: 0.75 }}>
              {commandItems.length > 0 ? (
                commandItems.map(item => (
                  <Box
                    key={item.id}
                    role="button"
                    tabIndex={0}
                    onClick={() => handleCommandItemSelect(item)}
                    onKeyDown={(event) => {
                      if (event.key === 'Enter') {
                        handleCommandItemSelect(item)
                      }
                    }}
                    sx={{
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      px: 2,
                      py: 1.25,
                      borderRadius: 2,
                      cursor: 'pointer',
                      bgcolor:
                        item.type === 'entity' && item.conversation.id === selectedConversationId
                          ? alpha(TOKENS.accent, 0.12)
                          : 'transparent',
                      transition: 'background 120ms ease',
                      outline: 'none',
                      '&:hover': {
                        bgcolor: alpha(TOKENS.accent, 0.16),
                      },
                      '&:focus-visible': {
                        boxShadow: `0 0 0 1px ${alpha(TOKENS.accent, 0.8)}`,
                      },
                    }}
                  >
                    {item.type === 'entity' ? (
                      <>
                        <Stack direction="row" spacing={1.5} alignItems="center" sx={{ minWidth: 0 }}>
                          <Avatar sx={{ width: 36, height: 36, fontSize: 14, ...(item.conversation.type ? avatarShapeStyles[item.conversation.type] : {}) }}>
                            {item.conversation.name?.substring(0, 2).toUpperCase() || '??'}
                          </Avatar>
                          <Box sx={{ minWidth: 0 }}>
                            <Typography variant="body2" fontWeight={600} noWrap>{item.conversation.name}</Typography>
                            <Typography variant="caption" sx={{ color: TOKENS.textSecondary }} noWrap>
                              {item.conversation.org ?? (item.conversation.scope === 'personal' ? 'Personal Space' : 'Organisation')} · {item.conversation.type ? conversationTypeLabel[item.conversation.type] : 'Unknown'}
                            </Typography>
                          </Box>
                        </Stack>
                        <Typography variant="caption" sx={{ color: TOKENS.textSecondary, flexShrink: 0 }}>
                          {item.conversation.time}
                        </Typography>
                      </>
                    ) : (
                      <>
                        <Stack direction="row" spacing={1.5} alignItems="center" sx={{ minWidth: 0 }}>
                          <Avatar sx={{ width: 36, height: 36, fontSize: 16, bgcolor: alpha(TOKENS.accent, 0.18), color: TOKENS.accent }}>
                            <Add fontSize="small" />
                          </Avatar>
                          <Box sx={{ minWidth: 0 }}>
                            <Typography variant="body2" fontWeight={600} noWrap>{item.label}</Typography>
                            {item.subtitle && (
                              <Typography variant="caption" sx={{ color: TOKENS.textSecondary }} noWrap>
                                {item.subtitle}
                              </Typography>
                            )}
                          </Box>
                        </Stack>
                        <Typography variant="caption" sx={{ color: TOKENS.textSecondary, flexShrink: 0 }}>
                          Action
                        </Typography>
                      </>
                    )}
                  </Box>
                ))
              ) : (
                <Typography variant="body2" sx={{ color: TOKENS.textSecondary, px: 2, py: 4, textAlign: 'center' }}>
                  No matching entities. Try different keywords.
                </Typography>
              )}
            </Box>
          </Paper>
        </Box>
      </Modal>

      {/* Entity Management Dialogs */}
      <EditContactDialog
        open={contactDialogMode === 'edit'}
        contact={dialogContact}
        onClose={() => {
          setContactDialogMode(null)
          setSelectedContact(null)
        }}
        onSave={handleSaveEntityEdit}
      />

      <DeleteContactDialog
        open={contactDialogMode === 'delete'}
        contact={dialogContact}
        onClose={() => {
          setContactDialogMode(null)
          setSelectedContact(null)
        }}
        onConfirm={handleConfirmEntityDelete}
      />
    </Box>
  )
}
