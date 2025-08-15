# Four-Word Identity Architecture
## The Foundation of a Decentralized Markdown Web

### Table of Contents
1. [Overview](#overview)
2. [Core Architecture](#core-architecture)
3. [Identity Packet Structure](#identity-packet-structure)
4. [The Markdown Web](#the-markdown-web)
5. [DHT Security Model](#dht-security-model)
6. [Universal Entity System](#universal-entity-system)
7. [Implementation Specification](#implementation-specification)
8. [Future Applications](#future-applications)

---

## Overview

The Four-Word Identity Architecture represents a complete reimagining of the internet as a decentralized, human-readable, markdown-based web. Every entity - individuals, organizations, projects, groups, channels - is identified by four dictionary words that hash to a unique DHT location containing their identity packet and storage information.

### Core Innovation
- **Human-readable addresses**: `ocean-forest-mountain-river` instead of IP addresses or hashes
- **Universal identity system**: Same mechanism for all entities
- **Cryptographic security**: Public/private key pairs with signature verification
- **Decentralized storage**: Each entity controls their own data and presentation
- **Markdown-based web**: All content is markdown files, creating a text-first internet

---

## Core Architecture

### Four-Word to DHT Flow
```
Four Words → BLAKE3 Hash → DHT Key → Identity Packet
    ↓
"ocean-forest-mountain-river" → 0x1a2b3c... → DHT[0x1a2b3c...] → {
    publicKey: "...",
    storageAddresses: [...],
    signature: "...",
    metadata: {...}
}
```

### Identity Resolution Process
1. **Input**: Four dictionary words (e.g., `ocean-forest-mountain-river`)
2. **Hash**: BLAKE3(`ocean-forest-mountain-river`) = DHT key
3. **Lookup**: DHT retrieval using the hash as key
4. **Validation**: Verify signature of four words using public key from packet
5. **Access**: Use storage addresses for direct content access

---

## Identity Packet Structure

Each identity packet stored in the DHT contains:

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

pub enum ContentType {
    Individual,      // Personal identity
    Organization,    // Company/group entity
    Project,         // Specific project within organization  
    Group,           // Sub-group within organization
    Channel,         // Communication channel
    Document,        // Standalone document/resource
}
```

### Cryptographic Security
- **Key Generation**: Each identity generates an ML-DSA (quantum-resistant) key pair
- **Signature Process**: Private key signs the four words as proof of ownership
- **Verification**: Anyone can verify identity by checking signature against public key
- **Update Authority**: Only the private key holder can update the identity packet

---

## The Markdown Web

### Vision: A Text-First Internet
The four-word identity system enables a complete reimagining of the web:

#### Website Structure
```
ocean-forest-mountain-river/
├── index.md                 # Homepage
├── blog/
│   ├── 2025-01-15-post.md
│   └── 2025-01-20-update.md
├── projects/
│   ├── project-alpha.md
│   └── project-beta.md
└── links/
    └── connections.md
```

#### Cross-Identity Linking
```markdown
# My Blog Post

Check out my friend's project at [river-stone-cloud-dream](river-stone-cloud-dream/projects/awesome-tool.md).

Our organization's announcement: [company-blue-star-tech](company-blue-star-tech/announcements/2025-expansion.md)

Join our discussion in [team-alpha-dev-chat](team-alpha-dev-chat).
```

#### Universal Link Format
- **Identity**: `[display-name](four-word-identity)`
- **Specific Document**: `[title](four-word-identity/path/to/document.md)`
- **Root Access**: `[website](four-word-identity)` → automatically resolves to index.md

### Content Discovery
- **Human Navigation**: Users can remember and share four-word addresses
- **Link Following**: Click four-word links to traverse the decentralized web
- **Search Integration**: Search by words, content, or metadata
- **Bookmark System**: Save four-word identities as bookmarks

---

## DHT Security Model

### Validation Rules
The DHT enforces strict security to prevent spam and ensure authenticity:

```rust
pub enum DHTPutResult {
    Success,
    InvalidFourWords,     // Words not in dictionary
    InvalidHash,          // Hash doesn't match four words
    InvalidSignature,     // Signature verification failed
    MalformedPacket,      // Packet structure invalid
}

pub fn validate_identity_packet(
    four_words: &str,
    packet: &IdentityPacket,
    dht_key: &[u8],
) -> DHTPutResult {
    // 1. Validate four words are in dictionary
    if !validate_dictionary_words(four_words) {
        return DHTPutResult::InvalidFourWords;
    }
    
    // 2. Verify DHT key matches hash of four words
    let expected_key = blake3::hash(four_words.as_bytes());
    if expected_key.as_bytes() != dht_key {
        return DHTPutResult::InvalidHash;
    }
    
    // 3. Verify signature of four words using public key
    if !verify_signature(&packet.public_key, four_words, &packet.signature) {
        return DHTPutResult::InvalidSignature;
    }
    
    // 4. Validate packet structure
    if !packet.is_well_formed() {
        return DHTPutResult::MalformedPacket;
    }
    
    DHTPutResult::Success
}
```

### Security Properties
- **Dictionary Constraint**: Only four-word-networking dictionary words accepted
- **Hash Integrity**: DHT key must be BLAKE3 hash of the four words
- **Signature Proof**: Must prove ownership with private key signature
- **No Collisions**: Mathematical impossibility of hash collisions
- **Spam Prevention**: Invalid packets are rejected at DHT level

---

## Universal Entity System

### All Entities Use Same Pattern

#### Individual Identity
```
# ocean-forest-mountain-river
Type: Individual
Owner: John Smith
Website: Personal blog and portfolio
```

#### Organization Identity  
```
# company-blue-star-tech
Type: Organization
Owner: Blue Star Technologies Inc.
Website: Corporate site, announcements, job postings
```

#### Project Identity
```
# project-quantum-secure
Type: Project (within company-blue-star-tech)
Owner: Blue Star Technologies Inc.
Website: Project documentation, updates, resources
```

#### Group Identity
```
# team-alpha-dev-chat
Type: Group (within company-blue-star-tech)  
Owner: Development Team Alpha
Website: Team updates, meeting notes, shared resources
```

#### Channel Identity
```
# channel-general-chat
Type: Channel (within team-alpha-dev-chat)
Owner: Team Alpha
Website: Channel archive, important announcements
```

### Hierarchical Relationships
- Organizations can reference their projects: `[Project Alpha](project-alpha-quantum)`
- Projects can reference their teams: `[Dev Team](team-alpha-dev-chat)`
- Teams can reference their channels: `[General Chat](channel-general-chat)`
- Cross-references: Any entity can link to any other entity

---

## Implementation Specification

### Phase 1: Identity Packet System
```rust
// Enhanced identity commands for full packet support
#[tauri::command]
pub async fn create_identity_packet(
    four_words: String,
    private_key: PrivateKey,
    content_type: ContentType,
    storage_addresses: Vec<NetworkAddress>,
    metadata: HashMap<String, Value>,
) -> Result<IdentityPacket, String>

#[tauri::command]  
pub async fn update_identity_packet(
    four_words: String,
    private_key: PrivateKey,
    updates: IdentityPacketUpdate,
) -> Result<(), String>

#[tauri::command]
pub async fn resolve_identity(
    four_words: String,
) -> Result<IdentityPacket, String>
```

### Phase 2: Markdown Web Interface
```typescript
// Web navigation interface
interface MarkdownWeb {
    navigateToIdentity(fourWords: string): Promise<WebsiteContent>;
    loadDocument(fourWords: string, path: string): Promise<MarkdownDocument>;
    createLink(fourWords: string, path?: string): string;
    parseLinks(markdown: string): FourWordLink[];
}

interface WebsiteContent {
    identity: IdentityPacket;
    homePage: MarkdownDocument;
    sitemap: string[];
    metadata: WebsiteMetadata;
}
```

### Phase 3: DHT Integration
```rust
// DHT validation and storage
pub struct SecureDHT {
    dht: DHT,
    validator: IdentityPacketValidator,
}

impl SecureDHT {
    pub async fn put_identity_packet(
        &self,
        four_words: &str,
        packet: IdentityPacket,
    ) -> Result<(), DHTPutResult> {
        let key = blake3::hash(four_words.as_bytes());
        let validation = self.validator.validate(&four_words, &packet, key.as_bytes());
        
        match validation {
            DHTPutResult::Success => {
                self.dht.put(key.as_bytes(), &packet).await
            },
            error => Err(error),
        }
    }
}
```

---

## Future Applications

### MCP Integration (Phase 2)
- **Application Layer**: MCP servers provide interactive applications
- **Service Discovery**: Four-word identities can advertise available MCP services  
- **Resource Access**: Applications can access markdown content through MCP protocols
- **API Endpoints**: Four-word identities can host MCP-based APIs

### Extended Use Cases
- **Personal Blogs**: Individual publishing with human-readable addresses
- **Corporate Sites**: Company websites with easy-to-remember addresses
- **Project Documentation**: Technical documentation with stable addresses
- **Team Collaboration**: Shared spaces with memorable identifiers
- **Academic Publishing**: Research papers and datasets with citation-friendly addresses
- **Community Forums**: Discussion spaces with human-friendly names

### Network Effects
- **Discoverability**: Four-word addresses are easy to share verbally
- **Memorability**: Users can remember important identities
- **Link Stability**: Content addresses don't change with server moves
- **Decentralization**: No central authority controls addressing
- **Censorship Resistance**: Content distributed across network participants

---

## Technical Benefits

### Developer Experience
- **Simple API**: Four words map directly to content
- **No DNS**: Eliminate DNS dependencies and single points of failure
- **Cryptographic Security**: Built-in authentication and integrity
- **Content Addressing**: Links remain valid regardless of server location
- **Offline Capability**: Content can be cached and served locally

### User Experience  
- **Human Addresses**: `ocean-forest-mountain-river` vs `https://xzy123.server.com/user/456`
- **Link Sharing**: Easy to communicate addresses verbally or in text
- **Bookmark Management**: Memorable addresses reduce bookmark clutter
- **Trust Model**: Cryptographic verification of content authenticity
- **Privacy**: No tracking through centralized DNS or servers

### Network Properties
- **Scalability**: DHT distributes load across all network participants
- **Resilience**: No single points of failure in addressing system
- **Global Consistency**: Same four words resolve to same content worldwide
- **Update Mechanism**: Content owners can update their addresses/content
- **Spam Resistance**: Dictionary constraint and cryptographic requirements

---

This architecture represents the foundation for a completely decentralized, human-readable web where every piece of content has a memorable address and cryptographic authenticity. The combination of four-word addressing, DHT storage, and markdown content creates a robust, scalable alternative to the current centralized web infrastructure.