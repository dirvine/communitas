/// Demo mode data for Communitas.
/// Pre-populated sample data for testing and demonstrations.
class DemoData {
  DemoData._();

  /// Demo user identity
  static const demoIdentity = DemoIdentity(
    fourWords: 'ocean-forest-moon-star',
    displayName: 'Demo User',
  );

  /// Demo contacts
  static const contacts = [
    DemoContact(
      fourWords: 'river-mountain-sun-cloud',
      displayName: 'Alice',
      status: 'online',
    ),
    DemoContact(
      fourWords: 'wind-valley-tree-stone',
      displayName: 'Bob',
      status: 'away',
    ),
    DemoContact(
      fourWords: 'fire-lake-sky-earth',
      displayName: 'Local Contact',
      status: 'offline',
    ),
  ];

  /// Demo organizations
  static const organizations = [
    DemoEntity(
      id: 'org-1',
      type: 'organization',
      name: 'Saorsa Labs',
      role: 'owner',
      description: 'Building the decentralized future',
      memberCount: 12,
    ),
    DemoEntity(
      id: 'org-2',
      type: 'organization',
      name: 'Open Source Collective',
      role: 'member',
      description: 'Open source enthusiasts community',
      memberCount: 156,
    ),
  ];

  /// Demo projects
  static const projects = [
    DemoEntity(
      id: 'proj-1',
      type: 'project',
      name: 'Communitas Flutter',
      role: 'owner',
      description: 'Cross-platform Flutter GUI for Communitas',
      memberCount: 5,
      parentId: 'org-1',
    ),
    DemoEntity(
      id: 'proj-2',
      type: 'project',
      name: 'Documentation',
      role: 'admin',
      description: 'Project documentation and guides',
      memberCount: 3,
      parentId: 'org-1',
    ),
  ];

  /// Demo channels
  static const channels = [
    DemoEntity(
      id: 'chan-1',
      type: 'channel',
      name: 'general',
      role: 'member',
      description: 'General discussion',
      memberCount: 12,
      parentId: 'org-1',
    ),
    DemoEntity(
      id: 'chan-2',
      type: 'channel',
      name: 'engineering',
      role: 'member',
      description: 'Engineering team discussions',
      memberCount: 8,
      parentId: 'org-1',
    ),
    DemoEntity(
      id: 'chan-3',
      type: 'channel',
      name: 'design',
      role: 'admin',
      description: 'Design and UX discussions',
      memberCount: 4,
      parentId: 'org-1',
    ),
  ];

  /// Demo groups
  static const groups = [
    DemoEntity(
      id: 'group-1',
      type: 'group',
      name: 'Core Team',
      role: 'member',
      description: 'Core development team',
      memberCount: 5,
    ),
  ];

  /// Demo messages
  static const messages = [
    DemoMessage(
      id: 'msg-1',
      senderId: 'river-mountain-sun-cloud',
      senderName: 'Alice',
      content: 'Hey everyone! How is the Flutter migration going?',
      timestamp: '10:30 AM',
      reactions: {'thumbsup': 2, 'heart': 1},
    ),
    DemoMessage(
      id: 'msg-2',
      senderId: 'ocean-forest-moon-star',
      senderName: 'Demo User',
      content: 'Great progress! Just finished the theme system.',
      timestamp: '10:32 AM',
      reactions: {},
    ),
    DemoMessage(
      id: 'msg-3',
      senderId: 'wind-valley-tree-stone',
      senderName: 'Bob',
      content: 'Nice! Can you share some screenshots?',
      timestamp: '10:35 AM',
      reactions: {},
      hasThread: true,
      threadReplyCount: 3,
    ),
  ];

  /// Demo kanban cards
  static const kanbanCards = [
    DemoKanbanCard(
      id: 'card-1',
      title: 'Implement login screen',
      description: 'Create login screen with vault selection',
      column: 'done',
      priority: 'high',
      assignee: 'Alice',
    ),
    DemoKanbanCard(
      id: 'card-2',
      title: 'Add sidebar navigation',
      description: 'Implement adaptive sidebar for desktop/mobile',
      column: 'in_progress',
      priority: 'critical',
      assignee: 'Demo User',
    ),
    DemoKanbanCard(
      id: 'card-3',
      title: 'Set up flutter_rust_bridge',
      description: 'Configure FFI bindings to Rust backend',
      column: 'to_do',
      priority: 'high',
      assignee: null,
    ),
    DemoKanbanCard(
      id: 'card-4',
      title: 'Voice call integration',
      description: 'Integrate flutter_webrtc for voice calls',
      column: 'backlog',
      priority: 'medium',
      assignee: 'Bob',
    ),
    DemoKanbanCard(
      id: 'card-5',
      title: 'Document API usage',
      description: 'Write documentation for FFI API',
      column: 'review',
      priority: 'low',
      assignee: 'Alice',
    ),
  ];
}

/// Demo identity model
class DemoIdentity {
  final String fourWords;
  final String displayName;

  const DemoIdentity({
    required this.fourWords,
    required this.displayName,
  });
}

/// Demo contact model
class DemoContact {
  final String fourWords;
  final String displayName;
  final String status;

  const DemoContact({
    required this.fourWords,
    required this.displayName,
    required this.status,
  });
}

/// Demo entity model (org, project, channel, group)
class DemoEntity {
  final String id;
  final String type;
  final String name;
  final String role;
  final String description;
  final int memberCount;
  final String? parentId;

  const DemoEntity({
    required this.id,
    required this.type,
    required this.name,
    required this.role,
    required this.description,
    required this.memberCount,
    this.parentId,
  });
}

/// Demo message model
class DemoMessage {
  final String id;
  final String senderId;
  final String senderName;
  final String content;
  final String timestamp;
  final Map<String, int> reactions;
  final bool hasThread;
  final int threadReplyCount;

  const DemoMessage({
    required this.id,
    required this.senderId,
    required this.senderName,
    required this.content,
    required this.timestamp,
    required this.reactions,
    this.hasThread = false,
    this.threadReplyCount = 0,
  });
}

/// Demo kanban card model
class DemoKanbanCard {
  final String id;
  final String title;
  final String description;
  final String column;
  final String priority;
  final String? assignee;

  const DemoKanbanCard({
    required this.id,
    required this.title,
    required this.description,
    required this.column,
    required this.priority,
    this.assignee,
  });
}
