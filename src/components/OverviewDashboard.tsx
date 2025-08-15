import React, { useState } from 'react'
import {
  Box,
  Paper,
  Typography,
  Tabs,
  Tab,
  IconButton,
  Tooltip,
  Stack,
} from '@mui/material'
import {
  Dashboard,
  NetworkCheck,
  Storage as StorageIcon,
  Close,
} from '@mui/icons-material'
import { NetworkHealth } from '../types'
import OverviewTab from './tabs/OverviewTab'
import NetworkDiagnosticsTab from './tabs/NetworkDiagnosticsTab'
import StorageTab from './tabs/StorageTab'

interface OverviewDashboardProps {
  networkHealth: NetworkHealth
  onClose?: () => void
}

interface TabPanelProps {
  children?: React.ReactNode
  index: number
  value: number
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props
  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      {...other}
    >
      {value === index && (
        <Box sx={{ pt: 2 }}>
          {children}
        </Box>
      )}
    </div>
  )
}

const OverviewDashboard: React.FC<OverviewDashboardProps> = ({ 
  networkHealth,
  onClose 
}) => {
  const [currentTab, setCurrentTab] = useState(0)

  const handleTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setCurrentTab(newValue)
  }

  return (
    <Paper
      sx={{
        position: 'fixed',
        top: '50%',
        left: '50%',
        transform: 'translate(-50%, -50%)',
        width: '90%',
        maxWidth: 1200,
        height: '80vh',
        display: 'flex',
        flexDirection: 'column',
        zIndex: 1300,
        overflow: 'hidden',
      }}
      elevation={24}
    >
      {/* Header */}
      <Box
        sx={{
          p: 2,
          borderBottom: 1,
          borderColor: 'divider',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        <Typography variant="h5" sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <Dashboard color="primary" />
          System Overview
        </Typography>
        {onClose && (
          <Tooltip title="Close">
            <IconButton onClick={onClose} size="small">
              <Close />
            </IconButton>
          </Tooltip>
        )}
      </Box>

      {/* Tabs */}
      <Box sx={{ borderBottom: 1, borderColor: 'divider', px: 2 }}>
        <Tabs value={currentTab} onChange={handleTabChange}>
          <Tab 
            icon={<Dashboard />} 
            iconPosition="start" 
            label="Overview" 
          />
          <Tab 
            icon={<NetworkCheck />} 
            iconPosition="start" 
            label="Network Diagnostics" 
          />
          <Tab 
            icon={<StorageIcon />} 
            iconPosition="start" 
            label="Storage" 
          />
        </Tabs>
      </Box>

      {/* Content */}
      <Box sx={{ flex: 1, overflow: 'auto', p: 2 }}>
        <TabPanel value={currentTab} index={0}>
          <OverviewTab networkHealth={networkHealth} />
        </TabPanel>
        <TabPanel value={currentTab} index={1}>
          <NetworkDiagnosticsTab />
        </TabPanel>
        <TabPanel value={currentTab} index={2}>
          <StorageTab />
        </TabPanel>
      </Box>
    </Paper>
  )
}

export default OverviewDashboard