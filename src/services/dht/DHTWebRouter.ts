import { EventEmitter } from 'events'
import { DHTStorage } from '../storage/dhtStorage'
import { MarkdownWebPublisher } from '../storage/markdownPublisher'
import { NetworkIdentity } from '../../types/collaboration'

export interface RouteMatch {
  identity: NetworkIdentity
  path: string
  isHome: boolean
  publisher: MarkdownWebPublisher
}

export interface RouteConfig {
  defaultEntryPoint: string
  allowDirectoryListing: boolean
  enableCaching: boolean
  cacheTimeoutMs: number
}

export interface CachedContent {
  content: string
  contentType: string
  lastModified: number
  etag: string
  publisher: MarkdownWebPublisher
}

export interface ForwardIdentityRecord {
  fourWordAddress: string
  publicKey: string
  dhtAddress: string
  webManifestHash: string
  lastUpdated: number
  signature: string
}

/**
 * DHTWebRouter handles routing of web requests through the DHT network
 * Ensures home.md is served as the default entry point for forward locations
 */
export class DHTWebRouter extends EventEmitter {
  private dht: DHTStorage
  private contentCache = new Map<string, CachedContent>()
  private identityCache = new Map<string, ForwardIdentityRecord>()
  private publisherCache = new Map<string, MarkdownWebPublisher>()
  private config: RouteConfig

  constructor(
    dht: DHTStorage,
    config: RouteConfig = {
      defaultEntryPoint: 'home.md',
      allowDirectoryListing: false,
      enableCaching: true,
      cacheTimeoutMs: 300000 // 5 minutes
    }
  ) {
    super()
    this.dht = dht
    this.config = config
  }

  /**
   * Route a web request to the appropriate DHT content
   * @param url - The URL to route (e.g., "ocean-forest-moon-star" or "ocean-forest-moon-star/about.md")
   * @returns RouteMatch with content and metadata
   */
  async route(url: string): Promise<RouteMatch> {
    try {
      const { identity, path } = this.parseUrl(url)
      
      if (!identity) {
        throw new Error('Invalid URL format: must contain four-word identity')
      }

      // Resolve the forward identity
      const identityRecord = await this.resolveForwardIdentity(identity)
      if (!identityRecord) {
        throw new Error(`Identity ${identity} not found on DHT`)
      }

      // Get or create publisher for this identity
      const publisher = await this.getPublisher(identityRecord)
      
      // Determine the actual path to serve
      const actualPath = this.resolvePath(path)
      const isHome = actualPath === this.config.defaultEntryPoint
      
      return {
        identity: {
          fourWords: identityRecord.fourWordAddress,
          publicKey: identityRecord.publicKey,
          dhtAddress: identityRecord.dhtAddress
        },
        path: actualPath,
        isHome,
        publisher
      }
    } catch (error) {
      this.emit('routingError', { url, error })
      throw error
    }
  }

  /**
   * Serve content for a routed request
   * @param routeMatch - The route match from route()
   * @returns Content with headers
   */
  async serveContent(routeMatch: RouteMatch): Promise<{
    content: string
    contentType: string
    headers: Record<string, string>
    statusCode: number
  }> {
    try {
      const cacheKey = `${routeMatch.identity.fourWords}/${routeMatch.path}`
      
      // Check cache if enabled
      if (this.config.enableCaching) {
        const cached = this.contentCache.get(cacheKey)
        if (cached && (Date.now() - cached.lastModified) < this.config.cacheTimeoutMs) {
          return {
            content: cached.content,
            contentType: cached.contentType,
            headers: {
              'Cache-Control': `max-age=${Math.floor(this.config.cacheTimeoutMs / 1000)}`,
              'ETag': cached.etag,
              'Last-Modified': new Date(cached.lastModified).toUTCString(),
              'X-DHT-Source': routeMatch.identity.fourWords,
              'X-DHT-Path': routeMatch.path,
              'X-DHT-Home': routeMatch.isHome.toString()
            },
            statusCode: 200
          }
        }
      }

      // Load content from publisher
      let content: string
      let contentType: string

      try {
        // Try to load the specific file
        const filePath = `/web/${routeMatch.path}`
        content = await routeMatch.publisher.getProcessedContent(filePath)
        contentType = this.getContentType(routeMatch.path)
      } catch (error) {
        // If file not found and not already home, try home.md
        if (routeMatch.path !== this.config.defaultEntryPoint) {
          try {
            content = await routeMatch.publisher.getProcessedContent(`/web/${this.config.defaultEntryPoint}`)
            contentType = this.getContentType(this.config.defaultEntryPoint)
            
            // Update route match to reflect we're serving home
            routeMatch.path = this.config.defaultEntryPoint
            routeMatch.isHome = true
          } catch (homeError) {
            throw new Error(`Neither ${routeMatch.path} nor ${this.config.defaultEntryPoint} found`)
          }
        } else {
          throw error
        }
      }

      // Render if markdown
      if (contentType === 'text/markdown') {
        content = await routeMatch.publisher.renderPage(routeMatch.path, content)
        contentType = 'text/html'
      }

      // Cache the content
      if (this.config.enableCaching) {
        const etag = this.generateETag(content)
        const cachedContent: CachedContent = {
          content,
          contentType,
          lastModified: Date.now(),
          etag,
          publisher: routeMatch.publisher
        }
        this.contentCache.set(cacheKey, cachedContent)
      }

      // Generate analytics if available
      await this.recordPageView(routeMatch)

      return {
        content,
        contentType,
        headers: {
          'Content-Type': contentType,
          'Cache-Control': `max-age=${Math.floor(this.config.cacheTimeoutMs / 1000)}`,
          'ETag': this.generateETag(content),
          'X-DHT-Source': routeMatch.identity.fourWords,
          'X-DHT-Path': routeMatch.path,
          'X-DHT-Home': routeMatch.isHome.toString(),
          'X-DHT-Network': 'Communitas-P2P'
        },
        statusCode: 200
      }
    } catch (error) {
      this.emit('servingError', { routeMatch, error })
      
      return {
        content: this.generateErrorPage(error instanceof Error ? error.message : 'Unknown error'),
        contentType: 'text/html',
        headers: {
          'Content-Type': 'text/html',
          'X-DHT-Error': 'true'
        },
        statusCode: 404
      }
    }
  }

  /**
   * Register a forward identity on the DHT
   * @param identity - The network identity to register
   * @param webManifestHash - Hash of the web manifest
   */
  async registerForwardIdentity(
    identity: NetworkIdentity, 
    webManifestHash: string
  ): Promise<void> {
    const record: ForwardIdentityRecord = {
      fourWordAddress: identity.fourWords,
      publicKey: identity.publicKey,
      dhtAddress: identity.dhtAddress,
      webManifestHash,
      lastUpdated: Date.now(),
      signature: await this.signRecord(identity, webManifestHash)
    }

    // Store in DHT at well-known location
    const recordData = JSON.stringify(record)
    const recordKey = this.generateIdentityKey(identity.fourWords)
    await this.dht.putWithMetadata(Buffer.from(recordData, 'utf-8'), {
      size: recordData.length,
      createdAt: Date.now(),
      mimeType: 'application/json',
      forwardIdentity: identity
    })

    // Update local cache
    this.identityCache.set(identity.fourWords, record)
    
    this.emit('identityRegistered', { identity, record })
  }

  /**
   * Update the web manifest for a forward identity
   * @param identity - The identity to update
   * @param newManifestHash - New manifest hash
   */
  async updateWebManifest(
    identity: NetworkIdentity,
    newManifestHash: string
  ): Promise<void> {
    const existingRecord = this.identityCache.get(identity.fourWords)
    if (!existingRecord) {
      throw new Error('Identity not registered')
    }

    await this.registerForwardIdentity(identity, newManifestHash)
    
    // Invalidate content cache for this identity
    this.invalidateContentCache(identity.fourWords)
    
    this.emit('manifestUpdated', { identity, newManifestHash })
  }

  /**
   * List all registered forward identities
   */
  async listForwardIdentities(): Promise<ForwardIdentityRecord[]> {
    return Array.from(this.identityCache.values())
  }

  /**
   * Get directory listing for a path (if enabled)
   */
  async getDirectoryListing(routeMatch: RouteMatch, path: string): Promise<{
    directories: string[]
    files: Array<{ name: string; size: number; modified: number }>
  }> {
    if (!this.config.allowDirectoryListing) {
      throw new Error('Directory listing disabled')
    }

    // This would query the publisher's file structure
    // Simplified implementation
    return {
      directories: ['assets', 'posts'],
      files: [
        { name: 'home.md', size: 1024, modified: Date.now() },
        { name: 'about.md', size: 512, modified: Date.now() - 86400000 }
      ]
    }
  }

  // Private methods

  private parseUrl(url: string): { identity: string; path: string } {
    // Handle various URL formats:
    // "ocean-forest-moon-star" -> { identity: "ocean-forest-moon-star", path: "" }
    // "ocean-forest-moon-star/" -> { identity: "ocean-forest-moon-star", path: "" }
    // "ocean-forest-moon-star/about.md" -> { identity: "ocean-forest-moon-star", path: "about.md" }
    // "dht://ocean-forest-moon-star/path" -> { identity: "ocean-forest-moon-star", path: "path" }
    
    let cleanUrl = url
    
    // Remove protocol if present
    if (cleanUrl.startsWith('dht://')) {
      cleanUrl = cleanUrl.slice(6)
    }
    if (cleanUrl.startsWith('https://') || cleanUrl.startsWith('http://')) {
      cleanUrl = cleanUrl.split('/').slice(-1)[0] // Get last part
    }
    
    // Split identity and path
    const parts = cleanUrl.split('/')
    const identity = parts[0]
    const path = parts.slice(1).join('/')
    
    return { identity, path }
  }

  private resolvePath(path: string): string {
    // If no path specified, serve default entry point
    if (!path || path === '' || path === '/') {
      return this.config.defaultEntryPoint
    }
    
    // Ensure .md extension for markdown files
    if (!path.includes('.') && path !== this.config.defaultEntryPoint) {
      return `${path}.md`
    }
    
    return path
  }

  private async resolveForwardIdentity(identity: string): Promise<ForwardIdentityRecord | null> {
    // Check cache first
    if (this.identityCache.has(identity)) {
      const cached = this.identityCache.get(identity)!
      // Check if cache is still valid (within 1 hour)
      if (Date.now() - cached.lastUpdated < 3600000) {
        return cached
      }
    }

    try {
      // Query DHT for identity record
      const recordKey = this.generateIdentityKey(identity)
      const recordData = await this.dht.get(recordKey)
      const record: ForwardIdentityRecord = JSON.parse(Buffer.from(recordData).toString('utf-8'))
      
      // Verify signature
      if (await this.verifyRecord(record)) {
        this.identityCache.set(identity, record)
        return record
      } else {
        throw new Error('Invalid identity signature')
      }
    } catch (error) {
      console.error(`Failed to resolve identity ${identity}:`, error)
      return null
    }
  }

  private async getPublisher(identityRecord: ForwardIdentityRecord): Promise<MarkdownWebPublisher> {
    const identity = identityRecord.fourWordAddress
    
    if (this.publisherCache.has(identity)) {
      return this.publisherCache.get(identity)!
    }

    // Create new publisher instance
    // This is simplified - in production would have proper publisher factory
    const mockPublisher = new (class extends MarkdownWebPublisher {
      constructor() {
        super({
          identity: {
            fourWords: identityRecord.fourWordAddress,
            publicKey: identityRecord.publicKey,
            dhtAddress: identityRecord.dhtAddress
          },
          dht: null as any, // Would inject proper DHT instance
          encoder: null as any, // Would inject proper encoder
          baseDirectory: '/web/'
        })
      }
    })()

    await mockPublisher.initialize()
    this.publisherCache.set(identity, mockPublisher)
    
    return mockPublisher
  }

  private getContentType(path: string): string {
    const ext = path.split('.').pop()?.toLowerCase()
    const mimeTypes: Record<string, string> = {
      'md': 'text/markdown',
      'html': 'text/html',
      'css': 'text/css',
      'js': 'application/javascript',
      'json': 'application/json',
      'png': 'image/png',
      'jpg': 'image/jpeg',
      'jpeg': 'image/jpeg',
      'gif': 'image/gif',
      'svg': 'image/svg+xml',
      'pdf': 'application/pdf'
    }
    return mimeTypes[ext || ''] || 'application/octet-stream'
  }

  private generateETag(content: string): string {
    // Simple hash-based ETag
    const crypto = require('crypto')
    return `"${crypto.createHash('md5').update(content).digest('hex')}"`
  }

  private generateIdentityKey(identity: string): string {
    return `forward-identity:${identity}`
  }

  private async signRecord(identity: NetworkIdentity, manifestHash: string): Promise<string> {
    // Simplified signing - in production would use proper cryptography
    const crypto = require('crypto')
    const data = `${identity.fourWords}:${manifestHash}:${Date.now()}`
    return crypto.createHash('sha256').update(data).digest('hex')
  }

  private async verifyRecord(record: ForwardIdentityRecord): Promise<boolean> {
    // Simplified verification - in production would verify cryptographic signature
    return record.signature.length > 0
  }

  private generateErrorPage(message: string): string {
    return `
    <!DOCTYPE html>
    <html>
    <head>
      <title>Not Found - Communitas P2P</title>
      <style>
        body { 
          font-family: -apple-system, BlinkMacSystemFont, sans-serif; 
          text-align: center; 
          padding: 50px;
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }
        .container {
          background: rgba(255,255,255,0.1);
          border-radius: 10px;
          padding: 40px;
          backdrop-filter: blur(10px);
          max-width: 500px;
          margin: 0 auto;
        }
        h1 { font-size: 3em; margin-bottom: 0; }
        p { font-size: 1.2em; opacity: 0.9; }
        .home-link {
          display: inline-block;
          margin-top: 20px;
          padding: 10px 20px;
          background: rgba(255,255,255,0.2);
          border-radius: 5px;
          text-decoration: none;
          color: white;
          transition: background 0.3s;
        }
        .home-link:hover {
          background: rgba(255,255,255,0.3);
        }
      </style>
    </head>
    <body>
      <div class="container">
        <h1>404</h1>
        <p>Content not found on the DHT</p>
        <p style="font-size: 0.9em; opacity: 0.7;">${message}</p>
        <a href="/" class="home-link">Try Home Page</a>
      </div>
      <script>
        // Auto-redirect to home.md after 5 seconds if not already there
        if (!location.pathname.includes('home.md')) {
          setTimeout(() => {
            const identity = location.pathname.split('/')[1] || '';
            if (identity) {
              location.href = '/' + identity + '/home.md';
            }
          }, 5000);
        }
      </script>
    </body>
    </html>
    `
  }

  private async recordPageView(routeMatch: RouteMatch): Promise<void> {
    try {
      // Record page view analytics
      await routeMatch.publisher.recordPageView(
        routeMatch.path, 
        'anonymous' // Would get real visitor ID from request
      )
      
      this.emit('pageView', {
        identity: routeMatch.identity.fourWords,
        path: routeMatch.path,
        timestamp: Date.now()
      })
    } catch (error) {
      // Don't fail the request if analytics fail
      console.warn('Failed to record page view:', error)
    }
  }

  private invalidateContentCache(identity: string): void {
    const keysToDelete: string[] = []
    for (const key of this.contentCache.keys()) {
      if (key.startsWith(`${identity}/`)) {
        keysToDelete.push(key)
      }
    }
    keysToDelete.forEach(key => this.contentCache.delete(key))
  }

  // Public utility methods

  /**
   * Clear all caches
   */
  async clearCache(): Promise<void> {
    this.contentCache.clear()
    this.identityCache.clear()
    this.emit('cacheCleared')
  }

  /**
   * Get cache statistics
   */
  getCacheStats(): {
    contentCacheSize: number
    identityCacheSize: number
    publisherCacheSize: number
  } {
    return {
      contentCacheSize: this.contentCache.size,
      identityCacheSize: this.identityCache.size,
      publisherCacheSize: this.publisherCache.size
    }
  }

  /**
   * Preload content for an identity
   */
  async preloadIdentity(identity: string): Promise<void> {
    try {
      const identityRecord = await this.resolveForwardIdentity(identity)
      if (identityRecord) {
        await this.getPublisher(identityRecord)
        
        // Preload home.md
        const routeMatch = await this.route(`${identity}/${this.config.defaultEntryPoint}`)
        await this.serveContent(routeMatch)
        
        this.emit('identityPreloaded', { identity })
      }
    } catch (error) {
      console.error(`Failed to preload identity ${identity}:`, error)
    }
  }
}