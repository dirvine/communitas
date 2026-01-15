import 'dart:async';
import 'dart:convert';
import 'dart:math';

import 'package:flutter/foundation.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

/// Represents a scene update from the Canvas server.
class CanvasSceneUpdate {
  /// The session ID this update belongs to.
  final String sessionId;

  /// Number of elements in the scene.
  final int elementCount;

  /// Timestamp of the update.
  final DateTime timestamp;

  /// Raw scene data (optional).
  final Map<String, dynamic>? sceneData;

  CanvasSceneUpdate({
    required this.sessionId,
    required this.elementCount,
    required this.timestamp,
    this.sceneData,
  });

  factory CanvasSceneUpdate.fromJson(Map<String, dynamic> json) {
    final sceneData = json['scene'] as Map<String, dynamic>?;
    final elements = sceneData?['elements'] as List<dynamic>? ?? [];

    return CanvasSceneUpdate(
      sessionId: json['session_id'] as String? ?? '',
      elementCount: elements.length,
      timestamp: DateTime.now(),
      sceneData: sceneData,
    );
  }

  @override
  String toString() =>
      'CanvasSceneUpdate(sessionId: $sessionId, elementCount: $elementCount, timestamp: $timestamp)';
}

/// Exception thrown when a Canvas operation fails.
class CanvasException implements Exception {
  final String message;
  final int? code;
  final dynamic data;

  CanvasException(this.message, {this.code, this.data});

  @override
  String toString() => 'CanvasException: $message (code: $code)';
}

/// Connection state for the Canvas WebSocket client.
enum CanvasConnectionState {
  disconnected,
  connecting,
  connected,
  reconnecting,
}

/// WebSocket client for communicating with the Communitas Canvas server.
///
/// The Canvas server provides real-time collaborative canvas/whiteboard
/// functionality via WebSocket and MCP (Model Context Protocol) JSON-RPC 2.0.
class CanvasClient {
  /// WebSocket URL for the Canvas server.
  final String wsUrl;

  /// HTTP URL for REST API calls.
  final String httpUrl;

  WebSocketChannel? _channel;
  StreamSubscription<dynamic>? _subscription;

  /// Current connection state.
  CanvasConnectionState _state = CanvasConnectionState.disconnected;

  /// Stream controller for connection state changes.
  final _stateController =
      StreamController<CanvasConnectionState>.broadcast();

  /// Stream controller for scene updates.
  final _sceneUpdateController =
      StreamController<CanvasSceneUpdate>.broadcast();

  /// Pending RPC requests awaiting response.
  final Map<int, Completer<Map<String, dynamic>>> _pendingRequests = {};

  /// Next JSON-RPC request ID.
  int _nextRequestId = 1;

  /// Set of subscribed session IDs.
  final Set<String> _subscribedSessions = {};

  /// Reconnection parameters.
  static const int _maxReconnectAttempts = 10;
  static const Duration _initialReconnectDelay = Duration(seconds: 1);
  static const Duration _maxReconnectDelay = Duration(seconds: 30);

  int _reconnectAttempts = 0;
  Timer? _reconnectTimer;
  bool _shouldReconnect = true;

  /// Creates a new Canvas client.
  ///
  /// [wsUrl] - WebSocket URL (default: ws://localhost:9473/ws)
  /// [httpUrl] - HTTP URL for REST calls (default: http://localhost:9473)
  CanvasClient({
    this.wsUrl = 'ws://localhost:9473/ws',
    this.httpUrl = 'http://localhost:9473',
  });

  // ============================================================
  // Connection Management
  // ============================================================

  /// Whether the client is currently connected.
  bool get isConnected => _state == CanvasConnectionState.connected;

  /// Current connection state.
  CanvasConnectionState get connectionState => _state;

  /// Stream of connection state changes.
  Stream<CanvasConnectionState> get connectionStateStream =>
      _stateController.stream;

  /// Stream of scene updates from subscribed sessions.
  Stream<CanvasSceneUpdate> get sceneUpdates => _sceneUpdateController.stream;

  /// Connect to the Canvas WebSocket server.
  Future<void> connect() async {
    if (_state == CanvasConnectionState.connected ||
        _state == CanvasConnectionState.connecting) {
      return;
    }

    _shouldReconnect = true;
    await _doConnect();
  }

  Future<void> _doConnect() async {
    _updateState(CanvasConnectionState.connecting);

    try {
      _channel = WebSocketChannel.connect(Uri.parse(wsUrl));

      // Wait for connection to establish
      await _channel!.ready;

      _subscription = _channel!.stream.listen(
        _onMessage,
        onError: _onError,
        onDone: _onDone,
        cancelOnError: false,
      );

      _reconnectAttempts = 0;
      _updateState(CanvasConnectionState.connected);

      // Re-subscribe to previously subscribed sessions
      for (final sessionId in _subscribedSessions.toList()) {
        await _sendSubscribe(sessionId);
      }
    } catch (e) {
      _updateState(CanvasConnectionState.disconnected);
      _scheduleReconnect();
      rethrow;
    }
  }

  /// Disconnect from the Canvas server.
  Future<void> disconnect() async {
    _shouldReconnect = false;
    _cancelReconnect();
    await _closeConnection();
    _updateState(CanvasConnectionState.disconnected);
  }

  Future<void> _closeConnection() async {
    await _subscription?.cancel();
    _subscription = null;

    try {
      await _channel?.sink.close();
    } catch (e) {
      debugPrint('Canvas client close error: $e');
    }
    _channel = null;

    // Fail all pending requests
    for (final completer in _pendingRequests.values) {
      if (!completer.isCompleted) {
        completer.completeError(
            CanvasException('Connection closed', code: -32000));
      }
    }
    _pendingRequests.clear();
  }

  void _updateState(CanvasConnectionState newState) {
    if (_state != newState) {
      _state = newState;
      _stateController.add(newState);
    }
  }

  // ============================================================
  // Auto-Reconnect with Exponential Backoff
  // ============================================================

  void _scheduleReconnect() {
    if (!_shouldReconnect) return;
    if (_reconnectAttempts >= _maxReconnectAttempts) {
      _updateState(CanvasConnectionState.disconnected);
      return;
    }

    _updateState(CanvasConnectionState.reconnecting);
    _reconnectAttempts++;

    // Calculate delay with exponential backoff
    final delayMs = _initialReconnectDelay.inMilliseconds *
        pow(2, _reconnectAttempts - 1).toInt();
    final cappedDelayMs = min(delayMs, _maxReconnectDelay.inMilliseconds);
    final delay = Duration(milliseconds: cappedDelayMs);

    _reconnectTimer = Timer(delay, () async {
      try {
        await _doConnect();
      } catch (e) {
        debugPrint('Canvas reconnect attempt $_reconnectAttempts failed: $e');
        // Reconnect will be scheduled again via _onDone or _onError
      }
    });
  }

  void _cancelReconnect() {
    _reconnectTimer?.cancel();
    _reconnectTimer = null;
  }

  // ============================================================
  // WebSocket Event Handlers
  // ============================================================

  void _onMessage(dynamic message) {
    try {
      final data = jsonDecode(message as String) as Map<String, dynamic>;
      _handleMessage(data);
    } catch (e) {
      debugPrint('Canvas client message parse error: $e');
    }
  }

  void _handleMessage(Map<String, dynamic> data) {
    // Check if it is a JSON-RPC response
    if (data.containsKey('id') && data['id'] != null) {
      _handleRpcResponse(data);
      return;
    }

    // Check if it is a notification (no id, has method)
    if (data.containsKey('method')) {
      _handleNotification(data);
      return;
    }

    // Check if it is a scene_update event
    if (data.containsKey('type') && data['type'] == 'scene_update') {
      _handleSceneUpdate(data);
      return;
    }

    // Handle direct scene updates (legacy format)
    if (data.containsKey('session_id') && data.containsKey('scene')) {
      _handleSceneUpdate(data);
      return;
    }
  }

  void _handleRpcResponse(Map<String, dynamic> data) {
    final id = data['id'] as int?;
    if (id == null) return;

    final completer = _pendingRequests.remove(id);
    if (completer == null || completer.isCompleted) return;

    if (data.containsKey('error')) {
      final error = data['error'] as Map<String, dynamic>;
      completer.completeError(CanvasException(
        error['message'] as String? ?? 'Unknown error',
        code: error['code'] as int?,
        data: error['data'],
      ));
    } else {
      completer.complete(data['result'] as Map<String, dynamic>? ?? {});
    }
  }

  void _handleNotification(Map<String, dynamic> data) {
    final method = data['method'] as String;
    final params = data['params'] as Map<String, dynamic>? ?? {};

    switch (method) {
      case 'scene_updated':
      case 'canvas/scene_updated':
        _handleSceneUpdate(params);
        break;
      // Add other notification handlers here
    }
  }

  void _handleSceneUpdate(Map<String, dynamic> data) {
    try {
      final update = CanvasSceneUpdate.fromJson(data);
      _sceneUpdateController.add(update);
    } catch (e) {
      debugPrint('Canvas client scene update parse error: $e');
    }
  }

  void _onError(dynamic error) {
    debugPrint('Canvas client connection error: $error');
    _closeConnection();
    _scheduleReconnect();
  }

  void _onDone() {
    // Connection closed
    if (_state != CanvasConnectionState.disconnected) {
      _closeConnection();
      _scheduleReconnect();
    }
  }

  // ============================================================
  // MCP Protocol - JSON-RPC 2.0 Tool Calls
  // ============================================================

  /// Call an MCP tool via JSON-RPC 2.0.
  ///
  /// [name] - The tool name (e.g., 'canvas/add_element')
  /// [args] - Tool arguments as a map
  ///
  /// Returns the tool result on success.
  /// Throws [CanvasException] on error or timeout.
  Future<Map<String, dynamic>> callTool(
    String name,
    Map<String, dynamic> args, {
    Duration timeout = const Duration(seconds: 30),
  }) async {
    if (!isConnected) {
      throw CanvasException('Not connected', code: -32003);
    }

    final id = _nextRequestId++;
    final request = {
      'jsonrpc': '2.0',
      'id': id,
      'method': 'tools/call',
      'params': {
        'name': name,
        'arguments': args,
      },
    };

    final completer = Completer<Map<String, dynamic>>();
    _pendingRequests[id] = completer;

    try {
      _channel!.sink.add(jsonEncode(request));

      return await completer.future.timeout(
        timeout,
        onTimeout: () {
          _pendingRequests.remove(id);
          throw CanvasException('Request timeout', code: -32001);
        },
      );
    } catch (e) {
      _pendingRequests.remove(id);
      rethrow;
    }
  }

  /// Send a raw JSON-RPC request.
  Future<Map<String, dynamic>> sendRequest(
    String method,
    Map<String, dynamic>? params, {
    Duration timeout = const Duration(seconds: 30),
  }) async {
    if (!isConnected) {
      throw CanvasException('Not connected', code: -32003);
    }

    final id = _nextRequestId++;
    final request = {
      'jsonrpc': '2.0',
      'id': id,
      'method': method,
      if (params != null) 'params': params,
    };

    final completer = Completer<Map<String, dynamic>>();
    _pendingRequests[id] = completer;

    try {
      _channel!.sink.add(jsonEncode(request));

      return await completer.future.timeout(
        timeout,
        onTimeout: () {
          _pendingRequests.remove(id);
          throw CanvasException('Request timeout', code: -32001);
        },
      );
    } catch (e) {
      _pendingRequests.remove(id);
      rethrow;
    }
  }

  // ============================================================
  // Scene Management
  // ============================================================

  /// Get the current scene for a session.
  ///
  /// Returns the scene as a JSON string.
  Future<String> getScene(String sessionId) async {
    final result = await callTool('canvas/get_scene', {
      'session_id': sessionId,
    });

    final scene = result['scene'];
    if (scene is String) {
      return scene;
    } else if (scene is Map) {
      return jsonEncode(scene);
    }
    return jsonEncode(result);
  }

  /// Create a new canvas session.
  Future<String> createSession({String? sessionId}) async {
    final result = await callTool('canvas/create_session', {
      if (sessionId != null) 'session_id': sessionId,
    });

    return result['session_id'] as String? ?? '';
  }

  /// Add an element to the canvas.
  Future<Map<String, dynamic>> addElement(
    String sessionId,
    Map<String, dynamic> element,
  ) async {
    return callTool('canvas/add_element', {
      'session_id': sessionId,
      'element': element,
    });
  }

  /// Update an existing element on the canvas.
  Future<Map<String, dynamic>> updateElement(
    String sessionId,
    String elementId,
    Map<String, dynamic> updates,
  ) async {
    return callTool('canvas/update_element', {
      'session_id': sessionId,
      'element_id': elementId,
      'updates': updates,
    });
  }

  /// Remove an element from the canvas.
  Future<Map<String, dynamic>> removeElement(
    String sessionId,
    String elementId,
  ) async {
    return callTool('canvas/remove_element', {
      'session_id': sessionId,
      'element_id': elementId,
    });
  }

  /// Clear all elements from a canvas session.
  Future<Map<String, dynamic>> clearCanvas(String sessionId) async {
    return callTool('canvas/clear', {
      'session_id': sessionId,
    });
  }

  // ============================================================
  // Session Subscription
  // ============================================================

  /// Subscribe to scene updates for a session.
  ///
  /// The client will receive [sceneUpdates] for this session.
  void subscribe(String sessionId) {
    _subscribedSessions.add(sessionId);
    if (isConnected) {
      _sendSubscribe(sessionId);
    }
  }

  /// Unsubscribe from scene updates for a session.
  void unsubscribe(String sessionId) {
    _subscribedSessions.remove(sessionId);
    if (isConnected) {
      _sendUnsubscribe(sessionId);
    }
  }

  Future<void> _sendSubscribe(String sessionId) async {
    try {
      await sendRequest('canvas/subscribe', {
        'session_id': sessionId,
      });
    } catch (e) {
      // Subscription may not be supported on all servers - log but continue
      debugPrint('Canvas subscribe error (may be unsupported): $e');
    }
  }

  Future<void> _sendUnsubscribe(String sessionId) async {
    try {
      await sendRequest('canvas/unsubscribe', {
        'session_id': sessionId,
      });
    } catch (e) {
      debugPrint('Canvas unsubscribe error: $e');
    }
  }

  /// Get the list of currently subscribed session IDs.
  Set<String> get subscribedSessions => Set.unmodifiable(_subscribedSessions);

  // ============================================================
  // Resource Management
  // ============================================================

  /// Dispose of the client and release all resources.
  void dispose() {
    _shouldReconnect = false;
    _cancelReconnect();
    _closeConnection();
    _stateController.close();
    _sceneUpdateController.close();
  }
}
