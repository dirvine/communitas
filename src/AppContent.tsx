import React, { useEffect } from 'react';
import { useAuth } from './components/auth';

interface AppContentProps {
  children: React.ReactNode;
  onAuthChange?: (fourWords: string | undefined) => void;
}

export const AppContent: React.FC<AppContentProps> = ({ children, onAuthChange }) => {
  const { authState } = useAuth();

  useEffect(() => {
    // When auth state changes, update the parent component
    if (authState.isAuthenticated && authState.user) {
      onAuthChange?.(authState.user.fourWordAddress);
    } else {
      onAuthChange?.(undefined);
    }
  }, [authState.isAuthenticated, authState.user, onAuthChange]);

  return <>{children}</>;
};