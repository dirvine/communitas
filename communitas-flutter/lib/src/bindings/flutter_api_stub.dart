// Web platform stub for flutter_api.dart
// This file provides the same types as flutter_api.dart but without FRB dependencies.
// All async methods throw UnsupportedError - web uses bridge mode instead.
//
// IMPORTANT: Keep this file in sync with flutter_api.dart types.

import 'package:flutter/foundation.dart';

/// Generate a random four-word identity.
/// On web, this is not available - use bridge API instead.
Future<String> generateIdWords() async {
  throw UnsupportedError(
    'generateIdWords is not available on web platform. Use bridge API instead.',
  );
}

/// Platform-specific integer type (stub for web).
typedef PlatformInt64 = int;

/// Stub CommunitasApi for web platform.
/// All methods throw UnsupportedError - web uses bridge mode.
abstract class CommunitasApi {
  Future<String> authCreateVault({
    required String fourWords,
    required String displayName,
    required String password,
  });

  Future<void> authDeleteVault({
    required String fourWords,
    required String password,
  });

  Future<FlutterSessionInfo?> authGetCurrentSession();

  Future<List<FlutterVaultInfo>> authListVaults();

  Future<FlutterSessionInfo> authLogin({
    required String fourWords,
    required String password,
  });

  Future<void> authLogout();

  Future<bool> authVaultExists({required String fourWords});

  static Future<CommunitasApi> create({
    required String fourWords,
    required String displayName,
    required String deviceName,
    required String storagePath,
  }) async {
    throw UnsupportedError(
      'CommunitasApi.create is not available on web platform. Use bridge mode instead.',
    );
  }

  Future<List<FlutterEvent>> entityAddMember({
    required FlutterEntityType entityType,
    required String entityId,
    required String memberId,
    required String role,
  });

  Future<List<FlutterEvent>> entityCreate({
    required String name,
    required FlutterEntityType entityType,
    String? description,
    String? parentOrgId,
  });

  Future<FlutterEntity> entityGet({required String entityId});

  Future<List<FlutterEntity>> entityList();

  Future<List<FlutterEntity>> entityListByType({
    required FlutterEntityType entityType,
  });

  Future<List<FlutterEvent>> entityRemoveMember({
    required FlutterEntityType entityType,
    required String entityId,
    required String memberId,
  });

  Future<FlutterUserProfile> getProfile();

  Future<List<FlutterEvent>> gossipConnectToPeer({required String fourWords});

  Future<FlutterNetworkInfo> gossipGetNetworkInfo();

  Future<List<FlutterEvent>> gossipStart({int? port});

  Future<List<FlutterEvent>> gossipStop();

  Future<List<FlutterEvent>> inviteAccept({required String inviteId});

  Future<List<FlutterEvent>> inviteCreate({
    required String recipientId,
    required FlutterEntityType entityType,
    required String entityId,
    required String role,
    String? message,
  });

  Future<List<FlutterEvent>> inviteReject({required String inviteId});

  Future<List<FlutterEvent>> inviteRevoke({required String inviteId});

  Future<List<FlutterEvent>> messageSend({
    required String entityId,
    required FlutterEntityType entityType,
    required String text,
    String? replyToId,
  });

  Future<List<FlutterEvent>> updateDisplayName({required String displayName});
}

/// Entity information
class FlutterEntity {
  final String id;
  final String name;
  final FlutterEntityType entityType;
  final String? description;
  final String createdBy;
  final PlatformInt64 createdAt;
  final BigInt memberCount;
  final String? parentOrgId;
  final String? networkFourWords;
  final bool isLocalOnly;

  const FlutterEntity({
    required this.id,
    required this.name,
    required this.entityType,
    this.description,
    required this.createdBy,
    required this.createdAt,
    required this.memberCount,
    this.parentOrgId,
    this.networkFourWords,
    required this.isLocalOnly,
  });

  @override
  int get hashCode =>
      id.hashCode ^
      name.hashCode ^
      entityType.hashCode ^
      description.hashCode ^
      createdBy.hashCode ^
      createdAt.hashCode ^
      memberCount.hashCode ^
      parentOrgId.hashCode ^
      networkFourWords.hashCode ^
      isLocalOnly.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterEntity &&
          runtimeType == other.runtimeType &&
          id == other.id &&
          name == other.name &&
          entityType == other.entityType &&
          description == other.description &&
          createdBy == other.createdBy &&
          createdAt == other.createdAt &&
          memberCount == other.memberCount &&
          parentOrgId == other.parentOrgId &&
          networkFourWords == other.networkFourWords &&
          isLocalOnly == other.isLocalOnly;
}

/// Entity type enumeration
enum FlutterEntityType {
  group,
  channel,
  project,
  organisation,
  person,
}

/// Event type for Flutter callbacks (stub)
abstract class FlutterEvent {
  const FlutterEvent._();

  factory FlutterEvent.networkingStarted({required String address}) =
      FlutterEventNetworkingStarted;
  factory FlutterEvent.networkingStopped() = FlutterEventNetworkingStopped;
  factory FlutterEvent.peerConnected({required String peerId}) =
      FlutterEventPeerConnected;
  factory FlutterEvent.peerDisconnected({required String peerId}) =
      FlutterEventPeerDisconnected;
  factory FlutterEvent.entityCreated({required String entityId}) =
      FlutterEventEntityCreated;
  factory FlutterEvent.entityUpdated({required String entityId}) =
      FlutterEventEntityUpdated;
  factory FlutterEvent.messageSent({
    required String messageId,
    required String entityId,
  }) = FlutterEventMessageSent;
  factory FlutterEvent.messageReceived({
    required String messageId,
    required String entityId,
  }) = FlutterEventMessageReceived;
  factory FlutterEvent.inviteCreated({required String inviteId}) =
      FlutterEventInviteCreated;
  factory FlutterEvent.inviteAccepted({required String inviteId}) =
      FlutterEventInviteAccepted;
  factory FlutterEvent.fileWritten({
    required String entityId,
    required String path,
  }) = FlutterEventFileWritten;
  factory FlutterEvent.fileDeleted({
    required String entityId,
    required String path,
  }) = FlutterEventFileDeleted;
  factory FlutterEvent.error({
    required String code,
    required String message,
  }) = FlutterEventError;
}

class FlutterEventNetworkingStarted extends FlutterEvent {
  final String address;
  FlutterEventNetworkingStarted({required this.address}) : super._();
}

class FlutterEventNetworkingStopped extends FlutterEvent {
  FlutterEventNetworkingStopped() : super._();
}

class FlutterEventPeerConnected extends FlutterEvent {
  final String peerId;
  FlutterEventPeerConnected({required this.peerId}) : super._();
}

class FlutterEventPeerDisconnected extends FlutterEvent {
  final String peerId;
  FlutterEventPeerDisconnected({required this.peerId}) : super._();
}

class FlutterEventEntityCreated extends FlutterEvent {
  final String entityId;
  FlutterEventEntityCreated({required this.entityId}) : super._();
}

class FlutterEventEntityUpdated extends FlutterEvent {
  final String entityId;
  FlutterEventEntityUpdated({required this.entityId}) : super._();
}

class FlutterEventMessageSent extends FlutterEvent {
  final String messageId;
  final String entityId;
  FlutterEventMessageSent({required this.messageId, required this.entityId})
      : super._();
}

class FlutterEventMessageReceived extends FlutterEvent {
  final String messageId;
  final String entityId;
  FlutterEventMessageReceived({required this.messageId, required this.entityId})
      : super._();
}

class FlutterEventInviteCreated extends FlutterEvent {
  final String inviteId;
  FlutterEventInviteCreated({required this.inviteId}) : super._();
}

class FlutterEventInviteAccepted extends FlutterEvent {
  final String inviteId;
  FlutterEventInviteAccepted({required this.inviteId}) : super._();
}

class FlutterEventFileWritten extends FlutterEvent {
  final String entityId;
  final String path;
  FlutterEventFileWritten({required this.entityId, required this.path})
      : super._();
}

class FlutterEventFileDeleted extends FlutterEvent {
  final String entityId;
  final String path;
  FlutterEventFileDeleted({required this.entityId, required this.path})
      : super._();
}

class FlutterEventError extends FlutterEvent {
  final String code;
  final String message;
  FlutterEventError({required this.code, required this.message}) : super._();
}

/// Network status information
class FlutterNetworkInfo {
  final bool isActive;
  final int? boundPort;
  final int peerCount;
  final String? externalAddress;
  final bool bootstrapConnected;

  const FlutterNetworkInfo({
    required this.isActive,
    this.boundPort,
    required this.peerCount,
    this.externalAddress,
    required this.bootstrapConnected,
  });

  @override
  int get hashCode =>
      isActive.hashCode ^
      boundPort.hashCode ^
      peerCount.hashCode ^
      externalAddress.hashCode ^
      bootstrapConnected.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterNetworkInfo &&
          runtimeType == other.runtimeType &&
          isActive == other.isActive &&
          boundPort == other.boundPort &&
          peerCount == other.peerCount &&
          externalAddress == other.externalAddress &&
          bootstrapConnected == other.bootstrapConnected;
}

/// Session information
class FlutterSessionInfo {
  final String sessionId;
  final String fourWords;
  final String displayName;

  const FlutterSessionInfo({
    required this.sessionId,
    required this.fourWords,
    required this.displayName,
  });

  @override
  int get hashCode =>
      sessionId.hashCode ^ fourWords.hashCode ^ displayName.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterSessionInfo &&
          runtimeType == other.runtimeType &&
          sessionId == other.sessionId &&
          fourWords == other.fourWords &&
          displayName == other.displayName;
}

/// User profile information
class FlutterUserProfile {
  final String fourWords;
  final String displayName;
  final String deviceName;
  final String deviceType;

  const FlutterUserProfile({
    required this.fourWords,
    required this.displayName,
    required this.deviceName,
    required this.deviceType,
  });

  @override
  int get hashCode =>
      fourWords.hashCode ^
      displayName.hashCode ^
      deviceName.hashCode ^
      deviceType.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterUserProfile &&
          runtimeType == other.runtimeType &&
          fourWords == other.fourWords &&
          displayName == other.displayName &&
          deviceName == other.deviceName &&
          deviceType == other.deviceType;
}

/// Vault information
class FlutterVaultInfo {
  final String fourWords;
  final String displayName;
  final BigInt createdAt;
  final BigInt lastAccessed;
  final BigInt sizeBytes;

  const FlutterVaultInfo({
    required this.fourWords,
    required this.displayName,
    required this.createdAt,
    required this.lastAccessed,
    required this.sizeBytes,
  });

  @override
  int get hashCode =>
      fourWords.hashCode ^
      displayName.hashCode ^
      createdAt.hashCode ^
      lastAccessed.hashCode ^
      sizeBytes.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterVaultInfo &&
          runtimeType == other.runtimeType &&
          fourWords == other.fourWords &&
          displayName == other.displayName &&
          createdAt == other.createdAt &&
          lastAccessed == other.lastAccessed &&
          sizeBytes == other.sizeBytes;
}
