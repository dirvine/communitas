import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react'
import { authService, SessionInfo, AuthToken } from '../../services/security/authenticationService'
import { NetworkIdentity } from '../../types/collaboration'
import { Box, CircularProgress, Alert, Button } from '@mui/material'
import { Login } from '@mui/icons-material'

interface AuthContextType {
  session: SessionInfo | null
  isAuthenticated: boolean
  isLoading: boolean
  login: (identity: NetworkIdentity, password: string) => Promise<boolean>
  logout: () => Promise<void>
  hasPermission: (action: string, resource: string, context?: Record<string, any>) => boolean
}

const AuthContext = createContext<AuthContextType | null>(null)

interface AuthProviderProps {
  children: ReactNode
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [session, setSession] = useState<SessionInfo | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [currentToken, setCurrentToken] = useState<string | null>(null)

  // Check for existing session on mount
  useEffect(() => {
    const checkExistingSession = async () => {
      const savedToken = localStorage.getItem('auth_token')
      if (savedToken) {
        const validSession = await authService.validateToken(savedToken)
        if (validSession) {
          setSession(validSession)
          setCurrentToken(savedToken)
        } else {
          localStorage.removeItem('auth_token')
        }
      }
      setIsLoading(false)
    }

    checkExistingSession()
  }, [])

  // Set up token refresh interval
  useEffect(() => {
    if (currentToken) {
      const refreshInterval = setInterval(async () => {
        const refreshToken = localStorage.getItem('refresh_token')
        if (refreshToken) {
          const newToken = await authService.refreshToken(refreshToken)
          if (newToken) {
            localStorage.setItem('auth_token', newToken.token)
            localStorage.setItem('refresh_token', newToken.refreshToken)
            setCurrentToken(newToken.token)
            
            const validSession = await authService.validateToken(newToken.token)
            setSession(validSession)
          } else {
            // Refresh failed, logout
            await logout()
          }
        }
      }, 30 * 60 * 1000) // Refresh every 30 minutes

      return () => clearInterval(refreshInterval)
    }
  }, [currentToken])

  const login = async (identity: NetworkIdentity, password: string): Promise<boolean> => {
    try {
      setIsLoading(true)
      const authToken = await authService.login(identity, password)
      
      localStorage.setItem('auth_token', authToken.token)
      localStorage.setItem('refresh_token', authToken.refreshToken)
      
      setCurrentToken(authToken.token)
      
      const validSession = await authService.validateToken(authToken.token)
      setSession(validSession)
      
      return true
    } catch (error) {
      console.error('Login failed:', error)
      return false
    } finally {
      setIsLoading(false)
    }
  }

  const logout = async (): Promise<void> => {
    if (currentToken) {
      await authService.logout(currentToken)
    }
    
    localStorage.removeItem('auth_token')
    localStorage.removeItem('refresh_token')
    
    setSession(null)
    setCurrentToken(null)
  }

  const hasPermission = (
    action: string,
    resource: string,
    context?: Record<string, any>
  ): boolean => {
    if (!session) return false
    return authService.hasPermission(session, action, resource, context)
  }

  const value: AuthContextType = {
    session,
    isAuthenticated: !!session,
    isLoading,
    login,
    logout,
    hasPermission
  }

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  )
}

export const useAuth = (): AuthContextType => {
  const context = useContext(AuthContext)
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider')
  }
  return context
}

interface AuthGuardProps {
  children: ReactNode
  fallback?: ReactNode
  requireAuth?: boolean
  requiredPermission?: { action: string; resource: string; context?: Record<string, any> }
}

export const AuthGuard: React.FC<AuthGuardProps> = ({
  children,
  fallback,
  requireAuth = true,
  requiredPermission
}) => {
  const { isAuthenticated, isLoading, hasPermission, session } = useAuth()

  if (isLoading) {
    return (
      <Box
        sx={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          height: '100vh'
        }}
      >
        <CircularProgress />
      </Box>
    )
  }

  if (requireAuth && !isAuthenticated) {
    return fallback || <LoginRequired />
  }

  if (requiredPermission && session) {
    const hasRequiredPermission = hasPermission(
      requiredPermission.action,
      requiredPermission.resource,
      requiredPermission.context
    )

    if (!hasRequiredPermission) {
      return <PermissionDenied permission={requiredPermission} />
    }
  }

  return <>{children}</>
}

const LoginRequired: React.FC = () => {
  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'center',
        alignItems: 'center',
        height: '100vh',
        gap: 2,
        p: 3
      }}
    >
      <Login sx={{ fontSize: 64, color: 'primary.main' }} />
      <Alert severity="warning" sx={{ maxWidth: 400 }}>
        Authentication required to access this resource
      </Alert>
      <Button variant="contained" onClick={() => window.location.reload()}>
        Login
      </Button>
    </Box>
  )
}

interface PermissionDeniedProps {
  permission: { action: string; resource: string }
}

const PermissionDenied: React.FC<PermissionDeniedProps> = ({ permission }) => {
  const { logout } = useAuth()

  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'center',
        alignItems: 'center',
        height: '100vh',
        gap: 2,
        p: 3
      }}
    >
      <Alert severity="error" sx={{ maxWidth: 600 }}>
        <Box>
          <strong>Access Denied</strong>
          <br />
          You don't have permission to {permission.action} {permission.resource}
        </Box>
      </Alert>
      <Button variant="outlined" onClick={logout}>
        Switch Account
      </Button>
    </Box>
  )
}

// Hook for permission checking
export const usePermissions = () => {
  const { hasPermission } = useAuth()
  
  return {
    canRead: (resource: string, context?: Record<string, any>) => 
      hasPermission('read', resource, context),
    canWrite: (resource: string, context?: Record<string, any>) => 
      hasPermission('write', resource, context),
    canDelete: (resource: string, context?: Record<string, any>) => 
      hasPermission('delete', resource, context),
    canCollaborate: (resource: string, context?: Record<string, any>) => 
      hasPermission('collaborate', resource, context),
    hasPermission
  }
}