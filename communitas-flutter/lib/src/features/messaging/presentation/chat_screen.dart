import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/colors.dart';
import '../../../shared/widgets/sidebar.dart';
import '../../../shared/widgets/adaptive_layout.dart';
import '../../../demo/demo_data.dart';

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

  @override
  void dispose() {
    _messageController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AdaptiveLayout(
      sidebar: const Sidebar(),
      body: Scaffold(
        appBar: AppBar(
          title: Text('#${widget.entityId}'),
          actions: [
            IconButton(
              icon: const Icon(Icons.phone),
              onPressed: () {},
              tooltip: 'Voice call',
            ),
            IconButton(
              icon: const Icon(Icons.videocam),
              onPressed: () {},
              tooltip: 'Video call',
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
              child: ListView.builder(
                controller: _scrollController,
                padding: const EdgeInsets.all(16),
                itemCount: DemoData.messages.length,
                itemBuilder: (context, index) {
                  final message = DemoData.messages[index];
                  return _buildMessage(message);
                },
              ),
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

  Widget _buildMessage(DemoMessage message) {
    final isMe = message.senderId == DemoData.demoIdentity.fourWords;

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
              color: CommunitasColors.jade,
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
                      return Container(
                        padding: const EdgeInsets.symmetric(
                          horizontal: 8,
                          vertical: 4,
                        ),
                        decoration: BoxDecoration(
                          color: CommunitasColors.fern,
                          borderRadius: BorderRadius.circular(12),
                        ),
                        child: Text(
                          '${_getEmoji(entry.key)} ${entry.value}',
                          style: const TextStyle(fontSize: 12),
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
                      // TODO: Open thread panel
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
    switch (name) {
      case 'thumbsup':
        return '👍';
      case 'heart':
        return '❤️';
      case 'laugh':
        return '😄';
      default:
        return '👍';
    }
  }

  void _sendMessage() {
    final text = _messageController.text.trim();
    if (text.isEmpty) return;

    // TODO: Send via FFI
    _messageController.clear();
  }
}
