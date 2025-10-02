/**
 * Centralized logging service for production code.
 *
 * This service replaces direct console.* calls which are forbidden in production.
 * In development mode, logs are output to console. In production, they can be
 * sent to Tauri backend for structured logging and monitoring.
 *
 * @example
 * ```typescript
 * import { logger } from './services/LoggingService'
 *
 * logger.info('Operation completed', { userId: '123' })
 * logger.warn('Rate limit approaching', { remaining: 10 })
 * logger.error('Failed to fetch data', { error })
 * ```
 */

export type LogLevel = 'debug' | 'info' | 'warn' | 'error'

export interface LogContext {
  [key: string]: unknown
}

export class LoggingService {
  private enabled: boolean
  private isDevelopment: boolean

  constructor() {
    // Enable logging in development, configurable in production
    this.isDevelopment = import.meta.env.DEV
    this.enabled = this.isDevelopment || import.meta.env.VITE_ENABLE_LOGGING === 'true'
  }

  /**
   * Log debug-level messages (verbose, development only)
   */
  debug(message: string, context?: LogContext): void {
    if (!this.enabled || !this.isDevelopment) return

    if (context) {
      console.debug(`[DEBUG] ${message}`, context)
    } else {
      console.debug(`[DEBUG] ${message}`)
    }
  }

  /**
   * Log informational messages
   */
  info(message: string, context?: LogContext): void {
    if (!this.enabled) return

    if (this.isDevelopment) {
      if (context) {
        console.info(`[INFO] ${message}`, context)
      } else {
        console.info(`[INFO] ${message}`)
      }
    } else {
      this.sendToBackend('info', message, context)
    }
  }

  /**
   * Log warning messages (potential issues)
   */
  warn(message: string, context?: LogContext): void {
    if (this.isDevelopment) {
      if (context) {
        console.warn(`[WARN] ${message}`, context)
      } else {
        console.warn(`[WARN] ${message}`)
      }
    } else {
      this.sendToBackend('warn', message, context)
    }
  }

  /**
   * Log error messages (critical issues)
   */
  error(message: string, context?: LogContext): void {
    if (this.isDevelopment) {
      if (context) {
        console.error(`[ERROR] ${message}`, context)
      } else {
        console.error(`[ERROR] ${message}`)
      }
    } else {
      this.sendToBackend('error', message, context)
    }
  }

  /**
   * Send log to Tauri backend for structured logging
   * In production, this would integrate with backend logging service
   */
  private sendToBackend(level: LogLevel, message: string, context?: LogContext): void {
    // TODO: Implement Tauri command for structured logging
    // For now, store critical logs in localStorage for debugging
    try {
      if (level === 'error' || level === 'warn') {
        const logEntry = {
          timestamp: new Date().toISOString(),
          level,
          message,
          context: context || {},
        }

        const existingLogs = localStorage.getItem('app_logs')
        const logs = existingLogs ? JSON.parse(existingLogs) : []
        logs.push(logEntry)

        // Keep only last 100 logs
        if (logs.length > 100) {
          logs.shift()
        }

        localStorage.setItem('app_logs', JSON.stringify(logs))
      }
    } catch (storageError) {
      // Silently fail - we don't want logging to break the app
    }
  }

  /**
   * Retrieve stored logs (for debugging)
   */
  getLogs(): Array<{ timestamp: string; level: LogLevel; message: string; context: LogContext }> {
    try {
      const stored = localStorage.getItem('app_logs')
      return stored ? JSON.parse(stored) : []
    } catch {
      return []
    }
  }

  /**
   * Clear stored logs
   */
  clearLogs(): void {
    try {
      localStorage.removeItem('app_logs')
    } catch {
      // Silently fail
    }
  }
}

// Export singleton instance
export const logger = new LoggingService()
export default logger
