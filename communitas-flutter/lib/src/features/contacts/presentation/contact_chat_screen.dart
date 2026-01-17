import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/colors.dart';
import '../../../shared/widgets/sidebar.dart';
import '../../../shared/widgets/adaptive_layout.dart';
import '../../../shared/widgets/collab_toolbar.dart';
import '../../../services/ffi_provider.dart';
import '../../../services/navigation_state.dart';
import '../../../services/unified_data_provider.dart';

/// Direct message chat screen for 1:1 contact conversations.
class ContactChatScreen extends ConsumerStatefulWidget {
  final String fourWords;

  const ContactChatScreen({
    super.key,
    required this.fourWords,
  });

  @override
  ConsumerState<ContactChatScreen> createState() => _ContactChatScreenState();
}

class _ContactChatScreenState extends ConsumerState<ContactChatScreen> {
  final _messageController = TextEditingController();

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(recentContactsProvider.notifier).record(widget.fourWords);
    });
  }

  @override
  void dispose() {
    _messageController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final contactsAsync = ref.watch(unifiedContactsProvider);
    final contactData = contactsAsync.maybeWhen(
      data: (contacts) {
        for (final contact in contacts) {
          if (contact.pubkeyHex == widget.fourWords) {
            return contact;
          }
        }
        return null;
      },
      orElse: () => null,
    );
    final messagesAsync =
        ref.watch(unifiedDirectMessagesProvider(widget.fourWords));

    return AdaptiveLayout(
      sidebar: const Sidebar(),
      body: Scaffold(
        appBar: AppBar(
          title: Row(
            children: [
              Container(
                width: 32,
                height: 32,
                decoration: BoxDecoration(
                  color: CommunitasColors.jade,
                  borderRadius: BorderRadius.circular(16),
                ),
                child: Center(
                  child: Text(
                    contactData?.displayName[0].toUpperCase() ?? '?',
                    style: const TextStyle(
                      color: CommunitasColors.cream,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
              ),
              const SizedBox(width: 12),
              Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    contactData?.displayName ?? 'Unknown',
                    style: const TextStyle(fontSize: 16),
                  ),
                  Row(
                    children: [
                      Container(
                        width: 8,
                        height: 8,
                        decoration: BoxDecoration(
                          color: CommunitasColors.statusColor(
                            contactData?.status ?? 'offline',
                          ),
                          shape: BoxShape.circle,
                        ),
                      ),
                      const SizedBox(width: 4),
                      Text(
                        contactData?.status ?? 'offline',
                        style: TextStyle(
                          fontSize: 12,
                          color: CommunitasColors.cream.withOpacity(0.7),
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ],
          ),
          actions: [
            ...CollabToolbar.contactActions(
              context,
              contactId: widget.fourWords,
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
            Expanded(
              child: messagesAsync.when(
                loading: () => const Center(child: CircularProgressIndicator()),
                error: (e, _) => Center(child: Text('Error loading messages: $e')),
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
                            'Start a conversation with ${contactData?.displayName ?? "this contact"}',
                            style: TextStyle(
                              color: CommunitasColors.cream.withOpacity(0.5),
                            ),
                          ),
                        ],
                      ),
                    );
                  }

                  return ListView.builder(
                    padding: const EdgeInsets.all(16),
                    itemCount: messages.length,
                    itemBuilder: (context, index) {
                      return _buildMessageTile(messages[index]);
                    },
                  );
                },
              ),
            ),
            Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: CommunitasColors.moss,
                border: Border(
                  top: BorderSide(color: CommunitasColors.fern),
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
                    ),
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

  Widget _buildMessageTile(UnifiedMessage message) {
    final identity = ref.watch(unifiedIdentityProvider);
    final isMe = message.senderId == identity.pubkeyHex ||
        message.senderId == identity.fourWords ||
        message.senderName == identity.displayName;

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
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
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
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
                Text(message.content),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _sendMessage() async {
    final text = _messageController.text.trim();
    if (text.isEmpty) return;

    _messageController.clear();

    if (!_ensureFfiAvailable()) return;

    final sender = ref.read(ffiMessageControllerProvider.notifier);
    await sender.sendDirectMessage(
      recipients: [widget.fourWords],
      text: text,
    );
    ref.invalidate(unifiedDirectMessagesProvider(widget.fourWords));
  }

  bool _ensureFfiAvailable() {
    final api = ref.read(communitasApiProvider);
    if (api == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Messaging is unavailable without the native backend.'),
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
}
