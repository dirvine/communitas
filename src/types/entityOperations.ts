/**
 * Entity Operations Types
 * Distinguishes between creating new entities (we own them)
 * and adding existing entities (we're joining them)
 */

// ============= Create Operations (Generate New Four-Words) =============

export interface CreateNewOrganizationInput {
  displayName: string;
  description?: string;
  // Four-Words will be generated automatically via saorsa_core
}

export interface CreateNewGroupInput {
  displayName: string;
  description?: string;
  organizationId?: string; // Optional for organization groups
  // Four-Words will be generated automatically
}

export interface CreateNewChannelInput {
  organizationId: string;
  displayName: string;
  description?: string;
  isPrivate?: boolean;
  // Four-Words will be generated automatically
}

export interface CreateNewProjectInput {
  organizationId: string;
  displayName: string;
  description?: string;
  // Four-Words will be generated automatically
}

export interface CreateNewContactInput {
  displayName: string;
  email?: string;
  relationship?: 'friend' | 'family' | 'colleague' | 'acquaintance';
  // Four-Words will be generated automatically for the contact entity
}

// ============= Add Operations (Use Existing Four-Words) =============

export interface AddExistingOrganizationInput {
  fourWords: string; // Required - the existing org's Four-Words
  displayName?: string; // Optional local alias
  // Will fetch org details from network using Four-Words
}

export interface AddExistingGroupInput {
  fourWords: string; // Required - the existing group's Four-Words
  displayName?: string; // Optional local alias
  organizationId?: string; // Optional organization context
  // Will fetch group details from network
}

export interface AddExistingChannelInput {
  fourWords: string; // Required - the existing channel's Four-Words
  organizationId: string;
  displayName?: string; // Optional local alias
  // Will fetch channel details from network
}

export interface AddExistingProjectInput {
  fourWords: string; // Required - the existing project's Four-Words
  organizationId: string;
  displayName?: string; // Optional local alias
  // Will fetch project details from network
}

export interface AddExistingContactInput {
  fourWords: string; // Required - the person's Four-Word identity
  displayName?: string; // Optional local alias/nickname
  relationship?: 'friend' | 'family' | 'colleague' | 'acquaintance';
  // Will resolve contact's identity from network
}

// ============= Common Types =============

export type EntityOperationMode = 'create' | 'add';

export interface EntityOperationResult {
  success: boolean;
  entityId: string; // Local ID (temporary for create, resolved for add)
  fourWords: string; // The Four-Words (generated or provided)
  isOwned: boolean; // True for create, false for add
  needsSync: boolean; // True if operation is queued for sync
  error?: string;
}

// ============= Validation =============

export interface FourWordsValidationResult {
  isValid: boolean;
  normalized?: string; // Four-Words in standard format (dash-separated)
  error?: string; // Specific validation error if invalid
}

// ============= Bootstrap Connection =============

export interface BootstrapConnectionInput {
  fourWords: string; // Friend's computer Four-Word connection identity
  saveAsBootstrap?: boolean; // Save as a trusted bootstrap node
  label?: string; // Optional label for this connection
}

export interface ConnectionIdentity {
  userFourWords: string; // Our user identity Four-Words
  endpointFourWords: string; // Our connection endpoint Four-Words (external IP)
  bootstrapNodes: string[]; // Current bootstrap node Four-Words
  isConnected: boolean;
  peerCount: number;
}