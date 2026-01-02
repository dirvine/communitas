import 'dart:convert';
import 'package:http/http.dart' as http;

/// HTTP client for communicating with the Communitas Bridge server.
///
/// The bridge server provides a REST API for all Communitas operations,
/// allowing the Flutter web app to interact with the Rust backend.
class BridgeClient {
  final String baseUrl;
  final http.Client _client;

  BridgeClient({required this.baseUrl}) : _client = http.Client();

  /// Close the HTTP client when done.
  void dispose() {
    _client.close();
  }

  // ============================================================
  // Core Initialization
  // ============================================================

  /// Initialize the bridge with a four-word identity.
  Future<bool> initialize(
      String fourWords, String displayName, String deviceName) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/core/initialize'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'four_words': fourWords,
        'display_name': displayName,
        'device_name': deviceName,
      }),
    );
    return response.statusCode == 200;
  }

  /// Check if the bridge is initialized and ready.
  Future<bool> checkStatus() async {
    try {
      final response = await _client.get(Uri.parse('$baseUrl/api/core/status'));
      return response.statusCode == 200;
    } catch (e) {
      return false;
    }
  }

  /// Get current session info from the bridge.
  Future<Map<String, dynamic>?> getSessionInfo() async {
    try {
      final response =
          await _client.get(Uri.parse('$baseUrl/api/core/session'));
      if (response.statusCode == 200) {
        return jsonDecode(response.body);
      }
      return null;
    } catch (e) {
      return null;
    }
  }

  // ============================================================
  // Identity Generation
  // ============================================================

  /// Generate a new four-word identity.
  Future<String?> generateIdentity() async {
    try {
      final response =
          await _client.get(Uri.parse('$baseUrl/api/identity/generate'));
      if (response.statusCode == 200) {
        final data = jsonDecode(response.body);
        return data['four_words'];
      }
      return null;
    } catch (e) {
      return null;
    }
  }

  // ============================================================
  // Channels
  // ============================================================

  /// Create a new channel.
  Future<Map<String, dynamic>> createChannel(
      String name, String description) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/channels'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({'name': name, 'description': description}),
    );
    return jsonDecode(response.body);
  }

  /// List all channels.
  Future<List<dynamic>> listChannels() async {
    final response = await _client.get(Uri.parse('$baseUrl/api/channels'));
    final data = jsonDecode(response.body);
    return data['channels'] ?? [];
  }

  /// Get messages for a channel.
  Future<List<dynamic>> getChannelMessages(String channelId) async {
    final response = await _client
        .get(Uri.parse('$baseUrl/api/channels/$channelId/messages'));
    final data = jsonDecode(response.body);
    return data['messages'] ?? [];
  }

  /// Send a message to a channel.
  Future<Map<String, dynamic>> sendMessage(
      String channelId, String content) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/channels/$channelId/messages'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({'content': content}),
    );
    return jsonDecode(response.body);
  }

  // ============================================================
  // Organizations
  // ============================================================

  /// Create a new organization.
  Future<Map<String, dynamic>> createOrganisation(
      String name, String description) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/organisations'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({'name': name, 'description': description}),
    );
    return jsonDecode(response.body);
  }

  /// List all organizations.
  Future<List<dynamic>> listOrganisations() async {
    final response =
        await _client.get(Uri.parse('$baseUrl/api/organisations'));
    final data = jsonDecode(response.body);
    return data['organisations'] ?? [];
  }

  /// Get organization details.
  Future<Map<String, dynamic>?> getOrganisation(String orgId) async {
    try {
      final response =
          await _client.get(Uri.parse('$baseUrl/api/organisations/$orgId'));
      if (response.statusCode == 200) {
        return jsonDecode(response.body);
      }
      return null;
    } catch (e) {
      return null;
    }
  }

  // ============================================================
  // Projects
  // ============================================================

  /// Create a new project.
  Future<Map<String, dynamic>> createProject(
      String name, String description, String? parentId) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/projects'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'name': name,
        'description': description,
        'parent_id': parentId,
      }),
    );
    return jsonDecode(response.body);
  }

  /// List all projects.
  Future<List<dynamic>> listProjects() async {
    final response = await _client.get(Uri.parse('$baseUrl/api/projects'));
    final data = jsonDecode(response.body);
    return data['projects'] ?? [];
  }

  // ============================================================
  // Groups
  // ============================================================

  /// Create a new group.
  Future<Map<String, dynamic>> createGroup(
      String name, String description) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/groups'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({'name': name, 'description': description}),
    );
    return jsonDecode(response.body);
  }

  /// List all groups.
  Future<List<dynamic>> listGroups() async {
    final response = await _client.get(Uri.parse('$baseUrl/api/groups'));
    final data = jsonDecode(response.body);
    return data['groups'] ?? [];
  }

  // ============================================================
  // Entities (Generic)
  // ============================================================

  /// List all entities.
  Future<List<dynamic>> listEntities() async {
    final response = await _client.get(Uri.parse('$baseUrl/api/entities'));
    final data = jsonDecode(response.body);
    return data['entities'] ?? [];
  }

  /// Get entity details.
  Future<Map<String, dynamic>?> getEntity(String entityId) async {
    try {
      final response =
          await _client.get(Uri.parse('$baseUrl/api/entities/$entityId'));
      if (response.statusCode == 200) {
        return jsonDecode(response.body);
      }
      return null;
    } catch (e) {
      return null;
    }
  }

  /// Join an entity.
  Future<bool> joinEntity(String entityId) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/entities/join'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({'entity_id': entityId}),
    );
    return response.statusCode == 200;
  }

  // ============================================================
  // Members
  // ============================================================

  /// Get members of an entity.
  Future<List<dynamic>> getMembers(String entityType, String entityId) async {
    final response = await _client
        .get(Uri.parse('$baseUrl/api/$entityType/$entityId/members'));
    final data = jsonDecode(response.body);
    return data['members'] ?? [];
  }

  /// Add a member to an entity.
  Future<bool> addMember(
      String entityType, String entityId, String fourWords, String role) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/$entityType/$entityId/members'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({'four_words': fourWords, 'role': role}),
    );
    return response.statusCode == 200;
  }

  // ============================================================
  // Virtual Disk Storage
  // ============================================================

  /// List files in an entity's virtual disk.
  Future<List<dynamic>> listFiles(
      String entityId, String diskType, String path) async {
    final response = await _client.get(
      Uri.parse(
          '$baseUrl/api/entities/$entityId/storage/$diskType/files?path=${Uri.encodeComponent(path)}'),
    );
    final data = jsonDecode(response.body);
    return data['files'] ?? [];
  }

  /// Upload a file to an entity's virtual disk.
  Future<Map<String, dynamic>> uploadFile(
      String entityId, String diskType, String path, String content) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/entities/$entityId/storage/$diskType/files'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({'path': path, 'content': content}),
    );
    return jsonDecode(response.body);
  }

  /// Download a file from an entity's virtual disk.
  Future<String?> downloadFile(
      String entityId, String diskType, String path) async {
    try {
      final response = await _client.get(
        Uri.parse(
            '$baseUrl/api/entities/$entityId/storage/$diskType/files/${Uri.encodeComponent(path)}'),
      );
      if (response.statusCode == 200) {
        return response.body;
      }
      return null;
    } catch (e) {
      return null;
    }
  }

  /// Delete a file from an entity's virtual disk.
  Future<bool> deleteFile(
      String entityId, String diskType, String path) async {
    final response = await _client.delete(
      Uri.parse(
          '$baseUrl/api/entities/$entityId/storage/$diskType/files/${Uri.encodeComponent(path)}'),
    );
    return response.statusCode == 200;
  }

  /// Get disk statistics for an entity.
  Future<Map<String, dynamic>?> getDiskStats(
      String entityId, String diskType) async {
    try {
      final response = await _client.get(
        Uri.parse(
            '$baseUrl/api/entities/$entityId/storage/$diskType/stats'),
      );
      if (response.statusCode == 200) {
        return jsonDecode(response.body);
      }
      return null;
    } catch (e) {
      return null;
    }
  }

  // ============================================================
  // Kanban Boards
  // ============================================================

  /// Create a Kanban board for a project.
  Future<Map<String, dynamic>> createBoard(
      String projectId, String name) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/projects/$projectId/boards'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({'name': name}),
    );
    return jsonDecode(response.body);
  }

  /// List Kanban boards for a project.
  Future<List<dynamic>> listBoards(String projectId) async {
    final response =
        await _client.get(Uri.parse('$baseUrl/api/projects/$projectId/boards'));
    final data = jsonDecode(response.body);
    return data['boards'] ?? [];
  }

  /// Create a column in a Kanban board.
  Future<Map<String, dynamic>> createColumn(
      String boardId, String name, int position) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/boards/$boardId/columns'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({'name': name, 'position': position}),
    );
    return jsonDecode(response.body);
  }

  /// Create a card in a Kanban board.
  Future<Map<String, dynamic>> createCard(
      String boardId, String title, String description, String columnId) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/boards/$boardId/cards'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'title': title,
        'description': description,
        'column_id': columnId,
      }),
    );
    return jsonDecode(response.body);
  }

  // ============================================================
  // P2P Network
  // ============================================================

  /// Start P2P networking.
  Future<bool> startNetworking() async {
    final response =
        await _client.post(Uri.parse('$baseUrl/api/network/start'));
    return response.statusCode == 200;
  }

  /// Get network connection info.
  Future<Map<String, dynamic>?> getConnectionInfo() async {
    try {
      final response =
          await _client.get(Uri.parse('$baseUrl/api/network/connection-info'));
      if (response.statusCode == 200) {
        return jsonDecode(response.body);
      }
      return null;
    } catch (e) {
      return null;
    }
  }

  /// List connected peers.
  Future<List<dynamic>> getPeers() async {
    final response =
        await _client.get(Uri.parse('$baseUrl/api/network/peers'));
    final data = jsonDecode(response.body);
    return data['peers'] ?? [];
  }

  /// Connect to a peer by four-word address.
  Future<bool> connectToPeer(String fourWords) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/network/connect'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({'four_words': fourWords}),
    );
    return response.statusCode == 200;
  }

  /// Disconnect from a peer.
  Future<bool> disconnectFromPeer(String fourWords) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/network/disconnect'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({'four_words': fourWords}),
    );
    return response.statusCode == 200;
  }

  // ============================================================
  // Contacts
  // ============================================================

  /// Create a contact.
  Future<Map<String, dynamic>> createContact(
      String fourWords, String displayName) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/contacts'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({'four_words': fourWords, 'display_name': displayName}),
    );
    return jsonDecode(response.body);
  }

  /// List all contacts.
  Future<List<dynamic>> listContacts() async {
    final response = await _client.get(Uri.parse('$baseUrl/api/contacts'));
    final data = jsonDecode(response.body);
    return data['contacts'] ?? [];
  }

  /// Get favorite contacts.
  Future<List<dynamic>> getFavoriteContacts() async {
    final response =
        await _client.get(Uri.parse('$baseUrl/api/contacts/favourites'));
    final data = jsonDecode(response.body);
    return data['contacts'] ?? [];
  }

  // ============================================================
  // Website Publishing
  // ============================================================

  /// Create a website for an entity.
  Future<Map<String, dynamic>> createWebsite(
      String entityId, String title, String content) async {
    final response = await _client.post(
      Uri.parse('$baseUrl/api/entities/$entityId/website'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({'title': title, 'content': content}),
    );
    return jsonDecode(response.body);
  }

  /// Get website for an entity.
  Future<Map<String, dynamic>?> getWebsite(String entityId) async {
    try {
      final response = await _client
          .get(Uri.parse('$baseUrl/api/entities/$entityId/website'));
      if (response.statusCode == 200) {
        return jsonDecode(response.body);
      }
      return null;
    } catch (e) {
      return null;
    }
  }
}
