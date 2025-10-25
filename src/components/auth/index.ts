export { AuthProvider, ProtectedRoute, useAuth } from '../../contexts/AuthContext';
// Removed: RoleManager - using ModernShellPrototype instead
export type { AuthContextType, AuthState, Permission, UserIdentity } from '../../contexts/AuthContext';
// Removed: ProfileManager - using ModernShellPrototype instead
export { AuthStatus } from './AuthStatus';
export { IdentityPicker } from './IdentityPicker';
export { IdentitySwitchMenu } from './IdentitySwitchMenu';
export { LoginDialog } from './LoginDialog';
export { PasskeyRegistration } from './PasskeyRegistration';
export { AccessDenied, CreateGuard, DeleteGuard, ManageGuard, RBACGuard, UpdateGuard } from './RBACGuard';
