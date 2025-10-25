/**
 * CollaboratorAvatars - Display active collaborators on a document
 *
 * Features:
 * - Avatar stack showing active editors
 * - User presence indicators
 * - Cursor position tracking
 * - Hover tooltips with user details
 * - Color-coded user indicators
 */

import {
    Circle as OnlineIcon,
    Edit as EditingIcon,
    Visibility as ViewingIcon
} from '@mui/icons-material';
import {
    Avatar,
    AvatarGroup, Badge, Box, Chip, Paper, Stack,
    Tooltip,
    Typography
} from '@mui/material';
import { alpha, styled } from '@mui/material/styles';
import React from 'react';

export interface Collaborator {
  userId: string;
  userName: string;
  userAvatar?: string;
  color: string;
  isEditing: boolean;
  cursorPosition?: number;
  lastActive: Date;
}

interface CollaboratorAvatarsProps {
  /** List of active collaborators */
  collaborators: Collaborator[];
  /** Current user ID to highlight */
  currentUserId?: string;
  /** Maximum avatars to show before grouping */
  maxAvatars?: number;
  /** Show detailed view with names */
  detailed?: boolean;
  /** Compact mode for smaller displays */
  compact?: boolean;
}

// Styled components
const CollaboratorChip = styled(Chip)(({ theme }) => ({
  background: alpha(theme.palette.background.paper, 0.9),
  backdropFilter: 'blur(10px)',
  '& .MuiChip-label': {
    display: 'flex',
    alignItems: 'center',
    gap: theme.spacing(0.5),
  },
}));

const OnlineBadge = styled(Badge)(({ theme }) => ({
  '& .MuiBadge-badge': {
    backgroundColor: '#44b700',
    color: '#44b700',
    boxShadow: `0 0 0 2px ${theme.palette.background.paper}`,
    '&::after': {
      position: 'absolute',
      top: 0,
      left: 0,
      width: '100%',
      height: '100%',
      borderRadius: '50%',
      animation: 'ripple 1.2s infinite ease-in-out',
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
}));

const EditingBadge = styled(Badge)(({ theme }) => ({
  '& .MuiBadge-badge': {
    backgroundColor: theme.palette.primary.main,
    color: theme.palette.primary.main,
    boxShadow: `0 0 0 2px ${theme.palette.background.paper}`,
  },
}));

// Generate consistent color from user ID
const stringToColor = (str: string): string => {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = str.charCodeAt(i) + ((hash << 5) - hash);
  }
  const hue = hash % 360;
  return `hsl(${hue}, 70%, 60%)`;
};

// Format last active time
const formatLastActive = (date: Date): string => {
  const seconds = Math.floor((new Date().getTime() - date.getTime()) / 1000);
  if (seconds < 60) return 'Just now';
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  return `${Math.floor(seconds / 86400)}d ago`;
};

export const CollaboratorAvatars: React.FC<CollaboratorAvatarsProps> = ({
  collaborators,
  currentUserId,
  maxAvatars = 5,
  detailed = false,
  compact = false,
}) => {
  // Sort collaborators: editing first, then by last active
  const sortedCollaborators = [...collaborators].sort((a, b) => {
    if (a.isEditing !== b.isEditing) return a.isEditing ? -1 : 1;
    return b.lastActive.getTime() - a.lastActive.getTime();
  });

  const editingCount = collaborators.filter((c) => c.isEditing).length;
  const viewingCount = collaborators.length - editingCount;

  // Detailed view with names and status
  if (detailed) {
    return (
      <Paper
        elevation={0}
        sx={{
          p: 2,
          background: (theme) => alpha(theme.palette.background.paper, 0.6),
          backdropFilter: 'blur(10px)',
        }}
      >
        <Typography variant="subtitle2" gutterBottom sx={{ fontWeight: 600 }}>
          Active Collaborators ({collaborators.length})
        </Typography>

        <Stack spacing={1.5} sx={{ mt: 2 }}>
          {sortedCollaborators.map((collaborator) => {
            const isCurrentUser = collaborator.userId === currentUserId;
            const avatarColor = collaborator.color || stringToColor(collaborator.userId);

            return (
              <Stack
                key={collaborator.userId}
                direction="row"
                alignItems="center"
                spacing={1.5}
                sx={{
                  p: 1,
                  borderRadius: 1,
                  background: isCurrentUser
                    ? (theme) => alpha(theme.palette.primary.main, 0.1)
                    : 'transparent',
                }}
              >
                {collaborator.isEditing ? (
                  <EditingBadge
                    overlap="circular"
                    anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                    variant="dot"
                  >
                    <Avatar
                      src={collaborator.userAvatar}
                      sx={{
                        width: 32,
                        height: 32,
                        bgcolor: avatarColor,
                        border: `2px solid ${avatarColor}`,
                      }}
                    >
                      {collaborator.userName.charAt(0).toUpperCase()}
                    </Avatar>
                  </EditingBadge>
                ) : (
                  <OnlineBadge
                    overlap="circular"
                    anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                    variant="dot"
                  >
                    <Avatar
                      src={collaborator.userAvatar}
                      sx={{
                        width: 32,
                        height: 32,
                        bgcolor: avatarColor,
                      }}
                    >
                      {collaborator.userName.charAt(0).toUpperCase()}
                    </Avatar>
                  </OnlineBadge>
                )}

                <Box sx={{ flex: 1 }}>
                  <Typography variant="body2" sx={{ fontWeight: 500 }}>
                    {collaborator.userName}
                    {isCurrentUser && ' (You)'}
                  </Typography>
                  <Stack direction="row" spacing={1} alignItems="center">
                    {collaborator.isEditing ? (
                      <EditingIcon sx={{ fontSize: 14, color: 'primary.main' }} />
                    ) : (
                      <ViewingIcon sx={{ fontSize: 14, color: 'text.secondary' }} />
                    )}
                    <Typography variant="caption" color="text.secondary">
                      {collaborator.isEditing ? 'Editing' : 'Viewing'}
                    </Typography>
                    <Typography variant="caption" color="text.secondary">
                      • {formatLastActive(collaborator.lastActive)}
                    </Typography>
                  </Stack>
                </Box>
              </Stack>
            );
          })}
        </Stack>
      </Paper>
    );
  }

  // Compact view - just avatar group
  if (compact) {
    return (
      <AvatarGroup
        max={maxAvatars}
        sx={{
          '& .MuiAvatar-root': {
            width: 28,
            height: 28,
            fontSize: '0.875rem',
          },
        }}
      >
        {sortedCollaborators.map((collaborator) => {
          const isCurrentUser = collaborator.userId === currentUserId;
          const avatarColor = collaborator.color || stringToColor(collaborator.userId);

          return (
            <Tooltip
              key={collaborator.userId}
              title={
                <Box>
                  <Typography variant="body2" sx={{ fontWeight: 500 }}>
                    {collaborator.userName}
                    {isCurrentUser && ' (You)'}
                  </Typography>
                  <Typography variant="caption">
                    {collaborator.isEditing ? 'Editing' : 'Viewing'} •{' '}
                    {formatLastActive(collaborator.lastActive)}
                  </Typography>
                </Box>
              }
            >
              {collaborator.isEditing ? (
                <EditingBadge overlap="circular" variant="dot">
                  <Avatar
                    src={collaborator.userAvatar}
                    sx={{
                      bgcolor: avatarColor,
                      border: `2px solid ${avatarColor}`,
                    }}
                  >
                    {collaborator.userName.charAt(0).toUpperCase()}
                  </Avatar>
                </EditingBadge>
              ) : (
                <Avatar
                  src={collaborator.userAvatar}
                  sx={{ bgcolor: avatarColor }}
                >
                  {collaborator.userName.charAt(0).toUpperCase()}
                </Avatar>
              )}
            </Tooltip>
          );
        })}
      </AvatarGroup>
    );
  }

  // Default view - avatars with status chips
  return (
    <Stack direction="row" spacing={2} alignItems="center">
      <AvatarGroup max={maxAvatars}>
        {sortedCollaborators.map((collaborator) => {
          const isCurrentUser = collaborator.userId === currentUserId;
          const avatarColor = collaborator.color || stringToColor(collaborator.userId);

          return (
            <Tooltip
              key={collaborator.userId}
              title={
                <Box>
                  <Typography variant="body2" sx={{ fontWeight: 500 }}>
                    {collaborator.userName}
                    {isCurrentUser && ' (You)'}
                  </Typography>
                  <Typography variant="caption">
                    {collaborator.isEditing ? 'Editing' : 'Viewing'} •{' '}
                    {formatLastActive(collaborator.lastActive)}
                  </Typography>
                </Box>
              }
            >
              {collaborator.isEditing ? (
                <EditingBadge
                  overlap="circular"
                  anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                  variant="dot"
                >
                  <Avatar
                    src={collaborator.userAvatar}
                    sx={{
                      bgcolor: avatarColor,
                      border: `2px solid ${avatarColor}`,
                    }}
                  >
                    {collaborator.userName.charAt(0).toUpperCase()}
                  </Avatar>
                </EditingBadge>
              ) : (
                <OnlineBadge
                  overlap="circular"
                  anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                  variant="dot"
                >
                  <Avatar
                    src={collaborator.userAvatar}
                    sx={{ bgcolor: avatarColor }}
                  >
                    {collaborator.userName.charAt(0).toUpperCase()}
                  </Avatar>
                </OnlineBadge>
              )}
            </Tooltip>
          );
        })}
      </AvatarGroup>

      {collaborators.length > 0 && (
        <Stack direction="row" spacing={1}>
          {editingCount > 0 && (
            <CollaboratorChip
              icon={<EditingIcon sx={{ fontSize: 16 }} />}
              label={`${editingCount} editing`}
              size="small"
              color="primary"
              variant="outlined"
            />
          )}
          {viewingCount > 0 && (
            <CollaboratorChip
              icon={<OnlineIcon sx={{ fontSize: 16 }} />}
              label={`${viewingCount} viewing`}
              size="small"
              variant="outlined"
            />
          )}
        </Stack>
      )}
    </Stack>
  );
};
