import React, { useState } from 'react';
import { Button } from '@mui/material';
import { Login as LoginIcon } from '@mui/icons-material';

export const AuthStatusDebug: React.FC = () => {
  const [clicked, setClicked] = useState(0);

  const handleClick = () => {
    setClicked(prev => prev + 1);
    console.log('🔴 DEBUG: Sign In button clicked!', clicked + 1);
    alert(`Sign In clicked ${clicked + 1} time(s)`);
  };

  return (
    <Button
      variant="outlined"
      startIcon={<LoginIcon />}
      onClick={handleClick}
      size="medium"
      color="primary"
    >
      Sign In (Debug: {clicked})
    </Button>
  );
};