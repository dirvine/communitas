import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/colors.dart';
import '../../../shared/widgets/sidebar.dart';
import '../../../shared/widgets/adaptive_layout.dart';
import '../../../shared/widgets/collab_toolbar.dart';
import '../../../services/unified_data_provider.dart';
import '../../../services/navigation_state.dart';
import '../../../services/ffi_provider.dart';
import '../../../bindings/api_exports.dart';

/// Chat screen for entity messaging.
class ChatScreen extends ConsumerStatefulWidget {
  final String entityType;
  final String entityId;

  const ChatScreen({
    super.key,
    required this.entityType,
    required this.entityId,
  });

  @override
  ConsumerState<ChatScreen> createState() => _ChatScreenState();
}

class _ChatScreenState extends ConsumerState<ChatScreen> {
  final _messageController = TextEditingController();
  final _scrollController = ScrollController();
  static const _reactionOptions = <String, String>{
    'thumbsup': '👍',
    'heart': '❤️',
    'laugh': '😄',
  };

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref
          .read(recentEntitiesProvider.notifier)
          .record(entityKey(widget.entityType, widget.entityId));
    });
  }

  @override
  void dispose() {
    _messageController.dispose();
    _scrollController.dispose();
    super.dispose();
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
            loading: () => Text(_fallbackTitle()),
            error: (_, __) => Text(_fallbackTitle()),
            data: (entity) {
              final name = entity?.name ?? widget.entityId;
              final prefix = widget.entityType == 'channel' ? '#' : '';
              return Text('$prefix$name');
            },
          ),
          actions: [
            ...CollabToolbar.entityActions(
              context,
              entityType: widget.entityType,
              entityId: widget.entityId,
              onVoice: _showCallsUnavailable,
              onVideo: _showCallsUnavailable,
              onShare: _showCallsUnavailable,
            ),
            IconButton(
              icon: const Icon(Icons.more_vert),
              onPressed: () {},
            ),
          ],
        ),
        body: Column(
          children: [
            // Messages list
            Expanded(
              child: _buildMessageList(),
            ),

            // Message input
            Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: CommunitasColors.moss,
                border: Border(
                  top: BorderSide(
                    color: CommunitasColors.fern,
                  ),
                ),
              ),
              child: Row(
                children: [
                  IconButton(
                    icon: const Icon(Icons.add),
                    onPressed: () {},
                  ),
                  Expanded(
                    child: TextField(
                      controller: _messageController,
                      decoration: const InputDecoration(
                        hintText: 'Type a message...',
                        border: InputBorder.none,
                      ),
                      maxLines: null,
                    ),
                  ),
                  IconButton(
                    icon: const Icon(Icons.emoji_emotions_outlined),
                    onPressed: () {},
                  ),
                  IconButton(
                    icon: const Icon(Icons.send),
                    color: CommunitasColors.jade,
                    onPressed: _sendMessage,
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  String _fallbackTitle() {
    final prefix = widget.entityType == 'channel' ? '#' : '';
    return '$prefix${widget.entityId}';
  }

  Widget _buildMessageList() {
    final messagesAsync = ref.watch(unifiedMessagesProvider(widget.entityId));

    return messagesAsync.when(
      loading: () => const Center(
        child: CircularProgressIndicator(),
      ),
      error: (e, _) => Center(
        child: Text('Error loading messages: $e'),
      ),
      data: (messages) {
        if (messages.isEmpty) {
          return Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Icon(
                  Icons.chat_bubble_outline,
                  size: 64,
                  color: CommunitasColors.cream.withOpacity(0.3),
                ),
                const SizedBox(height: 16),
                Text(
                  'No messages yet',
                  style: TextStyle(
                    color: CommunitasColors.cream.withOpacity(0.5),
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  'Start the conversation!',
                  style: TextStyle(
                    fontSize: 12,
                    color: CommunitasColors.cream.withOpacity(0.3),
                  ),
                ),
              ],
            ),
          );
        }

        return ListView.builder(
          controller: _scrollController,
          padding: const EdgeInsets.all(16),
          itemCount: messages.length,
          itemBuilder: (context, index) {
            final message = messages[index];
            return _buildMessage(message, messages);
          },
        );
      },
    );
  }

  Widget _buildMessage(UnifiedMessage message, List<UnifiedMessage> allMessages) {
    final identity = ref.watch(unifiedIdentityProvider);
    final isMe = message.senderId == identity.pubkeyHex ||
        message.senderId == identity.fourWords ||
        message.senderName == identity.displayName;

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Avatar
          Container(
            width: 36,
            height: 36,
            decoration: BoxDecoration(
              color: isMe ? CommunitasColors.jade : CommunitasColors.fern,
              borderRadius: BorderRadius.circular(18),
            ),
            child: Center(
              child: Text(
                message.senderName[0].toUpperCase(),
                style: const TextStyle(
                  color: CommunitasColors.cream,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ),
          ),
          const SizedBox(width: 12),

          // Message content
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Header
                Row(
                  children: [
                    Text(
                      message.senderName,
                      style: const TextStyle(fontWeight: FontWeight.w600),
                    ),
                    const SizedBox(width: 8),
                    Text(
                      message.timestamp,
                      style: TextStyle(
                        fontSize: 12,
                        color: CommunitasColors.cream.withOpacity(0.5),
                      ),
                    ),
                    if (message.editedAt != null) ...[
                      const SizedBox(width: 6),
                      Text(
                        '(edited)',
                        style: TextStyle(
                          fontSize: 11,
                          color: CommunitasColors.cream.withOpacity(0.4),
                        ),
                      ),
                    ],
                    const Spacer(),
                    PopupMenuButton<String>(
                      icon: const Icon(Icons.more_horiz, size: 18),
                      onSelected: (value) => _handleMessageAction(value, message, isMe),
                      itemBuilder: (context) => [
                        const PopupMenuItem(value: 'reply', child: Text('Reply')),
                        const PopupMenuItem(value: 'react', child: Text('Add reaction')),
                        if (isMe)
                          const PopupMenuItem(value: 'edit', child: Text('Edit message')),
                        if (isMe)
                          const PopupMenuItem(value: 'delete', child: Text('Delete message')),
                      ],
                    ),
                  ],
                ),
                const SizedBox(height: 4),

                // Content
                Text(message.content),

                // Reactions
                if (message.reactions.isNotEmpty) ...[
                  const SizedBox(height: 8),
                  Wrap(
                    spacing: 4,
                    children: message.reactions.entries.map((entry) {
                      final reacted = message.userReactions.contains(entry.key);
                      return InkWell(
                        onTap: () => _toggleReaction(message, entry.key),
                        child: Container(
                          padding: const EdgeInsets.symmetric(
                            horizontal: 8,
                            vertical: 4,
                          ),
                          decoration: BoxDecoration(
                            color: reacted ? CommunitasColors.jade : CommunitasColors.fern,
                            borderRadius: BorderRadius.circular(12),
                          ),
                          child: Text(
                            '${_getEmoji(entry.key)} ${entry.value}',
                            style: const TextStyle(fontSize: 12),
                          ),
                        ),
                      );
                    }).toList(),
                  ),
                ],

                // Thread indicator
                if (message.hasThread) ...[
                  const SizedBox(height: 8),
                  InkWell(
                    onTap: () {
                      _openThreadPanel(message, allMessages);
                    },
                    child: Text(
                      '${message.threadReplyCount} replies',
                      style: const TextStyle(
                        color: CommunitasColors.jade,
                        fontSize: 12,
                      ),
                    ),
                  ),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }

  String _getEmoji(String name) {
    return _reactionOptions[name] ?? '👍';
  }

  Future<void> _sendMessage({String? replyToId}) async {
    final text = _messageController.text.trim();
    if (text.isEmpty) return;

    _messageController.clear();

    if (!_ensureFfiAvailable()) return;

    final sender = ref.read(ffiMessageControllerProvider.notifier);
    final entityType = _parseEntityType(widget.entityType);
    await sender.sendMessage(
      entityId: widget.entityId,
      entityType: entityType,
      text: text,
      replyToId: replyToId,
    );
    ref.invalidate(unifiedMessagesProvider(widget.entityId));
  }

  bool _ensureFfiAvailable() {
    final api = ref.read(communitasApiProvider);
    if (api == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Please log in to send messages.'),
        ),
      );
      return false;
    }
    return true;
  }

  void _showCallsUnavailable() {
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Calls are not yet available in the Flutter UI.'),
      ),
    );
  }

  Future<void> _handleMessageAction(
    String action,
    UnifiedMessage message,
    bool isMe,
  ) async {
    switch (action) {
      case 'reply':
        await _promptReply(message);
        break;
      case 'react':
        await _showReactionPicker(message);
        break;
      case 'edit':
        if (isMe) {
          await _promptEdit(message);
        }
        break;
      case 'delete':
        if (isMe) {
          await _confirmDelete(message);
        }
        break;
    }
  }

  Future<void> _promptReply(UnifiedMessage message) async {
    if (!_ensureFfiAvailable()) return;
    final controller = TextEditingController();
    final result = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Reply'),
        content: TextField(
          controller: controller,
          decoration: const InputDecoration(hintText: 'Type your reply...'),
          maxLines: null,
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(controller.text.trim()),
            child: const Text('Send'),
          ),
        ],
      ),
    );
    controller.dispose();
    if (result == null || result.isEmpty) return;

    final sender = ref.read(ffiMessageControllerProvider.notifier);
    final entityType = _parseEntityType(widget.entityType);
    await sender.sendMessage(
      entityId: widget.entityId,
      entityType: entityType,
      text: result,
      replyToId: message.id,
    );
    ref.invalidate(unifiedMessagesProvider(widget.entityId));
  }

  Future<void> _promptEdit(UnifiedMessage message) async {
    if (!_ensureFfiAvailable()) return;
    final controller = TextEditingController(text: message.content);
    final result = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Edit message'),
        content: TextField(
          controller: controller,
          decoration: const InputDecoration(hintText: 'Update your message...'),
          maxLines: null,
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(controller.text.trim()),
            child: const Text('Save'),
          ),
        ],
      ),
    );
    controller.dispose();
    if (result == null || result.isEmpty || result == message.content) return;

    final sender = ref.read(ffiMessageControllerProvider.notifier);
    final entityType = _parseEntityType(widget.entityType);
    await sender.editMessage(
      entityId: widget.entityId,
      entityType: entityType,
      messageId: message.id,
      newText: result,
    );
    ref.invalidate(unifiedMessagesProvider(widget.entityId));
  }

  Future<void> _confirmDelete(UnifiedMessage message) async {
    if (!_ensureFfiAvailable()) return;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete message'),
        content: const Text('This cannot be undone. Delete this message?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('Delete'),
          ),
        ],
      ),
    );

    if (confirmed != true) return;

    final sender = ref.read(ffiMessageControllerProvider.notifier);
    final entityType = _parseEntityType(widget.entityType);
    await sender.deleteMessage(
      entityId: widget.entityId,
      entityType: entityType,
      messageId: message.id,
    );
    ref.invalidate(unifiedMessagesProvider(widget.entityId));
  }

  Future<void> _showReactionPicker(UnifiedMessage message) async {
    if (!_ensureFfiAvailable()) return;
    final selected = await showModalBottomSheet<String>(
      context: context,
      backgroundColor: CommunitasColors.moss,
      builder: (context) {
        return Padding(
          padding: const EdgeInsets.all(16),
          child: Wrap(
            spacing: 12,
            children: _reactionOptions.entries.map((entry) {
              return GestureDetector(
                onTap: () => Navigator.of(context).pop(entry.key),
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                  decoration: BoxDecoration(
                    color: CommunitasColors.fern,
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Text(
                    entry.value,
                    style: const TextStyle(fontSize: 20),
                  ),
                ),
              );
            }).toList(),
          ),
        );
      },
    );

    if (selected == null) return;
    await _toggleReaction(message, selected);
  }

  Future<void> _toggleReaction(UnifiedMessage message, String emojiKey) async {
    if (!_ensureFfiAvailable()) return;
    final sender = ref.read(ffiMessageControllerProvider.notifier);
    final entityType = _parseEntityType(widget.entityType);
    final hasReacted = message.userReactions.contains(emojiKey);

    if (hasReacted) {
      await sender.removeReaction(
        entityId: widget.entityId,
        entityType: entityType,
        messageId: message.id,
        emoji: emojiKey,
      );
    } else {
      await sender.addReaction(
        entityId: widget.entityId,
        entityType: entityType,
        messageId: message.id,
        emoji: emojiKey,
      );
    }

    ref.invalidate(unifiedMessagesProvider(widget.entityId));
  }

  void _openThreadPanel(UnifiedMessage parent, List<UnifiedMessage> allMessages) {
    final threadMessages =
        allMessages.where((m) => m.replyToId == parent.id).toList();
    if (threadMessages.isEmpty) return;

    showModalBottomSheet<void>(
      context: context,
      backgroundColor: CommunitasColors.moss,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(16)),
      ),
      builder: (context) {
        return Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Thread',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 12),
              Expanded(
                child: ListView.builder(
                  itemCount: threadMessages.length,
                  itemBuilder: (context, index) {
                    final message = threadMessages[index];
                    return Padding(
                      padding: const EdgeInsets.symmetric(vertical: 8),
                      child: Text('${message.senderName}: ${message.content}'),
                    );
                  },
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  FlutterEntityType _parseEntityType(String raw) {
    switch (raw) {
      case 'organisation':
      case 'organization':
        return FlutterEntityType.organisation;
      case 'project':
        return FlutterEntityType.project;
      case 'group':
        return FlutterEntityType.group;
      case 'person':
        return FlutterEntityType.person;
      case 'channel':
      default:
        return FlutterEntityType.channel;
    }
  }
}
