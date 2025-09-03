import React from 'react';
import { Box, Typography, Button } from '@mui/material';

export const TestPage: React.FC = () => {
  return (
    <Box sx={{ p: 4, textAlign: 'center' }}>
      <Typography variant="h4" gutterBottom>
        🧪 Test Page
      </Typography>
      <Typography variant="body1" sx={{ mb: 3 }}>
        This is a simple test page to verify routing is working.
      </Typography>
      <Button variant="contained" onClick={() => alert('Button works!')}>
        Test Button
      </Button>
    </Box>
  );
};