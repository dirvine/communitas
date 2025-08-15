import React from 'react'
import {
  Breadcrumbs,
  Link,
  Typography,
  Box,
  Chip,
  IconButton,
  Stack
} from '@mui/material'
import {
  NavigateNext as NavigateNextIcon,
  Home as HomeIcon,
  Business as BusinessIcon,
  Group as GroupIcon,
  Work as WorkIcon,
  Person as PersonIcon,
  ArrowBack as ArrowBackIcon
} from '@mui/icons-material'
import { useNavigation } from '../../contexts/NavigationContext'
import { motion } from 'framer-motion'

const BreadcrumbNavigation: React.FC = () => {
  const { state, switchToPersonal, switchToOrganization, selectEntity, navigateBack, canGoBack } = useNavigation()

  const getIcon = (type: string) => {
    switch (type) {
      case 'personal':
        return <HomeIcon sx={{ fontSize: 16, mr: 0.5 }} />
      case 'organization':
        return <BusinessIcon sx={{ fontSize: 16, mr: 0.5 }} />
      case 'group':
        return <GroupIcon sx={{ fontSize: 16, mr: 0.5 }} />
      case 'project':
        return <WorkIcon sx={{ fontSize: 16, mr: 0.5 }} />
      case 'individual':
        return <PersonIcon sx={{ fontSize: 16, mr: 0.5 }} />
      default:
        return null
    }
  }

  const handleBreadcrumbClick = (breadcrumb: any, index: number) => {
    // If clicking on the first breadcrumb
    if (index === 0) {
      if (breadcrumb.type === 'personal') {
        switchToPersonal()
      } else {
        // Navigate to organizations list
        selectEntity('overview')
      }
    } 
    // If clicking on organization breadcrumb
    else if (breadcrumb.type === 'organization' && breadcrumb.id) {
      switchToOrganization(breadcrumb.id, breadcrumb.label)
    }
    // If clicking on entity breadcrumb
    else if (breadcrumb.id) {
      selectEntity(breadcrumb.type, breadcrumb.id, breadcrumb.label)
    }
  }

  return (
    <Box
      component={motion.div}
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
      sx={{
        p: 2,
        bgcolor: 'background.paper',
        borderBottom: 1,
        borderColor: 'divider',
        display: 'flex',
        alignItems: 'center',
        gap: 2
      }}
    >
      {/* Back Button */}
      {canGoBack && (
        <IconButton
          size="small"
          onClick={navigateBack}
          sx={{
            bgcolor: 'action.hover',
            '&:hover': {
              bgcolor: 'action.selected'
            }
          }}
        >
          <ArrowBackIcon fontSize="small" />
        </IconButton>
      )}

      {/* Breadcrumbs */}
      <Breadcrumbs
        separator={<NavigateNextIcon fontSize="small" />}
        sx={{ flexGrow: 1 }}
      >
        {state.breadcrumbs.map((breadcrumb, index) => {
          const isLast = index === state.breadcrumbs.length - 1
          
          return isLast ? (
            <Chip
              key={breadcrumb.path}
              icon={getIcon(breadcrumb.type) as any}
              label={breadcrumb.label}
              size="small"
              color="primary"
              variant="filled"
            />
          ) : (
            <Link
              key={breadcrumb.path}
              component="button"
              variant="body2"
              onClick={() => handleBreadcrumbClick(breadcrumb, index)}
              sx={{
                display: 'flex',
                alignItems: 'center',
                textDecoration: 'none',
                color: 'text.primary',
                '&:hover': {
                  color: 'primary.main',
                  textDecoration: 'underline'
                }
              }}
            >
              {getIcon(breadcrumb.type)}
              {breadcrumb.label}
            </Link>
          )
        })}
      </Breadcrumbs>

      {/* Context Indicator */}
      <Stack direction="row" spacing={1}>
        <Chip
          label={state.context === 'personal' ? 'Personal Space' : 'Organization Space'}
          size="small"
          color={state.context === 'personal' ? 'success' : 'primary'}
          variant="outlined"
        />
        {state.organizationId && (
          <Chip
            label={`ID: ${state.organizationId.substring(0, 8)}...`}
            size="small"
            variant="outlined"
            sx={{ fontSize: '0.7rem' }}
          />
        )}
      </Stack>
    </Box>
  )
}

export default BreadcrumbNavigation