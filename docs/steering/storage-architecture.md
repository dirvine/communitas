# Storage Architecture & Web Publishing System

## Overview

Communitas implements a distributed, encrypted file storage system with integrated web publishing capabilities. Each entity (individuals, groups, organizations, channels, projects) has its own isolated storage mechanism with built-in Forward Error Correction (FEC) and DHT-based distribution.

## Core Storage Concepts

### Entity-Based Storage Isolation

Every entity in the system maintains its own independent storage space:

```
Storage Hierarchy:
├── Personal Storage (Individual Users)
├── Group Storage (Friend Groups)
├── Organization Storage
│   ├── Organization Root Storage
│   ├── Group Storage (within Organization)
│   ├── Channel Storage
│   ├── Project Storage
│   └── Individual Member Storage
└── Each entity has identical capabilities:
    ├── Voice Calling
    ├── Video Calling
    ├── Messaging
    └── File Storage (this document's focus)
```

### Storage Capabilities Per Entity

Each storage mechanism provides:
- **File Storage**: General file storage with encryption
- **Markdown Documents**: Primary content format
- **Media Embedding**: Images (initially), video (future)
- **Web Publishing**: Public website functionality
- **Collaborative Editing**: Real-time document collaboration
- **Version Control**: Document history and recovery

## Technical Architecture

### Storage Stack

```
┌─────────────────────────────────────────┐
│         User Interface Layer            │
│   (Markdown Editor/Browser/File UI)     │
├─────────────────────────────────────────┤
│         Application Layer               │
│   (Entity Management & Permissions)     │
├─────────────────────────────────────────┤
│         Encryption Layer                │
│   (Pre-FEC & Pre-DHT Encryption)       │
├─────────────────────────────────────────┤
│    Forward Error Correction Layer       │
│      (Saorsa FEC - Reed-Solomon)       │
├─────────────────────────────────────────┤
│       Distribution Layer                │
│  (DHT Storage & P2P Synchronization)   │
├─────────────────────────────────────────┤
│         Network Layer                   │
│    (Saorsa Core P2P Infrastructure)    │
└─────────────────────────────────────────┘
```

### Encryption Flow

```
Original File
    ↓
[First Encryption Layer]
    ↓
Encrypted Data
    ↓
[Forward Error Correction]
    ↓
FEC Chunks
    ↓
[Second Encryption Layer]
    ↓
Encrypted FEC Chunks
    ↓
[DHT Storage]
```

## Forward Error Correction (FEC) Implementation

### Using Saorsa FEC Crate

The system uses the `saorsa-fec` crate for Reed-Solomon error correction throughout the storage layer (see `src-tauri/src/storage/reed_solomon_manager.rs`):

```rust
use saorsa_fec::{FecCodec, FecParams};

// Configuration for group storage
const DATA_SHARDS: usize = 6;  // ~60% threshold
const PARITY_SHARDS: usize = 4; // ~40% redundancy
const TOTAL_SHARDS: usize = DATA_SHARDS + PARITY_SHARDS;

// For individual storage (no FEC needed)
// Files are stored directly with encryption only
```

### Recovery Requirements

- **Group Storage**: Requires 60% of members online to recover data
- **Individual Storage**: Direct storage without FEC splitting
- **Redundancy**: 40% of nodes can be offline without data loss
- **Distribution**: Shards distributed across group members

### FEC Strategy by Entity Type

```yaml
Individual Users:
  - No FEC splitting (single owner)
  - Direct encrypted storage
  - Personal redundancy via device sync

Groups/Organizations/Channels/Projects:
  - FEC with 60% recovery threshold
  - Shards distributed to all members
  - Automatic replication when members join
  - Graceful degradation with member churn
```

## Web Publishing System

### Markdown-Based Websites

Each entity can publish a website using markdown files:

```
entity_storage/
├── files/           # General file storage
├── documents/       # Collaborative documents
└── web/            # Website directory
    ├── home.md     # Homepage (like index.html)
    ├── about.md    # Additional pages
    ├── blog/       # Subdirectories supported
    │   ├── post1.md
    │   └── post2.md
    └── assets/     # Images and resources
        ├── logo.png
        └── banner.jpg
```

### Web Directory Structure

```markdown
# home.md Example

# Welcome to Our Organization

![Organization Logo](assets/logo.png)

## About Us
[Learn more about our mission](about.md)

## Latest Updates
- [Project Alpha Launch](blog/alpha-launch.md)
- [Community Guidelines](docs/guidelines.md)

## Resources
- [Download Our App](downloads/app.md)
- [Documentation](docs/home.md)
```

### URL Addressing Scheme

Each entity's website is accessible via:

```
Address Format: [entity-four-words]/web/[path]

Examples:
- ocean-forest-moon-star/web/home.md (Personal site)
- acme-corp-global-tech/web/home.md (Organization)
- acme-corp-global-tech/projects/alpha/web/home.md (Project)
- dev-team-acme-corp/web/home.md (Group within org)
```

### DHT Storage for Web Content

```rust
// DHT Key Generation
fn generate_web_key(entity_id: &FourWordAddress, path: &str) -> DhtKey {
    // Hash of four-word identity serves as base
    let base_hash = blake3::hash(entity_id.as_bytes());
    
    // Append path for specific files
    let full_key = format!("{}/web/{}", base_hash, path);
    DhtKey::from(blake3::hash(full_key.as_bytes()))
}

// Storage structure in DHT
struct WebContent {
    entity_id: FourWordAddress,
    path: String,
    content: Vec<u8>,        // Encrypted markdown
    content_type: String,    // "text/markdown" or "image/*"
    timestamp: u64,
    signature: Signature,
}
```

## Markdown Browser Implementation

### Core Features

The integrated markdown browser provides:

```typescript
interface MarkdownBrowser {
  // Navigation
  navigate(address: FourWordAddress, path: string): Promise<void>;
  back(): void;
  forward(): void;
  refresh(): Promise<void>;
  
  // Rendering
  renderMarkdown(content: string): HTMLElement;
  handleInternalLinks(link: string): void;
  loadExternalEntity(address: FourWordAddress): Promise<void>;
  
  // Media
  loadImage(src: string): Promise<Blob>;
  embedVideo(src: string): Promise<void>; // Future
  
  // Security
  sanitizeContent(markdown: string): string;
  validateLinks(links: string[]): LinkStatus[];
}
```

### Browser UI Components

```
┌─────────────────────────────────────────────┐
│ [←] [→] [⟳] ocean-forest-moon-star/web/home │
├─────────────────────────────────────────────┤
│                                             │
│  # Welcome to My Personal Site             │
│                                             │
│  This is my personal space on Communitas   │
│                                             │
│  [About Me](about.md) | [Projects](proj/)  │
│                                             │
│  ![Profile Picture](assets/profile.jpg)    │
│                                             │
└─────────────────────────────────────────────┘
```

### Link Resolution

```typescript
// Internal link: Relative to current entity
"[About](about.md)" → same-entity/web/about.md

// Cross-entity link: Full four-word address
"[Partner Org](partner-tech-global-corp/web/home.md)"

// Asset reference: Relative path
"![Logo](assets/logo.png)" → same-entity/web/assets/logo.png

// External web link: Not supported initially
"[External](https://example.com)" → Blocked/Warning
```

## Markdown Editor Integration

### Collaborative Editing Features

```typescript
interface MarkdownEditor {
  // Editing
  createDocument(path: string): Document;
  editDocument(doc: Document): void;
  saveDocument(doc: Document): Promise<void>;
  
  // Collaboration (via Yjs)
  enableCollaboration(doc: Document): Y.Doc;
  syncWithPeers(ydoc: Y.Doc): void;
  handleConflicts(conflicts: Conflict[]): Resolution;
  
  // Web Publishing
  publishToWeb(doc: Document): Promise<void>;
  previewAsWebpage(doc: Document): void;
  updateWebContent(path: string, content: string): Promise<void>;
  
  // Media Management
  insertImage(file: File): Promise<string>;
  embedVideo(url: string): string; // Future
  manageAssets(assets: Asset[]): void;
}
```

### Editor Modes

1. **Document Mode**: For general documents and notes
2. **Web Publishing Mode**: For website content with preview
3. **Collaborative Mode**: Real-time multi-user editing
4. **Offline Mode**: Local editing with sync when online

## Storage Implementation Details

### File Storage Operations

```rust
// Core storage trait for all entities
trait EntityStorage {
    async fn store_file(&self, file: File) -> Result<FileId>;
    async fn retrieve_file(&self, id: FileId) -> Result<File>;
    async fn delete_file(&self, id: FileId) -> Result<()>;
    async fn list_files(&self) -> Result<Vec<FileMetadata>>;
    
    // Web-specific operations
    async fn publish_web(&self, content: WebContent) -> Result<()>;
    async fn get_web_content(&self, path: &str) -> Result<WebContent>;
    async fn update_web(&self, path: &str, content: Vec<u8>) -> Result<()>;
}

// Group storage with FEC
struct GroupStorage {
    entity_id: FourWordAddress,
    members: Vec<PeerId>,
    fec_encoder: ReedSolomonEncoder,
    dht: Arc<DHT>,
}

impl GroupStorage {
    async fn store_with_fec(&self, data: Vec<u8>) -> Result<()> {
        // 1. Encrypt data
        let encrypted = self.encrypt(&data)?;
        
        // 2. Apply FEC
        let shards = self.fec_encoder.encode(&encrypted)?;
        
        // 3. Distribute shards to members
        for (shard, member) in shards.iter().zip(&self.members) {
            let encrypted_shard = self.encrypt_shard(shard)?;
            self.send_to_member(member, encrypted_shard).await?;
        }
        
        // 4. Store metadata in DHT
        let metadata = FileMetadata {
            id: FileId::new(),
            shards: shards.len(),
            members: self.members.clone(),
            threshold: 0.6,
        };
        self.dht.put(metadata).await?;
        
        Ok(())
    }
    
    async fn retrieve_with_fec(&self, id: FileId) -> Result<Vec<u8>> {
        // 1. Get metadata from DHT
        let metadata = self.dht.get(id).await?;
        
        // 2. Collect shards from online members
        let mut shards = Vec::new();
        for member in &metadata.members {
            if let Ok(shard) = self.request_from_member(member).await {
                shards.push(self.decrypt_shard(shard)?);
                if shards.len() >= metadata.threshold_count() {
                    break; // Have enough shards
                }
            }
        }
        
        // 3. Reconstruct with FEC
        let encrypted = self.fec_encoder.decode(&shards)?;
        
        // 4. Decrypt and return
        self.decrypt(&encrypted)
    }
}
```

### Media Handling

```rust
enum MediaType {
    Image(ImageFormat),
    Video(VideoFormat), // Future
    Audio(AudioFormat), // Future
}

struct MediaHandler {
    max_image_size: usize,     // e.g., 10MB
    max_video_size: usize,     // e.g., 100MB (future)
    supported_formats: Vec<String>,
}

impl MediaHandler {
    async fn process_image(&self, image: Vec<u8>) -> Result<ProcessedImage> {
        // 1. Validate format and size
        self.validate_image(&image)?;
        
        // 2. Generate thumbnails
        let thumbnail = self.create_thumbnail(&image)?;
        
        // 3. Optimize for web
        let optimized = self.optimize_image(&image)?;
        
        Ok(ProcessedImage {
            original: image,
            thumbnail,
            web_optimized: optimized,
        })
    }
    
    // Future: Video processing
    async fn process_video(&self, video: Vec<u8>) -> Result<ProcessedVideo> {
        // Transcoding, thumbnail generation, etc.
        todo!("Video support coming in Phase 2")
    }
}
```

## Security Considerations

### Encryption Layers

1. **Pre-FEC Encryption**: Protects data before splitting
2. **Post-FEC Encryption**: Protects individual shards
3. **Transport Encryption**: TLS for shard distribution
4. **At-Rest Encryption**: Local storage encryption

### Access Control

```rust
struct AccessControl {
    entity_id: FourWordAddress,
    owner: PeerId,
    permissions: HashMap<PeerId, Permission>,
}

enum Permission {
    Read,
    Write,
    Admin,
    WebPublish,
}

impl AccessControl {
    fn can_publish_web(&self, user: &PeerId) -> bool {
        matches!(
            self.permissions.get(user),
            Some(Permission::Admin | Permission::WebPublish)
        )
    }
}
```

### Content Validation

- Markdown sanitization to prevent XSS
- Image format validation
- Size limits enforcement
- Link validation for safety
- Content type verification

## Performance Optimization

### Caching Strategy

```rust
struct StorageCache {
    web_cache: LruCache<String, WebContent>,    // Frequently accessed web pages
    file_cache: LruCache<FileId, Vec<u8>>,     // Recent files
    shard_cache: LruCache<ShardId, Vec<u8>>,   // FEC shards
    
    // Cache configuration
    max_web_size: usize,      // e.g., 100MB
    max_file_size: usize,     // e.g., 500MB
    max_shard_size: usize,    // e.g., 200MB
    ttl: Duration,            // Time to live
}
```

### DHT Optimization

- **Web Content Priority**: Home.md cached aggressively
- **Predictive Loading**: Preload linked pages
- **Shard Locality**: Keep popular shards on more nodes
- **Bandwidth Management**: Rate limiting for large files

## Implementation Phases

### Phase 1: Core Storage (Current)
- ✅ Basic file storage per entity
- ✅ Encryption implementation
- ✅ DHT integration
- 🚧 FEC implementation with Saorsa FEC
- 🚧 Basic markdown editor

### Phase 2: Web Publishing (Next)
- 📅 Web directory structure
- 📅 Markdown browser implementation
- 📅 home.md as index page
- 📅 Internal link navigation
- 📅 Image embedding support

### Phase 3: Advanced Features
- 📅 Collaborative editing via Yjs
- 📅 Version control for documents
- 📅 Advanced media handling
- 📅 Cross-entity linking
- 📅 Search functionality

### Phase 4: Optimization
- 📅 Video support
- 📅 Advanced caching
- 📅 CDN-like distribution
- 📅 Progressive web loading
- 📅 Offline mode improvements

## API Reference

### Storage Commands (Tauri)

```rust
#[tauri::command]
async fn store_file(
    entity_id: String,
    file_path: String,
    content: Vec<u8>,
) -> Result<FileId, String>;

#[tauri::command]
async fn retrieve_file(
    entity_id: String,
    file_id: String,
) -> Result<Vec<u8>, String>;

#[tauri::command]
async fn publish_web_content(
    entity_id: String,
    path: String,
    markdown: String,
) -> Result<(), String>;

#[tauri::command]
async fn browse_entity_web(
    entity_address: String,
    path: Option<String>,
) -> Result<RenderedContent, String>;
```

### Frontend Integration

```typescript
// Storage service
class StorageService {
  async storeFile(
    entityId: string,
    file: File
  ): Promise<string>;
  
  async publishWebPage(
    entityId: string,
    path: string,
    content: string
  ): Promise<void>;
  
  async browseWeb(
    address: FourWordAddress,
    path?: string
  ): Promise<RenderedPage>;
}

// React components
function EntityStorage({ entity }: { entity: Entity }) {
  // File management UI
  // Web publishing UI
  // Markdown editor integration
}

function MarkdownBrowser() {
  // Browser UI
  // Navigation
  // Rendering
}
```

## Success Metrics

### Storage Performance
- File upload/download speed
- FEC recovery success rate (>99.9%)
- Storage availability (>99.5%)
- Cache hit ratio (>80%)

### Web Publishing
- Page load time (<500ms local, <2s remote)
- Markdown rendering accuracy
- Link resolution success rate
- Media loading performance

### User Experience
- Editor responsiveness
- Browser navigation smoothness
- Collaborative editing latency (<100ms)
- Storage UI intuitiveness

## Security Checklist

- [ ] All data encrypted before FEC
- [ ] All shards encrypted before distribution
- [ ] DHT keys properly hashed
- [ ] Markdown content sanitized
- [ ] Image formats validated
- [ ] Size limits enforced
- [ ] Access control verified
- [ ] Rate limiting implemented
- [ ] Audit logging enabled
- [ ] Recovery testing completed

This architecture provides a robust, distributed storage system with integrated web publishing capabilities, ensuring data availability, security, and user-friendly access to content across all entity types in the Communitas platform.