import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/router.dart';
import '../../../core/theme/colors.dart';
import '../providers/auth_provider.dart';

/// Login screen with vault selection.
class LoginScreen extends ConsumerStatefulWidget {
  const LoginScreen({super.key});

  @override
  ConsumerState<LoginScreen> createState() => _LoginScreenState();
}

class _LoginScreenState extends ConsumerState<LoginScreen> {
  final _passwordController = TextEditingController();
  bool _isLoading = false;
  VaultInfo? _selectedVault;

  @override
  void dispose() {
    _passwordController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final authState = ref.watch(authNotifierProvider);
    final vaults = authState.availableVaults;

    return Scaffold(
      body: Center(
        child: SingleChildScrollView(
          child: Container(
            constraints: const BoxConstraints(maxWidth: 400),
            padding: const EdgeInsets.all(32),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
              // Logo/Title
              const Icon(
                Icons.forest,
                size: 80,
                color: CommunitasColors.jade,
              ),
              const SizedBox(height: 24),
              Text(
                'Communitas',
                style: Theme.of(context).textTheme.headlineLarge,
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 8),
              Text(
                'Local-first collaboration',
                style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                      color: CommunitasColors.cream.withAlpha(179),
                    ),
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 48),

              // Vault selection
              if (vaults.isNotEmpty) ...[
                Text(
                  'Select an account',
                  style: Theme.of(context).textTheme.titleSmall,
                ),
                const SizedBox(height: 8),
                Container(
                  decoration: BoxDecoration(
                    border: Border.all(
                      color: CommunitasColors.cream.withAlpha(77),
                    ),
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Column(
                    children: [
                      for (final vault in vaults)
                        _buildVaultTile(vault),
                    ],
                  ),
                ),
                const SizedBox(height: 24),
              ] else ...[
                // No existing vaults message
                Container(
                  padding: const EdgeInsets.all(16),
                  decoration: BoxDecoration(
                    color: CommunitasColors.jade.withAlpha(26),
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(
                      color: CommunitasColors.jade.withAlpha(77),
                    ),
                  ),
                  child: Column(
                    children: [
                      Icon(
                        Icons.person_add_outlined,
                        color: CommunitasColors.jade.withAlpha(204),
                        size: 32,
                      ),
                      const SizedBox(height: 8),
                      Text(
                        'No accounts found',
                        style: TextStyle(
                          fontSize: 14,
                          fontWeight: FontWeight.w500,
                          color: CommunitasColors.jade.withAlpha(204),
                        ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        'Create a new identity to get started',
                        style: TextStyle(
                          fontSize: 12,
                          color: CommunitasColors.cream.withAlpha(153),
                        ),
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 24),
              ],

              // Password field (only if vault selected)
              if (_selectedVault != null) ...[
                Text(
                  'Unlock ${_selectedVault!.displayName}',
                  style: Theme.of(context).textTheme.titleSmall,
                ),
                const SizedBox(height: 8),
                TextField(
                  controller: _passwordController,
                  obscureText: true,
                  autofocus: true,
                  decoration: const InputDecoration(
                    labelText: 'Password',
                    prefixIcon: Icon(Icons.lock_outline),
                    hintText: 'Enter your password',
                  ),
                  textInputAction: TextInputAction.done,
                  onSubmitted: (_) => _handleLogin(),
                ),
                const SizedBox(height: 16),

                // Unlock button
                ElevatedButton(
                  onPressed: _isLoading ? null : _handleLogin,
                  child: _isLoading
                      ? const SizedBox(
                          height: 20,
                          width: 20,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Text('Unlock'),
                ),
                const SizedBox(height: 8),
                TextButton(
                  onPressed: () => setState(() => _selectedVault = null),
                  child: const Text('Use a different account'),
                ),
              ],

              if (_selectedVault == null) ...[
                const SizedBox(height: 8),
                // Create new identity
                ElevatedButton.icon(
                  onPressed: () => context.go(Routes.createIdentity),
                  icon: const Icon(Icons.add),
                  label: const Text('Create new identity'),
                ),
                const SizedBox(height: 8),

                // Recover existing identity
                TextButton.icon(
                  onPressed: () => context.go(Routes.recoverIdentity),
                  icon: const Icon(Icons.restore),
                  label: const Text('Recover existing identity'),
                ),
              ],
            ],
          ),
        ),
      ),
      ),
    );
  }

  Widget _buildVaultTile(VaultInfo vault) {
    final isSelected = _selectedVault?.fourWords == vault.fourWords;
    return InkWell(
      onTap: () {
        setState(() {
          _selectedVault = vault;
          _passwordController.clear();
        });
      },
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        decoration: BoxDecoration(
          color: isSelected ? CommunitasColors.jade.withAlpha(51) : null,
          border: Border(
            bottom: BorderSide(
              color: CommunitasColors.cream.withAlpha(26),
            ),
          ),
        ),
        child: Row(
          children: [
            Container(
              width: 40,
              height: 40,
              decoration: BoxDecoration(
                color: CommunitasColors.jade.withAlpha(77),
                borderRadius: BorderRadius.circular(20),
              ),
              child: Center(
                child: Text(
                  vault.displayName.isNotEmpty
                      ? vault.displayName[0].toUpperCase()
                      : '?',
                  style: const TextStyle(
                    fontSize: 18,
                    fontWeight: FontWeight.bold,
                    color: CommunitasColors.cream,
                  ),
                ),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Text(
                vault.displayName,
                style: const TextStyle(
                  fontWeight: FontWeight.w500,
                ),
              ),
            ),
            if (isSelected)
              const Icon(
                Icons.check_circle,
                color: CommunitasColors.jade,
              ),
          ],
        ),
      ),
    );
  }

  Future<void> _handleLogin() async {
    if (_selectedVault == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Please select an account first.')),
      );
      return;
    }

    setState(() => _isLoading = true);

    try {
      final authNotifier = ref.read(authNotifierProvider.notifier);
      final success = await authNotifier.login(
        _selectedVault!.fourWords,
        _passwordController.text,
      );

      if (mounted) {
        if (success) {
          context.go(Routes.home);
        } else {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('Login failed. Check your password.')),
          );
        }
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Login error: $e')),
        );
      }
    } finally {
      if (mounted) {
        setState(() => _isLoading = false);
      }
    }
  }
}
