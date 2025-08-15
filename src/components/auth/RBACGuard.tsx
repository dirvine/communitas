import React from 'react';
import { Box, Alert, Typography, Button } from '@mui/material';
import { Lock as LockIcon, Warning as WarningIcon } from '@mui/icons-material';
import { useAuth } from '../../contexts/AuthContext';
import { rbacManager, ResourceType, Action } from '../../utils/rbac';

// Props for RBAC-protected components
export interface RBACGuardProps {
  children: React.ReactNode;
  resource: ResourceType;
  action: Action;
  resourceId?: string;
  organizationId?: string;
  projectId?: string;
  resourceData?: Record<string, any>;
  fallback?: React.ReactNode;
  showFallback?: boolean;
  requireAuth?: boolean;
  onUnauthorized?: () => void;
}

/**
 * RBAC Guard Component
 * Conditionally renders children based on user permissions
 */
export const RBACGuard: React.FC<RBACGuardProps> = ({
  children,
  resource,
  action,
  resourceId,
  organizationId,
  projectId,
  resourceData,
  fallback,
  showFallback = false,
  requireAuth = true,
  onUnauthorized,
}) => {
  const { authState } = useAuth();

  // Check authentication first
  if (requireAuth && !authState.isAuthenticated) {
    if (showFallback) {
      return fallback ? (
        <>{fallback}</>
      ) : (
        <Alert severity="warning" icon={<LockIcon />}>
          <Typography variant="body2">
            You must be signed in to access this feature
          </Typography>
        </Alert>
      );
    }
    onUnauthorized?.();
    return null;
  }

  // Check permissions if user is authenticated
  if (authState.isAuthenticated && authState.user) {
    const hasPermission = rbacManager.hasPermission({
      user: authState.user,
      resource,
      action,
      resourceId,
      organizationId,
      projectId,
      resourceData,
    });

    if (!hasPermission) {
      if (showFallback) {
        return fallback ? (
          <>{fallback}</>
        ) : (
          <Alert severity="error" icon={<WarningIcon />}>
            <Typography variant="body2">
              You don't have permission to {action} {resource}
            </Typography>
          </Alert>
        );
      }
      onUnauthorized?.();
      return null;
    }
  }

  // User has permission, render children
  return <>{children}</>;
};

// Specific guards for common use cases
export interface CreateGuardProps {
  children: React.ReactNode;
  resource: ResourceType;
  organizationId?: string;
  projectId?: string;
  fallback?: React.ReactNode;
  showFallback?: boolean;
}

export const CreateGuard: React.FC<CreateGuardProps> = ({
  children,
  resource,
  organizationId,
  projectId,
  fallback,
  showFallback = false,
}) => (
  <RBACGuard
    resource={resource}
    action={Action.CREATE}
    organizationId={organizationId}
    projectId={projectId}
    fallback={fallback}
    showFallback={showFallback}
  >
    {children}
  </RBACGuard>
);

export const UpdateGuard: React.FC<CreateGuardProps & { resourceId?: string; resourceData?: Record<string, any> }> = ({
  children,
  resource,
  resourceId,
  organizationId,
  projectId,
  resourceData,
  fallback,
  showFallback = false,
}) => (
  <RBACGuard
    resource={resource}
    action={Action.UPDATE}
    resourceId={resourceId}
    organizationId={organizationId}
    projectId={projectId}
    resourceData={resourceData}
    fallback={fallback}
    showFallback={showFallback}
  >
    {children}
  </RBACGuard>
);

export const DeleteGuard: React.FC<CreateGuardProps & { resourceId?: string; resourceData?: Record<string, any> }> = ({
  children,
  resource,
  resourceId,
  organizationId,
  projectId,
  resourceData,
  fallback,
  showFallback = false,
}) => (
  <RBACGuard
    resource={resource}
    action={Action.DELETE}
    resourceId={resourceId}
    organizationId={organizationId}
    projectId={projectId}
    resourceData={resourceData}
    fallback={fallback}
    showFallback={showFallback}
  >
    {children}
  </RBACGuard>
);

export const ManageGuard: React.FC<CreateGuardProps & { resourceId?: string }> = ({
  children,
  resource,
  resourceId,
  organizationId,
  projectId,
  fallback,
  showFallback = false,
}) => (
  <RBACGuard
    resource={resource}
    action={Action.MANAGE}
    resourceId={resourceId}
    organizationId={organizationId}
    projectId={projectId}
    fallback={fallback}
    showFallback={showFallback}
  >
    {children}
  </RBACGuard>
);

// Component for showing access denied message
export interface AccessDeniedProps {
  title?: string;
  message?: string;
  showLoginButton?: boolean;
  onLoginClick?: () => void;
}

export const AccessDenied: React.FC<AccessDeniedProps> = ({
  title = 'Access Denied',
  message = 'You don\'t have permission to access this resource.',
  showLoginButton = false,
  onLoginClick,
}) => (
  <Box
    sx={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      justifyContent: 'center',
      p: 4,
      textAlign: 'center',
      minHeight: 200,
    }}
  >
    <LockIcon sx={{ fontSize: 48, color: 'text.secondary', mb: 2 }} />
    <Typography variant="h6" fontWeight={600} gutterBottom>
      {title}
    </Typography>
    <Typography variant="body1" color="text.secondary" sx={{ mb: 3 }}>
      {message}
    </Typography>
    {showLoginButton && (
      <Button
        variant="contained"
        startIcon={<LockIcon />}
        onClick={onLoginClick}
      >
        Sign In
      </Button>
    )}
  </Box>
);

export default RBACGuard;