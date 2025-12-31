import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/colors.dart';
import '../../../shared/widgets/sidebar.dart';
import '../../../shared/widgets/adaptive_layout.dart';

/// Network status panel showing P2P connectivity.
class NetworkPanelScreen extends ConsumerWidget {
  const NetworkPanelScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return AdaptiveLayout(
      sidebar: const Sidebar(),
      body: Scaffold(
        appBar: AppBar(
          title: const Text('Network'),
          actions: [
            IconButton(
              icon: const Icon(Icons.refresh),
              onPressed: () {},
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
              _buildStatusCard(context),
              const SizedBox(height: 24),

              // Bootstrap nodes
              Text(
                'Bootstrap Nodes',
                style: Theme.of(context).textTheme.titleLarge,
              ),
              const SizedBox(height: 16),
              _buildBootstrapNodes(),
              const SizedBox(height: 24),

              // Connected peers
              Text(
                'Connected Peers',
                style: Theme.of(context).textTheme.titleLarge,
              ),
              const SizedBox(height: 16),
              _buildPeerList(),
              const SizedBox(height: 24),

              // Network stats
              Text(
                'Statistics',
                style: Theme.of(context).textTheme.titleLarge,
              ),
              const SizedBox(height: 16),
              _buildStats(),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildStatusCard(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          colors: [
            CommunitasColors.online.withOpacity(0.2),
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
              color: CommunitasColors.online.withOpacity(0.2),
              borderRadius: BorderRadius.circular(32),
            ),
            child: const Icon(
              Icons.wifi,
              color: CommunitasColors.online,
              size: 32,
            ),
          ),
          const SizedBox(width: 24),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Connected',
                  style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                        color: CommunitasColors.online,
                      ),
                ),
                const SizedBox(height: 4),
                Text(
                  '5 peers • 3 bootstrap nodes',
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
                'Symmetric',
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

  Widget _buildBootstrapNodes() {
    final nodes = [
      _NodeInfo(
        name: 'saorsa-2.saorsalabs.com',
        address: '142.93.199.50:11000',
        status: 'connected',
        region: 'NYC1',
      ),
      _NodeInfo(
        name: 'saorsa-3.saorsalabs.com',
        address: '147.182.234.192:11000',
        status: 'connected',
        region: 'SFO3',
      ),
      _NodeInfo(
        name: 'saorsa-4.saorsalabs.com',
        address: '206.189.7.117:11000',
        status: 'standby',
        region: 'AMS3',
      ),
    ];

    return Column(
      children: nodes.map((node) => _buildNodeTile(node)).toList(),
    );
  }

  Widget _buildNodeTile(_NodeInfo node) {
    final isConnected = node.status == 'connected';

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
            width: 10,
            height: 10,
            decoration: BoxDecoration(
              color: isConnected
                  ? CommunitasColors.online
                  : CommunitasColors.offline,
              shape: BoxShape.circle,
            ),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  node.name,
                  style: const TextStyle(fontWeight: FontWeight.w500),
                ),
                Text(
                  node.address,
                  style: TextStyle(
                    fontSize: 12,
                    color: CommunitasColors.jade,
                  ),
                ),
              ],
            ),
          ),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: CommunitasColors.fern,
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              node.region,
              style: const TextStyle(fontSize: 11),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildPeerList() {
    final peers = [
      _PeerInfo(
        fourWords: 'river-mountain-sun-cloud',
        displayName: 'Alice',
        latency: '42ms',
        status: 'direct',
      ),
      _PeerInfo(
        fourWords: 'wind-valley-tree-stone',
        displayName: 'Bob',
        latency: '128ms',
        status: 'relayed',
      ),
    ];

    if (peers.isEmpty) {
      return Container(
        padding: const EdgeInsets.all(32),
        decoration: BoxDecoration(
          color: CommunitasColors.moss,
          borderRadius: BorderRadius.circular(12),
        ),
        child: Center(
          child: Text(
            'No peers connected',
            style: TextStyle(
              color: CommunitasColors.cream.withOpacity(0.5),
            ),
          ),
        ),
      );
    }

    return Column(
      children: peers.map((peer) => _buildPeerTile(peer)).toList(),
    );
  }

  Widget _buildPeerTile(_PeerInfo peer) {
    final isDirect = peer.status == 'direct';

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
                peer.displayName[0].toUpperCase(),
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
                  peer.displayName,
                  style: const TextStyle(fontWeight: FontWeight.w500),
                ),
                Text(
                  peer.fourWords,
                  style: TextStyle(
                    fontSize: 12,
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
                  peer.status,
                  style: TextStyle(
                    fontSize: 11,
                    color: isDirect
                        ? CommunitasColors.online
                        : CommunitasColors.warning,
                  ),
                ),
              ),
              const SizedBox(height: 4),
              Text(
                peer.latency,
                style: TextStyle(
                  fontSize: 12,
                  color: CommunitasColors.cream.withOpacity(0.5),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildStats() {
    return Wrap(
      spacing: 16,
      runSpacing: 16,
      children: [
        _buildStatCard('Messages Sent', '1,247'),
        _buildStatCard('Messages Received', '3,891'),
        _buildStatCard('Data Uploaded', '24.5 MB'),
        _buildStatCard('Data Downloaded', '128.3 MB'),
        _buildStatCard('Uptime', '4h 32m'),
        _buildStatCard('Reconnections', '2'),
      ],
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

class _NodeInfo {
  final String name;
  final String address;
  final String status;
  final String region;

  _NodeInfo({
    required this.name,
    required this.address,
    required this.status,
    required this.region,
  });
}

class _PeerInfo {
  final String fourWords;
  final String displayName;
  final String latency;
  final String status;

  _PeerInfo({
    required this.fourWords,
    required this.displayName,
    required this.latency,
    required this.status,
  });
}
