import { 
  Organization, 
  Group, 
  Project,
  CreateOrganizationRequest,
  CreateGroupRequest,
  CreateProjectRequest,
  InviteMemberRequest,
  Member,
  Role,
  PermissionCheck,
  OrganizationHierarchy
} from '../../types/organization'
import { safeInvoke } from '../../utils/tauri'

class OrganizationService {
  // Mock data storage - in production this would connect to P2P DHT
  private organizations: Map<string, Organization> = new Map()
  private groups: Map<string, Group> = new Map()  
  private projects: Map<string, Project> = new Map()
  
  constructor() {
    this.initializeMockData()
  }

  // Organization Management
  async createOrganization(request: CreateOrganizationRequest, ownerId: string): Promise<Organization> {
    // Try to call the backend first
    const org = await safeInvoke<Organization>('create_organization_dht', { request, ownerId })
    
    if (org) {
      // Cache the organization locally
      this.organizations.set(org.id, org)
      return org
    }
    
    // Fallback to local mock implementation
    console.warn('Backend organization creation not available, using local mock')
    
    const orgMock: Organization = {
        id: this.generateId(),
        name: request.name,
        description: request.description,
        created_at: new Date(),
        updated_at: new Date(),
        owner_id: ownerId,
        members: [{
          user_id: ownerId,
          display_name: 'Owner User',
          four_word_address: 'owner.test.user.here',
          role: 'Owner',
          joined_at: new Date()
        }],
        has_file_system: true,
        storage_quota: {
          allocated_gb: request.initial_storage_gb,
          used_gb: 0,
          available_gb: request.initial_storage_gb,
          last_updated: new Date()
        },
        settings: {
          visibility: request.visibility,
          default_member_role: 'Member',
          allow_member_invitations: true,
          require_approval_for_joins: request.visibility !== 'public'
        },
        groups: [],
        projects: []
      }
      
      this.organizations.set(orgMock.id, orgMock)
      return orgMock
  }

  async getOrganization(id: string): Promise<Organization | null> {
    return this.organizations.get(id) || null
  }

  async getUserOrganizations(userId: string): Promise<Organization[]> {
    return Array.from(this.organizations.values())
      .filter(org => org.members.some(member => member.user_id === userId))
  }

  async updateOrganization(id: string, updates: Partial<Organization>): Promise<Organization | null> {
    const org = this.organizations.get(id)
    if (!org) return null
    
    const updatedOrg = { ...org, ...updates, updated_at: new Date() }
    this.organizations.set(id, updatedOrg)
    return updatedOrg
  }

  // Group Management
  async createGroup(request: CreateGroupRequest, creatorId: string): Promise<Group> {
    // Try to call the backend first
    const group = await safeInvoke<Group>('create_group_dht', { request, creatorId })
    
    if (group) {
      // Cache the group locally
      this.groups.set(group.id, group)
      
      // Update local organization cache
      const org = this.organizations.get(request.organization_id)
      if (org) {
        org.groups.push(group)
        this.organizations.set(org.id, org)
      }
      
      return group
    }
    
    console.warn('Backend group creation not available, using local mock')
      
      // Fallback to local mock implementation
      const org = this.organizations.get(request.organization_id)
    if (!org) throw new Error('Organization not found')
    
    const groupMock: Group = {
        id: this.generateId(),
        name: request.name,
        description: request.description,
        organization_id: request.organization_id,
        created_at: new Date(),
        updated_at: new Date(),
        creator_id: creatorId,
        members: [
          // Add creator
          { 
            user_id: creatorId,
            display_name: 'Creator User',
            four_word_address: 'creator.test.user.here',
            role: 'Admin',
            joined_at: new Date()
          }
        ],
        has_file_system: false,
        chat_settings: {
          message_retention_days: 30,
          allow_file_sharing: true,
          allow_voice_messages: true,
          allow_video_calls: true
        },
        inherited_permissions: true
      }

      // Add initial members if specified
      if (request.initial_members) {
        for (const userId of request.initial_members) {
          if (userId !== creatorId) {
            const shortId = userId.substring(0, 8)
            const shortIdForAddress = userId.substring(0, 4)
            group.members.push({
              user_id: userId,
              display_name: `User ${shortId}`,
              four_word_address: `user.${shortIdForAddress}.test.here`,
              role: 'Member',
              joined_at: new Date()
            })
          }
        }
      }
      
      this.groups.set(groupMock.id, groupMock)
      
      // Update organization to include this group
      org.groups.push(groupMock)
      this.organizations.set(org.id, org)
      
      return groupMock
  }

  async getGroup(id: string): Promise<Group | null> {
    return this.groups.get(id) || null
  }

  async getOrganizationGroups(organizationId: string): Promise<Group[]> {
    return Array.from(this.groups.values())
      .filter(group => group.organization_id === organizationId)
  }

  // Project Management
  async createProject(request: CreateProjectRequest, ownerId: string): Promise<Project> {
    // Try to call the backend first
    const project = await safeInvoke<Project>('create_project_dht', { request, ownerId })
    
    if (project) {
      // Cache the project locally
      this.projects.set(project.id, project)
      
      // Update local organization cache
      const org = this.organizations.get(request.organization_id)
      if (org) {
        org.projects.push(project)
        this.organizations.set(org.id, org)
      }
      
      return project
    }
    
    // Fallback to local mock implementation
    console.warn('Backend project creation not available, using local mock')
    
    const org = this.organizations.get(request.organization_id)
      if (!org) throw new Error('Organization not found')
      
      const projectMock: Project = {
        id: this.generateId(),
        name: request.name,
        description: request.description,
        organization_id: request.organization_id,
        parent_group_id: request.parent_group_id,
        created_at: new Date(),
        updated_at: new Date(),
        owner_id: ownerId,
        members: [{
          user_id: ownerId,
          display_name: 'Project Owner',
          four_word_address: 'owner.project.test.here',
          role: 'Owner',
          joined_at: new Date()
        }],
        has_file_system: true,
        storage_quota: {
          allocated_gb: request.initial_storage_gb,
          used_gb: 0,
          available_gb: request.initial_storage_gb,
          last_updated: new Date()
        },
        status: 'active',
        deadline: request.deadline,
        priority: request.priority,
        settings: {
          allow_external_collaborators: false,
          require_approval_for_changes: false,
          version_control_enabled: true,
          backup_enabled: true
        }
      }

      // Add initial members if specified
      if (request.initial_members) {
        for (const userId of request.initial_members) {
          if (userId !== ownerId) {
            const shortId = userId.substring(0, 8)
            const shortIdForAddress = userId.substring(0, 4)
            projectMock.members.push({
              user_id: userId,
              display_name: `User ${shortId}`,
              four_word_address: `user.${shortIdForAddress}.project.here`,
              role: 'Member',
              joined_at: new Date()
            })
          }
        }
      }
      
      this.projects.set(projectMock.id, projectMock)
      
      // Update organization to include this project
      org.projects.push(projectMock)
      this.organizations.set(org.id, org)
      
      return projectMock
  }

  async getProject(id: string): Promise<Project | null> {
    return this.projects.get(id) || null
  }

  async getOrganizationProjects(organizationId: string): Promise<Project[]> {
    return Array.from(this.projects.values())
      .filter(project => project.organization_id === organizationId)
  }

  // Member Management
  async inviteMember(request: InviteMemberRequest, _inviterId: string): Promise<boolean> {
    // In a real implementation, this would send an invitation
    // For now, we'll just add the member directly
    
    switch (request.entity_type) {
      case 'organization':
        return this.addMemberToOrganization(request.entity_id, request.invitee_address, request.role)
      case 'group':
        return this.addMemberToGroup(request.entity_id, request.invitee_address, request.role)
      case 'project':
        return this.addMemberToProject(request.entity_id, request.invitee_address, request.role)
      default:
        return false
    }
  }

  private async addMemberToOrganization(orgId: string, userAddress: string, role: Role): Promise<boolean> {
    const org = this.organizations.get(orgId)
    if (!org) return false
    
    // Check if user is already a member
    if (org.members.some(m => m.four_word_address === userAddress)) return false
    
    const newMember: Member = {
      user_id: this.generateId(),
      display_name: `User ${userAddress.split('.')[0]}`,
      four_word_address: userAddress,
      role,
      joined_at: new Date()
    }
    
    org.members.push(newMember)
    org.updated_at = new Date()
    this.organizations.set(orgId, org)
    return true
  }

  private async addMemberToGroup(groupId: string, userAddress: string, role: Role): Promise<boolean> {
    const group = this.groups.get(groupId)
    if (!group) return false
    
    // Check if user is already a member
    if (group.members.some(m => m.four_word_address === userAddress)) return false
    
    const newMember: Member = {
      user_id: this.generateId(),
      display_name: `User ${userAddress.split('.')[0]}`,
      four_word_address: userAddress,
      role,
      joined_at: new Date()
    }
    
    group.members.push(newMember)
    group.updated_at = new Date()
    this.groups.set(groupId, group)
    return true
  }

  private async addMemberToProject(projectId: string, userAddress: string, role: Role): Promise<boolean> {
    const project = this.projects.get(projectId)
    if (!project) return false
    
    // Check if user is already a member
    if (project.members.some(m => m.four_word_address === userAddress)) return false
    
    const newMember: Member = {
      user_id: this.generateId(),
      display_name: `User ${userAddress.split('.')[0]}`,
      four_word_address: userAddress,
      role,
      joined_at: new Date()
    }
    
    project.members.push(newMember)
    project.updated_at = new Date()
    this.projects.set(projectId, project)
    return true
  }

  // Permission System
  async checkPermission(check: Omit<PermissionCheck, 'granted' | 'inherited_from'>): Promise<PermissionCheck> {
    // This is a simplified permission check - in production would be more sophisticated
    const result: PermissionCheck = {
      ...check,
      granted: false
    }
    
    switch (check.entity_type) {
      case 'organization':
        const org = this.organizations.get(check.entity_id)
        if (org) {
          const member = org.members.find(m => m.user_id === check.user_id)
          if (member) {
            result.granted = this.hasPermission(member.role, check.permission)
          }
        }
        break
        
      case 'group':
        const group = this.groups.get(check.entity_id)
        if (group) {
          const member = group.members.find(m => m.user_id === check.user_id)
          if (member) {
            result.granted = this.hasPermission(member.role, check.permission)
          } else if (group.inherited_permissions) {
            // Check organization permissions
            const orgCheck = await this.checkPermission({
              entity_type: 'organization',
              entity_id: group.organization_id,
              user_id: check.user_id,
              permission: check.permission
            })
            result.granted = orgCheck.granted
            if (orgCheck.granted) {
              result.inherited_from = {
                entity_type: 'organization',
                entity_id: group.organization_id
              }
            }
          }
        }
        break
        
      case 'project':
        const project = this.projects.get(check.entity_id)
        if (project) {
          const member = project.members.find(m => m.user_id === check.user_id)
          if (member) {
            result.granted = this.hasPermission(member.role, check.permission)
          }
        }
        break
    }
    
    return result
  }

  private hasPermission(role: Role, permission: string): boolean {
    // Define role hierarchy and permissions
    const roleHierarchy: Record<Role, string[]> = {
      'Owner': ['*'], // All permissions
      'Admin': [
        'read', 'write', 'delete', 'invite_members', 'manage_members',
        'manage_settings', 'manage_storage', 'manage_permissions'
      ],
      'Member': ['read', 'write', 'invite_members'],
      'Viewer': ['read'],
      'Guest': ['read'] // Limited read access
    }
    
    const rolePermissions = roleHierarchy[role] || []
    return rolePermissions.includes('*') || rolePermissions.includes(permission)
  }

  // Hierarchy Management
  async getOrganizationHierarchy(organizationId: string): Promise<OrganizationHierarchy | null> {
    const organization = this.organizations.get(organizationId)
    if (!organization) return null
    
    const groups = await this.getOrganizationGroups(organizationId)
    const projects = await this.getOrganizationProjects(organizationId)
    
    // Calculate totals
    const allMembers = new Set<string>()
    organization.members.forEach(m => allMembers.add(m.user_id))
    groups.forEach(g => g.members.forEach(m => allMembers.add(m.user_id)))
    projects.forEach(p => p.members.forEach(m => allMembers.add(m.user_id)))
    
    const totalStorageUsed = organization.storage_quota.used_gb + 
      projects.reduce((total, p) => total + p.storage_quota.used_gb, 0)
    
    return {
      organization,
      groups,
      projects,
      total_members: allMembers.size,
      total_storage_used_gb: totalStorageUsed
    }
  }

  // Utility methods
  private generateId(): string {
    return 'id_' + Math.random().toString(36).substring(2, 9)
  }

  private initializeMockData(): void {
    // Create a sample organization for testing
    const mockOrg: Organization = {
      id: 'org_sample_123',
      name: 'MaidSafe Foundation',
      description: 'Building the decentralized web for everyone',
      created_at: new Date('2024-01-01'),
      updated_at: new Date(),
      owner_id: 'user_owner_123',
      members: [
        {
          user_id: 'user_owner_123',
          display_name: 'Foundation Owner',
          four_word_address: 'foundation.owner.main.here',
          role: 'Owner',
          joined_at: new Date('2024-01-01')
        },
        {
          user_id: 'user_admin_456',
          display_name: 'Tech Lead',
          four_word_address: 'tech.lead.admin.here',
          role: 'Admin',
          joined_at: new Date('2024-01-15')
        },
        {
          user_id: 'user_member_789',
          display_name: 'Developer',
          four_word_address: 'developer.team.member.here',
          role: 'Member',
          joined_at: new Date('2024-02-01')
        }
      ],
      has_file_system: true,
      storage_quota: {
        allocated_gb: 100,
        used_gb: 25.5,
        available_gb: 74.5,
        last_updated: new Date()
      },
      settings: {
        visibility: 'public',
        default_member_role: 'Member',
        allow_member_invitations: true,
        require_approval_for_joins: false
      },
      groups: [],
      projects: []
    }
    
    this.organizations.set(mockOrg.id, mockOrg)

    // Create sample groups
    const mockGroup: Group = {
      id: 'group_general_123',
      name: 'General Discussion',
      description: 'Main communication channel for the organization',
      organization_id: 'org_sample_123',
      created_at: new Date('2024-01-02'),
      updated_at: new Date(),
      creator_id: 'user_owner_123',
      members: mockOrg.members.map(m => ({ ...m, role: m.role })),
      has_file_system: false,
      chat_settings: {
        message_retention_days: 365,
        allow_file_sharing: true,
        allow_voice_messages: true,
        allow_video_calls: true
      },
      inherited_permissions: true
    }

    this.groups.set(mockGroup.id, mockGroup)
    mockOrg.groups.push(mockGroup)

    // Create sample project
    const mockProject: Project = {
      id: 'project_communitas_123',
      name: 'Communitas Development',
      description: 'P2P collaboration platform development',
      organization_id: 'org_sample_123',
      parent_group_id: mockGroup.id,
      created_at: new Date('2024-01-05'),
      updated_at: new Date(),
      owner_id: 'user_admin_456',
      members: [
        mockOrg.members[1], // Tech Lead as owner
        mockOrg.members[2]  // Developer as member
      ],
      has_file_system: true,
      storage_quota: {
        allocated_gb: 50,
        used_gb: 12.3,
        available_gb: 37.7,
        last_updated: new Date()
      },
      status: 'active',
      deadline: new Date('2024-12-31'),
      priority: 'high',
      settings: {
        allow_external_collaborators: true,
        require_approval_for_changes: true,
        version_control_enabled: true,
        backup_enabled: true
      }
    }

    this.projects.set(mockProject.id, mockProject)
    mockOrg.projects.push(mockProject)
    
    this.organizations.set(mockOrg.id, mockOrg)
  }
}

// Singleton instance
export const organizationService = new OrganizationService()
