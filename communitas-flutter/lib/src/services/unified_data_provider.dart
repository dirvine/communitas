import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../main.dart' show kDemoMode;
import '../bindings/api_exports.dart';
import '../demo/demo_data.dart';
import '../features/auth/providers/auth_provider.dart';
import 'ffi_provider.dart';

/// Unified entity model for both demo and bridge data.
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

  factory UnifiedEntity.fromDemo(DemoEntity demo) {
    return UnifiedEntity(
      id: demo.id,
      type: demo.type,
      name: demo.name,
      role: demo.role,
      description: demo.description,
      memberCount: demo.memberCount,
      parentId: demo.parentId,
    );
  }

  factory UnifiedEntity.fromBridge(Map<String, dynamic> json) {
    return UnifiedEntity(
      id: json['id'] as String? ?? '',
      type: json['entity_type'] as String? ?? json['type'] as String? ?? 'unknown',
      name: json['name'] as String? ?? 'Unnamed',
      role: json['role'] as String? ?? 'member',
      description: json['description'] as String? ?? '',
      memberCount: json['member_count'] as int? ?? 0,
      parentId: json['parent_id'] as String?,
    );
  }

  /// Create from FFI FlutterEntity type.
  factory UnifiedEntity.fromFfi(FlutterEntity entity) {
    return UnifiedEntity(
      id: entity.id,
      type: entity.entityType.name,
      name: entity.name,
      role: 'member', // Role is determined by membership, not entity
      description: entity.description ?? '',
      memberCount: entity.memberCount.toInt(),
      parentId: entity.parentOrgId,
    );
  }
}

/// Unified contact model for both demo and bridge data.
class UnifiedContact {
  /// Hex-encoded ML-DSA-65 public key (THE identity).
  final String pubkeyHex;

  /// User-chosen display name (shown in UI).
  final String displayName;
  final String status;

  const UnifiedContact({
    required this.pubkeyHex,
    required this.displayName,
    required this.status,
  });

  factory UnifiedContact.fromDemo(DemoContact demo) {
    return UnifiedContact(
      // Use fourWords as placeholder for pubkeyHex in demo mode
      pubkeyHex: demo.fourWords,
      displayName: demo.displayName,
      status: demo.status,
    );
  }

  factory UnifiedContact.fromBridge(Map<String, dynamic> json) {
    return UnifiedContact(
      pubkeyHex: json['pubkey_hex'] as String? ?? json['four_words'] as String? ?? '',
      displayName: json['display_name'] as String? ?? json['four_words'] as String? ?? '',
      status: json['status'] as String? ?? 'offline',
    );
  }
}

/// Unified message model for both demo and bridge data.
class UnifiedMessage {
  final String id;
  final String senderId;
  final String senderName;
  final String content;
  final String timestamp;
  final Map<String, int> reactions;
  final bool hasThread;
  final int threadReplyCount;

  const UnifiedMessage({
    required this.id,
    required this.senderId,
    required this.senderName,
    required this.content,
    required this.timestamp,
    required this.reactions,
    this.hasThread = false,
    this.threadReplyCount = 0,
  });

  factory UnifiedMessage.fromDemo(DemoMessage demo) {
    return UnifiedMessage(
      id: demo.id,
      senderId: demo.senderId,
      senderName: demo.senderName,
      content: demo.content,
      timestamp: demo.timestamp,
      reactions: demo.reactions,
      hasThread: demo.hasThread,
      threadReplyCount: demo.threadReplyCount,
    );
  }

  factory UnifiedMessage.fromBridge(Map<String, dynamic> json) {
    return UnifiedMessage(
      id: json['id'] as String? ?? '',
      senderId: json['sender_id'] as String? ?? json['four_words'] as String? ?? '',
      senderName: json['sender_name'] as String? ?? json['display_name'] as String? ?? 'Unknown',
      content: json['content'] as String? ?? '',
      timestamp: json['timestamp'] as String? ?? '',
      reactions: (json['reactions'] as Map<String, dynamic>?)?.map(
        (k, v) => MapEntry(k, v as int),
      ) ?? {},
      hasThread: json['has_thread'] as bool? ?? false,
      threadReplyCount: json['thread_reply_count'] as int? ?? 0,
    );
  }
}

// ============================================================
// Unified Data Providers
// ============================================================

/// Provider for user identity info.
///
/// Returns pubkeyHex (permanent identity) and displayName (shown in UI).
final unifiedIdentityProvider = Provider<({String pubkeyHex, String displayName})>((ref) {
  final auth = ref.watch(authNotifierProvider);

  if (auth.isAuthenticated) {
    // Use pubkeyHex if available, fallback to fourWords during migration
    final pubkeyHex = auth.pubkeyHex ?? auth.fourWords ?? 'unknown-identity';
    return (
      pubkeyHex: pubkeyHex,
      displayName: auth.displayName ?? 'Unknown User',
    );
  }

  // Fallback to demo identity (using fourWords as placeholder for pubkeyHex)
  return (
    pubkeyHex: DemoData.demoIdentity.fourWords,
    displayName: DemoData.demoIdentity.displayName,
  );
});

/// Provider for all organizations.
final unifiedOrganizationsProvider = FutureProvider<List<UnifiedEntity>>((ref) async {
  // Demo mode: use demo data
  if (kDemoMode) {
    return DemoData.organizations.map((e) => UnifiedEntity.fromDemo(e)).toList();
  }

  // FFI mode: use direct Rust bindings
  try {
    final orgs = await ref.watch(ffiOrganizationsProvider.future);
    return orgs.map((e) => UnifiedEntity.fromFfi(e)).toList();
  } catch (e) {
    debugPrint('Error fetching organizations via FFI: $e');
    return [];
  }
});

/// Provider for all projects.
final unifiedProjectsProvider = FutureProvider<List<UnifiedEntity>>((ref) async {
  // Demo mode: use demo data
  if (kDemoMode) {
    return DemoData.projects.map((e) => UnifiedEntity.fromDemo(e)).toList();
  }

  // FFI mode: use direct Rust bindings
  try {
    final projects = await ref.watch(ffiProjectsProvider.future);
    return projects.map((e) => UnifiedEntity.fromFfi(e)).toList();
  } catch (e) {
    debugPrint('Error fetching projects via FFI: $e');
    return [];
  }
});

/// Provider for all channels.
final unifiedChannelsProvider = FutureProvider<List<UnifiedEntity>>((ref) async {
  // Demo mode: use demo data
  if (kDemoMode) {
    return DemoData.channels.map((e) => UnifiedEntity.fromDemo(e)).toList();
  }

  // FFI mode: use direct Rust bindings
  try {
    final channels = await ref.watch(ffiChannelsProvider.future);
    return channels.map((e) => UnifiedEntity.fromFfi(e)).toList();
  } catch (e) {
    debugPrint('Error fetching channels via FFI: $e');
    return [];
  }
});

/// Provider for all groups.
final unifiedGroupsProvider = FutureProvider<List<UnifiedEntity>>((ref) async {
  // Demo mode: use demo data
  if (kDemoMode) {
    return DemoData.groups.map((e) => UnifiedEntity.fromDemo(e)).toList();
  }

  // FFI mode: use direct Rust bindings
  try {
    final groups = await ref.watch(ffiGroupsProvider.future);
    return groups.map((e) => UnifiedEntity.fromFfi(e)).toList();
  } catch (e) {
    debugPrint('Error fetching groups via FFI: $e');
    return [];
  }
});

/// Provider for all contacts.
///
/// TODO: FFI method for listing contacts not yet available.
/// Currently uses demo data for all modes.
final unifiedContactsProvider = FutureProvider<List<UnifiedContact>>((ref) async {
  // Demo mode or FFI fallback: use demo data
  // TODO: Add FFI method for contacts when available in Rust
  return DemoData.contacts.map((e) => UnifiedContact.fromDemo(e)).toList();
});

/// Provider for messages in a channel.
///
/// TODO: FFI method for listing messages not yet available.
/// Currently uses demo data for all modes.
final unifiedMessagesProvider = FutureProvider.family<List<UnifiedMessage>, String>((ref, channelId) async {
  // Demo mode or FFI fallback: use demo data
  // TODO: Add FFI method for message history when available in Rust
  return DemoData.messages.map((e) => UnifiedMessage.fromDemo(e)).toList();
});

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

/// Provider for child entities under a parent.
final unifiedChildEntitiesProvider = FutureProvider.family<List<UnifiedEntity>, String>((ref, parentId) async {
  final projects = await ref.watch(unifiedProjectsProvider.future);
  final channels = await ref.watch(unifiedChannelsProvider.future);

  return [
    ...projects.where((p) => p.parentId == parentId),
    ...channels.where((c) => c.parentId == parentId),
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
