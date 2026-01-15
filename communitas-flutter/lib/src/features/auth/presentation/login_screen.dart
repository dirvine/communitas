import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/router.dart';
import '../../../core/theme/colors.dart';
import '../../../../main.dart';
import '../providers/auth_provider.dart';

/// Login screen with vault selection and demo user support.
class LoginScreen extends ConsumerStatefulWidget {
  const LoginScreen({super.key});

  @override
  ConsumerState<LoginScreen> createState() => _LoginScreenState();
}

class _LoginScreenState extends ConsumerState<LoginScreen> {
  final _usernameController = TextEditingController();
  final _passwordController = TextEditingController();
  bool _isLoading = false;
  String? _selectedFourWords;

  @override
  void dispose() {
    _usernameController.dispose();
    _passwordController.dispose();
    super.dispose();
  }

  /// Check if the entered credentials are for demo user.
  bool get _isDemoLogin =>
      _usernameController.text.trim().toLowerCase() == 'demo' &&
      _passwordController.text == 'demo';

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
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

              // Demo mode indicator (compile-time flag)
              if (kDemoMode) ...[
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: CommunitasColors.amber.withAlpha(51),
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: CommunitasColors.amber),
                  ),
                  child: const Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.science, color: CommunitasColors.amber),
                      SizedBox(width: 8),
                      Text(
                        'Demo Mode',
                        style: TextStyle(color: CommunitasColors.amber),
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 24),
              ],

              // Demo login hint
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: CommunitasColors.jade.withAlpha(26),
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(
                    color: CommunitasColors.jade.withAlpha(77),
                  ),
                ),
                child: Row(
                  children: [
                    Icon(
                      Icons.lightbulb_outline,
                      color: CommunitasColors.jade.withAlpha(204),
                      size: 20,
                    ),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        'Tip: Use "demo" / "demo" for quick testing',
                        style: TextStyle(
                          fontSize: 12,
                          color: CommunitasColors.jade.withAlpha(204),
                        ),
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 16),

              // Username field
              TextField(
                controller: _usernameController,
                decoration: const InputDecoration(
                  labelText: 'Username',
                  prefixIcon: Icon(Icons.person_outline),
                  hintText: 'Enter username or "demo"',
                ),
                textInputAction: TextInputAction.next,
                onChanged: (_) => setState(() {}),
              ),
              const SizedBox(height: 16),

              // Password field
              TextField(
                controller: _passwordController,
                obscureText: true,
                decoration: const InputDecoration(
                  labelText: 'Password',
                  prefixIcon: Icon(Icons.lock_outline),
                  hintText: 'Enter password',
                ),
                textInputAction: TextInputAction.done,
                onSubmitted: (_) => _handleLogin(),
                onChanged: (_) => setState(() {}),
              ),
              const SizedBox(height: 24),

              // Login button
              ElevatedButton(
                onPressed: _isLoading ? null : _handleLogin,
                style: _isDemoLogin
                    ? ElevatedButton.styleFrom(
                        backgroundColor: CommunitasColors.amber,
                        foregroundColor: CommunitasColors.deepForest,
                      )
                    : null,
                child: _isLoading
                    ? const SizedBox(
                        height: 20,
                        width: 20,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : Text(_isDemoLogin ? 'Login as Demo User' : 'Unlock'),
              ),
              const SizedBox(height: 16),

              // Create new identity
              TextButton(
                onPressed: () => context.go(Routes.createIdentity),
                child: const Text('Create new identity'),
              ),

              // Recover existing identity
              TextButton(
                onPressed: () => context.go(Routes.recoverIdentity),
                child: const Text('Recover existing identity'),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _handleLogin() async {
    setState(() => _isLoading = true);

    try {
      final authNotifier = ref.read(authNotifierProvider.notifier);

      bool success;

      // Check for demo user login
      if (_isDemoLogin) {
        success = await authNotifier.loginAsDemo();
      } else {
        // Normal login flow
        final fourWords = _selectedFourWords ?? 'ocean-forest-moon-star';
        success = await authNotifier.login(fourWords, _passwordController.text);
      }

      if (mounted) {
        if (success) {
          context.go(Routes.home);
        } else {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('Login failed. Check your credentials.')),
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
