import React from 'react'
import { Box, Typography, Button } from '@mui/material'

export default function AppSimple() {
  return (
    <Box sx={{ p: 4, textAlign: 'center' }}>
      <Typography variant="h4" gutterBottom>
        Communitas - Simple Test
      </Typography>
      <Typography variant="body1" paragraph>
        This is a simplified version to test if the app loads.
      </Typography>
      <Button variant="contained" onClick={() => alert('Hello!')}>
        Test Button
      </Button>
    </Box>
  )
}