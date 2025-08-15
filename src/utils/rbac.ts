import { UserIdentity, Permission } from '../contexts/AuthContext';

// Role definitions
export enum SystemRole {
  SUPER_ADMIN = 'super_admin',
  PLATFORM_ADMIN = 'platform_admin',
  USER = 'user',
}

export enum OrganizationRole {
  OWNER = 'owner',
  ADMIN = 'admin',
  MODERATOR = 'moderator',
  MEMBER = 'member',
  GUEST = 'guest',
}

export enum ProjectRole {
  LEAD = 'lead',
  MANAGER = 'manager',
  CONTRIBUTOR = 'contributor',
  VIEWER = 'viewer',
}

// Resource types
export enum ResourceType {
  SYSTEM = 'system',
  ORGANIZATION = 'organization',
  PROJECT = 'project',
  FILE = 'file',
  MESSAGE = 'message',
  USER = 'user',
  WEBSITE = 'website',
  BLOG = 'blog',
  CALL = 'call',
}

// Action types
export enum Action {
  CREATE = 'create',
  READ = 'read',
  UPDATE = 'update',
  DELETE = 'delete',
  MANAGE = 'manage',
  INVITE = 'invite',
  REMOVE = 'remove',
  MODERATE = 'moderate',
  PUBLISH = 'publish',
  ARCHIVE = 'archive',
}

// Role-based permission mappings
export interface RolePermission {
  role: string;
  resource: ResourceType;
  actions: Action[];
  conditions?: PermissionCondition[];
}

// Permission conditions for context-aware access control
export interface PermissionCondition {
  field: string;
  operator: 'equals' | 'not_equals' | 'contains' | 'not_contains' | 'greater_than' | 'less_than';
  value: any;
}

// Permission check context
export interface PermissionContext {
  user: UserIdentity;
  resource: ResourceType;
  action: Action;
  resourceId?: string;
  organizationId?: string;
  projectId?: string;
  resourceData?: Record<string, any>;
}

// Default role permissions
export const DEFAULT_ROLE_PERMISSIONS: RolePermission[] = [
  // System roles
  {
    role: SystemRole.SUPER_ADMIN,
    resource: ResourceType.SYSTEM,
    actions: [Action.CREATE, Action.READ, Action.UPDATE, Action.DELETE, Action.MANAGE],
  },
  {
    role: SystemRole.PLATFORM_ADMIN,
    resource: ResourceType.SYSTEM,
    actions: [Action.READ, Action.UPDATE, Action.MANAGE],
  },
  {
    role: SystemRole.USER,
    resource: ResourceType.SYSTEM,
    actions: [Action.READ],
  },

  // Organization roles
  {
    role: OrganizationRole.OWNER,
    resource: ResourceType.ORGANIZATION,
    actions: [Action.CREATE, Action.READ, Action.UPDATE, Action.DELETE, Action.MANAGE, Action.INVITE, Action.REMOVE],
  },
  {
    role: OrganizationRole.ADMIN,
    resource: ResourceType.ORGANIZATION,
    actions: [Action.READ, Action.UPDATE, Action.MANAGE, Action.INVITE, Action.REMOVE, Action.MODERATE],
  },
  {
    role: OrganizationRole.MODERATOR,
    resource: ResourceType.ORGANIZATION,
    actions: [Action.READ, Action.UPDATE, Action.MODERATE],
  },
  {
    role: OrganizationRole.MEMBER,
    resource: ResourceType.ORGANIZATION,
    actions: [Action.READ, Action.CREATE],
  },
  {
    role: OrganizationRole.GUEST,
    resource: ResourceType.ORGANIZATION,
    actions: [Action.READ],
  },

  // Project roles
  {
    role: ProjectRole.LEAD,
    resource: ResourceType.PROJECT,
    actions: [Action.CREATE, Action.READ, Action.UPDATE, Action.DELETE, Action.MANAGE, Action.INVITE, Action.REMOVE],
  },
  {
    role: ProjectRole.MANAGER,
    resource: ResourceType.PROJECT,
    actions: [Action.READ, Action.UPDATE, Action.MANAGE, Action.INVITE],
  },
  {
    role: ProjectRole.CONTRIBUTOR,
    resource: ResourceType.PROJECT,
    actions: [Action.READ, Action.CREATE, Action.UPDATE],
    conditions: [
      { field: 'ownerId', operator: 'equals', value: '{{userId}}' }
    ],
  },
  {
    role: ProjectRole.VIEWER,
    resource: ResourceType.PROJECT,
    actions: [Action.READ],
  },

  // File permissions
  {
    role: OrganizationRole.MEMBER,
    resource: ResourceType.FILE,
    actions: [Action.CREATE, Action.READ, Action.UPDATE],
    conditions: [
      { field: 'ownerId', operator: 'equals', value: '{{userId}}' }
    ],
  },
  {
    role: OrganizationRole.ADMIN,
    resource: ResourceType.FILE,
    actions: [Action.CREATE, Action.READ, Action.UPDATE, Action.DELETE, Action.MANAGE],
  },

  // Message permissions
  {
    role: OrganizationRole.MEMBER,
    resource: ResourceType.MESSAGE,
    actions: [Action.CREATE, Action.READ, Action.UPDATE],
    conditions: [
      { field: 'senderId', operator: 'equals', value: '{{userId}}' }
    ],
  },
  {
    role: OrganizationRole.MODERATOR,
    resource: ResourceType.MESSAGE,
    actions: [Action.CREATE, Action.READ, Action.UPDATE, Action.DELETE, Action.MODERATE],
  },

  // Website/Blog permissions
  {
    role: OrganizationRole.MEMBER,
    resource: ResourceType.WEBSITE,
    actions: [Action.CREATE, Action.READ, Action.UPDATE],
  },
  {
    role: OrganizationRole.ADMIN,
    resource: ResourceType.WEBSITE,
    actions: [Action.CREATE, Action.READ, Action.UPDATE, Action.DELETE, Action.PUBLISH],
  },
  {
    role: OrganizationRole.MEMBER,
    resource: ResourceType.BLOG,
    actions: [Action.CREATE, Action.READ, Action.UPDATE],
    conditions: [
      { field: 'authorId', operator: 'equals', value: '{{userId}}' }
    ],
  },
  {
    role: OrganizationRole.ADMIN,
    resource: ResourceType.BLOG,
    actions: [Action.CREATE, Action.READ, Action.UPDATE, Action.DELETE, Action.PUBLISH],
  },

  // Call permissions
  {
    role: OrganizationRole.MEMBER,
    resource: ResourceType.CALL,
    actions: [Action.CREATE, Action.READ],
  },
  {
    role: OrganizationRole.MODERATOR,
    resource: ResourceType.CALL,
    actions: [Action.CREATE, Action.READ, Action.MODERATE],
  },
];

// RBAC Utility Class
export class RBACManager {
  private rolePermissions: RolePermission[];

  constructor(rolePermissions: RolePermission[] = DEFAULT_ROLE_PERMISSIONS) {
    this.rolePermissions = rolePermissions;
  }

  /**
   * Check if a user has permission to perform an action on a resource
   */
  hasPermission(context: PermissionContext): boolean {
    const { user, resource, action } = context;

    // Check system-level permissions first
    if (this.hasSystemPermission(user, resource, action)) {
      return true;
    }

    // Check role-based permissions
    const userRoles = this.getUserRoles(user, context);
    
    for (const role of userRoles) {
      if (this.roleHasPermission(role, resource, action, context)) {
        return true;
      }
    }

    return false;
  }

  /**
   * Check if a user has system-level permissions (super admin, etc.)
   */
  private hasSystemPermission(user: UserIdentity, resource: ResourceType, action: Action): boolean {
    // Check if user has super admin permissions
    const systemPermissions = user.permissions.filter(p => p.resource === ResourceType.SYSTEM);
    
    for (const permission of systemPermissions) {
      if (permission.actions.includes(action.toString()) || permission.actions.includes(Action.MANAGE)) {
        return true;
      }
    }

    return false;
  }

  /**
   * Get all roles for a user in the given context
   */
  private getUserRoles(user: UserIdentity, context: PermissionContext): string[] {
    const roles: string[] = [];

    // Extract roles from user permissions
    for (const permission of user.permissions) {
      if (permission.scope) {
        // Context-specific role (e.g., organization:123:admin)
        const [scopeType, scopeId, role] = permission.scope.split(':');
        
        if (
          (scopeType === 'organization' && context.organizationId === scopeId) ||
          (scopeType === 'project' && context.projectId === scopeId)
        ) {
          roles.push(role);
        }
      } else {
        // System-wide role
        roles.push(permission.resource);
      }
    }

    return roles;
  }

  /**
   * Check if a specific role has permission for an action on a resource
   */
  private roleHasPermission(
    role: string,
    resource: ResourceType,
    action: Action,
    context: PermissionContext
  ): boolean {
    const rolePermissions = this.rolePermissions.filter(
      p => p.role === role && (p.resource === resource || p.resource === ResourceType.SYSTEM)
    );

    for (const rolePermission of rolePermissions) {
      if (
        rolePermission.actions.includes(action) ||
        rolePermission.actions.includes(Action.MANAGE)
      ) {
        // Check conditions if they exist
        if (rolePermission.conditions) {
          if (this.checkConditions(rolePermission.conditions, context)) {
            return true;
          }
        } else {
          return true;
        }
      }
    }

    return false;
  }

  /**
   * Check if all conditions are met for a permission
   */
  private checkConditions(conditions: PermissionCondition[], context: PermissionContext): boolean {
    for (const condition of conditions) {
      if (!this.checkCondition(condition, context)) {
        return false;
      }
    }
    return true;
  }

  /**
   * Check a single condition
   */
  private checkCondition(condition: PermissionCondition, context: PermissionContext): boolean {
    let actualValue: any;
    let expectedValue = condition.value;

    // Handle template values
    if (typeof expectedValue === 'string' && expectedValue.includes('{{')) {
      expectedValue = this.resolveTemplateValue(expectedValue, context);
    }

    // Get actual value from context
    if (context.resourceData && condition.field in context.resourceData) {
      actualValue = context.resourceData[condition.field];
    } else if (condition.field === 'userId') {
      actualValue = context.user.id;
    }

    // Perform comparison based on operator
    switch (condition.operator) {
      case 'equals':
        return actualValue === expectedValue;
      case 'not_equals':
        return actualValue !== expectedValue;
      case 'contains':
        return Array.isArray(actualValue) 
          ? actualValue.includes(expectedValue)
          : String(actualValue).includes(String(expectedValue));
      case 'not_contains':
        return Array.isArray(actualValue)
          ? !actualValue.includes(expectedValue)
          : !String(actualValue).includes(String(expectedValue));
      case 'greater_than':
        return Number(actualValue) > Number(expectedValue);
      case 'less_than':
        return Number(actualValue) < Number(expectedValue);
      default:
        return false;
    }
  }

  /**
   * Resolve template values like {{userId}}
   */
  private resolveTemplateValue(template: string, context: PermissionContext): any {
    return template.replace(/\{\{(\w+)\}\}/g, (match, field) => {
      switch (field) {
        case 'userId':
          return context.user.id;
        case 'organizationId':
          return context.organizationId || '';
        case 'projectId':
          return context.projectId || '';
        default:
          return match;
      }
    });
  }

  /**
   * Get all permitted actions for a user on a resource
   */
  getPermittedActions(
    user: UserIdentity,
    resource: ResourceType,
    resourceId?: string,
    organizationId?: string,
    projectId?: string,
    resourceData?: Record<string, any>
  ): Action[] {
    const permittedActions: Set<Action> = new Set();

    for (const action of Object.values(Action)) {
      const context: PermissionContext = {
        user,
        resource,
        action,
        resourceId,
        organizationId,
        projectId,
        resourceData,
      };

      if (this.hasPermission(context)) {
        permittedActions.add(action);
      }
    }

    return Array.from(permittedActions);
  }

  /**
   * Check if user can access a specific resource
   */
  canAccess(
    user: UserIdentity,
    resource: ResourceType,
    requiredAction: Action = Action.READ,
    resourceId?: string,
    organizationId?: string,
    projectId?: string,
    resourceData?: Record<string, any>
  ): boolean {
    const context: PermissionContext = {
      user,
      resource,
      action: requiredAction,
      resourceId,
      organizationId,
      projectId,
      resourceData,
    };

    return this.hasPermission(context);
  }
}

// Create singleton instance
export const rbacManager = new RBACManager();

// Utility functions for common permission checks
export const canCreateResource = (
  user: UserIdentity,
  resource: ResourceType,
  organizationId?: string,
  projectId?: string
): boolean => {
  return rbacManager.canAccess(user, resource, Action.CREATE, undefined, organizationId, projectId);
};

export const canReadResource = (
  user: UserIdentity,
  resource: ResourceType,
  resourceId?: string,
  organizationId?: string,
  projectId?: string,
  resourceData?: Record<string, any>
): boolean => {
  return rbacManager.canAccess(user, resource, Action.READ, resourceId, organizationId, projectId, resourceData);
};

export const canUpdateResource = (
  user: UserIdentity,
  resource: ResourceType,
  resourceId?: string,
  organizationId?: string,
  projectId?: string,
  resourceData?: Record<string, any>
): boolean => {
  return rbacManager.canAccess(user, resource, Action.UPDATE, resourceId, organizationId, projectId, resourceData);
};

export const canDeleteResource = (
  user: UserIdentity,
  resource: ResourceType,
  resourceId?: string,
  organizationId?: string,
  projectId?: string,
  resourceData?: Record<string, any>
): boolean => {
  return rbacManager.canAccess(user, resource, Action.DELETE, resourceId, organizationId, projectId, resourceData);
};

export const canManageResource = (
  user: UserIdentity,
  resource: ResourceType,
  resourceId?: string,
  organizationId?: string,
  projectId?: string
): boolean => {
  return rbacManager.canAccess(user, resource, Action.MANAGE, resourceId, organizationId, projectId);
};