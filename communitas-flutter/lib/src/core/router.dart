import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../features/auth/presentation/login_screen.dart';
import '../features/auth/presentation/create_identity_screen.dart';
import '../features/auth/presentation/recover_identity_screen.dart';
import '../features/home/presentation/home_screen.dart';
import '../features/entities/presentation/entity_detail_screen.dart';
import '../features/messaging/presentation/chat_screen.dart';
import '../features/kanban/presentation/kanban_board_screen.dart';
import '../features/drive/presentation/drive_browser_screen.dart';
import '../features/contacts/presentation/contact_chat_screen.dart';
import '../features/calls/presentation/active_call_screen.dart';
import '../features/network/presentation/network_panel_screen.dart';
import '../features/auth/providers/auth_provider.dart';

/// Navigation routes for Communitas
class Routes {
  Routes._();

  static const String login = '/login';
  static const String createIdentity = '/create-identity';
  static const String recoverIdentity = '/recover-identity';
  static const String home = '/';
  static const String entityDetail = '/entity/:type/:id';
  static const String entityChat = '/entity/:type/:id/chat';
  static const String entityDrive = '/entity/:type/:id/drive';
  static const String projectBoard = '/project/:id/board';
  static const String contactChat = '/contact/:fourWords/chat';
  static const String activeCall = '/call/:fourWords';
  static const String network = '/network';
}

/// Notifier that triggers router refresh when auth state changes.
class _RouterNotifier extends ChangeNotifier {
  _RouterNotifier(this._ref) {
    _ref.listen(authNotifierProvider, (_, __) => notifyListeners());
  }

  final Ref _ref;

  bool get isAuthenticated => _ref.read(authNotifierProvider).isAuthenticated;
}

/// Router notifier provider.
final _routerNotifierProvider = Provider<_RouterNotifier>((ref) {
  return _RouterNotifier(ref);
});

/// Router provider with auth-aware redirects.
/// Uses refreshListenable pattern to avoid recreating router on auth changes.
final routerProvider = Provider<GoRouter>((ref) {
  final notifier = ref.watch(_routerNotifierProvider);

  return GoRouter(
    initialLocation: Routes.home, // Start at home, redirect handles auth
    debugLogDiagnostics: true,
    refreshListenable: notifier,
    redirect: (context, state) {
      final isLoggedIn = notifier.isAuthenticated;
      final isLoggingIn = state.matchedLocation == Routes.login ||
          state.matchedLocation == Routes.createIdentity ||
          state.matchedLocation == Routes.recoverIdentity;

      // If not logged in and not on auth pages, redirect to login
      if (!isLoggedIn && !isLoggingIn) {
        return Routes.login;
      }

      // If logged in and on auth pages, redirect to home
      if (isLoggedIn && isLoggingIn) {
        return Routes.home;
      }

      return null; // No redirect
    },
    routes: [
      // Auth routes
      GoRoute(
        path: Routes.login,
        name: 'login',
        builder: (context, state) => const LoginScreen(),
      ),
      GoRoute(
        path: Routes.createIdentity,
        name: 'createIdentity',
        builder: (context, state) => const CreateIdentityScreen(),
      ),
      GoRoute(
        path: Routes.recoverIdentity,
        name: 'recoverIdentity',
        builder: (context, state) => const RecoverIdentityScreen(),
      ),

      // Main app routes (require auth)
      GoRoute(
        path: Routes.home,
        name: 'home',
        builder: (context, state) => const HomeScreen(),
      ),
      GoRoute(
        path: Routes.entityDetail,
        name: 'entityDetail',
        builder: (context, state) {
          final type = state.pathParameters['type']!;
          final id = state.pathParameters['id']!;
          return EntityDetailScreen(entityType: type, entityId: id);
        },
      ),
      GoRoute(
        path: Routes.entityChat,
        name: 'entityChat',
        builder: (context, state) {
          final type = state.pathParameters['type']!;
          final id = state.pathParameters['id']!;
          return ChatScreen(entityType: type, entityId: id);
        },
      ),
      GoRoute(
        path: Routes.entityDrive,
        name: 'entityDrive',
        builder: (context, state) {
          final type = state.pathParameters['type']!;
          final id = state.pathParameters['id']!;
          return DriveBrowserScreen(entityType: type, entityId: id);
        },
      ),
      GoRoute(
        path: Routes.projectBoard,
        name: 'projectBoard',
        builder: (context, state) {
          final id = state.pathParameters['id']!;
          return KanbanBoardScreen(projectId: id);
        },
      ),
      GoRoute(
        path: Routes.contactChat,
        name: 'contactChat',
        builder: (context, state) {
          final fourWords = state.pathParameters['fourWords']!;
          return ContactChatScreen(fourWords: fourWords);
        },
      ),
      GoRoute(
        path: Routes.activeCall,
        name: 'activeCall',
        builder: (context, state) {
          final fourWords = state.pathParameters['fourWords']!;
          return ActiveCallScreen(fourWords: fourWords);
        },
      ),
      GoRoute(
        path: Routes.network,
        name: 'network',
        builder: (context, state) => const NetworkPanelScreen(),
      ),
    ],
    errorBuilder: (context, state) => Scaffold(
      body: Center(
        child: Text('Page not found: ${state.uri.path}'),
      ),
    ),
  );
});
