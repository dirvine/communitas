import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_webrtc/flutter_webrtc.dart';

import '../core/theme/colors.dart';
import '../services/webrtc_provider.dart';

// ============================================================
// Video Renderer Widget
// ============================================================

/// Widget that displays a WebRTC video track
class WebRtcVideoView extends ConsumerStatefulWidget {
  /// Session ID for the WebRTC session
  final String sessionId;

  /// Track ID to display
  final String trackId;

  /// Whether to mirror the video (for local camera)
  final bool mirror;

  /// Object fit mode for the video
  final RTCVideoViewObjectFit objectFit;

  /// Placeholder widget when video is not available
  final Widget? placeholder;

  /// Decoration for the video container
  final BoxDecoration? decoration;

  const WebRtcVideoView({
    super.key,
    required this.sessionId,
    required this.trackId,
    this.mirror = false,
    this.objectFit = RTCVideoViewObjectFit.RTCVideoViewObjectFitCover,
    this.placeholder,
    this.decoration,
  });

  @override
  ConsumerState<WebRtcVideoView> createState() => _WebRtcVideoViewState();
}

class _WebRtcVideoViewState extends ConsumerState<WebRtcVideoView> {
  RTCVideoRenderer? _renderer;
  bool _isInitialized = false;

  @override
  void initState() {
    super.initState();
    _initializeRenderer();
  }

  Future<void> _initializeRenderer() async {
    final controller = ref.read(webRtcSessionProvider(widget.sessionId).notifier);
    _renderer = await controller.getRenderer(widget.trackId);
    if (mounted) {
      setState(() {
        _isInitialized = true;
      });
    }
  }

  @override
  void dispose() {
    // Renderer cleanup is handled by the controller
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (!_isInitialized || _renderer == null) {
      return widget.placeholder ?? _buildPlaceholder();
    }

    return Container(
      decoration: widget.decoration,
      child: ClipRRect(
        borderRadius: widget.decoration?.borderRadius?.resolve(TextDirection.ltr) ??
            BorderRadius.zero,
        child: RTCVideoView(
          _renderer!,
          mirror: widget.mirror,
          objectFit: widget.objectFit,
        ),
      ),
    );
  }

  Widget _buildPlaceholder() {
    return Container(
      color: CommunitasColors.deepForest,
      child: const Center(
        child: CircularProgressIndicator(
          color: CommunitasColors.jade,
        ),
      ),
    );
  }
}

// ============================================================
// Local Video Preview Widget
// ============================================================

/// Widget that displays the local camera preview
class LocalVideoPreview extends ConsumerWidget {
  /// Session ID for the WebRTC session
  final String sessionId;

  /// Size of the preview
  final Size size;

  /// Border radius for the preview
  final double borderRadius;

  /// Whether to show controls (mute/video toggle)
  final bool showControls;

  const LocalVideoPreview({
    super.key,
    required this.sessionId,
    this.size = const Size(120, 160),
    this.borderRadius = 12,
    this.showControls = true,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(webRtcSessionProvider(sessionId));
    final localTracks = state.localTracks;

    return SizedBox(
      width: size.width,
      height: size.height,
      child: Stack(
        children: [
          // Video preview
          Container(
            decoration: BoxDecoration(
              color: CommunitasColors.deepForest,
              borderRadius: BorderRadius.circular(borderRadius),
              border: Border.all(
                color: CommunitasColors.jade.withValues(alpha: 0.5),
                width: 2,
              ),
            ),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(borderRadius - 2),
              child: localTracks.isNotEmpty
                  ? WebRtcVideoView(
                      sessionId: sessionId,
                      trackId: localTracks.first.id,
                      mirror: true,
                    )
                  : _buildNoVideoPlaceholder(),
            ),
          ),

          // Controls overlay
          if (showControls)
            Positioned(
              bottom: 8,
              left: 0,
              right: 0,
              child: _buildControls(context, ref),
            ),
        ],
      ),
    );
  }

  Widget _buildNoVideoPlaceholder() {
    return Container(
      color: CommunitasColors.deepForest,
      child: const Center(
        child: Icon(
          Icons.videocam_off,
          color: CommunitasColors.fern,
          size: 32,
        ),
      ),
    );
  }

  Widget _buildControls(BuildContext context, WidgetRef ref) {
    final isVideoEnabled = ref.watch(isVideoEnabledProvider(sessionId));
    final isAudioEnabled = ref.watch(isAudioEnabledProvider(sessionId));
    final controller = ref.read(webRtcSessionProvider(sessionId).notifier);

    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        _ControlButton(
          icon: isAudioEnabled ? Icons.mic : Icons.mic_off,
          onPressed: controller.toggleAudio,
          isActive: isAudioEnabled,
        ),
        const SizedBox(width: 8),
        _ControlButton(
          icon: isVideoEnabled ? Icons.videocam : Icons.videocam_off,
          onPressed: controller.toggleVideo,
          isActive: isVideoEnabled,
        ),
      ],
    );
  }
}

class _ControlButton extends StatelessWidget {
  final IconData icon;
  final VoidCallback onPressed;
  final bool isActive;

  const _ControlButton({
    required this.icon,
    required this.onPressed,
    required this.isActive,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onPressed,
      child: Container(
        padding: const EdgeInsets.all(6),
        decoration: BoxDecoration(
          color: isActive
              ? CommunitasColors.jade.withValues(alpha: 0.8)
              : CommunitasColors.error.withValues(alpha: 0.8),
          shape: BoxShape.circle,
        ),
        child: Icon(
          icon,
          color: CommunitasColors.cream,
          size: 16,
        ),
      ),
    );
  }
}

// ============================================================
// Remote Video Grid Widget
// ============================================================

/// Widget that displays remote video streams in a grid layout
class RemoteVideoGrid extends ConsumerWidget {
  /// Session ID for the WebRTC session
  final String sessionId;

  /// Maximum columns in the grid
  final int maxColumns;

  /// Spacing between videos
  final double spacing;

  /// Border radius for video items
  final double borderRadius;

  /// Callback when a video is tapped
  final void Function(String trackId)? onVideoTap;

  const RemoteVideoGrid({
    super.key,
    required this.sessionId,
    this.maxColumns = 3,
    this.spacing = 8,
    this.borderRadius = 8,
    this.onVideoTap,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final remoteTracks = ref.watch(remoteVideoTracksProvider(sessionId));

    if (remoteTracks.isEmpty) {
      return _buildEmptyState();
    }

    return LayoutBuilder(
      builder: (context, constraints) {
        final columns = _calculateColumns(remoteTracks.length);
        final itemWidth = (constraints.maxWidth - (columns - 1) * spacing) / columns;
        final itemHeight = itemWidth * 0.75; // 4:3 aspect ratio

        return Wrap(
          spacing: spacing,
          runSpacing: spacing,
          children: remoteTracks.map((track) {
            return GestureDetector(
              onTap: () => onVideoTap?.call(track.id),
              child: SizedBox(
                width: itemWidth,
                height: itemHeight,
                child: Container(
                  decoration: BoxDecoration(
                    borderRadius: BorderRadius.circular(borderRadius),
                    border: Border.all(
                      color: CommunitasColors.moss,
                      width: 1,
                    ),
                  ),
                  child: ClipRRect(
                    borderRadius: BorderRadius.circular(borderRadius - 1),
                    child: WebRtcVideoView(
                      sessionId: sessionId,
                      trackId: track.id,
                    ),
                  ),
                ),
              ),
            );
          }).toList(),
        );
      },
    );
  }

  int _calculateColumns(int itemCount) {
    if (itemCount == 1) return 1;
    if (itemCount <= 4) return 2;
    return maxColumns;
  }

  Widget _buildEmptyState() {
    return Container(
      padding: const EdgeInsets.all(32),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            Icons.people_outline,
            size: 48,
            color: CommunitasColors.fern.withValues(alpha: 0.5),
          ),
          const SizedBox(height: 16),
          Text(
            'Waiting for participants...',
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

// ============================================================
// Canvas Video Streams Provider
// ============================================================

/// Provider that builds video streams map for canvas integration
final canvasVideoStreamsProvider =
    Provider.family<Map<String, Widget>, CanvasVideoConfig>((ref, config) {
  final localTracks = ref.watch(localVideoTracksProvider(config.sessionId));
  final remoteTracks = ref.watch(remoteVideoTracksProvider(config.sessionId));
  final allTracks = [...localTracks, ...remoteTracks];

  final Map<String, Widget> videoStreams = {};

  for (final track in allTracks) {
    final elementId = track.canvasElementId ?? config.pipMappings[track.id];
    if (elementId != null) {
      videoStreams[elementId] = WebRtcVideoView(
        sessionId: config.sessionId,
        trackId: track.id,
        mirror: track.isLocal,
      );
    }
  }

  // If we have a main video element and tracks, assign the first remote track
  if (config.mainVideoElementId != null && remoteTracks.isNotEmpty) {
    final mainTrack = remoteTracks.first;
    if (!videoStreams.containsKey(config.mainVideoElementId)) {
      videoStreams[config.mainVideoElementId!] = WebRtcVideoView(
        sessionId: config.sessionId,
        trackId: mainTrack.id,
      );
    }
  }

  return videoStreams;
});

/// Configuration for canvas video integration
@immutable
class CanvasVideoConfig {
  /// Session ID (used for both canvas and WebRTC)
  final String sessionId;

  /// Canvas element ID for the main video display
  final String? mainVideoElementId;

  /// Map of track IDs to canvas element IDs for PiP displays
  final Map<String, String> pipMappings;

  const CanvasVideoConfig({
    required this.sessionId,
    this.mainVideoElementId,
    this.pipMappings = const {},
  });

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is CanvasVideoConfig &&
          runtimeType == other.runtimeType &&
          sessionId == other.sessionId &&
          mainVideoElementId == other.mainVideoElementId &&
          pipMappings == other.pipMappings;

  @override
  int get hashCode =>
      sessionId.hashCode ^
      mainVideoElementId.hashCode ^
      pipMappings.hashCode;
}

// ============================================================
// Video Call Controls Widget
// ============================================================

/// Floating controls for a video call
class VideoCallControls extends ConsumerWidget {
  /// Session ID
  final String sessionId;

  /// Callback when call should end
  final VoidCallback? onEndCall;

  /// Callback when screen share is toggled
  final VoidCallback? onScreenShare;

  const VideoCallControls({
    super.key,
    required this.sessionId,
    this.onEndCall,
    this.onScreenShare,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final isVideoEnabled = ref.watch(isVideoEnabledProvider(sessionId));
    final isAudioEnabled = ref.watch(isAudioEnabledProvider(sessionId));
    final controller = ref.read(webRtcSessionProvider(sessionId).notifier);

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      decoration: BoxDecoration(
        color: CommunitasColors.moss.withValues(alpha: 0.95),
        borderRadius: BorderRadius.circular(32),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Mute audio
          _CallControlButton(
            icon: isAudioEnabled ? Icons.mic : Icons.mic_off,
            label: isAudioEnabled ? 'Mute' : 'Unmute',
            onPressed: controller.toggleAudio,
            isActive: isAudioEnabled,
          ),
          const SizedBox(width: 16),

          // Toggle video
          _CallControlButton(
            icon: isVideoEnabled ? Icons.videocam : Icons.videocam_off,
            label: isVideoEnabled ? 'Stop Video' : 'Start Video',
            onPressed: controller.toggleVideo,
            isActive: isVideoEnabled,
          ),
          const SizedBox(width: 16),

          // Screen share
          if (onScreenShare != null) ...[
            _CallControlButton(
              icon: Icons.screen_share,
              label: 'Share',
              onPressed: onScreenShare!,
            ),
            const SizedBox(width: 16),
          ],

          // End call
          if (onEndCall != null)
            _CallControlButton(
              icon: Icons.call_end,
              label: 'End',
              onPressed: onEndCall!,
              isDestructive: true,
            ),
        ],
      ),
    );
  }
}

class _CallControlButton extends StatelessWidget {
  final IconData icon;
  final String label;
  final VoidCallback onPressed;
  final bool isActive;
  final bool isDestructive;

  const _CallControlButton({
    required this.icon,
    required this.label,
    required this.onPressed,
    this.isActive = true,
    this.isDestructive = false,
  });

  @override
  Widget build(BuildContext context) {
    final backgroundColor = isDestructive
        ? CommunitasColors.error
        : (isActive ? CommunitasColors.jade : CommunitasColors.deepForest);

    return GestureDetector(
      onTap: onPressed,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: backgroundColor,
              shape: BoxShape.circle,
            ),
            child: Icon(
              icon,
              color: CommunitasColors.cream,
              size: 24,
            ),
          ),
          const SizedBox(height: 4),
          Text(
            label,
            style: TextStyle(
              color: CommunitasColors.cream.withValues(alpha: 0.9),
              fontSize: 11,
            ),
          ),
        ],
      ),
    );
  }
}
