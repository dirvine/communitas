/**
 * Feature Flags System for Progressive Migration
 * Allows gradual rollout of new unified platform features
 */

type FeatureFlag = {
  name: string
  description: string
  enabled: boolean
  rolloutPercentage?: number
  enabledForUsers?: string[]
  phase: 1 | 2 | 3 | 4
}

class FeatureFlagsService {
  private flags: Map<string, FeatureFlag>
  private userOverrides: Map<string, Set<string>>
  
  constructor() {
    this.flags = new Map()
    this.userOverrides = new Map()
    this.initializeFlags()
  }

  private initializeFlags() {
    // Phase 1: Foundation (Weeks 1-4)
    this.register({
      name: 'unified-design-system',
      description: 'New modern design system with glassmorphism and smooth animations',
      enabled: false,
      phase: 1
    })

    this.register({
      name: 'context-aware-navigation',
      description: 'Adaptive navigation that changes based on Personal/Org/Project context',
      enabled: false,
      phase: 1
    })

    this.register({
      name: 'four-word-identity',
      description: 'Human-readable four-word identity system',
      enabled: false,
      phase: 1
    })

    this.register({
      name: 'unified-storage-ui',
      description: 'New tree-based storage interface with preview',
      enabled: false,
      phase: 1
    })

    // Phase 2: Communication (Weeks 5-8)
    this.register({
      name: 'rich-messaging',
      description: 'Enhanced messaging with media, threading, and reactions',
      enabled: false,
      phase: 2
    })

    this.register({
      name: 'voice-video-calls',
      description: 'Integrated voice and video calling',
      enabled: false,
      phase: 2
    })

    this.register({
      name: 'message-threading',
      description: 'Slack-style message threads',
      enabled: false,
      phase: 2
    })

    this.register({
      name: 'emoji-reactions',
      description: 'React to messages with emojis',
      enabled: false,
      phase: 2
    })

    // Phase 3: Organization (Weeks 9-12)
    this.register({
      name: 'organization-hierarchy',
      description: 'Full organization structure with teams and projects',
      enabled: false,
      phase: 3
    })

    this.register({
      name: 'team-channels',
      description: 'Slack-like channels for teams',
      enabled: false,
      phase: 3
    })

    this.register({
      name: 'project-management',
      description: 'Project spaces with dedicated resources',
      enabled: false,
      phase: 3
    })

    this.register({
      name: 'permission-system',
      description: 'Granular permissions for orgs and projects',
      enabled: false,
      phase: 3
    })

    // Phase 4: Storage & Publishing (Weeks 13-16)
    this.register({
      name: 'website-builder',
      description: 'Markdown-based website builder and publisher',
      enabled: false,
      phase: 4
    })

    this.register({
      name: 'version-control',
      description: 'Git-like version control for files',
      enabled: false,
      phase: 4
    })

    this.register({
      name: 'dht-publishing',
      description: 'Publish websites to DHT with four-word addresses',
      enabled: false,
      phase: 4
    })

    this.register({
      name: 'interlinked-websites',
      description: 'Link websites between orgs, projects, and users',
      enabled: false,
      phase: 4
    })
  }

  private register(flag: FeatureFlag) {
    this.flags.set(flag.name, flag)
  }

  /**
   * Check if a feature is enabled for the current user
   */
  isEnabled(flagName: string, userId?: string): boolean {
    const flag = this.flags.get(flagName)
    if (!flag) {
      console.warn(`Unknown feature flag: ${flagName}`)
      return false
    }

    // Check user-specific override
    if (userId && this.userOverrides.has(userId)) {
      const overrides = this.userOverrides.get(userId)!
      if (overrides.has(flagName)) {
        return true
      }
    }

    // Check if user is in enabled list
    if (userId && flag.enabledForUsers?.includes(userId)) {
      return true
    }

    // Check rollout percentage
    if (flag.rolloutPercentage !== undefined && userId) {
      const hash = this.hashUserId(userId)
      const percentage = (hash % 100) + 1
      return percentage <= flag.rolloutPercentage
    }

    return flag.enabled
  }

  /**
   * Enable a feature globally
   */
  enable(flagName: string) {
    const flag = this.flags.get(flagName)
    if (flag) {
      flag.enabled = true
      this.persistFlags()
    }
  }

  /**
   * Disable a feature globally
   */
  disable(flagName: string) {
    const flag = this.flags.get(flagName)
    if (flag) {
      flag.enabled = false
      this.persistFlags()
    }
  }

  /**
   * Set rollout percentage for gradual deployment
   */
  setRolloutPercentage(flagName: string, percentage: number) {
    const flag = this.flags.get(flagName)
    if (flag) {
      flag.rolloutPercentage = Math.min(100, Math.max(0, percentage))
      this.persistFlags()
    }
  }

  /**
   * Enable a feature for specific users (beta testers)
   */
  enableForUsers(flagName: string, userIds: string[]) {
    const flag = this.flags.get(flagName)
    if (flag) {
      flag.enabledForUsers = [...(flag.enabledForUsers || []), ...userIds]
      this.persistFlags()
    }
  }

  /**
   * Override flags for a specific user session
   */
  setUserOverride(userId: string, flagName: string, enabled: boolean) {
    if (!this.userOverrides.has(userId)) {
      this.userOverrides.set(userId, new Set())
    }
    
    const overrides = this.userOverrides.get(userId)!
    if (enabled) {
      overrides.add(flagName)
    } else {
      overrides.delete(flagName)
    }
  }

  /**
   * Get all flags for a specific phase
   */
  getFlagsForPhase(phase: 1 | 2 | 3 | 4): FeatureFlag[] {
    return Array.from(this.flags.values()).filter(f => f.phase === phase)
  }

  /**
   * Get all enabled flags
   */
  getEnabledFlags(): FeatureFlag[] {
    return Array.from(this.flags.values()).filter(f => f.enabled)
  }

  /**
   * Get flag configuration for debugging
   */
  getDebugInfo(): Record<string, any> {
    const info: Record<string, any> = {}
    this.flags.forEach((flag, name) => {
      info[name] = {
        enabled: flag.enabled,
        phase: flag.phase,
        rollout: flag.rolloutPercentage,
        users: flag.enabledForUsers?.length || 0
      }
    })
    return info
  }

  /**
   * Load flags from localStorage
   */
  private loadFlags() {
    try {
      const stored = localStorage.getItem('communitas-feature-flags')
      if (stored) {
        const data = JSON.parse(stored)
        Object.entries(data).forEach(([name, config]: [string, any]) => {
          const flag = this.flags.get(name)
          if (flag) {
            Object.assign(flag, config)
          }
        })
      }
    } catch (e) {
      console.error('Failed to load feature flags:', e)
    }
  }

  /**
   * Persist flags to localStorage
   */
  private persistFlags() {
    try {
      const data: Record<string, any> = {}
      this.flags.forEach((flag, name) => {
        data[name] = {
          enabled: flag.enabled,
          rolloutPercentage: flag.rolloutPercentage,
          enabledForUsers: flag.enabledForUsers
        }
      })
      localStorage.setItem('communitas-feature-flags', JSON.stringify(data))
    } catch (e) {
      console.error('Failed to persist feature flags:', e)
    }
  }

  /**
   * Simple hash function for consistent user bucketing
   */
  private hashUserId(userId: string): number {
    let hash = 0
    for (let i = 0; i < userId.length; i++) {
      const char = userId.charCodeAt(i)
      hash = ((hash << 5) - hash) + char
      hash = hash & hash // Convert to 32-bit integer
    }
    return Math.abs(hash)
  }
}

// Singleton instance
export const featureFlags = new FeatureFlagsService()

// React hook for feature flags
export function useFeatureFlag(flagName: string, userId?: string): boolean {
  // In a real app, this would use React state and context
  return featureFlags.isEnabled(flagName, userId)
}

// Development utilities
if (process.env.NODE_ENV === 'development') {
  // @ts-ignore
  window.featureFlags = featureFlags
  
  // @ts-ignore
  window.enablePhase1 = () => {
    featureFlags.getFlagsForPhase(1).forEach(flag => {
      featureFlags.enable(flag.name)
    })
    console.log('Phase 1 features enabled!')
  }
  
  // @ts-ignore
  window.enableAllFeatures = () => {
    [1, 2, 3, 4].forEach(phase => {
      featureFlags.getFlagsForPhase(phase as any).forEach(flag => {
        featureFlags.enable(flag.name)
      })
    })
    console.log('All features enabled!')
  }
}