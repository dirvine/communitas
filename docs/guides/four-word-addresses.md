# Four-Word Addresses: Network Identity & DNS Replacement

Comprehensive guide to Communitas' dual use of the four-word-networking crate for both network addressing and human-readable identities.

## Overview

Communitas uses the `four-word-networking` crate for two distinct but complementary purposes:

1. **Network Address Encoding**: Converting IP addresses and ports into memorable word sequences
2. **DNS Replacement**: Human-readable entity identifiers that serve as network locations

Both uses leverage the same carefully curated dictionary of simple English words to provide anti-phishing protection and easy verification.

---

## Use Case 1: Network Address Encoding

### Purpose

Encode IP addresses (with ports) into human-readable word sequences for:
- Bootstrap node discovery
- Peer-to-peer connection sharing
- Network configuration
- Remembering connection endpoints

### How It Works

The four-word-networking crate encodes IP addresses and ports into words:

**IPv4 Encoding** (Always 4 words):
```rust
use four_word_networking::{encode_socket_addr, decode_socket_addr};

// Encode IPv4 address with port
let addr = "192.168.1.100:8080".parse()?;
let four_words = encode_socket_addr(&addr);
// → "ocean-forest-moon-star"

// Decode back to socket address
let decoded = decode_socket_addr("ocean-forest-moon-star")?;
// → 192.168.1.100:8080
```

**IPv6 Encoding** (More than 4 words):
```rust
// Encode IPv6 address with port
let addr = "[2001:0db8:85a3::8a2e:0370:7334]:8080".parse()?;
let words = encode_socket_addr(&addr);
// → "ocean-forest-moon-star-valley-river-cloud-wind"
// (8+ words to encode full IPv6 address)

// Decode back
let decoded = decode_socket_addr(&words)?;
// → [2001:0db8:85a3::8a2e:0370:7334]:8080
```

### Key Points

- **IPv4**: Always encodes to exactly 4 words (includes port)
- **IPv6**: Requires more than 4 words to encode the full address (includes port)
- **Bidirectional**: Can encode and decode perfectly
- **Port Included**: The port number is encoded in the word sequence

### Use Cases

**Bootstrap Nodes**:
```typescript
// Share bootstrap node addresses
const bootstrapNodes = [
  "ocean-forest-moon-star",      // 192.168.1.100:8080
  "valley-river-cloud-wind",     // 10.0.1.50:8080
];

// Connect to bootstrap nodes
for (const words of bootstrapNodes) {
  const addr = decode_socket_addr(words);
  await connect(addr);
}
```

**Peer Discovery**:
```typescript
// Share your peer address with others
const myAddress = encode_socket_addr(my_socket_addr);
console.log(`Connect to me at: ${myAddress}`);
// → "Connect to me at: mountain-lake-tree-bird"
```

**Configuration Files**:
```toml
[network]
# Human-readable network addresses
listen = "ocean-forest-moon-star"  # 0.0.0.0:8080
bootstrap = [
  "valley-river-cloud-wind",       # Public bootstrap node
  "mountain-lake-tree-bird"        # Backup bootstrap node
]
```

---

## Use Case 2: DNS Replacement (Entity Identifiers)

### Purpose

Replace DNS and traditional addressing with four-word identifiers for:
- User identities
- Organization identities
- Group identities
- Channel identities
- Project identities
- Any publishable entity

### Why Four Words?

**Anti-Phishing Protection**:
- Only dictionary words are valid
- Prevents lookalike character substitution (0 for O, 1 for l, etc.)
- Easy to verify authenticity out-of-band
- Phonetically distinct words prevent mishearing

**Human Usability**:
- Easy to remember (vs. cryptographic hashes)
- Easy to say over phone/voice
- Easy to write down
- Easy to type and verify

### The Dictionary

The four-word-networking crate provides a carefully curated dictionary:

**Selection Criteria**:
- Common English words (4-8 letters)
- Easy to spell
- Phonetically distinct (no homophones)
- No offensive terms
- No similar-sounding words
- No easily confused words

**Word Categories**:
```
Nature:  ocean, forest, mountain, valley, river
Weather: cloud, wind, rain, storm, snow
Objects: star, moon, tree, stone, leaf
Animals: bird, wolf, eagle, deer, bear
Actions: jump, swim, fly, run, walk
```

**Dictionary Size**: 2048 words (provides sufficient entropy for unique identities)

### How Entity Identifiers Work

```
1. User selects 4 words from dictionary
       ↓
2. Words are validated (all must be in dictionary)
       ↓
3. Hash of four-word sequence becomes network location
       ↓
4. Entity data published to that location in DHT
       ↓
5. Anyone can find entity by its four-word identifier
```

### Entity Types

**User Identity**:
```typescript
{
  fourWords: "ocean-forest-moon-star",
  displayName: "Alice",
  deviceName: "MacBook Pro",
  entityType: "User"
}
```

**Organization Identity**:
```typescript
{
  fourWords: "valley-river-cloud-wind",
  displayName: "Acme Corporation",
  entityType: "Organization"
}
```

**Group Identity**:
```typescript
{
  fourWords: "mountain-lake-tree-bird",
  displayName: "Engineering Team",
  members: ["ocean-forest-moon-star", ...],
  entityType: "Group"
}
```

**Channel Identity**:
```typescript
{
  fourWords: "stone-river-wind-leaf",
  displayName: "#general",
  entityType: "Channel"
}
```

### Publishing Data to Four-Word Identities

Each entity gets three virtual disks:

**Private Disk**:
- Encrypted, local-only storage
- Only entity owner can access
- For sensitive documents, credentials, etc.

**Public Disk**:
- Content-addressed, distributed storage
- Anyone can read
- For public documents, websites, etc.

**Shared Disk**:
- Group-accessible with shared encryption
- For team collaboration, shared files

**Example - Publishing a Website**:
```rust
// 1. Create entity with four-word identity
let entity = create_entity("ocean-forest-moon-star")?;

// 2. Write website content to public disk
entity.public_disk.write("/index.md", markdown_content)?;
entity.public_disk.write("/about.md", about_content)?;

// 3. Publish website root hash
let website_root = entity.public_disk.get_root_hash()?;
entity.set_website_root(website_root)?;

// 4. Anyone can now access website at:
// http://ocean-forest-moon-star.communitas/
```

**Example - Private Group Data**:
```rust
// Group "mountain-lake-tree-bird" shares private documents
let group = get_entity("mountain-lake-tree-bird")?;

// Write to shared disk (encrypted for group members)
group.shared_disk.write("/meeting-notes.md", notes)?;
group.shared_disk.write("/roadmap.md", roadmap)?;

// Only group members can decrypt and read
```

### DNS-Free Website Publishing

Four-word identities enable DNS-free websites:

**Traditional Web**:
```
Domain: example.com
DNS:    example.com → 192.168.1.100
HTTP:   GET http://example.com/index.html
```

**Communitas**:
```
Identity: ocean-forest-moon-star
DHT:      ocean-forest-moon-star → entity with website_root hash
Content:  Fetch content from DHT using root hash
Render:   Display markdown as website
```

**Advantages**:
- ✅ No domain registration required
- ✅ No DNS servers needed
- ✅ Can't be taken down by registrars
- ✅ Works offline (cached content)
- ✅ Content-addressed (verifiable integrity)
- ✅ Decentralized (no single point of failure)

### Network Location Resolution

When you use a four-word entity identifier:

```
1. Hash the four words → location key
2. Query DHT for entity data at that key
3. Retrieve entity metadata (public key, website root, etc.)
4. Access published content (websites, public files, etc.)
```

**Example Flow**:
```typescript
// User wants to view "ocean-forest-moon-star" website
const fourWords = "ocean-forest-moon-star";

// 1. Validate words are from dictionary
if (!validate_four_words(fourWords)) {
  throw new Error("Invalid four-word address");
}

// 2. Look up entity in DHT
const entity = await dht_lookup(fourWords);
// Returns: {
//   fourWords: "ocean-forest-moon-star",
//   publicKey: "...",
//   websiteRoot: "blake3_hash_of_website_content",
//   timestamp: 1699876543
// }

// 3. Fetch website content using root hash
const content = await fetch_content(entity.websiteRoot);

// 4. Render website
renderMarkdown(content);
```

---

## Validation & Anti-Phishing

### Dictionary Validation

**All words must be from the dictionary**:
```rust
use four_word_networking::validate_words;

// Valid - all words in dictionary
assert!(validate_words("ocean-forest-moon-star"));

// Invalid - "invalid" not in dictionary
assert!(!validate_words("ocean-forest-moon-invalid"));

// Invalid - typo "occean"
assert!(!validate_words("occean-forest-moon-star"));
```

### Anti-Phishing Protection

Dictionary validation prevents common phishing attacks:

**Character Substitution (Blocked)**:
```
Legitimate: ocean-forest-moon-star
Phishing:   0cean-forest-moon-star  (zero instead of O)
Result:     ❌ Rejected - "0cean" not in dictionary

Legitimate: valley-river-cloud-wind
Phishing:   valley-rlver-cloud-wind  (lowercase L instead of I)
Result:     ❌ Rejected - "rlver" not in dictionary
```

**Homophone Attacks (Prevented)**:
```
The dictionary excludes similar-sounding words:
✅ "their" in dictionary
❌ "there" excluded (sounds identical)
❌ "they're" excluded (sounds similar)

This prevents confusion when sharing addresses verbally.
```

**Typo Detection**:
```rust
// Typo suggestions
let suggestions = suggest_corrections("occean-forest-moon-star");
// → ["ocean-forest-moon-star"] (suggests correct spelling)

let suggestions = suggest_corrections("mountian-lake-tree-bird");
// → ["mountain-lake-tree-bird"] (suggests correct spelling)
```

---

## Comparison: Network Encoding vs Entity Identifiers

| Feature | Network Encoding | Entity Identifiers |
|---------|-----------------|-------------------|
| **Purpose** | Encode IP:port addresses | Human-readable entity names |
| **Word Count** | IPv4=4, IPv6=8+ | Always exactly 4 |
| **Includes Port** | Yes (embedded) | No (port not relevant) |
| **Reversible** | Yes (decode to IP:port) | No (maps to entity data) |
| **Dictionary** | Same 2048-word dictionary | Same 2048-word dictionary |
| **Anti-Phishing** | Yes (dictionary validation) | Yes (dictionary validation) |
| **Use Case** | Share connection endpoints | Identity/DNS replacement |

**Key Difference**:
- **Network Encoding**: Bidirectional (IP ⟷ words), includes port
- **Entity Identifiers**: One-way (words → DHT location), no port concept

---

## Examples

### Network Address Sharing

```rust
// Alice shares her peer address with Bob
let my_addr = "192.168.1.100:8080".parse()?;
let words = encode_socket_addr(&my_addr);
println!("Connect to me at: {}", words);
// → "Connect to me at: ocean-forest-moon-star"

// Bob connects using the words
let addr = decode_socket_addr("ocean-forest-moon-star")?;
connect_to_peer(addr).await?;
// → Connected to 192.168.1.100:8080
```

### Entity Website Publishing

```rust
// Alice creates an organization with four-word identity
let org = create_entity("valley-river-cloud-wind")?;

// Publish company website
org.public_disk.write("/index.md", homepage)?;
org.public_disk.write("/products.md", products)?;
org.public_disk.write("/contact.md", contact)?;

let website_root = org.public_disk.get_root_hash()?;
org.set_website_root(website_root)?;

// Anyone can now visit:
// - valley-river-cloud-wind.communitas/
// - No DNS needed
// - Content verified via hash
```

### Group Collaboration

```rust
// Engineering team creates group identity
let team = create_entity("mountain-lake-tree-bird")?;

// Share private documents with team
team.shared_disk.write("/sprint-planning.md", planning)?;
team.shared_disk.write("/architecture.md", arch_docs)?;

// Publish public documentation
team.public_disk.write("/api-docs.md", api_docs)?;

// Team members can access via four-word identifier
// mountain-lake-tree-bird → team entity → shared content
```

---

## Best Practices

### For Network Addresses

1. **Always Include Port**: The encoding includes port automatically
2. **Validate Before Decode**: Check word validity before attempting decode
3. **Handle IPv6**: Be prepared for >4 words with IPv6 addresses
4. **Use for Bootstrap**: Share bootstrap node addresses as four words

### For Entity Identifiers

1. **Choose Memorable Words**: Pick words that are easy to remember
2. **Verify Out-of-Band**: Confirm four-word identifiers through trusted channels
3. **Check Dictionary**: Validate all four words are in the dictionary
4. **No Spaces/Typos**: Use exact hyphen-separated lowercase words
5. **Backup Identity**: Securely store private keys associated with identity

### Anti-Phishing

1. **Always Validate**: Never trust four-word addresses without validation
2. **Visual Verification**: Show dictionary validation status in UI
3. **Typo Suggestions**: Offer corrections for common typos
4. **Out-of-Band Confirm**: Verify important addresses via phone/in-person

---

## API Reference

### Network Address Encoding

```rust
// Encode socket address to words
pub fn encode_socket_addr(addr: &SocketAddr) -> String;

// Decode words to socket address
pub fn decode_socket_addr(words: &str) -> Result<SocketAddr>;

// Examples
let words = encode_socket_addr(&"192.168.1.100:8080".parse()?);
let addr = decode_socket_addr("ocean-forest-moon-star")?;
```

### Entity Identifier Validation

```rust
// Validate four-word sequence
pub fn validate_words(four_words: &str) -> bool;

// Get word suggestions for typos
pub fn suggest_corrections(invalid_words: &str) -> Vec<String>;

// Check if word is in dictionary
pub fn is_valid_word(word: &str) -> bool;

// Examples
assert!(validate_words("ocean-forest-moon-star"));
assert!(!validate_words("ocean-forest-moon-invalid"));
let suggestions = suggest_corrections("occean-forest-moon-star");
```

### Entity Operations

```typescript
// Create entity with four-word identity
await invoke('create_entity', {
  fourWords: 'ocean-forest-moon-star',
  displayName: 'Alice',
  entityType: 'User'
});

// Resolve entity by four-word identifier
const entity = await invoke('resolve_entity', {
  fourWords: 'ocean-forest-moon-star'
});

// Publish to entity's public disk
await invoke('publish_content', {
  fourWords: 'ocean-forest-moon-star',
  path: '/index.md',
  content: markdown
});

// Retrieve entity's website
const website = await invoke('get_website', {
  fourWords: 'ocean-forest-moon-star'
});
```

---

## Troubleshooting

### "Invalid Four-Word Address"

**Problem**: Address fails validation

**Causes**:
- Word not in dictionary (typo or phishing attempt)
- Wrong number of words
- Missing hyphens or incorrect separators
- Uppercase characters (must be lowercase)

**Solutions**:
```typescript
// Check each word individually
const words = address.split('-');
for (const word of words) {
  if (!is_valid_word(word)) {
    console.log(`Invalid word: ${word}`);
  }
}

// Get suggestions
const suggestions = suggest_corrections(address);
console.log('Did you mean:', suggestions);
```

### "Cannot Decode Network Address"

**Problem**: Decode fails for network address encoding

**Causes**:
- Not a valid network address encoding
- Entity identifier being used as network address
- Corrupted word sequence

**Solutions**:
- Verify this is a network address encoding (not entity identifier)
- Check word count (IPv4=4 words, IPv6=8+ words)
- Validate all words are from dictionary

### "Entity Not Found"

**Problem**: Four-word entity identifier doesn't resolve

**Causes**:
- Entity hasn't been published to DHT
- Network connectivity issues
- Typo in four-word identifier

**Solutions**:
- Verify identifier is correct (check with sender)
- Check network connectivity
- Try again later (DHT propagation delay)

---

## See Also

- [four-word-networking crate](https://crates.io/crates/four-word-networking) - Rust implementation
- [Getting Started](getting-started.md) - Initial setup guide
- [Authentication](authentication.md) - Identity and security
- [Architecture](../architecture/) - System design overview
- [Virtual Disks](virtual-disks.md) - Per-entity storage

---

**Four words for networking, four words for identity! 🌊🌲🌙⭐**
