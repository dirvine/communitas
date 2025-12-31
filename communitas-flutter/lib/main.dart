import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'src/app.dart';

/// Compile-time flag for demo mode (no Rust backend required).
/// Set via: flutter run --dart-define=DEMO_MODE=true
const bool kDemoMode = bool.fromEnvironment('DEMO_MODE', defaultValue: false);

/// Compile-time flag for headless mode (CLI/TUI future use).
/// Set via: flutter run --dart-define=HEADLESS=true
const bool kHeadlessMode = bool.fromEnvironment('HEADLESS', defaultValue: false);

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  if (kHeadlessMode) {
    // Future: Run headless for CLI/TUI mode
    // await runHeadless();
    return;
  }

  runApp(
    const ProviderScope(
      child: CommunitasApp(),
    ),
  );
}
