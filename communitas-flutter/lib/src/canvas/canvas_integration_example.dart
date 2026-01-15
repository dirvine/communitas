/// Example integration showing how to use all Canvas components together.
///
/// This demonstrates the complete MVP flow:
/// 1. Canvas overlay with real-time collaboration
/// 2. Video rendering from WebRTC sessions
/// 3. Voice input for hands-free control
/// 4. TTS output for agent responses
library canvas_integration_example;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../core/theme/colors.dart';
import '../services/canvas_provider.dart';
import '../services/stt_provider.dart';
import '../services/tts_provider.dart';
import '../services/webrtc_provider.dart';
import '../widgets/canvas_widget.dart';
import '../widgets/tts_output_widget.dart';
import '../widgets/voice_input_widget.dart';
import '../widgets/webrtc_video_widget.dart';

/// Complete canvas integration example with all components
class CanvasIntegrationExample extends ConsumerStatefulWidget {
  /// Session ID for canvas and WebRTC
  final String sessionId;

  /// Callback when agent sends a message (from voice commands)
  final void Function(String message)? onAgentMessage;

  const CanvasIntegrationExample({
    super.key,
    required this.sessionId,
    this.onAgentMessage,
  });

  @override
  ConsumerState<CanvasIntegrationExample> createState() =>
      _CanvasIntegrationExampleState();
}

class _CanvasIntegrationExampleState
    extends ConsumerState<CanvasIntegrationExample> {
  bool _showSettings = false;

  @override
  void initState() {
    super.initState();
    _initializeServices();
  }

  Future<void> _initializeServices() async {
    try {
      // Initialize WebRTC for video
      final webrtcController =
          ref.read(webRtcSessionProvider(widget.sessionId).notifier);
      await webrtcController.startLocalVideo();

      // Set up voice command handler
      final sttController = ref.read(sttControllerProvider.notifier);
      sttController.onVoiceCommand = _handleVoiceCommand;
    } catch (e) {
      debugPrint('Failed to initialize canvas services: $e');
    }
  }

  void _handleVoiceCommand(String command) {
    // Process voice commands and potentially speak responses
    _processCommand(command);
  }

  Future<void> _processCommand(String command) async {
    final lowerCommand = command.toLowerCase();

    // Example voice commands
    if (lowerCommand.contains('help')) {
      await _speakResponse(
          'Available commands: zoom in, zoom out, pan left, pan right, '
          'select, deselect, and ask followed by your question.');
    } else if (lowerCommand.startsWith('ask ') ||
        lowerCommand.startsWith('tell me ')) {
      // Forward to agent
      final question = lowerCommand.replaceFirst(RegExp(r'^(ask|tell me)\s+'), '');
      widget.onAgentMessage?.call(question);

      // Placeholder response - in real implementation this would come from the agent
      await _speakResponse('Processing your request: $question');
    } else {
      // Pass to canvas controller for navigation/tool commands
      final canvasController =
          ref.read(canvasControllerProvider(widget.sessionId).notifier);

      if (lowerCommand.contains('zoom in')) {
        canvasController.handleScale(1.5, 0, 0);
        await _speakResponse('Zooming in');
      } else if (lowerCommand.contains('zoom out')) {
        canvasController.handleScale(0.67, 0, 0);
        await _speakResponse('Zooming out');
      }
    }
  }

  Future<void> _speakResponse(String response) async {
    final ttsController = ref.read(ttsControllerProvider.notifier);
    await ttsController.speak(response);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: CommunitasColors.deepForest,
      body: Stack(
        children: [
          // Main canvas
          Positioned.fill(
            child: CanvasOverlayWidget(
              sessionId: widget.sessionId,
              onInteraction: _handleCanvasInteraction,
              showConnectionStatus: true,
              showElements: true,
            ),
          ),

          // Local video preview (picture-in-picture)
          Positioned(
            top: 16,
            right: 16,
            child: LocalVideoPreview(
              sessionId: widget.sessionId,
              size: const Size(120, 160),
              showControls: true,
            ),
          ),

          // Voice input panel
          VoiceInputPanel(
            sessionId: widget.sessionId,
            alignment: Alignment.bottomRight,
            onVoiceCommand: _handleVoiceCommand,
          ),

          // TTS speaking indicator
          Positioned(
            bottom: 80,
            left: 16,
            right: 80,
            child: const TtsSpeakingIndicator(),
          ),

          // Settings toggle
          Positioned(
            top: 16,
            left: 16,
            child: _buildSettingsButton(),
          ),

          // Settings panel
          if (_showSettings)
            Positioned(
              top: 60,
              left: 16,
              child: _buildSettingsPanel(),
            ),

          // Status indicators
          Positioned(
            bottom: 16,
            left: 16,
            child: _buildStatusBar(),
          ),
        ],
      ),
    );
  }

  void _handleCanvasInteraction(CanvasInteraction interaction) {
    // Handle canvas interactions
    // Could trigger voice feedback, update state, etc.
    if (interaction.type == InteractionType.tap &&
        interaction.elementId != null) {
      // Optionally announce selected element
      // _speakResponse('Selected element');
    }
  }

  Widget _buildSettingsButton() {
    return GestureDetector(
      onTap: () => setState(() => _showSettings = !_showSettings),
      child: Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: CommunitasColors.moss.withValues(alpha: 0.9),
          shape: BoxShape.circle,
        ),
        child: Icon(
          _showSettings ? Icons.close : Icons.settings,
          color: CommunitasColors.cream,
          size: 24,
        ),
      ),
    );
  }

  Widget _buildSettingsPanel() {
    return Container(
      width: 280,
      decoration: BoxDecoration(
        color: CommunitasColors.moss.withValues(alpha: 0.95),
        borderRadius: BorderRadius.circular(16),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.3),
            blurRadius: 12,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Header
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              border: Border(
                bottom: BorderSide(
                  color: CommunitasColors.deepForest.withValues(alpha: 0.5),
                ),
              ),
            ),
            child: Row(
              children: [
                const Icon(Icons.tune, color: CommunitasColors.jade),
                const SizedBox(width: 8),
                Text(
                  'Settings',
                  style: TextStyle(
                    color: CommunitasColors.cream,
                    fontSize: 16,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ],
            ),
          ),

          // TTS Settings
          const Padding(
            padding: EdgeInsets.all(16),
            child: TtsSettingsPanel(),
          ),
        ],
      ),
    );
  }

  Widget _buildStatusBar() {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Connection status
        Consumer(
          builder: (context, ref, _) {
            final canvasState =
                ref.watch(canvasControllerProvider(widget.sessionId));
            return _StatusChip(
              icon: canvasState.isConnected
                  ? Icons.cloud_done
                  : Icons.cloud_off,
              label: canvasState.isConnected ? 'Connected' : 'Offline',
              color: canvasState.isConnected
                  ? CommunitasColors.jade
                  : CommunitasColors.error,
            );
          },
        ),
        const SizedBox(width: 8),

        // Voice status
        const VoiceStatusIndicator(),
        const SizedBox(width: 8),

        // TTS status
        const TtsStatusBadge(),
      ],
    );
  }
}

/// Simple status chip widget
class _StatusChip extends StatelessWidget {
  final IconData icon;
  final String label;
  final Color color;

  const _StatusChip({
    required this.icon,
    required this.label,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.2),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 14, color: color),
          const SizedBox(width: 4),
          Text(
            label,
            style: TextStyle(
              color: color,
              fontSize: 11,
              fontWeight: FontWeight.w500,
            ),
          ),
        ],
      ),
    );
  }
}
