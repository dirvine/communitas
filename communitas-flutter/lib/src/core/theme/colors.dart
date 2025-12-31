import 'package:flutter/material.dart';

/// Communitas color palette - "Warm Digital Commons" forest aesthetic.
class CommunitasColors {
  CommunitasColors._();

  // ============================================
  // Foundation Colors
  // ============================================

  /// Primary background - deep forest green
  static const Color deepForest = Color(0xFF1A241F);

  /// Secondary background - moss green
  static const Color moss = Color(0xFF2D3D36);

  /// Tertiary background - fern green
  static const Color fern = Color(0xFF394C43);

  /// Primary accent - jade green
  static const Color jade = Color(0xFF4CAF83);

  /// Secondary accent - warm amber
  static const Color amber = Color(0xFFE0B265);

  /// Light text - cream white
  static const Color cream = Color(0xFFF2EEE7);

  // ============================================
  // Entity Colors
  // ============================================

  /// Organization entity color
  static const Color organization = Color(0xFF3E8E7E);

  /// Project entity color
  static const Color project = Color(0xFFDAA520);

  /// Channel entity color
  static const Color channel = Color(0xFF4CAF83);

  /// Group entity color
  static const Color group = Color(0xFF9B59B6);

  /// Person/Contact entity color
  static const Color person = Color(0xFFFF7F7F);

  // ============================================
  // Status Colors
  // ============================================

  /// Online status
  static const Color online = Color(0xFF4CAF50);

  /// Away status
  static const Color away = Color(0xFFFFC107);

  /// Offline status
  static const Color offline = Color(0xFF9E9E9E);

  /// Do Not Disturb status
  static const Color doNotDisturb = Color(0xFFE74C3C);

  // ============================================
  // Role Badge Colors
  // ============================================

  /// Owner role badge
  static const Color owner = Color(0xFFE59933);

  /// Admin role badge
  static const Color admin = Color(0xFF4D80E6);

  /// Member role badge
  static const Color member = Color(0xFF808080);

  /// Guest role badge
  static const Color guest = Color(0xFF9999B3);

  // ============================================
  // Semantic Colors
  // ============================================

  /// Error/danger color
  static const Color error = Color(0xFFE74C3C);

  /// Warning color
  static const Color warning = Color(0xFFF39C12);

  /// Success color
  static const Color success = Color(0xFF27AE60);

  /// Info color
  static const Color info = Color(0xFF3498DB);

  // ============================================
  // Kanban Priority Colors
  // ============================================

  /// Critical priority
  static const Color priorityCritical = Color(0xFFE74C3C);

  /// High priority
  static const Color priorityHigh = Color(0xFFF39C12);

  /// Medium priority
  static const Color priorityMedium = Color(0xFF3498DB);

  /// Low priority
  static const Color priorityLow = Color(0xFF27AE60);

  // ============================================
  // Call Status Colors
  // ============================================

  /// Incoming call
  static const Color callIncoming = Color(0xFF4CAF50);

  /// Active call
  static const Color callActive = Color(0xFF27AE60);

  /// Call ended
  static const Color callEnded = Color(0xFF9E9E9E);

  /// Call rejected/missed
  static const Color callMissed = Color(0xFFE74C3C);

  // ============================================
  // Helper Methods
  // ============================================

  /// Get entity color by type name
  static Color entityColor(String entityType) {
    switch (entityType.toLowerCase()) {
      case 'organization':
      case 'org':
        return organization;
      case 'project':
        return project;
      case 'channel':
        return channel;
      case 'group':
        return group;
      case 'person':
      case 'contact':
        return person;
      default:
        return jade;
    }
  }

  /// Get role color by role name
  static Color roleColor(String role) {
    switch (role.toLowerCase()) {
      case 'owner':
        return owner;
      case 'admin':
        return admin;
      case 'member':
        return member;
      case 'guest':
        return guest;
      default:
        return member;
    }
  }

  /// Get status color by status name
  static Color statusColor(String status) {
    switch (status.toLowerCase()) {
      case 'online':
        return online;
      case 'away':
        return away;
      case 'offline':
        return offline;
      case 'dnd':
      case 'do_not_disturb':
        return doNotDisturb;
      default:
        return offline;
    }
  }
}
