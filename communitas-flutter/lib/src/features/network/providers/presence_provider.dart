import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../services/bridge_provider.dart';
import '../../../services/ffi_provider.dart';
import '../../../services/unified_data_provider.dart';

// ============================================================
// Presence & Connection Words Providers (ADR-012, ADR-013, ADR-014)
// ============================================================

/// Our connection words (ephemeral IP:port encoded as 4 words).
///
/// Share these words out-of-band for first-time connections.
/// Connection words change when your IP/port changes.
/// Different from identity words which are permanent.
///
/// Uses HTTP API to get connection words from Rust backend.
/// Falls back to null if the API is unavailable.
final connectionWordsProvider = FutureProvider<String?>((ref) async {
  final client = ref.watch(bridgeClientProvider);
  return client.getConnectionWords();
});

/// Our current presence record (ADR-014).
///
/// Contains connection_words, timestamp, and signature.
///
/// TODO: Add presence record API to Rust FFI.
final ourPresenceRecordProvider =
    FutureProvider<Map<String, dynamic>?>((ref) async {
  // TODO: When FFI exposes presence record, implement here
  return null;
});

/// Presence information model.
class PresenceInfo {
  /// Hex-encoded ML-DSA-65 public key (THE identity).
  final String pubkeyHex;

  /// User-chosen display name (shown in UI).
  final String displayName;

  final String? connectionWords;
  final bool isOnline;
  final DateTime? lastSeen;

  const PresenceInfo({
    required this.pubkeyHex,
    required this.displayName,
    this.connectionWords,
    required this.isOnline,
    this.lastSeen,
  });
}

/// Combined presence information for the current user.
final currentUserPresenceProvider = FutureProvider<PresenceInfo>((ref) async {
  final connectionWords = await ref.watch(connectionWordsProvider.future);
  final isOnline = await ref.watch(ffiNetworkStatusProvider.future);
  final identity = ref.watch(unifiedIdentityProvider);

  return PresenceInfo(
    pubkeyHex: identity.pubkeyHex,
    displayName: identity.displayName,
    connectionWords: connectionWords,
    isOnline: isOnline,
    lastSeen: isOnline ? DateTime.now() : null,
  );
});

/// Peer presence record model (ADR-014).
class PeerPresenceRecord {
  final String pubkeyHex;
  final String connectionWords;
  final DateTime timestamp;
  final bool isVerified;

  const PeerPresenceRecord({
    required this.pubkeyHex,
    required this.connectionWords,
    required this.timestamp,
    required this.isVerified,
  });

  factory PeerPresenceRecord.fromJson(Map<String, dynamic> json) {
    return PeerPresenceRecord(
      pubkeyHex: json['pubkey_hex'] as String? ?? '',
      connectionWords: json['connection_words'] as String? ?? '',
      timestamp: json['timestamp'] != null
          ? DateTime.fromMillisecondsSinceEpoch(
              (json['timestamp'] as num).toInt() * 1000)
          : DateTime.now(),
      isVerified: json['is_verified'] as bool? ?? false,
    );
  }
}

/// Query peer presence by pubkey.
///
/// TODO: Add peer presence query to Rust FFI.
final peerPresenceProvider =
    FutureProvider.family<PeerPresenceRecord?, String>((ref, pubkeyHex) async {
  // TODO: When FFI exposes peer presence query, implement here
  return null;
});

// ============================================================
// Presence Action Notifiers
// ============================================================

/// Notifier for presence-related actions.
class PresenceController extends StateNotifier<AsyncValue<void>> {
  final Ref _ref;

  PresenceController(this._ref) : super(const AsyncValue.data(null));

  /// Announce our presence to the network.
  ///
  /// TODO: Add presence announcement to Rust FFI.
  Future<bool> announcePresence() async {
    state = const AsyncValue.loading();
    try {
      // TODO: When FFI exposes presence announcement, implement here
      // For now, just refresh network info
      _ref.invalidate(ffiNetworkInfoProvider);
      state = const AsyncValue.data(null);
      return true;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return false;
    }
  }

  /// Connect to a peer using their connection words.
  ///
  /// Connection words are ephemeral (IP:port encoded as 4 words).
  /// After connecting, you'll receive their identity packet.
  Future<bool> connectWithConnectionWords(String connectionWords) async {
    state = const AsyncValue.loading();
    try {
      final controller = _ref.read(ffiNetworkControllerProvider.notifier);
      await controller.connectToPeer(connectionWords);

      // If we get here without throwing, connection was successful
      state = const AsyncValue.data(null);
      // Refresh network info after connecting
      _ref.invalidate(ffiNetworkInfoProvider);

      return true;
    } catch (e, st) {
      state = AsyncValue.error(e, st);
      return false;
    }
  }
}

final presenceControllerProvider =
    StateNotifierProvider<PresenceController, AsyncValue<void>>((ref) {
  return PresenceController(ref);
});

// ============================================================
// Connected Peers with Presence
// ============================================================

/// Peer information with presence data.
class PeerInfo {
  /// Hex-encoded ML-DSA-65 public key (THE identity).
  final String pubkeyHex;

  /// User-chosen display name (shown in UI).
  final String? displayName;
  final String connectionType;
  final String? latency;
  final bool isOnline;

  const PeerInfo({
    required this.pubkeyHex,
    this.displayName,
    required this.connectionType,
    this.latency,
    required this.isOnline,
  });

  factory PeerInfo.fromJson(Map<String, dynamic> json) {
    return PeerInfo(
      pubkeyHex: json['pubkey_hex'] as String? ?? json['four_words'] as String? ?? '',
      displayName: json['display_name'] as String?,
      connectionType: json['connection_type'] as String? ?? 'direct',
      latency: json['latency'] as String?,
      isOnline: json['is_online'] as bool? ?? true,
    );
  }
}

/// Provider for connected peers with full info.
///
/// TODO: Add peer list API to Rust FFI.
/// Currently returns empty list; use ffiPeerCountProvider for count.
final connectedPeersProvider = FutureProvider<List<PeerInfo>>((ref) async {
  // TODO: When FFI exposes peer list, implement here
  // For now, we can only get peer count from network info
  return [];
});

// ============================================================
// Network Status
// ============================================================

/// Network status information.
class NetworkStatus {
  final bool isActive;
  final int peerCount;
  final int bootstrapNodesConnected;
  final String? externalAddress;
  final String? connectionWords;
  final String? natType;

  const NetworkStatus({
    required this.isActive,
    required this.peerCount,
    required this.bootstrapNodesConnected,
    this.externalAddress,
    this.connectionWords,
    this.natType,
  });
}

/// Combined network status provider.
final networkStatusProvider = FutureProvider<NetworkStatus>((ref) async {
  final networkInfo = await ref.watch(ffiNetworkInfoProvider.future);
  final connectionWords = await ref.watch(connectionWordsProvider.future);

  return NetworkStatus(
    isActive: networkInfo?.isActive ?? false,
    peerCount: networkInfo?.peerCount ?? 0,
    bootstrapNodesConnected: networkInfo?.bootstrapConnected == true ? 1 : 0,
    externalAddress: networkInfo?.externalAddress,
    connectionWords: connectionWords,
    natType: null, // TODO: Add natType to FlutterNetworkInfo
  );
});
