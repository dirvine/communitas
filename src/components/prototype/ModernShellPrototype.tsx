import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  AppBar,
  Avatar,
  Badge,
  Box,
  Button,
  Chip,
  Divider,
  Drawer,
  IconButton,
  InputBase,
  List,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Menu,
  MenuItem,
  Modal,
  Paper,
  Stack,
  Tab,
  Tabs,
  Tooltip,
  Typography,
  alpha,
} from '@mui/material';
import {
  Add,
  AppsRounded,
  CallRounded,
  ChatBubbleOutlineRounded,
  CloseRounded,
  CreateRounded,
  ExpandMore,
  FolderRounded,
  InfoRounded,
  KeyboardArrowLeft,
  ChevronLeft,
  ChevronRight,
  NotificationsOffRounded,
  PeopleAltRounded,
  PersonRounded,
  SearchRounded,
  SendRounded,
  StorageOutlined,
  VideoCallRounded,
  WebRounded,
} from '@mui/icons-material';
import { useEntityDirectory } from '../../contexts/EntityDirectoryContext';
import { useAuth } from '../../contexts/AuthContext';
import { IdentityPicker } from '../auth/IdentityPicker';
import { UnifiedAuthFlow } from '../auth/UnifiedAuthFlow';
import { FirstLaunchWelcome } from '../auth/FirstLaunchWelcome';
import { MessageSyncService } from '../../services/MessageSyncService.browser';
import type { CRDTMessage } from '../../types/crdt';
import { EntityDocumentWorkspace } from '../documents/EntityDocumentWorkspace';
import type { DocumentStorageMode } from '../../types/documents';
import { fourWordsToDisplay } from '../../utils/identity';

type ConversationKind = 'organization' | 'channel' | 'group' | 'project' | 'person' | 'personal-root';

type Conversation = {
  id: string;
  entityId: string;
  title: string;
  subtitle: string;
  badge?: string;
  kind: ConversationKind;
  parentOrgId?: string;
  parentOrgName?: string;
  fourWords?: string;
  members?: number;
  updatedAt?: Date;
  unread?: number;
  pinned?: boolean;
  depth?: number;
  hasChildren?: boolean;
  clickable?: boolean;
};

type Message = {
  id: string;
  author: string;
  text: string;
  timestamp: string;
  own: boolean;
  status?: 'sent' | 'delivered' | 'read';
};

type CallState = {
  open: boolean;
  type: 'audio' | 'video';
  startedAt: number;
};

type ViewOption = {
  key: string;
  label: string;
};

const TOKENS = {
  surface0: '#0B1115',
  surface1: '#10171C',
  surface2: '#172027',
  surface3: '#1F2A32',
  textPrimary: '#F2F4F7',
  textSecondary: '#9BA6B2',
  accent: '#3DD68C',
  accentSecondary: '#4DA3FF',
  danger: '#F87171',
};

const serviceSingleton = new MessageSyncService();

const useMessageService = (peerId: string | undefined) => {
  const initialisedFor = useRef<string | null>(null);

  useEffect(() => {
    if (!peerId) return;
    if (initialisedFor.current === peerId) return;

    serviceSingleton
      .initialize(peerId)
      .then(() => {
        initialisedFor.current = peerId;
      })
      .catch(console.error);
  }, [peerId]);

  return serviceSingleton;
};

const convertMessage = (message: CRDTMessage, peerId: string): Message => {
  const text = message.content?.text ?? '';
  const author = message.content?.author ?? 'Unknown';
  const timestamp = new Date(message.metadata.timestamp ?? Date.now()).toISOString();
  const own = message.metadata.authorPeerId === peerId;
  const lamport = message.metadata.lamportClock ?? 0;
  const status =
    lamport > 2 ? 'read' : lamport === 2 ? 'delivered' : 'sent';

  return {
    id: message.metadata.id,
    text,
    author,
    timestamp,
    own,
    status,
  };
};

const formatRelationshipLabel = (relationship?: string): string => {
  if (!relationship) return 'Personal contact';
  switch (relationship) {
    case 'friend':
      return 'Friend • Personal contact';
    case 'family':
      return 'Family contact';
    case 'colleague':
      return 'Colleague';
    case 'acquaintance':
      return 'Acquaintance';
    default:
      return 'Personal contact';
  }
};

const formatRelative = (date?: Date) => {
  if (!date) return '';
  const diffMs = Date.now() - date.getTime();
  const seconds = Math.max(0, Math.floor(diffMs / 1000));
  if (seconds < 60) return 'now';
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h`;
  const days = Math.floor(hours / 24);
  if (days < 7) return `${days}d`;
  const weeks = Math.floor(days / 7);
  if (weeks < 4) return `${weeks}w`;
  const months = Math.floor(days / 30);
  if (months < 12) return `${months}mo`;
  const years = Math.floor(days / 365);
  return `${years}y`;
};

const ConversationListEmpty: React.FC<{ onCreate: (anchor: HTMLElement | null) => void }> = ({ onCreate }) => (
  <Paper
    elevation={0}
    sx={{
      mt: 6,
      mx: 2,
      p: 3,
      borderRadius: 3,
      background: alpha('#fff', 0.04),
      textAlign: 'center',
    }}
  >
    <Typography variant="h6" sx={{ color: TOKENS.textPrimary, mb: 1 }}>
      No spaces yet
    </Typography>
    <Typography variant="body2" sx={{ color: TOKENS.textSecondary, mb: 2 }}>
      Create your first channel, project, or contact to start collaborating.
    </Typography>
    <Button
      variant="contained"
      startIcon={<Add />}
      onClick={(event) => onCreate(event.currentTarget)}
      sx={{
        bgcolor: TOKENS.accent,
        color: '#041218',
        '&:hover': { bgcolor: alpha(TOKENS.accent, 0.85) },
      }}
    >
      New space
    </Button>
  </Paper>
);

const ActiveCallModal: React.FC<{ state: CallState; conversation?: Conversation; onClose: () => void }> = ({
  state,
  conversation,
  onClose,
}) => {
  const [elapsed, setElapsed] = useState(0);

  useEffect(() => {
    if (!state.open) return;
    const handle = setInterval(() => {
      setElapsed(Math.floor((Date.now() - state.startedAt) / 1000));
    }, 1000);
    return () => clearInterval(handle);
  }, [state]);

  const minutes = Math.floor(elapsed / 60)
    .toString()
    .padStart(2, '0');
  const seconds = (elapsed % 60).toString().padStart(2, '0');

  return (
    <Modal open={state.open} onClose={onClose}>
      <Paper
        sx={{
          position: 'absolute',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          bgcolor: TOKENS.surface2,
          minWidth: 360,
          borderRadius: 4,
          p: 4,
          display: 'grid',
          gap: 2,
        }}
      >
        <Stack direction="row" spacing={2} alignItems="center">
          <Avatar sx={{ width: 48, height: 48 }}>
            {conversation?.title.slice(0, 2).toUpperCase()}
          </Avatar>
          <Box>
            <Typography variant="h6" sx={{ color: TOKENS.textPrimary }}>
              {state.type === 'audio' ? 'Voice call' : 'Video call'}
            </Typography>
            <Typography variant="body2" sx={{ color: TOKENS.textSecondary }}>
              {conversation?.title ?? 'Unknown'}
            </Typography>
          </Box>
        </Stack>
        <Typography variant="h3" sx={{ color: TOKENS.textPrimary, textAlign: 'center' }}>
          {minutes}:{seconds}
        </Typography>
        <Typography sx={{ color: TOKENS.textSecondary, textAlign: 'center' }}>
          You are connected. Screen sharing and recording controls appear after the call starts.
        </Typography>
        <Stack direction="row" spacing={2}>
          <Button
            fullWidth
            variant="outlined"
            onClick={onClose}
            startIcon={<CloseRounded />}
            sx={{ color: TOKENS.textSecondary, borderColor: alpha('#fff', 0.1) }}
          >
            End call
          </Button>
          <Button
            fullWidth
            variant="contained"
            onClick={() => {
              navigator.clipboard.writeText(`Call with ${conversation?.title ?? 'contact'} ended at ${new Date().toLocaleTimeString()}`);
              onClose();
            }}
            startIcon={<NotificationsOffRounded />}
            sx={{ bgcolor: TOKENS.accent, color: '#041218', '&:hover': { bgcolor: alpha(TOKENS.accent, 0.85) } }}
          >
            Mute alerts
          </Button>
        </Stack>
      </Paper>
    </Modal>
  );
};

const CommandPalette: React.FC<{
  open: boolean;
  onClose: () => void;
  conversations: Conversation[];
  onSelect: (conversation: Conversation) => void;
}> = ({ open, onClose, conversations, onSelect }) => {
  const [query, setQuery] = useState('');
  const filtered = useMemo(() => {
    const term = query.trim().toLowerCase();
    if (!term) return conversations.slice(0, 12);
    return conversations.filter(item =>
      [item.title, item.subtitle, item.parentOrgName].filter(Boolean).join(' ').toLowerCase().includes(term),
    );
  }, [conversations, query]);

  useEffect(() => {
    if (!open) setQuery('');
  }, [open]);

  return (
    <Modal open={open} onClose={onClose}>
      <Box
        sx={{
          minHeight: '100vh',
          display: 'flex',
          justifyContent: 'center',
          pt: 12,
        }}
      >
        <Paper
          sx={{
            width: 'min(640px, 90vw)',
            bgcolor: TOKENS.surface2,
            borderRadius: 3,
            border: `1px solid ${alpha('#fff', 0.08)}`,
            overflow: 'hidden',
          }}
        >
          <Box sx={{ px: 3, py: 2, borderBottom: `1px solid ${alpha('#fff', 0.05)}` }}>
            <InputBase
              autoFocus
              fullWidth
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Jump to channel, project, contact..."
              startAdornment={<SearchRounded sx={{ color: TOKENS.textSecondary, mr: 1 }} />}
              sx={{ color: TOKENS.textPrimary, fontSize: 16 }}
            />
          </Box>
          <List sx={{ bgcolor: TOKENS.surface2, maxHeight: 360, overflowY: 'auto' }}>
            {filtered.length === 0 && (
              <Box sx={{ px: 3, py: 4 }}>
                <Typography sx={{ color: TOKENS.textSecondary, textAlign: 'center' }}>
                  No matches. Try a different search term.
                </Typography>
              </Box>
            )}
            {filtered.map(item => (
              <ListItemButton
                key={item.id}
                onClick={() => {
                  onSelect(item);
                  onClose();
                }}
                sx={{
                  '&:hover': { bgcolor: alpha(TOKENS.accent, 0.12) },
                }}
              >
                <ListItemIcon>
                  <Avatar
                    sx={{
                      width: 36,
                      height: 36,
                      bgcolor: alpha(TOKENS.accent, 0.2),
                      color: TOKENS.textPrimary,
                    }}
                  >
                    {item.title.slice(0, 2).toUpperCase()}
                  </Avatar>
                </ListItemIcon>
                <ListItemText
                  primary={
                    <Typography sx={{ color: TOKENS.textPrimary, fontWeight: 600 }}>
                      {item.title}
                    </Typography>
                  }
                  secondary={
                    <Typography sx={{ color: TOKENS.textSecondary }}>
                      {item.subtitle}
                    </Typography>
                  }
                />
              </ListItemButton>
            ))}
          </List>
        </Paper>
      </Box>
    </Modal>
  );
};

const ConversationDetailDrawer: React.FC<{
  open: boolean;
  onClose: () => void;
  conversation?: Conversation;
}> = ({ open, onClose, conversation }) => {
  const [tab, setTab] = useState<'overview' | 'files' | 'website'>('overview');

  useEffect(() => {
    if (!open) {
      setTab('overview');
    }
  }, [open]);

  return (
    <Drawer
      anchor="right"
      open={open}
      onClose={onClose}
      PaperProps={{
        sx: {
          width: 360,
          bgcolor: TOKENS.surface2,
          color: TOKENS.textPrimary,
        },
      }}
    >
      <Box sx={{ p: 3 }}>
        <Stack direction="row" spacing={1.5} alignItems="center">
          <IconButton onClick={onClose} sx={{ color: TOKENS.textSecondary }}>
            <KeyboardArrowLeft />
          </IconButton>
          <Avatar sx={{ width: 44, height: 44 }}>
            {conversation?.title.slice(0, 2).toUpperCase()}
          </Avatar>
          <Box>
            <Typography variant="h6">{conversation?.title}</Typography>
            <Typography variant="body2" sx={{ color: TOKENS.textSecondary }}>
              {conversation?.subtitle}
            </Typography>
          </Box>
        </Stack>
        <Tabs
          value={tab}
          onChange={(_, value) => setTab(value)}
          sx={{ mt: 3, mb: 2 }}
          textColor="inherit"
          indicatorColor="primary"
        >
          <Tab label="Overview" value="overview" />
          <Tab label="Files" value="files" />
          <Tab label="Website" value="website" />
        </Tabs>
        {tab === 'overview' && (
          <Stack spacing={2}>
            <Box>
              <Typography sx={{ fontWeight: 600, mb: 1 }}>Identity</Typography>
              <Typography sx={{ color: TOKENS.textSecondary, fontFamily: 'monospace' }}>
                {conversation?.fourWords ? fourWordsToDisplay(conversation.fourWords) : 'Not assigned'}
              </Typography>
            </Box>
            <Box>
              <Typography sx={{ fontWeight: 600, mb: 1 }}>Details</Typography>
              <Typography sx={{ color: TOKENS.textSecondary }}>
                {conversation?.kind === 'person'
                  ? 'Direct messages stay private to you and this contact.'
                  : conversation?.kind === 'channel'
                  ? 'Channels keep conversations organised for your organisation.'
                  : conversation?.kind === 'organization'
                  ? 'Organisation overview with announcements, members, and automations.'
                  : 'Shared workspace across devices and peers.'}
              </Typography>
            </Box>
            <Box>
              <Typography sx={{ fontWeight: 600, mb: 1 }}>Members online</Typography>
              <Chip
                label={`${conversation?.members ?? 0} collaborators`}
                size="small"
                sx={{ bgcolor: alpha(TOKENS.accent, 0.12), color: TOKENS.accent }}
              />
            </Box>
          </Stack>
        )}
        {tab === 'files' && conversation && (
          <EntityDocumentWorkspace
            entityId={conversation.entityId}
            entityName={conversation.title}
            storageMode="files"
            permissions={['read', 'write']}
          />
        )}
        {tab === 'website' && (
          <Box sx={{ mt: 2 }}>
            <Typography sx={{ color: TOKENS.textSecondary }}>
              Publish markdown files straight to the decentralised website address tied to this identity.
              Create a document in the "Website" storage mode and mark it as the home page to go live instantly.
            </Typography>
          </Box>
        )}
      </Box>
    </Drawer>
  );
};

const formatTimestamp = (iso: string) => {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return '';
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
};

export const ModernShellPrototypeScreen: React.FC = () => {
  const { authState, isFirstLaunch, getRecentIdentities, signInWithPasskey, login } = useAuth();
  const {
    organizations,
    personalGroups,
    personalUsers,
    createOrganization,
    createChannel,
    createProject,
    createGroup,
    createContact,
  } = useEntityDirectory();

  const [navExpanded, setNavExpanded] = useState(true);
  const [scopeFilter, setScopeFilter] = useState<'all' | 'org' | 'personal'>('all');
  const [search, setSearch] = useState('');
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [composerValue, setComposerValue] = useState('');
  const [callState, setCallState] = useState<CallState>({ open: false, type: 'audio', startedAt: Date.now() });
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [commandOpen, setCommandOpen] = useState(false);
  const [showIdentityPicker, setShowIdentityPicker] = useState(false);
  const [showUnifiedFlow, setShowUnifiedFlow] = useState(false);
  const [showWelcome, setShowWelcome] = useState(false);
  const [identityMenuAnchor, setIdentityMenuAnchor] = useState<HTMLElement | null>(null);
  const [recentIdentities, setRecentIdentities] = useState<Array<{ four_words: string; display_name: string; last_used: number; has_passkey: boolean }>>([]);
  const [createMenuAnchor, setCreateMenuAnchor] = useState<HTMLElement | null>(null);
  const [activeView, setActiveView] = useState<string>('chat');
  const [storageTab, setStorageTab] = useState<'private' | 'shared' | 'public'>('private');
  const [expandedOrgs, setExpandedOrgs] = useState<Set<string>>(() => new Set());

  const peerId = authState.user?.fourWordAddress ?? '';
  const messageService = useMessageService(peerId);

  const handleToggleOrg = useCallback((orgId: string) => {
    setExpandedOrgs(prev => {
      const next = new Set(prev);
      if (next.has(orgId)) {
        next.delete(orgId);
      } else {
        next.add(orgId);
      }
      return next;
    });
  }, []);

  const conversations = useMemo<Conversation[]>(() => {
    const items: Conversation[] = [];

    organizations.forEach(org => {
      items.push({
        id: `org:${org.id}`,
        entityId: org.id,
        title: org.name,
        subtitle: org.description ?? 'Organisation workspace',
        badge: 'ORG',
        kind: 'organization',
        fourWords: org.networkIdentity?.fourWords,
        members: org.users?.length ?? 0,
        updatedAt: org.updatedAt ? new Date(org.updatedAt) : undefined,
        depth: 0,
        hasChildren:
          (org.channels?.length ?? 0) + (org.projects?.length ?? 0) + (org.groups?.length ?? 0) > 0,
        clickable: true,
      });

      (org.channels ?? []).forEach(channel => {
        items.push({
          id: `channel:${channel.id}`,
          entityId: channel.id,
          title: channel.name.startsWith('#') ? channel.name : `#${channel.name}`,
          subtitle: `${org.name} • Channel`,
          badge: 'Channel',
          kind: 'channel',
          parentOrgId: org.id,
          parentOrgName: org.name,
          fourWords: channel.networkIdentity?.fourWords,
          members: channel.members?.length ?? 0,
          updatedAt: channel.updatedAt ? new Date(channel.updatedAt) : undefined,
          depth: 1,
          hasChildren: false,
          clickable: true,
        });
      });

      (org.projects ?? []).forEach(project => {
        items.push({
          id: `project:${project.id}`,
          entityId: project.id,
          title: project.name,
          subtitle: `${org.name} • Project`,
          badge: 'Project',
          kind: 'project',
          parentOrgId: org.id,
          parentOrgName: org.name,
          fourWords: project.networkIdentity?.fourWords,
          members: (project.members ?? []).length,
          updatedAt: project.updatedAt ? new Date(project.updatedAt) : undefined,
          depth: 1,
          hasChildren: false,
          clickable: true,
        });
      });

      (org.groups ?? []).forEach(group => {
        items.push({
          id: `group:${group.id}`,
          entityId: group.id,
          title: group.name,
          subtitle: `${org.name} • Group`,
          badge: 'Group',
          kind: 'group',
          parentOrgId: org.id,
          parentOrgName: org.name,
          fourWords: group.networkIdentity?.fourWords,
          members: group.members?.length ?? 0,
          updatedAt: group.updatedAt ? new Date(group.updatedAt) : undefined,
          depth: 1,
          hasChildren: false,
          clickable: true,
        });
      });
    });

    personalGroups.forEach(group => {
      items.push({
        id: `group:${group.id}`,
        entityId: group.id,
        title: group.name,
        subtitle: group.description ?? 'Personal group',
        badge: 'Personal',
        kind: 'group',
        fourWords: group.networkIdentity?.fourWords,
        members: group.members?.length ?? 0,
        updatedAt: group.updatedAt ? new Date(group.updatedAt) : undefined,
        depth: 0,
        hasChildren: false,
        clickable: true,
      });
    });

    personalUsers.forEach(user => {
      items.push({
        id: `person:${user.id}`,
        entityId: user.id,
        title: user.name,
        subtitle: formatRelationshipLabel(user.relationship),
        badge: 'DM',
        kind: 'person',
        fourWords: user.networkIdentity?.fourWords,
        members: 1,
        updatedAt: user.updatedAt ? new Date(user.updatedAt) : undefined,
        depth: 0,
        hasChildren: false,
        clickable: true,
      });
    });

    return items;
  }, [organizations, personalGroups, personalUsers]);

  const conversationMap = useMemo(() => {
    const map = new Map<string, Conversation>();
    conversations.forEach(conversation => {
      map.set(conversation.id, conversation);
    });
    return map;
  }, [conversations]);

  const organizationChildren = useMemo(() => {
    const map = new Map<string, Conversation[]>();
    conversations.forEach(conversation => {
      if (conversation.parentOrgId) {
        if (!map.has(conversation.parentOrgId)) {
          map.set(conversation.parentOrgId, []);
        }
        map.get(conversation.parentOrgId)!.push(conversation);
      }
    });
    map.forEach(children => {
      children.sort((a, b) => {
        const aTime = a.updatedAt?.getTime() ?? 0;
        const bTime = b.updatedAt?.getTime() ?? 0;
        return bTime - aTime;
      });
    });
    return map;
  }, [conversations]);

  const activeOrg = useMemo(() => {
    if (!selectedId) return null;
    const conversation = conversations.find(item => item.id === selectedId);
    if (!conversation) return null;
    if (conversation.kind === 'organization') {
      return organizations.find(org => org.id === conversation.entityId) ?? null;
    }
    if (conversation.parentOrgId) {
      return organizations.find(org => org.id === conversation.parentOrgId) ?? null;
    }
    return null;
  }, [conversations, organizations, selectedId]);

  const filteredConversations = useMemo(() => {
    const term = search.trim().toLowerCase();
    const includeOrgs = scopeFilter !== 'personal';
    const includePersonal = scopeFilter !== 'org';
    const result: Conversation[] = [];

    const matches = (conversation: Conversation): boolean => {
      if (!term) return true;
      return [conversation.title, conversation.subtitle, conversation.fourWords]
        .filter(Boolean)
        .join(' ')
        .toLowerCase()
        .includes(term);
    };

    if (includeOrgs) {
      organizations.forEach(org => {
        const orgConversation = conversationMap.get(`org:${org.id}`);
        if (!orgConversation) return;

        const children = organizationChildren.get(org.id) ?? [];
        const childMatches = children.filter(matches);
        const orgMatches = matches(orgConversation);
        const shouldIncludeOrg = term ? orgMatches || childMatches.length > 0 : true;
        if (!shouldIncludeOrg) return;

        result.push(orgConversation);

        const shouldShowChildren =
          expandedOrgs.has(org.id) || (term !== '' && childMatches.length > 0);

        if (shouldShowChildren) {
          const childrenToShow = term ? childMatches : children;
          childrenToShow.forEach(child => result.push(child));
        }
      });
    }

    if (includePersonal) {
      const personalCandidates = conversations.filter(
        item =>
          !item.parentOrgId &&
          (item.kind === 'group' || item.kind === 'person')
      );
      const personalMatches = personalCandidates.filter(matches);
      if (personalMatches.length > 0) {
        const personalHeader: Conversation = {
          id: 'personal:space',
          entityId: 'personal:space',
          title: 'Personal Space',
          subtitle: 'Direct messages and personal groups',
          badge: 'Personal',
          kind: 'personal-root',
          depth: 0,
          clickable: false,
          hasChildren: personalMatches.length > 0,
        };
        result.push(personalHeader);
        personalMatches.forEach(item => {
          result.push({ ...item, depth: 1 });
        });
      }
    }

    return result;
  }, [conversationMap, organizationChildren, expandedOrgs, organizations, conversations, scopeFilter, search]);

  const conversationRows = useMemo(() => {
    const rows: React.ReactNode[] = [];
    let lastSection: 'org' | 'personal' | null = null;

    filteredConversations.forEach(conversation => {
      if (conversation.kind === 'organization' && lastSection !== 'org') {
        rows.push(
          <Box key="section-organisations" sx={{ px: 3, pt: 3, pb: 1 }}>
            <Typography sx={{ color: TOKENS.textPrimary, fontWeight: 700, fontSize: 14, letterSpacing: 0.4 }}>
              Organisations
            </Typography>
            <Typography sx={{ color: TOKENS.textSecondary, fontSize: 12 }}>
              Teams, channels, and shared workspaces
            </Typography>
          </Box>
        );
        lastSection = 'org';
      }

      if (conversation.kind === 'personal-root') {
        rows.push(
          <Box key="section-personal" sx={{ px: 3, pt: lastSection === 'org' ? 3 : 2.5, pb: 1 }}>
            <Typography sx={{ color: TOKENS.textPrimary, fontWeight: 700, fontSize: 14, letterSpacing: 0.4 }}>
              Personal Space
            </Typography>
            <Typography sx={{ color: TOKENS.textSecondary, fontSize: 12 }}>
              Direct messages and personal groups
            </Typography>
          </Box>
        );
        lastSection = 'personal';
        return;
      }

      const active = conversation.id === selectedId;
      const isOrg = conversation.kind === 'organization';
      const disabled = conversation.clickable === false;
      const indent = conversation.parentOrgId ? (conversation.depth ?? 1) * 20 + 48 : 28;
      const childCount = organizationChildren.get(conversation.entityId)?.length ?? 0;
      const isExpanded = isOrg && expandedOrgs.has(conversation.entityId);

      rows.push(
        <ListItemButton
          key={conversation.id}
          disabled={disabled}
          disableRipple={disabled}
          selected={active}
          onClick={disabled ? undefined : () => setSelectedId(conversation.id)}
          sx={{
            mx: conversation.parentOrgId ? 1.5 : 1,
            my: 0.5,
            borderRadius: 3,
            bgcolor: active ? alpha(TOKENS.accent, 0.14) : 'transparent',
            '&:hover': { bgcolor: active ? alpha(TOKENS.accent, 0.2) : alpha('#fff', 0.04) },
            px: 2,
            minHeight: 60,
          }}
        >
          <Stack direction="row" alignItems="center" spacing={1.25} sx={{ width: '100%', pl: `${indent}px`, pr: 1 }}>
            <Avatar
              sx={{
                width: 40,
                height: 40,
                bgcolor: alpha(isOrg ? TOKENS.accent : TOKENS.accentSecondary, 0.18),
                color: TOKENS.textPrimary,
                fontSize: isOrg ? 16 : 13,
                fontWeight: 600,
              }}
            >
              {conversation.title.slice(0, 2).toUpperCase()}
            </Avatar>
            <Box sx={{ flexGrow: 1, minWidth: 0 }}>
              <Typography
                sx={{
                  color: TOKENS.textPrimary,
                  fontWeight: isOrg ? 600 : 500,
                  fontSize: 15,
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: 'nowrap',
                }}
              >
                {conversation.title}
              </Typography>
              {conversation.subtitle && (
                <Typography sx={{ color: TOKENS.textSecondary, fontSize: 12, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                  {conversation.subtitle}
                </Typography>
              )}
            </Box>
            {isOrg && childCount > 0 && (
              <IconButton
                size="small"
                onClick={(event) => {
                  event.stopPropagation();
                  handleToggleOrg(conversation.entityId);
                }}
                sx={{ color: TOKENS.textSecondary }}
              >
                {isExpanded ? <ExpandMore fontSize="small" /> : <ChevronRight fontSize="small" />}
              </IconButton>
            )}
            {conversation.updatedAt && (
              <Typography sx={{ color: TOKENS.textSecondary, fontSize: 12, flexShrink: 0 }}>
                {formatRelative(conversation.updatedAt)}
              </Typography>
            )}
          </Stack>
        </ListItemButton>
      );
    });

    return rows;
  }, [filteredConversations, selectedId, organizationChildren, expandedOrgs, handleToggleOrg]);

  const selectedConversation = useMemo(() => {
    const conversation = filteredConversations.find(item => item.id === selectedId);
    if (!conversation || conversation.clickable === false) {
      return null;
    }
    return conversation;
  }, [filteredConversations, selectedId]);

  const viewOptions = useMemo<ViewOption[]>(() => {
    if (!selectedConversation) return [];
    switch (selectedConversation.kind) {
      case 'organization':
        return [
          { key: 'overview', label: 'Overview' },
          { key: 'chat', label: 'Chat' },
          { key: 'threads', label: 'Threads' },
          { key: 'files', label: 'Files' },
          { key: 'storage', label: 'Storage' },
          { key: 'website', label: 'Website' },
        ];
      case 'channel':
        return [
          { key: 'chat', label: 'Chat' },
          { key: 'threads', label: 'Threads' },
          { key: 'files', label: 'Files' },
          { key: 'storage', label: 'Storage' },
          { key: 'website', label: 'Website' },
        ];
      case 'project':
        return [
          { key: 'chat', label: 'Chat' },
          { key: 'files', label: 'Files' },
          { key: 'storage', label: 'Storage' },
          { key: 'website', label: 'Website' },
          { key: 'board', label: 'Board' },
          { key: 'timeline', label: 'Timeline' },
        ];
      case 'group':
        return [
          { key: 'chat', label: 'Chat' },
          { key: 'threads', label: 'Threads' },
          { key: 'files', label: 'Files' },
          { key: 'storage', label: 'Storage' },
          { key: 'website', label: 'Website' },
        ];
      case 'personal-root':
        return [];
      case 'person':
      default:
        return [
          { key: 'chat', label: 'Chat' },
          { key: 'files', label: 'Files' },
          { key: 'storage', label: 'Storage' },
          { key: 'website', label: 'Website' },
        ];
    }
  }, [selectedConversation]);

  useEffect(() => {
    if (selectedId && filteredConversations.some(item => item.id === selectedId)) {
      const current = filteredConversations.find(item => item.id === selectedId);
      if (current?.clickable === false) {
        const fallback = filteredConversations.find(item => item.clickable !== false);
        if (fallback) {
          setSelectedId(fallback.id);
        }
      }
      return;
    }

    const firstSelectable = filteredConversations.find(item => item.clickable !== false);
    if (firstSelectable) {
      setSelectedId(firstSelectable.id);
    }
  }, [filteredConversations, selectedId]);

  useEffect(() => {
    if (!selectedConversation) return;
    const preferred = viewOptions[0]?.key ?? 'chat';
    setActiveView(preferred);
    setComposerValue('');
  }, [selectedConversation?.id]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    if (!viewOptions.some(option => option.key === activeView)) {
      setActiveView(viewOptions[0]?.key ?? 'chat');
    }
  }, [viewOptions, activeView]);

  useEffect(() => {
    if (activeView !== 'storage') {
      setStorageTab('private');
    }
  }, [activeView]);

  useEffect(() => {
    if (selectedConversation?.parentOrgId) {
      setExpandedOrgs(prev => {
        if (prev.has(selectedConversation.parentOrgId!)) {
          return prev;
        }
        const next = new Set(prev);
        next.add(selectedConversation.parentOrgId!);
        return next;
      });
    }
  }, [selectedConversation?.parentOrgId]);

  useEffect(() => {
    if (!search.trim()) {
      return;
    }
    setExpandedOrgs(prev => {
      let changed = false;
      const next = new Set(prev);
      organizations.forEach(org => {
        if (!next.has(org.id)) {
          next.add(org.id);
          changed = true;
        }
      });
      return changed ? next : prev;
    });
  }, [search, organizations]);

  const hasFilesOption = viewOptions.some(option => option.key === 'files');
  const hasWebsiteOption = viewOptions.some(option => option.key === 'website');
  const hasStorageOption = viewOptions.some(option => option.key === 'storage');
  const composerEnabled = activeView === 'chat';
  const isScrollableView = activeView === 'chat' || activeView === 'threads' || activeView === 'overview';

  useEffect(() => {
    const loadMessages = async () => {
      if (!selectedConversation || !peerId) return;
      const crdtMessages = await messageService.getMessages(selectedConversation.entityId);
      setMessages(crdtMessages.map(msg => convertMessage(msg, peerId)));
    };
    void loadMessages();
  }, [messageService, peerId, selectedConversation]);

  useEffect(() => {
    if (authState.loading || authState.isAuthenticated) return;
    (async () => {
      const firstLaunch = await isFirstLaunch();
      if (firstLaunch) {
        setShowWelcome(true);
      } else {
        setShowIdentityPicker(true);
      }
    })();
  }, [authState.loading, authState.isAuthenticated, isFirstLaunch]);

  useEffect(() => {
    if (!identityMenuAnchor) {
      setRecentIdentities([]);
      return;
    }
    getRecentIdentities().then(setRecentIdentities);
  }, [identityMenuAnchor, getRecentIdentities]);

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        setCommandOpen(value => !value);
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  const handleSendMessage = useCallback(async () => {
    if (!selectedConversation || !peerId || !composerValue.trim()) return;
    try {
      const crdtMessage = await messageService.sendMessage(
        selectedConversation.entityId,
        selectedConversation.kind === 'person'
          ? 'person'
          : selectedConversation.kind === 'group'
          ? 'group'
          : selectedConversation.kind === 'project'
          ? 'project'
          : selectedConversation.kind === 'channel'
          ? 'channel'
          : 'organisation',
        composerValue.trim(),
        authState.user?.name ?? 'You',
      );
      setComposerValue('');
      setMessages(prev => [...prev, convertMessage(crdtMessage, peerId)]);
    } catch (error) {
      console.error('Failed to send message', error);
    }
  }, [selectedConversation, peerId, composerValue, authState.user, messageService]);

  const handleCreateEntity = async (type: 'organization' | 'channel' | 'project' | 'group' | 'contact') => {
    switch (type) {
      case 'organization': {
        const result = await createOrganization({ displayName: 'New Organisation' });
        setSelectedId(`org:${result.entityId}`);
        break;
      }
      case 'channel': {
        if (!selectedConversation?.parentOrgId && selectedConversation?.kind !== 'organization') {
          alert('Select an organisation to create a channel.');
          break;
        }
        const organizationId =
          selectedConversation?.kind === 'organization'
            ? selectedConversation.entityId
            : selectedConversation?.parentOrgId ?? '';
        if (!organizationId) break;
        const result = await createChannel({
          organizationId,
          displayName: `channel-${Math.random().toString(36).slice(2, 6)}`,
        });
        setSelectedId(`channel:${result.entityId}`);
        break;
      }
      case 'project': {
        if (!selectedConversation?.parentOrgId && selectedConversation?.kind !== 'organization') {
          alert('Select an organisation to create a project.');
          break;
        }
        const organizationId =
          selectedConversation?.kind === 'organization'
            ? selectedConversation.entityId
            : selectedConversation?.parentOrgId ?? '';
        if (!organizationId) break;
        const result = await createProject({
          organizationId,
          displayName: `Project ${new Date().getFullYear()}`,
        });
        setSelectedId(`project:${result.entityId}`);
        break;
      }
      case 'group': {
        const orgId =
          selectedConversation?.kind === 'organization' ? selectedConversation.entityId : undefined;
        const result = await createGroup({
          organizationId: orgId,
          displayName: 'New Group',
        });
        setSelectedId(`group:${result.entityId}`);
        break;
      }
      case 'contact': {
        const result = await createContact({ displayName: 'New Contact' });
        setSelectedId(`person:${result.entityId}`);
        break;
      }
    }
    setCreateMenuAnchor(null);
  };

  const renderMessages = () => (
    <Stack spacing={2} sx={{ px: 3, py: 3 }}>
      {messages.length === 0 && (
        <Paper
          sx={{
            p: 3,
            borderRadius: 4,
            bgcolor: TOKENS.surface2,
            border: `1px solid ${alpha('#fff', 0.05)}`,
            textAlign: 'center',
          }}
        >
          <Typography sx={{ color: TOKENS.textSecondary }}>
            Start the conversation with a secure, locally stored message. Everything syncs peer-to-peer when you're online.
          </Typography>
        </Paper>
      )}
      {messages.map(message => (
        <Stack
          key={message.id}
          alignItems={message.own ? 'flex-end' : 'flex-start'}
          spacing={0.5}
        >
          {!message.own && (
            <Typography sx={{ color: TOKENS.textSecondary, fontSize: 12, fontWeight: 600 }}>
              {message.author}
            </Typography>
          )}
          <Paper
            sx={{
              px: 2,
              py: 1.5,
              borderRadius: 3,
              maxWidth: '70%',
              bgcolor: message.own ? alpha(TOKENS.accent, 0.18) : TOKENS.surface2,
              color: TOKENS.textPrimary,
              boxShadow: 'none',
            }}
          >
            <Typography sx={{ whiteSpace: 'pre-wrap' }}>{message.text}</Typography>
          </Paper>
          <Stack direction="row" spacing={1} alignItems="center">
            <Typography sx={{ color: TOKENS.textSecondary, fontSize: 12 }}>
              {formatTimestamp(message.timestamp)}
            </Typography>
            {message.own && (
              <Typography sx={{ color: TOKENS.textSecondary, fontSize: 12 }}>
                {message.status === 'read'
                  ? 'Read'
                  : message.status === 'delivered'
                  ? 'Delivered'
                  : 'Sending'}
              </Typography>
            )}
          </Stack>
        </Stack>
      ))}
    </Stack>
  );

  const renderThreadsView = () => (
    <Box sx={{ p: 3 }}>
      <Typography sx={{ color: TOKENS.textSecondary, mb: 2 }}>
        Threads keep deep-dive discussions tidy. Reply to a message to create a thread that stays linked to the original conversation.
      </Typography>
      <Stack spacing={2}>
        {[1, 2, 3].map(index => (
          <Paper
            key={index}
            sx={{
              p: 2,
              borderRadius: 3,
              bgcolor: TOKENS.surface2,
              border: `1px solid ${alpha('#fff', 0.04)}`,
            }}
          >
            <Stack direction="row" alignItems="center" spacing={1.5}>
              <Avatar
                sx={{
                  width: 36,
                  height: 36,
                  bgcolor: alpha(TOKENS.accentSecondary, 0.2),
                  color: TOKENS.textPrimary,
                  fontSize: 14,
                }}
              >
                T{index}
              </Avatar>
              <Box sx={{ flexGrow: 1 }}>
                <Typography sx={{ fontWeight: 600 }}>Design Review Thread #{index}</Typography>
                <Typography sx={{ color: TOKENS.textSecondary, fontSize: 13 }}>
                  Last reply 2h ago • 6 messages
                </Typography>
              </Box>
              <Chip
                size="small"
                label="Active"
                sx={{ bgcolor: alpha(TOKENS.accent, 0.12), color: TOKENS.accent }}
              />
            </Stack>
            <Typography sx={{ color: TOKENS.textSecondary, mt: 1.5 }}>
              Capture decision points, share assets, and loop in stakeholders without cluttering the main channel.
            </Typography>
          </Paper>
        ))}
      </Stack>
    </Box>
  );

  const renderFilesView = (mode: 'files' | 'web') => {
    if (!selectedConversation) return null;
    return (
      <Box sx={{ flexGrow: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        <EntityDocumentWorkspace
          entityId={selectedConversation.entityId}
          entityName={selectedConversation.title}
          storageMode={mode}
          permissions={['read', 'write']}
        />
      </Box>
    );
  };

  const renderWebsiteView = () => (
    <Box sx={{ p: 3, display: 'grid', gap: 2, height: '100%', overflow: 'hidden' }}>
      <Typography sx={{ color: TOKENS.textSecondary }}>
        The website workspace publishes markdown content directly to the distributed web using this entity’s four-word identity. Draft pages in the editor, then set a document as the home entry to go live instantly.
      </Typography>
      <Stack direction="row" spacing={1}>
        <Button
          variant="contained"
          startIcon={<FolderRounded />}
          onClick={() => setActiveView('files')}
          sx={{ bgcolor: TOKENS.accent, color: '#041218', '&:hover': { bgcolor: alpha(TOKENS.accent, 0.85) } }}
        >
          Open Files
        </Button>
        <Button
          variant="outlined"
          startIcon={<WebRounded />}
          sx={{ borderColor: alpha('#fff', 0.12), color: TOKENS.textSecondary }}
        >
          Preview Website
        </Button>
      </Stack>
      <Box sx={{ flexGrow: 1, minHeight: 220, overflow: 'hidden' }}>
        {renderFilesView('web')}
      </Box>
    </Box>
  );

  const renderPlaceholderView = (title: string, description: string) => (
    <Box sx={{ p: 3 }}>
      <Typography variant="h6" sx={{ color: TOKENS.textPrimary, mb: 1 }}>
        {title}
      </Typography>
      <Typography sx={{ color: TOKENS.textSecondary }}>{description}</Typography>
    </Box>
  );

  const renderStorageView = () => {
    if (!selectedConversation) {
      return renderPlaceholderView('Storage not available', 'Select an entity to manage its virtual disks.');
    }

    const storageDescription =
      storageTab === 'public'
        ? 'Publish read-only content to the distributed web. Everything is signed with this identity’s keys.'
        : storageTab === 'shared'
        ? 'Shared vaults are visible to approved collaborators across synced devices.'
        : 'Private vault encrypts files locally; only you can decrypt without sharing.';

    const workspaceMode: DocumentStorageMode = storageTab === 'public' ? 'web' : 'files';

    return (
      <Box sx={{ p: 3, display: 'grid', gap: 2, height: '100%', overflow: 'hidden' }}>
        <Stack direction="row" spacing={1}>
          <Chip
            label="Private"
            color={storageTab === 'private' ? 'success' : 'default'}
            onClick={() => setStorageTab('private')}
            sx={{ bgcolor: storageTab === 'private' ? alpha(TOKENS.accent, 0.2) : 'transparent', color: storageTab === 'private' ? TOKENS.accent : TOKENS.textSecondary }}
          />
          <Chip
            label="Shared"
            color={storageTab === 'shared' ? 'success' : 'default'}
            onClick={() => setStorageTab('shared')}
            sx={{ bgcolor: storageTab === 'shared' ? alpha(TOKENS.accent, 0.2) : 'transparent', color: storageTab === 'shared' ? TOKENS.accent : TOKENS.textSecondary }}
          />
          <Chip
            label="Public"
            color={storageTab === 'public' ? 'success' : 'default'}
            onClick={() => setStorageTab('public')}
            sx={{ bgcolor: storageTab === 'public' ? alpha(TOKENS.accent, 0.2) : 'transparent', color: storageTab === 'public' ? TOKENS.accent : TOKENS.textSecondary }}
          />
        </Stack>
        <Paper sx={{ p: 3, borderRadius: 3, bgcolor: TOKENS.surface2, border: `1px solid ${alpha('#fff', 0.05)}` }}>
          <Typography sx={{ fontWeight: 600, mb: 1 }}>Vault Overview</Typography>
          <Typography sx={{ color: TOKENS.textSecondary }}>{storageDescription}</Typography>
          <Typography sx={{ color: TOKENS.textSecondary, fontSize: 12, mt: 2 }}>
            Identity: {selectedConversation.fourWords ? fourWordsToDisplay(selectedConversation.fourWords) : 'Not assigned'}
          </Typography>
        </Paper>
        <Box sx={{ flexGrow: 1, minHeight: 220, overflow: 'hidden' }}>
          <EntityDocumentWorkspace
            key={`${selectedConversation.entityId}-${storageTab}`}
            entityId={selectedConversation.entityId}
            entityName={selectedConversation.title}
            storageMode={workspaceMode}
            permissions={['read', 'write']}
          />
        </Box>
      </Box>
    );
  };

  const renderOrganizationOverview = () => {
    if (!activeOrg) {
      return renderPlaceholderView(
        'Select an organisation',
        'Choose an organisation space to see members, channels, projects, and storage insights.'
      );
    }

    const members = (activeOrg.users ?? []).slice(0, 6);
    const channels = activeOrg.channels ?? [];
    const projects = activeOrg.projects ?? [];
    const groups = activeOrg.groups ?? [];

    return (
      <Box
        sx={{
          flexGrow: 1,
          px: 3,
          py: 3,
          overflowY: 'auto',
          display: 'grid',
          gap: 3,
          gridTemplateColumns: { xs: '1fr', md: 'repeat(2, minmax(0, 1fr))' },
        }}
      >
        <Paper sx={{ p: 3, borderRadius: 3, bgcolor: TOKENS.surface2, border: `1px solid ${alpha('#fff', 0.05)}` }}>
          <Typography sx={{ fontWeight: 600, mb: 1 }}>Members</Typography>
          <Typography sx={{ color: TOKENS.textSecondary, mb: 2 }}>
            {members.length} of {(activeOrg.users ?? []).length} team members shown.
          </Typography>
          <Stack spacing={1.25}>
            {members.map(member => (
              <Stack key={member.id} direction="row" spacing={1.5} alignItems="center">
                <Avatar sx={{ width: 36, height: 36, bgcolor: alpha(TOKENS.accentSecondary, 0.2) }}>
                  {member.name.slice(0, 2).toUpperCase()}
                </Avatar>
                <Box sx={{ flexGrow: 1 }}>
                  <Typography sx={{ fontWeight: 600 }}>{member.name}</Typography>
                  <Typography sx={{ color: TOKENS.textSecondary, fontSize: 13 }}>
                    {member.role ?? 'Member'}
                  </Typography>
                </Box>
                <Chip
                  size="small"
                  label={member.permissions?.includes('all') ? 'Owner' : 'Member'}
                  sx={{ bgcolor: alpha(TOKENS.accent, 0.12), color: TOKENS.accent }}
                />
              </Stack>
            ))}
          </Stack>
        </Paper>

        <Paper sx={{ p: 3, borderRadius: 3, bgcolor: TOKENS.surface2, border: `1px solid ${alpha('#fff', 0.05)}` }}>
          <Typography sx={{ fontWeight: 600, mb: 1 }}>Projects</Typography>
          <Typography sx={{ color: TOKENS.textSecondary, mb: 2 }}>
            Active initiatives and delivery milestones.
          </Typography>
          <Stack spacing={1.25}>
            {projects.length === 0 && (
              <Typography sx={{ color: TOKENS.textSecondary }}>
                No projects yet. Create one to track roadmap milestones.
              </Typography>
            )}
            {projects.slice(0, 5).map(project => (
              <Box
                key={project.id}
                sx={{
                  borderRadius: 2,
                  p: 2,
                  bgcolor: alpha('#fff', 0.03),
                  border: `1px solid ${alpha('#fff', 0.04)}`,
                }}
              >
                <Typography sx={{ fontWeight: 600 }}>{project.name}</Typography>
                <Typography sx={{ color: TOKENS.textSecondary, fontSize: 13 }}>
                  {project.description ?? 'No description provided.'}
                </Typography>
              </Box>
            ))}
          </Stack>
        </Paper>

        <Paper sx={{ p: 3, borderRadius: 3, bgcolor: TOKENS.surface2, border: `1px solid ${alpha('#fff', 0.05)}` }}>
          <Typography sx={{ fontWeight: 600, mb: 1 }}>Channels</Typography>
          <Typography sx={{ color: TOKENS.textSecondary, mb: 2 }}>
            Team spaces for real-time collaboration and announcements.
          </Typography>
          <Stack spacing={1}>
            {channels.length === 0 && (
              <Typography sx={{ color: TOKENS.textSecondary }}>
                Create a channel to begin topic-focused collaboration.
              </Typography>
            )}
            {channels.slice(0, 6).map(channel => (
              <Stack
                key={channel.id}
                direction="row"
                spacing={1.25}
                alignItems="center"
                sx={{ borderRadius: 2, p: 1.5, bgcolor: alpha('#fff', 0.03) }}
              >
                <Chip
                  label={`#${channel.name}`}
                  size="small"
                  sx={{ bgcolor: alpha(TOKENS.accentSecondary, 0.16), color: TOKENS.textPrimary }}
                />
                <Typography sx={{ color: TOKENS.textSecondary, fontSize: 13 }}>
                  {(channel.members ?? []).length} members
                </Typography>
              </Stack>
            ))}
          </Stack>
        </Paper>

        <Paper sx={{ p: 3, borderRadius: 3, bgcolor: TOKENS.surface2, border: `1px solid ${alpha('#fff', 0.05)}` }}>
          <Typography sx={{ fontWeight: 600, mb: 1 }}>Storage Overview</Typography>
          <Typography sx={{ color: TOKENS.textSecondary, mb: 2 }}>
            Private, shared, and published content usage across the organisation.
          </Typography>
          <Stack spacing={1.25}>
            {[
              { label: 'Private Disk', detail: 'Encrypted files for members only', usage: '512 MB used' },
              {
                label: 'Shared Spaces',
                detail: `${groups.length} shared groups`,
                usage: `${groups.reduce((acc, group) => acc + (group.members?.length ?? 0), 0)} shared collaborators`,
              },
              { label: 'Published Websites', detail: 'Identity-bound websites', usage: `${projects.length} published entries` },
            ].map(item => (
              <Box
                key={item.label}
                sx={{ borderRadius: 2, p: 1.5, bgcolor: alpha('#fff', 0.03), border: `1px solid ${alpha('#fff', 0.04)}` }}
              >
                <Typography sx={{ fontWeight: 600 }}>{item.label}</Typography>
                <Typography sx={{ color: TOKENS.textSecondary, fontSize: 13 }}>{item.detail}</Typography>
                <Typography sx={{ color: TOKENS.accent, fontSize: 12, mt: 0.5 }}>{item.usage}</Typography>
              </Box>
            ))}
          </Stack>
        </Paper>
      </Box>
    );
  };

  if (authState.loading) {
    return (
      <Box
        sx={{
          height: '100vh',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          bgcolor: TOKENS.surface0,
          color: TOKENS.textPrimary,
        }}
      >
        <Typography variant="h5">Preparing secure local workspace...</Typography>
      </Box>
    );
  }

  if (!authState.isAuthenticated) {
    return (
      <Box
        sx={{
          minHeight: '100vh',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          bgcolor: TOKENS.surface0,
          color: TOKENS.textPrimary,
          px: 3,
        }}
      >
        <Stack spacing={4} alignItems="center" maxWidth={520} width="100%">
          <Typography variant="h4" sx={{ textAlign: 'center' }}>
            Sign in to Communitas
          </Typography>
          {showWelcome && (
            <FirstLaunchWelcome open={showWelcome} onClose={() => setShowWelcome(false)} />
          )}
          {showUnifiedFlow && (
            <UnifiedAuthFlow
              initialMode="register"
              onSuccess={() => setShowUnifiedFlow(false)}
              onCancel={() => setShowUnifiedFlow(false)}
            />
          )}
          {showIdentityPicker && (
            <IdentityPicker
              onSelectIdentity={async (fourWords, usePasskey) => {
                if (usePasskey) {
                  const ok = await signInWithPasskey(fourWords);
                  if (!ok) alert('Passkey authentication failed.');
                } else {
                  await login(fourWords, fourWords);
                }
                setShowIdentityPicker(false);
              }}
              onCreateNew={() => setShowUnifiedFlow(true)}
              onManualEntry={(fourWords) => login(fourWords, fourWords)}
            />
          )}
        </Stack>
      </Box>
    );
  }

  return (
    <Box sx={{ display: 'flex', height: '100vh', bgcolor: TOKENS.surface0, color: TOKENS.textPrimary }}>
      <ActiveCallModal
        state={callState}
        conversation={selectedConversation}
        onClose={() => setCallState(prev => ({ ...prev, open: false }))}
      />

      <CommandPalette
        open={commandOpen}
        onClose={() => setCommandOpen(false)}
        conversations={conversations}
        onSelect={(conversation) => setSelectedId(conversation.id)}
      />

      <ConversationDetailDrawer
        open={drawerOpen}
        onClose={() => setDrawerOpen(false)}
        conversation={selectedConversation}
      />

      <Box
        sx={{
          width: navExpanded ? 84 : 36,
          bgcolor: TOKENS.surface1,
          borderRight: `1px solid ${alpha('#fff', 0.06)}`,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          py: 2,
          gap: 2,
          transition: 'width 180ms ease',
        }}
      >
        <Tooltip title="Back to identity menu" placement="right">
          <IconButton
            onClick={(event) => setIdentityMenuAnchor(event.currentTarget)}
            sx={{ color: TOKENS.textPrimary }}
          >
            <Avatar sx={{ width: 40, height: 40 }}>
              {authState.user?.name.slice(0, 2).toUpperCase()}
            </Avatar>
          </IconButton>
        </Tooltip>
        <Tooltip title="Home" placement="right">
          <IconButton sx={{ color: TOKENS.textSecondary, opacity: navExpanded ? 1 : 0.9 }}>
            <AppsRounded />
          </IconButton>
        </Tooltip>
        <Tooltip title="Messaging" placement="right">
          <IconButton sx={{ color: TOKENS.accent, opacity: navExpanded ? 1 : 0.9 }}>
            <ChatBubbleOutlineRounded />
          </IconButton>
        </Tooltip>
        <Tooltip title="Contacts" placement="right">
          <IconButton sx={{ color: TOKENS.textSecondary, opacity: navExpanded ? 1 : 0.9 }}>
            <PeopleAltRounded />
          </IconButton>
        </Tooltip>
        <Tooltip title="Storage" placement="right">
          <IconButton sx={{ color: TOKENS.textSecondary, opacity: navExpanded ? 1 : 0.9 }}>
            <FolderRounded />
          </IconButton>
        </Tooltip>
        <Box sx={{ flexGrow: 1 }} />
        <Tooltip title={navExpanded ? 'Collapse navigation' : 'Expand navigation'} placement="right">
          <IconButton onClick={() => setNavExpanded(prev => !prev)} sx={{ color: TOKENS.textSecondary }}>
            {navExpanded ? <ChevronLeft /> : <ChevronRight />}
          </IconButton>
        </Tooltip>
      </Box>

      <Box
        sx={{
          width: 320,
          bgcolor: TOKENS.surface1,
          borderRight: `1px solid ${alpha('#fff', 0.06)}`,
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        <AppBar
          elevation={0}
          position="static"
          sx={{ bgcolor: 'transparent', borderBottom: `1px solid ${alpha('#fff', 0.06)}`, px: 2 }}
        >
          <Stack direction="row" spacing={1} alignItems="center" justifyContent="space-between" sx={{ py: 1.5 }}>
            <Typography variant="h6">Conversations</Typography>
            <IconButton onClick={(event) => setCreateMenuAnchor(event.currentTarget)} sx={{ color: TOKENS.accent }}>
              <CreateRounded />
            </IconButton>
          </Stack>
          <Stack direction="row" spacing={1} sx={{ pb: 1.5 }}>
            <Chip
              size="small"
              label="All"
              onClick={() => setScopeFilter('all')}
              color={scopeFilter === 'all' ? 'success' : 'default'}
              sx={{ bgcolor: scopeFilter === 'all' ? alpha(TOKENS.accent, 0.2) : 'transparent', color: TOKENS.textSecondary }}
            />
            <Chip
              size="small"
              label="Org"
              onClick={() => setScopeFilter('org')}
              color={scopeFilter === 'org' ? 'success' : 'default'}
              sx={{ bgcolor: scopeFilter === 'org' ? alpha(TOKENS.accent, 0.2) : 'transparent', color: TOKENS.textSecondary }}
            />
            <Chip
              size="small"
              label="Personal"
              onClick={() => setScopeFilter('personal')}
              color={scopeFilter === 'personal' ? 'success' : 'default'}
              sx={{ bgcolor: scopeFilter === 'personal' ? alpha(TOKENS.accent, 0.2) : 'transparent', color: TOKENS.textSecondary }}
            />
          </Stack>
          <Paper
            sx={{
              px: 1.5,
              py: 1,
              mb: 1.5,
              display: 'flex',
              alignItems: 'center',
              borderRadius: 3,
              bgcolor: TOKENS.surface2,
            }}
          >
            <SearchRounded sx={{ color: TOKENS.textSecondary, mr: 1 }} />
            <InputBase
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="Search conversations"
              sx={{ color: TOKENS.textPrimary, fontSize: 14 }}
            />
          </Paper>
        </AppBar>
        <Box sx={{ flexGrow: 1, overflowY: 'auto' }}>
          {filteredConversations.length === 0 ? (
            <ConversationListEmpty onCreate={(anchor) => setCreateMenuAnchor(anchor)} />
          ) : (
            <List disablePadding>{conversationRows}</List>
          )}
        </Box>
      </Box>

      <Box sx={{ flexGrow: 1, display: 'flex', flexDirection: 'column' }}>
        <AppBar
          elevation={0}
          position="static"
          sx={{ bgcolor: TOKENS.surface1, borderBottom: `1px solid ${alpha('#fff', 0.06)}` }}
        >
          <Stack direction="row" spacing={2} alignItems="center" sx={{ px: 3, py: 1.5 }}>
            <Avatar sx={{ width: 44, height: 44 }}>
              {selectedConversation?.title.slice(0, 2).toUpperCase()}
            </Avatar>
            <Box sx={{ flexGrow: 1 }}>
              <Typography variant="h6">{selectedConversation?.title ?? 'Select a conversation'}</Typography>
              <Typography variant="body2" sx={{ color: TOKENS.textSecondary }}>
                {selectedConversation?.subtitle ?? 'Messages, files, and calls stay in sync across devices.'}
              </Typography>
            </Box>
            <Stack direction="row" spacing={1} alignItems="center">
              {hasFilesOption && (
                <Button
                  variant={activeView === 'files' ? 'contained' : 'outlined'}
                  onClick={() => setActiveView('files')}
                  startIcon={<FolderRounded />}
                  sx={{
                    bgcolor: activeView === 'files' ? TOKENS.accent : 'transparent',
                    color: activeView === 'files' ? '#041218' : TOKENS.textSecondary,
                    borderColor: alpha('#fff', 0.12),
                    '&:hover': {
                      bgcolor: activeView === 'files' ? alpha(TOKENS.accent, 0.85) : alpha('#fff', 0.08),
                    },
                  }}
                >
                  Files
                </Button>
              )}
              {hasWebsiteOption && (
                <Button
                  variant={activeView === 'website' ? 'contained' : 'outlined'}
                  onClick={() => setActiveView('website')}
                  startIcon={<WebRounded />}
                  sx={{
                    bgcolor: activeView === 'website' ? TOKENS.accentSecondary : 'transparent',
                    color: activeView === 'website' ? '#041218' : TOKENS.textSecondary,
                    borderColor: alpha('#fff', 0.12),
                    '&:hover': {
                      bgcolor: activeView === 'website' ? alpha(TOKENS.accentSecondary, 0.85) : alpha('#fff', 0.08),
                    },
                  }}
                >
                  Website
                </Button>
              )}
              {hasStorageOption && (
                <Button
                  variant={activeView === 'storage' ? 'contained' : 'outlined'}
                  onClick={() => setActiveView('storage')}
                  startIcon={<StorageOutlined />}
                  sx={{
                    bgcolor: activeView === 'storage' ? TOKENS.accentSecondary : 'transparent',
                    color: activeView === 'storage' ? '#041218' : TOKENS.textSecondary,
                    borderColor: alpha('#fff', 0.12),
                    '&:hover': {
                      bgcolor: activeView === 'storage' ? alpha(TOKENS.accentSecondary, 0.85) : alpha('#fff', 0.08),
                    },
                  }}
                >
                  Storage
                </Button>
              )}
              <Tooltip title="Start voice call">
                <IconButton
                  sx={{ color: TOKENS.textSecondary }}
                  onClick={() =>
                    setCallState({ open: true, type: 'audio', startedAt: Date.now() })
                  }
                >
                  <CallRounded />
                </IconButton>
              </Tooltip>
              <Tooltip title="Start video call">
                <IconButton
                  sx={{ color: TOKENS.textSecondary }}
                  onClick={() =>
                    setCallState({ open: true, type: 'video', startedAt: Date.now() })
                  }
                >
                  <VideoCallRounded />
                </IconButton>
              </Tooltip>
              <Tooltip title="Open details">
                <IconButton sx={{ color: TOKENS.textSecondary }} onClick={() => setDrawerOpen(true)}>
                  <InfoRounded />
                </IconButton>
              </Tooltip>
            </Stack>
          </Stack>
        </AppBar>
        {selectedConversation && viewOptions.length > 0 && (
          <Stack
            direction="row"
            spacing={1}
            sx={{
              px: 3,
              py: 1.5,
              borderBottom: `1px solid ${alpha('#fff', 0.04)}`,
              bgcolor: TOKENS.surface1,
            }}
          >
            {viewOptions.map(option => (
              <Chip
                key={option.key}
                label={option.label}
                onClick={() => setActiveView(option.key)}
                color={activeView === option.key ? 'success' : 'default'}
                sx={{
                  bgcolor: activeView === option.key ? alpha(TOKENS.accent, 0.2) : 'transparent',
                  color: activeView === option.key ? TOKENS.accent : TOKENS.textSecondary,
                }}
              />
            ))}
          </Stack>
        )}
        <Box sx={{ flexGrow: 1, display: 'flex', flexDirection: 'column', overflowY: isScrollableView ? 'auto' : 'hidden' }}>
          {selectedConversation ? (
            activeView === 'chat' ? (
              renderMessages()
            ) : activeView === 'threads' ? (
              renderThreadsView()
            ) : activeView === 'files' ? (
              renderFilesView('files')
            ) : activeView === 'storage' ? (
              renderStorageView()
            ) : activeView === 'website' ? (
              renderWebsiteView()
            ) : activeView === 'overview' ? (
              renderOrganizationOverview()
            ) : activeView === 'board' ? (
              renderPlaceholderView('Kanban board', 'Organise tasks across swimlanes, assign owners, and track work-in-progress limits.')
            ) : activeView === 'timeline' ? (
              renderPlaceholderView('Project timeline', 'Visualise milestones, dependencies, and delivery checkpoints for the team.')
            ) : (
              renderPlaceholderView('Coming soon', 'This workspace view is being finalised.')
            )
          ) : (
            renderPlaceholderView('Select a conversation', 'Choose a channel, project, or contact to get started.')
          )}
        </Box>
        {composerEnabled && (
          <Box sx={{ px: 3, py: 2, borderTop: `1px solid ${alpha('#fff', 0.06)}`, bgcolor: TOKENS.surface1 }}>
            <Paper
              component="form"
              onSubmit={(event) => {
                event.preventDefault();
                void handleSendMessage();
              }}
              sx={{
                display: 'flex',
                alignItems: 'center',
                px: 2,
                py: 1,
                borderRadius: 4,
                bgcolor: TOKENS.surface2,
              }}
            >
              <InputBase
                multiline
                maxRows={4}
                placeholder="Write a message..."
                value={composerValue}
                onChange={(event) => setComposerValue(event.target.value)}
                sx={{ flexGrow: 1, color: TOKENS.textPrimary, pr: 2 }}
              />
              <Divider orientation="vertical" flexItem sx={{ mx: 1, borderColor: alpha('#fff', 0.08) }} />
              <Button
                type="submit"
                variant="contained"
                startIcon={<SendRounded />}
                disabled={!composerValue.trim()}
                sx={{
                  bgcolor: TOKENS.accent,
                  color: '#041218',
                  '&:hover': { bgcolor: alpha(TOKENS.accent, 0.85) },
                }}
              >
                Send
              </Button>
            </Paper>
          </Box>
        )}
      </Box>

      <Menu
        anchorEl={identityMenuAnchor}
        open={Boolean(identityMenuAnchor)}
        onClose={() => setIdentityMenuAnchor(null)}
        PaperProps={{
          sx: {
            bgcolor: TOKENS.surface2,
            color: TOKENS.textPrimary,
            minWidth: 280,
          },
        }}
      >
        <MenuItem disabled>
          <Stack spacing={0.5}>
            <Typography sx={{ fontWeight: 600 }}>{authState.user?.name}</Typography>
            <Typography sx={{ color: TOKENS.textSecondary, fontSize: 12 }}>
              {authState.user?.fourWordAddress
                ? fourWordsToDisplay(authState.user.fourWordAddress)
                : 'No identity'}
            </Typography>
          </Stack>
        </MenuItem>
        <Divider sx={{ borderColor: alpha('#fff', 0.08) }} />
        {recentIdentities.map(identity => (
          <MenuItem
            key={identity.four_words}
            onClick={() => {
              setIdentityMenuAnchor(null);
              signInWithPasskey(identity.four_words).catch(() => {
                login(identity.four_words, identity.four_words);
              });
            }}
          >
            <Stack spacing={0.25}>
              <Typography>{identity.display_name}</Typography>
              <Typography sx={{ color: TOKENS.textSecondary, fontSize: 12 }}>
                {fourWordsToDisplay(identity.four_words)}
              </Typography>
            </Stack>
          </MenuItem>
        ))}
        <MenuItem
          onClick={() => {
            setIdentityMenuAnchor(null);
            setShowIdentityPicker(true);
          }}
        >
          Switch identity
        </MenuItem>
      </Menu>

      <Menu
        anchorEl={createMenuAnchor}
        open={Boolean(createMenuAnchor)}
        onClose={() => setCreateMenuAnchor(null)}
        PaperProps={{
          sx: { bgcolor: TOKENS.surface2, color: TOKENS.textPrimary, minWidth: 220 },
        }}
      >
        <MenuItem onClick={() => handleCreateEntity('organization')}>
          <ListItemIcon>
            <AppsRounded sx={{ color: TOKENS.textSecondary }} />
          </ListItemIcon>
          <ListItemText primary="Organisation" secondary="Set up a new workspace" />
        </MenuItem>
        <MenuItem onClick={() => handleCreateEntity('channel')}>
          <ListItemIcon>
            <ChatBubbleOutlineRounded sx={{ color: TOKENS.textSecondary }} />
          </ListItemIcon>
          <ListItemText primary="Channel" secondary="Dedicated topic thread" />
        </MenuItem>
        <MenuItem onClick={() => handleCreateEntity('project')}>
          <ListItemIcon>
            <FolderRounded sx={{ color: TOKENS.textSecondary }} />
          </ListItemIcon>
          <ListItemText primary="Project" secondary="Track deliverables" />
        </MenuItem>
        <MenuItem onClick={() => handleCreateEntity('group')}>
          <ListItemIcon>
            <PeopleAltRounded sx={{ color: TOKENS.textSecondary }} />
          </ListItemIcon>
          <ListItemText primary="Group" secondary="Multi-person collaboration" />
        </MenuItem>
        <MenuItem onClick={() => handleCreateEntity('contact')}>
          <ListItemIcon>
            <PersonRounded sx={{ color: TOKENS.textSecondary }} />
          </ListItemIcon>
          <ListItemText primary="Contact" secondary="Direct encrypted chat" />
        </MenuItem>
      </Menu>

      {showIdentityPicker && (
        <Modal open onClose={() => setShowIdentityPicker(false)}>
          <Box
            sx={{
              minHeight: '100vh',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              bgcolor: alpha('#000', 0.65),
              p: 2,
            }}
          >
            <Paper
              sx={{
                width: 'min(560px, 90vw)',
                maxHeight: '90vh',
                overflowY: 'auto',
                borderRadius: 4,
                bgcolor: TOKENS.surface1,
                p: 3,
              }}
            >
              <IdentityPicker
                onSelectIdentity={async (fourWords, usePasskey) => {
                  if (usePasskey) {
                    const ok = await signInWithPasskey(fourWords);
                    if (!ok) alert('Passkey authentication failed.');
                  } else {
                    await login(fourWords, fourWords);
                  }
                  setShowIdentityPicker(false);
                }}
                onCreateNew={() => {
                  setShowIdentityPicker(false);
                  setShowUnifiedFlow(true);
                }}
                onManualEntry={(fourWords) => {
                  void login(fourWords, fourWords);
                  setShowIdentityPicker(false);
                }}
              />
            </Paper>
          </Box>
        </Modal>
      )}
      {showUnifiedFlow && (
        <Modal open onClose={() => setShowUnifiedFlow(false)}>
          <Box
            sx={{
              minHeight: '100vh',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              bgcolor: alpha('#000', 0.65),
              p: 2,
            }}
          >
            <Paper
              sx={{
                width: 'min(600px, 95vw)',
                maxHeight: '90vh',
                overflowY: 'auto',
                borderRadius: 4,
                bgcolor: TOKENS.surface1,
                p: 3,
              }}
            >
              <UnifiedAuthFlow
                initialMode="register"
                onSuccess={() => setShowUnifiedFlow(false)}
                onCancel={() => setShowUnifiedFlow(false)}
              />
            </Paper>
          </Box>
        </Modal>
      )}
    </Box>
  );
};
