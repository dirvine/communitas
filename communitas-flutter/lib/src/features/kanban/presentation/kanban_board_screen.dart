import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/colors.dart';
import '../../../shared/widgets/sidebar.dart';
import '../../../shared/widgets/adaptive_layout.dart';
import '../../../demo/demo_data.dart';

/// Kanban board screen with 5 columns.
class KanbanBoardScreen extends ConsumerWidget {
  final String projectId;

  const KanbanBoardScreen({
    super.key,
    required this.projectId,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return AdaptiveLayout(
      sidebar: const Sidebar(),
      body: Scaffold(
        appBar: AppBar(
          title: const Text('Project Board'),
          actions: [
            IconButton(
              icon: const Icon(Icons.filter_list),
              onPressed: () {},
            ),
            IconButton(
              icon: const Icon(Icons.add),
              onPressed: () {},
              tooltip: 'Add card',
            ),
          ],
        ),
        body: SingleChildScrollView(
          scrollDirection: Axis.horizontal,
          padding: const EdgeInsets.all(16),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _buildColumn('Backlog', 'backlog'),
              _buildColumn('To Do', 'to_do'),
              _buildColumn('In Progress', 'in_progress'),
              _buildColumn('Review', 'review'),
              _buildColumn('Done', 'done'),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildColumn(String title, String columnId) {
    final cards = DemoData.kanbanCards
        .where((c) => c.column == columnId)
        .toList();

    return Container(
      width: 280,
      margin: const EdgeInsets.only(right: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Column header
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            child: Row(
              children: [
                Text(
                  title,
                  style: const TextStyle(
                    fontWeight: FontWeight.w600,
                    fontSize: 14,
                  ),
                ),
                const SizedBox(width: 8),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                  decoration: BoxDecoration(
                    color: CommunitasColors.fern,
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: Text(
                    '${cards.length}',
                    style: const TextStyle(fontSize: 12),
                  ),
                ),
                const Spacer(),
                IconButton(
                  icon: const Icon(Icons.add, size: 18),
                  onPressed: () {},
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(
                    minWidth: 28,
                    minHeight: 28,
                  ),
                ),
              ],
            ),
          ),

          // Cards
          Expanded(
            child: Container(
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: CommunitasColors.moss.withOpacity(0.5),
                borderRadius: BorderRadius.circular(8),
              ),
              child: ListView.builder(
                itemCount: cards.length,
                itemBuilder: (context, index) {
                  return _buildCard(cards[index]);
                },
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildCard(DemoKanbanCard card) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: CommunitasColors.moss,
        borderRadius: BorderRadius.circular(8),
        border: Border(
          left: BorderSide(
            color: _getPriorityColor(card.priority),
            width: 3,
          ),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            card.title,
            style: const TextStyle(fontWeight: FontWeight.w500),
          ),
          const SizedBox(height: 8),
          Text(
            card.description,
            style: TextStyle(
              fontSize: 12,
              color: CommunitasColors.cream.withOpacity(0.7),
            ),
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              // Priority badge
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  color: _getPriorityColor(card.priority).withOpacity(0.2),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  card.priority.toUpperCase(),
                  style: TextStyle(
                    fontSize: 10,
                    fontWeight: FontWeight.w600,
                    color: _getPriorityColor(card.priority),
                  ),
                ),
              ),
              const Spacer(),
              // Assignee
              if (card.assignee != null)
                Container(
                  width: 24,
                  height: 24,
                  decoration: BoxDecoration(
                    color: CommunitasColors.jade,
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Center(
                    child: Text(
                      card.assignee![0].toUpperCase(),
                      style: const TextStyle(
                        fontSize: 10,
                        fontWeight: FontWeight.bold,
                        color: CommunitasColors.cream,
                      ),
                    ),
                  ),
                ),
            ],
          ),
        ],
      ),
    );
  }

  Color _getPriorityColor(String priority) {
    switch (priority) {
      case 'critical':
        return CommunitasColors.priorityCritical;
      case 'high':
        return CommunitasColors.priorityHigh;
      case 'medium':
        return CommunitasColors.priorityMedium;
      case 'low':
        return CommunitasColors.priorityLow;
      default:
        return CommunitasColors.priorityMedium;
    }
  }
}
