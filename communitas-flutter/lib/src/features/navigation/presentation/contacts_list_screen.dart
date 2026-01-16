import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/router.dart';
import '../../../core/theme/colors.dart';
import '../../../shared/widgets/adaptive_layout.dart';
import '../../../shared/widgets/sidebar.dart';
import '../../../services/unified_data_provider.dart';

class ContactsListScreen extends ConsumerWidget {
  const ContactsListScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final contactsAsync = ref.watch(unifiedContactsProvider);

    return AdaptiveLayout(
      sidebar: const Sidebar(),
      body: Scaffold(
        appBar: AppBar(
          title: const Text('Contacts'),
        ),
        body: contactsAsync.when(
          loading: () => const Center(child: CircularProgressIndicator()),
          error: (e, _) => Center(child: Text('Failed to load contacts: $e')),
          data: (contacts) {
            if (contacts.isEmpty) {
              return const Center(child: Text('No contacts yet'));
            }

            return ListView.separated(
              padding: const EdgeInsets.all(16),
              itemCount: contacts.length,
              separatorBuilder: (_, __) => const Divider(height: 1),
              itemBuilder: (context, index) {
                final contact = contacts[index];
                return ListTile(
                  leading: CircleAvatar(
                    backgroundColor: CommunitasColors.person,
                    child: Text(
                      contact.displayName.isNotEmpty
                          ? contact.displayName[0].toUpperCase()
                          : '?',
                      style: const TextStyle(color: CommunitasColors.cream),
                    ),
                  ),
                  title: Text(contact.displayName),
                  subtitle: Text(contact.status),
                  onTap: () => context.go(
                    Routes.contactChat.replaceAll(':fourWords', contact.pubkeyHex),
                  ),
                );
              },
            );
          },
        ),
      ),
    );
  }
}
