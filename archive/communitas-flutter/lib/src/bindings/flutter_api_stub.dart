// Web platform stub for flutter_api.dart
// This file provides the same types as flutter_api.dart but without FRB dependencies.
// All async methods throw UnsupportedError - web runs demo-only.
//
// IMPORTANT: Keep this file in sync with flutter_api.dart types.

import 'dart:typed_data';

/// Generate a random four-word identity.
/// On web, this is not available - demo mode only.
Future<String> generateIdWords() async {
  throw UnsupportedError(
    'generateIdWords is not available on web platform. Demo mode only.',
  );
}

/// Validate a recovery mnemonic (BIP39).
Future<bool> validateRecoveryMnemonic({required String mnemonic}) async {
  throw UnsupportedError(
    'validateRecoveryMnemonic is not available on web platform. Use native FFI instead.',
  );
}

/// Preview identity details from a mnemonic without persisting keys.
Future<FlutterRecoveredIdentity> previewIdentityFromMnemonic({
  required String mnemonic,
  String? passphrase,
}) async {
  throw UnsupportedError(
    'previewIdentityFromMnemonic is not available on web platform. Use native FFI instead.',
  );
}

/// Recover identity from a mnemonic and persist keys to secure storage.
Future<FlutterRecoveredIdentity> recoverIdentityFromMnemonic({
  required String mnemonic,
  String? passphrase,
}) async {
  throw UnsupportedError(
    'recoverIdentityFromMnemonic is not available on web platform. Use native FFI instead.',
  );
}

/// Platform-specific integer type (stub for web).
typedef PlatformInt64 = int;

/// Stub CommunitasApi for web platform.
/// All methods throw UnsupportedError - demo mode only.
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

  Future<String> authExportVault({required bool includeData});

  Future<String> authImportVault({
    required String backupBase64,
    required String password,
  });

  static Future<CommunitasApi> create({
    required String fourWords,
    required String displayName,
    required String deviceName,
    required String storagePath,
  }) async {
    throw UnsupportedError(
      'CommunitasApi.create is not available on web platform. Demo mode only.',
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

  Future<String?> gossipGetConnectionWords();

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

  Future<List<FlutterEvent>> diskCreateDirectory({
    required String entityId,
    required FlutterDiskType diskType,
    required String path,
  });

  Future<List<FlutterEvent>> diskDeleteFile({
    required String entityId,
    required FlutterDiskType diskType,
    required String path,
  });

  Future<FlutterDiskStats> diskGetStats({
    required String entityId,
    required FlutterDiskType diskType,
  });

  Future<List<FlutterFileInfo>> diskListFiles({
    required String entityId,
    required FlutterDiskType diskType,
    required String path,
  });

  Future<Uint8List> diskReadFile({
    required String entityId,
    required FlutterDiskType diskType,
    required String path,
  });

  Future<List<FlutterEvent>> diskWriteFile({
    required String entityId,
    required FlutterDiskType diskType,
    required String path,
    required List<int> data,
  });

  Future<List<FlutterEvent>> messageSend({
    required String entityId,
    required FlutterEntityType entityType,
    required String text,
    String? replyToId,
  });

  Future<List<FlutterEvent>> messageSendDirect({
    required List<String> recipients,
    required String text,
  });

  Future<FlutterKanbanBoard> kanbanCreateBoard({
    required String entityId,
    required String boardName,
    String? description,
  });

  Future<FlutterKanbanCard> kanbanCreateCard({
    required String boardId,
    required String columnId,
    required String title,
    String? description,
    String? assignee,
  });

  Future<FlutterKanbanColumn> kanbanCreateColumn({
    required String boardId,
    required String columnName,
    int? position,
  });

  Future<List<FlutterEvent>> kanbanDeleteCard({
    required String boardId,
    required String cardId,
  });

  Future<FlutterKanbanBoard> kanbanGetBoard({required String boardId});

  Future<List<FlutterKanbanBoard>> kanbanListBoards({required String entityId});

  Future<List<FlutterKanbanCard>> kanbanListCards({
    required String boardId,
    String? columnId,
    String? state,
    String? assigneeId,
    String? tagId,
  });

  Future<List<FlutterKanbanColumn>> kanbanListColumns({required String boardId});

  Future<List<FlutterEvent>> kanbanMoveCard({
    required String boardId,
    required String cardId,
    required String targetColumnId,
    int? position,
  });

  Future<List<FlutterEvent>> kanbanUpdateCard({
    required String boardId,
    required String cardId,
    String? title,
    String? description,
    String? assignee,
  });

  Future<List<FlutterEvent>> messageDelete({
    required String entityId,
    required FlutterEntityType entityType,
    required String messageId,
  });

  Future<List<FlutterEvent>> messageEdit({
    required String entityId,
    required FlutterEntityType entityType,
    required String messageId,
    required String newText,
  });

  Future<List<FlutterEvent>> messageAddReaction({
    required String entityId,
    required FlutterEntityType entityType,
    required String messageId,
    required String emoji,
  });

  Future<List<FlutterEvent>> messageRemoveReaction({
    required String entityId,
    required FlutterEntityType entityType,
    required String messageId,
    required String emoji,
  });

  Future<FlutterMessage> messageGet({
    required String entityId,
    required String messageId,
  });

  Future<List<FlutterMessage>> messageListThread({
    required String entityId,
    required String parentMessageId,
  });

  Future<List<FlutterMessage>> messageListDirect({
    required String otherPeerId,
  });

  Future<List<FlutterMessage>> messageList({required String entityId});

  Future<FlutterContact> contactGet({required String contactId});

  Future<List<FlutterContact>> contactsList();

  Future<List<FlutterContact>> contactsListFavourites();

  Future<List<FlutterContact>> contactsSearch({required String query});

  Future<List<FlutterEvent>> contactCreate({
    required String displayName,
    String? fourWords,
    required bool isFavourite,
  });

  Future<List<FlutterEvent>> contactUpdate({
    required String contactId,
    String? displayName,
    bool? isFavourite,
  });

  Future<List<FlutterEvent>> contactDelete({required String contactId});

  Future<List<FlutterEvent>> contactLink({
    required String contactId,
    required String fourWords,
  });

  Future<List<FlutterEvent>> contactSetFavourite({required String fourWords});

  Future<List<FlutterEvent>> contactRemoveFavourite({required String fourWords});

  Future<List<FlutterEvent>> presenceAnnounce();

  Future<FlutterPresenceRecord?> presenceGetOurRecord();

  Future<FlutterPresenceRecord?> presenceGetCachedPeer({
    required String pubkeyHex,
  });

  Future<FlutterPresenceRecord?> presenceQueryPeer({
    required String pubkeyHex,
  });

  Future<List<String>> presenceListOnlinePeers();

  Future<FlutterPresenceStatus> presenceGetStatus({required String peerId});

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

  factory FlutterEvent.networkingStarted({required String address}) = FlutterEventNetworkingStarted;
  factory FlutterEvent.networkingStopped() = FlutterEventNetworkingStopped;
  factory FlutterEvent.peerConnected({required String peerId}) = FlutterEventPeerConnected;
  factory FlutterEvent.peerDisconnected({required String peerId}) = FlutterEventPeerDisconnected;
  factory FlutterEvent.entityCreated({required String entityId}) = FlutterEventEntityCreated;
  factory FlutterEvent.entityUpdated({required String entityId}) = FlutterEventEntityUpdated;
  factory FlutterEvent.messageSent({
    required String messageId,
    required String entityId,
  }) = FlutterEventMessageSent;
  factory FlutterEvent.messageReceived({
    required String messageId,
    required String entityId,
  }) = FlutterEventMessageReceived;
  factory FlutterEvent.directMessageSent({
    required List<String> messageIds,
    required List<String> recipients,
  }) = FlutterEventDirectMessageSent;
  factory FlutterEvent.messageDeleted({
    required String messageId,
    required String entityId,
  }) = FlutterEventMessageDeleted;
  factory FlutterEvent.messageEdited({
    required String messageId,
    required String entityId,
    required String newText,
    required BigInt editedAt,
  }) = FlutterEventMessageEdited;
  factory FlutterEvent.reactionAdded({
    required String messageId,
    required String entityId,
    required String emoji,
    required String reactorId,
  }) = FlutterEventReactionAdded;
  factory FlutterEvent.reactionRemoved({
    required String messageId,
    required String entityId,
    required String emoji,
    required String reactorId,
  }) = FlutterEventReactionRemoved;
  factory FlutterEvent.inviteCreated({required String inviteId}) = FlutterEventInviteCreated;
  factory FlutterEvent.inviteAccepted({required String inviteId}) = FlutterEventInviteAccepted;
  factory FlutterEvent.inviteRejected({required String inviteId}) = FlutterEventInviteRejected;
  factory FlutterEvent.inviteRevoked({required String inviteId}) = FlutterEventInviteRevoked;
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
  FlutterEventMessageSent({required this.messageId, required this.entityId}) : super._();
}

class FlutterEventMessageReceived extends FlutterEvent {
  final String messageId;
  final String entityId;
  FlutterEventMessageReceived({required this.messageId, required this.entityId}) : super._();
}

class FlutterEventDirectMessageSent extends FlutterEvent {
  final List<String> messageIds;
  final List<String> recipients;
  FlutterEventDirectMessageSent({
    required this.messageIds,
    required this.recipients,
  }) : super._();
}

class FlutterEventMessageDeleted extends FlutterEvent {
  final String messageId;
  final String entityId;
  FlutterEventMessageDeleted({required this.messageId, required this.entityId}) : super._();
}

class FlutterEventMessageEdited extends FlutterEvent {
  final String messageId;
  final String entityId;
  final String newText;
  final BigInt editedAt;
  FlutterEventMessageEdited({
    required this.messageId,
    required this.entityId,
    required this.newText,
    required this.editedAt,
  }) : super._();
}

class FlutterEventReactionAdded extends FlutterEvent {
  final String messageId;
  final String entityId;
  final String emoji;
  final String reactorId;
  FlutterEventReactionAdded({
    required this.messageId,
    required this.entityId,
    required this.emoji,
    required this.reactorId,
  }) : super._();
}

class FlutterEventReactionRemoved extends FlutterEvent {
  final String messageId;
  final String entityId;
  final String emoji;
  final String reactorId;
  FlutterEventReactionRemoved({
    required this.messageId,
    required this.entityId,
    required this.emoji,
    required this.reactorId,
  }) : super._();
}

class FlutterEventInviteCreated extends FlutterEvent {
  final String inviteId;
  FlutterEventInviteCreated({required this.inviteId}) : super._();
}

class FlutterEventInviteAccepted extends FlutterEvent {
  final String inviteId;
  FlutterEventInviteAccepted({required this.inviteId}) : super._();
}

class FlutterEventInviteRejected extends FlutterEvent {
  final String inviteId;
  FlutterEventInviteRejected({required this.inviteId}) : super._();
}

class FlutterEventInviteRevoked extends FlutterEvent {
  final String inviteId;
  FlutterEventInviteRevoked({required this.inviteId}) : super._();
}

class FlutterEventFileWritten extends FlutterEvent {
  final String entityId;
  final String path;
  FlutterEventFileWritten({required this.entityId, required this.path}) : super._();
}

class FlutterEventFileDeleted extends FlutterEvent {
  final String entityId;
  final String path;
  FlutterEventFileDeleted({required this.entityId, required this.path}) : super._();
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

  /// Number of bootstrap nodes in the peer cache
  final int bootstrapCount;

  const FlutterNetworkInfo({
    required this.isActive,
    this.boundPort,
    required this.peerCount,
    this.externalAddress,
    required this.bootstrapConnected,
    required this.bootstrapCount,
  });

  @override
  int get hashCode =>
      isActive.hashCode ^
      boundPort.hashCode ^
      peerCount.hashCode ^
      externalAddress.hashCode ^
      bootstrapConnected.hashCode ^
      bootstrapCount.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterNetworkInfo &&
          runtimeType == other.runtimeType &&
          isActive == other.isActive &&
          boundPort == other.boundPort &&
          peerCount == other.peerCount &&
          externalAddress == other.externalAddress &&
          bootstrapConnected == other.bootstrapConnected &&
          bootstrapCount == other.bootstrapCount;
}

/// Session information
class FlutterSessionInfo {
  final String sessionId;
  final String fourWords;
  final String displayName;

  /// Hex-encoded ML-DSA-87 public key (the user's cryptographic identity)
  final String pubkeyHex;

  const FlutterSessionInfo({
    required this.sessionId,
    required this.fourWords,
    required this.displayName,
    required this.pubkeyHex,
  });

  @override
  int get hashCode => sessionId.hashCode ^ fourWords.hashCode ^ displayName.hashCode ^ pubkeyHex.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterSessionInfo &&
          runtimeType == other.runtimeType &&
          sessionId == other.sessionId &&
          fourWords == other.fourWords &&
          displayName == other.displayName &&
          pubkeyHex == other.pubkeyHex;
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
  int get hashCode => fourWords.hashCode ^ displayName.hashCode ^ deviceName.hashCode ^ deviceType.hashCode;

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
      fourWords.hashCode ^ displayName.hashCode ^ createdAt.hashCode ^ lastAccessed.hashCode ^ sizeBytes.hashCode;

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

/// Message response data
class FlutterMessage {
  final String id;
  final String entityId;
  final String author;
  final String text;
  final PlatformInt64 timestamp;
  final String? replyToId;
  final List<FlutterReaction> reactions;
  final BigInt? editedAt;

  const FlutterMessage({
    required this.id,
    required this.entityId,
    required this.author,
    required this.text,
    required this.timestamp,
    this.replyToId,
    required this.reactions,
    this.editedAt,
  });

  @override
  int get hashCode =>
      id.hashCode ^
      entityId.hashCode ^
      author.hashCode ^
      text.hashCode ^
      timestamp.hashCode ^
      replyToId.hashCode ^
      reactions.hashCode ^
      editedAt.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterMessage &&
          runtimeType == other.runtimeType &&
          id == other.id &&
          entityId == other.entityId &&
          author == other.author &&
          text == other.text &&
          timestamp == other.timestamp &&
          replyToId == other.replyToId &&
          reactions == other.reactions &&
          editedAt == other.editedAt;
}

/// Reaction response data
class FlutterReaction {
  final String emoji;
  final int count;
  final bool userReacted;
  final List<String> peerIds;

  const FlutterReaction({
    required this.emoji,
    required this.count,
    required this.userReacted,
    required this.peerIds,
  });

  @override
  int get hashCode => emoji.hashCode ^ count.hashCode ^ userReacted.hashCode ^ peerIds.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterReaction &&
          runtimeType == other.runtimeType &&
          emoji == other.emoji &&
          count == other.count &&
          userReacted == other.userReacted &&
          peerIds == other.peerIds;
}

/// Contact response data
class FlutterContact {
  final String id;
  final String displayName;
  final String? fourWords;
  final bool isFavourite;
  final bool isOnline;
  final PlatformInt64 createdAt;
  final PlatformInt64? lastSeen;

  const FlutterContact({
    required this.id,
    required this.displayName,
    this.fourWords,
    required this.isFavourite,
    required this.isOnline,
    required this.createdAt,
    this.lastSeen,
  });

  @override
  int get hashCode =>
      id.hashCode ^
      displayName.hashCode ^
      fourWords.hashCode ^
      isFavourite.hashCode ^
      isOnline.hashCode ^
      createdAt.hashCode ^
      lastSeen.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterContact &&
          runtimeType == other.runtimeType &&
          id == other.id &&
          displayName == other.displayName &&
          fourWords == other.fourWords &&
          isFavourite == other.isFavourite &&
          isOnline == other.isOnline &&
          createdAt == other.createdAt &&
          lastSeen == other.lastSeen;
}

/// Disk stats response data
class FlutterDiskStats {
  final String entityId;
  final FlutterDiskType diskType;
  final BigInt usedBytes;
  final int fileCount;
  final int dirCount;

  const FlutterDiskStats({
    required this.entityId,
    required this.diskType,
    required this.usedBytes,
    required this.fileCount,
    required this.dirCount,
  });

  @override
  int get hashCode =>
      entityId.hashCode ^ diskType.hashCode ^ usedBytes.hashCode ^ fileCount.hashCode ^ dirCount.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterDiskStats &&
          runtimeType == other.runtimeType &&
          entityId == other.entityId &&
          diskType == other.diskType &&
          usedBytes == other.usedBytes &&
          fileCount == other.fileCount &&
          dirCount == other.dirCount;
}

/// Disk type enumeration
enum FlutterDiskType {
  private,
  public,
  shared,
}

/// File info response data
class FlutterFileInfo {
  final String path;
  final String name;
  final bool isDirectory;
  final BigInt sizeBytes;
  final PlatformInt64 modifiedAt;

  const FlutterFileInfo({
    required this.path,
    required this.name,
    required this.isDirectory,
    required this.sizeBytes,
    required this.modifiedAt,
  });

  @override
  int get hashCode => path.hashCode ^ name.hashCode ^ isDirectory.hashCode ^ sizeBytes.hashCode ^ modifiedAt.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterFileInfo &&
          runtimeType == other.runtimeType &&
          path == other.path &&
          name == other.name &&
          isDirectory == other.isDirectory &&
          sizeBytes == other.sizeBytes &&
          modifiedAt == other.modifiedAt;
}

/// Kanban board response data
class FlutterKanbanBoard {
  final String id;
  final String entityId;
  final String name;
  final String? description;
  final int columnCount;

  const FlutterKanbanBoard({
    required this.id,
    required this.entityId,
    required this.name,
    this.description,
    required this.columnCount,
  });

  @override
  int get hashCode => id.hashCode ^ entityId.hashCode ^ name.hashCode ^ description.hashCode ^ columnCount.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterKanbanBoard &&
          runtimeType == other.runtimeType &&
          id == other.id &&
          entityId == other.entityId &&
          name == other.name &&
          description == other.description &&
          columnCount == other.columnCount;
}

/// Kanban card response data
class FlutterKanbanCard {
  final String id;
  final String columnId;
  final String title;
  final String? description;
  final String? assignee;
  final int position;

  const FlutterKanbanCard({
    required this.id,
    required this.columnId,
    required this.title,
    this.description,
    this.assignee,
    required this.position,
  });

  @override
  int get hashCode =>
      id.hashCode ^ columnId.hashCode ^ title.hashCode ^ description.hashCode ^ assignee.hashCode ^ position.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterKanbanCard &&
          runtimeType == other.runtimeType &&
          id == other.id &&
          columnId == other.columnId &&
          title == other.title &&
          description == other.description &&
          assignee == other.assignee &&
          position == other.position;
}

/// Kanban column response data
class FlutterKanbanColumn {
  final String id;
  final String boardId;
  final String name;
  final int position;
  final String? color;
  final int? wipLimit;

  const FlutterKanbanColumn({
    required this.id,
    required this.boardId,
    required this.name,
    required this.position,
    this.color,
    this.wipLimit,
  });

  @override
  int get hashCode =>
      id.hashCode ^ boardId.hashCode ^ name.hashCode ^ position.hashCode ^ color.hashCode ^ wipLimit.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterKanbanColumn &&
          runtimeType == other.runtimeType &&
          id == other.id &&
          boardId == other.boardId &&
          name == other.name &&
          position == other.position &&
          color == other.color &&
          wipLimit == other.wipLimit;
}

/// Presence record exposed to Flutter
class FlutterPresenceRecord {
  final String pubkeyHex;
  final String connectionWords;
  final BigInt timestamp;
  final bool isVerified;

  const FlutterPresenceRecord({
    required this.pubkeyHex,
    required this.connectionWords,
    required this.timestamp,
    required this.isVerified,
  });

  @override
  int get hashCode => pubkeyHex.hashCode ^ connectionWords.hashCode ^ timestamp.hashCode ^ isVerified.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterPresenceRecord &&
          runtimeType == other.runtimeType &&
          pubkeyHex == other.pubkeyHex &&
          connectionWords == other.connectionWords &&
          timestamp == other.timestamp &&
          isVerified == other.isVerified;
}

/// Presence status for a peer (online/offline/unknown)
class FlutterPresenceStatus {
  final String peerId;
  final String status;
  final PlatformInt64 lastSeen;

  const FlutterPresenceStatus({
    required this.peerId,
    required this.status,
    required this.lastSeen,
  });

  @override
  int get hashCode => peerId.hashCode ^ status.hashCode ^ lastSeen.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterPresenceStatus &&
          runtimeType == other.runtimeType &&
          peerId == other.peerId &&
          status == other.status &&
          lastSeen == other.lastSeen;
}

/// Recovered identity preview/result
class FlutterRecoveredIdentity {
  final String fourWords;
  final String pubkeyHex;

  const FlutterRecoveredIdentity({
    required this.fourWords,
    required this.pubkeyHex,
  });

  @override
  int get hashCode => fourWords.hashCode ^ pubkeyHex.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FlutterRecoveredIdentity &&
          runtimeType == other.runtimeType &&
          fourWords == other.fourWords &&
          pubkeyHex == other.pubkeyHex;
}
