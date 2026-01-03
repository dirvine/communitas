import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/theme/colors.dart';
import '../../../demo/demo_data.dart';

/// Active voice/video call screen.
class ActiveCallScreen extends ConsumerStatefulWidget {
  final String fourWords;

  const ActiveCallScreen({
    super.key,
    required this.fourWords,
  });

  @override
  ConsumerState<ActiveCallScreen> createState() => _ActiveCallScreenState();
}

class _ActiveCallScreenState extends ConsumerState<ActiveCallScreen> {
  bool _isMuted = false;
  bool _isVideoOn = true;
  bool _isSpeakerOn = true;

  DemoContact? get contact {
    try {
      return DemoData.contacts.firstWhere(
        (c) => c.fourWords == widget.fourWords,
      );
    } catch (_) {
      return null;
    }
  }

  @override
  Widget build(BuildContext context) {
    final contactData = contact;

    return Scaffold(
      backgroundColor: CommunitasColors.deepForest,
      body: Stack(
        children: [
          // Video area (placeholder)
          Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Container(
                  width: 120,
                  height: 120,
                  decoration: BoxDecoration(
                    color: CommunitasColors.jade,
                    borderRadius: BorderRadius.circular(60),
                  ),
                  child: Center(
                    child: Text(
                      contactData?.displayName[0].toUpperCase() ?? '?',
                      style: const TextStyle(
                        fontSize: 48,
                        fontWeight: FontWeight.bold,
                        color: CommunitasColors.cream,
                      ),
                    ),
                  ),
                ),
                const SizedBox(height: 24),
                Text(
                  contactData?.displayName ?? 'Unknown',
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
                const SizedBox(height: 8),
                Text(
                  widget.fourWords,
                  style: TextStyle(
                    color: CommunitasColors.jade,
                    fontSize: 14,
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  '00:42', // Call duration placeholder
                  style: TextStyle(
                    color: CommunitasColors.cream.withOpacity(0.7),
                  ),
                ),
              ],
            ),
          ),

          // Self video preview (placeholder)
          Positioned(
            bottom: 120,
            right: 24,
            child: Container(
              width: 100,
              height: 150,
              decoration: BoxDecoration(
                color: CommunitasColors.moss,
                borderRadius: BorderRadius.circular(12),
                border: Border.all(color: CommunitasColors.fern),
              ),
              child: const Center(
                child: Icon(
                  Icons.person,
                  size: 48,
                  color: CommunitasColors.jade,
                ),
              ),
            ),
          ),

          // Call controls
          Positioned(
            bottom: 0,
            left: 0,
            right: 0,
            child: Container(
              padding: const EdgeInsets.all(24),
              decoration: BoxDecoration(
                gradient: LinearGradient(
                  begin: Alignment.topCenter,
                  end: Alignment.bottomCenter,
                  colors: [
                    Colors.transparent,
                    CommunitasColors.deepForest,
                  ],
                ),
              ),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                children: [
                  // Mute
                  _buildControlButton(
                    icon: _isMuted ? Icons.mic_off : Icons.mic,
                    label: _isMuted ? 'Unmute' : 'Mute',
                    isActive: !_isMuted,
                    onTap: () => setState(() => _isMuted = !_isMuted),
                  ),

                  // Video
                  _buildControlButton(
                    icon: _isVideoOn ? Icons.videocam : Icons.videocam_off,
                    label: _isVideoOn ? 'Stop Video' : 'Start Video',
                    isActive: _isVideoOn,
                    onTap: () => setState(() => _isVideoOn = !_isVideoOn),
                  ),

                  // End call
                  _buildControlButton(
                    icon: Icons.call_end,
                    label: 'End',
                    backgroundColor: CommunitasColors.error,
                    onTap: () => context.pop(),
                  ),

                  // Screen share
                  _buildControlButton(
                    icon: Icons.screen_share,
                    label: 'Share',
                    isActive: false,
                    onTap: () {},
                  ),

                  // Speaker
                  _buildControlButton(
                    icon: _isSpeakerOn ? Icons.volume_up : Icons.volume_off,
                    label: _isSpeakerOn ? 'Speaker' : 'Earpiece',
                    isActive: _isSpeakerOn,
                    onTap: () => setState(() => _isSpeakerOn = !_isSpeakerOn),
                  ),
                ],
              ),
            ),
          ),

          // Back button
          Positioned(
            top: MediaQuery.of(context).padding.top + 16,
            left: 16,
            child: IconButton(
              icon: const Icon(Icons.arrow_back),
              onPressed: () => context.pop(),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildControlButton({
    required IconData icon,
    required String label,
    required VoidCallback onTap,
    bool isActive = true,
    Color? backgroundColor,
  }) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Material(
          color: backgroundColor ??
              (isActive ? CommunitasColors.moss : CommunitasColors.fern),
          borderRadius: BorderRadius.circular(28),
          child: InkWell(
            onTap: onTap,
            borderRadius: BorderRadius.circular(28),
            child: Container(
              width: 56,
              height: 56,
              alignment: Alignment.center,
              child: Icon(
                icon,
                color: backgroundColor != null
                    ? CommunitasColors.cream
                    : (isActive
                        ? CommunitasColors.cream
                        : CommunitasColors.cream.withOpacity(0.5)),
              ),
            ),
          ),
        ),
        const SizedBox(height: 8),
        Text(
          label,
          style: TextStyle(
            fontSize: 12,
            color: CommunitasColors.cream.withOpacity(0.7),
          ),
        ),
      ],
    );
  }
}
