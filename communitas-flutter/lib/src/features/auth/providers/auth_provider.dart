import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';

import '../../../../main.dart';
import '../../../bindings/api_exports.dart';
import '../../../demo/demo_data.dart';
import '../../../services/bridge_client.dart';

/// Whether to use bridge mode for web.
/// Bridge mode connects to a Communitas bridge server via HTTP.
const bool kBridgeMode = kIsWeb && !kDemoMode;

/// Authentication state for Communitas.
class AuthState {
  final bool isAuthenticated;

  /// Hex-encoded ML-DSA-65 public key (THE identity).
  final String? pubkeyHex;

  /// Legacy: Four words used for vault storage filename.
  /// TODO: Migrate vault storage to use pubkeyHex.
  final String? fourWords;

  /// User-chosen display name (shown in UI).
  final String? displayName;
  final List<VaultInfo> availableVaults;
  final VaultInfo? currentVault;
  final CommunitasApi? api;

  const AuthState({
    this.isAuthenticated = false,
    this.pubkeyHex,
    this.fourWords,
    this.displayName,
    this.availableVaults = const [],
    this.currentVault,
    this.api,
  });

  AuthState copyWith({
    bool? isAuthenticated,
    String? pubkeyHex,
    String? fourWords,
    String? displayName,
    List<VaultInfo>? availableVaults,
    VaultInfo? currentVault,
    CommunitasApi? api,
  }) {
    return AuthState(
      isAuthenticated: isAuthenticated ?? this.isAuthenticated,
      pubkeyHex: pubkeyHex ?? this.pubkeyHex,
      fourWords: fourWords ?? this.fourWords,
      displayName: displayName ?? this.displayName,
      availableVaults: availableVaults ?? this.availableVaults,
      currentVault: currentVault ?? this.currentVault,
      api: api ?? this.api,
    );
  }
}

/// Vault information for identity storage.
class VaultInfo {
  final String id;
  final String fourWords;
  final String displayName;
  final DateTime createdAt;
  final DateTime lastUsed;

  const VaultInfo({
    required this.id,
    required this.fourWords,
    required this.displayName,
    required this.createdAt,
    required this.lastUsed,
  });
}

/// Authentication state notifier.
class AuthNotifier extends StateNotifier<AuthState> {
  AuthNotifier() : super(const AuthState()) {
    _initializeAuth();
  }

  Future<void> _initializeAuth() async {
    // Bridge mode: Connect to bridge server (web with real backend)
    if (kBridgeMode) {
      await _initializeFromBridge();
      return;
    }

    // Demo mode: Auto-login with demo identity
    if (kDemoMode) {
      await Future.delayed(const Duration(milliseconds: 500));
      state = AuthState(
        isAuthenticated: true,
        fourWords: DemoData.demoIdentity.fourWords,
        displayName: DemoData.demoIdentity.displayName,
        availableVaults: [
          VaultInfo(
            id: 'demo-vault',
            fourWords: DemoData.demoIdentity.fourWords,
            displayName: DemoData.demoIdentity.displayName,
            createdAt: DateTime.now().subtract(const Duration(days: 30)),
            lastUsed: DateTime.now(),
          ),
        ],
        currentVault: VaultInfo(
          id: 'demo-vault',
          fourWords: DemoData.demoIdentity.fourWords,
          displayName: DemoData.demoIdentity.displayName,
          createdAt: DateTime.now().subtract(const Duration(days: 30)),
          lastUsed: DateTime.now(),
        ),
      );
      return;
    }

    // Native mode: Try to list existing vaults
    try {
      await _loadVaults();
    } catch (e) {
      debugPrint('Error loading vaults: $e');
    }
  }

  /// Initialize from bridge server (web mode).
  Future<void> _initializeFromBridge() async {
    try {
      // Determine bridge URL from current location
      final bridgeUrl = _getBridgeUrl();
      final client = BridgeClient(baseUrl: bridgeUrl);

      // Check if bridge is available and has a session
      final status = await client.checkStatus();
      if (!status) {
        debugPrint('Bridge not available at $bridgeUrl');
        return;
      }

      // Get current session info from bridge
      final sessionInfo = await client.getSessionInfo();
      if (sessionInfo != null) {
        final fourWords = sessionInfo['four_words'] as String?;
        final displayName = sessionInfo['display_name'] as String?;

        if (fourWords != null) {
          state = AuthState(
            isAuthenticated: true,
            fourWords: fourWords,
            displayName: displayName ?? fourWords,
            availableVaults: [
              VaultInfo(
                id: 'bridge-session',
                fourWords: fourWords,
                displayName: displayName ?? fourWords,
                createdAt: DateTime.now(),
                lastUsed: DateTime.now(),
              ),
            ],
            currentVault: VaultInfo(
              id: 'bridge-session',
              fourWords: fourWords,
              displayName: displayName ?? fourWords,
              createdAt: DateTime.now(),
              lastUsed: DateTime.now(),
            ),
          );
          debugPrint('Authenticated via bridge: $fourWords');
          return;
        }
      }

      // Bridge available but no session - user needs to login
      debugPrint('Bridge available but no session');
    } catch (e) {
      debugPrint('Bridge initialization failed: $e');
    }
  }

  /// Get bridge URL for web mode.
  String _getBridgeUrl() {
    // Check for environment variable override
    const envUrl = String.fromEnvironment('BRIDGE_URL', defaultValue: '');
    if (envUrl.isNotEmpty) return envUrl;

    // On web, derive from current origin
    if (kIsWeb) {
      // Replace web server port with bridge port (3030)
      final uri = Uri.base;
      final host = uri.host;
      return '${uri.scheme}://$host:3030';
    }

    // Fallback for local development
    return 'http://localhost:3030';
  }

  /// Get storage path for vaults
  Future<String> _getStoragePath() async {
    final appDir = await getApplicationSupportDirectory();
    return appDir.path;
  }

  /// Load existing vaults from storage
  Future<void> _loadVaults() async {
    if (kDemoMode) return;

    // For native mode, we need to create a temporary API to list vaults
    // This is a chicken-and-egg problem - we need an API to list vaults
    // but need vault info to create API. We work around by checking the vault directory.
    try {
      final storagePath = await _getStoragePath();
      final vaultDir = Directory('$storagePath/vaults');

      if (await vaultDir.exists()) {
        final List<VaultInfo> vaults = [];
        await for (final entity in vaultDir.list()) {
          if (entity is Directory) {
            final name = entity.path.split('/').last;
            // Vault directories are named by four-words
            if (name.split('-').length >= 4) {
              vaults.add(VaultInfo(
                id: name,
                fourWords: name,
                displayName: name, // Will be updated after login
                createdAt: (await entity.stat()).modified,
                lastUsed: (await entity.stat()).accessed,
              ));
            }
          }
        }
        state = state.copyWith(availableVaults: vaults);
      }
    } catch (e) {
      debugPrint('Error scanning vault directory: $e');
    }
  }

  /// Login with password to unlock vault
  Future<bool> login(String fourWords, String password) async {
    if (kDemoMode) {
      state = state.copyWith(isAuthenticated: true);
      return true;
    }

    try {
      final storagePath = await _getStoragePath();

      // Create API with the identity
      final api = await CommunitasApi.create(
        fourWords: fourWords,
        displayName: fourWords, // Will be updated from vault
        deviceName: 'Flutter-${Platform.operatingSystem}',
        storagePath: storagePath,
      );

      // Login to vault
      final session = await api.authLogin(
        fourWords: fourWords,
        password: password,
      );

      state = state.copyWith(
        isAuthenticated: true,
        fourWords: session.fourWords,
        displayName: session.displayName,
        api: api,
        currentVault: VaultInfo(
          id: session.sessionId,
          fourWords: session.fourWords,
          displayName: session.displayName,
          createdAt: DateTime.now(),
          lastUsed: DateTime.now(),
        ),
      );
      return true;
    } catch (e) {
      debugPrint('Login failed: $e');
      return false;
    }
  }

  /// Create new identity with four-word address
  Future<String?> createIdentity({
    required String fourWords,
    required String displayName,
    required String password,
  }) async {
    if (kDemoMode) {
      state = state.copyWith(
        isAuthenticated: true,
        fourWords: 'demo-forest-moon-star',
        displayName: displayName,
      );
      return 'demo-forest-moon-star';
    }

    try {
      final storagePath = await _getStoragePath();

      // Create API with the new identity
      final api = await CommunitasApi.create(
        fourWords: fourWords,
        displayName: displayName,
        deviceName: 'Flutter-${Platform.operatingSystem}',
        storagePath: storagePath,
      );

      // Create vault
      await api.authCreateVault(
        fourWords: fourWords,
        displayName: displayName,
        password: password,
      );

      // Login to the new vault
      final session = await api.authLogin(
        fourWords: fourWords,
        password: password,
      );

      state = state.copyWith(
        isAuthenticated: true,
        fourWords: session.fourWords,
        displayName: session.displayName,
        api: api,
        currentVault: VaultInfo(
          id: session.sessionId,
          fourWords: session.fourWords,
          displayName: session.displayName,
          createdAt: DateTime.now(),
          lastUsed: DateTime.now(),
        ),
      );

      // Refresh vault list
      await _loadVaults();

      return fourWords;
    } catch (e) {
      debugPrint('Create identity failed: $e');
      return null;
    }
  }

  /// Logout and lock vault
  Future<void> logout() async {
    if (!kDemoMode && state.api != null) {
      try {
        await state.api!.authLogout();
      } catch (e) {
        debugPrint('Logout error: $e');
      }
    }
    state = state.copyWith(
      isAuthenticated: false,
      currentVault: null,
      api: null,
    );
  }

  /// Get the current API instance
  CommunitasApi? get api => state.api;

  /// Export identity backup
  Future<String?> exportIdentity() async {
    // TODO: Export encrypted identity backup
    return null;
  }

  /// Import identity from backup
  Future<bool> importIdentity(String backup, String password) async {
    // TODO: Import and decrypt identity
    return false;
  }
}

/// Global auth state provider.
final authNotifierProvider =
    StateNotifierProvider<AuthNotifier, AuthState>((ref) {
  return AuthNotifier();
});

/// Current four-words provider (convenience).
final currentFourWordsProvider = Provider<String?>((ref) {
  return ref.watch(authNotifierProvider).fourWords;
});
