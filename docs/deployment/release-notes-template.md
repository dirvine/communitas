# Release Notes Template

Use this template for all Communitas releases.

## Template

```markdown
# Communitas v{VERSION}

_Released: {DATE}_

## Highlights

- Key feature or improvement 1
- Key feature or improvement 2
- Key feature or improvement 3

## New Features

### Feature Name
Brief description of the feature and its benefits.

- Bullet point details
- Another detail

## Improvements

- Performance improvement description
- UX enhancement description
- Other improvements

## Bug Fixes

- **#123**: Fixed issue description
- **#456**: Another fix description

## Breaking Changes

None in this release.

(Or list breaking changes with migration steps:)
- **Change description**: Migration steps here

## Known Issues

- Current limitation 1
- Current limitation 2

## Upgrade Notes

### From v{PREVIOUS_VERSION}

1. Download the new version from [Releases](https://github.com/maidsafe/communitas/releases)
2. Install normally (your data will be preserved)
3. The app will automatically migrate any settings

### From Earlier Versions

If upgrading from a version older than {PREVIOUS_VERSION}:
- Backup your data directory first
- Follow the same installation steps

## System Requirements

- macOS 11.0 (Big Sur) or later
- 200MB disk space
- Internet connection for initial setup (optional after)

## Contributors

Thanks to all contributors who made this release possible!

## Checksums

```
SHA256 (Communitas-{VERSION}-universal.dmg) = {CHECKSUM}
```

## Support

- Issues: https://github.com/maidsafe/communitas/issues
- Documentation: https://github.com/maidsafe/communitas/tree/main/docs
```

---

## v0.8.0 Release Notes

# Communitas v0.8.0

_Released: 2026-01-24_

## Highlights

- **Production Release**: Production-ready local-first collaboration platform
- **Post-Quantum Security**: ML-DSA/ML-KEM cryptography protects against future threats
- **Complete Feature Set**: Messaging, Drive, Canvas, Kanban, and Calls

## New Features

### End-to-End Encrypted Messaging
Secure conversations with thread-based organization, reactions, and rich text editing. All messages are encrypted with ChaCha20-Poly1305.

### Virtual Drive System
Private, Public, and Shared storage with CRDT synchronization. Files sync automatically and work offline.

### Collaborative Canvas
Real-time whiteboard with drawing tools, shapes, and text. Multiple users can collaborate simultaneously with conflict-free merging.

### Kanban Project Management
CRDT-based boards with columns, swimlanes, cards, labels, and due dates. Full drag-and-drop support with analytics.

### Voice & Video Calls
WebRTC-based communication with device selection and call history.

### Interactive Onboarding
8-step tour introduces new users to all features with keyboard navigation support.

### Auto-Updates
Seamless background updates via GitHub Releases with version checking.

## Improvements

- 60fps UI performance with optimized rendering
- LRU caching for frequently accessed data
- Debounced operations reduce unnecessary processing
- Full keyboard navigation throughout the app
- Screen reader support with ARIA labels
- High contrast mode for accessibility

## System Requirements

- macOS 11.0 (Big Sur) or later
- 200MB disk space
- Internet connection for initial setup (works offline after)

## Contributors

Thanks to the MaidSafe team and Saorsa Labs for making this release possible!

## Support

- Issues: https://github.com/maidsafe/communitas/issues
- Documentation: https://github.com/maidsafe/communitas/tree/main/docs
