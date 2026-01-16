import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../core/router.dart';

/// Adaptive layout that shows sidebar on desktop and bottom nav on mobile.
class AdaptiveLayout extends StatelessWidget {
  final Widget sidebar;
  final Widget body;

  const AdaptiveLayout({
    super.key,
    required this.sidebar,
    required this.body,
  });

  @override
  Widget build(BuildContext context) {
    final screenWidth = MediaQuery.of(context).size.width;
    final isDesktop = screenWidth >= 768;

    if (isDesktop) {
      // Desktop: sidebar + body
      // Use LayoutBuilder to get exact height for sidebar
      return LayoutBuilder(
        builder: (context, constraints) {
          return Row(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              SizedBox(
                height: constraints.maxHeight,
                child: sidebar,
              ),
              const VerticalDivider(width: 1),
              Expanded(child: body),
            ],
          );
        },
      );
    } else {
      // Mobile: body with bottom navigation
      return Scaffold(
        body: body,
        bottomNavigationBar: _buildBottomNav(context),
      );
    }
  }

  Widget _buildBottomNav(BuildContext context) {
    final location = GoRouterState.of(context).uri.path;
    final selectedIndex = _indexForLocation(location);

    return NavigationBar(
      selectedIndex: selectedIndex,
      onDestinationSelected: (index) {
        final route = _routeForIndex(index);
        if (route != null) {
          context.go(route);
        }
      },
      destinations: const [
        NavigationDestination(
          icon: Icon(Icons.home_outlined),
          selectedIcon: Icon(Icons.home),
          label: 'Home',
        ),
        NavigationDestination(
          icon: Icon(Icons.chat_bubble_outline),
          selectedIcon: Icon(Icons.chat_bubble),
          label: 'Messages',
        ),
        NavigationDestination(
          icon: Icon(Icons.folder_outlined),
          selectedIcon: Icon(Icons.folder),
          label: 'Projects',
        ),
        NavigationDestination(
          icon: Icon(Icons.people_outline),
          selectedIcon: Icon(Icons.people),
          label: 'Contacts',
        ),
        NavigationDestination(
          icon: Icon(Icons.more_horiz),
          label: 'More',
        ),
      ],
    );
  }

  int _indexForLocation(String location) {
    if (location.startsWith(Routes.messages) || location.startsWith('/entity')) {
      return 1;
    }
    if (location.startsWith(Routes.projects) || location.startsWith('/project')) {
      return 2;
    }
    if (location.startsWith(Routes.contacts) || location.startsWith('/contact')) {
      return 3;
    }
    if (location.startsWith(Routes.more) || location.startsWith(Routes.network)) {
      return 4;
    }
    return 0;
  }

  String? _routeForIndex(int index) {
    switch (index) {
      case 0:
        return Routes.home;
      case 1:
        return Routes.messages;
      case 2:
        return Routes.projects;
      case 3:
        return Routes.contacts;
      case 4:
        return Routes.more;
      default:
        return Routes.home;
    }
  }
}
