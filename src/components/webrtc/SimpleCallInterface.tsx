import React from 'react';
import { Box, Button, Typography } from '@mui/material';
import { CallEnd } from '@mui/icons-material';
import { CallState } from '../../services/webrtc/WebRTCService';

interface SimpleCallInterfaceProps {
  call: CallState;
  onEndCall: () => void;
}

export const SimpleCallInterface: React.FC<SimpleCallInterfaceProps> = ({
  call,
  onEndCall
}) => {
  return (
    <Box
      sx={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: '#000',
        color: 'white',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 9999
      }}
    >
      <Typography variant="h4" sx={{ mb: 4 }}>
        {call.type === 'video' ? 'Video Call' : 'Audio Call'}
      </Typography>
      
      <Typography variant="h6" sx={{ mb: 8 }}>
        Connected
      </Typography>

      <Button
        variant="contained"
        color="error"
        onClick={onEndCall}
        startIcon={<CallEnd />}
        size="large"
        sx={{ borderRadius: '50px', px: 4, py: 2 }}
      >
        End Call
      </Button>
    </Box>
  );
};

