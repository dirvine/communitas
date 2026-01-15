import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/colors.dart';
import '../../../shared/widgets/sidebar.dart';
import '../../../shared/widgets/adaptive_layout.dart';
import '../providers/presence_provider.dart';

/// Network status panel showing P2P connectivity, presence, and connection words.
///
/// This screen implements the identity/presence system from ADR-012, ADR-013, ADR-014:
/// - Identity Words: Permanent WHO you are (derived from ML-DSA-65 pubkey)
/// - Connection Words: Ephemeral WHERE you are (IP:port encoded as 4 words)
/// - Presence: Network-wide peer discovery via signed records
class NetworkPanelScreen extends ConsumerStatefulWidget {
  const NetworkPanelScreen({super.key});

  @override
  ConsumerState<NetworkPanelScreen> createState() => _NetworkPanelScreenState();
}

class _NetworkPanelScreenState extends ConsumerState<NetworkPanelScreen> {
  final TextEditingController _connectionWordsController =
      TextEditingController();
  bool _isConnecting = false;

  @override
  void dispose() {
    _connectionWordsController.dispose();
    super.dispose();
  }

  Future<void> _connectToPeer() async {
    final words = _connectionWordsController.text.trim();
    if (words.isEmpty) return;

    setState(() => _isConnecting = true);

    final controller = ref.read(presenceControllerProvider.notifier);
    final success = await controller.connectWithConnectionWords(words);

    setState(() => _isConnecting = false);

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(success
              ? 'Connected to peer!'
              : 'Failed to connect. Check the connection words.'),
          backgroundColor:
              success ? CommunitasColors.online : CommunitasColors.error,
        ),
      );
      if (success) {
        _connectionWordsController.clear();
      }
    }
  }

  void _copyToClipboard(String text, String label) {
    Clipboard.setData(ClipboardData(text: text));
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('$label copied to clipboard'),
        duration: const Duration(seconds: 2),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final networkStatus = ref.watch(networkStatusProvider);
    final currentPresence = ref.watch(currentUserPresenceProvider);
    final connectedPeers = ref.watch(connectedPeersProvider);

    return AdaptiveLayout(
      sidebar: const Sidebar(),
      body: Scaffold(
        appBar: AppBar(
          title: const Text('Network'),
          actions: [
            IconButton(
              icon: const Icon(Icons.refresh),
              onPressed: () {
                ref.invalidate(networkStatusProvider);
                ref.invalidate(currentUserPresenceProvider);
                ref.invalidate(connectedPeersProvider);
              },
              tooltip: 'Refresh',
            ),
          ],
        ),
        body: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Connection status card
              _buildStatusCard(context, networkStatus, currentPresence),
              const SizedBox(height: 24),

              // Your Connection Words (shareable)
              _buildConnectionWordsCard(context, currentPresence),
              const SizedBox(height: 24),

              // Connect to Peer
              Text(
                'Connect to Peer',
                style: Theme.of(context).textTheme.titleLarge,
              ),
              const SizedBox(height: 16),
              _buildConnectCard(context),
              const SizedBox(height: 24),

              // Connected peers
              Text(
                'Connected Peers',
                style: Theme.of(context).textTheme.titleLarge,
              ),
              const SizedBox(height: 16),
              _buildPeerList(connectedPeers),
              const SizedBox(height: 24),

              // Network stats
              Text(
                'Statistics',
                style: Theme.of(context).textTheme.titleLarge,
              ),
              const SizedBox(height: 16),
              _buildStats(networkStatus),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildStatusCard(
    BuildContext context,
    AsyncValue<NetworkStatus> networkStatusAsync,
    AsyncValue<PresenceInfo> presenceAsync,
  ) {
    return networkStatusAsync.when(
      loading: () => _buildStatusCardContent(
        context,
        isOnline: false,
        peerCount: 0,
        bootstrapCount: 0,
        natType: 'Checking...',
      ),
      error: (e, _) => _buildStatusCardContent(
        context,
        isOnline: false,
        peerCount: 0,
        bootstrapCount: 0,
        natType: 'Error',
      ),
      data: (status) => _buildStatusCardContent(
        context,
        isOnline: status.isActive,
        peerCount: status.peerCount,
        bootstrapCount: status.bootstrapNodesConnected,
        natType: status.natType ?? 'Unknown',
      ),
    );
  }

  Widget _buildStatusCardContent(
    BuildContext context, {
    required bool isOnline,
    required int peerCount,
    required int bootstrapCount,
    required String natType,
  }) {
    return Container(
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          colors: [
            (isOnline ? CommunitasColors.online : CommunitasColors.offline)
                .withOpacity(0.2),
            CommunitasColors.moss,
          ],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ),
        borderRadius: BorderRadius.circular(16),
      ),
      child: Row(
        children: [
          Container(
            width: 64,
            height: 64,
            decoration: BoxDecoration(
              color: (isOnline
                      ? CommunitasColors.online
                      : CommunitasColors.offline)
                  .withOpacity(0.2),
              borderRadius: BorderRadius.circular(32),
            ),
            child: Icon(
              isOnline ? Icons.wifi : Icons.wifi_off,
              color:
                  isOnline ? CommunitasColors.online : CommunitasColors.offline,
              size: 32,
            ),
          ),
          const SizedBox(width: 24),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  isOnline ? 'Connected' : 'Offline',
                  style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                        color: isOnline
                            ? CommunitasColors.online
                            : CommunitasColors.offline,
                      ),
                ),
                const SizedBox(height: 4),
                Text(
                  '$peerCount peers \u2022 $bootstrapCount bootstrap nodes',
                  style: TextStyle(
                    color: CommunitasColors.cream.withOpacity(0.7),
                  ),
                ),
              ],
            ),
          ),
          Column(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              const Text('NAT Type'),
              Text(
                natType,
                style: TextStyle(
                  color: CommunitasColors.jade,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildConnectionWordsCard(
    BuildContext context,
    AsyncValue<PresenceInfo> presenceAsync,
  ) {
    return Container(
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        color: CommunitasColors.moss,
        borderRadius: BorderRadius.circular(16),
        border: Border.all(
          color: CommunitasColors.jade.withOpacity(0.3),
          width: 1,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(
                Icons.share_location,
                color: CommunitasColors.jade,
              ),
              const SizedBox(width: 12),
              Text(
                'Your Connection Words',
                style: Theme.of(context).textTheme.titleMedium?.copyWith(
                      color: CommunitasColors.jade,
                    ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            'Share these 4 words for others to connect to you. They encode your current IP:port.',
            style: TextStyle(
              fontSize: 12,
              color: CommunitasColors.cream.withOpacity(0.6),
            ),
          ),
          const SizedBox(height: 16),
          presenceAsync.when(
            loading: () => const Center(child: CircularProgressIndicator()),
            error: (e, _) => Text(
              'Unable to get connection words',
              style: TextStyle(color: CommunitasColors.error),
            ),
            data: (presence) {
              final words = presence.connectionWords;
              if (words == null || words.isEmpty) {
                return Container(
                  padding: const EdgeInsets.all(16),
                  decoration: BoxDecoration(
                    color: CommunitasColors.fern,
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Row(
                    children: [
                      Icon(Icons.info_outline,
                          color: CommunitasColors.cream.withOpacity(0.7)),
                      const SizedBox(width: 12),
                      Expanded(
                        child: Text(
                          'Start networking to get your connection words',
                          style: TextStyle(
                            color: CommunitasColors.cream.withOpacity(0.7),
                          ),
                        ),
                      ),
                    ],
                  ),
                );
              }
              return Row(
                children: [
                  Expanded(
                    child: Container(
                      padding: const EdgeInsets.symmetric(
                          horizontal: 20, vertical: 16),
                      decoration: BoxDecoration(
                        color: CommunitasColors.fern,
                        borderRadius: BorderRadius.circular(8),
                      ),
                      child: Text(
                        words,
                        style: const TextStyle(
                          fontSize: 20,
                          fontWeight: FontWeight.w600,
                          letterSpacing: 1.5,
                          fontFamily: 'monospace',
                        ),
                      ),
                    ),
                  ),
                  const SizedBox(width: 12),
                  IconButton(
                    onPressed: () =>
                        _copyToClipboard(words, 'Connection words'),
                    icon: const Icon(Icons.copy),
                    tooltip: 'Copy to clipboard',
                    style: IconButton.styleFrom(
                      backgroundColor: CommunitasColors.jade,
                      foregroundColor: CommunitasColors.cream,
                    ),
                  ),
                ],
              );
            },
          ),
          const SizedBox(height: 16),
          Divider(color: CommunitasColors.cream.withOpacity(0.1)),
          const SizedBox(height: 12),
          presenceAsync.when(
            loading: () => const SizedBox.shrink(),
            error: (_, __) => const SizedBox.shrink(),
            data: (presence) => Row(
              children: [
                Icon(
                  Icons.fingerprint,
                  size: 16,
                  color: CommunitasColors.cream.withOpacity(0.5),
                ),
                const SizedBox(width: 8),
                Text(
                  'Identity: ',
                  style: TextStyle(
                    fontSize: 12,
                    color: CommunitasColors.cream.withOpacity(0.5),
                  ),
                ),
                Text(
                  presence.identityWords,
                  style: TextStyle(
                    fontSize: 12,
                    fontFamily: 'monospace',
                    color: CommunitasColors.cream.withOpacity(0.7),
                  ),
                ),
                const Spacer(),
                Tooltip(
                  message:
                      'Your permanent identity (WHO you are).\nDifferent from connection words (WHERE you are).',
                  child: Icon(
                    Icons.help_outline,
                    size: 16,
                    color: CommunitasColors.cream.withOpacity(0.5),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildConnectCard(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        color: CommunitasColors.moss,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Enter the connection words shared by another user:',
            style: TextStyle(
              fontSize: 14,
              color: CommunitasColors.cream.withOpacity(0.7),
            ),
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _connectionWordsController,
                  decoration: InputDecoration(
                    hintText: 'e.g., ocean-forest-moon-star',
                    hintStyle: TextStyle(
                      color: CommunitasColors.cream.withOpacity(0.3),
                    ),
                    filled: true,
                    fillColor: CommunitasColors.fern,
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(8),
                      borderSide: BorderSide.none,
                    ),
                    contentPadding: const EdgeInsets.symmetric(
                      horizontal: 16,
                      vertical: 14,
                    ),
                  ),
                  style: const TextStyle(
                    fontFamily: 'monospace',
                    fontSize: 16,
                  ),
                  onSubmitted: (_) => _connectToPeer(),
                ),
              ),
              const SizedBox(width: 12),
              SizedBox(
                height: 48,
                child: ElevatedButton.icon(
                  onPressed: _isConnecting ? null : _connectToPeer,
                  icon: _isConnecting
                      ? const SizedBox(
                          width: 16,
                          height: 16,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.link),
                  label: Text(_isConnecting ? 'Connecting...' : 'Connect'),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: CommunitasColors.jade,
                    foregroundColor: CommunitasColors.cream,
                    padding: const EdgeInsets.symmetric(horizontal: 20),
                  ),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildPeerList(AsyncValue<List<PeerInfo>> peersAsync) {
    return peersAsync.when(
      loading: () => Container(
        padding: const EdgeInsets.all(32),
        decoration: BoxDecoration(
          color: CommunitasColors.moss,
          borderRadius: BorderRadius.circular(12),
        ),
        child: const Center(child: CircularProgressIndicator()),
      ),
      error: (e, _) => Container(
        padding: const EdgeInsets.all(32),
        decoration: BoxDecoration(
          color: CommunitasColors.moss,
          borderRadius: BorderRadius.circular(12),
        ),
        child: Center(
          child: Text(
            'Error loading peers',
            style: TextStyle(
              color: CommunitasColors.error,
            ),
          ),
        ),
      ),
      data: (peers) {
        if (peers.isEmpty) {
          return Container(
            padding: const EdgeInsets.all(32),
            decoration: BoxDecoration(
              color: CommunitasColors.moss,
              borderRadius: BorderRadius.circular(12),
            ),
            child: Center(
              child: Column(
                children: [
                  Icon(
                    Icons.people_outline,
                    size: 48,
                    color: CommunitasColors.cream.withOpacity(0.3),
                  ),
                  const SizedBox(height: 12),
                  Text(
                    'No peers connected',
                    style: TextStyle(
                      color: CommunitasColors.cream.withOpacity(0.5),
                    ),
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Share your connection words to invite others',
                    style: TextStyle(
                      fontSize: 12,
                      color: CommunitasColors.cream.withOpacity(0.3),
                    ),
                  ),
                ],
              ),
            ),
          );
        }

        return Column(
          children: peers.map((peer) => _buildPeerTile(peer)).toList(),
        );
      },
    );
  }

  Widget _buildPeerTile(PeerInfo peer) {
    final isDirect = peer.connectionType == 'direct';

    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: CommunitasColors.moss,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        children: [
          Container(
            width: 40,
            height: 40,
            decoration: BoxDecoration(
              color: CommunitasColors.jade,
              borderRadius: BorderRadius.circular(20),
            ),
            child: Center(
              child: Text(
                (peer.displayName ?? peer.fourWords)[0].toUpperCase(),
                style: const TextStyle(
                  fontWeight: FontWeight.bold,
                  color: CommunitasColors.cream,
                ),
              ),
            ),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  peer.displayName ?? peer.fourWords,
                  style: const TextStyle(fontWeight: FontWeight.w500),
                ),
                Text(
                  peer.fourWords,
                  style: TextStyle(
                    fontSize: 12,
                    fontFamily: 'monospace',
                    color: CommunitasColors.jade,
                  ),
                ),
              ],
            ),
          ),
          Column(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                decoration: BoxDecoration(
                  color: isDirect
                      ? CommunitasColors.online.withOpacity(0.2)
                      : CommunitasColors.warning.withOpacity(0.2),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  peer.connectionType,
                  style: TextStyle(
                    fontSize: 11,
                    color: isDirect
                        ? CommunitasColors.online
                        : CommunitasColors.warning,
                  ),
                ),
              ),
              if (peer.latency != null) ...[
                const SizedBox(height: 4),
                Text(
                  peer.latency!,
                  style: TextStyle(
                    fontSize: 12,
                    color: CommunitasColors.cream.withOpacity(0.5),
                  ),
                ),
              ],
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildStats(AsyncValue<NetworkStatus> networkStatusAsync) {
    return networkStatusAsync.when(
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (e, _) => Text('Error: $e'),
      data: (status) => Wrap(
        spacing: 16,
        runSpacing: 16,
        children: [
          _buildStatCard('Peers Connected', '${status.peerCount}'),
          _buildStatCard('Bootstrap Nodes', '${status.bootstrapNodesConnected}'),
          _buildStatCard(
              'Network Status', status.isActive ? 'Active' : 'Inactive'),
          if (status.externalAddress != null)
            _buildStatCard('External Address', status.externalAddress!),
        ],
      ),
    );
  }

  Widget _buildStatCard(String label, String value) {
    return Container(
      width: 150,
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: CommunitasColors.moss,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            value,
            style: const TextStyle(
              fontSize: 24,
              fontWeight: FontWeight.bold,
              color: CommunitasColors.jade,
            ),
          ),
          const SizedBox(height: 4),
          Text(
            label,
            style: TextStyle(
              fontSize: 12,
              color: CommunitasColors.cream.withOpacity(0.7),
            ),
          ),
        ],
      ),
    );
  }
}
