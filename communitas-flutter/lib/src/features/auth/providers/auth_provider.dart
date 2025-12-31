import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../../main.dart';
import '../../../demo/demo_data.dart';

/// Authentication state for Communitas.
class AuthState {
  final bool isAuthenticated;
  final String? fourWords;
  final String? displayName;
  final List<VaultInfo> availableVaults;
  final VaultInfo? currentVault;

  const AuthState({
    this.isAuthenticated = false,
    this.fourWords,
    this.displayName,
    this.availableVaults = const [],
    this.currentVault,
  });

  AuthState copyWith({
    bool? isAuthenticated,
    String? fourWords,
    String? displayName,
    List<VaultInfo>? availableVaults,
    VaultInfo? currentVault,
  }) {
    return AuthState(
      isAuthenticated: isAuthenticated ?? this.isAuthenticated,
      fourWords: fourWords ?? this.fourWords,
      displayName: displayName ?? this.displayName,
      availableVaults: availableVaults ?? this.availableVaults,
      currentVault: currentVault ?? this.currentVault,
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
    // In demo mode, auto-login with demo identity
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

    // TODO: Load vaults from secure storage
    // For now, start unauthenticated
  }

  /// Login with password to unlock vault
  Future<bool> login(String vaultId, String password) async {
    // TODO: Implement actual vault unlock via FFI
    // For demo, just authenticate
    if (kDemoMode) {
      state = state.copyWith(isAuthenticated: true);
      return true;
    }
    return false;
  }

  /// Create new identity with four-word address
  Future<String?> createIdentity({
    required String displayName,
    required String password,
  }) async {
    // TODO: Implement via FFI - generate ML-DSA keys, create vault
    // Returns the four-word address
    if (kDemoMode) {
      state = state.copyWith(
        isAuthenticated: true,
        fourWords: 'demo-forest-moon-star',
        displayName: displayName,
      );
      return 'demo-forest-moon-star';
    }
    return null;
  }

  /// Logout and lock vault
  void logout() {
    state = state.copyWith(
      isAuthenticated: false,
      currentVault: null,
    );
  }

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
