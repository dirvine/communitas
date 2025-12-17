# Known Issues - Communitas Swift

## Resolved Issues

### Async FFI Issue (RESOLVED 2024-11-24)

**Problem**: The UniFFI-generated async functions were hanging or crashing when called from Swift.

**Root Cause**: Tokio async runtime was not available when Swift's async/await tried to execute the Rust futures.

**Solution**: Converted all Rust async functions to synchronous functions using a global Tokio runtime with `block_on` wrappers:

```rust
static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime")
});

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    RUNTIME.block_on(future)
}

// All methods now use sync wrappers
pub fn get_profile(&self) -> SwiftUserProfile {
    block_on(async {
        let ctx = self.inner.read().await;
        // ...
    })
}
```

**Status**: All FFI functions now work correctly:
- Client creation
- Profile retrieval
- Entity creation/listing
- Message operations
- Networking status

## Current Status

All core functionality is working:
- Rust bindings compile for macOS and iOS (device + simulator)
- Swift Package Manager integration working
- XCFramework created for distribution
- Xcode project with iOS and macOS targets
- Full integration tests passing

## Build Instructions

### Quick Start
```bash
cd communitas-swift
make              # Build Rust and generate Swift bindings
swift build       # Build Swift package
```

### Full Build (Including iOS)
```bash
# Build for all platforms
cargo build -p communitas-bindings --target aarch64-apple-darwin --release
cargo build -p communitas-bindings --target aarch64-apple-ios --release
cargo build -p communitas-bindings --target aarch64-apple-ios-sim --release

# Regenerate Swift bindings
make generate-bindings

# Build Swift package
swift build

# Build Xcode project
xcodebuild -project Communitas.xcodeproj -scheme Communitas-macOS build
xcodebuild -project Communitas.xcodeproj -scheme Communitas-iOS build
```

### Running Tests
```bash
# Sync FFI test (standalone)
swiftc -parse-as-library \
    -I CommunitasKit/Sources/communitas_bindingsFFI \
    -L ../target/aarch64-apple-darwin/debug \
    -lcommunitas_bindings \
    -Xlinker -rpath -Xlinker ../target/aarch64-apple-darwin/debug \
    CommunitasKit/Sources/CommunitasKit/communitas_bindings.swift \
    test_full.swift -o test_full_sync
./test_full_sync
```

---

Last updated: 2024-11-24
