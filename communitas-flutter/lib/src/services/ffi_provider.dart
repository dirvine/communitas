import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../main.dart' show kDemoMode;
import '../bindings/api_exports.dart';
import '../features/auth/providers/auth_provider.dart';

// ============================================================
// FFI Provider - Direct Rust bindings via flutter_rust_bridge
// ============================================================
//
// This file provides Riverpod providers that use CommunitasApi (FFI)
// directly instead of HTTP. This is the preferred approach for
// native apps (macOS, iOS, Android, Windows, Linux).
//
// For demo mode, fallback data is provided.
// Web builds are not supported (use native apps only).

/// Provider for the CommunitasApi instance from auth state.
///
/// Returns null if not authenticated.
final communitasApiProvider = Provider<CommunitasApi?>((ref) {
  final auth = ref.watch(authNotifierProvider);
  return auth.api;
});

/// Whether FFI is available (authenticated with native API, non-demo builds).
final ffiAvailableProvider = Provider<bool>((ref) {
  final api = ref.watch(communitasApiProvider);
  return api != null && !kDemoMode;
});

// ============================================================
// Entity Providers (FFI)
// ============================================================

/// Provider for all entities via FFI.
final ffiEntitiesProvider = FutureProvider<List<FlutterEntity>>((ref) async {
  final api = ref.watch(communitasApiProvider);
  if (api == null) return [];

  try {
    return await api.entityList();
  } catch (e) {
    debugPrint('FFI entityList error: $e');
    return [];
  }
});

/// Provider for entities by type via FFI.
final ffiEntitiesByTypeProvider = FutureProvider.family<List<FlutterEntity>, FlutterEntityType>(
  (ref, entityType) async {
    final api = ref.watch(communitasApiProvider);
    if (api == null) return [];

    try {
      return await api.entityListByType(entityType: entityType);
    } catch (e) {
      debugPrint('FFI entityListByType error: $e');
      return [];
    }
  },
);

/// Provider for a single entity by ID via FFI.
final ffiEntityProvider = FutureProvider.family<FlutterEntity?, String>(
  (ref, entityId) async {
    final api = ref.watch(communitasApiProvider);
    if (api == null) return null;

    try {
      return await api.entityGet(entityId: entityId);
    } catch (e) {
      debugPrint('FFI entityGet error: $e');
      return null;
    }
  },
);

/// Provider for organizations via FFI.
final ffiOrganizationsProvider = FutureProvider<List<FlutterEntity>>((ref) async {
  return ref.watch(ffiEntitiesByTypeProvider(FlutterEntityType.organisation).future);
});

/// Provider for projects via FFI.
final ffiProjectsProvider = FutureProvider<List<FlutterEntity>>((ref) async {
  return ref.watch(ffiEntitiesByTypeProvider(FlutterEntityType.project).future);
});

/// Provider for channels via FFI.
final ffiChannelsProvider = FutureProvider<List<FlutterEntity>>((ref) async {
  return ref.watch(ffiEntitiesByTypeProvider(FlutterEntityType.channel).future);
});

/// Provider for groups via FFI.
final ffiGroupsProvider = FutureProvider<List<FlutterEntity>>((ref) async {
  return ref.watch(ffiEntitiesByTypeProvider(FlutterEntityType.group).future);
});

// ============================================================
// Network Providers (FFI)
// ============================================================

/// Provider for network info via FFI.
final ffiNetworkInfoProvider = FutureProvider<FlutterNetworkInfo?>((ref) async {
  final api = ref.watch(communitasApiProvider);
  if (api == null) return null;

  try {
    return await api.gossipGetNetworkInfo();
  } catch (e) {
    debugPrint('FFI gossipGetNetworkInfo error: $e');
    return null;
  }
});

/// Provider for network status (active or not).
final ffiNetworkStatusProvider = FutureProvider<bool>((ref) async {
  final networkInfo = await ref.watch(ffiNetworkInfoProvider.future);
  return networkInfo?.isActive ?? false;
});

/// Provider for peer count via FFI.
final ffiPeerCountProvider = FutureProvider<int>((ref) async {
  final networkInfo = await ref.watch(ffiNetworkInfoProvider.future);
  return networkInfo?.peerCount ?? 0;
});

// ============================================================
// Profile Provider (FFI)
// ============================================================

/// Provider for current user profile via FFI.
final ffiUserProfileProvider = FutureProvider<FlutterUserProfile?>((ref) async {
  final api = ref.watch(communitasApiProvider);
  if (api == null) return null;

  try {
    return await api.getProfile();
  } catch (e) {
    debugPrint('FFI getProfile error: $e');
    return null;
  }
});

// ============================================================
// Action Controllers (FFI)
// ============================================================

/// Controller for entity operations via FFI.
class FfiEntityController extends StateNotifier<AsyncValue<List<FlutterEvent>>> {
  final Ref _ref;

  FfiEntityController(this._ref) : super(const AsyncValue.data([]));

  /// Create a new entity via FFI.
  Future<List<FlutterEvent>> createEntity({
    required String name,
    required FlutterEntityType entityType,
    String? description,
    String? parentOrgId,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return [];
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.entityCreate(
        name: name,
        entityType: entityType,
        description: description,
        parentOrgId: parentOrgId,
      );
      state = AsyncValue.data(events);

      // Invalidate entity providers to refresh
      _ref.invalidate(ffiEntitiesProvider);
      _ref.invalidate(ffiEntitiesByTypeProvider(entityType));

      return events;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return [];
    }
  }

  /// Add a member to an entity via FFI.
  Future<List<FlutterEvent>> addMember({
    required FlutterEntityType entityType,
    required String entityId,
    required String memberId,
    required String role,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return [];
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.entityAddMember(
        entityType: entityType,
        entityId: entityId,
        memberId: memberId,
        role: role,
      );
      state = AsyncValue.data(events);
      return events;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return [];
    }
  }

  /// Remove a member from an entity via FFI.
  Future<List<FlutterEvent>> removeMember({
    required FlutterEntityType entityType,
    required String entityId,
    required String memberId,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return [];
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.entityRemoveMember(
        entityType: entityType,
        entityId: entityId,
        memberId: memberId,
      );
      state = AsyncValue.data(events);
      return events;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return [];
    }
  }
}

final ffiEntityControllerProvider =
    StateNotifierProvider<FfiEntityController, AsyncValue<List<FlutterEvent>>>((ref) {
  return FfiEntityController(ref);
});

/// Controller for network operations via FFI.
class FfiNetworkController extends StateNotifier<AsyncValue<List<FlutterEvent>>> {
  final Ref _ref;

  FfiNetworkController(this._ref) : super(const AsyncValue.data([]));

  /// Start the gossip network via FFI.
  Future<List<FlutterEvent>> startNetwork({int? port}) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return [];
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.gossipStart(port: port);
      state = AsyncValue.data(events);

      // Invalidate network info to refresh
      _ref.invalidate(ffiNetworkInfoProvider);

      return events;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return [];
    }
  }

  /// Stop the gossip network via FFI.
  Future<List<FlutterEvent>> stopNetwork() async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return [];
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.gossipStop();
      state = AsyncValue.data(events);

      // Invalidate network info to refresh
      _ref.invalidate(ffiNetworkInfoProvider);

      return events;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return [];
    }
  }

  /// Connect to a peer by four words via FFI.
  Future<List<FlutterEvent>> connectToPeer(String fourWords) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return [];
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.gossipConnectToPeer(fourWords: fourWords);
      state = AsyncValue.data(events);

      // Invalidate network info to refresh
      _ref.invalidate(ffiNetworkInfoProvider);

      return events;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return [];
    }
  }
}

final ffiNetworkControllerProvider =
    StateNotifierProvider<FfiNetworkController, AsyncValue<List<FlutterEvent>>>((ref) {
  return FfiNetworkController(ref);
});

/// Controller for message operations via FFI.
class FfiMessageController extends StateNotifier<AsyncValue<List<FlutterEvent>>> {
  final Ref _ref;

  FfiMessageController(this._ref) : super(const AsyncValue.data([]));

  /// Send a message to an entity via FFI.
  Future<List<FlutterEvent>> sendMessage({
    required String entityId,
    required FlutterEntityType entityType,
    required String text,
    String? replyToId,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return [];
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.messageSend(
        entityId: entityId,
        entityType: entityType,
        text: text,
        replyToId: replyToId,
      );
      state = AsyncValue.data(events);
      return events;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return [];
    }
  }

  /// Send a direct message to specific recipients via FFI.
  Future<List<FlutterEvent>> sendDirectMessage({
    required List<String> recipients,
    required String text,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return [];
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.messageSendDirect(
        recipients: recipients,
        text: text,
      );
      state = AsyncValue.data(events);
      return events;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return [];
    }
  }

  /// Edit a message via FFI.
  Future<List<FlutterEvent>> editMessage({
    required String entityId,
    required FlutterEntityType entityType,
    required String messageId,
    required String newText,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return [];
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.messageEdit(
        entityId: entityId,
        entityType: entityType,
        messageId: messageId,
        newText: newText,
      );
      state = AsyncValue.data(events);
      return events;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return [];
    }
  }

  /// Delete a message via FFI.
  Future<List<FlutterEvent>> deleteMessage({
    required String entityId,
    required FlutterEntityType entityType,
    required String messageId,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return [];
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.messageDelete(
        entityId: entityId,
        entityType: entityType,
        messageId: messageId,
      );
      state = AsyncValue.data(events);
      return events;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return [];
    }
  }

  /// Add a reaction via FFI.
  Future<List<FlutterEvent>> addReaction({
    required String entityId,
    required FlutterEntityType entityType,
    required String messageId,
    required String emoji,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return [];
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.messageAddReaction(
        entityId: entityId,
        entityType: entityType,
        messageId: messageId,
        emoji: emoji,
      );
      state = AsyncValue.data(events);
      return events;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return [];
    }
  }

  /// Remove a reaction via FFI.
  Future<List<FlutterEvent>> removeReaction({
    required String entityId,
    required FlutterEntityType entityType,
    required String messageId,
    required String emoji,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return [];
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.messageRemoveReaction(
        entityId: entityId,
        entityType: entityType,
        messageId: messageId,
        emoji: emoji,
      );
      state = AsyncValue.data(events);
      return events;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return [];
    }
  }
}

final ffiMessageControllerProvider =
    StateNotifierProvider<FfiMessageController, AsyncValue<List<FlutterEvent>>>((ref) {
  return FfiMessageController(ref);
});

/// Controller for invite operations via FFI.
class FfiInviteController extends StateNotifier<AsyncValue<List<FlutterEvent>>> {
  final Ref _ref;

  FfiInviteController(this._ref) : super(const AsyncValue.data([]));

  /// Create an invite via FFI.
  Future<List<FlutterEvent>> createInvite({
    required String recipientId,
    required FlutterEntityType entityType,
    required String entityId,
    required String role,
    String? message,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return [];
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.inviteCreate(
        recipientId: recipientId,
        entityType: entityType,
        entityId: entityId,
        role: role,
        message: message,
      );
      state = AsyncValue.data(events);
      return events;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return [];
    }
  }

  /// Accept an invite via FFI.
  Future<List<FlutterEvent>> acceptInvite(String inviteId) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return [];
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.inviteAccept(inviteId: inviteId);
      state = AsyncValue.data(events);

      // Invalidate entity providers to refresh
      _ref.invalidate(ffiEntitiesProvider);

      return events;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return [];
    }
  }

  /// Reject an invite via FFI.
  Future<List<FlutterEvent>> rejectInvite(String inviteId) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return [];
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.inviteReject(inviteId: inviteId);
      state = AsyncValue.data(events);
      return events;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return [];
    }
  }

  /// Revoke an invite via FFI.
  Future<List<FlutterEvent>> revokeInvite(String inviteId) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return [];
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.inviteRevoke(inviteId: inviteId);
      state = AsyncValue.data(events);
      return events;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return [];
    }
  }
}

final ffiInviteControllerProvider =
    StateNotifierProvider<FfiInviteController, AsyncValue<List<FlutterEvent>>>((ref) {
  return FfiInviteController(ref);
});

// ============================================================
// Disk Providers (FFI)
// ============================================================

final ffiDiskFilesProvider = FutureProvider.family<
    List<FlutterFileInfo>,
    ({
      String entityId,
      FlutterDiskType diskType,
      String path,
    })>((ref, params) async {
  final api = ref.watch(communitasApiProvider);
  if (api == null) return [];

  try {
    return await api.diskListFiles(
      entityId: params.entityId,
      diskType: params.diskType,
      path: params.path,
    );
  } catch (e) {
    debugPrint('FFI diskListFiles error: $e');
    return [];
  }
});

final ffiDiskStatsProvider = FutureProvider.family<
    FlutterDiskStats?,
    ({
      String entityId,
      FlutterDiskType diskType,
    })>((ref, params) async {
  final api = ref.watch(communitasApiProvider);
  if (api == null) return null;

  try {
    return await api.diskGetStats(
      entityId: params.entityId,
      diskType: params.diskType,
    );
  } catch (e) {
    debugPrint('FFI diskGetStats error: $e');
    return null;
  }
});

class FfiDiskController extends StateNotifier<AsyncValue<List<FlutterEvent>>> {
  final Ref _ref;

  FfiDiskController(this._ref) : super(const AsyncValue.data([]));

  Future<bool> createDirectory({
    required String entityId,
    required FlutterDiskType diskType,
    required String path,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return false;
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.diskCreateDirectory(
        entityId: entityId,
        diskType: diskType,
        path: path,
      );
      state = AsyncValue.data(events);
      return true;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return false;
    }
  }

  Future<bool> writeFile({
    required String entityId,
    required FlutterDiskType diskType,
    required String path,
    required List<int> data,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return false;
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.diskWriteFile(
        entityId: entityId,
        diskType: diskType,
        path: path,
        data: data,
      );
      state = AsyncValue.data(events);
      return true;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return false;
    }
  }

  Future<bool> deleteFile({
    required String entityId,
    required FlutterDiskType diskType,
    required String path,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return false;
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.diskDeleteFile(
        entityId: entityId,
        diskType: diskType,
        path: path,
      );
      state = AsyncValue.data(events);
      return true;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return false;
    }
  }
}

final ffiDiskControllerProvider =
    StateNotifierProvider<FfiDiskController, AsyncValue<List<FlutterEvent>>>((ref) {
  return FfiDiskController(ref);
});

// ============================================================
// Kanban Providers (FFI)
// ============================================================

final ffiKanbanBoardsProvider =
    FutureProvider.family<List<FlutterKanbanBoard>, String>((ref, entityId) async {
  final api = ref.watch(communitasApiProvider);
  if (api == null) return [];

  try {
    return await api.kanbanListBoards(entityId: entityId);
  } catch (e) {
    debugPrint('FFI kanbanListBoards error: $e');
    return [];
  }
});

final ffiKanbanColumnsProvider =
    FutureProvider.family<List<FlutterKanbanColumn>, String>((ref, boardId) async {
  final api = ref.watch(communitasApiProvider);
  if (api == null) return [];

  try {
    return await api.kanbanListColumns(boardId: boardId);
  } catch (e) {
    debugPrint('FFI kanbanListColumns error: $e');
    return [];
  }
});

final ffiKanbanCardsProvider = FutureProvider.family<
    List<FlutterKanbanCard>,
    ({
      String boardId,
      String? columnId,
      String? state,
      String? assigneeId,
      String? tagId,
    })>((ref, params) async {
  final api = ref.watch(communitasApiProvider);
  if (api == null) return [];

  try {
    return await api.kanbanListCards(
      boardId: params.boardId,
      columnId: params.columnId,
      state: params.state,
      assigneeId: params.assigneeId,
      tagId: params.tagId,
    );
  } catch (e) {
    debugPrint('FFI kanbanListCards error: $e');
    return [];
  }
});

class FfiKanbanController extends StateNotifier<AsyncValue<void>> {
  final Ref _ref;

  FfiKanbanController(this._ref) : super(const AsyncValue.data(null));

  Future<FlutterKanbanBoard?> createBoard({
    required String entityId,
    required String boardName,
    String? description,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return null;
    }

    state = const AsyncValue.loading();
    try {
      final board = await api.kanbanCreateBoard(
        entityId: entityId,
        boardName: boardName,
        description: description,
      );
      _ref.invalidate(ffiKanbanBoardsProvider(entityId));
      state = const AsyncValue.data(null);
      return board;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return null;
    }
  }

  Future<FlutterKanbanColumn?> createColumn({
    required String boardId,
    required String columnName,
    int? position,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return null;
    }

    state = const AsyncValue.loading();
    try {
      final column = await api.kanbanCreateColumn(
        boardId: boardId,
        columnName: columnName,
        position: position,
      );
      _ref.invalidate(ffiKanbanColumnsProvider(boardId));
      state = const AsyncValue.data(null);
      return column;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return null;
    }
  }

  Future<FlutterKanbanCard?> createCard({
    required String boardId,
    required String columnId,
    required String title,
    String? description,
    String? assignee,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return null;
    }

    state = const AsyncValue.loading();
    try {
      final card = await api.kanbanCreateCard(
        boardId: boardId,
        columnId: columnId,
        title: title,
        description: description,
        assignee: assignee,
      );
      _ref.invalidate(
        ffiKanbanCardsProvider((
          boardId: boardId,
          columnId: null,
          state: null,
          assigneeId: null,
          tagId: null,
        )),
      );
      state = const AsyncValue.data(null);
      return card;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return null;
    }
  }

  Future<bool> moveCard({
    required String boardId,
    required String cardId,
    required String targetColumnId,
    int? position,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return false;
    }

    state = const AsyncValue.loading();
    try {
      await api.kanbanMoveCard(
        boardId: boardId,
        cardId: cardId,
        targetColumnId: targetColumnId,
        position: position,
      );
      _ref.invalidate(
        ffiKanbanCardsProvider((
          boardId: boardId,
          columnId: null,
          state: null,
          assigneeId: null,
          tagId: null,
        )),
      );
      state = const AsyncValue.data(null);
      return true;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return false;
    }
  }

  Future<bool> updateCard({
    required String boardId,
    required String cardId,
    String? title,
    String? description,
    String? assignee,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return false;
    }

    state = const AsyncValue.loading();
    try {
      await api.kanbanUpdateCard(
        boardId: boardId,
        cardId: cardId,
        title: title,
        description: description,
        assignee: assignee,
      );
      _ref.invalidate(
        ffiKanbanCardsProvider((
          boardId: boardId,
          columnId: null,
          state: null,
          assigneeId: null,
          tagId: null,
        )),
      );
      state = const AsyncValue.data(null);
      return true;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return false;
    }
  }

  Future<bool> deleteCard({
    required String boardId,
    required String cardId,
  }) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return false;
    }

    state = const AsyncValue.loading();
    try {
      await api.kanbanDeleteCard(boardId: boardId, cardId: cardId);
      _ref.invalidate(
        ffiKanbanCardsProvider((
          boardId: boardId,
          columnId: null,
          state: null,
          assigneeId: null,
          tagId: null,
        )),
      );
      state = const AsyncValue.data(null);
      return true;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return false;
    }
  }
}

final ffiKanbanControllerProvider =
    StateNotifierProvider<FfiKanbanController, AsyncValue<void>>((ref) {
  return FfiKanbanController(ref);
});

/// Controller for profile operations via FFI.
class FfiProfileController extends StateNotifier<AsyncValue<List<FlutterEvent>>> {
  final Ref _ref;

  FfiProfileController(this._ref) : super(const AsyncValue.data([]));

  /// Update display name via FFI.
  Future<List<FlutterEvent>> updateDisplayName(String displayName) async {
    final api = _ref.read(communitasApiProvider);
    if (api == null) {
      state = AsyncValue.error('Not authenticated', StackTrace.current);
      return [];
    }

    state = const AsyncValue.loading();
    try {
      final events = await api.updateDisplayName(displayName: displayName);
      state = AsyncValue.data(events);

      // Invalidate profile to refresh
      _ref.invalidate(ffiUserProfileProvider);

      return events;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return [];
    }
  }
}

final ffiProfileControllerProvider =
    StateNotifierProvider<FfiProfileController, AsyncValue<List<FlutterEvent>>>((ref) {
  return FfiProfileController(ref);
});
