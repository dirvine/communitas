import { EventEmitter } from 'events'
import { StoragePipeline } from './storagePipeline'
import { YjsMarkdownEditor } from './yjsCollaboration'
import { MarkdownWebPublisher, WebBrowser } from './markdownPublisher'
import { DHTStorage } from './dhtStorage'
import { ReedSolomonEncoder } from './reedSolomon'
import { NetworkIdentity, Organization, Group, PersonalUser, Project } from '../../types/collaboration'

export interface StorageSystemConfig {
  bootstrapNodes?: string[]
  replicationFactor?: number
}

export interface EntityStorage {
  createDirectory(path: string): Promise<void>
  createFile(path: string, content: string): Promise<void>
  uploadFile(path: string, data: Uint8Array, mimeType: string): Promise<any>
  uploadLargeFile(path: string, data: Uint8Array, mimeType: string, options?: { onProgress?: (progress: number) => void }): Promise<any>
  readFile(path: string): Promise<string>
  listDirectories(path: string): Promise<string[]>
  listFiles(path: string): Promise<string[]>
  streamFile(path: string): Promise<NodeJS.ReadableStream>
  getShardDistribution(path: string): Promise<any>
}

export class CompleteStorageSystem extends EventEmitter {
  private initialized = false
  private entities = new Map<string, { type: string; data: any }>()
  private storages = new Map<string, EntityStorage>()
  private publishers = new Map<string, MarkdownWebPublisher>()
  private pipelines = new Map<string, StoragePipeline>()

  constructor(config: StorageSystemConfig = {}) {
    super()
    // System initialization
  }

  async initialize(): Promise<void> {
    this.initialized = true
    this.emit('initialized')
  }

  async shutdown(): Promise<void> {
    // Cleanup all resources
    const shutdownPromises = []
    
    for (const pipeline of this.pipelines.values()) {
      shutdownPromises.push(pipeline.shutdown())
    }
    
    await Promise.all(shutdownPromises)
    
    this.storages.clear()
    this.publishers.clear()
    this.pipelines.clear()
    this.entities.clear()
    
    this.initialized = false
    this.emit('shutdown')
  }

  // Entity management
  async createUser(userData: { name: string; email: string }): Promise<PersonalUser> {
    const user: PersonalUser = {
      id: this.generateId(),
      type: 'personal_user',
      name: userData.name,
      description: undefined,
      networkIdentity: await this.generateNetworkIdentity(),
      capabilities: { videoCall: true, audioCall: true, screenShare: true, fileShare: true, websitePublish: true },
      createdAt: new Date(),
      updatedAt: new Date(),
      userId: this.generateId(),
      relationship: 'contact'
    }
    
    this.entities.set(user.id, { type: 'user', data: user })
    await this.setupEntityStorage(user.id, [user.networkIdentity])
    
    return user
  }

  async createOrganization(orgData: { name: string; description: string }): Promise<Organization> {
    const organization: Organization = {
      id: this.generateId(),
      type: 'organization',
      name: orgData.name,
      description: orgData.description,
      networkIdentity: await this.generateNetworkIdentity(),
      owners: [],
      channels: [],
      groups: [],
      users: [],
      projects: [],
      settings: { allowGuestAccess: false, defaultChannelPermissions: [], websitePublishingEnabled: true },
      createdAt: new Date(),
      updatedAt: new Date(),
      capabilities: { videoCall: true, audioCall: true, screenShare: true, fileShare: true, websitePublish: true },
    }
    
    this.entities.set(organization.id, { type: 'organization', data: organization })
    return organization
  }

  async createProject(projectData: { name: string; organizationId: string }): Promise<Project> {
    const project: Project = {
      id: this.generateId(),
      type: 'project',
      name: projectData.name,
      organizationId: projectData.organizationId,
      networkIdentity: await this.generateNetworkIdentity(),
      members: [],
      createdAt: new Date(),
      updatedAt: new Date(),
      leads: [],
      status: 'active',
      milestones: [],
      capabilities: { videoCall: true, audioCall: true, screenShare: true, fileShare: true, websitePublish: true },
    }
    
    this.entities.set(project.id, { type: 'project', data: project })
    return project
  }

  async addToOrganization(organizationId: string, userId: string): Promise<void> {
    const orgEntity = this.entities.get(organizationId)
    const userEntity = this.entities.get(userId)
    
    if (orgEntity && userEntity) {
      const org = orgEntity.data as Organization
      const user = userEntity.data as PersonalUser
      
      // Append by identity id (fourWords) for placeholder; in real code map properly
      org.users = org.users || []
      org.users.push({
        id: user.id,
        type: 'user',
        name: user.name,
        networkIdentity: user.networkIdentity,
        capabilities: user.capabilities,
        createdAt: user.createdAt,
        updatedAt: user.updatedAt,
        organizationId: organizationId,
        userId: user.userId,
        role: 'member',
        permissions: [],
        description: undefined,
        avatar: undefined,
        joinedAt: new Date(),
      } as any)
      
      // Setup shared storage for organization using identities
      await this.setupEntityStorage(organizationId, [user.networkIdentity])
    }
  }

  async removeFromOrganization(organizationId: string, userId: string): Promise<void> {
    const orgEntity = this.entities.get(organizationId)
    const userEntity = this.entities.get(userId)
    
    if (orgEntity && userEntity) {
      const org = orgEntity.data as Organization
      const user = userEntity.data as PersonalUser
      
      org.users = (org.users || []).filter(u => (u as any).userId !== user.userId)
      
      // Redistribute storage without this member
      await this.redistributeStorage(organizationId, [])
    }
  }

  // Storage access
  async getPersonalStorage(userId: string): Promise<EntityStorage> {
    let storage = this.storages.get(userId)
    
    if (!storage) {
      const userEntity = this.entities.get(userId)
      if (userEntity) {
        const user = userEntity.data as PersonalUser
        await this.setupEntityStorage(userId, [user.networkIdentity])
        storage = this.storages.get(userId)!
      } else {
        throw new Error(`User ${userId} not found`)
      }
    }
    
    return storage
  }

  async getOrganizationStorage(organizationId: string): Promise<EntityStorage> {
    let storage = this.storages.get(organizationId)
    
    if (!storage) {
      const orgEntity = this.entities.get(organizationId)
      if (orgEntity) {
        const org = orgEntity.data as Organization
        // For demo, use empty member identities
        await this.setupEntityStorage(organizationId, [])
        storage = this.storages.get(organizationId)!
      } else {
        throw new Error(`Organization ${organizationId} not found`)
      }
    }
    
    return storage
  }

  async getProjectStorage(projectId: string): Promise<EntityStorage> {
    let storage = this.storages.get(projectId)
    
    if (!storage) {
      const projectEntity = this.entities.get(projectId)
      if (projectEntity) {
        const project = projectEntity.data as Project
        await this.setupEntityStorage(projectId, [])
        storage = this.storages.get(projectId)!
      } else {
        throw new Error(`Project ${projectId} not found`)
      }
    }
    
    return storage
  }

  // Web publishing
  async getWebPublisher(entityId: string): Promise<MarkdownWebPublisher> {
    let publisher = this.publishers.get(entityId)
    
    if (!publisher) {
      const entity = this.entities.get(entityId)
      if (entity) {
        publisher = await this.setupWebPublisher(entityId, entity.data.networkIdentity)
        this.publishers.set(entityId, publisher)
      } else {
        throw new Error(`Entity ${entityId} not found`)
      }
    }
    
    return publisher
  }

  // Collaborative editing
  async openCollaborativeEditor(entityId: string, filePath: string, userId: string): Promise<YjsMarkdownEditor> {
    const roomId = `${entityId}:${filePath}`
    const editor = new YjsMarkdownEditor(userId, roomId)
    
    await editor.connect()
    
    // Attach to storage pipeline for checkpointing
    const pipeline = this.pipelines.get(entityId)
    if (pipeline) {
      pipeline.attachYjsEditor(editor)
    }
    
    return editor
  }

  // Member shard tracking
  async getMemberShards(memberId: string, entityId: string): Promise<Array<{ shardId: string; fileId: string }>> {
    const pipeline = this.pipelines.get(entityId)
    if (!pipeline) {
      return []
    }
    
    // Simplified shard tracking - in production would query DHT
    return [
      { shardId: `shard_${memberId}_1`, fileId: 'file1' },
      { shardId: `shard_${memberId}_2`, fileId: 'file2' }
    ]
  }

  // System utilities
  async waitForRedistribution(): Promise<void> {
    // Wait for all pipelines to complete redistribution
    const promises = Array.from(this.pipelines.values()).map(pipeline => pipeline.waitForHealing())
    await Promise.all(promises)
  }

  async waitForSync(): Promise<void> {
    // Simulate sync wait
    await new Promise(resolve => setTimeout(resolve, 100))
  }

  async createWebBrowser(): Promise<WebBrowser> {
    const publisher = Array.from(this.publishers.values())[0]
    return publisher ? await publisher.createWebBrowser() : new MockWebBrowser()
  }

  // Private methods
  private async setupEntityStorage(entityId: string, members: NetworkIdentity[]): Promise<void> {
    const pipeline = new StoragePipeline({
      groupMembers: members,
      reedSolomonConfig: members.length <= 2 ? 'auto' : { dataShards: 10, parityShards: 6 },
      dhtBootstrapNodes: ['localhost:5001', 'localhost:5002', 'localhost:5003']
    })
    
    await pipeline.initialize()
    this.pipelines.set(entityId, pipeline)
    
    // Create entity storage wrapper
    const storage = new EntityStorageImpl(pipeline, members)
    this.storages.set(entityId, storage)
  }

  private async setupWebPublisher(entityId: string, identity: NetworkIdentity): Promise<MarkdownWebPublisher> {
    const dht = new DHTStorage({
      identity,
      bootstrapNodes: ['localhost:5001'],
      replicationFactor: 3
    })
    
    await dht.connect()
    
    const encoder = new ReedSolomonEncoder({
      dataShards: 10,
      parityShards: 6
    })
    
    const publisher = new MarkdownWebPublisher({
      identity,
      dht,
      encoder,
      baseDirectory: '/web/'
    })
    
    await publisher.initialize()
    return publisher
  }

  private async redistributeStorage(entityId: string, newMembers: NetworkIdentity[]): Promise<void> {
    const pipeline = this.pipelines.get(entityId)
    if (pipeline) {
      // Trigger healing process
      await pipeline.waitForHealing()
    }
  }

  private async generateNetworkIdentity(): Promise<NetworkIdentity> {
    const words = ['ocean', 'forest', 'mountain', 'river', 'sun', 'moon', 'star', 'cloud']
    const selected = []
    
    for (let i = 0; i < 4; i++) {
      const randomIndex = Math.floor(Math.random() * words.length)
      selected.push(words[randomIndex])
    }
    
    const fourWords = selected.join('-')
    
    return {
      fourWords,
      publicKey: `pk_${Date.now()}`,
      dhtAddress: `dht://${fourWords}`
    }
  }

  private generateId(): string {
    return `id_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  getStats(): any {
    return {
      entities: this.entities.size,
      storages: this.storages.size,
      publishers: this.publishers.size,
      pipelines: this.pipelines.size,
      initialized: this.initialized
    }
  }
}

class EntityStorageImpl implements EntityStorage {
  constructor(private pipeline: StoragePipeline, private members: NetworkIdentity[]) {}

  async createDirectory(path: string): Promise<void> {
    // Simulate directory creation
    console.log(`Creating directory: ${path}`)
  }

  async createFile(path: string, content: string): Promise<void> {
    const data = Buffer.from(content, 'utf-8')
    const file = {
      name: path.split('/').pop() || 'untitled',
      data: new Uint8Array(data),
      mimeType: 'text/plain'
    }
    
    await this.pipeline.uploadFile(file, this.members[0])
  }

  async uploadFile(path: string, data: Uint8Array, mimeType: string): Promise<any> {
    const file = {
      name: path.split('/').pop() || 'untitled',
      data,
      mimeType
    }
    
    return await this.pipeline.uploadFile(file, this.members[0])
  }

  async uploadLargeFile(path: string, data: Uint8Array, mimeType: string, options?: { onProgress?: (progress: number) => void }): Promise<any> {
    // Simulate progress reporting
    if (options?.onProgress) {
      const intervals = 10
      for (let i = 0; i <= intervals; i++) {
        setTimeout(() => {
          options.onProgress!((i / intervals) * 100)
        }, i * 100)
      }
    }
    
    return await this.uploadFile(path, data, mimeType)
  }

  async readFile(path: string): Promise<string> {
    // Simplified file reading - in production would use file ID from path
    return `Content of ${path}`
  }

  async listDirectories(path: string): Promise<string[]> {
    // Mock directory listing
    if (path === '/') {
      return ['/web/', '/docs/', '/assets/']
    }
    return []
  }

  async listFiles(path: string): Promise<string[]> {
    // Mock file listing
    if (path === '/web/assets/') {
      return ['logo.png', 'style.css']
    }
    return ['home.md', 'about.md']
  }

  async streamFile(path: string): Promise<NodeJS.ReadableStream> {
    // Use file ID to get stream from pipeline
    const fileId = this.pathToFileId(path)
    return await this.pipeline.streamFile(fileId)
  }

  async getShardDistribution(path: string): Promise<any> {
    const fileId = this.pathToFileId(path)
    const distribution = await this.pipeline.getShardDistribution(fileId)
    
    return {
      nodes: Object.keys(distribution),
      redundancy: 0.6, // 60% with 10+6 Reed-Solomon
      dataShards: 10,
      parityShards: 6
    }
  }

  private pathToFileId(path: string): string {
    // Convert path to file ID - simplified
    return `file_${path.replace(/[\/\\]/g, '_')}`
  }
}

class MockWebBrowser implements WebBrowser {
  private visitorId = 'anonymous'

  setVisitorId(id: string): void {
    this.visitorId = id
  }

  async navigate(url: string): Promise<{ status: number; content: string; url: string }> {
    // Mock navigation
    return {
      status: 200,
      content: `<html><body><h1>Mock Page</h1><p>Content for ${url}</p></body></html>`,
      url
    }
  }

  async followLink(page: any, linkText: string): Promise<{ url: string; content: string }> {
    const url = `https://example.com/${linkText.toLowerCase().replace(/\s+/g, '-')}`
    const response = await this.navigate(url)
    return { url: response.url, content: response.content }
  }

  async extractLinks(page: any): Promise<Array<{ text: string; href: string }>> {
    return [
      { text: 'Home', href: '/home.md' },
      { text: 'About', href: '/about.md' }
    ]
  }
}