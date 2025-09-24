# Overview

Communitas is a local-first, peer-to-peer collaboration platform that reimagines how teams communicate and work together without relying on central servers. It combines messaging, file sharing, voice/video calling, and project workspaces into a single decentralized application built on post-quantum cryptography.

The platform uses Four-Word identities (like "ocean-forest-moon-star") for human-verifiable addressing and enables DNS-free website publishing. Every entity (person, group, project, organization) functions both as a collaboration space and as a website, creating a new model for the decentralized web.

Built with Tauri v2 for the desktop application and headless nodes for network infrastructure, Communitas provides the functionality of WhatsApp, Dropbox, Slack, and Zoom in a single privacy-focused, quantum-resistant platform.

# User Preferences

Preferred communication style: Simple, everyday language.

# System Architecture

## Frontend Architecture

**Technology Stack**: React 18 with TypeScript, Material-UI components, and Vite build system. The frontend implements a dual UI system with both Legacy (Material-UI) and Experimental (WhatsApp-style) interfaces.

**State Management**: React Context with hooks for authentication, encryption, and navigation. The application maintains offline-first capabilities using IndexedDB for local storage and syncs when network connectivity returns.

**Routing**: React Router handles single-page application navigation with tab-based organization (Overview, Messages, Network, Storage, Advanced sections).

## Backend Architecture

**Desktop Application**: Tauri v2 framework with Rust backend implementing the desktop app logic. The Rust backend provides Tauri commands for frontend-backend communication and integrates with the Saorsa Core library.

**Core Library**: Built on Saorsa Core v0.3.17 which provides DHT (Distributed Hash Table), QUIC networking, identity management, group messaging, and virtual disk storage. This handles the peer-to-peer networking foundation.

**Cryptography**: Post-quantum cryptography using ML-DSA (signatures) and ML-KEM (key exchange) with ChaCha20-Poly1305 for symmetric encryption. All encryption operations use the saorsa-fec crate for forward error correction.

**Storage Engine**: Implements four distinct storage policies:
- PrivateMax: Local-only with random encryption keys
- PrivateScoped: DHT storage with namespace-derived keys  
- GroupScoped: Shared group storage with member access control
- PublicMarkdown: Public content with convergent encryption

## Data Storage Solutions

**Local Storage**: Virtual disks with content addressing via BLAKE3 hashing and Reed-Solomon erasure coding for data protection. Each entity (user, group, project) has separate encrypted storage containers.

**Distributed Storage**: DHT-based storage across the peer-to-peer network with configurable replication and geographic routing for performance optimization.

**Offline Capabilities**: IndexedDB for browser-based storage with automatic sync when network connectivity is restored. All operations work offline-first.

## Authentication and Authorization

**Identity System**: Four-Word identities generated using the four-word-networking crate with deterministic mapping between 256-bit keys and human-readable words. Each identity includes ML-DSA key pairs for cryptographic verification.

**Access Control**: Namespace-isolated encryption keys derived using HKDF-SHA256. Group-based access control with membership verification and cryptographic signatures for all identity claims.

**Device Management**: Secure credential storage using keyring integration with device-specific key derivation and signature-based ownership verification.

# External Dependencies

## Third-Party Services

**Network Infrastructure**: Digital Ocean droplets for bootstrap nodes across 6 global regions (AMS3, LON1, FRA1, NYC3, SFO3, SGP1) to facilitate initial peer discovery.

**Browser Integration**: Chrome DevTools MCP integration for advanced debugging and testing capabilities during development.

## Core Libraries

**Saorsa Ecosystem**: 
- saorsa-core (v0.3.17): DHT, QUIC, identities, groups, messaging
- saorsa-fec: Forward error correction and encryption
- saorsa-mls: Messaging Layer Security implementation
- four-word-networking (v2.3): Human-readable address generation

**Networking**: 
- ant-quic: QUIC protocol implementation
- WebRTC: Peer-to-peer voice/video calling with screen sharing

**Development Tools**:
- Playwright: End-to-end testing framework
- Puppeteer: Browser automation for testing
- MCP servers: Model Context Protocol for AI agent integration

## Build and Testing Infrastructure

**Frontend Build**: Vite with Hot Module Replacement, TypeScript compilation, and Node.js polyfills for browser compatibility.

**Backend Build**: Rust 2024 edition with Cargo for dependency management and cross-compilation support for multiple platforms.

**Testing**: Vitest for frontend unit tests, Rust's built-in test framework for backend testing, and comprehensive integration test suites for P2P network functionality.

**CI/CD**: GitHub Actions for automated testing, security auditing, and performance monitoring across multiple platforms and browsers.