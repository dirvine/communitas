import React, { useState } from 'react'
import { Box, Tabs, Tab, Paper } from '@mui/material'
import FileShareInterface from '../files/FileShareInterface'
import DocumentsInterface from '../documents/DocumentsInterface'

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
        <Box sx={{ height: '100%' }}>
          {children}
        </Box>
      )}
    </div>
  )
}

export default function FilesTab() {
  const [currentTab, setCurrentTab] = useState(0)

  const handleFileUpload = (files: File[]) => {
    console.log('Uploading files:', files)
  }

  const handleFileDownload = (file: any) => {
    console.log('Downloading file:', file)
  }

  const handleFileDelete = (fileId: string) => {
    console.log('Deleting file:', fileId)
  }

  const handleFileShare = (fileId: string, recipients: string[]) => {
    console.log('Sharing file:', fileId, 'with:', recipients)
  }

  const handleTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setCurrentTab(newValue)
  }

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <Paper square sx={{ borderBottom: 1, borderColor: 'divider' }}>
        <Tabs
          value={currentTab}
          onChange={handleTabChange}
          variant="fullWidth"
        >
          <Tab label="Documents" />
          <Tab label="File Sharing" />
        </Tabs>
      </Paper>

      <Box sx={{ flexGrow: 1, overflow: 'hidden' }}>
        <TabPanel value={currentTab} index={0}>
          <DocumentsInterface />
        </TabPanel>
        <TabPanel value={currentTab} index={1}>
          <FileShareInterface
            onFileUpload={handleFileUpload}
            onFileDownload={handleFileDownload}
            onFileDelete={handleFileDelete}
            onFileShare={handleFileShare}
          />
        </TabPanel>
      </Box>
    </Box>
  )
}
