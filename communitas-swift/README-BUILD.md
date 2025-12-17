# Building Communitas for iOS/macOS

This directory contains the Swift client implementation.

## Prerequisites
1. **Rust**: Install via `rustup`
2. **Xcode**: Install via App Store
3. **Targets**: Add iOS targets to Rust:
   ```bash
   rustup target add aarch64-apple-ios aarch64-apple-ios-sim aarch64-apple-darwin
   ```

## Build Instructions

### 1. Build the Rust Bindings
Run the Makefile to build the static library and generate the Swift bindings.

```bash
make
```

This will:
- Build `communitas-bindings` (Rust)
- Generate `communitas_bindings.swift` via UniFFI
- Place artifacts in `CommunitasKit/Sources/CommunitasKit`

### 2. Open in Xcode
The `CommunitasKit` folder is a Swift Package. You can open `CommunitasKit/Package.swift` directly in Xcode to develop the logic.

To run the App:
1. Open Xcode.
2. Select **File > New > Project...**
3. Choose **App**.
4. Name it `CommunitasApp`.
5. Close the new project window.
6. Drag the `CommunitasKit` folder into your Xcode project navigator.
7. In your App Target -> **General** -> **Frameworks, Libraries, and Embedded Content**, add `CommunitasKit`.
8. Copy the code from `CommunitasApp/Sources/` into your Xcode project's files.

## Troubleshooting

**"Library not found"**
Ensure the `libcommunitas_bindings.a` exists in `target/debug` or `target/release`. The `Package.swift` expects it in `../../target/debug` by default.

**"Missing symbols"**
If you see linker errors, ensure `communitas-core` dependencies are compatible with iOS (no `openssl` linking issues, etc.). We use `rustls` so it should be fine.
