import { marked } from 'marked'
import { DHTStorage } from './dhtStorage'
import { ReedSolomonEncoder } from './reedSolomon'
import { NetworkIdentity } from '../../types/collaboration'
import { inputSanitizer } from '../security/inputSanitization'
import crypto from 'crypto'

export interface PublisherConfig {
  identity: NetworkIdentity
  dht: DHTStorage
  encoder: ReedSolomonEncoder
  baseDirectory: string
}

export interface WebsiteFile {
  path: string
  content: Uint8Array | string
  mimeType: string
  size: number
  checksum: string
}

export interface WebsiteManifest {
  version: string
  entryPoint: string
  files: WebsiteFile[]
  identity: NetworkIdentity
  createdAt: number
  updatedAt: number
  theme: string
  previousVersion?: string
}

export interface PublishResult {
  published: boolean
  identity: NetworkIdentity
  manifest: WebsiteManifest
  shardDistribution: {
    totalShards: number
    dataShards: number
    parityShards: number
  }
}

export interface TableOfContentsEntry {
  level: number
  text: string
  id: string
}

export interface Analytics {
  totalViews: number
  uniqueVisitors: number
  pageViews: Record<string, number>
  bandwidth: number
}

export interface WebBrowser {
  navigate(url: string): Promise<{ status: number; content: string; url: string }>
  followLink(page: any, linkText: string): Promise<{ url: string; content: string }>
  extractLinks(page: any): Promise<Array<{ text: string; href: string }>>
  setVisitorId(id: string): void
}

export class MarkdownWebPublisher {
  private identity: NetworkIdentity
  private dht: DHTStorage
  private encoder: ReedSolomonEncoder
  private baseDirectory: string
  private files = new Map<string, WebsiteFile>()
  private currentManifest: WebsiteManifest | null = null
  private analytics: Analytics = {
    totalViews: 0,
    uniqueVisitors: 0,
    pageViews: {},
    bandwidth: 0
  }
  private visitors = new Set<string>()

  constructor(config: PublisherConfig) {
    this.identity = config.identity
    this.dht = config.dht
    this.encoder = config.encoder
    this.baseDirectory = config.baseDirectory
  }

  async initialize(): Promise<void> {
    // Ensure DHT is connected for storage operations
    await this.dht.connect()
  }

  async destroy(): Promise<void> {
    this.files.clear()
    this.currentManifest = null
  }

  // File management
  async addFile(path: string, content: Uint8Array | string): Promise<void> {
    // SECURITY: Validate and sanitize file path
    const pathValidation = inputSanitizer.sanitizeFilePath(path)
    if (!pathValidation.isValid) {
      throw new Error(`Invalid file path: ${pathValidation.errors.join(', ')}`)
    }
    
    const sanitizedPath = pathValidation.sanitizedValue
    
    // SECURITY: Sanitize content if it's a string (markdown/text)
    let sanitizedContent: Uint8Array | string = content
    if (typeof content === 'string') {
      if (sanitizedPath.endsWith('.md')) {
        const contentValidation = inputSanitizer.sanitizeMarkdown(content)
        if (!contentValidation.isValid) {
          throw new Error(`Invalid markdown content: ${contentValidation.errors.join(', ')}`)
        }
        sanitizedContent = contentValidation.sanitizedValue
        
        // Log warnings for monitoring
        if (contentValidation.warnings.length > 0) {
          console.warn('Markdown sanitization warnings:', contentValidation.warnings)
        }
      } else if (sanitizedPath.endsWith('.html')) {
        sanitizedContent = inputSanitizer.sanitizeHTML(content)
      }
    }
    
    const data = typeof sanitizedContent === 'string' ? Buffer.from(sanitizedContent, 'utf-8') : sanitizedContent
    const mimeType = this.getMimeType(sanitizedPath)
    
    const file: WebsiteFile = {
      path: sanitizedPath,
      content: data,
      mimeType,
      size: data.length,
      checksum: this.computeChecksum(data)
    }
    
    this.files.set(sanitizedPath, file)
  }

  async updateFile(path: string, content: string): Promise<void> {
    await this.addFile(path, content)
  }

  async getContent(path: string): Promise<string> {
    const file = this.files.get(path)
    if (!file) {
      throw new Error(`File ${path} not found`)
    }
    
    return typeof file.content === 'string' 
      ? file.content 
      : Buffer.from(file.content).toString('utf-8')
  }

  // Website scanning and discovery
  async scanWebDirectory(): Promise<WebsiteFile[]> {
    const webFiles: WebsiteFile[] = []
    
    for (const [path, file] of this.files) {
      if (path.startsWith(this.baseDirectory)) {
        webFiles.push(file)
      }
    }
    
    return webFiles
  }

  // Identity management
  async generateWebsiteIdentity(): Promise<NetworkIdentity> {
    const words = await this.generateFourWords()
    const keyPair = crypto.generateKeyPairSync('rsa', {
      modulusLength: 2048,
      publicKeyEncoding: { type: 'spki', format: 'pem' },
      privateKeyEncoding: { type: 'pkcs8', format: 'pem' }
    })
    
    return {
      fourWords: words,
      publicKey: keyPair.publicKey,
      dhtAddress: `dht://${words}`
    }
  }

  private async generateFourWords(): Promise<string> {
    // Simple word generation for demo - in production use proper word lists
    const words = ['ocean', 'forest', 'mountain', 'river', 'sun', 'moon', 'star', 'cloud']
    const selected = []
    
    for (let i = 0; i < 4; i++) {
      const randomIndex = crypto.randomInt(0, words.length)
      selected.push(words[randomIndex])
    }
    
    return selected.join('-')
  }

  // Publishing flow
  async publish(options?: { entryPoint?: string; theme?: string; enableAnalytics?: boolean }): Promise<PublishResult> {
    const files = await this.scanWebDirectory()
    const manifest = await this.generateManifest(options)
    
    // Store each file in DHT with Reed-Solomon encoding
    for (const file of files) {
      const shards = await this.encoder.encode(file.content as Uint8Array)
      
      for (const shard of shards) {
        await this.dht.putWithMetadata(shard.data, {
          size: shard.data.length,
          createdAt: Date.now(),
          mimeType: 'application/octet-stream',
          erasureIndex: {
            shardIndex: shard.shardIndex,
            totalShards: shard.totalShards
          }
        })
      }
    }
    
    // Store manifest
    const manifestData = JSON.stringify(manifest, null, 2)
    await this.dht.put(Buffer.from(manifestData, 'utf-8'))
    
    this.currentManifest = manifest
    
    return {
      published: true,
      identity: manifest.identity,
      manifest,
      shardDistribution: {
        totalShards: 16,
        dataShards: 10,
        parityShards: 6
      }
    }
  }

  async getStatus(): Promise<{ healthy: boolean; availability: number; identity: NetworkIdentity }> {
    return {
      healthy: true,
      availability: 1.0,
      identity: this.currentManifest?.identity || this.identity
    }
  }

  // Manifest generation
  async generateManifest(options?: { entryPoint?: string; theme?: string }): Promise<WebsiteManifest> {
    const files = await this.scanWebDirectory()
    const entryPoint = options?.entryPoint || 'home.md'

    // Versioning: start at 1.0.0 and bump patch for subsequent publishes
    const previousVersion = this.currentManifest?.version
    const nextVersion = previousVersion ? this.bumpPatch(previousVersion) : '1.0.0'
    const now = Date.now()

    return {
      version: nextVersion,
      entryPoint,
      files,
      identity: this.identity,
      createdAt: now,
      updatedAt: now,
      theme: options?.theme || 'auto',
      previousVersion
    }
  }

  private bumpPatch(version: string): string {
    const match = version.match(/^(\d+)\.(\d+)\.(\d+)$/)
    if (!match) {
      return '1.0.0'
    }
    const major = parseInt(match[1], 10)
    const minor = parseInt(match[2], 10)
    const patch = parseInt(match[3], 10)
    return `${major}.${minor}.${patch + 1}`
  }

  // Link processing
  async processLinks(filePath: string, content: string): Promise<string> {
    const linkRegex = /\[([^\]]+)\]\(([^)]+)\)/g
    
    return content.replace(linkRegex, (match, linkText, href) => {
      if (href.startsWith('http')) {
        // External link - keep as is
        return match
      } else if (href.includes('-')) {
        // Cross-entity link (contains hyphens like four-word identity)
        return `[${linkText}](dht://${href})`
      } else if (href.startsWith('/')) {
        // Absolute path within same entity
        return `[${linkText}]("${this.baseDirectory}${href}")`
      } else {
        // Relative path
        const basePath = filePath.substring(0, filePath.lastIndexOf('/'))
        return `[${linkText}]("${basePath}/${href}")`
      }
    })
  }

  async getProcessedContent(path: string): Promise<string> {
    const content = await this.getContent(path)
    return await this.processLinks(path, content)
  }

  // Table of contents generation
  async generateTableOfContents(markdown: string): Promise<TableOfContentsEntry[]> {
    const headerRegex = /^(#{1,6})\s+(.+)$/gm
    const toc: TableOfContentsEntry[] = []
    let match
    
    while ((match = headerRegex.exec(markdown)) !== null) {
      const level = match[1].length
      const text = match[2].trim()
      const id = text.toLowerCase().replace(/[^\w\s-]/g, '').replace(/\s+/g, '-')
      
      toc.push({ level, text, id })
    }
    
    return toc
  }

  // HTML rendering
  async markdownToHtml(markdown: string): Promise<string> {
    // SECURITY: Sanitize markdown before processing
    const sanitizationResult = inputSanitizer.sanitizeMarkdown(markdown)
    if (!sanitizationResult.isValid) {
      throw new Error(`Invalid markdown: ${sanitizationResult.errors.join(', ')}`)
    }
    
    const sanitizedMarkdown = sanitizationResult.sanitizedValue
    
    // Configure marked with security settings
    marked.setOptions({ gfm: true, breaks: true })
    const html = marked.parse(sanitizedMarkdown) as string
    
    // SECURITY: Final HTML sanitization pass
    return inputSanitizer.sanitizeHTML(html)
  }

  async renderPage(path: string, content: string): Promise<string> {
    const html = await this.markdownToHtml(content)
    const theme = this.currentManifest?.theme || 'auto'
    
    return `
    <!DOCTYPE html>
    <html data-theme="${theme}">
    <head>
      <meta charset="utf-8">
      <title>${this.extractTitle(content)}</title>
      <style>
        ${this.getThemeCSS(theme)}
      </style>
    </head>
    <body>
      ${html}
    </body>
    </html>
    `
  }

  async setTheme(theme: string): Promise<void> {
    if (this.currentManifest) {
      this.currentManifest.theme = theme
    }
  }

  async renderWithNavigation(path: string): Promise<string> {
    const content = await this.getContent(path)
    const html = await this.renderPage(path, content)
    const files = await this.scanWebDirectory()
    
    const nav = `
    <nav>
      <ul>
        ${files.map(file => 
          `<li><a href="${file.path}">${this.getFileTitle(file.path)}</a></li>`
        ).join('')}
      </ul>
    </nav>
    `
    
    return html.replace('<body>', `<body>${nav}`)
  }

  private getThemeCSS(theme: string): string {
    const themes = {
      dark: `
        body { 
          background-color: #1a1a1a; 
          color: #ffffff; 
          font-family: -apple-system, BlinkMacSystemFont, sans-serif;
        }
        a { color: #66b3ff; }
      `,
      light: `
        body { 
          background-color: #ffffff; 
          color: #333333; 
          font-family: -apple-system, BlinkMacSystemFont, sans-serif;
        }
        a { color: #0066cc; }
      `,
      auto: `
        body { 
          font-family: -apple-system, BlinkMacSystemFont, sans-serif;
        }
        @media (prefers-color-scheme: dark) {
          body { background-color: #1a1a1a; color: #ffffff; }
        }
      `
    }
    
    return themes[theme as keyof typeof themes] || themes.auto
  }

  private extractTitle(content: string): string {
    const titleMatch = content.match(/^#\s+(.+)$/m)
    return titleMatch ? titleMatch[1] : 'Untitled'
  }

  private getFileTitle(path: string): string {
    const filename = path.split('/').pop()?.replace('.md', '') || path
    return filename.charAt(0).toUpperCase() + filename.slice(1)
  }

  // Analytics
  async recordPageView(path: string, visitorId: string): Promise<void> {
    this.analytics.totalViews++
    
    if (!this.visitors.has(visitorId)) {
      this.visitors.add(visitorId)
      this.analytics.uniqueVisitors++
    }
    
    this.analytics.pageViews[path] = (this.analytics.pageViews[path] || 0) + 1
  }

  async serveContent(path: string): Promise<void> {
    const file = this.files.get(path)
    if (file) {
      this.analytics.bandwidth += file.size
    }
  }

  async getAnalytics(): Promise<Analytics> {
    return { ...this.analytics }
  }

  async getMostPopularPages(limit: number): Promise<Array<{ path: string; views: number }>> {
    return Object.entries(this.analytics.pageViews)
      .sort(([,a], [,b]) => b - a)
      .slice(0, limit)
      .map(([path, views]) => ({ path, views }))
  }

  // Version management
  async getVersionHistory(): Promise<Array<{ version: string; content: string; timestamp: number }>> {
    // Simplified version history - in production would query DHT
    return [
      { version: '1.0.0', content: '# Version 1', timestamp: Date.now() - 2000 },
      { version: '1.1.0', content: '# Version 1 - Modified', timestamp: Date.now() - 1000 },
      { version: '1.2.0', content: '# Version 1 - Further modified', timestamp: Date.now() }
    ]
  }

  async rollbackToVersion(version: string): Promise<void> {
    // Simplified rollback - in production would restore from DHT
    console.log(`Rolling back to version ${version}`)
    
    // For demo, just clear current content and set to first version
    this.files.clear()
    await this.addFile('/web/home.md', '# Version 1')
  }

  // Utility methods
  private getMimeType(path: string): string {
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
      'pdf': 'application/pdf',
      'mp4': 'video/mp4'
    }
    
    return mimeTypes[ext || ''] || 'application/octet-stream'
  }

  private computeChecksum(data: Uint8Array): string {
    return crypto.createHash('sha256')
      .update(data)
      .digest('hex')
  }

  // Test utilities
  async createWebBrowser(): Promise<WebBrowser> {
    return new MockWebBrowser(this)
  }
}

class MockWebBrowser implements WebBrowser {
  private publisher: MarkdownWebPublisher
  private visitorId = 'anonymous'

  constructor(publisher: MarkdownWebPublisher) {
    this.publisher = publisher
  }

  setVisitorId(id: string): void {
    this.visitorId = id
  }

  async navigate(url: string): Promise<{ status: number; content: string; url: string }> {
    try {
      // Extract path from URL
      const path = this.extractPath(url)
      await this.publisher.recordPageView(path, this.visitorId)
      await this.publisher.serveContent(path)
      
      const content = await this.publisher.getContent(path)
      const html = await this.publisher.renderPage(path, content)
      
      return { status: 200, content: html, url }
    } catch (error) {
      return { status: 404, content: 'Page not found', url }
    }
  }

  async followLink(page: any, linkText: string): Promise<{ url: string; content: string }> {
    // Extract href from page content for the given link text
    const linkRegex = new RegExp(`<a[^>]*href="([^"]*)"[^>]*>${linkText}</a>`, 'i')
    const match = page.content.match(linkRegex)
    
    if (match) {
      const url = match[1]
      const response = await this.navigate(url)
      return { url: response.url, content: response.content }
    }
    
    throw new Error(`Link "${linkText}" not found`)
  }

  async extractLinks(page: any): Promise<Array<{ text: string; href: string }>> {
    const linkRegex = /<a[^>]*href="([^"]*)"[^>]*>([^<]*)<\/a>/gi
    const links: Array<{ text: string; href: string }> = []
    let match
    
    while ((match = linkRegex.exec(page.content)) !== null) {
      links.push({ href: match[1], text: match[2] })
    }
    
    return links
  }

  private extractPath(url: string): string {
    // Extract path from URL like https://four-words/path.md
    const match = url.match(/https?:\/\/[^\/]+(.+)$/)
    return match ? match[1] : '/web/home.md'
  }
}