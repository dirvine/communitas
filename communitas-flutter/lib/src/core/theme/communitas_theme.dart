import 'package:flutter/material.dart';
import 'colors.dart';

/// Communitas theme system - "Warm Digital Commons" forest aesthetic.
class CommunitasTheme {
  CommunitasTheme._();

  /// Dark theme (primary - forest aesthetic)
  static ThemeData get darkTheme {
    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.dark,
      colorScheme: ColorScheme.dark(
        primary: CommunitasColors.jade,
        secondary: CommunitasColors.amber,
        surface: CommunitasColors.deepForest,
        error: CommunitasColors.error,
        onPrimary: CommunitasColors.cream,
        onSecondary: CommunitasColors.deepForest,
        onSurface: CommunitasColors.cream,
        outline: CommunitasColors.fern,
      ),
      scaffoldBackgroundColor: CommunitasColors.deepForest,
      appBarTheme: const AppBarTheme(
        backgroundColor: CommunitasColors.deepForest,
        foregroundColor: CommunitasColors.cream,
        elevation: 0,
      ),
      cardTheme: CardThemeData(
        color: CommunitasColors.moss,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
        ),
      ),
      listTileTheme: const ListTileThemeData(
        contentPadding: EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: CommunitasColors.moss,
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: BorderSide.none,
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: CommunitasColors.jade, width: 2),
        ),
        contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: CommunitasColors.jade,
          foregroundColor: CommunitasColors.cream,
          padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 12),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(8),
          ),
        ),
      ),
      textButtonTheme: TextButtonThemeData(
        style: TextButton.styleFrom(
          foregroundColor: CommunitasColors.jade,
        ),
      ),
      iconTheme: const IconThemeData(
        color: CommunitasColors.cream,
      ),
      dividerTheme: const DividerThemeData(
        color: CommunitasColors.fern,
        thickness: 1,
      ),
      chipTheme: ChipThemeData(
        backgroundColor: CommunitasColors.moss,
        labelStyle: const TextStyle(color: CommunitasColors.cream),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(16),
        ),
      ),
      textTheme: const TextTheme(
        headlineLarge: TextStyle(
          color: CommunitasColors.cream,
          fontWeight: FontWeight.bold,
        ),
        headlineMedium: TextStyle(
          color: CommunitasColors.cream,
          fontWeight: FontWeight.w600,
        ),
        titleLarge: TextStyle(
          color: CommunitasColors.cream,
          fontWeight: FontWeight.w600,
        ),
        titleMedium: TextStyle(
          color: CommunitasColors.cream,
        ),
        bodyLarge: TextStyle(
          color: CommunitasColors.cream,
        ),
        bodyMedium: TextStyle(
          color: CommunitasColors.cream,
        ),
        labelLarge: TextStyle(
          color: CommunitasColors.cream,
          fontWeight: FontWeight.w500,
        ),
      ),
    );
  }

  /// Light theme (alternative)
  static ThemeData get lightTheme {
    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.light,
      colorScheme: ColorScheme.light(
        primary: CommunitasColors.jade,
        secondary: CommunitasColors.amber,
        surface: CommunitasColors.cream,
        error: CommunitasColors.error,
      ),
      scaffoldBackgroundColor: CommunitasColors.cream,
    );
  }
}
