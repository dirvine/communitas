# Communitas - P2P Diagnostic Chat Application Specification

## Overview
Communitas is a diagnostic chat application that serves as both a comprehensive test harness for the P2P Foundation network and a demonstration of its capabilities. It provides real-time visibility into all network layers while functioning as a fully-featured group chat application.

## Core Features

### 1. Chat Functionality
- **Group Chat**: Support for up to 20 participants per group
- **Message Persistence**: 1-week retention for all messages
- **File Sharing**: Up to 5MB file transfers
- **Media Support**: Voice and video calls when both participants are online
- **Identity System**: Persistent identities using 4-word networking addresses

### 2. Network Diagnostics
- **Tabbed Interface** showing different diagnostic views:
  - **Overview Tab**: High-level network health and peer connections
  - **Messages Tab**: Active chat interface
  - **Network Tab**: Peer discovery, NAT traversal visualization
  - **Storage Tab**: DHT operations, replication status
  - **Advanced Tab**: Packet-level inspection (on-demand)
  
### 3. Test Harness Capabilities
- **Property Testing Integration**: Automated testing scenarios
- **Multi-node Testing**: Support for running multiple test nodes
- **Network Simulation**: Controllable network conditions
- **Performance Metrics**: Real-time throughput and latency monitoring

## Technical Requirements

### Networking
- **Transport**: QUIC-only implementation using ant-quic v0.5.0
- **Bootstrap**: Hardcoded bootstrap node at `quic.saorsalabs.com:8888`
- **NAT Traversal**: Automatic with visual feedback
- **Addressing**: 4-word networking addresses for all participants

### Storage
- **DHT Integration**: Full Kademlia DHT with K=8 replication
- **Message Storage**: 1-week TTL with automatic cleanup
- **File Storage**: Chunked storage with content addressing

### Security
- **Encryption**: Quantum-resistant cryptography (ML-KEM/ML-DSA)
- **Identity**: Persistent Ed25519 keys mapped to 4-word addresses
- **Groups**: Threshold cryptography for group management

### User Interface
- **Framework**: Tauri v2 with React frontend
- **Layout**: Modern tabbed interface with responsive design
- **Themes**: Support for light/dark modes with smooth transitions
- **Components**: Material-UI or Tailwind CSS for polished UX
- **Responsiveness**: Mobile-first design that works on all screen sizes

## Implementation Phases

### Phase 1: Core Chat (Week 1)
- Basic message sending/receiving
- 4-word identity system
- Bootstrap node connectivity
- Simple TUI with tabs

### Phase 2: Diagnostics (Week 2)
- Network visualization
- DHT operation monitoring
- NAT traversal display
- Performance metrics

### Phase 3: Advanced Features (Week 3)
- Voice/video calling
- File sharing
- Multi-node testing
- Property testing integration

### Phase 4: Polish & Testing (Week 4)
- Comprehensive testing
- Documentation
- Performance optimization
- Crates.io publication

## Testing Strategy

### Property Testing
- Network partitioning scenarios
- Message ordering guarantees
- Storage consistency
- Identity uniqueness

### Integration Testing
- Multi-node deployments on SSH server
- Cross-platform compatibility
- Network resilience testing
- Performance benchmarking

### Coverage Goals
- Increase codebase coverage from ~60% to 80%
- Focus on edge cases and error paths
- Stress testing with 20+ concurrent users

## Success Criteria
1. **Functional**: All chat features working reliably
2. **Performance**: <100ms message latency in optimal conditions
3. **Reliability**: 99.9% message delivery success rate
4. **Diagnostics**: Clear visibility into all network operations
5. **Testing**: 80% code coverage achieved
6. **Documentation**: Complete user and developer guides

## Non-Goals (Initial Version)
- Mobile app (terminal-only initially)
- Message search functionality
- User blocking/moderation
- Performance warnings (until solid metrics established)

## Dependencies
- `saorsa-core`: P2P Foundation core library
- `ant-quic v0.5.0`: QUIC transport with PQC
- `tauri v2`: Cross-platform desktop app framework
- `react`: Modern UI framework
- `tokio`: Async runtime
- `proptest`: Property testing framework

## Deliverables
1. `communitas` crate published to crates.io
2. Comprehensive documentation
3. Example configurations
4. Test suite with property tests
5. Performance benchmarks