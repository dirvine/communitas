import 'package:flutter/material.dart';

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
    return NavigationBar(
      selectedIndex: 0,
      onDestinationSelected: (index) {
        // TODO: Handle navigation
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
}
