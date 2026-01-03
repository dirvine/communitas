import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/theme/colors.dart';
import '../../../shared/widgets/sidebar.dart';
import '../../../shared/widgets/adaptive_layout.dart';
import '../../../demo/demo_data.dart';

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

  DemoContact? get contact {
    try {
      return DemoData.contacts.firstWhere(
        (c) => c.fourWords == widget.fourWords,
      );
    } catch (_) {
      return null;
    }
  }

  @override
  void dispose() {
    _messageController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final contactData = contact;

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
            IconButton(
              icon: const Icon(Icons.phone),
              onPressed: () {
                context.go('/call/${widget.fourWords}');
              },
              tooltip: 'Voice call',
            ),
            IconButton(
              icon: const Icon(Icons.videocam),
              onPressed: () {
                context.go('/call/${widget.fourWords}');
              },
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
            // Messages (placeholder)
            Expanded(
              child: Center(
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
              ),
            ),

            // Message input
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
                    onPressed: () {},
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
