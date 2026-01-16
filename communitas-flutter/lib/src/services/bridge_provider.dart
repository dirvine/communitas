// =============================================================================
// DEPRECATED: This file is deprecated in favor of ffi_provider.dart
// =============================================================================
//
// The HTTP bridge approach has been replaced with direct FFI bindings via
// flutter_rust_bridge. For native apps (macOS, iOS, Android, Windows, Linux),
// use ffi_provider.dart instead.
//
// This file is kept temporarily for backwards compatibility and will be
// removed in a future version.
//
// Migration guide:
// - bridgeClientProvider -> communitasApiProvider (from ffi_provider.dart)
// - bridgeStatusProvider -> ffiNetworkStatusProvider
// - channelsProvider -> ffiChannelsProvider
// - organisationsProvider -> ffiOrganizationsProvider
// - projectsProvider -> ffiProjectsProvider
// - groupsProvider -> ffiGroupsProvider
// - peersProvider -> ffiNetworkInfoProvider (for peer count)
// - connectionInfoProvider -> ffiNetworkInfoProvider
//
// =============================================================================

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'bridge_client.dart';

// ============================================================
// Bridge Configuration (DEPRECATED - use ffi_provider.dart)
// ============================================================

/// Provider for the bridge server URL.
///
/// On web, automatically detects the bridge URL based on the current host.
/// Can be overridden via BRIDGE_URL environment variable.
final bridgeUrlProvider = StateProvider<String>((ref) {
  // Check for environment variable override
  const envUrl = String.fromEnvironment('BRIDGE_URL', defaultValue: '');
  if (envUrl.isNotEmpty) return envUrl;

  // On web, use the current host's bridge endpoint
  if (kIsWeb) {
    // Replace web server port with bridge port (3030)
    final uri = Uri.base;
    final host = uri.host;
    // Use same scheme but always port 3030 for bridge
    return '${uri.scheme}://$host:3030';
  }

  // Default for local development
  return 'http://localhost:3030';
});

/// Provider for the bridge client instance.
final bridgeClientProvider = Provider<BridgeClient>((ref) {
  final baseUrl = ref.watch(bridgeUrlProvider);
  return BridgeClient(baseUrl: baseUrl);
});

/// Provider to check if bridge is connected and ready.
final bridgeStatusProvider = FutureProvider<bool>((ref) async {
  final client = ref.watch(bridgeClientProvider);
  return client.checkStatus();
});

/// Provider for current session info from bridge.
final bridgeSessionProvider =
    FutureProvider<Map<String, dynamic>?>((ref) async {
  final client = ref.watch(bridgeClientProvider);
  return client.getSessionInfo();
});

// ============================================================
// Data Providers
// ============================================================

/// Provider for all channels.
final channelsProvider = FutureProvider<List<dynamic>>((ref) async {
  final client = ref.watch(bridgeClientProvider);
  return client.listChannels();
});

/// Provider for all organizations.
final organisationsProvider = FutureProvider<List<dynamic>>((ref) async {
  final client = ref.watch(bridgeClientProvider);
  return client.listOrganisations();
});

/// Provider for all projects.
final projectsProvider = FutureProvider<List<dynamic>>((ref) async {
  final client = ref.watch(bridgeClientProvider);
  return client.listProjects();
});

/// Provider for all groups.
final groupsProvider = FutureProvider<List<dynamic>>((ref) async {
  final client = ref.watch(bridgeClientProvider);
  return client.listGroups();
});

/// Provider for all entities (organizations, projects, channels, groups).
final entitiesProvider = FutureProvider<List<dynamic>>((ref) async {
  final client = ref.watch(bridgeClientProvider);
  return client.listEntities();
});

/// Provider for messages in a specific channel.
final messagesProvider =
    FutureProvider.family<List<dynamic>, String>((ref, channelId) async {
  final client = ref.watch(bridgeClientProvider);
  return client.getChannelMessages(channelId);
});

/// Provider for members of a specific entity.
final membersProvider = FutureProvider.family<List<dynamic>, ({String entityType, String entityId})>(
    (ref, params) async {
  final client = ref.watch(bridgeClientProvider);
  return client.getMembers(params.entityType, params.entityId);
});

/// Provider for files in an entity's virtual disk.
final filesProvider = FutureProvider.family<List<dynamic>,
    ({String entityId, String diskType, String path})>((ref, params) async {
  final client = ref.watch(bridgeClientProvider);
  return client.listFiles(params.entityId, params.diskType, params.path);
});

/// Provider for disk statistics.
final diskStatsProvider = FutureProvider.family<Map<String, dynamic>?,
    ({String entityId, String diskType})>((ref, params) async {
  final client = ref.watch(bridgeClientProvider);
  return client.getDiskStats(params.entityId, params.diskType);
});

/// Provider for Kanban boards in a project.
final boardsProvider =
    FutureProvider.family<List<dynamic>, String>((ref, projectId) async {
  final client = ref.watch(bridgeClientProvider);
  return client.listBoards(projectId);
});

/// Provider for connected P2P peers.
final peersProvider = FutureProvider<List<dynamic>>((ref) async {
  final client = ref.watch(bridgeClientProvider);
  return client.getPeers();
});

/// Provider for network connection info.
final connectionInfoProvider =
    FutureProvider<Map<String, dynamic>?>((ref) async {
  final client = ref.watch(bridgeClientProvider);
  return client.getConnectionInfo();
});

/// Provider for all contacts.
final contactsProvider = FutureProvider<List<dynamic>>((ref) async {
  final client = ref.watch(bridgeClientProvider);
  return client.listContacts();
});

/// Provider for favorite contacts.
final favoriteContactsProvider = FutureProvider<List<dynamic>>((ref) async {
  final client = ref.watch(bridgeClientProvider);
  return client.getFavoriteContacts();
});

// ============================================================
// Action Notifiers
// ============================================================

/// Notifier for sending messages.
class MessageSender extends StateNotifier<AsyncValue<void>> {
  final BridgeClient _client;

  MessageSender(this._client) : super(const AsyncValue.data(null));

  Future<Map<String, dynamic>?> sendMessage(
      String channelId, String content) async {
    state = const AsyncValue.loading();
    try {
      final result = await _client.sendMessage(channelId, content);
      state = const AsyncValue.data(null);
      return result;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return null;
    }
  }
}

final messageSenderProvider =
    StateNotifierProvider<MessageSender, AsyncValue<void>>((ref) {
  final client = ref.watch(bridgeClientProvider);
  return MessageSender(client);
});

/// Notifier for entity creation.
class EntityCreator extends StateNotifier<AsyncValue<void>> {
  final BridgeClient _client;

  EntityCreator(this._client) : super(const AsyncValue.data(null));

  Future<Map<String, dynamic>?> createOrganisation(
      String name, String description) async {
    state = const AsyncValue.loading();
    try {
      final result = await _client.createOrganisation(name, description);
      state = const AsyncValue.data(null);
      return result;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return null;
    }
  }

  Future<Map<String, dynamic>?> createProject(
      String name, String description, String? parentId) async {
    state = const AsyncValue.loading();
    try {
      final result = await _client.createProject(name, description, parentId);
      state = const AsyncValue.data(null);
      return result;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return null;
    }
  }

  Future<Map<String, dynamic>?> createChannel(
      String name, String description) async {
    state = const AsyncValue.loading();
    try {
      final result = await _client.createChannel(name, description);
      state = const AsyncValue.data(null);
      return result;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return null;
    }
  }

  Future<Map<String, dynamic>?> createGroup(
      String name, String description) async {
    state = const AsyncValue.loading();
    try {
      final result = await _client.createGroup(name, description);
      state = const AsyncValue.data(null);
      return result;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return null;
    }
  }
}

final entityCreatorProvider =
    StateNotifierProvider<EntityCreator, AsyncValue<void>>((ref) {
  final client = ref.watch(bridgeClientProvider);
  return EntityCreator(client);
});

/// Notifier for network operations.
class NetworkController extends StateNotifier<AsyncValue<void>> {
  final BridgeClient _client;

  NetworkController(this._client) : super(const AsyncValue.data(null));

  Future<bool> startNetworking() async {
    state = const AsyncValue.loading();
    try {
      final result = await _client.startNetworking();
      state = const AsyncValue.data(null);
      return result;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return false;
    }
  }

  Future<bool> connectToPeer(String fourWords) async {
    state = const AsyncValue.loading();
    try {
      final result = await _client.connectToPeer(fourWords);
      state = const AsyncValue.data(null);
      return result;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return false;
    }
  }

  Future<bool> disconnectFromPeer(String fourWords) async {
    state = const AsyncValue.loading();
    try {
      final result = await _client.disconnectFromPeer(fourWords);
      state = const AsyncValue.data(null);
      return result;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return false;
    }
  }
}

final networkControllerProvider =
    StateNotifierProvider<NetworkController, AsyncValue<void>>((ref) {
  final client = ref.watch(bridgeClientProvider);
  return NetworkController(client);
});

/// Notifier for file operations.
class FileController extends StateNotifier<AsyncValue<void>> {
  final BridgeClient _client;

  FileController(this._client) : super(const AsyncValue.data(null));

  Future<Map<String, dynamic>?> uploadFile(
      String entityId, String diskType, String path, String content) async {
    state = const AsyncValue.loading();
    try {
      final result =
          await _client.uploadFile(entityId, diskType, path, content);
      state = const AsyncValue.data(null);
      return result;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return null;
    }
  }

  Future<String?> downloadFile(
      String entityId, String diskType, String path) async {
    state = const AsyncValue.loading();
    try {
      final result = await _client.downloadFile(entityId, diskType, path);
      state = const AsyncValue.data(null);
      return result;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return null;
    }
  }

  Future<bool> deleteFile(
      String entityId, String diskType, String path) async {
    state = const AsyncValue.loading();
    try {
      final result = await _client.deleteFile(entityId, diskType, path);
      state = const AsyncValue.data(null);
      return result;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return false;
    }
  }
}

final fileControllerProvider =
    StateNotifierProvider<FileController, AsyncValue<void>>((ref) {
  final client = ref.watch(bridgeClientProvider);
  return FileController(client);
});
