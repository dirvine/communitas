// Native platform initialization (iOS, Android, macOS, Windows, Linux)
// This file uses dart:io which is only available on native platforms.

import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'src/bindings/frb_generated.dart';

/// Initialize native platform features (Rust bridge).
Future<void> initializePlatform() async {
  try {
    final lib = _loadNativeLibrary();
    if (lib != null) {
      await RustLib.init(externalLibrary: lib);
      debugPrint('Rust bridge initialized successfully');
    } else {
      debugPrint('Could not find native library, running in demo mode');
    }
  } catch (e) {
    debugPrint('Failed to initialize Rust bridge: $e');
    // Continue in demo mode if Rust bridge fails
  }
}

/// Load the native Rust library for the current platform.
ExternalLibrary? _loadNativeLibrary() {
  if (Platform.isMacOS) {
    // For macOS development, load the dylib directly from the workspace
    // In production, this would be bundled with the app
    final home = Platform.environment['HOME'] ?? '/Users';
    final dylibPaths = [
      // Absolute development path
      '$home/Desktop/Devel/projects/communitas/target/release/libcommunitas_bindings.dylib',
      // Alternative via symlink
      '$home/Desktop/Devel/projects/communitas/communitas-bindings/target/release/libcommunitas_bindings.dylib',
    ];

    for (final path in dylibPaths) {
      final file = File(path);
      if (file.existsSync()) {
        debugPrint('Loading Rust library from: $path');
        try {
          return ExternalLibrary.open(path);
        } catch (e) {
          debugPrint('Failed to load from $path: $e');
        }
      } else {
        debugPrint('Library not found at: $path');
      }
    }
  }
  return null;
}
