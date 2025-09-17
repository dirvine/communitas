import { Element, ElementConfig, ElementIdentity, ElementCapabilities, ElementMembership, ElementStorage, ElementCommunication, ElementMetadata, ElementType } from '../../types/element';
import { ELEMENT_TEMPLATES, getElementTemplate } from './ElementTemplates';
import { generateFourWords } from '../../utils/fourWords';

export class ElementFactory {
  private static instance: ElementFactory;
  private elements = new Map<string, Element>();

  static getInstance(): ElementFactory {
    if (!ElementFactory.instance) {
      ElementFactory.instance = new ElementFactory();
    }
    return ElementFactory.instance;
  }

  async createElement(config: ElementConfig, creatorId: string): Promise<Element> {
    const template = getElementTemplate(config.type);

    // Generate network identity
    const fourWords = await this.generateUniqueFourWords();
    const networkIdentity = await this.generateNetworkIdentity(fourWords);

    // Create element identity
    const identity: ElementIdentity = {
      id: this.generateId(),
      type: config.type,
      scope: config.type.startsWith('personal') ? 'personal' : 'organization',
      name: config.name,
      description: config.description,
      avatar: config.type === 'personal-individual' ? undefined : this.generateAvatar(config.name),
      fourWords,
      publicKey: networkIdentity.publicKey,
      dhtAddress: networkIdentity.dhtAddress,
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    // Merge capabilities
    const capabilities: ElementCapabilities = {
      ...template.defaultCapabilities,
      ...config.capabilities,
    };

    // Create initial membership
    const membership: ElementMembership[] = [{
      userId: creatorId,
      role: 'owner',
      permissions: ['all'],
      joinedAt: new Date(),
      isActive: true,
    }];

    // Add initial members if provided
    if (config.initialMembers) {
      for (const memberId of config.initialMembers) {
        if (memberId !== creatorId) {
          membership.push({
            userId: memberId,
            role: 'member',
            permissions: ['read', 'write'],
            joinedAt: new Date(),
            isActive: true,
          });
        }
      }
    }

    // Merge storage settings
    const storage: ElementStorage = {
      ...template.defaultStorage,
      ...config.storage,
      webDriveAddress: capabilities.webDrive ? await this.generateWebDriveAddress(fourWords) : undefined,
    };

    // Merge metadata
    const metadata: ElementMetadata = {
      ...template.defaultMetadata,
      ...config.metadata,
    };

    // Create communication state
    const communication: ElementCommunication = {
      unreadCount: 0,
      lastActivity: new Date(),
      typingUsers: [],
    };

    // Create the element
    const element: Element = {
      identity,
      capabilities,
      membership,
      storage,
      communication,
      metadata,
      organizationId: config.organizationId,
      parentId: config.parentId,
    };

    // Store the element
    this.elements.set(identity.id, element);

    return element;
  }

  async updateElement(elementId: string, updates: Partial<Element>): Promise<Element | null> {
    const element = this.elements.get(elementId);
    if (!element) return null;

    const updatedElement = {
      ...element,
      ...updates,
      identity: { ...element.identity, ...updates.identity, updatedAt: new Date() },
      communication: { ...element.communication, ...updates.communication },
    };

    this.elements.set(elementId, updatedElement);
    return updatedElement;
  }

  getElement(elementId: string): Element | null {
    return this.elements.get(elementId) || null;
  }

  getElementsByType(type: ElementType): Element[] {
    return Array.from(this.elements.values()).filter(element => element.identity.type === type);
  }

  getElementsByScope(scope: 'personal' | 'organization'): Element[] {
    return Array.from(this.elements.values()).filter(element => element.identity.scope === scope);
  }

  getElementsByOrganization(organizationId: string): Element[] {
    return Array.from(this.elements.values()).filter(element => element.organizationId === organizationId);
  }

  getElementsByUser(userId: string): Element[] {
    return Array.from(this.elements.values()).filter(element =>
      element.membership.some(member => member.userId === userId && member.isActive)
    );
  }

  async deleteElement(elementId: string): Promise<boolean> {
    const element = this.elements.get(elementId);
    if (!element) return false;

    // Mark as deleted instead of removing
    element.metadata.status = 'deleted';
    element.identity.updatedAt = new Date();

    return true;
  }

  // Helper methods
  private generateId(): string {
    return `element_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private async generateUniqueFourWords(): Promise<string> {
    let fourWords: string;
    let attempts = 0;
    const maxAttempts = 100;

    do {
      fourWords = generateFourWords();
      attempts++;
    } while (this.isFourWordsTaken(fourWords) && attempts < maxAttempts);

    if (attempts >= maxAttempts) {
      throw new Error('Could not generate unique four-word address');
    }

    return fourWords;
  }

  private isFourWordsTaken(fourWords: string): boolean {
    return Array.from(this.elements.values()).some(element =>
      element.identity.fourWords === fourWords
    );
  }

  private async generateNetworkIdentity(fourWords: string): Promise<{ publicKey: string; dhtAddress: string }> {
    // In a real implementation, this would generate actual cryptographic keys
    // For now, we'll create deterministic but unique identifiers
    const hash = await this.simpleHash(fourWords);
    return {
      publicKey: `pk_${hash}`,
      dhtAddress: `dht://${fourWords}`,
    };
  }

  private async generateWebDriveAddress(fourWords: string): Promise<string> {
    return `drive://${fourWords}`;
  }

  private generateAvatar(name: string): string {
    // Generate a simple avatar based on the first letter
    const firstLetter = name.charAt(0).toUpperCase();
    return `avatar://${firstLetter}`;
  }

  private async simpleHash(input: string): Promise<string> {
    let hash = 0;
    for (let i = 0; i < input.length; i++) {
      const char = input.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash).toString(36);
  }

  // Element validation
  validateElementConfig(config: ElementConfig): { valid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (!config.name || config.name.trim().length === 0) {
      errors.push('Element name is required');
    }

    if (!config.type || !ELEMENT_TEMPLATES[config.type]) {
      errors.push('Invalid element type');
    }

    if (config.type.startsWith('organization') && !config.organizationId) {
      errors.push('Organization ID is required for organization elements');
    }

    return {
      valid: errors.length === 0,
      errors,
    };
  }

  // Element cloning (for templates)
  async cloneElement(elementId: string, newName: string, creatorId: string): Promise<Element | null> {
    const originalElement = this.elements.get(elementId);
    if (!originalElement) return null;

    const config: ElementConfig = {
      type: originalElement.identity.type,
      name: newName,
      description: originalElement.identity.description,
      organizationId: originalElement.organizationId,
      capabilities: { ...originalElement.capabilities },
      storage: { ...originalElement.storage },
      metadata: { ...originalElement.metadata },
    };

    return this.createElement(config, creatorId);
  }
}