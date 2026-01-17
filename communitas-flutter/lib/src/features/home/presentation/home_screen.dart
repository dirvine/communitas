import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/router.dart';
import '../../../core/theme/colors.dart';
import '../../../services/navigation_state.dart';
import '../../../services/unified_data_provider.dart';
import '../../../shared/widgets/adaptive_layout.dart';
import '../../../shared/widgets/sidebar.dart';

/// Home screen with adaptive layout (sidebar on desktop, bottom nav on mobile).
class HomeScreen extends ConsumerWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final identity = ref.watch(unifiedIdentityProvider);
    final orgsAsync = ref.watch(unifiedOrganizationsProvider);
    final groupsAsync = ref.watch(unifiedGroupsProvider);
    final projectsAsync = ref.watch(unifiedProjectsProvider);
    final channelsAsync = ref.watch(unifiedChannelsProvider);
    final contactsAsync = ref.watch(unifiedContactsProvider);
    final overrides = ref.watch(organizationCategoryOverridesProvider);

    final communitiesAsync = orgsAsync.whenData(
      (orgs) => _filterOrganizations(orgs, overrides, OrganizationCategory.community),
    );
    final organizationsAsync = orgsAsync.whenData(
      (orgs) => _filterOrganizations(orgs, overrides, OrganizationCategory.organization),
    );

    final personalGroupsAsync = groupsAsync.whenData(
      (groups) => groups
          .where((group) => group.parentId == null || group.parentId!.isEmpty)
          .toList(),
    );

    return AdaptiveLayout(
      sidebar: const Sidebar(),
      body: Scaffold(
        appBar: AppBar(
          title: const Text('Home'),
          actions: [
            IconButton(
              icon: const Icon(Icons.notifications_outlined),
              onPressed: () {},
            ),
            IconButton(
              icon: const Icon(Icons.settings_outlined),
              onPressed: () {},
            ),
          ],
        ),
        body: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _buildWelcomeCard(context, identity.displayName),
              const SizedBox(height: 32),
              Text(
                'Overview',
                style: Theme.of(context).textTheme.titleLarge,
              ),
              const SizedBox(height: 16),
              Wrap(
                spacing: 16,
                runSpacing: 16,
                children: [
                  _buildCountCard(
                    context,
                    asyncItems: organizationsAsync,
                    icon: Icons.business,
                    label: 'Organizations',
                    color: CommunitasColors.organization,
                  ),
                  _buildCountCard(
                    context,
                    asyncItems: communitiesAsync,
                    icon: Icons.public,
                    label: 'Communities',
                    color: CommunitasColors.jade,
                  ),
                  _buildCountCard(
                    context,
                    asyncItems: projectsAsync,
                    icon: Icons.folder_outlined,
                    label: 'Projects',
                    color: CommunitasColors.project,
                  ),
                  _buildCountCard(
                    context,
                    asyncItems: groupsAsync,
                    icon: Icons.group_outlined,
                    label: 'Groups',
                    color: CommunitasColors.group,
                  ),
                  _buildCountCard(
                    context,
                    asyncItems: channelsAsync,
                    icon: Icons.tag,
                    label: 'Channels',
                    color: CommunitasColors.channel,
                  ),
                  _buildCountCard(
                    context,
                    asyncItems: contactsAsync,
                    icon: Icons.people_outline,
                    label: 'Contacts',
                    color: CommunitasColors.person,
                  ),
                ],
              ),
              const SizedBox(height: 32),
              Text(
                'Your Spaces',
                style: Theme.of(context).textTheme.titleLarge,
              ),
              const SizedBox(height: 16),
              Wrap(
                spacing: 16,
                runSpacing: 16,
                children: [
                  _buildPersonalCard(
                    context,
                    ref: ref,
                    groupsAsync: personalGroupsAsync,
                    contactsAsync: contactsAsync,
                  ),
                  _buildEntityCard(
                    context,
                    ref: ref,
                    title: 'Communities',
                    asyncItems: communitiesAsync,
                    icon: Icons.public,
                    accent: CommunitasColors.jade,
                    emptyLabel: 'No communities yet',
                  ),
                  _buildEntityCard(
                    context,
                    ref: ref,
                    title: 'Organizations',
                    asyncItems: organizationsAsync,
                    icon: Icons.business,
                    accent: CommunitasColors.organization,
                    emptyLabel: 'No organizations yet',
                  ),
                  _buildEntityCard(
                    context,
                    ref: ref,
                    title: 'Projects',
                    asyncItems: projectsAsync,
                    icon: Icons.folder,
                    accent: CommunitasColors.project,
                    emptyLabel: 'No projects yet',
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildWelcomeCard(BuildContext context, String displayName) {
    return Container(
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          colors: [
            CommunitasColors.jade.withOpacity(0.2),
            CommunitasColors.moss,
          ],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ),
        borderRadius: BorderRadius.circular(16),
      ),
      child: Row(
        children: [
          const Icon(
            Icons.waving_hand,
            size: 48,
            color: CommunitasColors.amber,
          ),
          const SizedBox(width: 24),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Welcome back, $displayName',
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
                const SizedBox(height: 4),
                Text(
                  'Your local-first collaboration hub',
                  style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                        color: CommunitasColors.cream.withOpacity(0.7),
                      ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildCountCard<T>(
    BuildContext context, {
    required AsyncValue<List<T>> asyncItems,
    required IconData icon,
    required String label,
    required Color color,
  }) {
    return asyncItems.when(
      loading: () => _buildStatCard(context, icon: icon, label: label, value: '—', color: color),
      error: (_, __) => _buildStatCard(context, icon: icon, label: label, value: '!', color: color),
      data: (items) => _buildStatCard(
        context,
        icon: icon,
        label: label,
        value: items.length.toString(),
        color: color,
      ),
    );
  }

  Widget _buildStatCard(
    BuildContext context, {
    required IconData icon,
    required String label,
    required String value,
    required Color color,
  }) {
    return Container(
      width: 160,
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: CommunitasColors.moss,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(icon, color: color, size: 32),
          const SizedBox(height: 12),
          Text(
            value,
            style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                  color: color,
                  fontWeight: FontWeight.bold,
                ),
          ),
          Text(
            label,
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: CommunitasColors.cream.withOpacity(0.7),
                ),
          ),
        ],
      ),
    );
  }

  Widget _buildPersonalCard(
    BuildContext context, {
    required WidgetRef ref,
    required AsyncValue<List<UnifiedEntity>> groupsAsync,
    required AsyncValue<List<UnifiedContact>> contactsAsync,
  }) {
    return _buildCardContainer(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildCardHeader('Personal', Icons.person, CommunitasColors.person),
          const SizedBox(height: 12),
          _buildSublistHeader('Groups', onTap: () => context.go(Routes.messages)),
          _buildEntitySublist(
            context,
            ref: ref,
            asyncItems: groupsAsync,
            emptyLabel: 'No groups yet',
            onTap: (entity) => context.go(
              Routes.entityDetail
                  .replaceAll(':type', entity.type)
                  .replaceAll(':id', entity.id),
            ),
          ),
          const SizedBox(height: 12),
          _buildSublistHeader('Contacts', onTap: () => context.go(Routes.contacts)),
          _buildContactSublist(
            context,
            ref: ref,
            asyncItems: contactsAsync,
            emptyLabel: 'No contacts yet',
          ),
        ],
      ),
    );
  }

  Widget _buildEntityCard(
    BuildContext context, {
    required WidgetRef ref,
    required String title,
    required AsyncValue<List<UnifiedEntity>> asyncItems,
    required IconData icon,
    required Color accent,
    required String emptyLabel,
  }) {
    return _buildCardContainer(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildCardHeader(title, icon, accent),
          const SizedBox(height: 12),
          _buildEntitySublist(
            context,
            ref: ref,
            asyncItems: asyncItems,
            emptyLabel: emptyLabel,
            onTap: (entity) => context.go(
              Routes.entityDetail
                  .replaceAll(':type', entity.type)
                  .replaceAll(':id', entity.id),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildCardContainer({required Widget child}) {
    return Container(
      width: 320,
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: CommunitasColors.moss,
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: CommunitasColors.fern.withOpacity(0.6)),
      ),
      child: child,
    );
  }

  Widget _buildCardHeader(String title, IconData icon, Color accent) {
    return Row(
      children: [
        Container(
          width: 32,
          height: 32,
          decoration: BoxDecoration(
            color: accent.withOpacity(0.15),
            borderRadius: BorderRadius.circular(8),
          ),
          child: Icon(icon, color: accent, size: 18),
        ),
        const SizedBox(width: 12),
        Text(
          title,
          style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
        ),
      ],
    );
  }

  Widget _buildSublistHeader(String label, {VoidCallback? onTap}) {
    return Row(
      children: [
        Text(
          label,
          style: TextStyle(
            fontSize: 12,
            color: CommunitasColors.cream.withOpacity(0.7),
            fontWeight: FontWeight.w600,
          ),
        ),
        const Spacer(),
        if (onTap != null)
          TextButton(
            onPressed: onTap,
            child: const Text('View all'),
          ),
      ],
    );
  }

  Widget _buildEntitySublist(
    BuildContext context, {
    required WidgetRef ref,
    required AsyncValue<List<UnifiedEntity>> asyncItems,
    required String emptyLabel,
    required void Function(UnifiedEntity entity) onTap,
  }) {
    return asyncItems.when(
      loading: () => const Padding(
        padding: EdgeInsets.symmetric(vertical: 8),
        child: LinearProgressIndicator(),
      ),
      error: (e, _) => Padding(
        padding: const EdgeInsets.symmetric(vertical: 8),
        child: Text(
          'Failed to load: $e',
          style: const TextStyle(color: CommunitasColors.error, fontSize: 12),
        ),
      ),
      data: (items) {
        if (items.isEmpty) {
          return Padding(
            padding: const EdgeInsets.symmetric(vertical: 8),
            child: Text(
              emptyLabel,
              style: TextStyle(color: CommunitasColors.cream.withOpacity(0.6), fontSize: 12),
            ),
          );
        }
        final displayItems = items.take(4).toList();
        return Column(
          children: displayItems
              .map(
                (entity) => InkWell(
                  onTap: () {
                    ref
                        .read(recentEntitiesProvider.notifier)
                        .record(entityKey(entity.type, entity.id));
                    onTap(entity);
                  },
                  child: Padding(
                    padding: const EdgeInsets.symmetric(vertical: 6),
                    child: Row(
                      children: [
                        Icon(
                          _entityIcon(entity.type),
                          size: 16,
                          color: CommunitasColors.cream.withOpacity(0.7),
                        ),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            entity.name,
                            style: const TextStyle(fontSize: 13),
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                        Text(
                          '${entity.memberCount}',
                          style: TextStyle(
                            fontSize: 11,
                            color: CommunitasColors.cream.withOpacity(0.5),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              )
              .toList(),
        );
      },
    );
  }

  Widget _buildContactSublist(
    BuildContext context, {
    required WidgetRef ref,
    required AsyncValue<List<UnifiedContact>> asyncItems,
    required String emptyLabel,
  }) {
    return asyncItems.when(
      loading: () => const Padding(
        padding: EdgeInsets.symmetric(vertical: 8),
        child: LinearProgressIndicator(),
      ),
      error: (e, _) => Padding(
        padding: const EdgeInsets.symmetric(vertical: 8),
        child: Text(
          'Failed to load: $e',
          style: const TextStyle(color: CommunitasColors.error, fontSize: 12),
        ),
      ),
      data: (items) {
        if (items.isEmpty) {
          return Padding(
            padding: const EdgeInsets.symmetric(vertical: 8),
            child: Text(
              emptyLabel,
              style: TextStyle(color: CommunitasColors.cream.withOpacity(0.6), fontSize: 12),
            ),
          );
        }
        final displayItems = items.take(4).toList();
        return Column(
          children: displayItems
              .map(
                (contact) => InkWell(
                  onTap: () {
                    ref
                        .read(recentContactsProvider.notifier)
                        .record(contact.pubkeyHex);
                    context.go(
                      Routes.contactChat.replaceAll(':fourWords', contact.pubkeyHex),
                    );
                  },
                  child: Padding(
                    padding: const EdgeInsets.symmetric(vertical: 6),
                    child: Row(
                      children: [
                        CircleAvatar(
                          radius: 10,
                          backgroundColor: CommunitasColors.person,
                          child: Text(
                            contact.displayName.isNotEmpty
                                ? contact.displayName[0].toUpperCase()
                                : '?',
                            style: const TextStyle(
                              color: CommunitasColors.cream,
                              fontSize: 10,
                            ),
                          ),
                        ),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            contact.displayName,
                            style: const TextStyle(fontSize: 13),
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                        Container(
                          width: 8,
                          height: 8,
                          decoration: BoxDecoration(
                            color: CommunitasColors.statusColor(contact.status),
                            shape: BoxShape.circle,
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              )
              .toList(),
        );
      },
    );
  }

  static List<UnifiedEntity> _filterOrganizations(
    List<UnifiedEntity> orgs,
    Map<String, OrganizationCategory> overrides,
    OrganizationCategory category,
  ) {
    return orgs
        .where(
          (org) => resolveOrganizationCategory(org, overrides) == category,
        )
        .toList();
  }

  IconData _entityIcon(String type) {
    switch (type) {
      case 'organisation':
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
}
