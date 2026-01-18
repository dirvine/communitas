// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'flutter_api.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$FlutterEvent {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $FlutterEventCopyWith<$Res> {
  factory $FlutterEventCopyWith(FlutterEvent value, $Res Function(FlutterEvent) then) =
      _$FlutterEventCopyWithImpl<$Res, FlutterEvent>;
}

/// @nodoc
class _$FlutterEventCopyWithImpl<$Res, $Val extends FlutterEvent> implements $FlutterEventCopyWith<$Res> {
  _$FlutterEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$FlutterEvent_NetworkingStartedImplCopyWith<$Res> {
  factory _$$FlutterEvent_NetworkingStartedImplCopyWith(
          _$FlutterEvent_NetworkingStartedImpl value, $Res Function(_$FlutterEvent_NetworkingStartedImpl) then) =
      __$$FlutterEvent_NetworkingStartedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String address});
}

/// @nodoc
class __$$FlutterEvent_NetworkingStartedImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_NetworkingStartedImpl>
    implements _$$FlutterEvent_NetworkingStartedImplCopyWith<$Res> {
  __$$FlutterEvent_NetworkingStartedImplCopyWithImpl(
      _$FlutterEvent_NetworkingStartedImpl _value, $Res Function(_$FlutterEvent_NetworkingStartedImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? address = null,
  }) {
    return _then(_$FlutterEvent_NetworkingStartedImpl(
      address: null == address
          ? _value.address
          : address // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_NetworkingStartedImpl extends FlutterEvent_NetworkingStarted {
  const _$FlutterEvent_NetworkingStartedImpl({required this.address}) : super._();

  @override
  final String address;

  @override
  String toString() {
    return 'FlutterEvent.networkingStarted(address: $address)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_NetworkingStartedImpl &&
            (identical(other.address, address) || other.address == address));
  }

  @override
  int get hashCode => Object.hash(runtimeType, address);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_NetworkingStartedImplCopyWith<_$FlutterEvent_NetworkingStartedImpl> get copyWith =>
      __$$FlutterEvent_NetworkingStartedImplCopyWithImpl<_$FlutterEvent_NetworkingStartedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return networkingStarted(address);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return networkingStarted?.call(address);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (networkingStarted != null) {
      return networkingStarted(address);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return networkingStarted(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return networkingStarted?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (networkingStarted != null) {
      return networkingStarted(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_NetworkingStarted extends FlutterEvent {
  const factory FlutterEvent_NetworkingStarted({required final String address}) = _$FlutterEvent_NetworkingStartedImpl;
  const FlutterEvent_NetworkingStarted._() : super._();

  String get address;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_NetworkingStartedImplCopyWith<_$FlutterEvent_NetworkingStartedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_NetworkingStoppedImplCopyWith<$Res> {
  factory _$$FlutterEvent_NetworkingStoppedImplCopyWith(
          _$FlutterEvent_NetworkingStoppedImpl value, $Res Function(_$FlutterEvent_NetworkingStoppedImpl) then) =
      __$$FlutterEvent_NetworkingStoppedImplCopyWithImpl<$Res>;
}

/// @nodoc
class __$$FlutterEvent_NetworkingStoppedImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_NetworkingStoppedImpl>
    implements _$$FlutterEvent_NetworkingStoppedImplCopyWith<$Res> {
  __$$FlutterEvent_NetworkingStoppedImplCopyWithImpl(
      _$FlutterEvent_NetworkingStoppedImpl _value, $Res Function(_$FlutterEvent_NetworkingStoppedImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc

class _$FlutterEvent_NetworkingStoppedImpl extends FlutterEvent_NetworkingStopped {
  const _$FlutterEvent_NetworkingStoppedImpl() : super._();

  @override
  String toString() {
    return 'FlutterEvent.networkingStopped()';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is _$FlutterEvent_NetworkingStoppedImpl);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return networkingStopped();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return networkingStopped?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (networkingStopped != null) {
      return networkingStopped();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return networkingStopped(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return networkingStopped?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (networkingStopped != null) {
      return networkingStopped(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_NetworkingStopped extends FlutterEvent {
  const factory FlutterEvent_NetworkingStopped() = _$FlutterEvent_NetworkingStoppedImpl;
  const FlutterEvent_NetworkingStopped._() : super._();
}

/// @nodoc
abstract class _$$FlutterEvent_PeerConnectedImplCopyWith<$Res> {
  factory _$$FlutterEvent_PeerConnectedImplCopyWith(
          _$FlutterEvent_PeerConnectedImpl value, $Res Function(_$FlutterEvent_PeerConnectedImpl) then) =
      __$$FlutterEvent_PeerConnectedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String peerId});
}

/// @nodoc
class __$$FlutterEvent_PeerConnectedImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_PeerConnectedImpl>
    implements _$$FlutterEvent_PeerConnectedImplCopyWith<$Res> {
  __$$FlutterEvent_PeerConnectedImplCopyWithImpl(
      _$FlutterEvent_PeerConnectedImpl _value, $Res Function(_$FlutterEvent_PeerConnectedImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? peerId = null,
  }) {
    return _then(_$FlutterEvent_PeerConnectedImpl(
      peerId: null == peerId
          ? _value.peerId
          : peerId // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_PeerConnectedImpl extends FlutterEvent_PeerConnected {
  const _$FlutterEvent_PeerConnectedImpl({required this.peerId}) : super._();

  @override
  final String peerId;

  @override
  String toString() {
    return 'FlutterEvent.peerConnected(peerId: $peerId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_PeerConnectedImpl &&
            (identical(other.peerId, peerId) || other.peerId == peerId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, peerId);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_PeerConnectedImplCopyWith<_$FlutterEvent_PeerConnectedImpl> get copyWith =>
      __$$FlutterEvent_PeerConnectedImplCopyWithImpl<_$FlutterEvent_PeerConnectedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return peerConnected(peerId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return peerConnected?.call(peerId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (peerConnected != null) {
      return peerConnected(peerId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return peerConnected(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return peerConnected?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (peerConnected != null) {
      return peerConnected(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_PeerConnected extends FlutterEvent {
  const factory FlutterEvent_PeerConnected({required final String peerId}) = _$FlutterEvent_PeerConnectedImpl;
  const FlutterEvent_PeerConnected._() : super._();

  String get peerId;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_PeerConnectedImplCopyWith<_$FlutterEvent_PeerConnectedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_PeerDisconnectedImplCopyWith<$Res> {
  factory _$$FlutterEvent_PeerDisconnectedImplCopyWith(
          _$FlutterEvent_PeerDisconnectedImpl value, $Res Function(_$FlutterEvent_PeerDisconnectedImpl) then) =
      __$$FlutterEvent_PeerDisconnectedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String peerId});
}

/// @nodoc
class __$$FlutterEvent_PeerDisconnectedImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_PeerDisconnectedImpl>
    implements _$$FlutterEvent_PeerDisconnectedImplCopyWith<$Res> {
  __$$FlutterEvent_PeerDisconnectedImplCopyWithImpl(
      _$FlutterEvent_PeerDisconnectedImpl _value, $Res Function(_$FlutterEvent_PeerDisconnectedImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? peerId = null,
  }) {
    return _then(_$FlutterEvent_PeerDisconnectedImpl(
      peerId: null == peerId
          ? _value.peerId
          : peerId // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_PeerDisconnectedImpl extends FlutterEvent_PeerDisconnected {
  const _$FlutterEvent_PeerDisconnectedImpl({required this.peerId}) : super._();

  @override
  final String peerId;

  @override
  String toString() {
    return 'FlutterEvent.peerDisconnected(peerId: $peerId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_PeerDisconnectedImpl &&
            (identical(other.peerId, peerId) || other.peerId == peerId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, peerId);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_PeerDisconnectedImplCopyWith<_$FlutterEvent_PeerDisconnectedImpl> get copyWith =>
      __$$FlutterEvent_PeerDisconnectedImplCopyWithImpl<_$FlutterEvent_PeerDisconnectedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return peerDisconnected(peerId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return peerDisconnected?.call(peerId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (peerDisconnected != null) {
      return peerDisconnected(peerId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return peerDisconnected(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return peerDisconnected?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (peerDisconnected != null) {
      return peerDisconnected(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_PeerDisconnected extends FlutterEvent {
  const factory FlutterEvent_PeerDisconnected({required final String peerId}) = _$FlutterEvent_PeerDisconnectedImpl;
  const FlutterEvent_PeerDisconnected._() : super._();

  String get peerId;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_PeerDisconnectedImplCopyWith<_$FlutterEvent_PeerDisconnectedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_EntityCreatedImplCopyWith<$Res> {
  factory _$$FlutterEvent_EntityCreatedImplCopyWith(
          _$FlutterEvent_EntityCreatedImpl value, $Res Function(_$FlutterEvent_EntityCreatedImpl) then) =
      __$$FlutterEvent_EntityCreatedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String entityId});
}

/// @nodoc
class __$$FlutterEvent_EntityCreatedImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_EntityCreatedImpl>
    implements _$$FlutterEvent_EntityCreatedImplCopyWith<$Res> {
  __$$FlutterEvent_EntityCreatedImplCopyWithImpl(
      _$FlutterEvent_EntityCreatedImpl _value, $Res Function(_$FlutterEvent_EntityCreatedImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? entityId = null,
  }) {
    return _then(_$FlutterEvent_EntityCreatedImpl(
      entityId: null == entityId
          ? _value.entityId
          : entityId // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_EntityCreatedImpl extends FlutterEvent_EntityCreated {
  const _$FlutterEvent_EntityCreatedImpl({required this.entityId}) : super._();

  @override
  final String entityId;

  @override
  String toString() {
    return 'FlutterEvent.entityCreated(entityId: $entityId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_EntityCreatedImpl &&
            (identical(other.entityId, entityId) || other.entityId == entityId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, entityId);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_EntityCreatedImplCopyWith<_$FlutterEvent_EntityCreatedImpl> get copyWith =>
      __$$FlutterEvent_EntityCreatedImplCopyWithImpl<_$FlutterEvent_EntityCreatedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return entityCreated(entityId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return entityCreated?.call(entityId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (entityCreated != null) {
      return entityCreated(entityId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return entityCreated(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return entityCreated?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (entityCreated != null) {
      return entityCreated(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_EntityCreated extends FlutterEvent {
  const factory FlutterEvent_EntityCreated({required final String entityId}) = _$FlutterEvent_EntityCreatedImpl;
  const FlutterEvent_EntityCreated._() : super._();

  String get entityId;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_EntityCreatedImplCopyWith<_$FlutterEvent_EntityCreatedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_EntityUpdatedImplCopyWith<$Res> {
  factory _$$FlutterEvent_EntityUpdatedImplCopyWith(
          _$FlutterEvent_EntityUpdatedImpl value, $Res Function(_$FlutterEvent_EntityUpdatedImpl) then) =
      __$$FlutterEvent_EntityUpdatedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String entityId});
}

/// @nodoc
class __$$FlutterEvent_EntityUpdatedImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_EntityUpdatedImpl>
    implements _$$FlutterEvent_EntityUpdatedImplCopyWith<$Res> {
  __$$FlutterEvent_EntityUpdatedImplCopyWithImpl(
      _$FlutterEvent_EntityUpdatedImpl _value, $Res Function(_$FlutterEvent_EntityUpdatedImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? entityId = null,
  }) {
    return _then(_$FlutterEvent_EntityUpdatedImpl(
      entityId: null == entityId
          ? _value.entityId
          : entityId // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_EntityUpdatedImpl extends FlutterEvent_EntityUpdated {
  const _$FlutterEvent_EntityUpdatedImpl({required this.entityId}) : super._();

  @override
  final String entityId;

  @override
  String toString() {
    return 'FlutterEvent.entityUpdated(entityId: $entityId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_EntityUpdatedImpl &&
            (identical(other.entityId, entityId) || other.entityId == entityId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, entityId);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_EntityUpdatedImplCopyWith<_$FlutterEvent_EntityUpdatedImpl> get copyWith =>
      __$$FlutterEvent_EntityUpdatedImplCopyWithImpl<_$FlutterEvent_EntityUpdatedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return entityUpdated(entityId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return entityUpdated?.call(entityId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (entityUpdated != null) {
      return entityUpdated(entityId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return entityUpdated(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return entityUpdated?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (entityUpdated != null) {
      return entityUpdated(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_EntityUpdated extends FlutterEvent {
  const factory FlutterEvent_EntityUpdated({required final String entityId}) = _$FlutterEvent_EntityUpdatedImpl;
  const FlutterEvent_EntityUpdated._() : super._();

  String get entityId;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_EntityUpdatedImplCopyWith<_$FlutterEvent_EntityUpdatedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_MessageSentImplCopyWith<$Res> {
  factory _$$FlutterEvent_MessageSentImplCopyWith(
          _$FlutterEvent_MessageSentImpl value, $Res Function(_$FlutterEvent_MessageSentImpl) then) =
      __$$FlutterEvent_MessageSentImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String messageId, String entityId});
}

/// @nodoc
class __$$FlutterEvent_MessageSentImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_MessageSentImpl>
    implements _$$FlutterEvent_MessageSentImplCopyWith<$Res> {
  __$$FlutterEvent_MessageSentImplCopyWithImpl(
      _$FlutterEvent_MessageSentImpl _value, $Res Function(_$FlutterEvent_MessageSentImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? messageId = null,
    Object? entityId = null,
  }) {
    return _then(_$FlutterEvent_MessageSentImpl(
      messageId: null == messageId
          ? _value.messageId
          : messageId // ignore: cast_nullable_to_non_nullable
              as String,
      entityId: null == entityId
          ? _value.entityId
          : entityId // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_MessageSentImpl extends FlutterEvent_MessageSent {
  const _$FlutterEvent_MessageSentImpl({required this.messageId, required this.entityId}) : super._();

  @override
  final String messageId;
  @override
  final String entityId;

  @override
  String toString() {
    return 'FlutterEvent.messageSent(messageId: $messageId, entityId: $entityId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_MessageSentImpl &&
            (identical(other.messageId, messageId) || other.messageId == messageId) &&
            (identical(other.entityId, entityId) || other.entityId == entityId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, messageId, entityId);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_MessageSentImplCopyWith<_$FlutterEvent_MessageSentImpl> get copyWith =>
      __$$FlutterEvent_MessageSentImplCopyWithImpl<_$FlutterEvent_MessageSentImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return messageSent(messageId, entityId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return messageSent?.call(messageId, entityId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (messageSent != null) {
      return messageSent(messageId, entityId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return messageSent(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return messageSent?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (messageSent != null) {
      return messageSent(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_MessageSent extends FlutterEvent {
  const factory FlutterEvent_MessageSent({required final String messageId, required final String entityId}) =
      _$FlutterEvent_MessageSentImpl;
  const FlutterEvent_MessageSent._() : super._();

  String get messageId;
  String get entityId;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_MessageSentImplCopyWith<_$FlutterEvent_MessageSentImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_MessageReceivedImplCopyWith<$Res> {
  factory _$$FlutterEvent_MessageReceivedImplCopyWith(
          _$FlutterEvent_MessageReceivedImpl value, $Res Function(_$FlutterEvent_MessageReceivedImpl) then) =
      __$$FlutterEvent_MessageReceivedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String messageId, String entityId});
}

/// @nodoc
class __$$FlutterEvent_MessageReceivedImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_MessageReceivedImpl>
    implements _$$FlutterEvent_MessageReceivedImplCopyWith<$Res> {
  __$$FlutterEvent_MessageReceivedImplCopyWithImpl(
      _$FlutterEvent_MessageReceivedImpl _value, $Res Function(_$FlutterEvent_MessageReceivedImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? messageId = null,
    Object? entityId = null,
  }) {
    return _then(_$FlutterEvent_MessageReceivedImpl(
      messageId: null == messageId
          ? _value.messageId
          : messageId // ignore: cast_nullable_to_non_nullable
              as String,
      entityId: null == entityId
          ? _value.entityId
          : entityId // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_MessageReceivedImpl extends FlutterEvent_MessageReceived {
  const _$FlutterEvent_MessageReceivedImpl({required this.messageId, required this.entityId}) : super._();

  @override
  final String messageId;
  @override
  final String entityId;

  @override
  String toString() {
    return 'FlutterEvent.messageReceived(messageId: $messageId, entityId: $entityId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_MessageReceivedImpl &&
            (identical(other.messageId, messageId) || other.messageId == messageId) &&
            (identical(other.entityId, entityId) || other.entityId == entityId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, messageId, entityId);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_MessageReceivedImplCopyWith<_$FlutterEvent_MessageReceivedImpl> get copyWith =>
      __$$FlutterEvent_MessageReceivedImplCopyWithImpl<_$FlutterEvent_MessageReceivedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return messageReceived(messageId, entityId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return messageReceived?.call(messageId, entityId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (messageReceived != null) {
      return messageReceived(messageId, entityId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return messageReceived(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return messageReceived?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (messageReceived != null) {
      return messageReceived(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_MessageReceived extends FlutterEvent {
  const factory FlutterEvent_MessageReceived({required final String messageId, required final String entityId}) =
      _$FlutterEvent_MessageReceivedImpl;
  const FlutterEvent_MessageReceived._() : super._();

  String get messageId;
  String get entityId;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_MessageReceivedImplCopyWith<_$FlutterEvent_MessageReceivedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_DirectMessageSentImplCopyWith<$Res> {
  factory _$$FlutterEvent_DirectMessageSentImplCopyWith(
          _$FlutterEvent_DirectMessageSentImpl value, $Res Function(_$FlutterEvent_DirectMessageSentImpl) then) =
      __$$FlutterEvent_DirectMessageSentImplCopyWithImpl<$Res>;
  @useResult
  $Res call({List<String> messageIds, List<String> recipients});
}

/// @nodoc
class __$$FlutterEvent_DirectMessageSentImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_DirectMessageSentImpl>
    implements _$$FlutterEvent_DirectMessageSentImplCopyWith<$Res> {
  __$$FlutterEvent_DirectMessageSentImplCopyWithImpl(
      _$FlutterEvent_DirectMessageSentImpl _value, $Res Function(_$FlutterEvent_DirectMessageSentImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? messageIds = null,
    Object? recipients = null,
  }) {
    return _then(_$FlutterEvent_DirectMessageSentImpl(
      messageIds: null == messageIds
          ? _value._messageIds
          : messageIds // ignore: cast_nullable_to_non_nullable
              as List<String>,
      recipients: null == recipients
          ? _value._recipients
          : recipients // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_DirectMessageSentImpl extends FlutterEvent_DirectMessageSent {
  const _$FlutterEvent_DirectMessageSentImpl(
      {required final List<String> messageIds, required final List<String> recipients})
      : _messageIds = messageIds,
        _recipients = recipients,
        super._();

  final List<String> _messageIds;
  @override
  List<String> get messageIds {
    if (_messageIds is EqualUnmodifiableListView) return _messageIds;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_messageIds);
  }

  final List<String> _recipients;
  @override
  List<String> get recipients {
    if (_recipients is EqualUnmodifiableListView) return _recipients;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_recipients);
  }

  @override
  String toString() {
    return 'FlutterEvent.directMessageSent(messageIds: $messageIds, recipients: $recipients)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_DirectMessageSentImpl &&
            const DeepCollectionEquality().equals(other._messageIds, _messageIds) &&
            const DeepCollectionEquality().equals(other._recipients, _recipients));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, const DeepCollectionEquality().hash(_messageIds), const DeepCollectionEquality().hash(_recipients));

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_DirectMessageSentImplCopyWith<_$FlutterEvent_DirectMessageSentImpl> get copyWith =>
      __$$FlutterEvent_DirectMessageSentImplCopyWithImpl<_$FlutterEvent_DirectMessageSentImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return directMessageSent(messageIds, recipients);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return directMessageSent?.call(messageIds, recipients);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (directMessageSent != null) {
      return directMessageSent(messageIds, recipients);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return directMessageSent(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return directMessageSent?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (directMessageSent != null) {
      return directMessageSent(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_DirectMessageSent extends FlutterEvent {
  const factory FlutterEvent_DirectMessageSent(
      {required final List<String> messageIds,
      required final List<String> recipients}) = _$FlutterEvent_DirectMessageSentImpl;
  const FlutterEvent_DirectMessageSent._() : super._();

  List<String> get messageIds;
  List<String> get recipients;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_DirectMessageSentImplCopyWith<_$FlutterEvent_DirectMessageSentImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_MessageDeletedImplCopyWith<$Res> {
  factory _$$FlutterEvent_MessageDeletedImplCopyWith(
          _$FlutterEvent_MessageDeletedImpl value, $Res Function(_$FlutterEvent_MessageDeletedImpl) then) =
      __$$FlutterEvent_MessageDeletedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String messageId, String entityId});
}

/// @nodoc
class __$$FlutterEvent_MessageDeletedImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_MessageDeletedImpl>
    implements _$$FlutterEvent_MessageDeletedImplCopyWith<$Res> {
  __$$FlutterEvent_MessageDeletedImplCopyWithImpl(
      _$FlutterEvent_MessageDeletedImpl _value, $Res Function(_$FlutterEvent_MessageDeletedImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? messageId = null,
    Object? entityId = null,
  }) {
    return _then(_$FlutterEvent_MessageDeletedImpl(
      messageId: null == messageId
          ? _value.messageId
          : messageId // ignore: cast_nullable_to_non_nullable
              as String,
      entityId: null == entityId
          ? _value.entityId
          : entityId // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_MessageDeletedImpl extends FlutterEvent_MessageDeleted {
  const _$FlutterEvent_MessageDeletedImpl({required this.messageId, required this.entityId}) : super._();

  @override
  final String messageId;
  @override
  final String entityId;

  @override
  String toString() {
    return 'FlutterEvent.messageDeleted(messageId: $messageId, entityId: $entityId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_MessageDeletedImpl &&
            (identical(other.messageId, messageId) || other.messageId == messageId) &&
            (identical(other.entityId, entityId) || other.entityId == entityId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, messageId, entityId);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_MessageDeletedImplCopyWith<_$FlutterEvent_MessageDeletedImpl> get copyWith =>
      __$$FlutterEvent_MessageDeletedImplCopyWithImpl<_$FlutterEvent_MessageDeletedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return messageDeleted(messageId, entityId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return messageDeleted?.call(messageId, entityId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (messageDeleted != null) {
      return messageDeleted(messageId, entityId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return messageDeleted(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return messageDeleted?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (messageDeleted != null) {
      return messageDeleted(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_MessageDeleted extends FlutterEvent {
  const factory FlutterEvent_MessageDeleted({required final String messageId, required final String entityId}) =
      _$FlutterEvent_MessageDeletedImpl;
  const FlutterEvent_MessageDeleted._() : super._();

  String get messageId;
  String get entityId;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_MessageDeletedImplCopyWith<_$FlutterEvent_MessageDeletedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_MessageEditedImplCopyWith<$Res> {
  factory _$$FlutterEvent_MessageEditedImplCopyWith(
          _$FlutterEvent_MessageEditedImpl value, $Res Function(_$FlutterEvent_MessageEditedImpl) then) =
      __$$FlutterEvent_MessageEditedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String messageId, String entityId, String newText, BigInt editedAt});
}

/// @nodoc
class __$$FlutterEvent_MessageEditedImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_MessageEditedImpl>
    implements _$$FlutterEvent_MessageEditedImplCopyWith<$Res> {
  __$$FlutterEvent_MessageEditedImplCopyWithImpl(
      _$FlutterEvent_MessageEditedImpl _value, $Res Function(_$FlutterEvent_MessageEditedImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? messageId = null,
    Object? entityId = null,
    Object? newText = null,
    Object? editedAt = null,
  }) {
    return _then(_$FlutterEvent_MessageEditedImpl(
      messageId: null == messageId
          ? _value.messageId
          : messageId // ignore: cast_nullable_to_non_nullable
              as String,
      entityId: null == entityId
          ? _value.entityId
          : entityId // ignore: cast_nullable_to_non_nullable
              as String,
      newText: null == newText
          ? _value.newText
          : newText // ignore: cast_nullable_to_non_nullable
              as String,
      editedAt: null == editedAt
          ? _value.editedAt
          : editedAt // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_MessageEditedImpl extends FlutterEvent_MessageEdited {
  const _$FlutterEvent_MessageEditedImpl(
      {required this.messageId, required this.entityId, required this.newText, required this.editedAt})
      : super._();

  @override
  final String messageId;
  @override
  final String entityId;
  @override
  final String newText;
  @override
  final BigInt editedAt;

  @override
  String toString() {
    return 'FlutterEvent.messageEdited(messageId: $messageId, entityId: $entityId, newText: $newText, editedAt: $editedAt)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_MessageEditedImpl &&
            (identical(other.messageId, messageId) || other.messageId == messageId) &&
            (identical(other.entityId, entityId) || other.entityId == entityId) &&
            (identical(other.newText, newText) || other.newText == newText) &&
            (identical(other.editedAt, editedAt) || other.editedAt == editedAt));
  }

  @override
  int get hashCode => Object.hash(runtimeType, messageId, entityId, newText, editedAt);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_MessageEditedImplCopyWith<_$FlutterEvent_MessageEditedImpl> get copyWith =>
      __$$FlutterEvent_MessageEditedImplCopyWithImpl<_$FlutterEvent_MessageEditedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return messageEdited(messageId, entityId, newText, editedAt);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return messageEdited?.call(messageId, entityId, newText, editedAt);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (messageEdited != null) {
      return messageEdited(messageId, entityId, newText, editedAt);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return messageEdited(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return messageEdited?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (messageEdited != null) {
      return messageEdited(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_MessageEdited extends FlutterEvent {
  const factory FlutterEvent_MessageEdited(
      {required final String messageId,
      required final String entityId,
      required final String newText,
      required final BigInt editedAt}) = _$FlutterEvent_MessageEditedImpl;
  const FlutterEvent_MessageEdited._() : super._();

  String get messageId;
  String get entityId;
  String get newText;
  BigInt get editedAt;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_MessageEditedImplCopyWith<_$FlutterEvent_MessageEditedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_ReactionAddedImplCopyWith<$Res> {
  factory _$$FlutterEvent_ReactionAddedImplCopyWith(
          _$FlutterEvent_ReactionAddedImpl value, $Res Function(_$FlutterEvent_ReactionAddedImpl) then) =
      __$$FlutterEvent_ReactionAddedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String messageId, String entityId, String emoji, String reactorId});
}

/// @nodoc
class __$$FlutterEvent_ReactionAddedImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_ReactionAddedImpl>
    implements _$$FlutterEvent_ReactionAddedImplCopyWith<$Res> {
  __$$FlutterEvent_ReactionAddedImplCopyWithImpl(
      _$FlutterEvent_ReactionAddedImpl _value, $Res Function(_$FlutterEvent_ReactionAddedImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? messageId = null,
    Object? entityId = null,
    Object? emoji = null,
    Object? reactorId = null,
  }) {
    return _then(_$FlutterEvent_ReactionAddedImpl(
      messageId: null == messageId
          ? _value.messageId
          : messageId // ignore: cast_nullable_to_non_nullable
              as String,
      entityId: null == entityId
          ? _value.entityId
          : entityId // ignore: cast_nullable_to_non_nullable
              as String,
      emoji: null == emoji
          ? _value.emoji
          : emoji // ignore: cast_nullable_to_non_nullable
              as String,
      reactorId: null == reactorId
          ? _value.reactorId
          : reactorId // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_ReactionAddedImpl extends FlutterEvent_ReactionAdded {
  const _$FlutterEvent_ReactionAddedImpl(
      {required this.messageId, required this.entityId, required this.emoji, required this.reactorId})
      : super._();

  @override
  final String messageId;
  @override
  final String entityId;
  @override
  final String emoji;
  @override
  final String reactorId;

  @override
  String toString() {
    return 'FlutterEvent.reactionAdded(messageId: $messageId, entityId: $entityId, emoji: $emoji, reactorId: $reactorId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_ReactionAddedImpl &&
            (identical(other.messageId, messageId) || other.messageId == messageId) &&
            (identical(other.entityId, entityId) || other.entityId == entityId) &&
            (identical(other.emoji, emoji) || other.emoji == emoji) &&
            (identical(other.reactorId, reactorId) || other.reactorId == reactorId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, messageId, entityId, emoji, reactorId);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_ReactionAddedImplCopyWith<_$FlutterEvent_ReactionAddedImpl> get copyWith =>
      __$$FlutterEvent_ReactionAddedImplCopyWithImpl<_$FlutterEvent_ReactionAddedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return reactionAdded(messageId, entityId, emoji, reactorId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return reactionAdded?.call(messageId, entityId, emoji, reactorId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (reactionAdded != null) {
      return reactionAdded(messageId, entityId, emoji, reactorId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return reactionAdded(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return reactionAdded?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (reactionAdded != null) {
      return reactionAdded(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_ReactionAdded extends FlutterEvent {
  const factory FlutterEvent_ReactionAdded(
      {required final String messageId,
      required final String entityId,
      required final String emoji,
      required final String reactorId}) = _$FlutterEvent_ReactionAddedImpl;
  const FlutterEvent_ReactionAdded._() : super._();

  String get messageId;
  String get entityId;
  String get emoji;
  String get reactorId;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_ReactionAddedImplCopyWith<_$FlutterEvent_ReactionAddedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_ReactionRemovedImplCopyWith<$Res> {
  factory _$$FlutterEvent_ReactionRemovedImplCopyWith(
          _$FlutterEvent_ReactionRemovedImpl value, $Res Function(_$FlutterEvent_ReactionRemovedImpl) then) =
      __$$FlutterEvent_ReactionRemovedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String messageId, String entityId, String emoji, String reactorId});
}

/// @nodoc
class __$$FlutterEvent_ReactionRemovedImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_ReactionRemovedImpl>
    implements _$$FlutterEvent_ReactionRemovedImplCopyWith<$Res> {
  __$$FlutterEvent_ReactionRemovedImplCopyWithImpl(
      _$FlutterEvent_ReactionRemovedImpl _value, $Res Function(_$FlutterEvent_ReactionRemovedImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? messageId = null,
    Object? entityId = null,
    Object? emoji = null,
    Object? reactorId = null,
  }) {
    return _then(_$FlutterEvent_ReactionRemovedImpl(
      messageId: null == messageId
          ? _value.messageId
          : messageId // ignore: cast_nullable_to_non_nullable
              as String,
      entityId: null == entityId
          ? _value.entityId
          : entityId // ignore: cast_nullable_to_non_nullable
              as String,
      emoji: null == emoji
          ? _value.emoji
          : emoji // ignore: cast_nullable_to_non_nullable
              as String,
      reactorId: null == reactorId
          ? _value.reactorId
          : reactorId // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_ReactionRemovedImpl extends FlutterEvent_ReactionRemoved {
  const _$FlutterEvent_ReactionRemovedImpl(
      {required this.messageId, required this.entityId, required this.emoji, required this.reactorId})
      : super._();

  @override
  final String messageId;
  @override
  final String entityId;
  @override
  final String emoji;
  @override
  final String reactorId;

  @override
  String toString() {
    return 'FlutterEvent.reactionRemoved(messageId: $messageId, entityId: $entityId, emoji: $emoji, reactorId: $reactorId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_ReactionRemovedImpl &&
            (identical(other.messageId, messageId) || other.messageId == messageId) &&
            (identical(other.entityId, entityId) || other.entityId == entityId) &&
            (identical(other.emoji, emoji) || other.emoji == emoji) &&
            (identical(other.reactorId, reactorId) || other.reactorId == reactorId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, messageId, entityId, emoji, reactorId);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_ReactionRemovedImplCopyWith<_$FlutterEvent_ReactionRemovedImpl> get copyWith =>
      __$$FlutterEvent_ReactionRemovedImplCopyWithImpl<_$FlutterEvent_ReactionRemovedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return reactionRemoved(messageId, entityId, emoji, reactorId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return reactionRemoved?.call(messageId, entityId, emoji, reactorId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (reactionRemoved != null) {
      return reactionRemoved(messageId, entityId, emoji, reactorId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return reactionRemoved(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return reactionRemoved?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (reactionRemoved != null) {
      return reactionRemoved(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_ReactionRemoved extends FlutterEvent {
  const factory FlutterEvent_ReactionRemoved(
      {required final String messageId,
      required final String entityId,
      required final String emoji,
      required final String reactorId}) = _$FlutterEvent_ReactionRemovedImpl;
  const FlutterEvent_ReactionRemoved._() : super._();

  String get messageId;
  String get entityId;
  String get emoji;
  String get reactorId;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_ReactionRemovedImplCopyWith<_$FlutterEvent_ReactionRemovedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_InviteCreatedImplCopyWith<$Res> {
  factory _$$FlutterEvent_InviteCreatedImplCopyWith(
          _$FlutterEvent_InviteCreatedImpl value, $Res Function(_$FlutterEvent_InviteCreatedImpl) then) =
      __$$FlutterEvent_InviteCreatedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String inviteId});
}

/// @nodoc
class __$$FlutterEvent_InviteCreatedImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_InviteCreatedImpl>
    implements _$$FlutterEvent_InviteCreatedImplCopyWith<$Res> {
  __$$FlutterEvent_InviteCreatedImplCopyWithImpl(
      _$FlutterEvent_InviteCreatedImpl _value, $Res Function(_$FlutterEvent_InviteCreatedImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? inviteId = null,
  }) {
    return _then(_$FlutterEvent_InviteCreatedImpl(
      inviteId: null == inviteId
          ? _value.inviteId
          : inviteId // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_InviteCreatedImpl extends FlutterEvent_InviteCreated {
  const _$FlutterEvent_InviteCreatedImpl({required this.inviteId}) : super._();

  @override
  final String inviteId;

  @override
  String toString() {
    return 'FlutterEvent.inviteCreated(inviteId: $inviteId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_InviteCreatedImpl &&
            (identical(other.inviteId, inviteId) || other.inviteId == inviteId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, inviteId);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_InviteCreatedImplCopyWith<_$FlutterEvent_InviteCreatedImpl> get copyWith =>
      __$$FlutterEvent_InviteCreatedImplCopyWithImpl<_$FlutterEvent_InviteCreatedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return inviteCreated(inviteId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return inviteCreated?.call(inviteId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (inviteCreated != null) {
      return inviteCreated(inviteId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return inviteCreated(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return inviteCreated?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (inviteCreated != null) {
      return inviteCreated(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_InviteCreated extends FlutterEvent {
  const factory FlutterEvent_InviteCreated({required final String inviteId}) = _$FlutterEvent_InviteCreatedImpl;
  const FlutterEvent_InviteCreated._() : super._();

  String get inviteId;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_InviteCreatedImplCopyWith<_$FlutterEvent_InviteCreatedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_InviteAcceptedImplCopyWith<$Res> {
  factory _$$FlutterEvent_InviteAcceptedImplCopyWith(
          _$FlutterEvent_InviteAcceptedImpl value, $Res Function(_$FlutterEvent_InviteAcceptedImpl) then) =
      __$$FlutterEvent_InviteAcceptedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String inviteId});
}

/// @nodoc
class __$$FlutterEvent_InviteAcceptedImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_InviteAcceptedImpl>
    implements _$$FlutterEvent_InviteAcceptedImplCopyWith<$Res> {
  __$$FlutterEvent_InviteAcceptedImplCopyWithImpl(
      _$FlutterEvent_InviteAcceptedImpl _value, $Res Function(_$FlutterEvent_InviteAcceptedImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? inviteId = null,
  }) {
    return _then(_$FlutterEvent_InviteAcceptedImpl(
      inviteId: null == inviteId
          ? _value.inviteId
          : inviteId // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_InviteAcceptedImpl extends FlutterEvent_InviteAccepted {
  const _$FlutterEvent_InviteAcceptedImpl({required this.inviteId}) : super._();

  @override
  final String inviteId;

  @override
  String toString() {
    return 'FlutterEvent.inviteAccepted(inviteId: $inviteId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_InviteAcceptedImpl &&
            (identical(other.inviteId, inviteId) || other.inviteId == inviteId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, inviteId);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_InviteAcceptedImplCopyWith<_$FlutterEvent_InviteAcceptedImpl> get copyWith =>
      __$$FlutterEvent_InviteAcceptedImplCopyWithImpl<_$FlutterEvent_InviteAcceptedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return inviteAccepted(inviteId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return inviteAccepted?.call(inviteId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (inviteAccepted != null) {
      return inviteAccepted(inviteId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return inviteAccepted(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return inviteAccepted?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (inviteAccepted != null) {
      return inviteAccepted(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_InviteAccepted extends FlutterEvent {
  const factory FlutterEvent_InviteAccepted({required final String inviteId}) = _$FlutterEvent_InviteAcceptedImpl;
  const FlutterEvent_InviteAccepted._() : super._();

  String get inviteId;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_InviteAcceptedImplCopyWith<_$FlutterEvent_InviteAcceptedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_InviteRejectedImplCopyWith<$Res> {
  factory _$$FlutterEvent_InviteRejectedImplCopyWith(
          _$FlutterEvent_InviteRejectedImpl value, $Res Function(_$FlutterEvent_InviteRejectedImpl) then) =
      __$$FlutterEvent_InviteRejectedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String inviteId});
}

/// @nodoc
class __$$FlutterEvent_InviteRejectedImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_InviteRejectedImpl>
    implements _$$FlutterEvent_InviteRejectedImplCopyWith<$Res> {
  __$$FlutterEvent_InviteRejectedImplCopyWithImpl(
      _$FlutterEvent_InviteRejectedImpl _value, $Res Function(_$FlutterEvent_InviteRejectedImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? inviteId = null,
  }) {
    return _then(_$FlutterEvent_InviteRejectedImpl(
      inviteId: null == inviteId
          ? _value.inviteId
          : inviteId // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_InviteRejectedImpl extends FlutterEvent_InviteRejected {
  const _$FlutterEvent_InviteRejectedImpl({required this.inviteId}) : super._();

  @override
  final String inviteId;

  @override
  String toString() {
    return 'FlutterEvent.inviteRejected(inviteId: $inviteId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_InviteRejectedImpl &&
            (identical(other.inviteId, inviteId) || other.inviteId == inviteId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, inviteId);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_InviteRejectedImplCopyWith<_$FlutterEvent_InviteRejectedImpl> get copyWith =>
      __$$FlutterEvent_InviteRejectedImplCopyWithImpl<_$FlutterEvent_InviteRejectedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return inviteRejected(inviteId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return inviteRejected?.call(inviteId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (inviteRejected != null) {
      return inviteRejected(inviteId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return inviteRejected(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return inviteRejected?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (inviteRejected != null) {
      return inviteRejected(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_InviteRejected extends FlutterEvent {
  const factory FlutterEvent_InviteRejected({required final String inviteId}) = _$FlutterEvent_InviteRejectedImpl;
  const FlutterEvent_InviteRejected._() : super._();

  String get inviteId;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_InviteRejectedImplCopyWith<_$FlutterEvent_InviteRejectedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_InviteRevokedImplCopyWith<$Res> {
  factory _$$FlutterEvent_InviteRevokedImplCopyWith(
          _$FlutterEvent_InviteRevokedImpl value, $Res Function(_$FlutterEvent_InviteRevokedImpl) then) =
      __$$FlutterEvent_InviteRevokedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String inviteId});
}

/// @nodoc
class __$$FlutterEvent_InviteRevokedImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_InviteRevokedImpl>
    implements _$$FlutterEvent_InviteRevokedImplCopyWith<$Res> {
  __$$FlutterEvent_InviteRevokedImplCopyWithImpl(
      _$FlutterEvent_InviteRevokedImpl _value, $Res Function(_$FlutterEvent_InviteRevokedImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? inviteId = null,
  }) {
    return _then(_$FlutterEvent_InviteRevokedImpl(
      inviteId: null == inviteId
          ? _value.inviteId
          : inviteId // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_InviteRevokedImpl extends FlutterEvent_InviteRevoked {
  const _$FlutterEvent_InviteRevokedImpl({required this.inviteId}) : super._();

  @override
  final String inviteId;

  @override
  String toString() {
    return 'FlutterEvent.inviteRevoked(inviteId: $inviteId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_InviteRevokedImpl &&
            (identical(other.inviteId, inviteId) || other.inviteId == inviteId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, inviteId);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_InviteRevokedImplCopyWith<_$FlutterEvent_InviteRevokedImpl> get copyWith =>
      __$$FlutterEvent_InviteRevokedImplCopyWithImpl<_$FlutterEvent_InviteRevokedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return inviteRevoked(inviteId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return inviteRevoked?.call(inviteId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (inviteRevoked != null) {
      return inviteRevoked(inviteId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return inviteRevoked(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return inviteRevoked?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (inviteRevoked != null) {
      return inviteRevoked(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_InviteRevoked extends FlutterEvent {
  const factory FlutterEvent_InviteRevoked({required final String inviteId}) = _$FlutterEvent_InviteRevokedImpl;
  const FlutterEvent_InviteRevoked._() : super._();

  String get inviteId;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_InviteRevokedImplCopyWith<_$FlutterEvent_InviteRevokedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_FileWrittenImplCopyWith<$Res> {
  factory _$$FlutterEvent_FileWrittenImplCopyWith(
          _$FlutterEvent_FileWrittenImpl value, $Res Function(_$FlutterEvent_FileWrittenImpl) then) =
      __$$FlutterEvent_FileWrittenImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String entityId, String path});
}

/// @nodoc
class __$$FlutterEvent_FileWrittenImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_FileWrittenImpl>
    implements _$$FlutterEvent_FileWrittenImplCopyWith<$Res> {
  __$$FlutterEvent_FileWrittenImplCopyWithImpl(
      _$FlutterEvent_FileWrittenImpl _value, $Res Function(_$FlutterEvent_FileWrittenImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? entityId = null,
    Object? path = null,
  }) {
    return _then(_$FlutterEvent_FileWrittenImpl(
      entityId: null == entityId
          ? _value.entityId
          : entityId // ignore: cast_nullable_to_non_nullable
              as String,
      path: null == path
          ? _value.path
          : path // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_FileWrittenImpl extends FlutterEvent_FileWritten {
  const _$FlutterEvent_FileWrittenImpl({required this.entityId, required this.path}) : super._();

  @override
  final String entityId;
  @override
  final String path;

  @override
  String toString() {
    return 'FlutterEvent.fileWritten(entityId: $entityId, path: $path)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_FileWrittenImpl &&
            (identical(other.entityId, entityId) || other.entityId == entityId) &&
            (identical(other.path, path) || other.path == path));
  }

  @override
  int get hashCode => Object.hash(runtimeType, entityId, path);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_FileWrittenImplCopyWith<_$FlutterEvent_FileWrittenImpl> get copyWith =>
      __$$FlutterEvent_FileWrittenImplCopyWithImpl<_$FlutterEvent_FileWrittenImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return fileWritten(entityId, path);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return fileWritten?.call(entityId, path);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (fileWritten != null) {
      return fileWritten(entityId, path);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return fileWritten(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return fileWritten?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (fileWritten != null) {
      return fileWritten(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_FileWritten extends FlutterEvent {
  const factory FlutterEvent_FileWritten({required final String entityId, required final String path}) =
      _$FlutterEvent_FileWrittenImpl;
  const FlutterEvent_FileWritten._() : super._();

  String get entityId;
  String get path;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_FileWrittenImplCopyWith<_$FlutterEvent_FileWrittenImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_FileDeletedImplCopyWith<$Res> {
  factory _$$FlutterEvent_FileDeletedImplCopyWith(
          _$FlutterEvent_FileDeletedImpl value, $Res Function(_$FlutterEvent_FileDeletedImpl) then) =
      __$$FlutterEvent_FileDeletedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String entityId, String path});
}

/// @nodoc
class __$$FlutterEvent_FileDeletedImplCopyWithImpl<$Res>
    extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_FileDeletedImpl>
    implements _$$FlutterEvent_FileDeletedImplCopyWith<$Res> {
  __$$FlutterEvent_FileDeletedImplCopyWithImpl(
      _$FlutterEvent_FileDeletedImpl _value, $Res Function(_$FlutterEvent_FileDeletedImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? entityId = null,
    Object? path = null,
  }) {
    return _then(_$FlutterEvent_FileDeletedImpl(
      entityId: null == entityId
          ? _value.entityId
          : entityId // ignore: cast_nullable_to_non_nullable
              as String,
      path: null == path
          ? _value.path
          : path // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_FileDeletedImpl extends FlutterEvent_FileDeleted {
  const _$FlutterEvent_FileDeletedImpl({required this.entityId, required this.path}) : super._();

  @override
  final String entityId;
  @override
  final String path;

  @override
  String toString() {
    return 'FlutterEvent.fileDeleted(entityId: $entityId, path: $path)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_FileDeletedImpl &&
            (identical(other.entityId, entityId) || other.entityId == entityId) &&
            (identical(other.path, path) || other.path == path));
  }

  @override
  int get hashCode => Object.hash(runtimeType, entityId, path);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_FileDeletedImplCopyWith<_$FlutterEvent_FileDeletedImpl> get copyWith =>
      __$$FlutterEvent_FileDeletedImplCopyWithImpl<_$FlutterEvent_FileDeletedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return fileDeleted(entityId, path);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return fileDeleted?.call(entityId, path);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (fileDeleted != null) {
      return fileDeleted(entityId, path);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return fileDeleted(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return fileDeleted?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (fileDeleted != null) {
      return fileDeleted(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_FileDeleted extends FlutterEvent {
  const factory FlutterEvent_FileDeleted({required final String entityId, required final String path}) =
      _$FlutterEvent_FileDeletedImpl;
  const FlutterEvent_FileDeleted._() : super._();

  String get entityId;
  String get path;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_FileDeletedImplCopyWith<_$FlutterEvent_FileDeletedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FlutterEvent_ErrorImplCopyWith<$Res> {
  factory _$$FlutterEvent_ErrorImplCopyWith(
          _$FlutterEvent_ErrorImpl value, $Res Function(_$FlutterEvent_ErrorImpl) then) =
      __$$FlutterEvent_ErrorImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String code, String message});
}

/// @nodoc
class __$$FlutterEvent_ErrorImplCopyWithImpl<$Res> extends _$FlutterEventCopyWithImpl<$Res, _$FlutterEvent_ErrorImpl>
    implements _$$FlutterEvent_ErrorImplCopyWith<$Res> {
  __$$FlutterEvent_ErrorImplCopyWithImpl(_$FlutterEvent_ErrorImpl _value, $Res Function(_$FlutterEvent_ErrorImpl) _then)
      : super(_value, _then);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? code = null,
    Object? message = null,
  }) {
    return _then(_$FlutterEvent_ErrorImpl(
      code: null == code
          ? _value.code
          : code // ignore: cast_nullable_to_non_nullable
              as String,
      message: null == message
          ? _value.message
          : message // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FlutterEvent_ErrorImpl extends FlutterEvent_Error {
  const _$FlutterEvent_ErrorImpl({required this.code, required this.message}) : super._();

  @override
  final String code;
  @override
  final String message;

  @override
  String toString() {
    return 'FlutterEvent.error(code: $code, message: $message)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FlutterEvent_ErrorImpl &&
            (identical(other.code, code) || other.code == code) &&
            (identical(other.message, message) || other.message == message));
  }

  @override
  int get hashCode => Object.hash(runtimeType, code, message);

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FlutterEvent_ErrorImplCopyWith<_$FlutterEvent_ErrorImpl> get copyWith =>
      __$$FlutterEvent_ErrorImplCopyWithImpl<_$FlutterEvent_ErrorImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String address) networkingStarted,
    required TResult Function() networkingStopped,
    required TResult Function(String peerId) peerConnected,
    required TResult Function(String peerId) peerDisconnected,
    required TResult Function(String entityId) entityCreated,
    required TResult Function(String entityId) entityUpdated,
    required TResult Function(String messageId, String entityId) messageSent,
    required TResult Function(String messageId, String entityId) messageReceived,
    required TResult Function(List<String> messageIds, List<String> recipients) directMessageSent,
    required TResult Function(String messageId, String entityId) messageDeleted,
    required TResult Function(String messageId, String entityId, String newText, BigInt editedAt) messageEdited,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionAdded,
    required TResult Function(String messageId, String entityId, String emoji, String reactorId) reactionRemoved,
    required TResult Function(String inviteId) inviteCreated,
    required TResult Function(String inviteId) inviteAccepted,
    required TResult Function(String inviteId) inviteRejected,
    required TResult Function(String inviteId) inviteRevoked,
    required TResult Function(String entityId, String path) fileWritten,
    required TResult Function(String entityId, String path) fileDeleted,
    required TResult Function(String code, String message) error,
  }) {
    return error(code, message);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String address)? networkingStarted,
    TResult? Function()? networkingStopped,
    TResult? Function(String peerId)? peerConnected,
    TResult? Function(String peerId)? peerDisconnected,
    TResult? Function(String entityId)? entityCreated,
    TResult? Function(String entityId)? entityUpdated,
    TResult? Function(String messageId, String entityId)? messageSent,
    TResult? Function(String messageId, String entityId)? messageReceived,
    TResult? Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult? Function(String messageId, String entityId)? messageDeleted,
    TResult? Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult? Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult? Function(String inviteId)? inviteCreated,
    TResult? Function(String inviteId)? inviteAccepted,
    TResult? Function(String inviteId)? inviteRejected,
    TResult? Function(String inviteId)? inviteRevoked,
    TResult? Function(String entityId, String path)? fileWritten,
    TResult? Function(String entityId, String path)? fileDeleted,
    TResult? Function(String code, String message)? error,
  }) {
    return error?.call(code, message);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String address)? networkingStarted,
    TResult Function()? networkingStopped,
    TResult Function(String peerId)? peerConnected,
    TResult Function(String peerId)? peerDisconnected,
    TResult Function(String entityId)? entityCreated,
    TResult Function(String entityId)? entityUpdated,
    TResult Function(String messageId, String entityId)? messageSent,
    TResult Function(String messageId, String entityId)? messageReceived,
    TResult Function(List<String> messageIds, List<String> recipients)? directMessageSent,
    TResult Function(String messageId, String entityId)? messageDeleted,
    TResult Function(String messageId, String entityId, String newText, BigInt editedAt)? messageEdited,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionAdded,
    TResult Function(String messageId, String entityId, String emoji, String reactorId)? reactionRemoved,
    TResult Function(String inviteId)? inviteCreated,
    TResult Function(String inviteId)? inviteAccepted,
    TResult Function(String inviteId)? inviteRejected,
    TResult Function(String inviteId)? inviteRevoked,
    TResult Function(String entityId, String path)? fileWritten,
    TResult Function(String entityId, String path)? fileDeleted,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(code, message);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FlutterEvent_NetworkingStarted value) networkingStarted,
    required TResult Function(FlutterEvent_NetworkingStopped value) networkingStopped,
    required TResult Function(FlutterEvent_PeerConnected value) peerConnected,
    required TResult Function(FlutterEvent_PeerDisconnected value) peerDisconnected,
    required TResult Function(FlutterEvent_EntityCreated value) entityCreated,
    required TResult Function(FlutterEvent_EntityUpdated value) entityUpdated,
    required TResult Function(FlutterEvent_MessageSent value) messageSent,
    required TResult Function(FlutterEvent_MessageReceived value) messageReceived,
    required TResult Function(FlutterEvent_DirectMessageSent value) directMessageSent,
    required TResult Function(FlutterEvent_MessageDeleted value) messageDeleted,
    required TResult Function(FlutterEvent_MessageEdited value) messageEdited,
    required TResult Function(FlutterEvent_ReactionAdded value) reactionAdded,
    required TResult Function(FlutterEvent_ReactionRemoved value) reactionRemoved,
    required TResult Function(FlutterEvent_InviteCreated value) inviteCreated,
    required TResult Function(FlutterEvent_InviteAccepted value) inviteAccepted,
    required TResult Function(FlutterEvent_InviteRejected value) inviteRejected,
    required TResult Function(FlutterEvent_InviteRevoked value) inviteRevoked,
    required TResult Function(FlutterEvent_FileWritten value) fileWritten,
    required TResult Function(FlutterEvent_FileDeleted value) fileDeleted,
    required TResult Function(FlutterEvent_Error value) error,
  }) {
    return error(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult? Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult? Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult? Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult? Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult? Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult? Function(FlutterEvent_MessageSent value)? messageSent,
    TResult? Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult? Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult? Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult? Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult? Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult? Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult? Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult? Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult? Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult? Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult? Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult? Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult? Function(FlutterEvent_Error value)? error,
  }) {
    return error?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FlutterEvent_NetworkingStarted value)? networkingStarted,
    TResult Function(FlutterEvent_NetworkingStopped value)? networkingStopped,
    TResult Function(FlutterEvent_PeerConnected value)? peerConnected,
    TResult Function(FlutterEvent_PeerDisconnected value)? peerDisconnected,
    TResult Function(FlutterEvent_EntityCreated value)? entityCreated,
    TResult Function(FlutterEvent_EntityUpdated value)? entityUpdated,
    TResult Function(FlutterEvent_MessageSent value)? messageSent,
    TResult Function(FlutterEvent_MessageReceived value)? messageReceived,
    TResult Function(FlutterEvent_DirectMessageSent value)? directMessageSent,
    TResult Function(FlutterEvent_MessageDeleted value)? messageDeleted,
    TResult Function(FlutterEvent_MessageEdited value)? messageEdited,
    TResult Function(FlutterEvent_ReactionAdded value)? reactionAdded,
    TResult Function(FlutterEvent_ReactionRemoved value)? reactionRemoved,
    TResult Function(FlutterEvent_InviteCreated value)? inviteCreated,
    TResult Function(FlutterEvent_InviteAccepted value)? inviteAccepted,
    TResult Function(FlutterEvent_InviteRejected value)? inviteRejected,
    TResult Function(FlutterEvent_InviteRevoked value)? inviteRevoked,
    TResult Function(FlutterEvent_FileWritten value)? fileWritten,
    TResult Function(FlutterEvent_FileDeleted value)? fileDeleted,
    TResult Function(FlutterEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(this);
    }
    return orElse();
  }
}

abstract class FlutterEvent_Error extends FlutterEvent {
  const factory FlutterEvent_Error({required final String code, required final String message}) =
      _$FlutterEvent_ErrorImpl;
  const FlutterEvent_Error._() : super._();

  String get code;
  String get message;

  /// Create a copy of FlutterEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FlutterEvent_ErrorImplCopyWith<_$FlutterEvent_ErrorImpl> get copyWith => throw _privateConstructorUsedError;
}
