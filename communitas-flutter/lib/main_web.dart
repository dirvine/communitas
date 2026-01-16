// Web platform initialization
// This file is used when building for web - no dart:ffi or dart:io available.

import 'package:flutter/material.dart';

/// Initialize web platform features.
/// On web, we run in demo mode without Rust FFI.
Future<void> initializePlatform() async {
  debugPrint('Web platform detected - Rust FFI not available');
  debugPrint('Running in demo mode with mock data');
  // No-op on web - all features use demo/mock implementations
}
