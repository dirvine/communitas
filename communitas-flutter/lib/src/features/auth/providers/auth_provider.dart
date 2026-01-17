import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';

import '../../../bindings/api_exports.dart';

/// Authentication state for Communitas.
class AuthState {
  final bool isAuthenticated;

  /// Hex-encoded ML-DSA-87 public key (THE identity, Level 5 PQC).
  final String? pubkeyHex;

  /// Legacy: Four words used for vault storage filename.
  final String? fourWords;

  /// User-chosen display name (shown in UI).
  final String? displayName;
  final List<VaultInfo> availableVaults;
  final VaultInfo? currentVault;
  final CommunitasApi? api;
  final String? storagePath;

  const AuthState({
    this.isAuthenticated = false,
    this.pubkeyHex,
    this.fourWords,
    this.displayName,
    this.availableVaults = const [],
    this.currentVault,
    this.api,
    this.storagePath,
  });

  AuthState copyWith({
    bool? isAuthenticated,
    String? pubkeyHex,
    String? fourWords,
    String? displayName,
    List<VaultInfo>? availableVaults,
    VaultInfo? currentVault,
    CommunitasApi? api,
    String? storagePath,
  }) {
    return AuthState(
      isAuthenticated: isAuthenticated ?? this.isAuthenticated,
      pubkeyHex: pubkeyHex ?? this.pubkeyHex,
      fourWords: fourWords ?? this.fourWords,
      displayName: displayName ?? this.displayName,
      availableVaults: availableVaults ?? this.availableVaults,
      currentVault: currentVault ?? this.currentVault,
      api: api ?? this.api,
      storagePath: storagePath ?? this.storagePath,
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

  /// Start gossip network after successful authentication.
  /// Called automatically by all login/create methods.
  Future<void> _autoConnectNetwork(CommunitasApi api) async {
    try {
      debugPrint('Auto-connecting to gossip network...');
      await api.gossipStart();
      debugPrint('Gossip network started successfully');
    } catch (e) {
      // Log but don't fail login - network can be started manually later
      debugPrint('Auto-connect to network failed (non-fatal): $e');
    }
  }

  Future<void> _initializeAuth() async {
    // Try to list existing vaults
    try {
      await _loadVaults();
    } catch (e) {
      debugPrint('Error loading vaults: $e');
    }
  }

  /// Get storage path for vaults
  Future<String> _getStoragePath() async {
    final appDir = await getApplicationSupportDirectory();
    return appDir.path;
  }

  /// Load existing vaults from storage
  Future<void> _loadVaults() async {
    // We need to create a temporary API to list vaults
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
        state = state.copyWith(
          availableVaults: vaults,
          storagePath: storagePath,
        );
      }
    } catch (e) {
      debugPrint('Error scanning vault directory: $e');
    }
  }

  /// Login with password to unlock vault
  Future<bool> login(String fourWords, String password) async {
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
        pubkeyHex: session.pubkeyHex,
        fourWords: session.fourWords,
        displayName: session.displayName,
        api: api,
        storagePath: storagePath,
        currentVault: VaultInfo(
          id: session.sessionId,
          fourWords: session.fourWords,
          displayName: session.displayName,
          createdAt: DateTime.now(),
          lastUsed: DateTime.now(),
        ),
      );

      // Auto-connect to gossip network
      await _autoConnectNetwork(api);

      return true;
    } catch (e) {
      debugPrint('Login failed: $e');
      return false;
    }
  }

  /// Create new identity with display name and password.
  ///
  /// The vault identifier (four_words) is auto-generated internally.
  /// Identity is based on ML-DSA-65 keypair, not the four-word address.
  Future<String?> createIdentity({
    required String displayName,
    required String password,
  }) async {
    try {
      final storagePath = await _getStoragePath();

      // Auto-generate vault identifier (not user identity - just storage key)
      final fourWords = await generateIdWords();

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
        pubkeyHex: session.pubkeyHex,
        fourWords: session.fourWords,
        displayName: session.displayName,
        api: api,
        storagePath: storagePath,
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

      // Auto-connect to gossip network
      await _autoConnectNetwork(api);

      return fourWords;
    } catch (e) {
      debugPrint('Create identity failed: $e');
      return null;
    }
  }

  /// Recover identity from BIP39 mnemonic phrase (ADR-016).
  ///
  /// Validates the mnemonic, derives keys, and creates a vault.
  Future<String?> recoverIdentity({
    required String mnemonic,
    String? passphrase,
    required String displayName,
    required String password,
  }) async {
    try {
      final storagePath = await _getStoragePath();

      final normalizedPassphrase =
          passphrase != null && passphrase.trim().isNotEmpty
              ? passphrase.trim()
              : null;

      final recovered = await recoverIdentityFromMnemonic(
        mnemonic: mnemonic,
        passphrase: normalizedPassphrase,
      );
      final fourWords = recovered.fourWords;

      // Create API with the recovered identity
      final api = await CommunitasApi.create(
        fourWords: fourWords,
        displayName: displayName,
        deviceName: 'Flutter-${Platform.operatingSystem}',
        storagePath: storagePath,
      );

      final exists = await api.authVaultExists(fourWords: fourWords);
      if (!exists) {
        await api.authCreateVault(
          fourWords: fourWords,
          displayName: displayName,
          password: password,
        );
      }

      // Login to the new vault
      final session = await api.authLogin(
        fourWords: fourWords,
        password: password,
      );

      state = state.copyWith(
        isAuthenticated: true,
        pubkeyHex: recovered.pubkeyHex,
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

      // Auto-connect to gossip network
      await _autoConnectNetwork(api);

      return fourWords;
    } catch (e) {
      debugPrint('Recover identity failed: $e');
      return null;
    }
  }

  /// Logout and lock vault
  Future<void> logout() async {
    if (state.api != null) {
      try {
        // Stop gossip network before logout
        debugPrint('Stopping gossip network...');
        await state.api!.gossipStop();
        debugPrint('Gossip network stopped');
      } catch (e) {
        debugPrint('Stop network error (non-fatal): $e');
      }
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
    final api = state.api;
    if (api == null) return null;

    try {
      return await api.authExportVault(includeData: true);
    } catch (e) {
      debugPrint('Export identity failed: $e');
      return null;
    }
  }

  /// Import identity from backup
  Future<bool> importIdentity(String backup, String password) async {
    try {
      final storagePath = await _getStoragePath();
      CommunitasApi api = state.api ??
          await CommunitasApi.create(
            fourWords: await generateIdWords(),
            displayName: 'Import',
            deviceName: 'Flutter-${Platform.operatingSystem}',
            storagePath: storagePath,
          );

      final fourWords = await api.authImportVault(
        backupBase64: backup,
        password: password,
      );

      final session = await api.authLogin(
        fourWords: fourWords,
        password: password,
      );

      state = state.copyWith(
        isAuthenticated: true,
        pubkeyHex: session.pubkeyHex,
        fourWords: session.fourWords,
        displayName: session.displayName,
        api: api,
        storagePath: storagePath,
        currentVault: VaultInfo(
          id: session.sessionId,
          fourWords: session.fourWords,
          displayName: session.displayName,
          createdAt: DateTime.now(),
          lastUsed: DateTime.now(),
        ),
      );

      await _loadVaults();

      // Auto-connect to gossip network
      await _autoConnectNetwork(api);

      return true;
    } catch (e) {
      debugPrint('Import identity failed: $e');
      return false;
    }
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
