import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../bindings/api_exports.dart';
import '../../../core/theme/colors.dart';
import '../../../demo/demo_data.dart';
import '../../../services/ffi_provider.dart';
import '../../../services/unified_data_provider.dart';
import '../../../shared/widgets/adaptive_layout.dart';
import '../../../shared/widgets/sidebar.dart';
import '../../../shared/widgets/collab_toolbar.dart';

/// Kanban board screen with 5 columns.
class KanbanBoardScreen extends ConsumerWidget {
  final String projectId;

  const KanbanBoardScreen({
    super.key,
    required this.projectId,
  });

  static const _defaultColumns = <String>[
    'Backlog',
    'To Do',
    'In Progress',
    'Review',
    'Done',
  ];

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final projectAsync =
        ref.watch(unifiedEntityByIdProvider((type: 'project', id: projectId)));
    final projectName = projectAsync.maybeWhen(
      data: (entity) => entity?.name,
      orElse: () => null,
    );

    return AdaptiveLayout(
      sidebar: const Sidebar(),
      body: kIsWeb
          ? _buildDemoScaffold(context, projectName)
          : _buildFfiScaffold(context, ref, projectName),
    );
  }

  Widget _buildDemoScaffold(BuildContext context, String? projectName) {
    return Scaffold(
      appBar: AppBar(
        title: Text(projectName == null || projectName.isEmpty
            ? 'Project Board'
            : '$projectName · Board'),
        actions: [
          ...CollabToolbar.entityActions(
            context,
            entityType: 'project',
            entityId: projectId,
          ),
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
            _buildDemoColumn('Backlog', 'backlog'),
            _buildDemoColumn('To Do', 'to_do'),
            _buildDemoColumn('In Progress', 'in_progress'),
            _buildDemoColumn('Review', 'review'),
            _buildDemoColumn('Done', 'done'),
          ],
        ),
      ),
    );
  }

  Widget _buildFfiScaffold(BuildContext context, WidgetRef ref, String? projectName) {
    final api = ref.watch(communitasApiProvider);
    if (api == null) {
      return const Scaffold(
        body: Center(
          child: Text('Kanban requires the native backend.'),
        ),
      );
    }

    final boardsAsync = ref.watch(ffiKanbanBoardsProvider(projectId));

    return boardsAsync.when(
      loading: () => const Scaffold(
        body: Center(child: CircularProgressIndicator()),
      ),
      error: (e, _) => Scaffold(
        body: Center(child: Text('Failed to load boards: $e')),
      ),
      data: (boards) {
        if (boards.isEmpty) {
          return Scaffold(
            appBar: AppBar(
              title: Text(projectName == null || projectName.isEmpty
                  ? 'Project Board'
                  : '$projectName · Board'),
              actions: [
                ...CollabToolbar.entityActions(
                  context,
                  entityType: 'project',
                  entityId: projectId,
                ),
              ],
            ),
            body: Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  const Text('No board yet.'),
                  const SizedBox(height: 16),
                  ElevatedButton(
                    onPressed: () => _createBoardWithDefaults(context, ref),
                    child: const Text('Create board'),
                  ),
                ],
              ),
            ),
          );
        }

        final board = boards.first;
        final columnsAsync = ref.watch(ffiKanbanColumnsProvider(board.id));
        final cardsAsync = ref.watch(ffiKanbanCardsProvider((
          boardId: board.id,
          columnId: null,
          state: null,
          assigneeId: null,
          tagId: null,
        )));

        return columnsAsync.when(
          loading: () => const Scaffold(
            body: Center(child: CircularProgressIndicator()),
          ),
          error: (e, _) => Scaffold(
            body: Center(child: Text('Failed to load columns: $e')),
          ),
          data: (columns) {
            if (columns.isEmpty) {
              return Scaffold(
                appBar: AppBar(
                  title: Text(projectName == null || projectName.isEmpty
                      ? board.name
                      : '$projectName · ${board.name}'),
                  actions: [
                    ...CollabToolbar.entityActions(
                      context,
                      entityType: 'project',
                      entityId: projectId,
                    ),
                  ],
                ),
                body: Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      const Text('No columns yet.'),
                      const SizedBox(height: 16),
                      ElevatedButton(
                        onPressed: () => _createDefaultColumns(context, ref, board),
                        child: const Text('Create default columns'),
                      ),
                    ],
                  ),
                ),
              );
            }

            return cardsAsync.when(
              loading: () => const Scaffold(
                body: Center(child: CircularProgressIndicator()),
              ),
              error: (e, _) => Scaffold(
                body: Center(child: Text('Failed to load cards: $e')),
              ),
              data: (cards) {
                final sortedColumns = [...columns]
                  ..sort((a, b) => a.position.compareTo(b.position));

                final cardsByColumn = <String, List<FlutterKanbanCard>>{};
                for (final card in cards) {
                  cardsByColumn.putIfAbsent(card.columnId, () => []).add(card);
                }
                for (final entry in cardsByColumn.entries) {
                  entry.value.sort((a, b) => a.position.compareTo(b.position));
                }

                final defaultColumnId = sortedColumns.isNotEmpty ? sortedColumns.first.id : null;

                return Scaffold(
                  appBar: AppBar(
                    title: Text(projectName == null || projectName.isEmpty
                        ? board.name
                        : '$projectName · ${board.name}'),
                    actions: [
                      ...CollabToolbar.entityActions(
                        context,
                        entityType: 'project',
                        entityId: projectId,
                      ),
                      IconButton(
                        icon: const Icon(Icons.add),
                        onPressed: defaultColumnId == null
                            ? null
                            : () => _promptCreateCard(
                                  context,
                                  ref,
                                  boardId: board.id,
                                  columnId: defaultColumnId,
                                ),
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
                        for (final column in sortedColumns)
                          _buildFfiColumn(
                            context,
                            ref,
                            board,
                            column,
                            cardsByColumn[column.id] ?? const [],
                          ),
                      ],
                    ),
                  ),
                );
              },
            );
          },
        );
      },
    );
  }

  Widget _buildDemoColumn(String title, String columnId) {
    final cards = DemoData.kanbanCards.where((c) => c.column == columnId).toList();

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
                  return _buildDemoCard(cards[index]);
                },
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildDemoCard(DemoKanbanCard card) {
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

  Widget _buildFfiColumn(
    BuildContext context,
    WidgetRef ref,
    FlutterKanbanBoard board,
    FlutterKanbanColumn column,
    List<FlutterKanbanCard> cards,
  ) {
    return Container(
      width: 280,
      margin: const EdgeInsets.only(right: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            child: Row(
              children: [
                Text(
                  column.name,
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
                  onPressed: () => _promptCreateCard(
                    context,
                    ref,
                    boardId: board.id,
                    columnId: column.id,
                  ),
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(
                    minWidth: 28,
                    minHeight: 28,
                  ),
                ),
              ],
            ),
          ),
          Expanded(
            child: Container(
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: CommunitasColors.moss.withOpacity(0.5),
                borderRadius: BorderRadius.circular(8),
              ),
              child: ListView.builder(
                itemCount: cards.length,
                itemBuilder: (context, index) => _buildFfiCard(cards[index]),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildFfiCard(FlutterKanbanCard card) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: CommunitasColors.moss,
        borderRadius: BorderRadius.circular(8),
        border: Border(
          left: BorderSide(
            color: CommunitasColors.priorityMedium,
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
          if (card.description != null && card.description!.isNotEmpty) ...[
            const SizedBox(height: 8),
            Text(
              card.description!,
              style: TextStyle(
                fontSize: 12,
                color: CommunitasColors.cream.withOpacity(0.7),
              ),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
          ],
          const SizedBox(height: 12),
          Row(
            children: [
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  color: CommunitasColors.priorityMedium.withOpacity(0.2),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  'TASK',
                  style: TextStyle(
                    fontSize: 10,
                    fontWeight: FontWeight.w600,
                    color: CommunitasColors.priorityMedium,
                  ),
                ),
              ),
              const Spacer(),
              if (card.assignee != null && card.assignee!.isNotEmpty)
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

  Future<void> _createBoardWithDefaults(BuildContext context, WidgetRef ref) async {
    final controller = ref.read(ffiKanbanControllerProvider.notifier);
    final board = await controller.createBoard(
      entityId: projectId,
      boardName: 'Project Board',
    );
    if (board == null) return;
    await _createDefaultColumns(context, ref, board);
  }

  Future<void> _createDefaultColumns(
    BuildContext context,
    WidgetRef ref,
    FlutterKanbanBoard board,
  ) async {
    final controller = ref.read(ffiKanbanControllerProvider.notifier);
    for (var i = 0; i < _defaultColumns.length; i++) {
      await controller.createColumn(
        boardId: board.id,
        columnName: _defaultColumns[i],
        position: i,
      );
    }
    ref.invalidate(ffiKanbanColumnsProvider(board.id));
  }

  Future<void> _promptCreateCard(
    BuildContext context,
    WidgetRef ref, {
    required String boardId,
    required String columnId,
  }) async {
    final titleController = TextEditingController();
    final descriptionController = TextEditingController();
    final result = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('New card'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: titleController,
              decoration: const InputDecoration(hintText: 'Title'),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: descriptionController,
              decoration: const InputDecoration(hintText: 'Description'),
              maxLines: 4,
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('Create'),
          ),
        ],
      ),
    );

    if (result != true) {
      titleController.dispose();
      descriptionController.dispose();
      return;
    }

    final title = titleController.text.trim();
    final description = descriptionController.text.trim();
    titleController.dispose();
    descriptionController.dispose();

    if (title.isEmpty) return;
    final controller = ref.read(ffiKanbanControllerProvider.notifier);
    await controller.createCard(
      boardId: boardId,
      columnId: columnId,
      title: title,
      description: description.isEmpty ? null : description,
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
