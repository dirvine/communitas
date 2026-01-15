import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../core/router.dart';
import '../../core/theme/colors.dart';
import '../../features/auth/providers/auth_provider.dart';
import '../../services/unified_data_provider.dart';

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
    'myOrganizations': true,
    'myCommunities': true,
    'personal': true,
    'directMessages': true,
  };

  @override
  Widget build(BuildContext context) {
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
                      Container(height: 1, color: CommunitasColors.fern),

                      // My Organizations (Owner role)
                      _buildSection(
                        'My Organizations',
                        'myOrganizations',
                        _buildMyOrganizations(),
                      ),

                      // My Communities (Member/Admin role)
                      _buildSection(
                        'My Communities',
                        'myCommunities',
                        _buildMyCommunities(),
                      ),

                      // Personal Space
                      _buildSection(
                        'Personal',
                        'personal',
                        _buildPersonalSpace(),
                      ),

                      // Direct Messages
                      _buildSection(
                        'Direct Messages',
                        'directMessages',
                        _buildDirectMessages(),
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

  Widget _buildSection(String title, String sectionKey, Widget child) {
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
                GestureDetector(
                  onTap: () {
                    // TODO: Show create entity dialog
                  },
                  child: Icon(
                    Icons.add,
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

  Widget _buildMyOrganizations() {
    final orgsAsync = ref.watch(unifiedOrganizationsProvider);

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
        // Filter orgs where user is owner
        final myOrgs = orgs.where((org) => org.role == 'owner').toList();

        return Column(
          children: myOrgs.map((org) => _buildEntityItemAsync(
            context: context,
            entity: org,
            parentId: org.id,
          )).toList(),
        );
      },
    );
  }

  Widget _buildMyCommunities() {
    final orgsAsync = ref.watch(unifiedOrganizationsProvider);

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
        // Filter orgs where user is NOT owner
        final communities = orgs.where((org) => org.role != 'owner').toList();

        return Column(
          children: communities.map((org) => _buildEntityItemAsync(
            context: context,
            entity: org,
            parentId: org.id,
          )).toList(),
        );
      },
    );
  }

  Widget _buildPersonalSpace() {
    final groupsAsync = ref.watch(unifiedGroupsProvider);

    return groupsAsync.when(
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
        return Column(
          children: groups.map((group) => _buildEntityItemUnified(
            context: context,
            entity: group,
            children: [],
          )).toList(),
        );
      },
    );
  }

  Widget _buildDirectMessages() {
    final contactsAsync = ref.watch(unifiedContactsProvider);

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
        return Column(
          children: contacts.map((contact) => _buildContactItemUnified(context, contact)).toList(),
        );
      },
    );
  }

  /// Build entity item with async child loading (for bridge mode).
  Widget _buildEntityItemAsync({
    required BuildContext context,
    required UnifiedEntity entity,
    required String parentId,
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
          onTap: () => context.go(route),
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
                _buildRoleBadge(entity.role),
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
  }) {
    final icon = _getEntityIcon(entity.type);
    final color = _getEntityColor(entity.type);
    final route = '/entity/${entity.type}/${entity.id}';

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        GestureDetector(
          onTap: () => context.go(route),
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
                _buildRoleBadge(entity.role),
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
              )).toList(),
            ),
          ),
      ],
    );
  }

  Widget _buildContactItemUnified(BuildContext context, UnifiedContact contact) {
    final statusColor = _getStatusColor(contact.status);
    // Use pubkeyHex for routing (the permanent identity)
    final route = '/contact/${contact.pubkeyHex}/chat';
    final displayInitial = contact.displayName.isNotEmpty ? contact.displayName[0].toUpperCase() : '?';

    return GestureDetector(
      onTap: () => context.go(route),
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

  Widget _buildUserHeader(BuildContext context) {
    final identity = ref.watch(unifiedIdentityProvider);
    final isDemoUser = ref.watch(isDemoUserProvider);

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
            ],
          ),
          // Demo mode badge
          if (isDemoUser) ...[
            const SizedBox(height: 8),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: CommunitasColors.amber.withAlpha(51),
                borderRadius: BorderRadius.circular(4),
                border: Border.all(
                  color: CommunitasColors.amber.withAlpha(128),
                ),
              ),
              child: const Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(
                    Icons.science,
                    color: CommunitasColors.amber,
                    size: 12,
                  ),
                  SizedBox(width: 4),
                  Text(
                    'Demo Mode',
                    style: TextStyle(
                      color: CommunitasColors.amber,
                      fontSize: 10,
                      fontWeight: FontWeight.w600,
                      decoration: TextDecoration.none,
                      fontFamily: 'Roboto',
                      inherit: false,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }

  // Helper methods
  IconData _getEntityIcon(String type) {
    switch (type) {
      case 'organization': return Icons.business;
      case 'project': return Icons.folder;
      case 'channel': return Icons.tag;
      case 'group': return Icons.group;
      default: return Icons.folder;
    }
  }

  Color _getEntityColor(String type) {
    switch (type) {
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
