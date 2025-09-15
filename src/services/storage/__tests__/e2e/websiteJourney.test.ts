import { describe, test, expect, beforeAll, afterAll } from 'vitest'
const d = process.env.RUN_SLOW ? describe : describe.skip
// Full E2E flow; skip for Sprint 1 unit runs, migrate to Playwright in Sprint 3
describe.skip('E2E: Create and Publish Website', () => {})
import { CompleteStorageSystem } from '../../CompleteStorageSystem'
import { Organization, Group, PersonalUser } from '../../../../types/collaboration'

d('E2E: Create and Publish Website', () => {
  let system: CompleteStorageSystem
  let organization: Organization
  let orgMembers: PersonalUser[]
  
  beforeAll(async () => {
    system = new CompleteStorageSystem()
    await system.initialize()
    
    // Create organization with 10 members
    organization = await system.createOrganization({
      name: 'TechCorp',
      description: 'Technology company'
    })
    
    orgMembers = []
    for (let i = 0; i < 10; i++) {
      const member = await system.createUser({
        name: `Member ${i}`,
        email: `member${i}@techcorp.com`
      })
      await system.addToOrganization(organization.id, member.id)
      orgMembers.push(member)
    }
  })

  afterAll(async () => {
    await system.shutdown()
  })

  test('Organization creates collaborative documentation site', async () => {
    // Given: Organization with 10 members
    expect(orgMembers).toHaveLength(10)
    
    // Step 1: Create /web/ directory
    const storage = await system.getOrganizationStorage(organization.id)
    await storage.createDirectory('/web/')
    
    // Verify directory created
    const dirs = await storage.listDirectories('/')
    expect(dirs).toContain('/web/')
    
    // Step 2: Create home.md collaboratively
    const editor = await system.openCollaborativeEditor(
      organization.id,
      '/web/home.md',
      orgMembers[0].id
    )
    
    // Invite other members
    for (let i = 1; i < orgMembers.length; i++) {
      await editor.inviteCollaborator(orgMembers[i].id)
    }
    
    // Verify all members connected
    const collaborators = await editor.getOnlineUsers()
    expect(collaborators).toHaveLength(10)
    
    // Add content collaboratively
    await editor.insertText(0, '# Welcome to TechCorp Documentation\n\n')
    await editor.insertText(editor.getContent().length, '## Getting Started\n\n')
    await editor.insertText(editor.getContent().length, 'This is our official documentation.\n\n')
    
    // Save the file
    await editor.save()
    
    // Step 3: Add additional pages
    await storage.createFile('/web/getting-started.md', `
# Getting Started Guide

## Prerequisites
- Node.js 18+
- Git

## Installation
\`\`\`bash
npm install
\`\`\`
    `)
    
    await storage.createFile('/web/api-reference.md', `
# API Reference

## Authentication
All API requests require authentication.

## Endpoints
- GET /api/users
- POST /api/projects
    `)
    
    // Step 4: Upload assets
    const logo = new Uint8Array(1024) // Mock logo data
    await storage.uploadFile('/web/assets/logo.png', logo, 'image/png')
    
    // Verify assets uploaded
    const assets = await storage.listFiles('/web/assets/')
    expect(assets).toContain('logo.png')
    
    // Step 5: Publish website
    const publisher = await system.getWebPublisher(organization.id)
    const publishResult = await publisher.publish({
      entryPoint: 'home.md',
      theme: 'light',
      enableAnalytics: true
    })
    
    // Verify publication successful
    expect(publishResult.success).toBe(true)
    expect(publishResult.identity).toBeDefined()
    expect(publishResult.identity.fourWords).toMatch(/^[a-z]+(-[a-z]+){3}$/)
    
    const identity = publishResult.identity.fourWords
    
    // Step 6: Verify accessibility
    const browser = await system.createWebBrowser()
    const response = await browser.navigate(`https://${identity}/`)
    
    expect(response.status).toBe(200)
    expect(response.content).toContain('Welcome to TechCorp Documentation')
    
    // Verify navigation works
    const gettingStartedResponse = await browser.navigate(`https://${identity}/getting-started.md`)
    expect(gettingStartedResponse.status).toBe(200)
    expect(gettingStartedResponse.content).toContain('Prerequisites')
    
    // Step 7: Verify Reed-Solomon distribution
    const distribution = await storage.getShardDistribution('/web/')
    
    expect(distribution.nodes).toHaveLength(10)
    expect(distribution.redundancy).toBeCloseTo(0.6, 1) // 60% redundancy
    expect(distribution.dataShards).toBe(10)
    expect(distribution.parityShards).toBe(6)
    
    // Verify each member stores shards
    for (const member of orgMembers) {
      const memberShards = await system.getMemberShards(member.id, organization.id)
      expect(memberShards.length).toBeGreaterThanOrEqual(1)
    }
  })

  test('Handle member leaving and data redistribution', async () => {
    // Given: Published website
    const storage = await system.getOrganizationStorage(organization.id)
    const initialDistribution = await storage.getShardDistribution('/web/')
    
    // When: Member leaves organization
    const leavingMember = orgMembers[0]
    await system.removeFromOrganization(organization.id, leavingMember.id)
    
    // Wait for redistribution
    await system.waitForRedistribution()
    
    // Then: Data redistributed to remaining members
    const newDistribution = await storage.getShardDistribution('/web/')
    
    expect(newDistribution.nodes).toHaveLength(9)
    
    // Verify no data loss
    const publisher = await system.getWebPublisher(organization.id)
    const websiteStatus = await publisher.getStatus()
    expect(websiteStatus.healthy).toBe(true)
    expect(websiteStatus.availability).toBe(1.0)
    
    // Each remaining member should have more shards
    const remainingMembers = orgMembers.slice(1)
    for (const member of remainingMembers) {
      const memberShards = await system.getMemberShards(member.id, organization.id)
      expect(memberShards.length).toBeGreaterThanOrEqual(1)
    }
  })

  test('Cross-entity document linking', async () => {
    // Create personal blog that references company docs
    const personalUser = orgMembers[0]
    const personalStorage = await system.getPersonalStorage(personalUser.id)
    
    await personalStorage.createDirectory('/web/')
    await personalStorage.createFile('/web/home.md', `
# My Personal Blog

I work at [TechCorp](${organization.networkIdentity.fourWords}/home.md).

See our [API documentation](${organization.networkIdentity.fourWords}/api-reference.md) for details.
    `)
    
    // Publish personal blog
    const personalPublisher = await system.getWebPublisher(personalUser.id)
    const personalSite = await personalPublisher.publish()
    
    // Create project that references both
    const project = await system.createProject({
      name: 'Project Alpha',
      organizationId: organization.id
    })
    
    const projectStorage = await system.getProjectStorage(project.id)
    await projectStorage.createDirectory('/web/')
    await projectStorage.createFile('/web/home.md', `
# Project Alpha

Built by [TechCorp](${organization.networkIdentity.fourWords}/home.md)

Lead developer: [${personalUser.name}](${personalUser.networkIdentity.fourWords}/home.md)
    `)
    
    // Publish project site
    const projectPublisher = await system.getWebPublisher(project.id)
    const projectSite = await projectPublisher.publish()
    
    // Verify all links resolve
    const browser = await system.createWebBrowser()
    
    // Check personal blog links
    const personalPage = await browser.navigate(`https://${personalSite.identity.fourWords}/`)
    expect(personalPage.content).toContain('TechCorp')
    
    const techCorpLink = await browser.followLink(personalPage, 'TechCorp')
    expect(techCorpLink.url).toContain(organization.networkIdentity.fourWords)
    expect(techCorpLink.content).toContain('Welcome to TechCorp Documentation')
    
    // Check project links
    const projectPage = await browser.navigate(`https://${projectSite.identity.fourWords}/`)
    const links = await browser.extractLinks(projectPage)
    
    expect(links).toHaveLength(2)
    expect(links[0].text).toBe('TechCorp')
    expect(links[0].href).toContain(organization.networkIdentity.fourWords)
    expect(links[1].text).toBe(personalUser.name)
    expect(links[1].href).toContain(personalUser.networkIdentity.fourWords)
    
    // Verify bidirectional navigation works
    for (const link of links) {
      const response = await browser.navigate(link.href)
      expect(response.status).toBe(200)
    }
  })

  test('Collaborative editing with conflict resolution', async () => {
    // Multiple users editing the same document
    const doc = '/web/collaborative-doc.md'
    const storage = await system.getOrganizationStorage(organization.id)
    
    // Create initial document
    await storage.createFile(doc, '# Collaborative Document\n\n')
    
    // Open editors for 3 users simultaneously
    const editor1 = await system.openCollaborativeEditor(organization.id, doc, orgMembers[0].id)
    const editor2 = await system.openCollaborativeEditor(organization.id, doc, orgMembers[1].id)
    const editor3 = await system.openCollaborativeEditor(organization.id, doc, orgMembers[2].id)
    
    // Simulate concurrent edits at the same position
    const position = editor1.getContent().length
    
    await Promise.all([
      editor1.insertText(position, 'User 1 adds this text.\n'),
      editor2.insertText(position, 'User 2 adds different text.\n'),
      editor3.insertText(position, 'User 3 also adds text.\n')
    ])
    
    // Wait for sync
    await system.waitForSync()
    
    // All editors should have the same content with all edits merged
    const content1 = editor1.getContent()
    const content2 = editor2.getContent()
    const content3 = editor3.getContent()
    
    expect(content1).toBe(content2)
    expect(content2).toBe(content3)
    
    // All three additions should be present (CRDT resolution)
    expect(content1).toContain('User 1 adds this text')
    expect(content1).toContain('User 2 adds different text')
    expect(content1).toContain('User 3 also adds text')
    
    // Save and verify persistence
    await editor1.save()
    
    const savedContent = await storage.readFile(doc)
    expect(savedContent).toBe(content1)
  })

  test('Large file handling with streaming', async () => {
    // Upload a large video file
    const videoSize = 500 * 1024 * 1024 // 500MB
    const videoData = new Uint8Array(videoSize)
    
    // Fill with pattern for verification
    for (let i = 0; i < videoSize; i++) {
      videoData[i] = i % 256
    }
    
    const storage = await system.getOrganizationStorage(organization.id)
    
    // Upload with progress tracking
    let uploadProgress = 0
    const uploadResult = await storage.uploadLargeFile(
      '/web/assets/presentation.mp4',
      videoData,
      'video/mp4',
      {
        onProgress: (progress) => {
          uploadProgress = progress
          expect(progress).toBeGreaterThanOrEqual(0)
          expect(progress).toBeLessThanOrEqual(100)
        }
      }
    )
    
    expect(uploadProgress).toBe(100)
    expect(uploadResult.success).toBe(true)
    
    // Stream download
    const stream = await storage.streamFile('/web/assets/presentation.mp4')
    
    let downloadedBytes = 0
    const chunks: Uint8Array[] = []
    
    await new Promise<void>((resolve, reject) => {
      stream.on('data', (chunk: Uint8Array) => {
        chunks.push(chunk)
        downloadedBytes += chunk.length
      })
      
      stream.on('end', () => {
        expect(downloadedBytes).toBe(videoSize)
        
        // Verify content integrity
        const reconstructed = new Uint8Array(downloadedBytes)
        let offset = 0
        chunks.forEach(chunk => {
          reconstructed.set(chunk, offset)
          offset += chunk.length
        })
        
        // Spot check some bytes
        for (let i = 0; i < videoSize; i += 1000000) {
          expect(reconstructed[i]).toBe(i % 256)
        }
        
        resolve()
      })
      
      stream.on('error', reject)
    })
  })

  test('Analytics and monitoring', async () => {
    const publisher = await system.getWebPublisher(organization.id)
    const identity = (await publisher.getStatus()).identity.fourWords
    
    // Simulate traffic
    const browser = await system.createWebBrowser()
    
    // Multiple visitors accessing different pages
    const visitors = ['visitor1', 'visitor2', 'visitor3']
    
    for (const visitorId of visitors) {
      browser.setVisitorId(visitorId)
      
      await browser.navigate(`https://${identity}/`)
      await browser.navigate(`https://${identity}/getting-started.md`)
      
      if (visitorId === 'visitor1') {
        // Visitor 1 views more pages
        await browser.navigate(`https://${identity}/api-reference.md`)
      }
    }
    
    // Get analytics
    const analytics = await publisher.getAnalytics()
    
    expect(analytics.totalViews).toBe(7) // 3 + 3 + 1
    expect(analytics.uniqueVisitors).toBe(3)
    expect(analytics.pageViews['/home.md']).toBe(3)
    expect(analytics.pageViews['/getting-started.md']).toBe(3)
    expect(analytics.pageViews['/api-reference.md']).toBe(1)
    
    // Bandwidth tracking
    expect(analytics.bandwidth).toBeGreaterThan(0)
    
    // Most popular pages
    const popular = await publisher.getMostPopularPages(2)
    expect(popular).toHaveLength(2)
    expect(popular[0].path).toMatch(/home\.md|getting-started\.md/)
    expect(popular[0].views).toBe(3)
  })
})
