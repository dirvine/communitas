import {
    Add as AddIcon
} from '@mui/icons-material'
import {
    alpha, Box,
    IconButton, Popover, Tooltip
} from '@mui/material'
import React, { useState } from 'react'

const COMMON_REACTIONS = ['👍', '❤️', '😄', '🎉', '👏', '🔥', '✅', '😍']

interface MessageReactionPickerProps {
  messageId: string
  onReact: (messageId: string, emoji: string) => void
  existingReactions?: Array<{ emoji: string; count: number; userReacted?: boolean }>
}

export const MessageReactionPicker: React.FC<MessageReactionPickerProps> = ({
  messageId,
  onReact, existingReactions: _existingReactions = [],
}) => {
  const [anchorEl, setAnchorEl] = useState<HTMLElement | null>(null)

  const handleOpen = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget)
  }

  const handleClose = () => {
    setAnchorEl(null)
  }

  const handleReact = (emoji: string) => {
    onReact(messageId, emoji)
    handleClose()
  }

  const open = Boolean(anchorEl)

  return (
    <>
      <Tooltip title="Add reaction">
        <IconButton
          size="small"
          onClick={handleOpen}
          sx={{
            opacity: 0.6,
            '&:hover': { opacity: 1 },
          }}
        >
          <AddIcon fontSize="small" />
        </IconButton>
      </Tooltip>

      <Popover
        open={open}
        anchorEl={anchorEl}
        onClose={handleClose}
        anchorOrigin={{
          vertical: 'bottom',
          horizontal: 'center',
        }}
        transformOrigin={{
          vertical: 'top',
          horizontal: 'center',
        }}
      >
        <Box
          sx={{
            display: 'flex',
            gap: 0.5,
            p: 1,
            backgroundColor: (theme) => theme.palette.background.paper,
          }}
        >
          {COMMON_REACTIONS.map((emoji) => (
            <IconButton
              key={emoji}
              onClick={() => handleReact(emoji)}
              sx={{
                fontSize: '1.5rem',
                width: 40,
                height: 40,
                transition: 'all 0.2s',
                '&:hover': {
                  transform: 'scale(1.2)',
                  backgroundColor: (theme) => alpha(theme.palette.primary.main, 0.1),
                },
              }}
            >
              {emoji}
            </IconButton>
          ))}
        </Box>
      </Popover>
    </>
  )
}

interface MessageReactionsDisplayProps {
  reactions: Array<{ emoji: string; count: number; userReacted?: boolean }>
  onReactionClick?: (emoji: string) => void
}

export const MessageReactionsDisplay: React.FC<MessageReactionsDisplayProps> = ({
  reactions,
  onReactionClick,
}) => {
  if (!reactions || reactions.length === 0) return null

  return (
    <Box sx={{ display: 'flex', gap: 0.5, flexWrap: 'wrap', mt: 0.5 }}>
      {reactions.map((reaction) => (
        <Box
          key={reaction.emoji}
          onClick={() => onReactionClick?.(reaction.emoji)}
          sx={{
            display: 'flex',
            alignItems: 'center',
            gap: 0.5,
            px: 1,
            py: 0.5,
            borderRadius: 2,
            backgroundColor: (theme) =>
              reaction.userReacted
                ? alpha(theme.palette.primary.main, 0.2)
                : alpha(theme.palette.action.hover, 0.5),
            border: (theme) =>
              reaction.userReacted
                ? `1px solid ${alpha(theme.palette.primary.main, 0.4)}`
                : `1px solid ${alpha(theme.palette.divider, 0.3)}`,
            cursor: 'pointer',
            transition: 'all 0.2s',
            '&:hover': {
              transform: 'scale(1.05)',
              backgroundColor: (theme) => alpha(theme.palette.primary.main, 0.15),
            },
          }}
        >
          <span style={{ fontSize: '0.9rem' }}>{reaction.emoji}</span>
          <span style={{ fontSize: '0.75rem', fontWeight: 500 }}>{reaction.count}</span>
        </Box>
      ))}
    </Box>
  )
}
