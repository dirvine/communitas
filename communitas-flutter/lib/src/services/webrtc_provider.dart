import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_webrtc/flutter_webrtc.dart' as webrtc;

import 'canvas_provider.dart';

// ============================================================
// WebRTC Configuration
// ============================================================

/// Default ICE servers for WebRTC connections
const List<Map<String, dynamic>> defaultIceServers = [
  {'urls': 'stun:stun.l.google.com:19302'},
  {'urls': 'stun:stun1.l.google.com:19302'},
];

/// WebRTC configuration for peer connections
final Map<String, dynamic> webrtcConfiguration = {
  'iceServers': defaultIceServers,
  'sdpSemantics': 'unified-plan',
};

/// Media constraints for video
const Map<String, dynamic> videoConstraints = {
  'audio': true,
  'video': {
    'mandatory': {
      'minWidth': '640',
      'minHeight': '480',
      'minFrameRate': '15',
    },
    'facingMode': 'user',
    'optional': [],
  },
};

// ============================================================
// WebRTC State Types
// ============================================================

/// Connection state for a WebRTC peer
enum WebRtcConnectionState {
  /// Not connected
  disconnected,

  /// Connection is being established
  connecting,

  /// Connected and active
  connected,

  /// Connection failed
  failed,

  /// Connection closed
  closed,
}

/// Represents a video track with its renderer
@immutable
class VideoTrackInfo {
  /// Unique identifier for this track
  final String id;

  /// The WebRTC video track
  final webrtc.MediaStreamTrack track;

  /// The media stream this track belongs to (may be null for remote tracks)
  final webrtc.MediaStream? stream;

  /// Whether this is a local track (camera) or remote track
  final bool isLocal;

  /// Canvas element ID if associated with a canvas element
  final String? canvasElementId;

  const VideoTrackInfo({
    required this.id,
    required this.track,
    this.stream,
    required this.isLocal,
    this.canvasElementId,
  });

  VideoTrackInfo copyWith({
    String? id,
    webrtc.MediaStreamTrack? track,
    webrtc.MediaStream? stream,
    bool? isLocal,
    String? canvasElementId,
  }) {
    return VideoTrackInfo(
      id: id ?? this.id,
      track: track ?? this.track,
      stream: stream ?? this.stream,
      isLocal: isLocal ?? this.isLocal,
      canvasElementId: canvasElementId ?? this.canvasElementId,
    );
  }
}

/// State for a WebRTC session
@immutable
class WebRtcState {
  /// Session identifier
  final String sessionId;

  /// Current connection state
  final WebRtcConnectionState connectionState;

  /// Local video tracks
  final List<VideoTrackInfo> localTracks;

  /// Remote video tracks
  final List<VideoTrackInfo> remoteTracks;

  /// Error message if any
  final String? errorMessage;

  /// Whether local video is enabled
  final bool isVideoEnabled;

  /// Whether local audio is enabled
  final bool isAudioEnabled;

  const WebRtcState({
    required this.sessionId,
    this.connectionState = WebRtcConnectionState.disconnected,
    this.localTracks = const [],
    this.remoteTracks = const [],
    this.errorMessage,
    this.isVideoEnabled = true,
    this.isAudioEnabled = true,
  });

  WebRtcState copyWith({
    String? sessionId,
    WebRtcConnectionState? connectionState,
    List<VideoTrackInfo>? localTracks,
    List<VideoTrackInfo>? remoteTracks,
    String? errorMessage,
    bool? isVideoEnabled,
    bool? isAudioEnabled,
  }) {
    return WebRtcState(
      sessionId: sessionId ?? this.sessionId,
      connectionState: connectionState ?? this.connectionState,
      localTracks: localTracks ?? this.localTracks,
      remoteTracks: remoteTracks ?? this.remoteTracks,
      errorMessage: errorMessage ?? this.errorMessage,
      isVideoEnabled: isVideoEnabled ?? this.isVideoEnabled,
      isAudioEnabled: isAudioEnabled ?? this.isAudioEnabled,
    );
  }

  /// Get all tracks (local + remote)
  List<VideoTrackInfo> get allTracks => [...localTracks, ...remoteTracks];

  /// Get track by ID
  VideoTrackInfo? getTrack(String trackId) {
    try {
      return allTracks.firstWhere((track) => track.id == trackId);
    } on StateError {
      return null;
    }
  }
}

// ============================================================
// WebRTC Session Controller
// ============================================================

/// Controller for managing a WebRTC session
class WebRtcSessionController extends StateNotifier<WebRtcState> {
  final Ref _ref;
  final String _sessionId;

  webrtc.RTCPeerConnection? _peerConnection;
  webrtc.MediaStream? _localStream;
  final Map<String, webrtc.RTCVideoRenderer> _renderers = {};

  WebRtcSessionController(this._ref, this._sessionId)
      : super(WebRtcState(sessionId: _sessionId));

  /// Initialize and start local video
  Future<void> startLocalVideo() async {
    if (state.localTracks.isNotEmpty) return;

    try {
      state = state.copyWith(connectionState: WebRtcConnectionState.connecting);

      // Get user media
      _localStream = await webrtc.navigator.mediaDevices.getUserMedia(videoConstraints);

      final videoTracks = _localStream!.getVideoTracks();
      final localTrackInfos = <VideoTrackInfo>[];

      for (final track in videoTracks) {
        final trackInfo = VideoTrackInfo(
          id: track.id ?? 'local-${DateTime.now().millisecondsSinceEpoch}',
          track: track,
          stream: _localStream!,
          isLocal: true,
        );
        localTrackInfos.add(trackInfo);
      }

      state = state.copyWith(
        localTracks: localTrackInfos,
        connectionState: WebRtcConnectionState.connected,
        errorMessage: null,
      );
    } catch (e) {
      state = state.copyWith(
        connectionState: WebRtcConnectionState.failed,
        errorMessage: 'Failed to start video: $e',
      );
    }
  }

  /// Stop local video
  Future<void> stopLocalVideo() async {
    for (final track in state.localTracks) {
      await track.track.stop();
    }
    await _localStream?.dispose();
    _localStream = null;

    state = state.copyWith(
      localTracks: [],
      connectionState: WebRtcConnectionState.disconnected,
    );
  }

  /// Toggle video on/off
  Future<void> toggleVideo() async {
    final enabled = !state.isVideoEnabled;

    for (final trackInfo in state.localTracks) {
      trackInfo.track.enabled = enabled;
    }

    state = state.copyWith(isVideoEnabled: enabled);
  }

  /// Toggle audio on/off
  Future<void> toggleAudio() async {
    final enabled = !state.isAudioEnabled;

    if (_localStream != null) {
      for (final track in _localStream!.getAudioTracks()) {
        track.enabled = enabled;
      }
    }

    state = state.copyWith(isAudioEnabled: enabled);
  }

  /// Create a peer connection for a call
  Future<void> initializePeerConnection() async {
    try {
      _peerConnection = await webrtc.createPeerConnection(webrtcConfiguration);

      _peerConnection!.onIceCandidate = (candidate) {
        // Send ICE candidate via signaling
        _sendSignalingMessage({
          'type': 'ice-candidate',
          'candidate': candidate.toMap(),
        });
      };

      _peerConnection!.onTrack = (event) {
        if (event.track.kind == 'video') {
          final trackInfo = VideoTrackInfo(
            id: event.track.id ?? 'remote-${DateTime.now().millisecondsSinceEpoch}',
            track: event.track,
            stream: event.streams.isNotEmpty ? event.streams[0] : null,
            isLocal: false,
          );

          state = state.copyWith(
            remoteTracks: [...state.remoteTracks, trackInfo],
          );
        }
      };

      _peerConnection!.onConnectionState = (connectionState) {
        switch (connectionState) {
          case webrtc.RTCPeerConnectionState.RTCPeerConnectionStateConnected:
            state = state.copyWith(connectionState: WebRtcConnectionState.connected);
            break;
          case webrtc.RTCPeerConnectionState.RTCPeerConnectionStateDisconnected:
            state = state.copyWith(connectionState: WebRtcConnectionState.disconnected);
            break;
          case webrtc.RTCPeerConnectionState.RTCPeerConnectionStateFailed:
            state = state.copyWith(connectionState: WebRtcConnectionState.failed);
            break;
          case webrtc.RTCPeerConnectionState.RTCPeerConnectionStateClosed:
            state = state.copyWith(connectionState: WebRtcConnectionState.closed);
            break;
          default:
            break;
        }
      };

      // Add local tracks to connection
      if (_localStream != null) {
        for (final track in _localStream!.getTracks()) {
          await _peerConnection!.addTrack(track, _localStream!);
        }
      }
    } catch (e) {
      state = state.copyWith(
        connectionState: WebRtcConnectionState.failed,
        errorMessage: 'Failed to create peer connection: $e',
      );
    }
  }

  /// Handle incoming signaling message
  Future<void> handleSignalingMessage(Map<String, dynamic> message) async {
    final type = message['type'] as String?;

    switch (type) {
      case 'offer':
        await _handleOffer(message);
        break;
      case 'answer':
        await _handleAnswer(message);
        break;
      case 'ice-candidate':
        await _handleIceCandidate(message);
        break;
    }
  }

  Future<void> _handleOffer(Map<String, dynamic> message) async {
    if (_peerConnection == null) {
      await initializePeerConnection();
    }

    final sdp = message['sdp'] as String;
    await _peerConnection!.setRemoteDescription(
      webrtc.RTCSessionDescription(sdp, 'offer'),
    );

    final answer = await _peerConnection!.createAnswer();
    await _peerConnection!.setLocalDescription(answer);

    _sendSignalingMessage({
      'type': 'answer',
      'sdp': answer.sdp,
    });
  }

  Future<void> _handleAnswer(Map<String, dynamic> message) async {
    final sdp = message['sdp'] as String;
    await _peerConnection?.setRemoteDescription(
      webrtc.RTCSessionDescription(sdp, 'answer'),
    );
  }

  Future<void> _handleIceCandidate(Map<String, dynamic> message) async {
    final candidateMap = message['candidate'] as Map<String, dynamic>;
    final candidate = webrtc.RTCIceCandidate(
      candidateMap['candidate'] as String,
      candidateMap['sdpMid'] as String?,
      candidateMap['sdpMLineIndex'] as int?,
    );
    await _peerConnection?.addCandidate(candidate);
  }

  void _sendSignalingMessage(Map<String, dynamic> message) {
    // Send via canvas client or dedicated signaling channel
    final canvasClient = _ref.read(canvasClientProvider);
    if (canvasClient.isConnected) {
      canvasClient.callTool('webrtc/signal', {
        'session_id': _sessionId,
        'message': message,
      });
    } else {
      debugPrint('Cannot send signaling message: not connected');
    }
  }

  /// Associate a video track with a canvas element
  Future<void> bindToCanvasElement(String trackId, String elementId) async {
    final trackIndex = state.allTracks.indexWhere((t) => t.id == trackId);
    if (trackIndex == -1) {
      debugPrint('Cannot bind to canvas: track $trackId not found');
      return;
    }

    final track = state.allTracks[trackIndex];
    final updatedTrack = track.copyWith(canvasElementId: elementId);

    if (track.isLocal) {
      final localTracks = [...state.localTracks];
      final localIndex = localTracks.indexWhere((t) => t.id == trackId);
      if (localIndex != -1) {
        localTracks[localIndex] = updatedTrack;
        state = state.copyWith(localTracks: localTracks);
      }
    } else {
      final remoteTracks = [...state.remoteTracks];
      final remoteIndex = remoteTracks.indexWhere((t) => t.id == trackId);
      if (remoteIndex != -1) {
        remoteTracks[remoteIndex] = updatedTrack;
        state = state.copyWith(remoteTracks: remoteTracks);
      }
    }

    // Update canvas element with video info
    final canvasController = _ref.read(canvasControllerProvider(_sessionId).notifier);
    await canvasController.updateElement(elementId, {
      'videoTrackId': trackId,
      'type': 'video',
    });
  }

  /// Get or create a renderer for a track
  Future<webrtc.RTCVideoRenderer> getRenderer(String trackId) async {
    if (_renderers.containsKey(trackId)) {
      return _renderers[trackId]!;
    }

    final renderer = webrtc.RTCVideoRenderer();
    await renderer.initialize();
    _renderers[trackId] = renderer;

    // Find the track and set its stream
    final trackInfo = state.getTrack(trackId);
    if (trackInfo != null) {
      renderer.srcObject = trackInfo.stream;
    }

    return renderer;
  }

  @override
  void dispose() {
    // Clean up renderers
    for (final renderer in _renderers.values) {
      renderer.dispose();
    }
    _renderers.clear();

    // Clean up streams
    _localStream?.dispose();
    _peerConnection?.close();

    super.dispose();
  }
}

// ============================================================
// Providers
// ============================================================

/// Provider for WebRTC session controller (family by session ID)
final webRtcSessionProvider =
    StateNotifierProvider.family<WebRtcSessionController, WebRtcState, String>(
        (ref, sessionId) {
  return WebRtcSessionController(ref, sessionId);
});

/// Provider for local video tracks in a session
final localVideoTracksProvider =
    Provider.family<List<VideoTrackInfo>, String>((ref, sessionId) {
  final state = ref.watch(webRtcSessionProvider(sessionId));
  return state.localTracks;
});

/// Provider for remote video tracks in a session
final remoteVideoTracksProvider =
    Provider.family<List<VideoTrackInfo>, String>((ref, sessionId) {
  final state = ref.watch(webRtcSessionProvider(sessionId));
  return state.remoteTracks;
});

/// Provider for WebRTC connection state
final webRtcConnectionStateProvider =
    Provider.family<WebRtcConnectionState, String>((ref, sessionId) {
  final state = ref.watch(webRtcSessionProvider(sessionId));
  return state.connectionState;
});

/// Provider for checking if video is enabled
final isVideoEnabledProvider = Provider.family<bool, String>((ref, sessionId) {
  final state = ref.watch(webRtcSessionProvider(sessionId));
  return state.isVideoEnabled;
});

/// Provider for checking if audio is enabled
final isAudioEnabledProvider = Provider.family<bool, String>((ref, sessionId) {
  final state = ref.watch(webRtcSessionProvider(sessionId));
  return state.isAudioEnabled;
});
