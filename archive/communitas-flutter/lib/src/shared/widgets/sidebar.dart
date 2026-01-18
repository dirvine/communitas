import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../core/router.dart';
import '../../core/theme/colors.dart';
import '../../services/unified_data_provider.dart';
import '../../services/navigation_state.dart';
import '../../services/ffi_provider.dart';
import '../../bindings/api_exports.dart';
import '../../features/auth/providers/auth_provider.dart';
import 'quick_switcher_dialog.dart';

/// Main navigation sidebar for desktop layout.
///
/// Organized according to APP_SPECIFICATION.md:
/// - Profile Header
/// - My Organizations (role = owner)
/// - My Communities (role != owner)
/// - Personal (isPersonal = true)
/// - Direct Messages (contacts)
class Sidebar extends ConsumerStatefulWidget {
  const Sidebar({super.key});

  @override
  ConsumerState<Sidebar> createState() => _SidebarState();
}

class _SidebarState extends ConsumerState<Sidebar> {
  // Track expanded state for each section
  final Map<String, bool> _expandedSections = {
    'recents': true,
    'starred': true,
    'personal': true,
    'communities': true,
    'organizations': true,
  };

  @override
  Widget build(BuildContext context) {
    final starredKeys = ref.watch(starredEntitiesProvider);

    return Material(
      type: MaterialType.canvas,
      color: CommunitasColors.moss,
      child: DefaultTextStyle(
        style: const TextStyle(
          color: CommunitasColors.cream,
          fontSize: 14,
          fontWeight: FontWeight.normal,
          decoration: TextDecoration.none,
          fontFamily: 'Roboto',
          inherit: false,
        ),
        child: SizedBox(
          width: 280,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // User header
              _buildUserHeader(context),
              Container(height: 1, color: CommunitasColors.fern),

              // Scrollable content
              Expanded(
                child: SingleChildScrollView(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      // Quick nav
                      _navItem(context, Icons.home, 'Home', Routes.home, isQuickNav: true),
                      _navItem(context, Icons.lan, 'Network', Routes.network, isQuickNav: true),
                      _navActionItem(
                        context,
                        Icons.search,
                        'Quick switcher',
                        onTap: () => _showQuickSwitcher(context),
                        shortcutHint: 'Cmd/Ctrl+K',
                      ),
                      Container(height: 1, color: CommunitasColors.fern),

                      // Recents
                      _buildSection(
                        'Recents',
                        'recents',
                        _buildRecents(),
                        onAdd: _clearRecents,
                        actionIcon: Icons.clear_all,
                        actionTooltip: 'Clear recents',
                      ),

                      // Starred
                      _buildSection(
                        'Starred',
                        'starred',
                        _buildStarred(),
                      ),

                      // Personal Space
                      _buildSection(
                        'Personal',
                        'personal',
                      _buildPersonalSpace(starredKeys),
                        onAdd: () => _showCreateEntityDialog(
                          context,
                          FlutterEntityType.group,
                        ),
                      ),

                      // Communities (non-commercial)
                      _buildSection(
                        'Communities',
                        'communities',
                        _buildCommunities(starredKeys),
                        onAdd: () => _showCreateEntityDialog(
                          context,
                          FlutterEntityType.organisation,
                        ),
                      ),

                      // Organizations (commercial)
                      _buildSection(
                        'Organizations',
                        'organizations',
                        _buildOrganizations(starredKeys),
                        onAdd: () => _showCreateEntityDialog(
                          context,
                          FlutterEntityType.organisation,
                        ),
                      ),

                    ],
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildSection(
    String title,
    String sectionKey,
    Widget child, {
    VoidCallback? onAdd,
    IconData actionIcon = Icons.add,
    String? actionTooltip,
  }) {
    final isExpanded = _expandedSections[sectionKey] ?? true;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        GestureDetector(
          onTap: () {
            setState(() {
              _expandedSections[sectionKey] = !isExpanded;
            });
          },
          behavior: HitTestBehavior.opaque,
          child: Container(
            padding: const EdgeInsets.fromLTRB(16, 12, 16, 8),
            child: Row(
              children: [
                Icon(
                  isExpanded ? Icons.expand_more : Icons.chevron_right,
                  color: CommunitasColors.cream.withAlpha(179),
                  size: 18,
                ),
                const SizedBox(width: 4),
                Text(
                  title,
                  style: TextStyle(
                    color: CommunitasColors.cream.withAlpha(179),
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    letterSpacing: 0.5,
                    decoration: TextDecoration.none,
                    fontFamily: 'Roboto',
                    inherit: false,
                  ),
                ),
                const Spacer(),
                if (onAdd != null)
                  IconButton(
                    visualDensity: VisualDensity.compact,
                    tooltip: actionTooltip,
                    onPressed: onAdd,
                    icon: Icon(
                      actionIcon,
                      color: CommunitasColors.cream.withAlpha(128),
                      size: 16,
                    ),
                  ),
              ],
            ),
          ),
        ),
        if (isExpanded) child,
        const SizedBox(height: 8),
      ],
    );
  }

  Widget _buildOrganizations(Set<String> starredKeys) {
    final orgsAsync = ref.watch(unifiedOrganizationsProvider);
    final overrides = ref.watch(organizationCategoryOverridesProvider);

    return orgsAsync.when(
      loading: () => const Padding(
        padding: EdgeInsets.all(16),
        child: SizedBox(
          height: 20,
          width: 20,
          child: CircularProgressIndicator(strokeWidth: 2),
        ),
      ),
      error: (e, _) => Padding(
        padding: const EdgeInsets.all(16),
        child: Text('Error: $e', style: const TextStyle(color: CommunitasColors.error, fontSize: 12)),
      ),
      data: (orgs) {
        final organizations = orgs.where(
          (org) => resolveOrganizationCategory(org, overrides) == OrganizationCategory.organization,
        ).toList();
        if (organizations.isEmpty) {
          return _buildEmptySection('No organizations yet');
        }

        return Column(
          children: organizations.map((org) => _buildEntityItemAsync(
            context: context,
            entity: org,
            parentId: org.id,
            isStarred: starredKeys.contains(entityKey(org.type, org.id)),
          )).toList(),
        );
      },
    );
  }

  Widget _buildCommunities(Set<String> starredKeys) {
    final orgsAsync = ref.watch(unifiedOrganizationsProvider);
    final overrides = ref.watch(organizationCategoryOverridesProvider);

    return orgsAsync.when(
      loading: () => const Padding(
        padding: EdgeInsets.all(16),
        child: SizedBox(
          height: 20,
          width: 20,
          child: CircularProgressIndicator(strokeWidth: 2),
        ),
      ),
      error: (e, _) => Padding(
        padding: const EdgeInsets.all(16),
        child: Text('Error: $e', style: const TextStyle(color: CommunitasColors.error, fontSize: 12)),
      ),
      data: (orgs) {
        final communities = orgs.where(
          (org) => resolveOrganizationCategory(org, overrides) == OrganizationCategory.community,
        ).toList();
        if (communities.isEmpty) {
          return _buildEmptySection('No communities yet');
        }

        return Column(
          children: communities.map((org) => _buildEntityItemAsync(
            context: context,
            entity: org,
            parentId: org.id,
            isStarred: starredKeys.contains(entityKey(org.type, org.id)),
          )).toList(),
        );
      },
    );
  }

  Widget _buildPersonalSpace(Set<String> starredKeys) {
    final groupsAsync = ref.watch(unifiedGroupsProvider);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildSubsectionHeader('Groups'),
        groupsAsync.when(
          loading: () => const Padding(
            padding: EdgeInsets.all(16),
            child: SizedBox(
              height: 20,
              width: 20,
              child: CircularProgressIndicator(strokeWidth: 2),
            ),
          ),
          error: (e, _) => Padding(
            padding: const EdgeInsets.all(16),
            child: Text('Error: $e', style: const TextStyle(color: CommunitasColors.error, fontSize: 12)),
          ),
          data: (groups) {
            final personalGroups = groups
                .where((group) => group.parentId == null || group.parentId!.isEmpty)
                .toList();
            if (personalGroups.isEmpty) {
              return _buildEmptySection('No personal groups yet');
            }
            return Column(
              children: personalGroups.map((group) => _buildEntityItemUnified(
                context: context,
                entity: group,
                children: [],
                isStarred: starredKeys.contains(entityKey(group.type, group.id)),
              )).toList(),
            );
          },
        ),
        const SizedBox(height: 8),
        _buildSubsectionHeader(
          'Contacts',
          onAdd: () => _showCreateContactDialog(context),
        ),
        _buildDirectMessages(),
      ],
    );
  }

  Widget _buildDirectMessages() {
    final contactsAsync = ref.watch(unifiedContactsProvider);
    final starredContacts = ref.watch(starredContactsProvider);

    return contactsAsync.when(
      loading: () => const Padding(
        padding: EdgeInsets.all(16),
        child: SizedBox(
          height: 20,
          width: 20,
          child: CircularProgressIndicator(strokeWidth: 2),
        ),
      ),
      error: (e, _) => Padding(
        padding: const EdgeInsets.all(16),
        child: Text('Error: $e', style: const TextStyle(color: CommunitasColors.error, fontSize: 12)),
      ),
      data: (contacts) {
        if (contacts.isEmpty) {
          return _buildEmptySection('No contacts yet');
        }
        return Column(
          children: contacts.map((contact) => _buildContactItemUnified(
            context,
            contact,
            isStarred: starredContacts.contains(contact.pubkeyHex),
          )).toList(),
        );
      },
    );
  }

  /// Build entity item with async child loading (FFI/demo).
  Widget _buildEntityItemAsync({
    required BuildContext context,
    required UnifiedEntity entity,
    required String parentId,
    required bool isStarred,
  }) {
    final icon = _getEntityIcon(entity.type);
    final color = _getEntityColor(entity.type);
    final route = '/entity/${entity.type}/${entity.id}';

    // Watch for child entities
    final childrenAsync = ref.watch(unifiedChildEntitiesProvider(parentId));

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        GestureDetector(
          onTap: () {
            _recordRecentEntity(entity);
            context.go(route);
          },
          onSecondaryTapDown: (details) => _showEntityMenu(entity, details.globalPosition),
          onLongPressStart: (details) => _showEntityMenu(entity, details.globalPosition),
          behavior: HitTestBehavior.opaque,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
            child: Row(
              children: [
                Container(
                  width: 28,
                  height: 28,
                  decoration: BoxDecoration(
                    color: color.withAlpha(51),
                    borderRadius: BorderRadius.circular(6),
                  ),
                  child: Icon(icon, color: color, size: 16),
                ),
                const SizedBox(width: 10),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        entity.name,
                        style: const TextStyle(
                          color: CommunitasColors.cream,
                          fontSize: 13,
                          fontWeight: FontWeight.w500,
                          decoration: TextDecoration.none,
                          fontFamily: 'Roboto',
                          inherit: false,
                        ),
                        overflow: TextOverflow.ellipsis,
                      ),
                      const SizedBox(height: 2),
                      Text(
                        '${entity.memberCount} members',
                        style: TextStyle(
                          color: CommunitasColors.cream.withAlpha(128),
                          fontSize: 11,
                          decoration: TextDecoration.none,
                          fontFamily: 'Roboto',
                          inherit: false,
                        ),
                      ),
                    ],
                  ),
                ),
                if (isStarred)
                  const Padding(
                    padding: EdgeInsets.only(right: 6),
                    child: Icon(Icons.star, size: 12, color: CommunitasColors.amber),
                  ),
                _buildRoleBadge(entity.role),
                PopupMenuButton<String>(
                  icon: const Icon(Icons.more_horiz, size: 16),
                  onSelected: (value) => _handleEntityMenuAction(entity, value),
                  itemBuilder: (context) => _entityMenuItems(entity, isStarred),
                ),
              ],
            ),
          ),
        ),
        // Nested children (projects/channels under org)
        childrenAsync.when(
          loading: () => const SizedBox.shrink(),
          error: (_, __) => const SizedBox.shrink(),
          data: (children) {
            if (children.isEmpty) return const SizedBox.shrink();
            return Padding(
              padding: const EdgeInsets.only(left: 24),
              child: Column(
                children: children.map((child) => _buildEntityItemUnified(
                  context: context,
                  entity: child,
                  children: [],
                  isStarred: ref.watch(starredEntitiesProvider).contains(entityKey(child.type, child.id)),
                )).toList(),
              ),
            );
          },
        ),
      ],
    );
  }

  /// Build entity item with unified entity (no async children).
  Widget _buildEntityItemUnified({
    required BuildContext context,
    required UnifiedEntity entity,
    required List<UnifiedEntity> children,
    required bool isStarred,
  }) {
    final icon = _getEntityIcon(entity.type);
    final color = _getEntityColor(entity.type);
    final route = '/entity/${entity.type}/${entity.id}';

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        GestureDetector(
          onTap: () {
            _recordRecentEntity(entity);
            context.go(route);
          },
          onSecondaryTapDown: (details) => _showEntityMenu(entity, details.globalPosition),
          onLongPressStart: (details) => _showEntityMenu(entity, details.globalPosition),
          behavior: HitTestBehavior.opaque,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
            child: Row(
              children: [
                Container(
                  width: 28,
                  height: 28,
                  decoration: BoxDecoration(
                    color: color.withAlpha(51),
                    borderRadius: BorderRadius.circular(6),
                  ),
                  child: Icon(icon, color: color, size: 16),
                ),
                const SizedBox(width: 10),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        entity.name,
                        style: const TextStyle(
                          color: CommunitasColors.cream,
                          fontSize: 13,
                          fontWeight: FontWeight.w500,
                          decoration: TextDecoration.none,
                          fontFamily: 'Roboto',
                          inherit: false,
                        ),
                        overflow: TextOverflow.ellipsis,
                      ),
                      const SizedBox(height: 2),
                      Text(
                        '${entity.memberCount} members',
                        style: TextStyle(
                          color: CommunitasColors.cream.withAlpha(128),
                          fontSize: 11,
                          decoration: TextDecoration.none,
                          fontFamily: 'Roboto',
                          inherit: false,
                        ),
                      ),
                    ],
                  ),
                ),
                if (isStarred)
                  const Padding(
                    padding: EdgeInsets.only(right: 6),
                    child: Icon(Icons.star, size: 12, color: CommunitasColors.amber),
                  ),
                _buildRoleBadge(entity.role),
                PopupMenuButton<String>(
                  icon: const Icon(Icons.more_horiz, size: 16),
                  onSelected: (value) => _handleEntityMenuAction(entity, value),
                  itemBuilder: (context) => _entityMenuItems(entity, isStarred),
                ),
              ],
            ),
          ),
        ),
        // Nested children (projects/channels under org)
        if (children.isNotEmpty)
          Padding(
            padding: const EdgeInsets.only(left: 24),
            child: Column(
              children: children.map((child) => _buildEntityItemUnified(
                context: context,
                entity: child,
                children: [],
                isStarred: ref.watch(starredEntitiesProvider).contains(entityKey(child.type, child.id)),
              )).toList(),
            ),
          ),
      ],
    );
  }

  Widget _buildContactItemUnified(
    BuildContext context,
    UnifiedContact contact, {
    bool isRecent = false,
    bool isStarred = false,
  }) {
    final statusColor = _getStatusColor(contact.status);
    // Use pubkeyHex for routing (the permanent identity)
    final route = '/contact/${contact.pubkeyHex}/chat';
    final displayInitial = contact.displayName.isNotEmpty ? contact.displayName[0].toUpperCase() : '?';

    return GestureDetector(
      onTap: () {
        _recordRecentContact(contact.pubkeyHex);
        context.go(route);
      },
      onSecondaryTapDown: (details) => _showContactMenu(contact, details.globalPosition),
      onLongPressStart: (details) => _showContactMenu(contact, details.globalPosition),
      behavior: HitTestBehavior.opaque,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
        child: Row(
          children: [
            Stack(
              children: [
                Container(
                  width: 32,
                  height: 32,
                  decoration: BoxDecoration(
                    color: CommunitasColors.person.withAlpha(51),
                    borderRadius: BorderRadius.circular(16),
                  ),
                  child: Center(
                    child: Text(
                      displayInitial,
                      style: const TextStyle(
                        color: CommunitasColors.person,
                        fontSize: 14,
                        fontWeight: FontWeight.bold,
                        decoration: TextDecoration.none,
                        fontFamily: 'Roboto',
                        inherit: false,
                      ),
                    ),
                  ),
                ),
                Positioned(
                  right: 0,
                  bottom: 0,
                  child: Container(
                    width: 10,
                    height: 10,
                    decoration: BoxDecoration(
                      color: statusColor,
                      shape: BoxShape.circle,
                      border: Border.all(color: CommunitasColors.moss, width: 2),
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(width: 10),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    contact.displayName,
                    style: const TextStyle(
                      color: CommunitasColors.cream,
                      fontSize: 13,
                      fontWeight: FontWeight.w500,
                      decoration: TextDecoration.none,
                      fontFamily: 'Roboto',
                      inherit: false,
                    ),
                  ),
                  Text(
                    _truncatePubkeyHex(contact.pubkeyHex),
                    style: TextStyle(
                      color: CommunitasColors.jade.withAlpha(179),
                      fontSize: 10,
                      fontFamily: 'monospace',
                      decoration: TextDecoration.none,
                      inherit: false,
                    ),
                  ),
                ],
              ),
            ),
            if (isStarred)
              const Padding(
                padding: EdgeInsets.only(right: 6),
                child: Icon(Icons.star, size: 14, color: CommunitasColors.amber),
              )
            else if (isRecent)
              const Padding(
                padding: EdgeInsets.only(right: 6),
                child: Icon(Icons.history, size: 14, color: CommunitasColors.cream),
              ),
            PopupMenuButton<String>(
              icon: const Icon(Icons.more_horiz, size: 16),
              onSelected: (value) => _handleContactMenuAction(contact, value),
              itemBuilder: (context) => _contactMenuItems(isStarred),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildRoleBadge(String role) {
    final badgeColor = _getRoleBadgeColor(role);
    final badgeIcon = _getRoleBadgeIcon(role);

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(
        color: badgeColor.withAlpha(51),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(badgeIcon, color: badgeColor, size: 10),
          const SizedBox(width: 3),
          Text(
            role.substring(0, 1).toUpperCase() + role.substring(1),
            style: TextStyle(
              color: badgeColor,
              fontSize: 9,
              fontWeight: FontWeight.w600,
              decoration: TextDecoration.none,
              fontFamily: 'Roboto',
              inherit: false,
            ),
          ),
        ],
      ),
    );
  }

  Widget _navItem(BuildContext context, IconData icon, String label, String? route, {bool isQuickNav = false}) {
    return GestureDetector(
      onTap: route != null ? () => context.go(route) : null,
      behavior: HitTestBehavior.opaque,
      child: Container(
        padding: EdgeInsets.symmetric(
          horizontal: 16,
          vertical: isQuickNav ? 10 : 12,
        ),
        child: Row(
          children: [
            Icon(icon, color: CommunitasColors.cream, size: 18),
            const SizedBox(width: 10),
            Text(
              label,
              style: const TextStyle(
                color: CommunitasColors.cream,
                fontSize: 13,
                decoration: TextDecoration.none,
                fontFamily: 'Roboto',
                inherit: false,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _navActionItem(
    BuildContext context,
    IconData icon,
    String label, {
    required VoidCallback onTap,
    String? shortcutHint,
  }) {
    return GestureDetector(
      onTap: onTap,
      behavior: HitTestBehavior.opaque,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
        child: Row(
          children: [
            Icon(icon, color: CommunitasColors.cream, size: 18),
            const SizedBox(width: 10),
            Expanded(
              child: Text(
                label,
                style: const TextStyle(
                  color: CommunitasColors.cream,
                  fontSize: 13,
                  decoration: TextDecoration.none,
                  fontFamily: 'Roboto',
                  inherit: false,
                ),
              ),
            ),
            if (shortcutHint != null)
              Text(
                shortcutHint,
                style: TextStyle(
                  fontSize: 10,
                  color: CommunitasColors.cream.withAlpha(128),
                ),
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildRecents() {
    final recentKeys = ref.watch(recentEntitiesProvider);
    final recentContacts = ref.watch(recentContactsProvider);
    final entitiesAsync = ref.watch(unifiedAllEntitiesProvider);
    final contactsAsync = ref.watch(unifiedContactsProvider);

    if (recentKeys.isEmpty && recentContacts.isEmpty) {
      return _buildEmptySection('No recent items');
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (recentKeys.isNotEmpty) ...[
          _buildSubsectionHeader('Entities'),
          entitiesAsync.when(
            loading: () => const Padding(
              padding: EdgeInsets.all(16),
              child: SizedBox(
                height: 20,
                width: 20,
                child: CircularProgressIndicator(strokeWidth: 2),
              ),
            ),
            error: (e, _) => Padding(
              padding: const EdgeInsets.all(16),
              child: Text('Error: $e', style: const TextStyle(color: CommunitasColors.error, fontSize: 12)),
            ),
            data: (entities) {
              final resolved = _resolveEntityKeys(recentKeys, entities);
              if (resolved.isEmpty) {
                return _buildEmptySection('No recent entities');
              }
              final starredKeys = ref.watch(starredEntitiesProvider);
              return Column(
                children: resolved.take(6).map((entity) => _buildEntityItemUnified(
                  context: context,
                  entity: entity,
                  children: const <UnifiedEntity>[],
                  isStarred: starredKeys.contains(entityKey(entity.type, entity.id)),
                )).toList(),
              );
            },
          ),
        ],
        if (recentKeys.isNotEmpty && recentContacts.isNotEmpty) const SizedBox(height: 8),
        if (recentContacts.isNotEmpty) ...[
          _buildSubsectionHeader('Contacts'),
          contactsAsync.when(
            loading: () => const Padding(
              padding: EdgeInsets.all(16),
              child: SizedBox(
                height: 20,
                width: 20,
                child: CircularProgressIndicator(strokeWidth: 2),
              ),
            ),
            error: (e, _) => Padding(
              padding: const EdgeInsets.all(16),
              child: Text('Error: $e', style: const TextStyle(color: CommunitasColors.error, fontSize: 12)),
            ),
            data: (contacts) {
              final resolved = _resolveContactKeys(recentContacts, contacts);
              if (resolved.isEmpty) {
                return _buildEmptySection('No recent contacts');
              }
            return Column(
              children: resolved.take(6).map((contact) => _buildContactItemUnified(
                context,
                contact,
                isRecent: true,
                isStarred: ref.watch(starredContactsProvider).contains(contact.pubkeyHex),
              )).toList(),
            );
            },
          ),
        ],
      ],
    );
  }

  Widget _buildStarred() {
    final starredKeys = ref.watch(starredEntitiesProvider);
    final starredContacts = ref.watch(starredContactsProvider);
    final entitiesAsync = ref.watch(unifiedAllEntitiesProvider);
    final contactsAsync = ref.watch(unifiedContactsProvider);

    if (starredKeys.isEmpty && starredContacts.isEmpty) {
      return _buildEmptySection('No starred items');
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (starredKeys.isNotEmpty) ...[
          _buildSubsectionHeader('Entities'),
          entitiesAsync.when(
            loading: () => const Padding(
              padding: EdgeInsets.all(16),
              child: SizedBox(
                height: 20,
                width: 20,
                child: CircularProgressIndicator(strokeWidth: 2),
              ),
            ),
            error: (e, _) => Padding(
              padding: const EdgeInsets.all(16),
              child: Text('Error: $e', style: const TextStyle(color: CommunitasColors.error, fontSize: 12)),
            ),
            data: (entities) {
              final resolved = _resolveEntityKeys(starredKeys.toList(), entities);
              if (resolved.isEmpty) {
                return _buildEmptySection('No starred entities');
              }
              return Column(
                children: resolved.take(6).map((entity) => _buildEntityItemUnified(
                  context: context,
                  entity: entity,
                  children: const <UnifiedEntity>[],
                  isStarred: true,
                )).toList(),
              );
            },
          ),
        ],
        if (starredKeys.isNotEmpty && starredContacts.isNotEmpty) const SizedBox(height: 8),
        if (starredContacts.isNotEmpty) ...[
          _buildSubsectionHeader('Contacts'),
          contactsAsync.when(
            loading: () => const Padding(
              padding: EdgeInsets.all(16),
              child: SizedBox(
                height: 20,
                width: 20,
                child: CircularProgressIndicator(strokeWidth: 2),
              ),
            ),
            error: (e, _) => Padding(
              padding: const EdgeInsets.all(16),
              child: Text('Error: $e', style: const TextStyle(color: CommunitasColors.error, fontSize: 12)),
            ),
            data: (contacts) {
              final resolved = _resolveContactKeys(starredContacts.toList(), contacts);
              if (resolved.isEmpty) {
                return _buildEmptySection('No starred contacts');
              }
              return Column(
                children: resolved.take(6).map((contact) => _buildContactItemUnified(
                  context,
                  contact,
                  isStarred: true,
                )).toList(),
              );
            },
          ),
        ],
      ],
    );
  }

  List<UnifiedEntity> _resolveEntityKeys(List<String> keys, List<UnifiedEntity> entities) {
    final map = {for (final entity in entities) entityKey(entity.type, entity.id): entity};
    final resolved = <UnifiedEntity>[];
    for (final key in keys) {
      final entity = map[key];
      if (entity != null) {
        resolved.add(entity);
      }
    }
    return resolved;
  }

  List<UnifiedContact> _resolveContactKeys(List<String> keys, List<UnifiedContact> contacts) {
    final map = {for (final contact in contacts) contact.pubkeyHex: contact};
    final resolved = <UnifiedContact>[];
    for (final key in keys) {
      final contact = map[key];
      if (contact != null) {
        resolved.add(contact);
      }
    }
    return resolved;
  }

  void _showQuickSwitcher(BuildContext context) {
    showDialog<void>(
      context: context,
      builder: (context) => const QuickSwitcherDialog(),
    );
  }

  Widget _buildSubsectionHeader(String title, {VoidCallback? onAdd}) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 4, 16, 4),
      child: Row(
        children: [
          Text(
            title,
            style: TextStyle(
              color: CommunitasColors.cream.withAlpha(179),
              fontSize: 10,
              fontWeight: FontWeight.w600,
              letterSpacing: 0.6,
            ),
          ),
          const Spacer(),
          if (onAdd != null)
            IconButton(
              visualDensity: VisualDensity.compact,
              onPressed: onAdd,
              icon: Icon(
                Icons.add,
                color: CommunitasColors.cream.withAlpha(128),
                size: 14,
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildEmptySection(String label) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: Text(
        label,
        style: TextStyle(
          color: CommunitasColors.cream.withAlpha(128),
          fontSize: 12,
        ),
      ),
    );
  }

  Widget _buildUserHeader(BuildContext context) {
    final identity = ref.watch(unifiedIdentityProvider);

    return Container(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                width: 40,
                height: 40,
                decoration: BoxDecoration(
                  color: CommunitasColors.jade,
                  borderRadius: BorderRadius.circular(20),
                ),
                child: const Icon(
                  Icons.person,
                  color: CommunitasColors.cream,
                  size: 24,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      identity.displayName,
                      style: const TextStyle(
                        fontWeight: FontWeight.w600,
                        fontSize: 14,
                        color: CommunitasColors.cream,
                        decoration: TextDecoration.none,
                        fontFamily: 'Roboto',
                        inherit: false,
                      ),
                    ),
                    Text(
                      _truncatePubkeyHex(identity.pubkeyHex),
                      style: const TextStyle(
                        fontSize: 11,
                        color: CommunitasColors.jade,
                        fontFamily: 'monospace',
                        decoration: TextDecoration.none,
                        inherit: false,
                      ),
                    ),
                  ],
                ),
              ),
              Container(
                width: 8,
                height: 8,
                decoration: const BoxDecoration(
                  color: CommunitasColors.online,
                  shape: BoxShape.circle,
                ),
              ),
              const SizedBox(width: 8),
              PopupMenuButton<String>(
                icon: Icon(
                  Icons.more_vert,
                  color: CommunitasColors.cream.withAlpha(179),
                  size: 18,
                ),
                tooltip: 'Account options',
                onSelected: (value) => _handleUserMenuAction(value),
                itemBuilder: (context) => [
                  const PopupMenuItem(
                    value: 'settings',
                    child: Row(
                      children: [
                        Icon(Icons.settings, size: 18),
                        SizedBox(width: 8),
                        Text('Settings'),
                      ],
                    ),
                  ),
                  const PopupMenuItem(
                    value: 'more',
                    child: Row(
                      children: [
                        Icon(Icons.more_horiz, size: 18),
                        SizedBox(width: 8),
                        Text('More'),
                      ],
                    ),
                  ),
                  const PopupMenuDivider(),
                  const PopupMenuItem(
                    value: 'logout',
                    child: Row(
                      children: [
                        Icon(Icons.logout, size: 18, color: CommunitasColors.error),
                        SizedBox(width: 8),
                        Text('Logout', style: TextStyle(color: CommunitasColors.error)),
                      ],
                    ),
                  ),
                ],
              ),
            ],
          ),
        ],
      ),
    );
  }

  Future<void> _handleUserMenuAction(String action) async {
    switch (action) {
      case 'settings':
      case 'more':
        context.go(Routes.more);
        break;
      case 'logout':
        final authNotifier = ref.read(authNotifierProvider.notifier);
        await authNotifier.logout();
        if (mounted) {
          context.go(Routes.login);
        }
        break;
    }
  }

  Future<void> _showCreateEntityDialog(
    BuildContext context,
    FlutterEntityType defaultType,
  ) async {
    final nameController = TextEditingController();
    final descriptionController = TextEditingController();
    FlutterEntityType selectedType = defaultType;
    String? selectedParentId;

    List<UnifiedEntity> orgs = [];
    try {
      orgs = await ref.read(unifiedOrganizationsProvider.future);
    } catch (_) {}

    await showDialog<void>(
      context: context,
      builder: (context) {
        return StatefulBuilder(
          builder: (context, setState) {
            final needsParent =
                selectedType == FlutterEntityType.project ||
                selectedType == FlutterEntityType.channel;

            return AlertDialog(
              title: const Text('Create entity'),
              content: SingleChildScrollView(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    TextField(
                      controller: nameController,
                      decoration: const InputDecoration(labelText: 'Name'),
                    ),
                    const SizedBox(height: 12),
                    DropdownButtonFormField<FlutterEntityType>(
                      value: selectedType,
                      decoration: const InputDecoration(labelText: 'Type'),
                      items: const [
                        DropdownMenuItem(
                          value: FlutterEntityType.organisation,
                          child: Text('Organization'),
                        ),
                        DropdownMenuItem(
                          value: FlutterEntityType.project,
                          child: Text('Project'),
                        ),
                        DropdownMenuItem(
                          value: FlutterEntityType.channel,
                          child: Text('Channel'),
                        ),
                        DropdownMenuItem(
                          value: FlutterEntityType.group,
                          child: Text('Group'),
                        ),
                      ],
                      onChanged: (value) {
                        if (value == null) return;
                        setState(() {
                          selectedType = value;
                          if (selectedType == FlutterEntityType.organisation ||
                              selectedType == FlutterEntityType.group) {
                            selectedParentId = null;
                          }
                        });
                      },
                    ),
                    if (needsParent) ...[
                      const SizedBox(height: 12),
                      DropdownButtonFormField<String>(
                        value: selectedParentId,
                        decoration:
                            const InputDecoration(labelText: 'Parent org'),
                        items: orgs
                            .map((org) => DropdownMenuItem(
                                  value: org.id,
                                  child: Text(org.name),
                                ))
                            .toList(),
                        onChanged: (value) {
                          setState(() {
                            selectedParentId = value;
                          });
                        },
                      ),
                    ],
                    const SizedBox(height: 12),
                    TextField(
                      controller: descriptionController,
                      decoration:
                          const InputDecoration(labelText: 'Description'),
                      maxLines: 2,
                    ),
                  ],
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: const Text('Cancel'),
                ),
                FilledButton(
                  onPressed: () async {
                    final name = nameController.text.trim();
                    if (name.isEmpty) return;

                    final controller =
                        ref.read(ffiEntityControllerProvider.notifier);
                    await controller.createEntity(
                      name: name,
                      entityType: selectedType,
                      description: descriptionController.text.trim().isEmpty
                          ? null
                          : descriptionController.text.trim(),
                      parentOrgId: selectedParentId,
                    );

                    ref.invalidate(unifiedOrganizationsProvider);
                    ref.invalidate(unifiedProjectsProvider);
                    ref.invalidate(unifiedChannelsProvider);
                    ref.invalidate(unifiedGroupsProvider);

                    Navigator.of(context).pop();
                  },
                  child: const Text('Create'),
                ),
              ],
            );
          },
        );
      },
    );

    nameController.dispose();
    descriptionController.dispose();
  }

  Future<void> _showCreateContactDialog(BuildContext context) async {
    final nameController = TextEditingController();
    final fourWordsController = TextEditingController();
    bool favourite = false;

    await showDialog<void>(
      context: context,
      builder: (context) {
        return StatefulBuilder(
          builder: (context, setState) {
            return AlertDialog(
              title: const Text('Add contact'),
              content: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  TextField(
                    controller: nameController,
                    decoration:
                        const InputDecoration(labelText: 'Display name'),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    controller: fourWordsController,
                    decoration: const InputDecoration(
                      labelText: 'Four words (optional)',
                    ),
                  ),
                  const SizedBox(height: 12),
                  SwitchListTile(
                    title: const Text('Favorite'),
                    value: favourite,
                    onChanged: (value) {
                      setState(() {
                        favourite = value;
                      });
                    },
                  ),
                ],
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: const Text('Cancel'),
                ),
                FilledButton(
                  onPressed: () async {
                    final displayName = nameController.text.trim();
                    if (displayName.isEmpty) return;

                    final api = ref.read(communitasApiProvider);
                    if (api != null) {
                      await api.contactCreate(
                        displayName: displayName,
                        fourWords: fourWordsController.text.trim().isEmpty
                            ? null
                            : fourWordsController.text.trim(),
                        isFavourite: favourite,
                      );
                    }

                    ref.invalidate(unifiedContactsProvider);
                    Navigator.of(context).pop();
                  },
                  child: const Text('Add'),
                ),
              ],
            );
          },
        );
      },
    );

    nameController.dispose();
    fourWordsController.dispose();
  }

  Future<void> _showEntityMenu(UnifiedEntity entity, Offset position) async {
    final overlay = Overlay.of(context).context.findRenderObject() as RenderBox;
    final result = await showMenu<String>(
      context: context,
      position: RelativeRect.fromRect(
        Rect.fromPoints(position, position),
        Offset.zero & overlay.size,
      ),
      items: _entityMenuItems(
        entity,
        ref.read(starredEntitiesProvider).contains(entityKey(entity.type, entity.id)),
      ),
    );
    if (result == null) return;
    await _handleEntityMenuAction(entity, result);
  }

  Future<void> _showContactMenu(UnifiedContact contact, Offset position) async {
    final overlay = Overlay.of(context).context.findRenderObject() as RenderBox;
    final result = await showMenu<String>(
      context: context,
      position: RelativeRect.fromRect(
        Rect.fromPoints(position, position),
        Offset.zero & overlay.size,
      ),
      items: _contactMenuItems(
        ref.read(starredContactsProvider).contains(contact.pubkeyHex),
      ),
    );
    if (result == null) return;
    await _handleContactMenuAction(contact, result);
  }

  List<PopupMenuEntry<String>> _entityMenuItems(UnifiedEntity entity, bool isStarred) {
    final items = <PopupMenuEntry<String>>[
      const PopupMenuItem(value: 'open', child: Text('Open')),
      const PopupMenuItem(value: 'chat', child: Text('Open chat')),
      const PopupMenuItem(value: 'drive', child: Text('Open drive')),
    ];
    if (entity.type == 'project') {
      items.add(const PopupMenuItem(value: 'board', child: Text('Open board')));
    }
    items.add(
      PopupMenuItem(
        value: isStarred ? 'unstar' : 'star',
        child: Text(isStarred ? 'Unstar' : 'Star'),
      ),
    );
    if (entity.type == 'organisation' || entity.type == 'organization') {
      items.add(const PopupMenuDivider());
      items.add(const PopupMenuItem(value: 'mark_org', child: Text('Mark as organization')));
      items.add(const PopupMenuItem(value: 'mark_community', child: Text('Mark as community')));
    }
    items.add(const PopupMenuDivider());
    items.add(const PopupMenuItem(value: 'copy_id', child: Text('Copy ID')));
    items.add(const PopupMenuItem(value: 'copy_name', child: Text('Copy name')));
    return items;
  }

  List<PopupMenuEntry<String>> _contactMenuItems(bool isStarred) {
    return [
      const PopupMenuItem(value: 'message', child: Text('Message')),
      PopupMenuItem(
        value: isStarred ? 'unstar' : 'star',
        child: Text(isStarred ? 'Unstar' : 'Star'),
      ),
      const PopupMenuDivider(),
      const PopupMenuItem(value: 'copy_id', child: Text('Copy ID')),
      const PopupMenuItem(value: 'copy_name', child: Text('Copy name')),
    ];
  }

  Future<void> _handleEntityMenuAction(UnifiedEntity entity, String action) async {
    switch (action) {
      case 'open':
        _recordRecentEntity(entity);
        context.go('/entity/${entity.type}/${entity.id}');
        break;
      case 'chat':
        _recordRecentEntity(entity);
        context.go(
          Routes.entityChat.replaceAll(':type', entity.type).replaceAll(':id', entity.id),
        );
        break;
      case 'drive':
        _recordRecentEntity(entity);
        context.go(
          Routes.entityDrive.replaceAll(':type', entity.type).replaceAll(':id', entity.id),
        );
        break;
      case 'board':
        _recordRecentEntity(entity);
        context.go(
          Routes.projectBoard.replaceAll(':id', entity.id),
        );
        break;
      case 'star':
      case 'unstar':
        await ref.read(starredEntitiesProvider.notifier).toggle(entityKey(entity.type, entity.id));
        break;
      case 'mark_org':
        await ref
            .read(organizationCategoryOverridesProvider.notifier)
            .setCategory(entity.id, OrganizationCategory.organization);
        break;
      case 'mark_community':
        await ref
            .read(organizationCategoryOverridesProvider.notifier)
            .setCategory(entity.id, OrganizationCategory.community);
        break;
      case 'copy_id':
        _copyToClipboard(entity.id, 'Entity ID');
        break;
      case 'copy_name':
        _copyToClipboard(entity.name, 'Entity name');
        break;
    }
  }

  Future<void> _handleContactMenuAction(UnifiedContact contact, String action) async {
    switch (action) {
      case 'message':
        _recordRecentContact(contact.pubkeyHex);
        context.go(
          Routes.contactChat.replaceAll(':fourWords', contact.pubkeyHex),
        );
        break;
      case 'star':
      case 'unstar':
        await ref
            .read(starredContactsProvider.notifier)
            .toggle(contact.pubkeyHex);
        break;
      case 'copy_id':
        _copyToClipboard(contact.pubkeyHex, 'Contact ID');
        break;
      case 'copy_name':
        _copyToClipboard(contact.displayName, 'Contact name');
        break;
    }
  }

  void _copyToClipboard(String value, String label) {
    Clipboard.setData(ClipboardData(text: value));
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('$label copied'),
        duration: const Duration(seconds: 2),
      ),
    );
  }

  void _recordRecentEntity(UnifiedEntity entity) {
    ref
        .read(recentEntitiesProvider.notifier)
        .record(entityKey(entity.type, entity.id));
  }

  void _recordRecentContact(String contactId) {
    ref.read(recentContactsProvider.notifier).record(contactId);
  }

  Future<void> _clearRecents() async {
    await ref.read(recentEntitiesProvider.notifier).clear();
    await ref.read(recentContactsProvider.notifier).clear();
  }

  // Helper methods
  IconData _getEntityIcon(String type) {
    switch (type) {
      case 'organisation':
      case 'organization': return Icons.business;
      case 'project': return Icons.folder;
      case 'channel': return Icons.tag;
      case 'group': return Icons.group;
      default: return Icons.folder;
    }
  }

  Color _getEntityColor(String type) {
    switch (type) {
      case 'organisation':
      case 'organization': return CommunitasColors.organization;
      case 'project': return CommunitasColors.project;
      case 'channel': return CommunitasColors.channel;
      case 'group': return CommunitasColors.group;
      default: return CommunitasColors.jade;
    }
  }

  Color _getRoleBadgeColor(String role) {
    switch (role) {
      case 'owner': return CommunitasColors.owner;
      case 'admin': return CommunitasColors.admin;
      case 'member': return CommunitasColors.member;
      case 'guest': return CommunitasColors.guest;
      default: return CommunitasColors.member;
    }
  }

  IconData _getRoleBadgeIcon(String role) {
    switch (role) {
      case 'owner': return Icons.workspace_premium;
      case 'admin': return Icons.shield;
      case 'member': return Icons.person;
      case 'guest': return Icons.visibility;
      default: return Icons.person;
    }
  }

  Color _getStatusColor(String status) {
    switch (status) {
      case 'online': return CommunitasColors.online;
      case 'away': return CommunitasColors.away;
      case 'busy': return CommunitasColors.error;
      default: return CommunitasColors.offline;
    }
  }

  /// Truncate pubkey hex for display (show first few chars).
  /// For four-word addresses (demo mode), show first two words.
  String _truncatePubkeyHex(String pubkeyHex) {
    // Check if it looks like a four-word address (contains dashes)
    if (pubkeyHex.contains('-')) {
      final parts = pubkeyHex.split('-');
      if (parts.length >= 4) {
        return '${parts[0]}-${parts[1]}-...';
      }
      return pubkeyHex;
    }
    // For hex pubkey, show first 8 chars
    if (pubkeyHex.length > 12) {
      return '${pubkeyHex.substring(0, 8)}...';
    }
    return pubkeyHex;
  }
}
