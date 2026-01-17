import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../bindings/api_exports.dart';
import '../features/auth/providers/auth_provider.dart';
import 'ffi_provider.dart';

enum OrganizationCategory {
  organization,
  community,
}

class OrganizationCategoryOverrides
    extends StateNotifier<Map<String, OrganizationCategory>> {
  static const _prefsKey = 'organization_category_overrides';

  OrganizationCategoryOverrides() : super(const {}) {
    _load();
  }

  Future<void> _load() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_prefsKey);
    if (raw == null || raw.isEmpty) return;
    try {
      final decoded = jsonDecode(raw) as Map<String, dynamic>;
      final parsed = <String, OrganizationCategory>{};
      for (final entry in decoded.entries) {
        final value = entry.value?.toString();
        final category = OrganizationCategory.values
            .firstWhere((c) => c.name == value, orElse: () => OrganizationCategory.organization);
        parsed[entry.key] = category;
      }
      state = parsed;
    } catch (e) {
      debugPrint('Failed to load org category overrides: $e');
    }
  }

  Future<void> _persist() async {
    final prefs = await SharedPreferences.getInstance();
    final payload = {
      for (final entry in state.entries) entry.key: entry.value.name,
    };
    await prefs.setString(_prefsKey, jsonEncode(payload));
  }

  Future<void> setCategory(String entityId, OrganizationCategory category) async {
    state = {...state, entityId: category};
    await _persist();
  }
}

final organizationCategoryOverridesProvider =
    StateNotifierProvider<OrganizationCategoryOverrides, Map<String, OrganizationCategory>>((ref) {
  return OrganizationCategoryOverrides();
});

OrganizationCategory resolveOrganizationCategory(
  UnifiedEntity entity,
  Map<String, OrganizationCategory> overrides,
) {
  final override = overrides[entity.id];
  if (override != null) return override;
  final text = '${entity.name} ${entity.description}'.toLowerCase();
  if (text.contains('community') ||
      text.contains('collective') ||
      text.contains('nonprofit') ||
      text.contains('non-profit') ||
      text.contains('foundation')) {
    return OrganizationCategory.community;
  }
  return OrganizationCategory.organization;
}

/// Unified entity model for FFI data.
class UnifiedEntity {
  final String id;
  final String type;
  final String name;
  final String role;
  final String description;
  final int memberCount;
  final String? parentId;

  const UnifiedEntity({
    required this.id,
    required this.type,
    required this.name,
    required this.role,
    required this.description,
    required this.memberCount,
    this.parentId,
  });

  /// Create from FFI FlutterEntity type.
  factory UnifiedEntity.fromFfi(
    FlutterEntity entity, {
    String role = 'member',
  }) {
    return UnifiedEntity(
      id: entity.id,
      type: entity.entityType.name,
      name: entity.name,
      role: role,
      description: entity.description ?? '',
      memberCount: entity.memberCount.toInt(),
      parentId: entity.parentOrgId,
    );
  }
}

/// Unified contact model for FFI data.
class UnifiedContact {
  /// Hex-encoded ML-DSA-87 public key (THE identity, Level 5 PQC).
  final String pubkeyHex;

  /// User-chosen display name (shown in UI).
  final String displayName;
  final String status;

  const UnifiedContact({
    required this.pubkeyHex,
    required this.displayName,
    required this.status,
  });

  /// Create from FFI FlutterContact type.
  factory UnifiedContact.fromFfi(FlutterContact contact) {
    return UnifiedContact(
      pubkeyHex: contact.id,
      displayName: contact.displayName.isNotEmpty
          ? contact.displayName
          : (contact.fourWords ?? contact.id),
      status: contact.isOnline ? 'online' : 'offline',
    );
  }
}

/// Unified message model for FFI data.
class UnifiedMessage {
  final String id;
  final String senderId;
  final String senderName;
  final String content;
  final String timestamp;
  final Map<String, int> reactions;
  final Set<String> userReactions;
  final DateTime? editedAt;
  final String? replyToId;
  final bool hasThread;
  final int threadReplyCount;

  const UnifiedMessage({
    required this.id,
    required this.senderId,
    required this.senderName,
    required this.content,
    required this.timestamp,
    required this.reactions,
    this.userReactions = const {},
    this.editedAt,
    this.replyToId,
    this.hasThread = false,
    this.threadReplyCount = 0,
  });

  UnifiedMessage copyWith({
    String? id,
    String? senderId,
    String? senderName,
    String? content,
    String? timestamp,
    Map<String, int>? reactions,
    Set<String>? userReactions,
    DateTime? editedAt,
    String? replyToId,
    bool? hasThread,
    int? threadReplyCount,
  }) {
    return UnifiedMessage(
      id: id ?? this.id,
      senderId: senderId ?? this.senderId,
      senderName: senderName ?? this.senderName,
      content: content ?? this.content,
      timestamp: timestamp ?? this.timestamp,
      reactions: reactions ?? this.reactions,
      userReactions: userReactions ?? this.userReactions,
      editedAt: editedAt ?? this.editedAt,
      replyToId: replyToId ?? this.replyToId,
      hasThread: hasThread ?? this.hasThread,
      threadReplyCount: threadReplyCount ?? this.threadReplyCount,
    );
  }

  /// Create from FFI FlutterMessage type.
  factory UnifiedMessage.fromFfi(FlutterMessage message) {
    final reactionMap = <String, int>{};
    final userReactions = <String>{};
    for (final reaction in message.reactions) {
      reactionMap[reaction.emoji] = reaction.count;
      if (reaction.userReacted) {
        userReactions.add(reaction.emoji);
      }
    }

    final editedAt = message.editedAt != null
        ? DateTime.fromMillisecondsSinceEpoch(message.editedAt!.toInt())
        : null;

    return UnifiedMessage(
      id: message.id,
      senderId: message.author,
      senderName: message.author,
      content: message.text,
      timestamp: DateTime.fromMillisecondsSinceEpoch(
        message.timestamp.toInt(),
      ).toIso8601String(),
      reactions: reactionMap,
      userReactions: userReactions,
      editedAt: editedAt,
      replyToId: message.replyToId,
      hasThread: false,
      threadReplyCount: 0,
    );
  }
}

// ============================================================
// Unified Data Providers
// ============================================================

/// Provider for user identity info.
///
/// Returns pubkeyHex (permanent identity) and displayName (shown in UI).
final unifiedIdentityProvider = Provider<({String pubkeyHex, String displayName, String? fourWords})>((ref) {
  final auth = ref.watch(authNotifierProvider);

  if (!auth.isAuthenticated) {
    // Not authenticated - return empty identity
    return (
      pubkeyHex: '',
      displayName: 'Not Logged In',
      fourWords: null,
    );
  }

  // Use pubkeyHex if available, fallback to fourWords during migration
  final pubkeyHex = auth.pubkeyHex ?? auth.fourWords ?? 'unknown-identity';
  return (
    pubkeyHex: pubkeyHex,
    displayName: auth.displayName ?? 'Unknown User',
    fourWords: auth.fourWords,
  );
});

/// Provider for all organizations.
final unifiedOrganizationsProvider = FutureProvider<List<UnifiedEntity>>((ref) async {
  final identity = ref.watch(unifiedIdentityProvider);

  try {
    final orgs = await ref.watch(ffiOrganizationsProvider.future);
    return orgs
        .map((e) => UnifiedEntity.fromFfi(e, role: _inferRole(e, identity)))
        .toList();
  } catch (e) {
    debugPrint('Error fetching organizations via FFI: $e');
    return [];
  }
});

/// Provider for all projects.
final unifiedProjectsProvider = FutureProvider<List<UnifiedEntity>>((ref) async {
  final identity = ref.watch(unifiedIdentityProvider);

  try {
    final projects = await ref.watch(ffiProjectsProvider.future);
    return projects
        .map((e) => UnifiedEntity.fromFfi(e, role: _inferRole(e, identity)))
        .toList();
  } catch (e) {
    debugPrint('Error fetching projects via FFI: $e');
    return [];
  }
});

/// Provider for all channels.
final unifiedChannelsProvider = FutureProvider<List<UnifiedEntity>>((ref) async {
  final identity = ref.watch(unifiedIdentityProvider);

  try {
    final channels = await ref.watch(ffiChannelsProvider.future);
    return channels
        .map((e) => UnifiedEntity.fromFfi(e, role: _inferRole(e, identity)))
        .toList();
  } catch (e) {
    debugPrint('Error fetching channels via FFI: $e');
    return [];
  }
});

/// Provider for all groups.
final unifiedGroupsProvider = FutureProvider<List<UnifiedEntity>>((ref) async {
  final identity = ref.watch(unifiedIdentityProvider);

  try {
    final groups = await ref.watch(ffiGroupsProvider.future);
    return groups
        .map((e) => UnifiedEntity.fromFfi(e, role: _inferRole(e, identity)))
        .toList();
  } catch (e) {
    debugPrint('Error fetching groups via FFI: $e');
    return [];
  }
});

/// Provider for all contacts.
final unifiedContactsProvider = FutureProvider<List<UnifiedContact>>((ref) async {
  final api = ref.watch(communitasApiProvider);
  if (api == null) {
    return [];
  }

  try {
    final contacts = await api.contactsList();
    return contacts.map(UnifiedContact.fromFfi).toList();
  } catch (e) {
    debugPrint('Error fetching contacts via FFI: $e');
    return [];
  }
});

/// Provider for messages in a channel.
final unifiedMessagesProvider = FutureProvider.family<List<UnifiedMessage>, String>((ref, channelId) async {
  final api = ref.watch(communitasApiProvider);
  if (api == null) {
    return [];
  }

  try {
    final messages = await api.messageList(entityId: channelId);
    return _applyThreadCounts(messages.map(UnifiedMessage.fromFfi).toList());
  } catch (e) {
    debugPrint('Error fetching messages via FFI: $e');
    return [];
  }
});

/// Provider for direct messages with a peer.
final unifiedDirectMessagesProvider =
    FutureProvider.family<List<UnifiedMessage>, String>((ref, peerId) async {
  final api = ref.watch(communitasApiProvider);
  if (api == null) {
    return [];
  }

  try {
    final messages = await api.messageListDirect(otherPeerId: peerId);
    return messages.map(UnifiedMessage.fromFfi).toList();
  } catch (e) {
    debugPrint('Error fetching direct messages via FFI: $e');
    return [];
  }
});

List<UnifiedMessage> _applyThreadCounts(List<UnifiedMessage> messages) {
  if (messages.isEmpty) return messages;

  final replyCounts = <String, int>{};
  for (final message in messages) {
    final parentId = message.replyToId;
    if (parentId != null && parentId.isNotEmpty) {
      replyCounts[parentId] = (replyCounts[parentId] ?? 0) + 1;
    }
  }

  return messages
      .map((message) {
        final count = replyCounts[message.id] ?? 0;
        return message.copyWith(
          hasThread: count > 0,
          threadReplyCount: count,
        );
      })
      .toList();
}

/// Provider for entities by type.
final unifiedEntitiesByTypeProvider = FutureProvider.family<List<UnifiedEntity>, String>((ref, type) async {
  switch (type) {
    case 'organization':
    case 'organisation': // British spelling
      return ref.watch(unifiedOrganizationsProvider.future);
    case 'project':
      return ref.watch(unifiedProjectsProvider.future);
    case 'channel':
      return ref.watch(unifiedChannelsProvider.future);
    case 'group':
      return ref.watch(unifiedGroupsProvider.future);
    default:
      return [];
  }
});

/// Provider for all entities across types.
final unifiedAllEntitiesProvider = FutureProvider<List<UnifiedEntity>>((ref) async {
  final orgs = await ref.watch(unifiedOrganizationsProvider.future);
  final groups = await ref.watch(unifiedGroupsProvider.future);
  final projects = await ref.watch(unifiedProjectsProvider.future);
  final channels = await ref.watch(unifiedChannelsProvider.future);

  return [
    ...orgs,
    ...groups,
    ...projects,
    ...channels,
  ];
});

/// Provider for child entities under a parent.
final unifiedChildEntitiesProvider = FutureProvider.family<List<UnifiedEntity>, String>((ref, parentId) async {
  final projects = await ref.watch(unifiedProjectsProvider.future);
  final channels = await ref.watch(unifiedChannelsProvider.future);
  final groups = await ref.watch(unifiedGroupsProvider.future);

  return [
    ...projects.where((p) => p.parentId == parentId),
    ...channels.where((c) => c.parentId == parentId),
    ...groups.where((g) => g.parentId == parentId),
  ];
});

/// Provider for a single entity by type and ID.
final unifiedEntityByIdProvider = FutureProvider.family<UnifiedEntity?, ({String type, String id})>((ref, params) async {
  final entities = await ref.watch(unifiedEntitiesByTypeProvider(params.type).future);
  for (final entity in entities) {
    if (entity.id == params.id) {
      return entity;
    }
  }
  return null;
});

String _inferRole(
  FlutterEntity entity,
  ({String pubkeyHex, String displayName, String? fourWords}) identity,
) {
  if (entity.createdBy == identity.pubkeyHex ||
      (identity.fourWords != null && entity.createdBy == identity.fourWords)) {
    return 'owner';
  }
  return 'member';
}
