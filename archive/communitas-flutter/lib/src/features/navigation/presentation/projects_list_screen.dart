import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/router.dart';
import '../../../core/theme/colors.dart';
import '../../../shared/widgets/adaptive_layout.dart';
import '../../../shared/widgets/sidebar.dart';
import '../../../services/unified_data_provider.dart';
import '../../../services/navigation_state.dart';

class ProjectsListScreen extends ConsumerWidget {
  const ProjectsListScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final projectsAsync = ref.watch(unifiedProjectsProvider);

    return AdaptiveLayout(
      sidebar: const Sidebar(),
      body: Scaffold(
        appBar: AppBar(
          title: const Text('Projects'),
        ),
        body: projectsAsync.when(
          loading: () => const Center(child: CircularProgressIndicator()),
          error: (e, _) => Center(
            child: Text('Failed to load projects: $e'),
          ),
          data: (projects) {
            if (projects.isEmpty) {
              return const Center(child: Text('No projects yet'));
            }

            return ListView.separated(
              padding: const EdgeInsets.all(16),
              itemCount: projects.length,
              separatorBuilder: (_, __) => const Divider(height: 1),
              itemBuilder: (context, index) {
                final project = projects[index];
                return ListTile(
                  leading: const Icon(Icons.folder, color: CommunitasColors.project),
                  title: Text(project.name),
                  subtitle: Text('${project.memberCount} members'),
                  onTap: () {
                    ref
                        .read(recentEntitiesProvider.notifier)
                        .record(entityKey(project.type, project.id));
                    context.go(
                      '${Routes.entityDetail.replaceAll(':type', project.type).replaceAll(':id', project.id)}',
                    );
                  },
                );
              },
            );
          },
        ),
      ),
    );
  }
}
