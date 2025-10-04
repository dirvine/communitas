export { AuthProvider, useAuth, ProtectedRoute } from '../../contexts/AuthContext';
export { LoginDialog } from './LoginDialog';
// Removed: ProfileManager - using ModernShellPrototype instead
export { AuthStatus } from './AuthStatus';
export { IdentityPicker } from './IdentityPicker';
export { IdentitySwitchMenu } from './IdentitySwitchMenu';
export { PasskeyRegistration } from './PasskeyRegistration';
export { RBACGuard, CreateGuard, UpdateGuard, DeleteGuard, ManageGuard, AccessDenied } from './RBACGuard';
// Removed: RoleManager - using ModernShellPrototype instead
export type { UserIdentity, Permission, AuthState, AuthContextType } from '../../contexts/AuthContext';