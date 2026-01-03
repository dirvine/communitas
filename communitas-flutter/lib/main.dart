import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'src/app.dart';

// Conditional imports for native-only features
// Web platform doesn't support dart:ffi or dart:io
import 'main_native.dart' if (dart.library.html) 'main_web.dart' as platform;

/// Compile-time flag for demo mode (no Rust backend required).
/// Set via: flutter run --dart-define=DEMO_MODE=true
/// Web builds use bridge mode by default (connect to bridge server).
/// Set DEMO_MODE=true to use demo mode on web.
const bool kDemoMode = bool.fromEnvironment('DEMO_MODE', defaultValue: false);

/// Compile-time flag for headless mode (CLI/TUI future use).
/// Set via: flutter run --dart-define=HEADLESS=true
const bool kHeadlessMode =
    bool.fromEnvironment('HEADLESS', defaultValue: false);

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  if (kHeadlessMode) {
    // Future: Run headless for CLI/TUI mode
    // await runHeadless();
    return;
  }

  // Initialize platform-specific features (Rust bridge on native, noop on web)
  if (!kDemoMode) {
    await platform.initializePlatform();
  } else {
    debugPrint('Running in demo mode (web or DEMO_MODE=true)');
  }

  runApp(
    const ProviderScope(
      child: CommunitasApp(),
    ),
  );
}
