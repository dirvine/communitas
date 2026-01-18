// Communitas Flutter smoke test.
//
// Verifies that the app can be instantiated and renders without errors.

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:communitas/main.dart';

void main() {
  testWidgets('Communitas app smoke test', (WidgetTester tester) async {
    // Build the Communitas app wrapped in ProviderScope (required by Riverpod)
    await tester.pumpWidget(
      const ProviderScope(
        child: CommunitasApp(),
      ),
    );

    // Verify that the app renders without errors
    // The app title should appear somewhere in the widget tree
    expect(find.byType(MaterialApp), findsOneWidget);
  });
}
