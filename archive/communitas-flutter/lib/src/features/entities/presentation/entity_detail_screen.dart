import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/router.dart';
import '../../../core/theme/colors.dart';
import '../../../shared/widgets/sidebar.dart';
import '../../../shared/widgets/adaptive_layout.dart';
import '../../../shared/widgets/collab_toolbar.dart';
import '../../../services/navigation_state.dart';
import '../../../services/unified_data_provider.dart';

/// Entity detail screen with quick actions (Chat, Drive, Board).
class EntityDetailScreen extends ConsumerStatefulWidget {
  final String entityType;
  final String entityId;

  const EntityDetailScreen({
    super.key,
    required this.entityType,
    required this.entityId,
  });

  @override
  ConsumerState<EntityDetailScreen> createState() => _EntityDetailScreenState();
}

class _EntityDetailScreenState extends ConsumerState<EntityDetailScreen> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(recentEntitiesProvider.notifier).record(
            entityKey(widget.entityType, widget.entityId),
          );
    });
  }

  @override
  Widget build(BuildContext context) {
    final entityAsync = ref.watch(
      unifiedEntityByIdProvider((type: widget.entityType, id: widget.entityId)),
    );

    return AdaptiveLayout(
      sidebar: const Sidebar(),
      body: Scaffold(
        appBar: AppBar(
          title: entityAsync.when(
            loading: () => const Text('Loading...'),
            error: (_, __) => Text('Entity ${widget.entityId}'),
            data: (entity) => Text(entity?.name ?? 'Unknown Entity'),
          ),
          actions: CollabToolbar.entityActions(
            context,
            entityType: widget.entityType,
            entityId: widget.entityId,
          ),
        ),
        body: _buildDetailsBody(context, ref),
      ),
    );
  }

  Widget _buildDetailsBody(BuildContext context, WidgetRef ref) {
    final entityAsync = ref.watch(
      unifiedEntityByIdProvider((type: widget.entityType, id: widget.entityId)),
    );

    return entityAsync.when(
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (e, _) => Center(child: Text('Error: $e')),
      data: (entity) {
        final name = entity?.name ?? 'Unknown Entity';
        final description = entity?.description ?? 'No description';
        final memberCount = entity?.memberCount ?? 0;
        final role = entity?.role ?? 'member';
        final overrides = ref.watch(organizationCategoryOverridesProvider);
        final isOrg = entity?.type == 'organisation' || entity?.type == 'organization';
        final category = (entity != null && isOrg)
            ? resolveOrganizationCategory(entity, overrides)
            : null;

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
                            color: CommunitasColors.entityColor(widget.entityType),
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
                        if (category != null) ...[
                          const SizedBox(width: 12),
                          _buildInfoChip(
                            category == OrganizationCategory.community
                                ? Icons.public
                                : Icons.business,
                            category == OrganizationCategory.community
                                ? 'Community'
                                : 'Organization',
                          ),
                        ],
                      ],
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 24),
              Text(
                'Actions',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 12),
              _buildActionTile(
                context,
                icon: Icons.chat,
                label: 'Open Chat',
                onTap: () => context.go(
                  Routes.entityChat
                      .replaceAll(':type', widget.entityType)
                      .replaceAll(':id', widget.entityId),
                ),
              ),
              _buildActionTile(
                context,
                icon: Icons.folder,
                label: 'Open Drive',
                onTap: () => context.go(
                  Routes.entityDrive
                      .replaceAll(':type', widget.entityType)
                      .replaceAll(':id', widget.entityId),
                ),
              ),
              if (widget.entityType == 'project')
                _buildActionTile(
                  context,
                  icon: Icons.view_kanban,
                  label: 'Open Kanban Board',
                  onTap: () => context.go(
                    Routes.projectBoard.replaceAll(':id', widget.entityId),
                  ),
                ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildActionTile(
    BuildContext context, {
    required IconData icon,
    required String label,
    required VoidCallback onTap,
  }) {
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      decoration: BoxDecoration(
        color: CommunitasColors.moss,
        borderRadius: BorderRadius.circular(12),
      ),
      child: ListTile(
        leading: Icon(icon, color: CommunitasColors.jade),
        title: Text(label),
        trailing: const Icon(Icons.chevron_right),
        onTap: onTap,
      ),
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
    switch (widget.entityType) {
      case 'organization':
      case 'organisation':
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
