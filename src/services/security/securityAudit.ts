/**
 * Security Audit and Monitoring Service
 * Provides comprehensive security monitoring, logging, and compliance
 */

import { cryptoManager } from './cryptoManager'
import { authService } from './authenticationService'

export interface SecurityEvent {
  id: string
  timestamp: number
  eventType: SecurityEventType
  severity: SecuritySeverity
  userId?: string
  ipAddress?: string
  userAgent?: string
  resource?: string
  action?: string
  details: Record<string, any>
  riskScore: number
}

export enum SecurityEventType {
  // Authentication events
  LOGIN_SUCCESS = 'login_success',
  LOGIN_FAILURE = 'login_failure',
  LOGOUT = 'logout',
  PASSWORD_CHANGE = 'password_change',
  ACCOUNT_LOCKOUT = 'account_lockout',

  // Authorization events
  PERMISSION_GRANTED = 'permission_granted',
  PERMISSION_REVOKED = 'permission_revoked',
  UNAUTHORIZED_ACCESS = 'unauthorized_access',
  ELEVATED_PRIVILEGE = 'elevated_privilege',

  // Data protection events
  DATA_ENCRYPTION = 'data_encryption',
  DATA_DECRYPTION = 'data_decryption',
  KEY_ROTATION = 'key_rotation',
  SENSITIVE_DATA_ACCESS = 'sensitive_data_access',

  // Network security events
  SUSPICIOUS_TRAFFIC = 'suspicious_traffic',
  BRUTE_FORCE_ATTEMPT = 'brute_force_attempt',
  SQL_INJECTION_ATTEMPT = 'sql_injection_attempt',
  XSS_ATTEMPT = 'xss_attempt',

  // System security events
  CONFIGURATION_CHANGE = 'configuration_change',
  SECURITY_POLICY_VIOLATION = 'security_policy_violation',
  INTEGRITY_CHECK_FAILURE = 'integrity_check_failure',
  ANOMALOUS_ACTIVITY = 'anomalous_activity'
}

export enum SecuritySeverity {
  LOW = 'low',
  MEDIUM = 'medium',
  HIGH = 'high',
  CRITICAL = 'critical'
}

export interface SecurityMetrics {
  totalEvents: number
  eventsBySeverity: Record<SecuritySeverity, number>
  eventsByType: Record<SecurityEventType, number>
  recentIncidents: SecurityEvent[]
  riskScore: number
  complianceStatus: ComplianceStatus
}

export interface ComplianceStatus {
  gdprCompliant: boolean
  hipaaCompliant: boolean
  soc2Compliant: boolean
  lastAuditDate: number
  nextAuditDate: number
  issues: string[]
}

export class SecurityAuditService {
  private static instance: SecurityAuditService
  private events: SecurityEvent[] = []
  private readonly maxEvents = 10000 // Keep last 10k events
  private readonly retentionPeriod = 90 * 24 * 60 * 60 * 1000 // 90 days

  static getInstance(): SecurityAuditService {
    if (!SecurityAuditService.instance) {
      SecurityAuditService.instance = new SecurityAuditService()
    }
    return SecurityAuditService.instance
  }

  /**
   * Log a security event
   */
  async logEvent(
    eventType: SecurityEventType,
    severity: SecuritySeverity,
    details: Record<string, any> = {},
    userId?: string,
    ipAddress?: string,
    userAgent?: string
  ): Promise<void> {
    const event: SecurityEvent = {
      id: cryptoManager.generateSecureId(),
      timestamp: Date.now(),
      eventType,
      severity,
      userId,
      ipAddress,
      userAgent,
      details,
      riskScore: this.calculateRiskScore(eventType, severity, details)
    }

    this.events.push(event)

    // Maintain event limit
    if (this.events.length > this.maxEvents) {
      this.events = this.events.slice(-this.maxEvents)
    }

    // Log critical events immediately
    if (severity === SecuritySeverity.CRITICAL) {
      console.error('CRITICAL SECURITY EVENT:', event)
      await this.alertSecurityTeam(event)
    }

    // Log high severity events
    if (severity === SecuritySeverity.HIGH) {
      console.warn('HIGH SEVERITY SECURITY EVENT:', event)
    }
  }

  /**
   * Get security metrics
   */
  getSecurityMetrics(timeRangeMs: number = 24 * 60 * 60 * 1000): SecurityMetrics {
    const now = Date.now()
    const cutoff = now - timeRangeMs

    const recentEvents = this.events.filter(e => e.timestamp >= cutoff)

    const eventsBySeverity = recentEvents.reduce((acc, event) => {
      acc[event.severity] = (acc[event.severity] || 0) + 1
      return acc
    }, {} as Record<SecuritySeverity, number>)

    const eventsByType = recentEvents.reduce((acc, event) => {
      acc[event.eventType] = (acc[event.eventType] || 0) + 1
      return acc
    }, {} as Record<SecurityEventType, number>)

    const riskScore = recentEvents.reduce((sum, event) => sum + event.riskScore, 0) / Math.max(recentEvents.length, 1)

    return {
      totalEvents: recentEvents.length,
      eventsBySeverity,
      eventsByType,
      recentIncidents: recentEvents.filter(e => e.severity === SecuritySeverity.HIGH || e.severity === SecuritySeverity.CRITICAL).slice(-10),
      riskScore,
      complianceStatus: this.getComplianceStatus()
    }
  }

  /**
   * Perform security audit
   */
  async performSecurityAudit(): Promise<{
    passed: boolean
    issues: string[]
    recommendations: string[]
    score: number
  }> {
    const issues: string[] = []
    const recommendations: string[] = []

    // Check authentication security
    const authMetrics = authService.getSecurityMetrics()
    if (authMetrics.lockedUsers > 0) {
      issues.push(`${authMetrics.lockedUsers} users are currently locked out`)
    }
    if (authMetrics.recentFailedLogins > 10) {
      issues.push(`High number of failed login attempts: ${authMetrics.recentFailedLogins}`)
      recommendations.push('Consider implementing additional authentication measures')
    }

    // Check for suspicious patterns
    const suspiciousEvents = this.events.filter(e =>
      e.eventType === SecurityEventType.BRUTE_FORCE_ATTEMPT ||
      e.eventType === SecurityEventType.UNAUTHORIZED_ACCESS
    ).length

    if (suspiciousEvents > 5) {
      issues.push(`${suspiciousEvents} suspicious security events detected`)
      recommendations.push('Review access logs and consider IP blocking')
    }

    // Check key expiration
    const keys = cryptoManager.listKeyPairs()
    const expiredKeys = keys.filter(key => key.expiresAt && Date.now() > key.expiresAt).length
    if (expiredKeys > 0) {
      issues.push(`${expiredKeys} cryptographic keys have expired`)
      recommendations.push('Rotate expired keys immediately')
    }

    // Calculate security score (0-100)
    const baseScore = 100
    const deductions = issues.length * 10 + recommendations.length * 5
    const score = Math.max(0, baseScore - deductions)

    return {
      passed: issues.length === 0,
      issues,
      recommendations,
      score
    }
  }

  /**
   * Get compliance status
   */
  private getComplianceStatus(): ComplianceStatus {
    const now = Date.now()
    const lastAudit = now - (30 * 24 * 60 * 60 * 1000) // 30 days ago
    const nextAudit = now + (90 * 24 * 60 * 60 * 1000) // 90 days from now

    const issues: string[] = []

    // Check for compliance issues
    const recentEvents = this.events.filter(e => e.timestamp >= lastAudit)
    const criticalEvents = recentEvents.filter(e => e.severity === SecuritySeverity.CRITICAL).length

    if (criticalEvents > 0) {
      issues.push(`${criticalEvents} critical security events in last audit period`)
    }

    return {
      gdprCompliant: issues.length === 0,
      hipaaCompliant: issues.length === 0,
      soc2Compliant: issues.length === 0,
      lastAuditDate: lastAudit,
      nextAuditDate: nextAudit,
      issues
    }
  }

  /**
   * Calculate risk score for an event
   */
  private calculateRiskScore(eventType: SecurityEventType, severity: SecuritySeverity, details: Record<string, any>): number {
    let baseScore = 0

    // Base score by severity
    switch (severity) {
      case SecuritySeverity.LOW: baseScore = 10; break
      case SecuritySeverity.MEDIUM: baseScore = 30; break
      case SecuritySeverity.HIGH: baseScore = 70; break
      case SecuritySeverity.CRITICAL: baseScore = 100; break
    }

    // Adjust by event type
    switch (eventType) {
      case SecurityEventType.UNAUTHORIZED_ACCESS:
      case SecurityEventType.BRUTE_FORCE_ATTEMPT:
      case SecurityEventType.SQL_INJECTION_ATTEMPT:
      case SecurityEventType.XSS_ATTEMPT:
        baseScore += 20
        break
      case SecurityEventType.LOGIN_FAILURE:
        baseScore += 5
        break
    }

    // Adjust by details
    if (details.ipAddress && this.isSuspiciousIP(details.ipAddress)) {
      baseScore += 15
    }

    return Math.min(100, baseScore)
  }

  /**
   * Check if IP address is suspicious
   */
  private isSuspiciousIP(ipAddress: string): boolean {
    // Simple check for known suspicious patterns
    // In production, this would integrate with threat intelligence feeds
    const suspiciousPatterns = [
      /^192\.168\./,  // Private network (shouldn't be in logs)
      /^10\./,        // Private network
      /^172\./,       // Private network
    ]

    return suspiciousPatterns.some(pattern => pattern.test(ipAddress))
  }

  /**
   * Alert security team for critical events
   */
  private async alertSecurityTeam(event: SecurityEvent): Promise<void> {
    // In production, this would send alerts via email, Slack, etc.
    console.error('🚨 SECURITY ALERT:', {
      event: event.eventType,
      severity: event.severity,
      userId: event.userId,
      timestamp: new Date(event.timestamp).toISOString(),
      details: event.details
    })

    // Could integrate with external alerting systems here
  }

  /**
   * Clean up old events
   */
  cleanup(): void {
    const cutoff = Date.now() - this.retentionPeriod
    this.events = this.events.filter(event => event.timestamp >= cutoff)
  }

  /**
   * Export security events for compliance
   */
  exportEvents(startDate: number, endDate: number): SecurityEvent[] {
    return this.events.filter(event =>
      event.timestamp >= startDate && event.timestamp <= endDate
    )
  }

  /**
   * Get events for a specific user
   */
  getUserEvents(userId: string, limit: number = 100): SecurityEvent[] {
    return this.events
      .filter(event => event.userId === userId)
      .slice(-limit)
  }
}

// Export singleton instance
export const securityAudit = SecurityAuditService.getInstance()

// Helper functions for common security events
export const logSecurityEvent = (
  eventType: SecurityEventType,
  severity: SecuritySeverity,
  details: Record<string, any> = {},
  userId?: string,
  ipAddress?: string,
  userAgent?: string
) => {
  securityAudit.logEvent(eventType, severity, details, userId, ipAddress, userAgent)
}

export const logAuthSuccess = (userId: string, ipAddress?: string) => {
  logSecurityEvent(SecurityEventType.LOGIN_SUCCESS, SecuritySeverity.LOW, {}, userId, ipAddress)
}

export const logAuthFailure = (userId: string, ipAddress?: string, reason?: string) => {
  logSecurityEvent(
    SecurityEventType.LOGIN_FAILURE,
    SecuritySeverity.MEDIUM,
    { reason },
    userId,
    ipAddress
  )
}

export const logUnauthorizedAccess = (userId: string, resource: string, ipAddress?: string) => {
  logSecurityEvent(
    SecurityEventType.UNAUTHORIZED_ACCESS,
    SecuritySeverity.HIGH,
    { resource },
    userId,
    ipAddress
  )
}

export const logSuspiciousActivity = (activity: string, details: Record<string, any>, ipAddress?: string) => {
  logSecurityEvent(
    SecurityEventType.ANOMALOUS_ACTIVITY,
    SecuritySeverity.HIGH,
    { activity, ...details },
    undefined,
    ipAddress
  )
}