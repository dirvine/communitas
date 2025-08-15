import React from 'react'
import { Box } from '@mui/material'
import OrganizationDashboardEnhanced from '../organization/OrganizationDashboardEnhanced'

const OrganizationTab: React.FC = () => {
  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <OrganizationDashboardEnhanced />
    </Box>
  )
}

export default OrganizationTab
