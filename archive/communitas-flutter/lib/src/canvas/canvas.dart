/// Canvas Integration Module
///
/// This module provides the complete Canvas integration for Communitas,
/// including:
/// - Real-time collaborative canvas with WebSocket communication
/// - WebRTC video rendering on canvas elements
/// - Speech-to-text voice input for hands-free control
/// - Text-to-speech output for agent responses
///
/// ## Quick Start
///
/// ```dart
/// import 'package:communitas/src/canvas/canvas.dart';
///
/// // In your widget tree:
/// CanvasOverlayWidget(
///   sessionId: 'my-session-id',
///   onInteraction: (interaction) => print('Interaction: $interaction'),
/// );
/// ```
library canvas;

// Services
export '../services/canvas_client.dart';
export '../services/canvas_provider.dart';
export '../services/webrtc_provider.dart';
export '../services/stt_provider.dart';
export '../services/tts_provider.dart';

// Widgets
export '../widgets/canvas_widget.dart';
export '../widgets/webrtc_video_widget.dart';
export '../widgets/voice_input_widget.dart';
export '../widgets/tts_output_widget.dart';
