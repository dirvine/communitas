import { describe, test, expect, beforeEach, afterEach } from 'vitest'
import { MarkdownWebPublisher } from '../markdownPublisher'
import { FourWordIdentity } from '../../../types/collaboration'
import { DHTStorage } from '../dhtStorage'
import { ReedSolomonEncoder } from '../reedSolomon'

describe('MarkdownWebPublisher', () => {
  let publisher: MarkdownWebPublisher
  let testIdentity: FourWordIdentity
  let dht: DHTStorage
  let encoder: ReedSolomonEncoder
  
  beforeEach(async () => {
    testIdentity = {
      fourWords: 'ocean-forest-moon-star',
      publicKey: 'pk_test_123',
      dhtAddress: 'dht://ocean-forest-moon-star'
    }
    
    dht = new DHTStorage({
      identity: testIdentity,
      bootstrapNodes: ['localhost:5001'],
      replicationFactor: 3
    })
    
    encoder = new ReedSolomonEncoder({
      dataShards: 10,
      parityShards: 6
    })
    
    publisher = new MarkdownWebPublisher({
      identity: testIdentity,
      dht,
      encoder,
      baseDirectory: '/web/'
    })
    
    await publisher.initialize()
  })

  afterEach(async () => {
    await publisher.destroy()
  })

  describe('Publishing flow', () => {
    test('should detect files in /web/ directory', async () => {
      // Given: Files in /web/ including home.md
      await publisher.addFile('/web/home.md', '# Welcome\n\nThis is the home page.')
      await publisher.addFile('/web/about.md', '# About\n\nAbout us page.')
      await publisher.addFile('/web/docs/getting-started.md', '# Getting Started')
      await publisher.addFile('/web/assets/logo.png', new Uint8Array(1024))
      
      // When: Scan for publishable content
      const files = await publisher.scanWebDirectory()
      
      // Then: All markdown files identified
      expect(files).toHaveLength(4)
      expect(files.find(f => f.path === '/web/home.md')).toBeDefined()
      expect(files.find(f => f.path === '/web/about.md')).toBeDefined()
      expect(files.find(f => f.path === '/web/docs/getting-started.md')).toBeDefined()
      expect(files.find(f => f.path === '/web/assets/logo.png')).toBeDefined()
    })

    test('should generate four-word identity for website', async () => {
      // Given: Request for new identity
      // When: Generate
      const identity = await publisher.generateWebsiteIdentity()
      
      // Then: Valid four-word string returned
      expect(identity.fourWords).toMatch(/^[a-z]+(-[a-z]+){3}$/)
      expect(identity.publicKey).toBeTruthy()
      expect(identity.dhtAddress).toStartWith('dht://')
    })

    test('should convert markdown to browsable website', async () => {
      // Given: home.md and other pages
      await publisher.addFile('/web/home.md', '# Welcome\n\n[About](about.md)')
      await publisher.addFile('/web/about.md', '# About\n\n[Back to home](home.md)')
      
      // When: Publish
      const result = await publisher.publish()
      
      // Then: Website accessible via identity
      expect(result.published).toBe(true)
      expect(result.identity).toBeDefined()
      expect(result.identity.fourWords).toBeTruthy()
      expect(result.manifest).toBeDefined()
      expect(result.manifest.entryPoint).toBe('home.md')
    })
  })

  describe('Cross-linking', () => {
    test('should resolve cross-entity links', async () => {
      // Given: Link to another entity's content
      const markdown = `
# My Page

Check out [Alice's Blog](alice-river-sun-cloud/blog/latest.md)
See [Project Docs](acme-mountain-wind-star/projects/alpha/docs.md)
      `
      
      await publisher.addFile('/web/home.md', markdown)
      
      // When: Parse markdown link
      const result = await publisher.publish()
      const processedContent = await publisher.getProcessedContent('/web/home.md')
      
      // Then: Resolves to correct DHT address
      expect(processedContent).toContain('dht://alice-river-sun-cloud/blog/latest.md')
      expect(processedContent).toContain('dht://acme-mountain-wind-star/projects/alpha/docs.md')
    })

    test('should handle relative and absolute paths', async () => {
      // Given: Various link formats
      const markdown = `
# Links Test

- [Relative link](docs/guide.md)
- [Absolute link](/about.md)
- [External entity](other-entity-words/page.md)
- [Regular URL](https://example.com)
      `
      
      await publisher.addFile('/web/test.md', markdown)
      await publisher.addFile('/web/docs/guide.md', '# Guide')
      await publisher.addFile('/web/about.md', '# About')
      
      // When: Process links
      const processed = await publisher.processLinks('/web/test.md', markdown)
      
      // Then: All resolve correctly
      expect(processed).toContain('"/web/docs/guide.md"')
      expect(processed).toContain('"/web/about.md"')
      expect(processed).toContain('dht://other-entity-words/page.md')
      expect(processed).toContain('https://example.com')
    })

    test('should generate table of contents from markdown headers', async () => {
      const markdown = `
# Main Title

## Section 1
Content for section 1

### Subsection 1.1
More content

## Section 2
Content for section 2
      `
      
      const toc = await publisher.generateTableOfContents(markdown)
      
      expect(toc).toHaveLength(4)
      expect(toc[0]).toEqual({ level: 1, text: 'Main Title', id: 'main-title' })
      expect(toc[1]).toEqual({ level: 2, text: 'Section 1', id: 'section-1' })
      expect(toc[2]).toEqual({ level: 3, text: 'Subsection 1.1', id: 'subsection-11' })
      expect(toc[3]).toEqual({ level: 2, text: 'Section 2', id: 'section-2' })
    })
  })

  describe('Manifest generation', () => {
    test('should create manifest with all website metadata', async () => {
      await publisher.addFile('/web/home.md', '# Home')
      await publisher.addFile('/web/about.md', '# About')
      await publisher.addFile('/web/assets/style.css', 'body { margin: 0; }')
      
      const manifest = await publisher.generateManifest()
      
      expect(manifest.version).toBe('1.0.0')
      expect(manifest.entryPoint).toBe('home.md')
      expect(manifest.files).toHaveLength(3)
      expect(manifest.identity).toEqual(testIdentity)
      expect(manifest.createdAt).toBeLessThanOrEqual(Date.now())
      expect(manifest.theme).toBe('auto')
    })

    test('should include file checksums in manifest', async () => {
      await publisher.addFile('/web/home.md', '# Home')
      
      const manifest = await publisher.generateManifest()
      const homeFile = manifest.files.find(f => f.path === '/web/home.md')
      
      expect(homeFile).toBeDefined()
      expect(homeFile!.checksum).toBeTruthy()
      expect(homeFile!.size).toBe(6)
      expect(homeFile!.mimeType).toBe('text/markdown')
    })
  })

  describe('Storage and distribution', () => {
    test('should distribute website files using Reed-Solomon', async () => {
      await publisher.addFile('/web/home.md', '# Large Content\n'.repeat(10000))
      
      const result = await publisher.publish()
      
      expect(result.shardDistribution).toBeDefined()
      expect(result.shardDistribution.totalShards).toBe(16)
      expect(result.shardDistribution.dataShards).toBe(10)
      expect(result.shardDistribution.parityShards).toBe(6)
    })

    test('should encrypt files before DHT storage', async () => {
      const content = '# Secret Content'
      await publisher.addFile('/web/secret.md', content)
      
      const result = await publisher.publish()
      
      // Verify encryption occurred
      const storedBlock = await dht.getRawBlock(result.manifest.files[0].blockId)
      expect(storedBlock.encryptedData).not.toContain(content)
    })

    test('should handle binary assets correctly', async () => {
      const imageData = new Uint8Array(1024 * 10) // 10KB image
      for (let i = 0; i < imageData.length; i++) {
        imageData[i] = i % 256
      }
      
      await publisher.addFile('/web/assets/image.png', imageData)
      
      const result = await publisher.publish()
      const imageFile = result.manifest.files.find(f => f.path === '/web/assets/image.png')
      
      expect(imageFile).toBeDefined()
      expect(imageFile!.mimeType).toBe('image/png')
      expect(imageFile!.size).toBe(10240)
    })
  })

  describe('Website rendering', () => {
    test('should convert markdown to HTML', async () => {
      const markdown = `
# Title

**Bold text** and *italic text*

- List item 1
- List item 2

[Link](https://example.com)

\`\`\`javascript
const code = "example";
\`\`\`
      `
      
      const html = await publisher.markdownToHtml(markdown)
      
      expect(html).toContain('<h1>Title</h1>')
      expect(html).toContain('<strong>Bold text</strong>')
      expect(html).toContain('<em>italic text</em>')
      expect(html).toContain('<ul>')
      expect(html).toContain('<li>List item 1</li>')
      expect(html).toContain('<a href="https://example.com">Link</a>')
      expect(html).toContain('<pre><code class="language-javascript">')
    })

    test('should apply custom CSS themes', async () => {
      await publisher.setTheme('dark')
      
      const html = await publisher.renderPage('/web/home.md', '# Title')
      
      expect(html).toContain('data-theme="dark"')
      expect(html).toContain('background-color: #1a1a1a')
    })

    test('should include navigation between pages', async () => {
      await publisher.addFile('/web/home.md', '# Home')
      await publisher.addFile('/web/page1.md', '# Page 1')
      await publisher.addFile('/web/page2.md', '# Page 2')
      
      const html = await publisher.renderWithNavigation('/web/home.md')
      
      expect(html).toContain('<nav')
      expect(html).toContain('href="/web/page1.md"')
      expect(html).toContain('href="/web/page2.md"')
    })
  })

  describe('Analytics and metrics', () => {
    test('should track page views', async () => {
      await publisher.addFile('/web/home.md', '# Home')
      await publisher.publish()
      
      // Simulate page views
      await publisher.recordPageView('/web/home.md', 'visitor-1')
      await publisher.recordPageView('/web/home.md', 'visitor-2')
      await publisher.recordPageView('/web/home.md', 'visitor-1') // Duplicate
      
      const analytics = await publisher.getAnalytics()
      
      expect(analytics.totalViews).toBe(3)
      expect(analytics.uniqueVisitors).toBe(2)
      expect(analytics.pageViews['/web/home.md']).toBe(3)
    })

    test('should track bandwidth usage', async () => {
      await publisher.addFile('/web/home.md', '# Home')
      await publisher.addFile('/web/assets/image.png', new Uint8Array(1024 * 100)) // 100KB
      await publisher.publish()
      
      // Simulate content delivery
      await publisher.serveContent('/web/home.md')
      await publisher.serveContent('/web/assets/image.png')
      
      const analytics = await publisher.getAnalytics()
      
      expect(analytics.bandwidth).toBeGreaterThan(100 * 1024)
    })
  })

  describe('Update and versioning', () => {
    test('should handle content updates', async () => {
      // Initial publish
      await publisher.addFile('/web/home.md', '# Version 1')
      const v1 = await publisher.publish()
      
      // Update content
      await publisher.updateFile('/web/home.md', '# Version 2')
      const v2 = await publisher.publish()
      
      expect(v2.manifest.version).not.toBe(v1.manifest.version)
      expect(v2.manifest.previousVersion).toBe(v1.manifest.version)
    })

    test('should maintain version history', async () => {
      await publisher.addFile('/web/home.md', '# Version 1')
      await publisher.publish()
      
      await publisher.updateFile('/web/home.md', '# Version 2')
      await publisher.publish()
      
      await publisher.updateFile('/web/home.md', '# Version 3')
      await publisher.publish()
      
      const history = await publisher.getVersionHistory()
      
      expect(history).toHaveLength(3)
      expect(history[0].content).toContain('Version 1')
      expect(history[1].content).toContain('Version 2')
      expect(history[2].content).toContain('Version 3')
    })

    test('should support rollback to previous versions', async () => {
      await publisher.addFile('/web/home.md', '# Version 1')
      const v1 = await publisher.publish()
      
      await publisher.updateFile('/web/home.md', '# Version 2')
      await publisher.publish()
      
      await publisher.rollbackToVersion(v1.manifest.version)
      
      const current = await publisher.getContent('/web/home.md')
      expect(current).toBe('# Version 1')
    })
  })
})