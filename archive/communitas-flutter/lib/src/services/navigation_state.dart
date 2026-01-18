import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

const _recentEntitiesKey = 'recent_entities';
const _recentContactsKey = 'recent_contacts';
const _starredEntitiesKey = 'starred_entities';
const _starredContactsKey = 'starred_contacts';

String entityKey(String type, String id) => '$type:$id';

({String type, String id})? parseEntityKey(String key) {
  final index = key.indexOf(':');
  if (index <= 0 || index >= key.length - 1) return null;
  return (type: key.substring(0, index), id: key.substring(index + 1));
}

class RecentEntities extends StateNotifier<List<String>> {
  RecentEntities() : super(const []) {
    _load();
  }

  Future<void> _load() async {
    final prefs = await SharedPreferences.getInstance();
    final stored = prefs.getStringList(_recentEntitiesKey) ?? const [];
    state = List<String>.from(stored);
  }

  Future<void> record(String key) async {
    final updated = <String>[key, ...state.where((value) => value != key)];
    if (updated.length > 20) {
      updated.removeRange(20, updated.length);
    }
    state = updated;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setStringList(_recentEntitiesKey, updated);
  }

  Future<void> clear() async {
    state = const [];
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_recentEntitiesKey);
  }
}

final recentEntitiesProvider =
    StateNotifierProvider<RecentEntities, List<String>>((ref) {
  return RecentEntities();
});

class RecentContacts extends StateNotifier<List<String>> {
  RecentContacts() : super(const []) {
    _load();
  }

  Future<void> _load() async {
    final prefs = await SharedPreferences.getInstance();
    final stored = prefs.getStringList(_recentContactsKey) ?? const [];
    state = List<String>.from(stored);
  }

  Future<void> record(String contactId) async {
    final updated = <String>[contactId, ...state.where((value) => value != contactId)];
    if (updated.length > 20) {
      updated.removeRange(20, updated.length);
    }
    state = updated;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setStringList(_recentContactsKey, updated);
  }

  Future<void> clear() async {
    state = const [];
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_recentContactsKey);
  }
}

final recentContactsProvider =
    StateNotifierProvider<RecentContacts, List<String>>((ref) {
  return RecentContacts();
});

class StarredEntities extends StateNotifier<Set<String>> {
  StarredEntities() : super(<String>{}) {
    _load();
  }

  Future<void> _load() async {
    final prefs = await SharedPreferences.getInstance();
    final stored = prefs.getStringList(_starredEntitiesKey) ?? const [];
    state = stored.toSet();
  }

  Future<void> toggle(String key) async {
    final updated = {...state};
    if (updated.contains(key)) {
      updated.remove(key);
    } else {
      updated.add(key);
    }
    state = updated;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setStringList(_starredEntitiesKey, updated.toList());
  }

  Future<void> remove(String key) async {
    final updated = {...state}..remove(key);
    state = updated;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setStringList(_starredEntitiesKey, updated.toList());
  }
}

final starredEntitiesProvider =
    StateNotifierProvider<StarredEntities, Set<String>>((ref) {
  return StarredEntities();
});

class StarredContacts extends StateNotifier<Set<String>> {
  StarredContacts() : super(<String>{}) {
    _load();
  }

  Future<void> _load() async {
    final prefs = await SharedPreferences.getInstance();
    final stored = prefs.getStringList(_starredContactsKey) ?? const [];
    state = stored.toSet();
  }

  Future<void> toggle(String contactId) async {
    final updated = {...state};
    if (updated.contains(contactId)) {
      updated.remove(contactId);
    } else {
      updated.add(contactId);
    }
    state = updated;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setStringList(_starredContactsKey, updated.toList());
  }

  Future<void> remove(String contactId) async {
    final updated = {...state}..remove(contactId);
    state = updated;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setStringList(_starredContactsKey, updated.toList());
  }
}

final starredContactsProvider =
    StateNotifierProvider<StarredContacts, Set<String>>((ref) {
  return StarredContacts();
});
