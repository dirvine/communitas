import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../bindings/api_exports.dart';
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
  final api = ref.watch(communitasApiProvider);
  if (api == null) return null;
  try {
    return await api.gossipGetConnectionWords();
  } catch (e) {
    return null;
  }
});

/// Our current presence record (ADR-014).
///
/// Contains connection_words, timestamp, and signature.
///
final ourPresenceRecordProvider =
    FutureProvider<Map<String, dynamic>?>((ref) async {
  final api = ref.watch(communitasApiProvider);
  if (api == null) return null;
  try {
    final record = await api.presenceGetOurRecord();
    if (record == null) return null;
    return {
      'pubkey_hex': record.pubkeyHex,
      'connection_words': record.connectionWords,
      'timestamp': record.timestamp,
      'is_verified': record.isVerified,
    };
  } catch (e) {
    return null;
  }
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
final peerPresenceProvider =
    FutureProvider.family<PeerPresenceRecord?, String>((ref, pubkeyHex) async {
  final api = ref.watch(communitasApiProvider);
  if (api == null) return null;
  try {
    final record = await api.presenceQueryPeer(pubkeyHex: pubkeyHex);
    if (record == null) return null;
    return PeerPresenceRecord(
      pubkeyHex: record.pubkeyHex,
      connectionWords: record.connectionWords,
      timestamp: DateTime.fromMillisecondsSinceEpoch(
        record.timestamp.toInt() * 1000,
      ),
      isVerified: record.isVerified,
    );
  } catch (e) {
    return null;
  }
});

// ============================================================
// Presence Action Notifiers
// ============================================================

/// Notifier for presence-related actions.
class PresenceController extends StateNotifier<AsyncValue<void>> {
  final Ref _ref;

  PresenceController(this._ref) : super(const AsyncValue.data(null));

  /// Announce our presence to the network.
  Future<bool> announcePresence() async {
    state = const AsyncValue.loading();
    try {
      final api = _ref.read(communitasApiProvider);
      if (api == null) {
        state = AsyncValue.error('Not authenticated', StackTrace.current);
        return false;
      }

      await api.presenceAnnounce();
      _ref.invalidate(ourPresenceRecordProvider);
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
final connectedPeersProvider = FutureProvider<List<PeerInfo>>((ref) async {
  final api = ref.watch(communitasApiProvider);
  if (api == null) return [];

  try {
    final peers = await api.presenceListOnlinePeers();
    if (peers.isEmpty) return [];

    final contacts = await ref.watch(unifiedContactsProvider.future);
    return peers.map((pubkeyHex) {
      UnifiedContact? contact;
      for (final candidate in contacts) {
        if (candidate.pubkeyHex == pubkeyHex) {
          contact = candidate;
          break;
        }
      }

      return PeerInfo(
        pubkeyHex: pubkeyHex,
        displayName: contact?.displayName,
        connectionType: 'gossip',
        latency: null,
        isOnline: true,
      );
    }).toList();
  } catch (e) {
    return [];
  }
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
    natType: null,
  );
});
