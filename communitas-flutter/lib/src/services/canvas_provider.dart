import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'bridge_provider.dart';
import 'canvas_client.dart';

// Re-export canvas client types for convenience
export 'canvas_client.dart'
    show CanvasSceneUpdate, CanvasConnectionState, CanvasException;

// ============================================================
// Canvas Configuration
// ============================================================

/// Provider for the canvas WebSocket URL.
final canvasWsUrlProvider = StateProvider<String>((ref) {
  const envUrl = String.fromEnvironment('CANVAS_WS_URL', defaultValue: '');
  if (envUrl.isNotEmpty) return envUrl;

  // Derive from bridge URL
  final bridgeUrl = ref.watch(bridgeUrlProvider);
  return bridgeUrl
      .replaceFirst('http://', 'ws://')
      .replaceFirst('https://', 'wss://');
});

/// Provider for the canvas HTTP URL.
final canvasHttpUrlProvider = StateProvider<String>((ref) {
  const envUrl = String.fromEnvironment('CANVAS_HTTP_URL', defaultValue: '');
  if (envUrl.isNotEmpty) return envUrl;

  return ref.watch(bridgeUrlProvider);
});

// ============================================================
// Canvas Client Provider
// ============================================================

/// Provider for the canvas client singleton instance.
final canvasClientProvider = Provider<CanvasClient>((ref) {
  final wsUrl = ref.watch(canvasWsUrlProvider);
  final httpUrl = ref.watch(canvasHttpUrlProvider);
  final client = CanvasClient(wsUrl: wsUrl, httpUrl: httpUrl);

  ref.onDispose(() {
    client.dispose();
  });

  return client;
});

// ============================================================
// Connection State Providers
// ============================================================

/// Provider for the canvas connection state stream.
final canvasConnectionProvider = StreamProvider<CanvasConnectionState>((ref) {
  final client = ref.watch(canvasClientProvider);
  return client.connectionStateStream;
});

/// Provider for the current canvas connection state.
final canvasConnectionStateProvider = Provider<CanvasConnectionState>((ref) {
  final client = ref.watch(canvasClientProvider);
  return client.connectionState;
});

/// Provider to check if the canvas is currently connected.
final isCanvasConnectedProvider = Provider<bool>((ref) {
  final client = ref.watch(canvasClientProvider);
  return client.isConnected;
});

// ============================================================
// Scene State Providers
// ============================================================

/// Provider for canvas scene updates.
final canvasSceneStreamProvider = StreamProvider<CanvasSceneUpdate>((ref) {
  final client = ref.watch(canvasClientProvider);
  return client.sceneUpdates;
});

/// Provider for canvas scene updates for a specific session.
///
/// Filters scene updates to only those matching the session ID.
final canvasSceneProvider =
    StreamProvider.family<CanvasSceneUpdate, String>((ref, sessionId) {
  final client = ref.watch(canvasClientProvider);
  return client.sceneUpdates.where((update) => update.sessionId == sessionId);
});

// ============================================================
// Canvas Element Types for Widget Painting
// ============================================================

/// Represents a generic canvas element for rendering.
@immutable
class CanvasElement {
  final String id;
  final String type;
  final double x;
  final double y;
  final double width;
  final double height;
  final double rotation;
  final double opacity;
  final Map<String, dynamic>? metadata;

  const CanvasElement({
    required this.id,
    required this.type,
    required this.x,
    required this.y,
    required this.width,
    required this.height,
    this.rotation = 0.0,
    this.opacity = 1.0,
    this.metadata,
  });

  factory CanvasElement.fromJson(Map<String, dynamic> json) {
    return CanvasElement(
      id: json['id'] as String? ?? '',
      type: json['type'] as String? ?? 'unknown',
      x: (json['x'] as num?)?.toDouble() ?? 0.0,
      y: (json['y'] as num?)?.toDouble() ?? 0.0,
      width: (json['width'] as num?)?.toDouble() ?? 100.0,
      height: (json['height'] as num?)?.toDouble() ?? 100.0,
      rotation: (json['rotation'] as num?)?.toDouble() ?? 0.0,
      opacity: (json['opacity'] as num?)?.toDouble() ?? 1.0,
      metadata: json['metadata'] as Map<String, dynamic>?,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'type': type,
      'x': x,
      'y': y,
      'width': width,
      'height': height,
      'rotation': rotation,
      'opacity': opacity,
      if (metadata != null) 'metadata': metadata,
    };
  }
}

/// Represents a viewport transform for pan/zoom.
@immutable
class CanvasViewport {
  final double offsetX;
  final double offsetY;
  final double scale;

  const CanvasViewport({
    this.offsetX = 0.0,
    this.offsetY = 0.0,
    this.scale = 1.0,
  });

  CanvasViewport copyWith({
    double? offsetX,
    double? offsetY,
    double? scale,
  }) {
    return CanvasViewport(
      offsetX: offsetX ?? this.offsetX,
      offsetY: offsetY ?? this.offsetY,
      scale: scale ?? this.scale,
    );
  }
}

/// Canvas session state for widget rendering.
@immutable
class CanvasState {
  final String sessionId;
  final List<CanvasElement> elements;
  final CanvasViewport viewport;
  final String? selectedElementId;
  final bool isConnected;
  final String? errorMessage;

  const CanvasState({
    required this.sessionId,
    this.elements = const [],
    this.viewport = const CanvasViewport(),
    this.selectedElementId,
    this.isConnected = false,
    this.errorMessage,
  });

  CanvasState copyWith({
    String? sessionId,
    List<CanvasElement>? elements,
    CanvasViewport? viewport,
    String? selectedElementId,
    bool? isConnected,
    String? errorMessage,
  }) {
    return CanvasState(
      sessionId: sessionId ?? this.sessionId,
      elements: elements ?? this.elements,
      viewport: viewport ?? this.viewport,
      selectedElementId: selectedElementId ?? this.selectedElementId,
      isConnected: isConnected ?? this.isConnected,
      errorMessage: errorMessage ?? this.errorMessage,
    );
  }

  CanvasState clearSelection() {
    return CanvasState(
      sessionId: sessionId,
      elements: elements,
      viewport: viewport,
      selectedElementId: null,
      isConnected: isConnected,
      errorMessage: errorMessage,
    );
  }
}

// ============================================================
// Session-Based Canvas State Controller
// ============================================================

/// Controller for managing per-session canvas state and interactions.
class SessionCanvasController extends StateNotifier<CanvasState> {
  final Ref _ref;
  final String _sessionId;
  StreamSubscription<CanvasSceneUpdate>? _sceneSubscription;
  StreamSubscription<CanvasConnectionState>? _connectionSubscription;

  static const double _minScale = 0.25;
  static const double _maxScale = 4.0;

  SessionCanvasController(this._ref, this._sessionId)
      : super(CanvasState(sessionId: _sessionId)) {
    _initialize();
  }

  void _initialize() {
    final client = _ref.read(canvasClientProvider);

    // Listen to connection changes
    _connectionSubscription =
        client.connectionStateStream.listen((connectionState) {
      state = state.copyWith(
        isConnected: connectionState == CanvasConnectionState.connected,
      );
    });

    // Listen to scene updates and filter for our session
    _sceneSubscription = client.sceneUpdates.listen((update) {
      if (update.sessionId == _sessionId) {
        _handleSceneUpdate(update);
      }
    });

    // Subscribe to this session and connect
    client.subscribe(_sessionId);
    _connectToSession();
  }

  Future<void> _connectToSession() async {
    final client = _ref.read(canvasClientProvider);
    try {
      if (!client.isConnected) {
        await client.connect();
      }
      state = state.copyWith(isConnected: client.isConnected, errorMessage: null);

      // Fetch initial scene
      await _fetchScene();
    } catch (e) {
      state = state.copyWith(
        isConnected: false,
        errorMessage: 'Failed to connect: $e',
      );
    }
  }

  Future<void> _fetchScene() async {
    final client = _ref.read(canvasClientProvider);
    if (!client.isConnected) {
      debugPrint('Cannot fetch scene: not connected');
      return;
    }

    try {
      final sceneJson = await client.getScene(_sessionId);
      // Parse the JSON and update elements
      final sceneData = jsonDecode(sceneJson) as Map<String, dynamic>?;
      if (sceneData != null) {
        final elements = sceneData['elements'] as List<dynamic>?;
        if (elements != null) {
          state = state.copyWith(
            elements: elements
                .map((e) => CanvasElement.fromJson(e as Map<String, dynamic>))
                .toList(),
          );
        }
      }
    } catch (e) {
      debugPrint('Failed to fetch scene: $e');
    }
  }

  void _handleSceneUpdate(CanvasSceneUpdate update) {
    if (update.sceneData != null) {
      final elements = update.sceneData!['elements'] as List<dynamic>?;
      if (elements != null) {
        state = state.copyWith(
          elements: elements
              .map((e) => CanvasElement.fromJson(e as Map<String, dynamic>))
              .toList(),
        );
      }
    }
  }

  @override
  void dispose() {
    _sceneSubscription?.cancel();
    _connectionSubscription?.cancel();
    final client = _ref.read(canvasClientProvider);
    client.unsubscribe(_sessionId);
    super.dispose();
  }

  /// Handle tap interaction, returns element ID if hit.
  Future<String?> handleTap(double x, double y) async {
    final canvasX = _screenToCanvasX(x);
    final canvasY = _screenToCanvasY(y);

    final elementId = _findElementAt(canvasX, canvasY);
    state = state.copyWith(selectedElementId: elementId);

    return elementId;
  }

  void handlePanUpdate(double deltaX, double deltaY) {
    final newViewport = state.viewport.copyWith(
      offsetX: state.viewport.offsetX + deltaX,
      offsetY: state.viewport.offsetY + deltaY,
    );
    state = state.copyWith(viewport: newViewport);
  }

  void handleScale(double scale, double focalX, double focalY) {
    final newScale =
        (state.viewport.scale * scale).clamp(_minScale, _maxScale);

    final newViewport = state.viewport.copyWith(scale: newScale);
    state = state.copyWith(viewport: newViewport);
  }

  /// Add an element to the canvas.
  Future<CanvasElement?> addElement(CanvasElement element) async {
    final client = _ref.read(canvasClientProvider);
    if (!client.isConnected) {
      debugPrint('Cannot add element: not connected');
      return null;
    }

    try {
      final result = await client.addElement(_sessionId, element.toJson());
      final newElement = CanvasElement.fromJson(result);
      state = state.copyWith(elements: [...state.elements, newElement]);
      return newElement;
    } catch (e) {
      debugPrint('Failed to add element: $e');
      return null;
    }
  }

  /// Remove an element from the canvas.
  Future<bool> removeElement(String elementId) async {
    final client = _ref.read(canvasClientProvider);
    if (!client.isConnected) {
      debugPrint('Cannot remove element: not connected');
      return false;
    }

    try {
      await client.removeElement(_sessionId, elementId);
      state = state.copyWith(
        elements: state.elements.where((e) => e.id != elementId).toList(),
        selectedElementId:
            state.selectedElementId == elementId ? null : state.selectedElementId,
      );
      return true;
    } catch (e) {
      debugPrint('Failed to remove element: $e');
      return false;
    }
  }

  /// Update an element on the canvas.
  Future<bool> updateElement(
      String elementId, Map<String, dynamic> updates) async {
    final client = _ref.read(canvasClientProvider);
    if (!client.isConnected) {
      debugPrint('Cannot update element: not connected');
      return false;
    }

    try {
      await client.updateElement(_sessionId, elementId, updates);

      final index = state.elements.indexWhere((e) => e.id == elementId);
      if (index != -1) {
        final updated = CanvasElement.fromJson({
          ...state.elements[index].toJson(),
          ...updates,
        });
        final newElements = [...state.elements];
        newElements[index] = updated;
        state = state.copyWith(elements: newElements);
      }
      return true;
    } catch (e) {
      debugPrint('Failed to update element: $e');
      return false;
    }
  }

  /// Clear all elements from the canvas.
  Future<bool> clearCanvas() async {
    final client = _ref.read(canvasClientProvider);
    if (!client.isConnected) {
      debugPrint('Cannot clear canvas: not connected');
      return false;
    }

    try {
      await client.clearCanvas(_sessionId);
      state = state.copyWith(elements: [], selectedElementId: null);
      return true;
    } catch (e) {
      debugPrint('Failed to clear canvas: $e');
      return false;
    }
  }

  void resetViewport() {
    state = state.copyWith(viewport: const CanvasViewport());
  }

  void clearSelection() {
    state = state.clearSelection();
  }

  double _screenToCanvasX(double screenX) {
    return (screenX - state.viewport.offsetX) / state.viewport.scale;
  }

  double _screenToCanvasY(double screenY) {
    return (screenY - state.viewport.offsetY) / state.viewport.scale;
  }

  String? _findElementAt(double x, double y) {
    for (var i = state.elements.length - 1; i >= 0; i--) {
      final element = state.elements[i];
      if (x >= element.x &&
          x <= element.x + element.width &&
          y >= element.y &&
          y <= element.y + element.height) {
        return element.id;
      }
    }
    return null;
  }
}

/// Family provider for per-session canvas state controllers.
final canvasControllerProvider =
    StateNotifierProvider.family<SessionCanvasController, CanvasState, String>(
        (ref, sessionId) {
  return SessionCanvasController(ref, sessionId);
});

/// Provider for the selected element ID in a session.
final selectedElementProvider =
    Provider.family<String?, String>((ref, sessionId) {
  final state = ref.watch(canvasControllerProvider(sessionId));
  return state.selectedElementId;
});

// ============================================================
// Canvas Operations Controller (non-session-scoped)
// ============================================================

/// Controller for canvas operations without session scope.
class CanvasOperationsController extends StateNotifier<AsyncValue<void>> {
  final Ref _ref;

  CanvasOperationsController(this._ref) : super(const AsyncValue.data(null));

  CanvasClient get _client => _ref.read(canvasClientProvider);

  /// Connect to the canvas server.
  Future<bool> connect() async {
    state = const AsyncValue.loading();
    try {
      await _client.connect();
      state = const AsyncValue.data(null);
      return true;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return false;
    }
  }

  /// Disconnect from the canvas server.
  Future<void> disconnect() async {
    state = const AsyncValue.loading();
    try {
      await _client.disconnect();
      state = const AsyncValue.data(null);
    } catch (e, st) {
      state = AsyncValue.error(e, st);
    }
  }

  /// Create a new canvas session.
  Future<String?> createSession({String? sessionId}) async {
    state = const AsyncValue.loading();
    try {
      final id = await _client.createSession(sessionId: sessionId);
      state = const AsyncValue.data(null);
      return id;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return null;
    }
  }
}

/// Provider for canvas operations controller.
final canvasOperationsProvider =
    StateNotifierProvider<CanvasOperationsController, AsyncValue<void>>((ref) {
  return CanvasOperationsController(ref);
});
