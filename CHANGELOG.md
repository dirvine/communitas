# Changelog

All notable changes to the Communitas app will be documented in this file.

## [0.2.8] - 2025-08-13

### Added
- **Four-Word Identity Packet Architecture**: Complete implementation of comprehensive identity system
  - Identity packets with public keys, storage addresses, and network forwards  
  - DHT validation rules preventing spam with dictionary word constraints
  - Signature-based ownership verification for all identity claims
  - Universal entity system supporting individuals, organizations, projects, groups, and channels
  - Foundation for decentralized markdown web with human-readable addressing
  
- **Enhanced Identity Commands**: 10 new Tauri commands for identity management
  - `generate_four_word_identity` - Generate new four-word identities using four-word-networking crate
  - `validate_four_word_identity` - Validate format and dictionary membership
  - `check_identity_availability` - Check if identity is claimed
  - `claim_four_word_identity` - Claim and register new identity
  - `calculate_dht_id` - Generate BLAKE3 hash for DHT key
  - `get_identity_info` - Retrieve complete identity information with visual elements
  - Additional batch operations and statistics commands

- **Four-Word-Networking Integration**: Full integration with four-word-networking v2.3
  - Curated dictionary ensures controlled vocabulary
  - Word validation using `FourWordAdaptiveEncoder`
  - Consistent with existing four-word addressing in the ecosystem

## [0.2.7] - 2025-01-11

### Added
- **Network Auto-Connection**: App automatically connects to P2P network on startup
  - Sequential connection attempts to Digital Ocean bootstrap nodes
  - DHT initialization with fallback creation
  - Comprehensive error handling and logging
  
- **Projects Entity Support**: Full implementation of Projects within Organizations
  - Project creation dialog with priority levels (Low/Medium/High/Critical)
  - Deadline tracking and storage allocation (1-100GB)
  - Project member management with four-word addresses
  - Voice/video conferencing and file sharing per project
  
- **Enhanced Organization Dashboard**: Three-tab navigation system
  - Projects tab with project cards and management
  - Groups tab for team collaboration
  - Individuals tab for member directory
  - Unified search across all entity types
  
- **Backend Integration**: Improved service layer with Tauri backend
  - `create_organization_dht` command for organization creation
  - `create_group_dht` command for group creation
  - `create_project_dht` command for project creation
  - Automatic fallback to mock data when backend unavailable

- **Context Switching Navigation**: Hierarchical navigation system
  - NavigationContext provider for centralized navigation state
  - Breadcrumb navigation with visual back button
  - Context-aware sidebar showing Personal/Organization hierarchy
  - Expandable organization structure with groups and projects
  - Real-time context indicators and navigation history

### Changed
- OrganizationService now attempts backend calls before using mock data
- Organization dashboard displays real-time member counts and storage usage
- Entity cards show communication options (voice/video/messaging/files)

### Fixed
- Network connection timeout handling
- DHT initialization when not available from P2P node
- Port conflicts during development (port 1420)

## [0.2.6] - Previous Release

- Initial Communitas implementation
- Basic organization structure
- P2P networking foundation