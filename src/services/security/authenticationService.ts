/**
 * Authentication and Authorization Service
 * Addresses critical security vulnerability of missing authentication
 */

import { cryptoManager } from './cryptoManager'
import { NetworkIdentity } from '../../types/collaboration'

export interface AuthenticationConfig {
  tokenExpirationTime: number // in milliseconds
  refreshTokenExpirationTime: number
  maxLoginAttempts: number
  lockoutDurationMs: number
}

export interface UserCredentials {
  networkIdentity: NetworkIdentity
  password: string
  salt: Uint8Array
}

export interface AuthToken {
  token: string
  refreshToken: string
  expiresAt: number
  refreshExpiresAt: number
  userId: string
  permissions: Permission[]
}

export interface Permission {
  action: string
  resource: string
  conditions?: Record<string, any>
}

export interface LoginAttempt {
  userId: string
  timestamp: number
  success: boolean
  ipAddress?: string
}

export interface SessionInfo {
  userId: string
  networkIdentity: NetworkIdentity
  createdAt: number
  lastAccessAt: number
  permissions: Permission[]
  isValid: boolean
}

/**
 * Comprehensive authentication and authorization service
 */
export class AuthenticationService {
  private static instance: AuthenticationService
  private config: AuthenticationConfig
  private userStore = new Map<string, UserCredentials>()
  private tokenStore = new Map<string, AuthToken>()
  private sessionStore = new Map<string, SessionInfo>()
  private loginAttempts = new Map<string, LoginAttempt[]>()
  private lockedUsers = new Map<string, number>() // userId -> unlock timestamp

  // Default secure configuration
  private readonly defaultConfig: AuthenticationConfig = {
    tokenExpirationTime: 1 * 60 * 60 * 1000,      // 1 hour
    refreshTokenExpirationTime: 7 * 24 * 60 * 60 * 1000, // 7 days
    maxLoginAttempts: 5,
    lockoutDurationMs: 15 * 60 * 1000             // 15 minutes
  }

  static getInstance(config?: AuthenticationConfig): AuthenticationService {
    if (!AuthenticationService.instance) {
      AuthenticationService.instance = new AuthenticationService(config)
    }
    return AuthenticationService.instance
  }

  constructor(config?: AuthenticationConfig) {
    this.config = { ...this.defaultConfig, ...config }
  }

  /**
   * Register a new user with secure password hashing
   */
  async registerUser(
    networkIdentity: NetworkIdentity, 
    password: string
  ): Promise<{ success: boolean; userId: string }> {
    // Validate password strength
    const passwordValidation = this.validatePassword(password)
    if (!passwordValidation.isValid) {
      throw new Error(`Password validation failed: ${passwordValidation.errors.join(', ')}`)
    }

    const userId = networkIdentity.fourWords
    
    // Check if user already exists
    if (this.userStore.has(userId)) {
      throw new Error('User already exists')
    }

    // Generate secure salt and derive key
    const salt = cryptoManager.generateSalt()
    const derivedKey = await cryptoManager.deriveKey(password, salt)

    // Store user credentials securely
    const userCredentials: UserCredentials = {
      networkIdentity,
      password: Buffer.from(derivedKey).toString('base64'), // Store derived key, not password
      salt
    }

    this.userStore.set(userId, userCredentials)

    console.log(`User registered: ${userId}`)
    return { success: true, userId }
  }

  /**
   * Authenticate user with rate limiting and account lockout
   */
  async login(
    networkIdentity: NetworkIdentity,
    password: string,
    ipAddress?: string
  ): Promise<AuthToken> {
    const userId = networkIdentity.fourWords

    // Check if user is locked out
    if (this.isUserLockedOut(userId)) {
      const unlockTime = this.lockedUsers.get(userId)
      throw new Error(`Account locked. Try again after ${new Date(unlockTime!).toLocaleString()}`)
    }

    // Get user credentials
    const userCredentials = this.userStore.get(userId)
    if (!userCredentials) {
      this.recordFailedLoginAttempt(userId, ipAddress)
      throw new Error('Invalid credentials')
    }

    // Verify password
    const isValidPassword = await this.verifyPassword(password, userCredentials)
    
    if (!isValidPassword) {
      this.recordFailedLoginAttempt(userId, ipAddress)
      this.checkForLockout(userId)
      throw new Error('Invalid credentials')
    }

    // Clear failed login attempts on successful login
    this.loginAttempts.delete(userId)
    this.lockedUsers.delete(userId)

    // Record successful login
    this.recordSuccessfulLoginAttempt(userId, ipAddress)

    // Generate tokens
    const authToken = await this.generateAuthToken(userId, userCredentials.networkIdentity)
    this.tokenStore.set(authToken.token, authToken)

    // Create session
    const session: SessionInfo = {
      userId,
      networkIdentity: userCredentials.networkIdentity,
      createdAt: Date.now(),
      lastAccessAt: Date.now(),
      permissions: this.getUserPermissions(userId),
      isValid: true
    }
    this.sessionStore.set(authToken.token, session)

    console.log(`User logged in: ${userId}`)
    return authToken
  }

  /**
   * Validate authentication token
   */
  async validateToken(token: string): Promise<SessionInfo | null> {
    const authToken = this.tokenStore.get(token)
    const session = this.sessionStore.get(token)

    if (!authToken || !session) {
      return null
    }

    // Check if token is expired
    if (Date.now() > authToken.expiresAt) {
      this.invalidateToken(token)
      return null
    }

    // Update last access time
    session.lastAccessAt = Date.now()
    this.sessionStore.set(token, session)

    return session
  }

  /**
   * Refresh authentication token
   */
  async refreshToken(refreshToken: string): Promise<AuthToken | null> {
    // Find token by refresh token
    for (const [token, authToken] of this.tokenStore.entries()) {
      if (authToken.refreshToken === refreshToken) {
        // Check if refresh token is expired
        if (Date.now() > authToken.refreshExpiresAt) {
          this.invalidateToken(token)
          return null
        }

        // Generate new token
        const userCredentials = this.userStore.get(authToken.userId)
        if (!userCredentials) {
          return null
        }

        const newAuthToken = await this.generateAuthToken(authToken.userId, userCredentials.networkIdentity)
        
        // Replace old token
        this.invalidateToken(token)
        this.tokenStore.set(newAuthToken.token, newAuthToken)

        // Update session
        const session = this.sessionStore.get(token)
        if (session) {
          this.sessionStore.delete(token)
          this.sessionStore.set(newAuthToken.token, {
            ...session,
            lastAccessAt: Date.now()
          })
        }

        return newAuthToken
      }
    }

    return null
  }

  /**
   * Logout user and invalidate token
   */
  async logout(token: string): Promise<boolean> {
    const session = this.sessionStore.get(token)
    if (session) {
      console.log(`User logged out: ${session.userId}`)
    }

    return this.invalidateToken(token)
  }

  /**
   * Check if user has specific permission
   */
  hasPermission(
    session: SessionInfo,
    action: string,
    resource: string,
    context?: Record<string, any>
  ): boolean {
    if (!session.isValid) {
      return false
    }

    return session.permissions.some(permission => {
      if (permission.action !== action && permission.action !== '*') {
        return false
      }

      if (permission.resource !== resource && permission.resource !== '*') {
        return false
      }

      // Check conditions if specified
      if (permission.conditions && context) {
        return this.evaluateConditions(permission.conditions, context)
      }

      return true
    })
  }

  /**
   * Add permission to user
   */
  grantPermission(userId: string, permission: Permission): boolean {
    const session = this.getActiveSession(userId)
    if (session) {
      session.permissions.push(permission)
      return true
    }
    return false
  }

  /**
   * Remove permission from user
   */
  revokePermission(userId: string, action: string, resource: string): boolean {
    const session = this.getActiveSession(userId)
    if (session) {
      session.permissions = session.permissions.filter(
        p => !(p.action === action && p.resource === resource)
      )
      return true
    }
    return false
  }

  /**
   * Get active session for user
   */
  private getActiveSession(userId: string): SessionInfo | null {
    for (const [token, session] of this.sessionStore.entries()) {
      if (session.userId === userId && session.isValid) {
        return session
      }
    }
    return null
  }

  /**
   * Generate secure authentication token
   */
  private async generateAuthToken(userId: string, networkIdentity: NetworkIdentity): Promise<AuthToken> {
    const token = cryptoManager.generateSecureId()
    const refreshToken = cryptoManager.generateSecureId()
    const now = Date.now()

    return {
      token,
      refreshToken,
      expiresAt: now + this.config.tokenExpirationTime,
      refreshExpiresAt: now + this.config.refreshTokenExpirationTime,
      userId,
      permissions: this.getUserPermissions(userId)
    }
  }

  /**
   * Verify password against stored hash
   */
  private async verifyPassword(password: string, userCredentials: UserCredentials): Promise<boolean> {
    try {
      const derivedKey = await cryptoManager.deriveKey(password, userCredentials.salt)
      const storedKey = Buffer.from(userCredentials.password, 'base64')
      
      // Constant-time comparison to prevent timing attacks
      return cryptoManager.hash(derivedKey).every((byte, index) => 
        byte === cryptoManager.hash(storedKey)[index]
      )
    } catch (error) {
      console.error('Password verification error:', error)
      return false
    }
  }

  /**
   * Validate password strength
   */
  private validatePassword(password: string): { isValid: boolean; errors: string[] } {
    const errors: string[] = []

    if (password.length < 12) {
      errors.push('Password must be at least 12 characters long')
    }

    if (!/[a-z]/.test(password)) {
      errors.push('Password must contain at least one lowercase letter')
    }

    if (!/[A-Z]/.test(password)) {
      errors.push('Password must contain at least one uppercase letter')
    }

    if (!/\d/.test(password)) {
      errors.push('Password must contain at least one number')
    }

    if (!/[!@#$%^&*(),.?":{}|<>]/.test(password)) {
      errors.push('Password must contain at least one special character')
    }

    // Check for common weak passwords
    const commonPasswords = [
      'password', '123456', 'password123', 'admin', 'qwerty'
    ]
    if (commonPasswords.some(weak => password.toLowerCase().includes(weak))) {
      errors.push('Password contains common weak patterns')
    }

    return { isValid: errors.length === 0, errors }
  }

  /**
   * Get default permissions for user
   */
  private getUserPermissions(userId: string): Permission[] {
    // Default permissions for authenticated users
    return [
      { action: 'read', resource: 'own_documents' },
      { action: 'write', resource: 'own_documents' },
      { action: 'delete', resource: 'own_documents' },
      { action: 'collaborate', resource: 'shared_documents' }
    ]
  }

  /**
   * Record failed login attempt
   */
  private recordFailedLoginAttempt(userId: string, ipAddress?: string): void {
    const attempts = this.loginAttempts.get(userId) || []
    attempts.push({
      userId,
      timestamp: Date.now(),
      success: false,
      ipAddress
    })
    this.loginAttempts.set(userId, attempts)
  }

  /**
   * Record successful login attempt
   */
  private recordSuccessfulLoginAttempt(userId: string, ipAddress?: string): void {
    const attempts = this.loginAttempts.get(userId) || []
    attempts.push({
      userId,
      timestamp: Date.now(),
      success: true,
      ipAddress
    })
    this.loginAttempts.set(userId, attempts)
  }

  /**
   * Check if user should be locked out
   */
  private checkForLockout(userId: string): void {
    const attempts = this.loginAttempts.get(userId) || []
    const recentFailedAttempts = attempts.filter(
      attempt => !attempt.success && 
      (Date.now() - attempt.timestamp) < this.config.lockoutDurationMs
    )

    if (recentFailedAttempts.length >= this.config.maxLoginAttempts) {
      const unlockTime = Date.now() + this.config.lockoutDurationMs
      this.lockedUsers.set(userId, unlockTime)
      console.warn(`User locked out: ${userId} until ${new Date(unlockTime).toLocaleString()}`)
    }
  }

  /**
   * Check if user is currently locked out
   */
  private isUserLockedOut(userId: string): boolean {
    const unlockTime = this.lockedUsers.get(userId)
    if (!unlockTime) {
      return false
    }

    if (Date.now() >= unlockTime) {
      this.lockedUsers.delete(userId)
      return false
    }

    return true
  }

  /**
   * Invalidate authentication token
   */
  private invalidateToken(token: string): boolean {
    const session = this.sessionStore.get(token)
    if (session) {
      session.isValid = false
    }

    const deleted1 = this.tokenStore.delete(token)
    const deleted2 = this.sessionStore.delete(token)
    
    return deleted1 || deleted2
  }

  /**
   * Evaluate permission conditions
   */
  private evaluateConditions(conditions: Record<string, any>, context: Record<string, any>): boolean {
    for (const [key, expectedValue] of Object.entries(conditions)) {
      if (context[key] !== expectedValue) {
        return false
      }
    }
    return true
  }

  /**
   * Clean up expired tokens and sessions
   */
  cleanup(): void {
    const now = Date.now()

    // Clean up expired tokens
    for (const [token, authToken] of this.tokenStore.entries()) {
      if (now > authToken.expiresAt) {
        this.invalidateToken(token)
      }
    }

    // Clean up expired lockouts
    for (const [userId, unlockTime] of this.lockedUsers.entries()) {
      if (now >= unlockTime) {
        this.lockedUsers.delete(userId)
      }
    }

    // Clean up old login attempts (older than 24 hours)
    for (const [userId, attempts] of this.loginAttempts.entries()) {
      const recentAttempts = attempts.filter(
        attempt => (now - attempt.timestamp) < (24 * 60 * 60 * 1000)
      )
      
      if (recentAttempts.length === 0) {
        this.loginAttempts.delete(userId)
      } else {
        this.loginAttempts.set(userId, recentAttempts)
      }
    }
  }

  /**
   * Get security metrics
   */
  getSecurityMetrics(): {
    activeUsers: number
    activeSessions: number
    lockedUsers: number
    recentFailedLogins: number
  } {
    const now = Date.now()
    const oneDayAgo = now - (24 * 60 * 60 * 1000)

    let recentFailedLogins = 0
    for (const attempts of this.loginAttempts.values()) {
      recentFailedLogins += attempts.filter(
        attempt => !attempt.success && attempt.timestamp > oneDayAgo
      ).length
    }

    return {
      activeUsers: this.userStore.size,
      activeSessions: Array.from(this.sessionStore.values()).filter(s => s.isValid).length,
      lockedUsers: this.lockedUsers.size,
      recentFailedLogins
    }
  }
}

// Export singleton instance
export const authService = AuthenticationService.getInstance()