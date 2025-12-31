import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../core/router.dart';
import '../../core/theme/colors.dart';
import '../../demo/demo_data.dart';

/// Main navigation sidebar for desktop layout.
class Sidebar extends ConsumerStatefulWidget {
  const Sidebar({super.key});

  @override
  ConsumerState<Sidebar> createState() => _SidebarState();
}

class _SidebarState extends ConsumerState<Sidebar> {
  String? _expandedSection;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 260,
      color: CommunitasColors.moss,
      child: Column(
        children: [
          // User header
          _buildUserHeader(),
          const Divider(height: 1),

          // Navigation
          Expanded(
            child: ListView(
              padding: const EdgeInsets.symmetric(vertical: 8),
              children: [
                // Home
                _buildNavItem(
                  icon: Icons.home_outlined,
                  label: 'Home',
                  route: Routes.home,
                ),

                const SizedBox(height: 8),

                // Organizations section
                _buildSection(
                  title: 'My Organizations',
                  icon: Icons.business,
                  color: CommunitasColors.owner,
                  items: DemoData.organizations
                      .where((o) => o.role == 'owner')
                      .map((o) => _EntityItem(
                            id: o.id,
                            name: o.name,
                            type: o.type,
                            color: CommunitasColors.organization,
                          ))
                      .toList(),
                ),

                // Communities section
                _buildSection(
                  title: 'My Communities',
                  icon: Icons.groups,
                  color: CommunitasColors.jade,
                  items: DemoData.organizations
                      .where((o) => o.role != 'owner')
                      .map((o) => _EntityItem(
                            id: o.id,
                            name: o.name,
                            type: o.type,
                            color: CommunitasColors.organization,
                          ))
                      .toList(),
                ),

                // Projects section
                _buildSection(
                  title: 'Projects',
                  icon: Icons.folder_outlined,
                  color: CommunitasColors.project,
                  items: DemoData.projects
                      .map((p) => _EntityItem(
                            id: p.id,
                            name: p.name,
                            type: p.type,
                            color: CommunitasColors.project,
                          ))
                      .toList(),
                ),

                // Channels section
                _buildSection(
                  title: 'Channels',
                  icon: Icons.tag,
                  color: CommunitasColors.channel,
                  items: DemoData.channels
                      .map((c) => _EntityItem(
                            id: c.id,
                            name: c.name,
                            type: c.type,
                            color: CommunitasColors.channel,
                          ))
                      .toList(),
                ),

                // Groups section
                _buildSection(
                  title: 'Groups',
                  icon: Icons.group_outlined,
                  color: CommunitasColors.group,
                  items: DemoData.groups
                      .map((g) => _EntityItem(
                            id: g.id,
                            name: g.name,
                            type: g.type,
                            color: CommunitasColors.group,
                          ))
                      .toList(),
                ),

                const Divider(),

                // Contacts section
                _buildSection(
                  title: 'Contacts',
                  icon: Icons.person_outline,
                  color: CommunitasColors.person,
                  items: DemoData.contacts
                      .map((c) => _EntityItem(
                            id: c.fourWords,
                            name: c.displayName,
                            type: 'contact',
                            color: CommunitasColors.person,
                            status: c.status,
                          ))
                      .toList(),
                ),

                const Divider(),

                // Network status
                _buildNavItem(
                  icon: Icons.lan_outlined,
                  label: 'Network',
                  route: Routes.network,
                  trailing: Container(
                    width: 8,
                    height: 8,
                    decoration: const BoxDecoration(
                      color: CommunitasColors.online,
                      shape: BoxShape.circle,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildUserHeader() {
    return Container(
      padding: const EdgeInsets.all(16),
      child: Row(
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
                  DemoData.demoIdentity.displayName,
                  style: const TextStyle(fontWeight: FontWeight.w600),
                ),
                Text(
                  DemoData.demoIdentity.fourWords,
                  style: TextStyle(
                    fontSize: 11,
                    color: CommunitasColors.jade,
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
    );
  }

  Widget _buildNavItem({
    required IconData icon,
    required String label,
    required String route,
    Widget? trailing,
  }) {
    return ListTile(
      dense: true,
      leading: Icon(icon, size: 20),
      title: Text(label),
      trailing: trailing,
      onTap: () => context.go(route),
    );
  }

  Widget _buildSection({
    required String title,
    required IconData icon,
    required Color color,
    required List<_EntityItem> items,
  }) {
    final isExpanded = _expandedSection == title;

    return Column(
      children: [
        InkWell(
          onTap: () {
            setState(() {
              _expandedSection = isExpanded ? null : title;
            });
          },
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: Row(
              children: [
                Icon(icon, size: 16, color: color),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    title,
                    style: TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.w600,
                      color: CommunitasColors.cream.withOpacity(0.7),
                    ),
                  ),
                ),
                Icon(
                  isExpanded ? Icons.expand_less : Icons.expand_more,
                  size: 16,
                  color: CommunitasColors.cream.withOpacity(0.5),
                ),
                IconButton(
                  icon: const Icon(Icons.add, size: 16),
                  onPressed: () {
                    // TODO: Show create entity sheet
                  },
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(
                    minWidth: 24,
                    minHeight: 24,
                  ),
                ),
              ],
            ),
          ),
        ),
        if (isExpanded)
          ...items.map((item) => _buildEntityTile(item)),
      ],
    );
  }

  Widget _buildEntityTile(_EntityItem item) {
    return ListTile(
      dense: true,
      contentPadding: const EdgeInsets.only(left: 40, right: 16),
      leading: Container(
        width: 24,
        height: 24,
        decoration: BoxDecoration(
          color: item.color.withOpacity(0.2),
          borderRadius: BorderRadius.circular(4),
        ),
        child: Center(
          child: Text(
            item.name[0].toUpperCase(),
            style: TextStyle(
              color: item.color,
              fontWeight: FontWeight.bold,
              fontSize: 12,
            ),
          ),
        ),
      ),
      title: Text(
        item.name,
        style: const TextStyle(fontSize: 14),
      ),
      trailing: item.status != null
          ? Container(
              width: 8,
              height: 8,
              decoration: BoxDecoration(
                color: CommunitasColors.statusColor(item.status!),
                shape: BoxShape.circle,
              ),
            )
          : null,
      onTap: () {
        if (item.type == 'contact') {
          context.go('/contact/${item.id}/chat');
        } else {
          context.go('/entity/${item.type}/${item.id}/chat');
        }
      },
    );
  }
}

class _EntityItem {
  final String id;
  final String name;
  final String type;
  final Color color;
  final String? status;

  _EntityItem({
    required this.id,
    required this.name,
    required this.type,
    required this.color,
    this.status,
  });
}
