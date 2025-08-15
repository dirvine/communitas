import { useMemo } from 'react';
import { useAuth } from '../contexts/AuthContext';
import { rbacManager, ResourceType, Action, PermissionContext } from '../utils/rbac';

// Hook return type
export interface RBACHook {
  // Permission checks
  hasPermission: (
    resource: ResourceType,
    action: Action,
    resourceId?: string,
    organizationId?: string,
    projectId?: string,
    resourceData?: Record<string, any>
  ) => boolean;
  
  // Convenience methods for common actions
  canCreate: (resource: ResourceType, organizationId?: string, projectId?: string) => boolean;
  canRead: (
    resource: ResourceType,
    resourceId?: string,
    organizationId?: string,
    projectId?: string,
    resourceData?: Record<string, any>
  ) => boolean;
  canUpdate: (
    resource: ResourceType,
    resourceId?: string,
    organizationId?: string,
    projectId?: string,
    resourceData?: Record<string, any>
  ) => boolean;
  canDelete: (
    resource: ResourceType,
    resourceId?: string,
    organizationId?: string,
    projectId?: string,
    resourceData?: Record<string, any>
  ) => boolean;
  canManage: (
    resource: ResourceType,
    resourceId?: string,
    organizationId?: string,
    projectId?: string
  ) => boolean;
  
  // Get all permitted actions for a resource
  getPermittedActions: (
    resource: ResourceType,
    resourceId?: string,
    organizationId?: string,
    projectId?: string,
    resourceData?: Record<string, any>
  ) => Action[];
  
  // Role checks
  hasRole: (role: string, scope?: string) => boolean;
  getRoles: (scope?: string) => string[];
  
  // User info
  user: ReturnType<typeof useAuth>['authState']['user'];
  isAuthenticated: boolean;
  loading: boolean;
}

/**
 * Custom hook for RBAC (Role-Based Access Control)
 * Provides easy access to permission checks throughout the application
 */
export const useRBAC = (): RBACHook => {
  const { authState } = useAuth();
  const { user, isAuthenticated, loading } = authState;

  const rbacMethods = useMemo(() => {
    // If user is not authenticated, return restrictive permissions
    if (!user) {
      return {
        hasPermission: () => false,
        canCreate: () => false,
        canRead: () => false,
        canUpdate: () => false,
        canDelete: () => false,
        canManage: () => false,
        getPermittedActions: () => [],
        hasRole: () => false,
        getRoles: () => [],
      };
    }

    return {
      hasPermission: (
        resource: ResourceType,
        action: Action,
        resourceId?: string,
        organizationId?: string,
        projectId?: string,
        resourceData?: Record<string, any>
      ) => {
        const context: PermissionContext = {
          user,
          resource,
          action,
          resourceId,
          organizationId,
          projectId,
          resourceData,
        };
        return rbacManager.hasPermission(context);
      },

      canCreate: (resource: ResourceType, organizationId?: string, projectId?: string) => {
        const context: PermissionContext = {
          user,
          resource,
          action: Action.CREATE,
          organizationId,
          projectId,
        };
        return rbacManager.hasPermission(context);
      },

      canRead: (
        resource: ResourceType,
        resourceId?: string,
        organizationId?: string,
        projectId?: string,
        resourceData?: Record<string, any>
      ) => {
        const context: PermissionContext = {
          user,
          resource,
          action: Action.READ,
          resourceId,
          organizationId,
          projectId,
          resourceData,
        };
        return rbacManager.hasPermission(context);
      },

      canUpdate: (
        resource: ResourceType,
        resourceId?: string,
        organizationId?: string,
        projectId?: string,
        resourceData?: Record<string, any>
      ) => {
        const context: PermissionContext = {
          user,
          resource,
          action: Action.UPDATE,
          resourceId,
          organizationId,
          projectId,
          resourceData,
        };
        return rbacManager.hasPermission(context);
      },

      canDelete: (
        resource: ResourceType,
        resourceId?: string,
        organizationId?: string,
        projectId?: string,
        resourceData?: Record<string, any>
      ) => {
        const context: PermissionContext = {
          user,
          resource,
          action: Action.DELETE,
          resourceId,
          organizationId,
          projectId,
          resourceData,
        };
        return rbacManager.hasPermission(context);
      },

      canManage: (
        resource: ResourceType,
        resourceId?: string,
        organizationId?: string,
        projectId?: string
      ) => {
        const context: PermissionContext = {
          user,
          resource,
          action: Action.MANAGE,
          resourceId,
          organizationId,
          projectId,
        };
        return rbacManager.hasPermission(context);
      },

      getPermittedActions: (
        resource: ResourceType,
        resourceId?: string,
        organizationId?: string,
        projectId?: string,
        resourceData?: Record<string, any>
      ) => {
        return rbacManager.getPermittedActions(
          user,
          resource,
          resourceId,
          organizationId,
          projectId,
          resourceData
        );
      },

      hasRole: (role: string, scope?: string) => {
        return user.permissions.some(permission => {
          if (scope) {
            return permission.scope === scope && permission.resource === role;
          }
          return permission.resource === role;
        });
      },

      getRoles: (scope?: string) => {
        const roles: string[] = [];
        
        user.permissions.forEach(permission => {
          if (scope) {
            if (permission.scope === scope) {
              roles.push(permission.resource);
            }
          } else if (!permission.scope) {
            roles.push(permission.resource);
          }
        });
        
        return roles;
      },
    };
  }, [user]);

  return {
    ...rbacMethods,
    user,
    isAuthenticated,
    loading,
  };
};

// Specialized hooks for specific contexts
export interface OrganizationRBACHook extends Omit<RBACHook, 'canCreate' | 'canRead' | 'canUpdate' | 'canDelete' | 'canManage'> {
  canCreateInOrg: (resource: ResourceType) => boolean;
  canReadInOrg: (resource: ResourceType, resourceId?: string, resourceData?: Record<string, any>) => boolean;
  canUpdateInOrg: (resource: ResourceType, resourceId?: string, resourceData?: Record<string, any>) => boolean;
  canDeleteInOrg: (resource: ResourceType, resourceId?: string, resourceData?: Record<string, any>) => boolean;
  canManageInOrg: (resource: ResourceType, resourceId?: string) => boolean;
  isOrgOwner: boolean;
  isOrgAdmin: boolean;
  isOrgModerator: boolean;
  orgRole: string | null;
}

/**
 * Hook for organization-scoped RBAC
 */
export const useOrganizationRBAC = (organizationId: string): OrganizationRBACHook => {
  const baseRBAC = useRBAC();

  const organizationMethods = useMemo(() => ({
    canCreateInOrg: (resource: ResourceType) => 
      baseRBAC.canCreate(resource, organizationId),
    
    canReadInOrg: (resource: ResourceType, resourceId?: string, resourceData?: Record<string, any>) =>
      baseRBAC.canRead(resource, resourceId, organizationId, undefined, resourceData),
    
    canUpdateInOrg: (resource: ResourceType, resourceId?: string, resourceData?: Record<string, any>) =>
      baseRBAC.canUpdate(resource, resourceId, organizationId, undefined, resourceData),
    
    canDeleteInOrg: (resource: ResourceType, resourceId?: string, resourceData?: Record<string, any>) =>
      baseRBAC.canDelete(resource, resourceId, organizationId, undefined, resourceData),
    
    canManageInOrg: (resource: ResourceType, resourceId?: string) =>
      baseRBAC.canManage(resource, resourceId, organizationId),
    
    isOrgOwner: baseRBAC.hasRole('owner', `organization:${organizationId}`),
    isOrgAdmin: baseRBAC.hasRole('admin', `organization:${organizationId}`),
    isOrgModerator: baseRBAC.hasRole('moderator', `organization:${organizationId}`),
    
    orgRole: (() => {
      const roles = baseRBAC.getRoles(`organization:${organizationId}`);
      return roles.length > 0 ? roles[0] : null;
    })(),
  }), [baseRBAC, organizationId]);

  return {
    ...baseRBAC,
    ...organizationMethods,
  };
};

export interface ProjectRBACHook extends Omit<RBACHook, 'canCreate' | 'canRead' | 'canUpdate' | 'canDelete' | 'canManage'> {
  canCreateInProject: (resource: ResourceType) => boolean;
  canReadInProject: (resource: ResourceType, resourceId?: string, resourceData?: Record<string, any>) => boolean;
  canUpdateInProject: (resource: ResourceType, resourceId?: string, resourceData?: Record<string, any>) => boolean;
  canDeleteInProject: (resource: ResourceType, resourceId?: string, resourceData?: Record<string, any>) => boolean;
  canManageInProject: (resource: ResourceType, resourceId?: string) => boolean;
  isProjectLead: boolean;
  isProjectManager: boolean;
  projectRole: string | null;
}

/**
 * Hook for project-scoped RBAC
 */
export const useProjectRBAC = (projectId: string, organizationId?: string): ProjectRBACHook => {
  const baseRBAC = useRBAC();

  const projectMethods = useMemo(() => ({
    canCreateInProject: (resource: ResourceType) => 
      baseRBAC.canCreate(resource, organizationId, projectId),
    
    canReadInProject: (resource: ResourceType, resourceId?: string, resourceData?: Record<string, any>) =>
      baseRBAC.canRead(resource, resourceId, organizationId, projectId, resourceData),
    
    canUpdateInProject: (resource: ResourceType, resourceId?: string, resourceData?: Record<string, any>) =>
      baseRBAC.canUpdate(resource, resourceId, organizationId, projectId, resourceData),
    
    canDeleteInProject: (resource: ResourceType, resourceId?: string, resourceData?: Record<string, any>) =>
      baseRBAC.canDelete(resource, resourceId, organizationId, projectId, resourceData),
    
    canManageInProject: (resource: ResourceType, resourceId?: string) =>
      baseRBAC.canManage(resource, resourceId, organizationId, projectId),
    
    isProjectLead: baseRBAC.hasRole('lead', `project:${projectId}`),
    isProjectManager: baseRBAC.hasRole('manager', `project:${projectId}`),
    
    projectRole: (() => {
      const roles = baseRBAC.getRoles(`project:${projectId}`);
      return roles.length > 0 ? roles[0] : null;
    })(),
  }), [baseRBAC, projectId, organizationId]);

  return {
    ...baseRBAC,
    ...projectMethods,
  };
};

export default useRBAC;