import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../../main.dart';
import '../../../bindings/api_exports.dart';
import '../../../core/router.dart';
import '../../../core/theme/colors.dart';
import '../providers/auth_provider.dart';

/// Create new identity screen with four-word address generation.
class CreateIdentityScreen extends ConsumerStatefulWidget {
  const CreateIdentityScreen({super.key});

  @override
  ConsumerState<CreateIdentityScreen> createState() =>
      _CreateIdentityScreenState();
}

class _CreateIdentityScreenState extends ConsumerState<CreateIdentityScreen> {
  final _displayNameController = TextEditingController();
  final _passwordController = TextEditingController();
  final _confirmPasswordController = TextEditingController();

  String? _generatedFourWords;
  bool _isGenerating = false;

  @override
  void dispose() {
    _displayNameController.dispose();
    _passwordController.dispose();
    _confirmPasswordController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.go(Routes.login),
        ),
        title: const Text('Create Identity'),
      ),
      body: Center(
        child: Container(
          constraints: const BoxConstraints(maxWidth: 400),
          padding: const EdgeInsets.all(32),
          child: SingleChildScrollView(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                // Four-word address display
                Container(
                  padding: const EdgeInsets.all(24),
                  decoration: BoxDecoration(
                    color: CommunitasColors.moss,
                    borderRadius: BorderRadius.circular(16),
                    border: Border.all(
                      color: _generatedFourWords != null
                          ? CommunitasColors.jade
                          : CommunitasColors.fern,
                    ),
                  ),
                  child: Column(
                    children: [
                      const Icon(
                        Icons.fingerprint,
                        size: 48,
                        color: CommunitasColors.jade,
                      ),
                      const SizedBox(height: 16),
                      Text(
                        _generatedFourWords ?? 'Your Four-Word Address',
                        style: Theme.of(context).textTheme.titleLarge?.copyWith(
                              color: _generatedFourWords != null
                                  ? CommunitasColors.jade
                                  : CommunitasColors.cream.withOpacity(0.5),
                              fontWeight: FontWeight.bold,
                            ),
                        textAlign: TextAlign.center,
                      ),
                      const SizedBox(height: 8),
                      Text(
                        'This is your unique, human-readable identity',
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                              color: CommunitasColors.cream.withOpacity(0.7),
                            ),
                        textAlign: TextAlign.center,
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 24),

                // Generate button
                OutlinedButton.icon(
                  onPressed: _isGenerating ? null : _generateFourWords,
                  icon: _isGenerating
                      ? const SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.refresh),
                  label: Text(
                    _generatedFourWords != null ? 'Regenerate' : 'Generate',
                  ),
                ),
                const SizedBox(height: 32),

                // Display name
                TextField(
                  controller: _displayNameController,
                  decoration: const InputDecoration(
                    labelText: 'Display Name',
                    prefixIcon: Icon(Icons.person_outline),
                    helperText: 'How others will see you',
                  ),
                ),
                const SizedBox(height: 16),

                // Password
                TextField(
                  controller: _passwordController,
                  obscureText: true,
                  decoration: const InputDecoration(
                    labelText: 'Password',
                    prefixIcon: Icon(Icons.lock_outline),
                    helperText: 'Protects your local vault',
                  ),
                ),
                const SizedBox(height: 16),

                // Confirm password
                TextField(
                  controller: _confirmPasswordController,
                  obscureText: true,
                  decoration: const InputDecoration(
                    labelText: 'Confirm Password',
                    prefixIcon: Icon(Icons.lock_outline),
                  ),
                ),
                const SizedBox(height: 32),

                // Create button
                ElevatedButton(
                  onPressed: _generatedFourWords != null ? _handleCreate : null,
                  child: const Text('Create Identity'),
                ),
                const SizedBox(height: 16),

                // Security notice
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: CommunitasColors.info.withOpacity(0.1),
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Row(
                    children: [
                      const Icon(
                        Icons.info_outline,
                        color: CommunitasColors.info,
                        size: 20,
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: Text(
                          'Your identity uses post-quantum cryptography (ML-DSA) for future-proof security.',
                          style:
                              Theme.of(context).textTheme.bodySmall?.copyWith(
                                    color: CommunitasColors.info,
                                  ),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Future<void> _generateFourWords() async {
    setState(() => _isGenerating = true);

    try {
      String fourWords;
      if (kDemoMode) {
        // Demo mode: use placeholder
        await Future.delayed(const Duration(milliseconds: 500));
        fourWords = 'ocean-forest-moon-star';
      } else {
        // Native mode: generate real four-word address via FFI
        fourWords = await generateIdWords();
      }

      setState(() {
        _generatedFourWords = fourWords;
        _isGenerating = false;
      });
    } catch (e) {
      debugPrint('Error generating four words: $e');
      setState(() => _isGenerating = false);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Failed to generate identity: $e')),
        );
      }
    }
  }

  Future<void> _handleCreate() async {
    // Validate password match
    if (_passwordController.text != _confirmPasswordController.text) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Passwords do not match')),
      );
      return;
    }

    // Validate display name
    if (_displayNameController.text.trim().isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Please enter a display name')),
      );
      return;
    }

    // Validate password length
    if (_passwordController.text.length < 4) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Password must be at least 4 characters')),
      );
      return;
    }

    // Create identity via auth provider
    final authNotifier = ref.read(authNotifierProvider.notifier);
    final result = await authNotifier.createIdentity(
      fourWords: _generatedFourWords!,
      displayName: _displayNameController.text.trim(),
      password: _passwordController.text,
    );

    if (result != null && mounted) {
      // Success - navigate to home
      context.go(Routes.home);
    } else if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Failed to create identity')),
      );
    }
  }
}
