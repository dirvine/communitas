import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../core/router.dart';
import '../../core/theme/colors.dart';
import '../../services/navigation_state.dart';
import '../../services/unified_data_provider.dart';

class QuickSwitcherDialog extends ConsumerStatefulWidget {
  const QuickSwitcherDialog({super.key});

  @override
  ConsumerState<QuickSwitcherDialog> createState() => _QuickSwitcherDialogState();
}

class _QuickSwitcherDialogState extends ConsumerState<QuickSwitcherDialog> {
  static const double _itemExtent = 64;

  final _controller = TextEditingController();
  final _scrollController = ScrollController();

  int _selectedIndex = 0;
  List<_QuickItem> _items = const [];

  @override
  void initState() {
    super.initState();
    _controller.addListener(_onQueryChanged);
  }

  @override
  void dispose() {
    _controller
      ..removeListener(_onQueryChanged)
      ..dispose();
    _scrollController.dispose();
    super.dispose();
  }

  void _onQueryChanged() {
    setState(() {
      _selectedIndex = 0;
    });
  }

  @override
  Widget build(BuildContext context) {
    final query = _controller.text.trim().toLowerCase();
    final allEntitiesAsync = ref.watch(unifiedAllEntitiesProvider);
    final contactsAsync = ref.watch(unifiedContactsProvider);
    final recentKeys = ref.watch(recentEntitiesProvider);
    final recentContacts = ref.watch(recentContactsProvider);
    final starredKeys = ref.watch(starredEntitiesProvider);
    final starredContacts = ref.watch(starredContactsProvider);

    return Shortcuts(
      shortcuts: const {
        LogicalKeySet(LogicalKeyboardKey.arrowDown): _MoveSelectionIntent(1),
        LogicalKeySet(LogicalKeyboardKey.arrowUp): _MoveSelectionIntent(-1),
        LogicalKeySet(LogicalKeyboardKey.enter): _ActivateSelectionIntent(),
        LogicalKeySet(LogicalKeyboardKey.escape): _CloseSwitcherIntent(),
        LogicalKeySet(LogicalKeyboardKey.keyN, control: true): _MoveSelectionIntent(1),
        LogicalKeySet(LogicalKeyboardKey.keyP, control: true): _MoveSelectionIntent(-1),
      },
      child: Actions(
        actions: {
          _MoveSelectionIntent: CallbackAction<_MoveSelectionIntent>(
            onInvoke: (intent) {
              _moveSelection(intent.delta);
              return null;
            },
          ),
          _ActivateSelectionIntent: CallbackAction<_ActivateSelectionIntent>(
            onInvoke: (intent) {
              _activateSelection();
              return null;
            },
          ),
          _CloseSwitcherIntent: CallbackAction<_CloseSwitcherIntent>(
            onInvoke: (intent) {
              Navigator.of(context).pop();
              return null;
            },
          ),
        },
        child: Focus(
          autofocus: true,
          child: Dialog(
            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 560, maxHeight: 560),
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    TextField(
                      controller: _controller,
                      autofocus: true,
                      decoration: const InputDecoration(
                        hintText: 'Search entities and contacts...',
                        prefixIcon: Icon(Icons.search),
                        border: OutlineInputBorder(),
                      ),
                      onSubmitted: (_) => _activateSelection(),
                    ),
                    const SizedBox(height: 16),
                    Expanded(
                      child: allEntitiesAsync.when(
                        loading: () => const Center(child: CircularProgressIndicator()),
                        error: (e, _) => Center(
                          child: Text(
                            'Failed to load entities: $e',
                            style: const TextStyle(color: CommunitasColors.error),
                          ),
                        ),
                        data: (entities) {
                          return contactsAsync.when(
                            loading: () => const Center(child: CircularProgressIndicator()),
                            error: (e, _) => Center(
                              child: Text(
                                'Failed to load contacts: $e',
                                style: const TextStyle(color: CommunitasColors.error),
                              ),
                            ),
                            data: (contacts) {
                              if (entities.isEmpty && contacts.isEmpty) {
                                return const Center(child: Text('No results'));
                              }

                              final items = _filteredItems(
                                entities: entities,
                                contacts: contacts,
                                recentKeys: recentKeys,
                                recentContacts: recentContacts,
                                starredKeys: starredKeys,
                                starredContacts: starredContacts,
                                query: query,
                              );

                              _items = items;
                              if (_selectedIndex >= items.length) {
                                _selectedIndex = items.isEmpty ? 0 : items.length - 1;
                              }

                              if (items.isEmpty) {
                                return const Center(child: Text('No matches'));
                              }

                              return ListView.builder(
                                controller: _scrollController,
                                itemCount: items.length,
                                itemExtent: _itemExtent,
                                itemBuilder: (context, index) {
                                  final item = items[index];
                                  final isSelected = index == _selectedIndex;

                                  return MouseRegion(
                                    onEnter: (_) {
                                      setState(() {
                                        _selectedIndex = index;
                                      });
                                    },
                                    child: Container(
                                      margin: const EdgeInsets.symmetric(vertical: 2),
                                      decoration: BoxDecoration(
                                        color: isSelected
                                            ? CommunitasColors.fern.withOpacity(0.5)
                                            : Colors.transparent,
                                        borderRadius: BorderRadius.circular(8),
                                      ),
                                      child: ListTile(
                                        dense: true,
                                        selected: isSelected,
                                        leading: Icon(item.icon, color: item.iconColor),
                                        title: Text(item.label),
                                        subtitle: Text(item.subtitle),
                                        trailing: item.trailing,
                                        onTap: item.onSelect,
                                      ),
                                    ),
                                  );
                                },
                              );
                            },
                          );
                        },
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }

  void _moveSelection(int delta) {
    if (_items.isEmpty) return;
    setState(() {
      final next = _selectedIndex + delta;
      if (next < 0) {
        _selectedIndex = _items.length - 1;
      } else if (next >= _items.length) {
        _selectedIndex = 0;
      } else {
        _selectedIndex = next;
      }
    });
    _scrollToSelected();
  }

  void _activateSelection() {
    if (_items.isEmpty) return;
    final item = _items[_selectedIndex];
    item.onSelect();
  }

  void _scrollToSelected() {
    if (!_scrollController.hasClients) return;
    final target = _selectedIndex * _itemExtent;
    _scrollController.animateTo(
      target,
      duration: const Duration(milliseconds: 120),
      curve: Curves.easeOut,
    );
  }

  List<_QuickItem> _filteredItems({
    required List<UnifiedEntity> entities,
    required List<UnifiedContact> contacts,
    required List<String> recentKeys,
    required List<String> recentContacts,
    required Set<String> starredKeys,
    required Set<String> starredContacts,
    required String query,
  }) {
    if (query.isEmpty) {
      final recentEntities = _resolveEntityKeys(recentKeys, entities);
      final recentContactsResolved = _resolveContactKeys(recentContacts, contacts);
      final starredEntities = _resolveEntityKeys(starredKeys.toList(), entities)
          .where((entity) => !recentEntities.any((recent) => recent.id == entity.id))
          .toList();
      final starredContactsResolved = _resolveContactKeys(starredContacts.toList(), contacts)
          .where((contact) => !recentContactsResolved.any((recent) => recent.pubkeyHex == contact.pubkeyHex))
          .toList();

      final combined = [
        ...recentEntities.map((entity) => _entityItem(entity, starredKeys)),
        ...recentContactsResolved.map((contact) => _contactItem(contact, starredContacts)),
        ...starredEntities.map((entity) => _entityItem(entity, starredKeys)),
        ...starredContactsResolved.map((contact) => _contactItem(contact, starredContacts)),
      ];

      if (combined.isNotEmpty) {
        return combined;
      }

      final fallbackEntities = entities.take(8).map((entity) => _entityItem(entity, starredKeys));
      final fallbackContacts = contacts.take(4).map((contact) => _contactItem(contact, starredContacts));
      return [...fallbackEntities, ...fallbackContacts];
    }

    final filteredEntities = entities
        .where((entity) => entity.name.toLowerCase().contains(query))
        .toList();
    final filteredContacts = contacts
        .where((contact) {
          final name = contact.displayName.toLowerCase();
          final key = contact.pubkeyHex.toLowerCase();
          return name.contains(query) || key.contains(query);
        })
        .toList();

    final items = <_QuickItem>[];
    items.addAll(filteredEntities.map((entity) => _entityItem(entity, starredKeys)));
    items.addAll(filteredContacts.map((contact) => _contactItem(contact, starredContacts)));
    return items.take(20).toList();
  }

  List<UnifiedEntity> _resolveEntityKeys(
    List<String> keys,
    List<UnifiedEntity> entities,
  ) {
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

  List<UnifiedContact> _resolveContactKeys(
    List<String> keys,
    List<UnifiedContact> contacts,
  ) {
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

  _QuickItem _entityItem(UnifiedEntity entity, Set<String> starredKeys) {
    final isStarred = starredKeys.contains(entityKey(entity.type, entity.id));
    final label = entity.name;
    final subtitle = _entityLabel(entity.type);

    return _QuickItem(
      label: label,
      subtitle: subtitle,
      icon: _entityIcon(entity.type),
      iconColor: CommunitasColors.jade,
      trailing: isStarred
          ? const Icon(Icons.star, size: 16, color: CommunitasColors.amber)
          : null,
      onSelect: () {
        ref.read(recentEntitiesProvider.notifier).record(entityKey(entity.type, entity.id));
        context.go(
          Routes.entityDetail
              .replaceAll(':type', entity.type)
              .replaceAll(':id', entity.id),
        );
        Navigator.of(context).pop();
      },
    );
  }

  _QuickItem _contactItem(UnifiedContact contact, Set<String> starredContacts) {
    final isStarred = starredContacts.contains(contact.pubkeyHex);
    return _QuickItem(
      label: contact.displayName,
      subtitle: 'Contact - ${contact.status}',
      icon: Icons.person,
      iconColor: CommunitasColors.statusColor(contact.status),
      trailing: isStarred
          ? const Icon(Icons.star, size: 16, color: CommunitasColors.amber)
          : const Icon(Icons.chat_bubble_outline, size: 16),
      onSelect: () {
        ref.read(recentContactsProvider.notifier).record(contact.pubkeyHex);
        context.go(
          Routes.contactChat.replaceAll(':fourWords', contact.pubkeyHex),
        );
        Navigator.of(context).pop();
      },
    );
  }

  String _entityLabel(String type) {
    switch (type) {
      case 'organisation':
      case 'organization':
        return 'Organization';
      case 'project':
        return 'Project';
      case 'channel':
        return 'Channel';
      case 'group':
        return 'Group';
      default:
        return type;
    }
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

class _QuickItem {
  final String label;
  final String subtitle;
  final IconData icon;
  final Color iconColor;
  final Widget? trailing;
  final VoidCallback onSelect;

  const _QuickItem({
    required this.label,
    required this.subtitle,
    required this.icon,
    required this.iconColor,
    required this.onSelect,
    this.trailing,
  });
}

class _MoveSelectionIntent extends Intent {
  final int delta;

  const _MoveSelectionIntent(this.delta);
}

class _ActivateSelectionIntent extends Intent {
  const _ActivateSelectionIntent();
}

class _CloseSwitcherIntent extends Intent {
  const _CloseSwitcherIntent();
}
