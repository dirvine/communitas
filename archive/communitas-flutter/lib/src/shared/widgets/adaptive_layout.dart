import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:go_router/go_router.dart';

import '../../core/router.dart';
import 'quick_switcher_dialog.dart';

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
    final scaffold = _extractScaffold(body);
    final content = isDesktop
        ? Row(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              SizedBox(
                height: double.infinity,
                child: sidebar,
              ),
              const VerticalDivider(width: 1),
              Expanded(child: scaffold.body ?? const SizedBox.shrink()),
            ],
          )
        : (scaffold.body ?? const SizedBox.shrink());

    final mergedScaffold = Scaffold(
      appBar: scaffold.appBar,
      body: content,
      bottomNavigationBar: isDesktop
          ? scaffold.bottomNavigationBar
          : (scaffold.bottomNavigationBar ?? _buildBottomNav(context)),
      floatingActionButton: scaffold.floatingActionButton,
      floatingActionButtonLocation: scaffold.floatingActionButtonLocation,
      floatingActionButtonAnimator: scaffold.floatingActionButtonAnimator,
      drawer: scaffold.drawer,
      endDrawer: scaffold.endDrawer,
      drawerEnableOpenDragGesture: scaffold.drawerEnableOpenDragGesture,
      endDrawerEnableOpenDragGesture: scaffold.endDrawerEnableOpenDragGesture,
      bottomSheet: scaffold.bottomSheet,
      backgroundColor: scaffold.backgroundColor,
      resizeToAvoidBottomInset: scaffold.resizeToAvoidBottomInset,
      primary: scaffold.primary,
      extendBody: scaffold.extendBody,
      extendBodyBehindAppBar: scaffold.extendBodyBehindAppBar,
    );

    return Shortcuts(
      shortcuts: _navigationShortcuts(),
      child: Actions(
        actions: {
          _NavigateIntent: CallbackAction<_NavigateIntent>(
            onInvoke: (intent) {
              context.go(intent.route);
              return null;
            },
          ),
          _OpenSwitcherIntent: CallbackAction<_OpenSwitcherIntent>(
            onInvoke: (intent) {
              showDialog<void>(
                context: context,
                builder: (context) => const QuickSwitcherDialog(),
              );
              return null;
            },
          ),
        },
        child: Focus(
          autofocus: true,
          child: mergedScaffold,
        ),
      ),
    );
  }

  Scaffold _extractScaffold(Widget body) {
    if (body is Scaffold) {
      return body;
    }
    return Scaffold(body: body);
  }

  Map<ShortcutActivator, Intent> _navigationShortcuts() {
    return const {
      SingleActivator(LogicalKeyboardKey.digit1, control: true):
          _NavigateIntent(Routes.home),
      SingleActivator(LogicalKeyboardKey.digit1, meta: true):
          _NavigateIntent(Routes.home),
      SingleActivator(LogicalKeyboardKey.digit2, control: true):
          _NavigateIntent(Routes.messages),
      SingleActivator(LogicalKeyboardKey.digit2, meta: true):
          _NavigateIntent(Routes.messages),
      SingleActivator(LogicalKeyboardKey.digit3, control: true):
          _NavigateIntent(Routes.projects),
      SingleActivator(LogicalKeyboardKey.digit3, meta: true):
          _NavigateIntent(Routes.projects),
      SingleActivator(LogicalKeyboardKey.digit4, control: true):
          _NavigateIntent(Routes.contacts),
      SingleActivator(LogicalKeyboardKey.digit4, meta: true):
          _NavigateIntent(Routes.contacts),
      SingleActivator(LogicalKeyboardKey.digit5, control: true):
          _NavigateIntent(Routes.more),
      SingleActivator(LogicalKeyboardKey.digit5, meta: true):
          _NavigateIntent(Routes.more),
      SingleActivator(LogicalKeyboardKey.keyK, control: true):
          _OpenSwitcherIntent(),
      SingleActivator(LogicalKeyboardKey.keyK, meta: true):
          _OpenSwitcherIntent(),
    };
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

class _NavigateIntent extends Intent {
  final String route;

  const _NavigateIntent(this.route);
}

class _OpenSwitcherIntent extends Intent {
  const _OpenSwitcherIntent();
}
