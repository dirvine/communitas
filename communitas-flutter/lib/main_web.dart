// Web platform initialization
// This file is used when building for web - no dart:ffi or dart:io available.

import 'package:flutter/material.dart';

/// Initialize web platform features.
/// Web is not fully supported - FFI backend required for core functionality.
Future<void> initializePlatform() async {
  debugPrint('Web platform detected - Rust FFI not available');
  debugPrint('Native app required for full functionality');
  // No-op on web - features will show appropriate errors when backend unavailable
}
