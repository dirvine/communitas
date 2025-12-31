import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/colors.dart';
import '../../../shared/widgets/sidebar.dart';
import '../../../shared/widgets/adaptive_layout.dart';

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
    return AdaptiveLayout(
      sidebar: const Sidebar(),
      body: DefaultTabController(
        length: entityType == 'project' ? 5 : 4,
        child: Scaffold(
          appBar: AppBar(
            title: Text(_getEntityName()),
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
              _buildDetailsTab(),
            ],
          ),
        ),
      ),
    );
  }

  String _getEntityName() {
    // TODO: Get from provider
    return 'Entity $entityId';
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

  Widget _buildDetailsTab() {
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
                      child: const Icon(
                        Icons.business,
                        color: CommunitasColors.cream,
                        size: 32,
                      ),
                    ),
                    const SizedBox(width: 16),
                    const Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            'Entity Name',
                            style: TextStyle(
                              fontSize: 20,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                          SizedBox(height: 4),
                          Text(
                            'Entity description goes here',
                            style: TextStyle(
                              color: CommunitasColors.jade,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
