import React from 'react'
import {
  Card,
  CardContent,
  Typography,
  Box,
  Chip,
  IconButton,
  Tooltip,
  Stack,
} from '@mui/material'
import {
  ContentCopy as CopyIcon,
  Verified as VerifiedIcon,
  Person as PersonIcon,
} from '@mui/icons-material'
import { IdentityInfo } from '../../types'

interface IdentityCardProps {
  identity: IdentityInfo
  onCopyAddress?: () => void
}

const IdentityCard: React.FC<IdentityCardProps> = ({ identity, onCopyAddress }) => {
  const handleCopyAddress = async () => {
    try {
      await navigator.clipboard.writeText(identity.four_word_address)
      if (onCopyAddress) {
        onCopyAddress()
      }
    } catch (err) {
      console.error('Failed to copy address:', err)
    }
  }

  const getVerificationColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'verified':
        return 'success'
      case 'generated':
        return 'info'
      case 'loaded':
        return 'primary'
      default:
        return 'default'
    }
  }

  const formatCreatedDate = (dateString: string) => {
    try {
      return new Date(dateString).toLocaleDateString()
    } catch {
      return 'Unknown'
    }
  }

  return (
    <Card elevation={2}>
      <CardContent>
        <Stack spacing={2}>
          {/* Header */}
          <Box display="flex" alignItems="center" justifyContent="space-between">
            <Box display="flex" alignItems="center" gap={1}>
              <PersonIcon color="primary" />
              <Typography variant="h6">
                {identity.display_name || 'My Identity'}
              </Typography>
              {identity.is_primary && (
                <Chip size="small" label="Primary" color="primary" />
              )}
            </Box>
            <Chip
              icon={<VerifiedIcon />}
              label={identity.verification_status}
              color={getVerificationColor(identity.verification_status)}
              size="small"
            />
          </Box>

          {/* Four-word address */}
          <Box>
            <Typography variant="body2" color="text.secondary" gutterBottom>
              Network Address
            </Typography>
            <Box display="flex" alignItems="center" gap={1}>
              <Typography
                variant="h5"
                component="div"
                sx={{
                  fontFamily: 'monospace',
                  wordBreak: 'break-all',
                  color: 'primary.main',
                  fontWeight: 'bold',
                }}
              >
                {identity.four_word_address}
              </Typography>
              <Tooltip title="Copy address to clipboard">
                <IconButton
                  size="small"
                  onClick={handleCopyAddress}
                  sx={{ ml: 1 }}
                >
                  <CopyIcon fontSize="small" />
                </IconButton>
              </Tooltip>
            </Box>
          </Box>

          {/* Additional info */}
          <Box display="flex" justifyContent="space-between">
            <Box>
              <Typography variant="body2" color="text.secondary">
                Created
              </Typography>
              <Typography variant="body2">
                {formatCreatedDate(identity.created_at)}
              </Typography>
            </Box>
            {identity.public_key_hex && (
              <Box>
                <Typography variant="body2" color="text.secondary">
                  Public Key
                </Typography>
                <Typography
                  variant="body2"
                  sx={{
                    fontFamily: 'monospace',
                    fontSize: '0.75rem',
                    maxWidth: '120px',
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                  }}
                >
                  {identity.public_key_hex.slice(0, 16)}...
                </Typography>
              </Box>
            )}
          </Box>
        </Stack>
      </CardContent>
    </Card>
  )
}

export default IdentityCard
