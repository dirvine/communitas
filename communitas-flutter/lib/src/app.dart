import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'core/router.dart';
import 'core/theme/communitas_theme.dart';

/// The main Communitas application widget.
class CommunitasApp extends ConsumerWidget {
  const CommunitasApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final router = ref.watch(routerProvider);

    return MaterialApp.router(
      title: 'Communitas',
      debugShowCheckedModeBanner: false,
      theme: CommunitasTheme.lightTheme,
      darkTheme: CommunitasTheme.darkTheme,
      themeMode: ThemeMode.dark, // Forest theme is dark by default
      routerConfig: router,
    );
  }
}
