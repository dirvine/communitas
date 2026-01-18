import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../core/theme/colors.dart';
import '../services/tts_provider.dart';

// ============================================================
// TTS Control Button
// ============================================================

/// Button to control TTS playback
class TtsControlButton extends ConsumerWidget {
  /// Size of the button
  final double size;

  const TtsControlButton({
    super.key,
    this.size = 48,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final ttsState = ref.watch(ttsControllerProvider);
    final controller = ref.read(ttsControllerProvider.notifier);

    return GestureDetector(
      onTap: () => _handleTap(controller, ttsState),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 200),
        width: size,
        height: size,
        decoration: BoxDecoration(
          shape: BoxShape.circle,
          color: _getBackgroundColor(ttsState),
          boxShadow: [
            BoxShadow(
              color: ttsState.isSpeaking
                  ? CommunitasColors.jade.withValues(alpha: 0.4)
                  : Colors.black26,
              blurRadius: ttsState.isSpeaking ? 12 : 6,
              spreadRadius: ttsState.isSpeaking ? 2 : 0,
            ),
          ],
        ),
        child: Stack(
          alignment: Alignment.center,
          children: [
            // Speaking animation
            if (ttsState.isSpeaking)
              _SpeakingAnimation(size: size),

            // Icon
            Icon(
              _getIcon(ttsState),
              color: CommunitasColors.cream,
              size: size * 0.45,
            ),

            // Queue badge
            if (ttsState.queueLength > 0)
              Positioned(
                top: 0,
                right: 0,
                child: Container(
                  padding: const EdgeInsets.all(4),
                  decoration: BoxDecoration(
                    color: CommunitasColors.jade,
                    shape: BoxShape.circle,
                  ),
                  child: Text(
                    '${ttsState.queueLength}',
                    style: const TextStyle(
                      color: CommunitasColors.cream,
                      fontSize: 10,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }

  void _handleTap(TtsController controller, TtsState state) {
    if (state.isSpeaking) {
      controller.stop();
    } else if (state.isPaused) {
      controller.resume();
    }
    // If not speaking or paused, button just shows status
  }

  Color _getBackgroundColor(TtsState state) {
    switch (state.status) {
      case TtsStatus.speaking:
        return CommunitasColors.jade;
      case TtsStatus.paused:
        return CommunitasColors.fern;
      case TtsStatus.error:
        return CommunitasColors.error;
      case TtsStatus.unavailable:
        return CommunitasColors.moss;
      default:
        return CommunitasColors.deepForest;
    }
  }

  IconData _getIcon(TtsState state) {
    switch (state.status) {
      case TtsStatus.speaking:
        return Icons.stop;
      case TtsStatus.paused:
        return Icons.play_arrow;
      case TtsStatus.error:
        return Icons.volume_off;
      case TtsStatus.unavailable:
        return Icons.volume_mute;
      default:
        return Icons.volume_up;
    }
  }
}

/// Animated speaking indicator
class _SpeakingAnimation extends StatefulWidget {
  final double size;

  const _SpeakingAnimation({required this.size});

  @override
  State<_SpeakingAnimation> createState() => _SpeakingAnimationState();
}

class _SpeakingAnimationState extends State<_SpeakingAnimation>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(milliseconds: 800),
      vsync: this,
    )..repeat(reverse: true);
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
        return Container(
          width: widget.size * (1.0 + 0.15 * _controller.value),
          height: widget.size * (1.0 + 0.15 * _controller.value),
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            border: Border.all(
              color: CommunitasColors.jade.withValues(alpha: 0.5 - 0.3 * _controller.value),
              width: 2,
            ),
          ),
        );
      },
    );
  }
}

// ============================================================
// TTS Settings Panel
// ============================================================

/// Panel for configuring TTS settings
class TtsSettingsPanel extends ConsumerWidget {
  const TtsSettingsPanel({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final ttsState = ref.watch(ttsControllerProvider);
    final controller = ref.read(ttsControllerProvider.notifier);

    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: CommunitasColors.moss.withValues(alpha: 0.95),
        borderRadius: BorderRadius.circular(16),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header
          Row(
            children: [
              const Icon(Icons.settings_voice, color: CommunitasColors.jade),
              const SizedBox(width: 8),
              Text(
                'Voice Settings',
                style: TextStyle(
                  color: CommunitasColors.cream,
                  fontSize: 16,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),

          // Speech Rate
          _SettingsSlider(
            label: 'Speed',
            value: ttsState.speechRate,
            min: 0.0,
            max: 1.0,
            icon: Icons.speed,
            onChanged: (value) => controller.setSpeechRate(value),
          ),
          const SizedBox(height: 12),

          // Pitch
          _SettingsSlider(
            label: 'Pitch',
            value: (ttsState.pitch - 0.5) / 1.5, // Normalize to 0-1
            min: 0.0,
            max: 1.0,
            icon: Icons.tune,
            onChanged: (value) => controller.setPitch(0.5 + value * 1.5),
          ),
          const SizedBox(height: 12),

          // Volume
          _SettingsSlider(
            label: 'Volume',
            value: ttsState.volume,
            min: 0.0,
            max: 1.0,
            icon: Icons.volume_up,
            onChanged: (value) => controller.setVolume(value),
          ),
          const SizedBox(height: 16),

          // Voice selector
          if (ttsState.availableVoices.isNotEmpty) ...[
            Text(
              'Voice',
              style: TextStyle(
                color: CommunitasColors.cream.withValues(alpha: 0.7),
                fontSize: 14,
              ),
            ),
            const SizedBox(height: 8),
            _VoiceSelector(
              voices: ttsState.availableVoices,
              selectedVoice: ttsState.selectedVoice,
              onVoiceSelected: (voice) => controller.setVoice(voice),
            ),
          ],
        ],
      ),
    );
  }
}

/// Slider for TTS settings
class _SettingsSlider extends StatelessWidget {
  final String label;
  final double value;
  final double min;
  final double max;
  final IconData icon;
  final ValueChanged<double> onChanged;

  const _SettingsSlider({
    required this.label,
    required this.value,
    required this.min,
    required this.max,
    required this.icon,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Icon(icon, color: CommunitasColors.fern, size: 20),
        const SizedBox(width: 12),
        SizedBox(
          width: 60,
          child: Text(
            label,
            style: TextStyle(
              color: CommunitasColors.cream.withValues(alpha: 0.7),
              fontSize: 14,
            ),
          ),
        ),
        Expanded(
          child: SliderTheme(
            data: SliderThemeData(
              activeTrackColor: CommunitasColors.jade,
              inactiveTrackColor: CommunitasColors.deepForest,
              thumbColor: CommunitasColors.jade,
              overlayColor: CommunitasColors.jade.withValues(alpha: 0.2),
              trackHeight: 4,
            ),
            child: Slider(
              value: value,
              min: min,
              max: max,
              onChanged: onChanged,
            ),
          ),
        ),
      ],
    );
  }
}

/// Voice selector dropdown
class _VoiceSelector extends StatelessWidget {
  final List<TtsVoice> voices;
  final TtsVoice? selectedVoice;
  final ValueChanged<TtsVoice> onVoiceSelected;

  const _VoiceSelector({
    required this.voices,
    required this.selectedVoice,
    required this.onVoiceSelected,
  });

  @override
  Widget build(BuildContext context) {
    // Filter to English voices for simplicity
    final englishVoices = voices.where((v) => v.locale.startsWith('en')).toList();

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      decoration: BoxDecoration(
        color: CommunitasColors.deepForest,
        borderRadius: BorderRadius.circular(8),
      ),
      child: DropdownButton<TtsVoice>(
        value: selectedVoice,
        isExpanded: true,
        dropdownColor: CommunitasColors.deepForest,
        underline: const SizedBox.shrink(),
        icon: const Icon(Icons.arrow_drop_down, color: CommunitasColors.fern),
        items: englishVoices.map((voice) {
          return DropdownMenuItem<TtsVoice>(
            value: voice,
            child: Row(
              children: [
                if (voice.isEnhanced)
                  const Icon(Icons.star, color: CommunitasColors.jade, size: 16),
                if (voice.isEnhanced) const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    voice.name,
                    style: const TextStyle(
                      color: CommunitasColors.cream,
                      fontSize: 14,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                Text(
                  voice.locale,
                  style: TextStyle(
                    color: CommunitasColors.cream.withValues(alpha: 0.5),
                    fontSize: 12,
                  ),
                ),
              ],
            ),
          );
        }).toList(),
        onChanged: (voice) {
          if (voice != null) {
            onVoiceSelected(voice);
          }
        },
      ),
    );
  }
}

// ============================================================
// Speaking Indicator
// ============================================================

/// Indicator that shows when TTS is speaking
class TtsSpeakingIndicator extends ConsumerWidget {
  const TtsSpeakingIndicator({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final ttsState = ref.watch(ttsControllerProvider);

    if (!ttsState.isSpeaking && ttsState.currentText == null) {
      return const SizedBox.shrink();
    }

    return AnimatedContainer(
      duration: const Duration(milliseconds: 200),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: CommunitasColors.moss.withValues(alpha: 0.9),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(
          color: CommunitasColors.jade.withValues(alpha: 0.5),
          width: 1,
        ),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          _SoundWaveAnimation(),
          const SizedBox(width: 8),
          Flexible(
            child: Text(
              ttsState.currentText ?? '',
              style: TextStyle(
                color: CommunitasColors.cream.withValues(alpha: 0.9),
                fontSize: 13,
              ),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ],
      ),
    );
  }
}

/// Animated sound wave
class _SoundWaveAnimation extends StatefulWidget {
  @override
  State<_SoundWaveAnimation> createState() => _SoundWaveAnimationState();
}

class _SoundWaveAnimationState extends State<_SoundWaveAnimation>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(milliseconds: 600),
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
          children: List.generate(4, (index) {
            final offset = index * 0.15;
            final height = 8 + 8 * (0.5 + 0.5 *
                (1 - ((_controller.value + offset) % 1.0 - 0.5).abs() * 2));

            return Padding(
              padding: const EdgeInsets.symmetric(horizontal: 1),
              child: Container(
                width: 3,
                height: height,
                decoration: BoxDecoration(
                  color: CommunitasColors.jade,
                  borderRadius: BorderRadius.circular(1.5),
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
// TTS Status Badge
// ============================================================

/// Small badge showing TTS status
class TtsStatusBadge extends ConsumerWidget {
  const TtsStatusBadge({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final status = ref.watch(ttsStatusProvider);

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: _getStatusColor(status).withValues(alpha: 0.2),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            _getStatusIcon(status),
            size: 14,
            color: _getStatusColor(status),
          ),
          const SizedBox(width: 4),
          Text(
            _getStatusText(status),
            style: TextStyle(
              color: _getStatusColor(status),
              fontSize: 11,
              fontWeight: FontWeight.w500,
            ),
          ),
        ],
      ),
    );
  }

  Color _getStatusColor(TtsStatus status) {
    switch (status) {
      case TtsStatus.ready:
        return CommunitasColors.jade;
      case TtsStatus.speaking:
        return CommunitasColors.jade;
      case TtsStatus.paused:
        return CommunitasColors.fern;
      case TtsStatus.unavailable:
        return CommunitasColors.moss;
      case TtsStatus.error:
        return CommunitasColors.error;
      default:
        return CommunitasColors.moss;
    }
  }

  IconData _getStatusIcon(TtsStatus status) {
    switch (status) {
      case TtsStatus.ready:
        return Icons.volume_up;
      case TtsStatus.speaking:
        return Icons.graphic_eq;
      case TtsStatus.paused:
        return Icons.pause;
      case TtsStatus.unavailable:
        return Icons.volume_off;
      case TtsStatus.error:
        return Icons.error_outline;
      default:
        return Icons.hourglass_empty;
    }
  }

  String _getStatusText(TtsStatus status) {
    switch (status) {
      case TtsStatus.ready:
        return 'Ready';
      case TtsStatus.speaking:
        return 'Speaking';
      case TtsStatus.paused:
        return 'Paused';
      case TtsStatus.unavailable:
        return 'Unavailable';
      case TtsStatus.error:
        return 'Error';
      default:
        return 'Loading';
    }
  }
}
