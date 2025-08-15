# Four-Word Identity System Specification

## Version: 1.0.0
## Date: 2025-01-13
## Feature: Four-Word Identity System with DHT Integration

## Executive Summary

The Four-Word Identity System provides human-memorable network addresses that map directly to DHT identifiers. Each user, organization, or project receives a unique four-word combination (e.g., "ocean-forest-moon-star") that serves as both their memorable identity and their cryptographically verifiable network address.

## System Architecture

### Core Components

#### 1. Identity Generator
- **Purpose**: Create unique, memorable four-word combinations
- **Algorithm**: Cryptographically secure random selection from curated word lists
- **Validation**: Ensure no collisions, offensive combinations, or confusion
- **Word Lists**: 4 categories × 2048 words each = 16.7 trillion combinations

#### 2. DHT Mapper
- **Purpose**: Convert four-word addresses to DHT node IDs
- **Algorithm**: BLAKE3(four-words) → 256-bit DHT ID
- **Verification**: Nodes prove ownership via cryptographic signature
- **Collision Resolution**: Statistical impossibility with 256-bit space

#### 3. Identity Verifier
- **Purpose**: Verify claimed four-word addresses
- **Process**: 
  1. Client claims four-word address
  2. Calculate expected DHT ID via BLAKE3
  3. Verify node at DHT ID responds with valid signature
  4. Cache verification for performance

#### 4. Visual Identity Generator
- **Purpose**: Create consistent visual representation from four-words
- **Features**:
  - Deterministic gradient generation
  - Avatar patterns
  - Theme colors
  - Icon generation

## Technical Specification

### Four-Word Format
```
Structure: word1-word2-word3-word4
Example: ocean-forest-moon-star
Constraints:
- Lowercase only
- Hyphen separator
- No special characters
- No numbers
```

### Word Categories
```
Category 1: Nature (2048 words)
- Elements: ocean, forest, mountain, river
- Weather: storm, rain, snow, thunder
- Celestial: moon, star, sun, comet

Category 2: Colors/Qualities (2048 words)
- Colors: azure, crimson, golden, silver
- Qualities: swift, bright, gentle, strong

Category 3: Objects (2048 words)
- Natural: stone, crystal, pearl, diamond
- Crafted: sword, shield, crown, bridge

Category 4: Concepts (2048 words)
- Abstract: dream, wisdom, courage, harmony
- Actions: dance, sing, fly, glow
```

### DHT Integration
```typescript
interface FourWordIdentity {
  fourWords: string           // "ocean-forest-moon-star"
  dhtId: Uint8Array           // BLAKE3 hash
  publicKey: Uint8Array       // ML-KEM public key
  signature: Uint8Array       // Proof of ownership
  metadata: {
    created: Date
    type: 'personal' | 'organization' | 'project'
    parent?: string           // For org/project hierarchy
    verified: boolean
  }
}
```

### Verification Protocol
```typescript
async function verifyIdentity(fourWords: string): Promise<boolean> {
  // 1. Calculate expected DHT ID
  const expectedId = blake3.hash(fourWords)
  
  // 2. Query DHT for node
  const node = await dht.findNode(expectedId)
  
  // 3. Challenge-response
  const challenge = crypto.randomBytes(32)
  const response = await node.sign(challenge)
  
  // 4. Verify signature
  return crypto.verify(response, challenge, node.publicKey)
}
```

## User Experience

### Identity Creation Flow
1. **Generation**
   - User requests new identity
   - System generates 5 candidate four-word combinations
   - User selects preferred combination
   - System verifies availability

2. **Claiming**
   - Generate ML-KEM keypair
   - Calculate DHT ID from four-words
   - Sign claim with private key
   - Broadcast to network

3. **Verification**
   - Other nodes verify DHT ID matches four-words
   - Verify signature proves ownership
   - Cache verification result

### Visual Identity
```typescript
interface VisualIdentity {
  gradient: string            // Linear gradient CSS
  colors: {
    primary: string           // Main color
    secondary: string         // Accent color
    accent: string           // Highlight color
  }
  avatar: {
    pattern: string          // SVG pattern
    background: string       // Gradient or solid
    initials: string         // First letters
  }
  theme: 'warm' | 'cool' | 'neutral'
}
```

## Security Considerations

### Cryptographic Foundation
- **Hash Function**: BLAKE3 for DHT ID generation
- **Signature**: ML-DSA for quantum resistance
- **Key Exchange**: ML-KEM for secure communications
- **Entropy**: Minimum 128 bits for word selection

### Attack Vectors & Mitigations
1. **Identity Squatting**
   - Mitigation: First-come-first-served with proof-of-work
   - Rate limiting on registrations
   
2. **Collision Attacks**
   - Mitigation: 256-bit hash space
   - Statistical analysis for anomalies

3. **Impersonation**
   - Mitigation: Cryptographic signatures
   - Peer verification network

4. **Dictionary Attacks**
   - Mitigation: Large word space (16.7 trillion)
   - No predictable patterns

## Performance Requirements

### Latency Targets
- Identity generation: < 10ms
- DHT lookup: < 100ms
- Verification: < 50ms
- Visual generation: < 5ms

### Scalability
- Support 1 billion identities
- 100,000 verifications/second
- Sub-linear DHT growth

### Caching Strategy
- LRU cache for recent verifications
- TTL: 1 hour for positive, 5 minutes for negative
- Distributed cache across nodes

## Implementation Phases

### Phase 1: Core Identity System
- Word list curation
- Four-word generator
- BLAKE3 integration
- Basic validation

### Phase 2: DHT Integration
- DHT ID mapping
- Node registration
- Peer discovery
- Verification protocol

### Phase 3: Visual Identity
- Gradient generation
- Avatar system
- Theme extraction
- Icon generation

### Phase 4: Security Hardening
- ML-KEM integration
- ML-DSA signatures
- Rate limiting
- Anomaly detection

### Phase 5: Performance Optimization
- Caching layer
- Batch verification
- Parallel lookups
- CDN for avatars

### Phase 6: Identity Packet Architecture (IMPLEMENTED)
- Identity packet structure with public keys and storage addresses
- DHT validation rules for spam prevention
- Signature-based ownership verification
- Universal entity system (individuals, organizations, projects)
- Markdown web with human-readable linking

## Identity Packet Architecture

### Identity Packet Structure
Each four-word identity resolves to a comprehensive identity packet stored in the DHT:

```rust
pub struct IdentityPacket {
    // Cryptographic Identity
    pub public_key: PublicKey,           // For signature verification
    pub signature: Signature,            // Four words signed with private key
    
    // Network Information  
    pub storage_addresses: Vec<NetworkAddress>, // Current storage locations
    pub network_forwards: Vec<NetworkAddress>,  // Direct communication endpoints
    
    // Content Metadata
    pub content_type: ContentType,       // Personal, Organization, Project, etc.
    pub last_updated: Timestamp,         // For freshness validation
    pub version: u32,                    // Packet version for updates
    
    // Web Presence
    pub website_root: Option<String>,    // Main website/blog entry point
    pub metadata: HashMap<String, Value>, // Flexible metadata storage
}
```

### Universal Entity System
The same four-word mechanism works for all entity types:
- **Individuals**: Personal identity packets (`ocean-forest-mountain-river`)
- **Organizations**: Company/group entities (`company-blue-star-tech`)
- **Projects**: Project-specific identities (`project-quantum-secure`)
- **Groups**: Sub-groups within organizations (`team-alpha-dev-chat`)
- **Channels**: Communication channels (`channel-general-chat`)
- **Documents**: Standalone resources with identity

### Markdown Web Integration
Four-word identities enable a decentralized markdown-based web:
- **Human-readable links**: `[My Friend](ocean-forest-mountain-river)`
- **Document linking**: `[Project Docs](project-alpha/docs/specification.md)`
- **Cross-entity references**: Link between any identities
- **Content addressing**: Stable addresses regardless of server location

### DHT Security Model
The DHT enforces strict validation to prevent spam:
1. **Dictionary Constraint**: Only four-word-networking dictionary words accepted
2. **Hash Integrity**: DHT key must be BLAKE3 hash of the four words  
3. **Signature Proof**: Must prove ownership with private key signature
4. **Packet Validation**: Well-formed packet structure required

## API Specification

### REST Endpoints
```typescript
// Generate new identity
POST /api/identity/generate
Response: {
  candidates: string[]        // 5 four-word options
}

// Claim identity
POST /api/identity/claim
Body: {
  fourWords: string
  publicKey: string
}
Response: {
  success: boolean
  dhtId: string
  signature: string
}

// Verify identity
GET /api/identity/verify/:fourWords
Response: {
  valid: boolean
  dhtId: string
  publicKey: string
  metadata: object
}

// Get visual identity
GET /api/identity/visual/:fourWords
Response: {
  gradient: string
  colors: object
  avatar: string            // Base64 SVG
}
```

### WebSocket Events
```typescript
// Identity claimed
{
  event: 'identity.claimed',
  data: {
    fourWords: string
    dhtId: string
    timestamp: number
  }
}

// Identity verified
{
  event: 'identity.verified',
  data: {
    fourWords: string
    verifier: string
    timestamp: number
  }
}
```

## Testing Requirements

### Unit Tests
- Word generation randomness
- BLAKE3 hashing consistency
- Collision detection
- Visual generation determinism

### Integration Tests
- DHT registration flow
- Peer verification
- Cache behavior
- API endpoints

### Performance Tests
- 1M identity generations
- 100K concurrent verifications
- DHT scaling to 1B nodes
- Visual generation throughput

### Security Tests
- Cryptographic validation
- Attack surface analysis
- Penetration testing
- Fuzzing word combinations

## Success Metrics

### Technical Metrics
- Zero collisions in production
- < 100ms average verification time
- 99.99% uptime for identity service
- < 0.01% false positive rate

### User Metrics
- 90% users remember their four-words after 1 week
- < 30 seconds to complete identity creation
- 95% satisfaction with generated combinations
- < 1% request regeneration

## Migration Strategy

### From Existing Systems
1. Generate four-words for existing users
2. Allow grace period with both systems
3. Gradual migration with fallback
4. Full cutover after 6 months

### Data Preservation
- Maintain mapping of old IDs to four-words
- Export/import identity proofs
- Backup keypairs securely
- Audit trail of changes

## Documentation Requirements

### User Documentation
- Quick start guide
- Four-word best practices
- Security guidelines
- Troubleshooting

### Developer Documentation
- API reference
- Integration guide
- SDK documentation
- Example applications

### Operations Documentation
- Deployment guide
- Monitoring setup
- Incident response
- Scaling procedures

## Compliance & Privacy

### GDPR Compliance
- No personal data in four-words
- Right to be forgotten (identity release)
- Data portability (export keypair)
- Privacy by design

### Security Standards
- NIST compliance for cryptography
- OWASP for web security
- ISO 27001 alignment
- Regular security audits

## Conclusion

The Four-Word Identity System provides a revolutionary approach to network identity, combining human memorability with cryptographic security. By mapping memorable four-word combinations directly to DHT node IDs, we create a system that is both user-friendly and technically robust, forming the foundation for the Communitas unified platform's identity layer.