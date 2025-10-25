import { useState } from 'react'
import {
  Avatar,
  Badge,
  Box,
  Chip,
  IconButton,
  ListItem,
  ListItemAvatar,
  ListItemText,
  Menu,
  MenuItem,
  Typography
} from '@mui/material'
import { MoreVert, Person } from '@mui/icons-material'
import type { MemberInfo, MemberRole } from '@/types/memberManagement'

interface MemberCardProps {
  member: MemberInfo
  canManage: boolean
  onRemove?: (memberId: string) => void
  onRoleChange?: (memberId: string, newRole: MemberRole) => void
}

export function MemberCard({ member, canManage, onRemove, onRoleChange }: MemberCardProps) {
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null)

  const getRoleBadgeColor = (role: MemberRole): 'error' | 'warning' | 'primary' | 'default' => {
    switch (role) {
      case 'owner':
        return 'error'
      case 'admin':
        return 'warning'
      case 'member':
        return 'primary'
      case 'guest':
        return 'default'
    }
  }

  const formatRelativeTime = (timestamp: number): string => {
    const now = Date.now()
    const diffMs = now - timestamp
    const diffMins = Math.floor(diffMs / 60000)

    if (diffMins < 1) return 'just now'
    if (diffMins < 60) return `${diffMins}m ago`
    const diffHours = Math.floor(diffMins / 60)
    if (diffHours < 24) return `${diffHours}h ago`
    const diffDays = Math.floor(diffHours / 24)
    return `${diffDays}d ago`
  }

  const isOnline = !member.deleted

  return (
    <ListItem
      secondaryAction={
        canManage ? (
          <IconButton
            onClick={(e) => setAnchorEl(e.currentTarget)}
            aria-label="more options"
          >
            <MoreVert />
          </IconButton>
        ) : undefined
      }
    >
      <ListItemAvatar>
        <Badge
          color={isOnline ? 'success' : 'default'}
          variant="dot"
          overlap="circular"
          anchorOrigin={{
            vertical: 'bottom',
            horizontal: 'right'
          }}
        >
          <Avatar>
            <Person />
          </Avatar>
        </Badge>
      </ListItemAvatar>

      <ListItemText
        primary={member.member_id}
        secondary={
          <Box display="flex" gap={1} alignItems="center" flexWrap="wrap">
            <Chip
              label={member.role.toUpperCase()}
              size="small"
              color={getRoleBadgeColor(member.role as MemberRole)}
            />
            <Typography variant="caption" color="textSecondary">
              Joined: {formatRelativeTime(member.joined_at)}
            </Typography>
          </Box>
        }
      />

      {canManage && (
        <Menu
          anchorEl={anchorEl}
          open={Boolean(anchorEl)}
          onClose={() => setAnchorEl(null)}
        >
          <MenuItem
            onClick={() => {
              setAnchorEl(null)
              if (onRoleChange) {
                onRoleChange(member.member_id, member.role)
              }
            }}
          >
            Change Role
          </MenuItem>
          <MenuItem
            onClick={() => {
              setAnchorEl(null)
              if (onRemove) {
                onRemove(member.member_id)
              }
            }}
          >
            Remove Member
          </MenuItem>
        </Menu>
      )}
    </ListItem>
  )
}
