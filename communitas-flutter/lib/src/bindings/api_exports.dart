// Conditional exports for FFI bindings.
// Web platform uses stub implementations that throw UnsupportedError.
// Native platforms use real flutter_rust_bridge bindings.

export 'flutter_api_stub.dart'
    if (dart.library.io) 'flutter_api.dart';
