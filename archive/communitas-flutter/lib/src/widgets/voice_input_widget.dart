import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../core/theme/colors.dart';
import '../services/stt_provider.dart';

// ============================================================
// Voice Input Button
// ============================================================

/// Floating action button for voice input
class VoiceInputButton extends ConsumerStatefulWidget {
  /// Session ID for canvas integration
  final String sessionId;

  /// Size of the button
  final double size;

  /// Callback when voice command is recognized
  final void Function(String command)? onVoiceCommand;

  const VoiceInputButton({
    super.key,
    required this.sessionId,
    this.size = 56,
    this.onVoiceCommand,
  });

  @override
  ConsumerState<VoiceInputButton> createState() => _VoiceInputButtonState();
}

class _VoiceInputButtonState extends ConsumerState<VoiceInputButton> {
  @override
  void initState() {
    super.initState();
    _updateVoiceCommandCallback();
  }

  @override
  void didUpdateWidget(VoiceInputButton oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.onVoiceCommand != widget.onVoiceCommand) {
      _updateVoiceCommandCallback();
    }
  }

  @override
  void dispose() {
    // Clear the callback when widget is disposed
    final controller = ref.read(sttControllerProvider.notifier);
    if (controller.onVoiceCommand == widget.onVoiceCommand) {
      controller.onVoiceCommand = null;
    }
    super.dispose();
  }

  void _updateVoiceCommandCallback() {
    final controller = ref.read(sttControllerProvider.notifier);
    controller.onVoiceCommand = widget.onVoiceCommand;
  }

  @override
  Widget build(BuildContext context) {
    final sttState = ref.watch(sttControllerProvider);
    final controller = ref.read(sttControllerProvider.notifier);

    return GestureDetector(
      onTap: () => _handleTap(controller, sttState),
      onLongPress: () => _handleLongPress(controller, sttState),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 200),
        width: widget.size,
        height: widget.size,
        decoration: BoxDecoration(
          shape: BoxShape.circle,
          color: _getBackgroundColor(sttState),
          boxShadow: [
            BoxShadow(
              color: _getShadowColor(sttState),
              blurRadius: sttState.isListening ? 16 : 8,
              spreadRadius: sttState.isListening ? 2 : 0,
            ),
          ],
        ),
        child: Stack(
          alignment: Alignment.center,
          children: [
            // Pulsing ring when listening
            if (sttState.isListening)
              _PulsingRing(size: widget.size),

            // Microphone icon
            Icon(
              _getIcon(sttState),
              color: CommunitasColors.cream,
              size: widget.size * 0.45,
            ),
          ],
        ),
      ),
    );
  }

  void _handleTap(SttController controller, SttState state) {
    if (state.isListening) {
      controller.stopListening();
    } else if (state.isAvailable) {
      controller.startListening(continuous: false);
    }
  }

  void _handleLongPress(SttController controller, SttState state) {
    if (!state.isListening && state.isAvailable) {
      // Start continuous listening mode
      controller.startListening(continuous: true);
    }
  }

  Color _getBackgroundColor(SttState state) {
    switch (state.status) {
      case SttStatus.listening:
        return CommunitasColors.jade;
      case SttStatus.processing:
        return CommunitasColors.fern;
      case SttStatus.error:
        return CommunitasColors.error;
      case SttStatus.unavailable:
        return CommunitasColors.moss;
      default:
        return CommunitasColors.deepForest;
    }
  }

  Color _getShadowColor(SttState state) {
    if (state.isListening) {
      return CommunitasColors.jade.withValues(alpha: 0.5);
    }
    return Colors.black26;
  }

  IconData _getIcon(SttState state) {
    switch (state.status) {
      case SttStatus.listening:
        return Icons.mic;
      case SttStatus.processing:
        return Icons.hearing;
      case SttStatus.error:
        return Icons.mic_off;
      case SttStatus.unavailable:
        return Icons.mic_none;
      default:
        return Icons.mic_none;
    }
  }
}

/// Animated pulsing ring effect
class _PulsingRing extends StatefulWidget {
  final double size;

  const _PulsingRing({required this.size});

  @override
  State<_PulsingRing> createState() => _PulsingRingState();
}

class _PulsingRingState extends State<_PulsingRing>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _scaleAnimation;
  late Animation<double> _opacityAnimation;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(milliseconds: 1000),
      vsync: this,
    )..repeat();

    _scaleAnimation = Tween<double>(begin: 1.0, end: 1.5).animate(
      CurvedAnimation(parent: _controller, curve: Curves.easeOut),
    );

    _opacityAnimation = Tween<double>(begin: 0.6, end: 0.0).animate(
      CurvedAnimation(parent: _controller, curve: Curves.easeOut),
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _controller,
      builder: (context, child) {
        return Transform.scale(
          scale: _scaleAnimation.value,
          child: Container(
            width: widget.size,
            height: widget.size,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              border: Border.all(
                color: CommunitasColors.jade.withValues(alpha: _opacityAnimation.value),
                width: 3,
              ),
            ),
          ),
        );
      },
    );
  }
}

// ============================================================
// Voice Transcript Display
// ============================================================

/// Widget that displays live transcription
class VoiceTranscriptDisplay extends ConsumerWidget {
  /// Whether to show interim results
  final bool showInterim;

  /// Maximum height of the display
  final double maxHeight;

  const VoiceTranscriptDisplay({
    super.key,
    this.showInterim = true,
    this.maxHeight = 100,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final sttState = ref.watch(sttControllerProvider);
    final result = sttState.currentResult;

    if (result == null || result.text.isEmpty) {
      if (sttState.isListening) {
        return _buildListeningIndicator();
      }
      return const SizedBox.shrink();
    }

    return AnimatedContainer(
      duration: const Duration(milliseconds: 200),
      constraints: BoxConstraints(maxHeight: maxHeight),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      decoration: BoxDecoration(
        color: CommunitasColors.moss.withValues(alpha: 0.95),
        borderRadius: BorderRadius.circular(16),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.2),
            blurRadius: 8,
            offset: const Offset(0, 2),
          ),
        ],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Status indicator
          Row(
            children: [
              Icon(
                result.isFinal ? Icons.check_circle : Icons.hearing,
                color: result.isFinal ? CommunitasColors.jade : CommunitasColors.fern,
                size: 16,
              ),
              const SizedBox(width: 8),
              Text(
                result.isFinal ? 'Recognized' : 'Listening...',
                style: TextStyle(
                  color: CommunitasColors.cream.withValues(alpha: 0.7),
                  fontSize: 12,
                ),
              ),
              if (result.confidence > 0) ...[
                const Spacer(),
                Text(
                  '${(result.confidence * 100).toInt()}%',
                  style: TextStyle(
                    color: CommunitasColors.cream.withValues(alpha: 0.5),
                    fontSize: 12,
                  ),
                ),
              ],
            ],
          ),
          const SizedBox(height: 8),
          // Transcript text
          Text(
            result.text,
            style: TextStyle(
              color: CommunitasColors.cream,
              fontSize: 16,
              fontStyle: result.isFinal ? FontStyle.normal : FontStyle.italic,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildListeningIndicator() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      decoration: BoxDecoration(
        color: CommunitasColors.moss.withValues(alpha: 0.95),
        borderRadius: BorderRadius.circular(16),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          _ListeningDots(),
          const SizedBox(width: 12),
          Text(
            'Listening...',
            style: TextStyle(
              color: CommunitasColors.cream.withValues(alpha: 0.7),
              fontSize: 14,
            ),
          ),
        ],
      ),
    );
  }
}

/// Animated dots for listening indicator
class _ListeningDots extends StatefulWidget {
  @override
  State<_ListeningDots> createState() => _ListeningDotsState();
}

class _ListeningDotsState extends State<_ListeningDots>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(milliseconds: 1500),
      vsync: this,
    )..repeat();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _controller,
      builder: (context, child) {
        return Row(
          mainAxisSize: MainAxisSize.min,
          children: List.generate(3, (index) {
            final delay = index * 0.2;
            final progress = (_controller.value + delay) % 1.0;
            final scale = 0.5 + 0.5 * (1 - (progress - 0.5).abs() * 2);

            return Padding(
              padding: const EdgeInsets.symmetric(horizontal: 2),
              child: Transform.scale(
                scale: scale,
                child: Container(
                  width: 8,
                  height: 8,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: CommunitasColors.jade.withValues(alpha: 0.5 + 0.5 * scale),
                  ),
                ),
              ),
            );
          }),
        );
      },
    );
  }
}

// ============================================================
// Voice Input Panel (Combined)
// ============================================================

/// Complete voice input panel with button and transcript
class VoiceInputPanel extends ConsumerWidget {
  /// Session ID for canvas integration
  final String sessionId;

  /// Position on screen
  final Alignment alignment;

  /// Callback when voice command is recognized
  final void Function(String command)? onVoiceCommand;

  const VoiceInputPanel({
    super.key,
    required this.sessionId,
    this.alignment = Alignment.bottomRight,
    this.onVoiceCommand,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final sttState = ref.watch(sttControllerProvider);

    return Positioned(
      bottom: alignment == Alignment.bottomRight || alignment == Alignment.bottomLeft
          ? 16
          : null,
      top: alignment == Alignment.topRight || alignment == Alignment.topLeft
          ? 16
          : null,
      right: alignment == Alignment.bottomRight || alignment == Alignment.topRight
          ? 16
          : null,
      left: alignment == Alignment.bottomLeft || alignment == Alignment.topLeft
          ? 16
          : null,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: alignment == Alignment.bottomRight || alignment == Alignment.topRight
            ? CrossAxisAlignment.end
            : CrossAxisAlignment.start,
        children: [
          // Transcript display (above button when active)
          if (sttState.isListening || sttState.currentResult != null) ...[
            const VoiceTranscriptDisplay(),
            const SizedBox(height: 12),
          ],

          // Voice input button
          VoiceInputButton(
            sessionId: sessionId,
            onVoiceCommand: onVoiceCommand,
          ),
        ],
      ),
    );
  }
}

// ============================================================
// Voice Status Indicator
// ============================================================

/// Small status indicator for voice availability
class VoiceStatusIndicator extends ConsumerWidget {
  const VoiceStatusIndicator({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final status = ref.watch(sttStatusProvider);

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: _getStatusColor(status).withValues(alpha: 0.2),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            width: 8,
            height: 8,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: _getStatusColor(status),
            ),
          ),
          const SizedBox(width: 6),
          Text(
            _getStatusText(status),
            style: TextStyle(
              color: _getStatusColor(status),
              fontSize: 12,
              fontWeight: FontWeight.w500,
            ),
          ),
        ],
      ),
    );
  }

  Color _getStatusColor(SttStatus status) {
    switch (status) {
      case SttStatus.ready:
        return CommunitasColors.jade;
      case SttStatus.listening:
        return CommunitasColors.jade;
      case SttStatus.processing:
        return CommunitasColors.fern;
      case SttStatus.unavailable:
        return CommunitasColors.moss;
      case SttStatus.error:
        return CommunitasColors.error;
      default:
        return CommunitasColors.moss;
    }
  }

  String _getStatusText(SttStatus status) {
    switch (status) {
      case SttStatus.ready:
        return 'Voice Ready';
      case SttStatus.listening:
        return 'Listening';
      case SttStatus.processing:
        return 'Processing';
      case SttStatus.unavailable:
        return 'Voice Unavailable';
      case SttStatus.error:
        return 'Voice Error';
      default:
        return 'Initializing';
    }
  }
}
