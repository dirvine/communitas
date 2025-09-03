Communitas System Specification

1. Introduction

Communitas is the flagship application of the Saorsa ecosystem.  It aims to combine the best aspects of modern messaging and collaboration platforms—namely chat, file sharing, voice/video calling and project workspaces—into a fully decentralised system.  The platform is built on a post‑quantum‑secure P2P stack and provides a new kind of web where every entity is reachable via a human‑readable four‑word address.  In this document we specify the architecture, components and interactions that make up the Communitas system.  Where possible we cite existing documentation and source files from the Saorsa codebase to ground the design in concrete artefacts.

2. Scope and Entities

Communitas treats entities as first‑class citizens.  An entity can be one of:
	•	Individual – a single user with their own identity, storage and communication capabilities.
	•	Organisation – a grouping of individuals that can contain sub‑groups, channels and projects.
	•	Group – a small set of identities (friends, colleagues) with shared chat and storage.
	•	Channel – a topic‑based space within an organisation or group; it inherits membership from its parent and has its own storage and web directory.
	•	Project – a container for a specific piece of work, with its own members, storage and web presence.

Every entity is identified by a Four‑Word Identity: four dictionary words chosen from the four-word-networking crate.  These words are hashed using BLAKE3 to produce a DHT key; the resulting record contains an identity packet with a public key, signature and metadata ￼.  Because the words are human‑friendly yet cryptographically bound to the identity, all entities share a uniform naming scheme ￼.  The identity packet includes the entity’s public key, signature over the four words, storage addresses and optional metadata such as content type and version ￼.  Individuals may attach crypto payment addresses such as Bitcoin, Ethereum and Autonomi tokens to their packet.

3. High‑Level Architecture

Communitas is implemented as a Tauri desktop application with a React front‑end.  The front‑end displays a WhatsApp‑like chat list with an Organisation tab.  Clicking an organisation reveals its Individuals, Groups, Channels and Projects.  Each entity can send messages (text, audio, video), participate in calls and access its shared storage container.  The back‑end is written in Rust and communicates with the Saorsa P2P stack via tauri::command endpoints.  Key architectural layers are:
	1.	Application Layer – the React/Tauri interface, routing user interactions to the backend.
	2.	Storage Engine Layer – manages encryption, erasure‑coding and policy enforcement.  It interfaces with saorsa-fec for forward‑error‑corrected shards and with saorsa-seal for threshold sealing.
	3.	Network Layer – responsible for node discovery, DHT operations and transport.  It integrates the trust‑weighted Kademlia DHT (saorsa-core), QUIC transport with PQC binding (ant-quic) and the placement engine for shard distribution.
	4.	Saorsa Ecosystem – supporting crates like saorsa-pqc (post‑quantum primitives), saorsa-mls (MLS messaging), saorsa-fec (erasure coding), saorsa-seal (threshold encryption) and four-word-networking for address generation.

In the Saorsa architecture document the storage model is described as content‑addressed with Reed–Solomon erasure coding ￼.  This erasure coding provides resilience against node churn and loss: data is split into k data shards plus m parity shards, any k of which can reconstruct the original.  The encryption engine uses the FEC client, the MLS client and a cipher suite to encrypt and chunk data ￼.

4. Identity and Addressing

4.1 Four‑Word Generation

The four-word-networking crate provides a deterministic mapping between 256‑bit keys and four human‑readable words.  When an entity is created, a fresh ML‑DSA (post‑quantum) key pair is generated.  The public key is hashed to derive a 256‑bit identifier, which is then encoded into four words.  These words are the entity’s primary address.  The private key signs the four words as proof of ownership ￼.  The four words are also included in the identity packet so that others can verify the signature.

4.2 Identity Packet Structure

An identity packet in the DHT includes:
	•	public_key – the entity’s ML‑DSA public key.
	•	signature – a signature of the four words using the private key ￼.
	•	storage_addresses – a list of network addresses, each pointing to a storage container ￼.
	•	network_forwards – optional direct communication endpoints ￼.
	•	content_type – enumerated as Individual, Organisation, Project, Group, Channel or Document ￼.
	•	website_root – optional pointer to the root of the entity’s Markdown website ￼.
	•	metadata – flexible key–value metadata, including crypto payment addresses.

Entities publish their identity packet to the DHT under the hash of their four words.  The DHT enforces validation rules: the words must exist in the dictionary, the DHT key must match the BLAKE3 hash of the words, and the signature must verify against the included public key ￼.  Updates to the packet are signed by the same private key, ensuring that only the owner can modify their record.

5. Storage System

5.1 Storage Containers

Every entity has a storage container—a logical root directory within the system.  The container holds files, folders, and a special subdirectory called web.  Containers are versioned; updates result in new manifests with new content hashes.  The container’s address is the hash of its four‑word identity; this address is stored in the entity’s identity packet for discovery.

5.2 Two‑Stage Backup Mechanism

Communitas employs a two‑stage backup strategy:
	1.	Group‑level FEC shards – When a group (or channel, project or organisation) stores data, the data is erasure‑coded using saorsa-fec.  The k+m shards are distributed across the devices of all group members.  Each member stores a share of every file (encrypted and signed).  This local replication allows group members to back each other up without relying solely on the DHT.  Threshold sealing via saorsa-seal encrypts the content key such that any t members can reconstruct it; fewer than t cannot decrypt.  Local shards are stored on disk in a separate “sealed shards” directory.
	2.	Network‑level DHT storage – In addition to local shards, a copy of the erasure‑coded shards is stored on the Saorsa DHT.  The DHT uses a trust‑weighted Kademlia protocol to replicate each shard to k distinct nodes based on XOR distance and trust scores.  This ensures that even if group members go offline, the data remains recoverable.  Shard placement is biased by EigenTrust scores to favour reliable peers, and Reed–Solomon parameters are chosen based on desired durability ￼.

The combination of local and DHT storage provides high availability: local peers can serve each other quickly, while the DHT offers a fallback and global accessibility.  All shards are encrypted; the encryption key is sealed using threshold cryptography for group data or the individual’s key for personal data.  When retrieving an object, the system first attempts to reconstruct it from local shards; if that fails it fetches from the DHT.

5.3 Web Directory and Markdown Web

Within each storage container there is a web directory.  The web/home.md file (and any other Markdown files) forms the basis of the Markdown web.  The four‑word identity of the container acts as a website address; the BLAKE3 hash of the four words becomes the DHT key for the site root.  A typical site might look like this:￼

ocean-forest-mountain-river/
├── index.md
├── blog/
│   ├── 2025-01-15-post.md
│   └── 2025-01-20-update.md
└── links/
    └── connections.md

Cross‑site links use the four‑word identity of the target, optionally with a path: [My Project](river-stone-cloud-dream/projects/alpha.md) ￼.  Clicking such a link triggers a DHT lookup for the target identity, then fetches the Markdown file using the storage system.  Because all content is Markdown, pages can be collaboratively edited by group members.  The front‑end provides a Markdown editor and viewer with image and video embedding support.

6. Messaging and Communication

6.1 Text, Audio and Video

Messaging is built on the Messaging Layer Security (MLS) protocol via the saorsa-mls crate.  When a conversation (individual chat, group, channel or project) is created, an MLS group is formed with all participants.  TreeKEM key exchange provides forward secrecy and post‑compromise security.  Messages are encrypted and signed; each participant maintains a ratchet state.  For audio and video calls the same MLS context is used to derive session keys.  ant-quic provides QUIC streams multiplexed for media; PQC channel binding derives keys from ML‑KEM shared secrets.

6.2 Video Conferencing and Screen Sharing

Video conferences use multi‑stream QUIC sessions.  The initiating peer opens a “conference” stream and invites participants via the MLS group.  Each participant sends and receives media frames over their own streams.  Screen sharing leverages the same pipeline: the screen is captured, encoded, and sent over a separate stream.  Bandwidth and latency are managed by ant‑quic’s congestion control; QoS classes ensure control messages are prioritised over bulk media.

6.3 Threshold Encryption for Group Data

For group or project storage, encryption keys are sealed using the saorsa-seal crate.  A t-of-n Shamir secret sharing scheme is applied to the content key.  Each share is encrypted to a group member’s public key; any t members can cooperate to decrypt the data.  This prevents a single compromised device from decrypting group files.  Group messages themselves use MLS, but file storage uses threshold sealing to protect long‑term data.

7. Network Layer and Routing

7.1 DHT and Trust Bias

The system relies on a trust‑weighted Kademlia DHT for storage and lookup.  Peers maintain buckets of contacts sorted by XOR distance; multiple parallel queries are used for lookups.  An EigenTrust algorithm runs periodically to compute global reputation scores from local interaction receipts.  These scores influence bucket eviction and next‑hop selection.  Nodes with poor trust are avoided for storing shards or as forwarders.  Providers also advertise their available capacity; this information will later feed into a PUT‑priced storage market.

7.2 Hyperbolic Routing and Greedy Assist

In addition to Kademlia, the system optionally embeds nodes in a hyperbolic plane.  Each node maintains a coordinate derived from network connectivity.  During lookup the client first attempts a greedy hop in hyperbolic space; if no progress can be made it falls back to Kademlia.  This “greedy assist” reduces path stretch and lowers latency on large networks.

7.3 NAT Traversal and Connectivity

ant-quic handles NAT traversal via QUIC’s built‑in hole punching.  Peers exchange transport addresses through the identity packet and via MLS messages.  When connecting to a peer, the application tries all known addresses in parallel until one succeeds.

7.4 Storage Market (Future Work)

While group members store data for free, the network will support paid storage for large public datasets.  A PUT‑priced market will allow users to purchase additional capacity using the Autonomi token on Arbitrum.  Providers advertise their free space; a price function increases as capacity fills up.  Shards are only accepted by providers if accompanied by a valid payment receipt.  This market is outside the scope of the initial implementation but the system should be designed to integrate it easily.

8. Security Model

8.1 Cryptographic Primitives

All identities use post‑quantum cryptography.  Key pairs are generated using ML‑DSA for signatures and ML‑KEM for key exchange ￼.  Four‑word addresses are signed to prevent squatting; the signature is verified on lookup ￼.  Storage encryption uses symmetric keys derived via HKDF from namespace keys; these keys are then sealed with threshold cryptography as needed.

8.2 Data Confidentiality and Integrity

Data is encrypted end‑to‑end: chat messages via MLS, files via the encryption engine.  Erasure‑coded shards include checksums and BLAKE3 hashes; corrupted or tampered shards are rejected.  Only authorised members (enough shares or the individual’s private key) can decrypt data.  The DHT refuses to store identity packets that fail dictionary, hash or signature checks ￼.  Each write to the DHT is signed and authenticated.

8.3 Access Control and Permissions

Permissions are enforced at both the storage layer and the UI.  Storage policies define replication factors, retention periods, encryption modes and access permissions.  The Policy Manager validates operations and audits them ￼.  In the UI, entities cannot see containers they are not a member of.  When a new member is added to a group or project, the threshold shares are rotated to revoke access from removed members.

9. User Interface and Experience

The front‑end follows a clean, modern design inspired by WhatsApp and Slack with a hierarchical navigation structure. The application supports both legacy and experimental UI modes for progressive migration.

9.1 Navigation Hierarchy

**Top-Level Sections:**
- **Organization** - Corporate and structured collaboration spaces
- **Groups & People** - Personal and informal communication spaces

**Organization Structure:**
- **Organizations** - Top-level corporate entities
  - **Projects** - Specific work initiatives with dedicated storage and communication
  - **Groups** - Departmental or cross-functional teams
  - **Channels** - Topic-based discussion spaces within organizations
  - **People** - Individual members within the organization

**Groups & People Structure:**
- **Groups** - Personal friend groups and communities
- **People** - Individual contacts and direct messaging

9.2 Entity Features

Each entity (Organization, Project, Group, Channel, Individual) provides:
- **Communication**: Text messaging, voice calls, video calls, screen sharing
- **Storage**: Secure file storage with encryption and backup
- **Web Presence**: Markdown-based websites accessible via four-word addresses
- **Presence Status**: Online/offline indicators and activity status
- **Collaboration Tools**: File sharing, collaborative editing, task management

9.3 UI Components

- **Main Sidebar**: Hierarchical navigation with expandable sections
- **Chat View**: Message history with real-time updates and media support
- **Call Controls**: Audio/video call initiation and screen sharing
- **File Browser**: Entity storage with drag-and-drop upload
- **Web Tab**: Markdown website editor and viewer
- **Call Panels**: Floating windows for active calls with participant controls

9.4 Implementation Status

**✅ Implemented:**
- Basic navigation structure (Organization/Groups & People)
- Identity management with four-word addresses
- Theme switching (light/dark modes)
- Responsive design for desktop
- Network health monitoring
- Encryption status indicators

**🚧 In Development:**
- Full hierarchical organization structure
- Voice/video calling integration
- Screen sharing capabilities
- Advanced file browser with web tab
- Collaborative markdown editing
- Presence status and unread counts

**📅 Planned:**
- Complete WebRTC integration
- Advanced collaboration features
- Plugin system for extensions
- Mobile responsive design

10. Local AI Integration

Communitas plans to integrate local AI capabilities.  Each client will ship with a compact language model running entirely on the user’s device.  The AI can summarise conversations, auto‑generate meeting notes, suggest file names or assist with Markdown authoring.  Crucially, no data used by the AI leaves the user’s machine.  This preserves the privacy of conversations and files while providing intelligent assistance.  Model updates are delivered through the Saorsa network but are applied client‑side.

11. Implementation Plan
	1.	Core Libraries – finalise saorsa-pqc primitives and four-word-networking dictionary.  Implement ant-quic transport with PQC channel binding.
	2.	DHT and Routing – integrate trust‑weighted Kademlia, EigenTrust scoring and hyperbolic greedy assist into saorsa-core.
	3.	Erasure Coding and Seal – release saorsa-fec for encode/decode and saorsa-seal for threshold sealing.  Integrate these into a new StorageEngine.
	4.	Messaging Layer – complete MLS implementation (saorsa-mls) with PQC ciphersuites.  Expose group management functions.
	5.	Communitas Application – build Tauri front‑end with React and implement the organisational hierarchy, chat service, call service, storage browser and Markdown editor.
	6.	Two‑Stage Backup – implement local shard distribution and retrieval logic.  Add user settings for local shard storage limits.
	7.	Web Directory – implement packing/publishing/resolving of Markdown websites.  Provide GUI for editing and linking.
	8.	Future Features – integrate the paid storage market, AI assistance and mobile clients.

12. Future Enhancements

Beyond the initial release, Communitas will add:
	•	Storage Market – enabling users to purchase extra space and provide storage for others.
	•	Mobile Applications – native iOS/Android clients using the same P2P protocols.
	•	Federated Search – search across four‑word identities and content metadata.
	•	Extensible Plugins – allow developers to build extensions, such as calendar integrations or custom editors.

13. Conclusion

This specification describes how Communitas combines a human‑friendly addressing scheme, strong cryptography, erasure‑coded storage and a modern UI to create a privacy‑respecting collaboration platform.  By leveraging the Saorsa ecosystem, users gain complete control over their identity, data and communications.  The two‑stage backup strategy ensures resilience even in small groups, while the future storage market enables scalability for public data.  The integration of local AI further demonstrates that powerful features need not come at the cost of privacy.  Together, these elements offer a blueprint for a new, decentralised internet built on trust and freedom.￼
