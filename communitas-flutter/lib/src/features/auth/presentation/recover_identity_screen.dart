import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../../main.dart';
import '../../../core/router.dart';
import '../../../core/theme/colors.dart';
import '../providers/auth_provider.dart';

/// Recover identity from BIP39 mnemonic phrase (ADR-016).
class RecoverIdentityScreen extends ConsumerStatefulWidget {
  const RecoverIdentityScreen({super.key});

  @override
  ConsumerState<RecoverIdentityScreen> createState() =>
      _RecoverIdentityScreenState();
}

class _RecoverIdentityScreenState extends ConsumerState<RecoverIdentityScreen> {
  final _mnemonicController = TextEditingController();
  final _passphraseController = TextEditingController();
  final _displayNameController = TextEditingController();
  final _passwordController = TextEditingController();
  final _confirmPasswordController = TextEditingController();

  String? _recoveredFourWords;
  String? _validationError;
  bool _isValidating = false;
  bool _isRecovering = false;
  bool _showAdvanced = false;

  @override
  void dispose() {
    _mnemonicController.dispose();
    _passphraseController.dispose();
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
        title: const Text('Recover Identity'),
      ),
      body: Center(
        child: Container(
          constraints: const BoxConstraints(maxWidth: 500),
          padding: const EdgeInsets.all(32),
          child: SingleChildScrollView(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                // Info banner
                Container(
                  padding: const EdgeInsets.all(16),
                  decoration: BoxDecoration(
                    color: CommunitasColors.info.withOpacity(0.1),
                    borderRadius: BorderRadius.circular(12),
                    border: Border.all(color: CommunitasColors.info),
                  ),
                  child: Row(
                    children: [
                      const Icon(
                        Icons.restore,
                        color: CommunitasColors.info,
                        size: 24,
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: Text(
                          'Enter your recovery phrase to restore your identity on this device.',
                          style:
                              Theme.of(context).textTheme.bodyMedium?.copyWith(
                                    color: CommunitasColors.info,
                                  ),
                        ),
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 24),

                // Mnemonic input
                TextField(
                  controller: _mnemonicController,
                  maxLines: 4,
                  decoration: InputDecoration(
                    labelText: 'Recovery Phrase',
                    hintText: 'Enter your 12 or 24 word recovery phrase...',
                    prefixIcon: const Padding(
                      padding: EdgeInsets.only(bottom: 60),
                      child: Icon(Icons.vpn_key_outlined),
                    ),
                    errorText: _validationError,
                    helperText: 'Separate words with spaces',
                  ),
                  onChanged: (_) {
                    // Clear validation state on edit
                    if (_validationError != null || _recoveredFourWords != null) {
                      setState(() {
                        _validationError = null;
                        _recoveredFourWords = null;
                      });
                    }
                  },
                ),
                const SizedBox(height: 16),

                // Advanced options toggle
                GestureDetector(
                  onTap: () => setState(() => _showAdvanced = !_showAdvanced),
                  child: Row(
                    children: [
                      Icon(
                        _showAdvanced
                            ? Icons.expand_less
                            : Icons.expand_more,
                        size: 20,
                        color: CommunitasColors.cream.withOpacity(0.7),
                      ),
                      const SizedBox(width: 4),
                      Text(
                        'Advanced options',
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                              color: CommunitasColors.cream.withOpacity(0.7),
                            ),
                      ),
                    ],
                  ),
                ),

                // Passphrase (advanced)
                if (_showAdvanced) ...[
                  const SizedBox(height: 12),
                  TextField(
                    controller: _passphraseController,
                    obscureText: true,
                    decoration: const InputDecoration(
                      labelText: 'BIP39 Passphrase (optional)',
                      prefixIcon: Icon(Icons.security),
                      helperText: 'Only if you set a passphrase when creating',
                    ),
                  ),
                ],
                const SizedBox(height: 24),

                // Validate button
                OutlinedButton.icon(
                  onPressed: _isValidating ? null : _validateMnemonic,
                  icon: _isValidating
                      ? const SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.check_circle_outline),
                  label: const Text('Validate Phrase'),
                ),
                const SizedBox(height: 24),

                // Recovered identity display
                if (_recoveredFourWords != null) ...[
                  Container(
                    padding: const EdgeInsets.all(24),
                    decoration: BoxDecoration(
                      color: CommunitasColors.moss,
                      borderRadius: BorderRadius.circular(16),
                      border: Border.all(color: CommunitasColors.jade),
                    ),
                    child: Column(
                      children: [
                        const Icon(
                          Icons.check_circle,
                          size: 48,
                          color: CommunitasColors.jade,
                        ),
                        const SizedBox(height: 16),
                        Text(
                          _recoveredFourWords!,
                          style:
                              Theme.of(context).textTheme.titleLarge?.copyWith(
                                    color: CommunitasColors.jade,
                                    fontWeight: FontWeight.bold,
                                  ),
                          textAlign: TextAlign.center,
                        ),
                        const SizedBox(height: 8),
                        Text(
                          'Identity verified successfully',
                          style:
                              Theme.of(context).textTheme.bodySmall?.copyWith(
                                    color: CommunitasColors.cream.withOpacity(0.7),
                                  ),
                          textAlign: TextAlign.center,
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: 24),

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

                  // New vault password
                  TextField(
                    controller: _passwordController,
                    obscureText: true,
                    decoration: const InputDecoration(
                      labelText: 'New Vault Password',
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
                  const SizedBox(height: 24),

                  // Recover button
                  ElevatedButton(
                    onPressed: _isRecovering ? null : _handleRecover,
                    child: _isRecovering
                        ? const SizedBox(
                            height: 20,
                            width: 20,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Text('Restore Identity'),
                  ),
                ],
                const SizedBox(height: 24),

                // Security notice
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: CommunitasColors.warning.withOpacity(0.1),
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Icon(
                        Icons.warning_amber_rounded,
                        color: CommunitasColors.warning,
                        size: 20,
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: Text(
                          'Never share your recovery phrase. Anyone with these words can access your identity.',
                          style:
                              Theme.of(context).textTheme.bodySmall?.copyWith(
                                    color: CommunitasColors.warning,
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

  Future<void> _validateMnemonic() async {
    final mnemonic = _mnemonicController.text.trim();
    if (mnemonic.isEmpty) {
      setState(() => _validationError = 'Please enter your recovery phrase');
      return;
    }

    // Normalize: collapse whitespace, lowercase
    final words = mnemonic.toLowerCase().split(RegExp(r'\s+'));
    final wordCount = words.length;

    // Check word count
    if (![12, 15, 18, 21, 24].contains(wordCount)) {
      setState(() => _validationError =
          'Invalid word count ($wordCount). Must be 12, 15, 18, 21, or 24 words.');
      return;
    }

    setState(() {
      _isValidating = true;
      _validationError = null;
    });

    try {
      if (kDemoMode) {
        // Demo mode: simulate validation
        await Future.delayed(const Duration(milliseconds: 500));

        // Check if using a known test mnemonic
        final normalizedMnemonic = words.join(' ');
        if (normalizedMnemonic.startsWith('abandon abandon abandon')) {
          setState(() {
            _recoveredFourWords = 'test-vault-demo-key';
            _isValidating = false;
          });
        } else {
          setState(() {
            _validationError = 'Invalid mnemonic (demo mode only accepts test vectors)';
            _isValidating = false;
          });
        }
      } else {
        // Native mode: validate via FFI
        // TODO: Call native validation when bindings are ready
        final normalizedMnemonic = words.join(' ');
        // ignore: unused_local_variable
        final passphrase = _passphraseController.text.isNotEmpty
            ? _passphraseController.text
            : null;

        // For now, use demo fallback until bindings ready
        // When ready: validateMnemonic(normalizedMnemonic, passphrase)
        await Future.delayed(const Duration(milliseconds: 500));
        if (normalizedMnemonic.startsWith('abandon abandon abandon')) {
          setState(() {
            _recoveredFourWords = 'recovered-identity-four-words';
            _isValidating = false;
          });
        } else {
          setState(() {
            _validationError = 'Invalid mnemonic phrase';
            _isValidating = false;
          });
        }
      }
    } catch (e) {
      debugPrint('Validation error: $e');
      setState(() {
        _validationError = 'Validation failed: $e';
        _isValidating = false;
      });
    }
  }

  Future<void> _handleRecover() async {
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

    setState(() => _isRecovering = true);

    try {
      final authNotifier = ref.read(authNotifierProvider.notifier);

      // Recover identity via auth provider
      final result = await authNotifier.recoverIdentity(
        mnemonic: _mnemonicController.text.trim(),
        passphrase: _passphraseController.text.isNotEmpty
            ? _passphraseController.text
            : null,
        displayName: _displayNameController.text.trim(),
        password: _passwordController.text,
      );

      if (result != null && mounted) {
        // Success - navigate to home
        context.go(Routes.home);
      } else if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Failed to recover identity')),
        );
      }
    } catch (e) {
      debugPrint('Recovery error: $e');
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Recovery failed: $e')),
        );
      }
    } finally {
      if (mounted) {
        setState(() => _isRecovering = false);
      }
    }
  }
}
