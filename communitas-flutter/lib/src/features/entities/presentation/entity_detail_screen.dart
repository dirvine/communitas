import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/colors.dart';
import '../../../shared/widgets/sidebar.dart';
import '../../../shared/widgets/adaptive_layout.dart';
import '../../../services/unified_data_provider.dart';

/// Entity detail screen with tabs (Chat, Drive, Board, Documents, Details).
class EntityDetailScreen extends ConsumerWidget {
  final String entityType;
  final String entityId;

  const EntityDetailScreen({
    super.key,
    required this.entityType,
    required this.entityId,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final entityAsync = ref.watch(unifiedEntityByIdProvider((type: entityType, id: entityId)));

    return AdaptiveLayout(
      sidebar: const Sidebar(),
      body: DefaultTabController(
        length: entityType == 'project' ? 5 : 4,
        child: Scaffold(
          appBar: AppBar(
            title: entityAsync.when(
              loading: () => const Text('Loading...'),
              error: (_, __) => Text('Entity $entityId'),
              data: (entity) => Text(entity?.name ?? 'Unknown Entity'),
            ),
            bottom: TabBar(
              tabs: [
                const Tab(icon: Icon(Icons.chat), text: 'Chat'),
                const Tab(icon: Icon(Icons.folder), text: 'Drive'),
                if (entityType == 'project')
                  const Tab(icon: Icon(Icons.view_kanban), text: 'Board'),
                const Tab(icon: Icon(Icons.description), text: 'Docs'),
                const Tab(icon: Icon(Icons.info_outline), text: 'Details'),
              ],
            ),
          ),
          body: TabBarView(
            children: [
              _buildChatTab(),
              _buildDriveTab(),
              if (entityType == 'project') _buildBoardTab(),
              _buildDocsTab(),
              _buildDetailsTab(ref),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildChatTab() {
    return const Center(
      child: Text('Chat content here'),
    );
  }

  Widget _buildDriveTab() {
    return const Center(
      child: Text('Drive content here'),
    );
  }

  Widget _buildBoardTab() {
    return const Center(
      child: Text('Kanban board here'),
    );
  }

  Widget _buildDocsTab() {
    return const Center(
      child: Text('Documents here'),
    );
  }

  Widget _buildDetailsTab(WidgetRef ref) {
    final entityAsync = ref.watch(unifiedEntityByIdProvider((type: entityType, id: entityId)));

    return entityAsync.when(
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (e, _) => Center(child: Text('Error: $e')),
      data: (entity) {
        final name = entity?.name ?? 'Unknown Entity';
        final description = entity?.description ?? 'No description';
        final memberCount = entity?.memberCount ?? 0;
        final role = entity?.role ?? 'member';

        return SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Container(
                padding: const EdgeInsets.all(24),
                decoration: BoxDecoration(
                  color: CommunitasColors.moss,
                  borderRadius: BorderRadius.circular(16),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Container(
                          width: 64,
                          height: 64,
                          decoration: BoxDecoration(
                            color: CommunitasColors.entityColor(entityType),
                            borderRadius: BorderRadius.circular(12),
                          ),
                          child: Icon(
                            _getEntityIcon(),
                            color: CommunitasColors.cream,
                            size: 32,
                          ),
                        ),
                        const SizedBox(width: 16),
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                name,
                                style: const TextStyle(
                                  fontSize: 20,
                                  fontWeight: FontWeight.bold,
                                ),
                              ),
                              const SizedBox(height: 4),
                              Text(
                                description,
                                style: const TextStyle(
                                  color: CommunitasColors.jade,
                                ),
                              ),
                            ],
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 16),
                    Row(
                      children: [
                        _buildInfoChip(Icons.people, '$memberCount members'),
                        const SizedBox(width: 12),
                        _buildInfoChip(_getRoleIcon(role), role),
                      ],
                    ),
                  ],
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildInfoChip(IconData icon, String label) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      decoration: BoxDecoration(
        color: CommunitasColors.fern.withAlpha(128),
        borderRadius: BorderRadius.circular(16),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 14, color: CommunitasColors.cream),
          const SizedBox(width: 6),
          Text(
            label,
            style: const TextStyle(
              fontSize: 12,
              color: CommunitasColors.cream,
            ),
          ),
        ],
      ),
    );
  }

  IconData _getEntityIcon() {
    switch (entityType) {
      case 'organization':
        return Icons.business;
      case 'project':
        return Icons.folder;
      case 'channel':
        return Icons.tag;
      case 'group':
        return Icons.group;
      default:
        return Icons.folder;
    }
  }

  IconData _getRoleIcon(String role) {
    switch (role) {
      case 'owner':
        return Icons.workspace_premium;
      case 'admin':
        return Icons.shield;
      case 'member':
        return Icons.person;
      case 'guest':
        return Icons.visibility;
      default:
        return Icons.person;
    }
  }
}
