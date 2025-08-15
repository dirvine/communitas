# Technology Stack

## Core Framework

### Tauri
- **Version**: 2.x
- **Purpose**: Desktop application framework
- **Benefits**: 
  - Small bundle size
  - Native performance
  - Security-first design
  - Cross-platform deployment

## Frontend Stack

### React
- **Version**: 18.x (from package.json)
- **Purpose**: User interface library
- **Language**: TypeScript
- **Build Tool**: Vite

### Development Tools
- **Bundler**: Vite
- **Language**: TypeScript
- **Package Manager**: npm
- **Hot Reload**: Vite dev server

### UI Dependencies
Based on package.json analysis:
- **@mui/material**: Material UI components
- **@emotion/react**: CSS-in-JS styling
- **react-router-dom**: Routing
- **framer-motion**: Animations
- **monaco-editor**: Code editing
- **yjs**: Collaborative editing
- **react-markdown**: Markdown rendering

## Backend Stack

### Rust
- **Version**: 2024 edition (1.85+)
- **Purpose**: System integration and business logic
- **Framework**: Tauri Core
- **Build Tool**: Cargo

### Key Rust Dependencies (from Cargo.toml)
- **tauri**: Core framework (v2.6.1)
- **serde**: Serialization/deserialization
- **tokio**: Async runtime
- **saorsa-core**: P2P networking
- **sqlx**: Database access (SQLite)
- **keyring**: Secure credential storage
- **chacha20poly1305**: Encryption
- **blake3**: Hashing

## Build & Development

### Development Environment
```bash
# Prerequisites
- Node.js 18+
- Rust 1.85+
- Platform-specific build tools

# Development
npm install
npm run tauri dev
```

### Build Pipeline
```bash
# Production build
npm run tauri build

# Platform-specific outputs
- Windows: .msi, .exe
- macOS: .dmg, .app
- Linux: .deb, .rpm, .AppImage
```

## Platform Integration

### System APIs (via Tauri)
- File system access
- Native dialogs
- System notifications
- Window management
- System tray integration
- Auto-updater support

### Security Features
- Content Security Policy
- API allowlisting
- Secure IPC communication
- No Node.js runtime exposure

## Development Tools

### Code Quality
- **Rust**: `cargo clippy`, `cargo fmt`
- **TypeScript**: ESLint, Prettier
- **Testing**: Vitest (frontend), cargo test (backend)

### IDE Support
- VS Code with Rust Analyzer
- Tauri extension for VS Code
- React Developer Tools

## Performance Characteristics

### Bundle Size
- Significantly smaller than Electron
- No Node.js runtime overhead
- Native OS integrations

### Memory Usage
- Lower memory footprint
- Rust memory safety
- Efficient resource management

### Startup Time
- Fast application startup
- Native performance
- Optimized for desktop use

## Version Management

### Frontend Dependencies
Managed via `package.json`:
- React ecosystem packages
- Development tooling
- Type definitions

### Backend Dependencies
Managed via `Cargo.toml`:
- Tauri framework
- Rust crates
- System integration libraries

## Browser Compatibility

Not applicable - desktop application uses system webview:
- Windows: WebView2
- macOS: WKWebView
- Linux: WebKitGTK

## Deployment Strategy

### Distribution
- Direct download from website
- Platform-specific app stores
- Auto-update mechanism via Tauri

### CI/CD
- GitHub Actions for multi-platform builds
- Automated testing and quality checks
- Release automation

This technology stack provides a modern, secure, and performant foundation for cross-platform desktop application development.