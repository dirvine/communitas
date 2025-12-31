import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/colors.dart';
import '../../../shared/widgets/sidebar.dart';
import '../../../shared/widgets/adaptive_layout.dart';

/// Home screen with adaptive layout (sidebar on desktop, bottom nav on mobile).
class HomeScreen extends ConsumerWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return AdaptiveLayout(
      sidebar: const Sidebar(),
      body: _buildDashboard(context),
    );
  }

  Widget _buildDashboard(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Home'),
        actions: [
          IconButton(
            icon: const Icon(Icons.notifications_outlined),
            onPressed: () {},
          ),
          IconButton(
            icon: const Icon(Icons.settings_outlined),
            onPressed: () {},
          ),
        ],
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Welcome card
            Container(
              padding: const EdgeInsets.all(24),
              decoration: BoxDecoration(
                gradient: LinearGradient(
                  colors: [
                    CommunitasColors.jade.withOpacity(0.2),
                    CommunitasColors.moss,
                  ],
                  begin: Alignment.topLeft,
                  end: Alignment.bottomRight,
                ),
                borderRadius: BorderRadius.circular(16),
              ),
              child: Row(
                children: [
                  const Icon(
                    Icons.waving_hand,
                    size: 48,
                    color: CommunitasColors.amber,
                  ),
                  const SizedBox(width: 24),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Welcome to Communitas',
                          style: Theme.of(context).textTheme.headlineSmall,
                        ),
                        const SizedBox(height: 4),
                        Text(
                          'Your local-first collaboration hub',
                          style:
                              Theme.of(context).textTheme.bodyMedium?.copyWith(
                                    color: CommunitasColors.cream.withOpacity(0.7),
                                  ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 32),

            // Quick stats
            Text(
              'Overview',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 16),
            Wrap(
              spacing: 16,
              runSpacing: 16,
              children: [
                _buildStatCard(
                  context,
                  icon: Icons.business,
                  label: 'Organizations',
                  value: '2',
                  color: CommunitasColors.organization,
                ),
                _buildStatCard(
                  context,
                  icon: Icons.folder_outlined,
                  label: 'Projects',
                  value: '5',
                  color: CommunitasColors.project,
                ),
                _buildStatCard(
                  context,
                  icon: Icons.people_outline,
                  label: 'Contacts',
                  value: '12',
                  color: CommunitasColors.person,
                ),
                _buildStatCard(
                  context,
                  icon: Icons.message_outlined,
                  label: 'Unread',
                  value: '3',
                  color: CommunitasColors.jade,
                ),
              ],
            ),
            const SizedBox(height: 32),

            // Recent activity
            Text(
              'Recent Activity',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 16),
            _buildActivityItem(
              context,
              icon: Icons.message,
              title: 'New message in #general',
              subtitle: 'Alice: Hey everyone!',
              time: '2 min ago',
            ),
            _buildActivityItem(
              context,
              icon: Icons.check_circle,
              title: 'Task completed',
              subtitle: 'Implement login screen',
              time: '1 hour ago',
            ),
            _buildActivityItem(
              context,
              icon: Icons.person_add,
              title: 'Contact request',
              subtitle: 'Bob wants to connect',
              time: '3 hours ago',
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatCard(
    BuildContext context, {
    required IconData icon,
    required String label,
    required String value,
    required Color color,
  }) {
    return Container(
      width: 150,
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: CommunitasColors.moss,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(icon, color: color, size: 32),
          const SizedBox(height: 12),
          Text(
            value,
            style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                  color: color,
                  fontWeight: FontWeight.bold,
                ),
          ),
          Text(
            label,
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: CommunitasColors.cream.withOpacity(0.7),
                ),
          ),
        ],
      ),
    );
  }

  Widget _buildActivityItem(
    BuildContext context, {
    required IconData icon,
    required String title,
    required String subtitle,
    required String time,
  }) {
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: CommunitasColors.moss,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        children: [
          Container(
            padding: const EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: CommunitasColors.fern,
              borderRadius: BorderRadius.circular(8),
            ),
            child: Icon(icon, size: 20),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(title, style: const TextStyle(fontWeight: FontWeight.w500)),
                Text(
                  subtitle,
                  style: TextStyle(
                    color: CommunitasColors.cream.withOpacity(0.7),
                    fontSize: 12,
                  ),
                ),
              ],
            ),
          ),
          Text(
            time,
            style: TextStyle(
              color: CommunitasColors.cream.withOpacity(0.5),
              fontSize: 12,
            ),
          ),
        ],
      ),
    );
  }
}
