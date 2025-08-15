import React from 'react';
import { Box, Container, Typography, Button, Paper, Alert } from '@mui/material';
import DesktopMacIcon from '@mui/icons-material/DesktopMac';
import DownloadIcon from '@mui/icons-material/Download';
import InfoIcon from '@mui/icons-material/Info';

export const BrowserFallback: React.FC = () => {
  return (
    <Container maxWidth="sm">
      <Box 
        sx={{ 
          minHeight: '100vh', 
          display: 'flex', 
          alignItems: 'center', 
          justifyContent: 'center',
          py: 4
        }}
      >
        <Paper elevation={3} sx={{ p: 4, textAlign: 'center' }}>
          <DesktopMacIcon sx={{ fontSize: 64, color: 'primary.main', mb: 2 }} />
          
          <Typography variant="h4" component="h1" gutterBottom>
            Communitas Desktop App Required
          </Typography>
          
          <Typography variant="body1" color="text.secondary" paragraph>
            This application requires the Communitas desktop app to function properly.
            The desktop app provides secure P2P networking, identity management, and 
            encrypted storage features.
          </Typography>

          <Alert severity="info" sx={{ mb: 3 }}>
            <Typography variant="body2">
              You're currently viewing the web interface. To access full functionality,
              please download and run the desktop application.
            </Typography>
          </Alert>

          <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center', mb: 3 }}>
            <Button
              variant="contained"
              size="large"
              startIcon={<DownloadIcon />}
              href="https://github.com/p2pfoundation/communitas/releases"
              target="_blank"
            >
              Download for macOS
            </Button>
          </Box>

          <Typography variant="caption" color="text.secondary" display="block">
            Also available for Windows and Linux
          </Typography>

          <Box sx={{ mt: 4, pt: 3, borderTop: 1, borderColor: 'divider' }}>
            <Typography variant="body2" color="text.secondary" paragraph>
              <InfoIcon sx={{ fontSize: 16, verticalAlign: 'middle', mr: 1 }} />
              Running in development? Make sure to start the app with:
            </Typography>
            <Paper sx={{ p: 2, bgcolor: 'grey.100' }}>
              <Typography variant="body2" fontFamily="monospace">
                npm run tauri dev
              </Typography>
            </Paper>
          </Box>
        </Paper>
      </Box>
    </Container>
  );
};