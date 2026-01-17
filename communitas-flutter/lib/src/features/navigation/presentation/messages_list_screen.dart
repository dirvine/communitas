import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/router.dart';
import '../../../core/theme/colors.dart';
import '../../../shared/widgets/adaptive_layout.dart';
import '../../../shared/widgets/sidebar.dart';
import '../../../services/unified_data_provider.dart';
import '../../../services/navigation_state.dart';

class MessagesListScreen extends ConsumerWidget {
  const MessagesListScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final channelsAsync = ref.watch(unifiedChannelsProvider);
    final groupsAsync = ref.watch(unifiedGroupsProvider);

    return AdaptiveLayout(
      sidebar: const Sidebar(),
      body: Scaffold(
        appBar: AppBar(
          title: const Text('Messages'),
        ),
        body: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            _buildSectionHeader('Channels'),
            channelsAsync.when(
              loading: () => const _LoadingRow(),
              error: (e, _) => _ErrorRow(message: 'Failed to load channels: $e'),
              data: (channels) => _EntityList(
                entities: channels,
                onTap: (entity) {
                  ref
                      .read(recentEntitiesProvider.notifier)
                      .record(entityKey(entity.type, entity.id));
                  context.go(
                    '${Routes.entityChat.replaceAll(':type', entity.type).replaceAll(':id', entity.id)}',
                  );
                },
              ),
            ),
            const SizedBox(height: 24),
            _buildSectionHeader('Groups'),
            groupsAsync.when(
              loading: () => const _LoadingRow(),
              error: (e, _) => _ErrorRow(message: 'Failed to load groups: $e'),
              data: (groups) => _EntityList(
                entities: groups,
                onTap: (entity) {
                  ref
                      .read(recentEntitiesProvider.notifier)
                      .record(entityKey(entity.type, entity.id));
                  context.go(
                    '${Routes.entityChat.replaceAll(':type', entity.type).replaceAll(':id', entity.id)}',
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSectionHeader(String title) {
    return Text(
      title,
      style: const TextStyle(
        fontSize: 14,
        fontWeight: FontWeight.w600,
        color: CommunitasColors.cream,
      ),
    );
  }
}

class _EntityList extends StatelessWidget {
  final List<UnifiedEntity> entities;
  final void Function(UnifiedEntity entity) onTap;

  const _EntityList({
    required this.entities,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    if (entities.isEmpty) {
      return const Padding(
        padding: EdgeInsets.symmetric(vertical: 8),
        child: Text(
          'No items yet',
          style: TextStyle(color: CommunitasColors.cream),
        ),
      );
    }

    return Column(
      children: entities
          .map((entity) => ListTile(
                contentPadding: EdgeInsets.zero,
                leading: const Icon(Icons.chat_bubble_outline,
                    color: CommunitasColors.jade),
                title: Text(entity.name),
                subtitle: Text('${entity.memberCount} members'),
                onTap: () => onTap(entity),
              ))
          .toList(),
    );
  }
}

class _LoadingRow extends StatelessWidget {
  const _LoadingRow();

  @override
  Widget build(BuildContext context) {
    return const Padding(
      padding: EdgeInsets.symmetric(vertical: 8),
      child: LinearProgressIndicator(),
    );
  }
}

class _ErrorRow extends StatelessWidget {
  final String message;

  const _ErrorRow({required this.message});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Text(
        message,
        style: const TextStyle(color: CommunitasColors.error),
      ),
    );
  }
}
