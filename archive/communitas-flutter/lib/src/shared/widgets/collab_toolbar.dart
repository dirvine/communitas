import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../core/router.dart';
import '../../core/theme/colors.dart';

class CollabToolbar {
  static List<Widget> entityActions(
    BuildContext context, {
    required String entityType,
    required String entityId,
    VoidCallback? onVoice,
    VoidCallback? onVideo,
    VoidCallback? onShare,
  }) {
    final actions = <Widget>[];

    actions.addAll([
      IconButton(
        icon: const Icon(Icons.chat_bubble_outline),
        tooltip: 'Open chat',
        onPressed: () {
          context.go(
            Routes.entityChat
                .replaceAll(':type', entityType)
                .replaceAll(':id', entityId),
          );
        },
      ),
      IconButton(
        icon: const Icon(Icons.folder_open),
        tooltip: 'Open drive',
        onPressed: () {
          context.go(
            Routes.entityDrive
                .replaceAll(':type', entityType)
                .replaceAll(':id', entityId),
          );
        },
      ),
    ]);

    if (entityType == 'project') {
      actions.add(
        IconButton(
          icon: const Icon(Icons.view_kanban_outlined),
          tooltip: 'Open board',
          onPressed: () {
            context.go(
              Routes.projectBoard.replaceAll(':id', entityId),
            );
          },
        ),
      );
    }

    actions.add(
      IconButton(
        icon: const Icon(Icons.description_outlined),
        tooltip: 'Open docs (coming soon)',
        onPressed: () => _showComingSoon(context, 'Docs'),
      ),
    );

    actions.add(const SizedBox(width: 6));

    actions.addAll([
      IconButton(
        icon: const Icon(Icons.phone),
        tooltip: 'Voice call',
        onPressed: onVoice ?? () => _showComingSoon(context, 'Voice calls'),
      ),
      IconButton(
        icon: const Icon(Icons.videocam),
        tooltip: 'Video call',
        onPressed: onVideo ?? () => _showComingSoon(context, 'Video calls'),
      ),
      IconButton(
        icon: const Icon(Icons.screen_share),
        tooltip: 'Screen share',
        onPressed: onShare ?? () => _showComingSoon(context, 'Screen sharing'),
      ),
    ]);

    return actions;
  }

  static List<Widget> contactActions(
    BuildContext context, {
    required String contactId,
    VoidCallback? onVoice,
    VoidCallback? onVideo,
    VoidCallback? onShare,
  }) {
    return [
      IconButton(
        icon: const Icon(Icons.phone),
        tooltip: 'Voice call',
        onPressed: onVoice ?? () => _showComingSoon(context, 'Voice calls'),
      ),
      IconButton(
        icon: const Icon(Icons.videocam),
        tooltip: 'Video call',
        onPressed: onVideo ?? () => _showComingSoon(context, 'Video calls'),
      ),
      IconButton(
        icon: const Icon(Icons.screen_share),
        tooltip: 'Screen share',
        onPressed: onShare ?? () => _showComingSoon(context, 'Screen sharing'),
      ),
      IconButton(
        icon: const Icon(Icons.chat_bubble_outline),
        tooltip: 'Open chat',
        onPressed: () {
          context.go(
            Routes.contactChat.replaceAll(':fourWords', contactId),
          );
        },
      ),
    ];
  }

  static void _showComingSoon(BuildContext context, String label) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('$label are not yet available in the Flutter UI.'),
        backgroundColor: CommunitasColors.moss,
      ),
    );
  }
}
