import SwiftUI
import CommunitasKit

// MARK: - Active View Routing

/// Represents the active view in the main content area
public enum ActiveView: Equatable {
    case home
    case chat(entityType: String, entityId: String, entityName: String)
    case contactChat(fourWords: String, displayName: String?)
    case drive(entityType: String, entityId: String)
    case call(peerFourWords: String)
    case project(projectId: String)
    case networkPanel

    public static func == (lhs: ActiveView, rhs: ActiveView) -> Bool {
        switch (lhs, rhs) {
        case (.home, .home):
            return true
        case let (.chat(t1, i1, _), .chat(t2, i2, _)):
            return t1 == t2 && i1 == i2
        case let (.contactChat(f1, _), .contactChat(f2, _)):
            return f1 == f2
        case let (.drive(t1, i1), .drive(t2, i2)):
            return t1 == t2 && i1 == i2
        case let (.call(p1), .call(p2)):
            return p1 == p2
        case let (.project(p1), .project(p2)):
            return p1 == p2
        case (.networkPanel, .networkPanel):
            return true
        default:
            return false
        }
    }
}

// MARK: - File System Types

public struct FileItem: Identifiable, Hashable {
    public let id: String
    public let name: String
    public let path: String
    public let isDirectory: Bool
    public let sizeBytes: UInt64
    public let modifiedAt: Date

    public init(id: String, name: String, path: String, isDirectory: Bool, sizeBytes: UInt64, modifiedAt: Date) {
        self.id = id
        self.name = name
        self.path = path
        self.isDirectory = isDirectory
        self.sizeBytes = sizeBytes
        self.modifiedAt = modifiedAt
    }

    public static func == (lhs: FileItem, rhs: FileItem) -> Bool {
        lhs.id == rhs.id
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }
}

// MARK: - Document Types

public struct DocumentItem: Identifiable, Hashable, Codable {
    public let id: String
    public var name: String
    public var content: String
    public let createdAt: Date
    public var modifiedAt: Date
    public let authorFourWords: String

    public init(id: String, name: String, content: String, createdAt: Date, modifiedAt: Date, authorFourWords: String) {
        self.id = id
        self.name = name
        self.content = content
        self.createdAt = createdAt
        self.modifiedAt = modifiedAt
        self.authorFourWords = authorFourWords
    }

    public static func == (lhs: DocumentItem, rhs: DocumentItem) -> Bool {
        lhs.id == rhs.id
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }
}

// MARK: - Contact Types

public struct ContactItem: Identifiable, Hashable, Codable {
    public let id: String
    public let fourWords: String?              // Optional: nil for local-only contacts
    public var displayName: String?
    public var isFavourite: Bool
    public var isOnline: Bool
    public var lastSeen: Date?
    // Local-only / Network-linked tracking
    public var isLocalOnly: Bool               // true = placeholder, false = network-linked
    public var linkedAt: Date?                 // When contact was linked to network identity
    public var lastSyncAt: Date?               // Last successful sync timestamp
    // Endpoint tracking for direct reconnection
    public var lastSeenEndpoint: String?       // Four-word encoded IP:port
    public var endpointUpdatedAt: Date?        // When endpoint was last updated
    public var endpointSuccessCount: UInt32    // Successful connections via endpoint
    public var endpointFailureCount: UInt32    // Failed connections via endpoint

    public init(
        id: String,
        fourWords: String?,
        displayName: String?,
        isFavourite: Bool,
        isOnline: Bool,
        lastSeen: Date?,
        isLocalOnly: Bool = false,
        linkedAt: Date? = nil,
        lastSyncAt: Date? = nil,
        lastSeenEndpoint: String? = nil,
        endpointUpdatedAt: Date? = nil,
        endpointSuccessCount: UInt32 = 0,
        endpointFailureCount: UInt32 = 0
    ) {
        self.id = id
        self.fourWords = fourWords
        self.displayName = displayName
        self.isFavourite = isFavourite
        self.isOnline = isOnline
        self.lastSeen = lastSeen
        self.isLocalOnly = isLocalOnly
        self.linkedAt = linkedAt
        self.lastSyncAt = lastSyncAt
        self.lastSeenEndpoint = lastSeenEndpoint
        self.endpointUpdatedAt = endpointUpdatedAt
        self.endpointSuccessCount = endpointSuccessCount
        self.endpointFailureCount = endpointFailureCount
    }

    /// Returns effective name for display: displayName if set, otherwise fourWords or "Unknown"
    public var effectiveName: String {
        displayName ?? fourWords ?? "Unknown Contact"
    }

    /// Returns true if this contact is linked to a network identity
    public var isLinked: Bool {
        fourWords != nil && !isLocalOnly
    }

    public static func == (lhs: ContactItem, rhs: ContactItem) -> Bool {
        lhs.id == rhs.id
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }
}

// MARK: - Kanban Types

public struct KanbanCard: Identifiable, Codable, Hashable {
    public let id: String
    public var title: String
    public var cardDescription: String?
    public var column: String
    public var assignee: String?
    public var priority: Int  // 0 = low, 1 = normal, 2 = high
    public let createdAt: Date
    public var updatedAt: Date
    public var sortOrder: Int  // For ordering within column
    public var commentCount: Int  // Number of comments on this card

    public init(
        id: String = UUID().uuidString,
        title: String,
        cardDescription: String? = nil,
        column: String = "backlog",
        assignee: String? = nil,
        priority: Int = 1,
        createdAt: Date = Date(),
        updatedAt: Date = Date(),
        sortOrder: Int = 0,
        commentCount: Int = 0
    ) {
        self.id = id
        self.title = title
        self.cardDescription = cardDescription
        self.column = column
        self.assignee = assignee
        self.priority = priority
        self.createdAt = createdAt
        self.updatedAt = updatedAt
        self.sortOrder = sortOrder
        self.commentCount = commentCount
    }

    public static func == (lhs: KanbanCard, rhs: KanbanCard) -> Bool {
        lhs.id == rhs.id
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }
}

// MARK: - Network Configuration

public struct NetworkConfig {
    public static let defaultBootstrapAddress = "138.197.29.195:4433"
    public static let defaultPort: UInt16 = 0  // 0 = auto-select
}

// MARK: - App State

public class AppState: ObservableObject {
    @Published public var fourWords: String = "ocean-forest-moon-star"
    @Published public var displayName: String = ""
    @Published public var deviceName: String = ""
    @Published public var isInitialized: Bool = false
    @Published public var isNetworking: Bool = false
    @Published public var externalAddressDetectionFailed: Bool = false
    @Published public var isAuthenticated: Bool = false
    @Published public var errorMessage: String?
    @Published public var showError: Bool = false
    @Published public var entities: [SwiftEntity] = []
    @Published public var messages: [String: [SwiftMessage]] = [:]
    @Published public var unreadCounts: [String: Int] = [:]
    @Published public var files: [String: [FileItem]] = [:]
    @Published public var documents: [String: [DocumentItem]] = [:]
    @Published public var contacts: [ContactItem] = []
    @Published public var kanbanCards: [String: [KanbanCard]] = [:]
    @Published public var isPresenceActive: Bool = false

    // MARK: - Kanban Board Tracking (Rust backend)
    /// Maps entity/project ID to Kanban board ID
    private var kanbanBoardIds: [String: String] = [:]
    /// Maps board ID to column name -> column ID
    private var kanbanColumnMappings: [String: [String: String]] = [:]
    @Published public var connectionIdentity: String?
    @Published public var bootstrapAddress: String = NetworkConfig.defaultBootstrapAddress

    // MARK: - Thread State (Slack-style threading)
    /// Currently selected thread parent message
    @Published public var selectedThreadMessage: SwiftMessage?
    /// Thread messages keyed by parent message ID
    @Published public var threadMessages: [String: [SwiftMessage]] = [:]
    /// Thread reply counts keyed by parent message ID
    @Published public var threadReplyCounts: [String: Int] = [:]
    /// Whether the thread panel is showing
    @Published public var isThreadPanelOpen: Bool = false

    /// Active view for navigation routing
    @Published public var activeView: ActiveView = .home

    /// Discovery client - used only to list vaults before login
    private var discoveryClient: CommunitasClient?

    /// Active client - used after successful login
    public var client: CommunitasClient?

    /// Timer for polling contact requests
    private var contactPollTimer: Timer?

    /// Initialize discovery client for vault listing
    public init() {
        initializeDiscoveryClient()
    }

    private func initializeDiscoveryClient() {
        let storagePath = Self.getStorageDirectory().path
        do {
            // Create a minimal client just for vault discovery
            // Using a placeholder identity that won't be used for anything else
            discoveryClient = try CommunitasClient(
                fourWords: "discovery-client-temp-init",
                displayName: "Discovery",
                deviceName: Host.current().localizedName ?? "Mac",
                storagePath: storagePath
            )
            print("[Communitas] Discovery client initialized for vault listing")
        } catch {
            print("[Communitas] Warning: Could not initialize discovery client: \(error)")
            discoveryClient = nil
        }
    }

    /// List all vaults in the storage directory (for identity picker)
    public func listAllVaults() -> [SwiftVaultInfo] {
        // First try discovery client, then fall back to active client
        let clientToUse = discoveryClient ?? client
        guard let client = clientToUse else {
            print("[Communitas] No client available for vault listing")
            return []
        }

        do {
            let vaults = try client.authListVaults()
            print("[Communitas] Found \(vaults.count) vault(s)")
            return vaults
        } catch {
            print("[Communitas] Error listing vaults: \(error)")
            return []
        }
    }

    /// Check for existing session on startup
    /// Called on app launch to see if user is already authenticated
    public func checkExistingSession() {
        // Without a client, we can't check for sessions
        // User will start at welcome screen and log in or create identity
        guard let client = client else {
            // No session - show authentication view
            return
        }

        // Check for existing session from a previous run
        if let session = client.authGetCurrentSession() {
            self.fourWords = session.fourWords
            self.displayName = session.displayName
            self.isAuthenticated = true
            self.isInitialized = true
        }
    }

    /// Get the fixed storage directory for all identities
    /// Multi-identity is now handled via vaults within this single directory
    public static func getStorageDirectory() -> URL {
        // Fixed location: ~/Library/Application Support/Communitas/
        // Multiple identities stored as separate vaults within
        let baseDir = FileManager.default
            .urls(for: .applicationSupportDirectory, in: .userDomainMask)
            .first?
            .appendingPathComponent("Communitas")
            ?? FileManager.default.temporaryDirectory.appendingPathComponent("communitas-data")

        // Ensure directory exists
        try? FileManager.default.createDirectory(at: baseDir, withIntermediateDirectories: true)

        return baseDir
    }

    /// Initialize client with provided credentials (called from AuthenticationView)
    public func initializeClientWithCredentials(fourWords: String, displayName: String) {
        do {
            let storageDir = Self.getStorageDirectory()
            try? FileManager.default.createDirectory(at: storageDir, withIntermediateDirectories: true)

            client = try CommunitasClient(
                fourWords: fourWords,
                displayName: displayName,
                deviceName: Host.current().localizedName ?? "Mac",
                storagePath: storageDir.path
            )
        } catch {
            errorMessage = "Failed to initialize client: \(error.localizedDescription)"
        }
    }

    /// Logout and return to auth screen
    public func logout() {
        do {
            try client?.authLogout()
        } catch {
            print("Logout error: \(error)")
        }
        isAuthenticated = false
        isInitialized = false
        fourWords = ""
        displayName = ""
        entities = []
        activeView = .home
    }

    public func initialize() {
        do {
            // Create storage directory
            let tempDir = FileManager.default.temporaryDirectory.appendingPathComponent("communitas-data")
            try? FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)

            // Create client using sync API (tokio runtime handles async internally)
            client = try CommunitasClient(
                fourWords: fourWords,
                displayName: "macOS User",
                deviceName: "Mac",
                storagePath: tempDir.path
            )

            // Get profile
            if let profile = client?.getProfile() {
                self.displayName = profile.displayName
                self.deviceName = profile.deviceName
                self.fourWords = profile.fourWords
            }

            // Check networking status
            isNetworking = client?.isNetworkingActive() ?? false

            // Load entities
            loadEntities()

            isInitialized = true
            errorMessage = nil

        } catch {
            errorMessage = error.localizedDescription
            isInitialized = false
        }
    }

    public func loadEntities() {
        do {
            entities = try client?.entityList() ?? []
        } catch {
            print("Error loading entities: \(error)")
        }
    }

    public func createEntity(name: String, type: SwiftEntityType, description: String?, parentOrgId: String? = nil, networkFourWords: String? = nil) {
        // Guard against nil client - this would silently fail with optional chaining
        guard let client = client else {
            print("[Communitas] ERROR: Cannot create entity - client is nil (not authenticated)")
            errorMessage = "Not authenticated. Please log in first."
            showError = true
            return
        }

        do {
            print("[Communitas] Creating entity '\(name)' of type \(type)")
            let entity = try client.entityCreate(
                name: name,
                entityType: type,
                description: description,
                parentOrgId: parentOrgId
            )
            print("[Communitas] Entity created successfully: \(entity.id)")

            // If network four-words provided, link the entity to network
            if let fourWords = networkFourWords {
                print("[Communitas] Linking entity to network: \(fourWords)")
                linkEntity(entityId: entity.id, fourWords: fourWords)
            }

            loadEntities()
        } catch {
            print("[Communitas] ERROR creating entity: \(error)")
            errorMessage = "Failed to create entity: \(error.localizedDescription)"
            showError = true
        }
    }

    /// Create a local-only entity (not linked to network identity)
    public func createLocalEntity(name: String, type: SwiftEntityType, description: String?, parentOrgId: String? = nil) {
        // Guard against nil client - this would silently fail with optional chaining
        guard let client = client else {
            print("[Communitas] ERROR: Cannot create entity - client is nil (not authenticated)")
            errorMessage = "Not authenticated. Please log in first."
            showError = true
            return
        }

        do {
            print("[Communitas] Creating local-only entity '\(name)' of type \(type)")
            let entity = try client.entityCreateLocal(
                name: name,
                entityType: type,
                description: description
            )
            print("[Communitas] Local entity created successfully: \(entity.id) (local-only)")
            loadEntities()
        } catch {
            print("[Communitas] ERROR creating local entity: \(error)")
            errorMessage = "Failed to create local entity: \(error.localizedDescription)"
            showError = true
        }
    }

    public func deleteEntity(id: String) {
        // Entity deletion not yet implemented in Rust bindings
        // For now, just remove from local list (will reappear on reload)
        entities.removeAll { $0.id == id }
        // Show message that this is a local-only operation
        print("Note: Entity deletion is not yet persisted - entity \(id) removed from local view only")
    }

    public func startNetworking(port: UInt16? = nil) {
        do {
            let _ = try client?.gossipStart(port: port)
            isNetworking = true

            // Start contact polling timer (poll every 5 seconds)
            startContactPolling()
        } catch {
            errorMessage = "Failed to start networking: \(error.localizedDescription)"
        }
    }

    public func stopNetworking() {
        do {
            // Stop contact polling
            stopContactPolling()

            try client?.gossipStop()
            isNetworking = false
        } catch {
            errorMessage = "Failed to stop networking: \(error.localizedDescription)"
        }
    }

    // MARK: - Messaging Methods

    public func loadMessages(for entityId: String) {
        do {
            let msgs = try client?.messageGetForEntity(entityId: entityId, limit: 100, beforeId: nil) ?? []
            messages[entityId] = msgs.sorted { $0.createdAt < $1.createdAt }
        } catch {
            print("Error loading messages for \(entityId): \(error)")
        }
    }

    public func sendMessage(to entityId: String, text: String, replyToId: String? = nil) {
        guard !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        do {
            let messageId = try client?.messageSend(entityId: entityId, text: text, replyToId: replyToId)
            if messageId != nil {
                // Reload messages to include the new one
                loadMessages(for: entityId)
            }
        } catch {
            errorMessage = "Failed to send message: \(error.localizedDescription)"
        }
    }

    public func getMessages(for entityId: String) -> [SwiftMessage] {
        return messages[entityId] ?? []
    }

    // MARK: - Thread Methods (Slack-style threading)

    /// Open a thread panel for a message
    public func openThread(for message: SwiftMessage) {
        selectedThreadMessage = message
        isThreadPanelOpen = true
        loadThreadMessages(entityId: message.entityId, parentMessageId: message.id)
    }

    /// Close the thread panel
    public func closeThread() {
        isThreadPanelOpen = false
        selectedThreadMessage = nil
    }

    /// Load thread replies for a parent message
    public func loadThreadMessages(entityId: String, parentMessageId: String) {
        do {
            let replies = try client?.messageGetThread(entityId: entityId, parentMessageId: parentMessageId) ?? []
            threadMessages[parentMessageId] = replies.sorted { $0.createdAt < $1.createdAt }
            threadReplyCounts[parentMessageId] = replies.count
        } catch {
            print("Error loading thread messages for \(parentMessageId): \(error)")
        }
    }

    /// Send a reply in a thread
    public func sendThreadReply(to parentMessage: SwiftMessage, text: String) {
        guard !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        do {
            let messageId = try client?.messageSend(
                entityId: parentMessage.entityId,
                text: text,
                replyToId: parentMessage.id
            )
            if messageId != nil {
                // Reload thread messages to include the new reply
                loadThreadMessages(entityId: parentMessage.entityId, parentMessageId: parentMessage.id)
                // Also reload main messages to update any thread indicators
                loadMessages(for: parentMessage.entityId)
            }
        } catch {
            errorMessage = "Failed to send thread reply: \(error.localizedDescription)"
        }
    }

    /// Get the number of replies for a message
    public func getReplyCount(for messageId: String) -> Int {
        return threadReplyCounts[messageId] ?? 0
    }

    /// Get thread messages for a parent message
    public func getThreadMessages(for parentMessageId: String) -> [SwiftMessage] {
        return threadMessages[parentMessageId] ?? []
    }

    /// Update reply counts for all messages in an entity
    public func updateThreadReplyCounts(for entityId: String) {
        let entityMessages = messages[entityId] ?? []
        for message in entityMessages {
            // Only check messages that could be thread parents (not replies themselves)
            if message.replyToId == nil {
                do {
                    let replies = try client?.messageGetThread(entityId: entityId, parentMessageId: message.id) ?? []
                    if !replies.isEmpty {
                        threadReplyCounts[message.id] = replies.count
                    }
                } catch {
                    // Ignore errors for reply count updates
                }
            }
        }
    }

    // MARK: - Direct Messaging Methods (for contact-to-contact chat)

    /// Load direct messages with a peer (contact)
    public func loadDirectMessages(for peerFourWords: String) {
        do {
            let msgs = try client?.messageGetDirect(peerFourWords: peerFourWords) ?? []
            messages[peerFourWords] = msgs.sorted { $0.createdAt < $1.createdAt }
        } catch {
            print("Error loading direct messages for \(peerFourWords): \(error)")
        }
    }

    /// Send a direct message to a peer (contact)
    public func sendDirectMessage(to peerFourWords: String, text: String) {
        guard !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        do {
            let messageId = try client?.messageSendDirect(recipientFourWords: peerFourWords, text: text)
            if messageId != nil {
                // Reload direct messages to include the new one
                loadDirectMessages(for: peerFourWords)
            }
        } catch {
            errorMessage = "Failed to send direct message: \(error.localizedDescription)"
            print("Error sending direct message: \(error)")
        }
    }

    /// Get direct messages with a peer
    public func getDirectMessages(for peerFourWords: String) -> [SwiftMessage] {
        return messages[peerFourWords] ?? []
    }

    // MARK: - Mutual Contact Discovery

    /// Prefix for contact notification messages
    private static let contactNotificationPrefix = "[CONTACT_REQUEST]"

    /// Send a contact notification to a peer so they can add us back
    private func sendContactNotification(to peerFourWords: String) {
        let notificationText = "\(Self.contactNotificationPrefix) \(fourWords) added you as a contact"
        do {
            _ = try client?.messageSendDirect(recipientFourWords: peerFourWords, text: notificationText)
            print("[Communitas] Sent contact notification to \(peerFourWords)")
        } catch {
            print("[Communitas] Failed to send contact notification: \(error)")
        }
    }

    /// Poll for contact requests from all known contacts
    /// This should be called periodically to discover mutual contacts
    public func pollForContactRequests() {
        guard isNetworking else { return }

        // Check direct messages from each network-linked contact for contact request messages
        // (Skip local-only contacts as they don't have four-word addresses)
        for contact in contacts {
            guard let contactFourWords = contact.fourWords else {
                // Skip local-only contacts
                continue
            }
            loadDirectMessages(for: contactFourWords)
            checkForContactRequestFromPeer(contactFourWords)
        }

        // Also check if there are any peers who messaged us that we haven't added as contacts
        discoverNewContactsFromMessages()
    }

    /// Check if a peer sent us a contact request and auto-add them if needed
    private func checkForContactRequestFromPeer(_ peerFourWords: String) {
        guard let msgs = messages[peerFourWords] else { return }

        // Check if peer sent us a contact request (meaning they added us)
        let hasContactRequest = msgs.contains { msg in
            msg.text.contains(Self.contactNotificationPrefix) && msg.author == peerFourWords
        }

        if hasContactRequest {
            // Check if they're already our contact
            if !contacts.contains(where: { $0.fourWords == peerFourWords }) {
                // Auto-add them as a contact (they added us first)
                print("[Communitas] Auto-adding mutual contact: \(peerFourWords)")
                addContactSilently(fourWords: peerFourWords)
            }
        }
    }

    /// Discover new contacts from direct message senders who aren't in our contact list
    private func discoverNewContactsFromMessages() {
        // Find all unique message senders
        var allSenders: Set<String> = []
        for (peerFourWords, msgs) in messages {
            for msg in msgs {
                if msg.author != fourWords { // Not from ourselves
                    allSenders.insert(msg.author)
                }
            }
            // Also consider the peer key itself if they sent us messages
            if msgs.contains(where: { $0.author != fourWords }) {
                allSenders.insert(peerFourWords)
            }
        }

        // Add any senders who aren't already contacts
        for sender in allSenders {
            if !contacts.contains(where: { $0.fourWords == sender }) && sender != fourWords {
                print("[Communitas] Discovered new contact from messages: \(sender)")
                addContactSilently(fourWords: sender)
            }
        }
    }

    /// Add a contact silently without sending a notification back (to avoid infinite loops)
    private func addContactSilently(fourWords contactFourWords: String) {
        let normalizedFourWords = contactFourWords
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .replacingOccurrences(of: " ", with: "-")
            .lowercased()

        guard !normalizedFourWords.isEmpty else { return }
        guard !contacts.contains(where: { $0.fourWords == normalizedFourWords }) else { return }

        let newContact = ContactItem(
            id: UUID().uuidString,
            fourWords: normalizedFourWords,
            displayName: nil,
            isFavourite: false,
            isOnline: false,
            lastSeen: nil,
            isLocalOnly: false  // Discovered contacts are network-linked
        )

        contacts.append(newContact)
        saveContactsLocally()

        // Add to gossip network
        if isNetworking {
            do {
                try client?.gossipAddContact(fourWords: normalizedFourWords)
            } catch {
                print("[Communitas] Failed to add discovered contact to network: \(error)")
            }
        }

        print("[Communitas] Silently added contact: \(normalizedFourWords)")
    }

    /// Start polling for contact requests
    private func startContactPolling() {
        // Cancel any existing timer
        contactPollTimer?.invalidate()

        // Poll immediately
        pollForContactRequests()

        // Then poll every 5 seconds
        contactPollTimer = Timer.scheduledTimer(withTimeInterval: 5.0, repeats: true) { [weak self] _ in
            DispatchQueue.main.async {
                self?.pollForContactRequests()
            }
        }
        print("[Communitas] Started contact polling timer")
    }

    /// Stop polling for contact requests
    private func stopContactPolling() {
        contactPollTimer?.invalidate()
        contactPollTimer = nil
        print("[Communitas] Stopped contact polling timer")
    }

    // MARK: - Storage/Drive Methods

    /// Get the storage directory for an entity
    public func getStorageDirectory(for entityId: String) -> URL {
        let baseDir = FileManager.default
            .urls(for: .applicationSupportDirectory, in: .userDomainMask)
            .first?
            .appendingPathComponent("Communitas/drives/\(entityId)")
            ?? FileManager.default.temporaryDirectory.appendingPathComponent("communitas-drives/\(entityId)")
        try? FileManager.default.createDirectory(at: baseDir, withIntermediateDirectories: true)
        return baseDir
    }

    /// Load files for an entity from local storage
    public func loadFiles(for entityId: String, path: String = "") {
        let baseDir = getStorageDirectory(for: entityId)
        let targetDir = path.isEmpty ? baseDir : baseDir.appendingPathComponent(path)

        do {
            let contents = try FileManager.default.contentsOfDirectory(
                at: targetDir,
                includingPropertiesForKeys: [.isDirectoryKey, .fileSizeKey, .contentModificationDateKey],
                options: [.skipsHiddenFiles]
            )

            let items = contents.compactMap { url -> FileItem? in
                let resourceValues = try? url.resourceValues(forKeys: [.isDirectoryKey, .fileSizeKey, .contentModificationDateKey])
                return FileItem(
                    id: url.path,
                    name: url.lastPathComponent,
                    path: url.path.replacingOccurrences(of: baseDir.path, with: ""),
                    isDirectory: resourceValues?.isDirectory ?? false,
                    sizeBytes: UInt64(resourceValues?.fileSize ?? 0),
                    modifiedAt: resourceValues?.contentModificationDate ?? Date()
                )
            }.sorted { $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending }

            files[entityId] = items
        } catch {
            print("Error loading files for \(entityId): \(error)")
            files[entityId] = []
        }
    }

    /// Get files for an entity
    public func getFiles(for entityId: String) -> [FileItem] {
        return files[entityId] ?? []
    }

    /// Create a new folder
    public func createFolder(in entityId: String, name: String) {
        let baseDir = getStorageDirectory(for: entityId)
        let folderURL = baseDir.appendingPathComponent(name)

        do {
            try FileManager.default.createDirectory(at: folderURL, withIntermediateDirectories: true)
            loadFiles(for: entityId)
        } catch {
            errorMessage = "Failed to create folder: \(error.localizedDescription)"
        }
    }

    /// Write file from data
    public func writeFile(in entityId: String, name: String, data: Data) {
        let baseDir = getStorageDirectory(for: entityId)
        let fileURL = baseDir.appendingPathComponent(name)

        do {
            try data.write(to: fileURL)
            loadFiles(for: entityId)
        } catch {
            errorMessage = "Failed to write file: \(error.localizedDescription)"
        }
    }

    /// Read file data
    public func readFile(in entityId: String, path: String) -> Data? {
        let baseDir = getStorageDirectory(for: entityId)
        let fileURL = baseDir.appendingPathComponent(path)
        return try? Data(contentsOf: fileURL)
    }

    /// Delete file or folder
    public func deleteFile(in entityId: String, path: String) {
        let baseDir = getStorageDirectory(for: entityId)
        let fileURL = baseDir.appendingPathComponent(path)

        do {
            try FileManager.default.removeItem(at: fileURL)
            loadFiles(for: entityId)
        } catch {
            errorMessage = "Failed to delete: \(error.localizedDescription)"
        }
    }

    /// Get storage stats for an entity
    public func getStorageStats(for entityId: String) -> (totalBytes: UInt64, fileCount: Int) {
        let baseDir = getStorageDirectory(for: entityId)
        var totalBytes: UInt64 = 0
        var fileCount = 0

        if let enumerator = FileManager.default.enumerator(at: baseDir, includingPropertiesForKeys: [.fileSizeKey]) {
            for case let fileURL as URL in enumerator {
                if let size = try? fileURL.resourceValues(forKeys: [.fileSizeKey]).fileSize {
                    totalBytes += UInt64(size)
                    fileCount += 1
                }
            }
        }

        return (totalBytes, fileCount)
    }

    // MARK: - Document Methods

    /// Get documents storage directory for an entity
    private func getDocumentsDirectory(for entityId: String) -> URL {
        let baseDir = FileManager.default
            .urls(for: .applicationSupportDirectory, in: .userDomainMask)
            .first?
            .appendingPathComponent("Communitas/documents/\(entityId)")
            ?? FileManager.default.temporaryDirectory.appendingPathComponent("communitas-documents/\(entityId)")
        try? FileManager.default.createDirectory(at: baseDir, withIntermediateDirectories: true)
        return baseDir
    }

    /// Load documents for an entity
    public func loadDocuments(for entityId: String) {
        let baseDir = getDocumentsDirectory(for: entityId)

        do {
            let contents = try FileManager.default.contentsOfDirectory(
                at: baseDir,
                includingPropertiesForKeys: nil,
                options: [.skipsHiddenFiles]
            )

            var docs: [DocumentItem] = []
            for fileURL in contents where fileURL.pathExtension == "json" {
                if let data = try? Data(contentsOf: fileURL),
                   let doc = try? JSONDecoder().decode(DocumentItem.self, from: data) {
                    docs.append(doc)
                }
            }

            documents[entityId] = docs.sorted { $0.modifiedAt > $1.modifiedAt }
        } catch {
            print("Error loading documents for \(entityId): \(error)")
            documents[entityId] = []
        }
    }

    /// Get documents for an entity
    public func getDocuments(for entityId: String) -> [DocumentItem] {
        return documents[entityId] ?? []
    }

    /// Create a new document
    public func createDocument(in entityId: String, name: String) {
        let doc = DocumentItem(
            id: UUID().uuidString,
            name: name,
            content: "",
            createdAt: Date(),
            modifiedAt: Date(),
            authorFourWords: fourWords
        )

        saveDocument(doc, for: entityId)
        loadDocuments(for: entityId)
    }

    /// Save a document
    public func saveDocument(_ document: DocumentItem, for entityId: String) {
        let baseDir = getDocumentsDirectory(for: entityId)
        let fileURL = baseDir.appendingPathComponent("\(document.id).json")

        var doc = document
        doc.modifiedAt = Date()

        do {
            let data = try JSONEncoder().encode(doc)
            try data.write(to: fileURL)
        } catch {
            errorMessage = "Failed to save document: \(error.localizedDescription)"
        }
    }

    /// Update document content
    public func updateDocumentContent(documentId: String, entityId: String, content: String) {
        guard var doc = documents[entityId]?.first(where: { $0.id == documentId }) else { return }
        doc.content = content
        doc.modifiedAt = Date()
        saveDocument(doc, for: entityId)
        loadDocuments(for: entityId)
    }

    /// Rename a document
    public func renameDocument(documentId: String, entityId: String, newName: String) {
        guard var doc = documents[entityId]?.first(where: { $0.id == documentId }) else { return }
        doc.name = newName
        saveDocument(doc, for: entityId)
        loadDocuments(for: entityId)
    }

    /// Delete a document
    public func deleteDocument(documentId: String, entityId: String) {
        let baseDir = getDocumentsDirectory(for: entityId)
        let fileURL = baseDir.appendingPathComponent("\(documentId).json")

        do {
            try FileManager.default.removeItem(at: fileURL)
            loadDocuments(for: entityId)
        } catch {
            errorMessage = "Failed to delete document: \(error.localizedDescription)"
        }
    }

    /// Get document by ID
    public func getDocument(id: String, entityId: String) -> DocumentItem? {
        return documents[entityId]?.first { $0.id == id }
    }

    // MARK: - Kanban Methods (Rust Backend)

    /// Default column definitions for new boards
    private static let defaultColumns: [(name: String, color: String)] = [
        ("backlog", "#6B7280"),      // Gray
        ("todo", "#3B82F6"),         // Blue
        ("in_progress", "#F59E0B"),  // Amber
        ("review", "#8B5CF6"),       // Purple
        ("done", "#10B981")          // Green
    ]

    /// Ensure a Kanban board exists for an entity, creating one if needed
    /// Returns the board ID
    private func ensureKanbanBoard(for entityId: String) -> String? {
        // Check if we already have a board ID cached
        if let boardId = kanbanBoardIds[entityId] {
            return boardId
        }

        guard let client = client else {
            print("[Communitas] Cannot create Kanban board: client not initialized")
            return nil
        }

        do {
            // Create a new board for this entity
            let boardId = try client.kanbanCreateBoard(
                projectId: entityId,
                name: "Board",
                description: nil
            )

            // Add default columns
            var columnMapping: [String: String] = [:]
            for column in Self.defaultColumns {
                let columnId = try client.kanbanAddColumn(
                    boardId: boardId,
                    name: column.name,
                    color: column.color,
                    wipLimit: nil
                )
                columnMapping[column.name] = columnId
            }

            // Cache the board ID and column mappings
            kanbanBoardIds[entityId] = boardId
            kanbanColumnMappings[boardId] = columnMapping

            print("[Communitas] Created Kanban board '\(boardId)' with \(columnMapping.count) columns for entity \(entityId)")
            return boardId
        } catch {
            print("[Communitas] Error creating Kanban board for \(entityId): \(error)")
            return nil
        }
    }

    /// Get column ID for a column name, ensuring board exists
    private func getColumnId(for columnName: String, boardId: String) -> String? {
        return kanbanColumnMappings[boardId]?[columnName]
    }

    /// Load column mappings for a board from the Rust backend
    private func loadColumnMappings(for boardId: String) {
        guard let client = client else { return }

        do {
            let board = try client.kanbanGetBoard(boardId: boardId)
            var mapping: [String: String] = [:]
            for column in board.columns {
                mapping[column.name] = column.id
            }
            kanbanColumnMappings[boardId] = mapping
        } catch {
            print("[Communitas] Error loading column mappings: \(error)")
        }
    }

    /// Load Kanban cards for an entity from Rust backend
    public func loadKanbanCards(for entityId: String) {
        guard let boardId = ensureKanbanBoard(for: entityId) else {
            kanbanCards[entityId] = []
            return
        }

        // Ensure we have column mappings
        if kanbanColumnMappings[boardId] == nil {
            loadColumnMappings(for: boardId)
        }

        guard let client = client else {
            kanbanCards[entityId] = []
            return
        }

        do {
            let swiftCards = try client.kanbanListCards(boardId: boardId, columnId: nil)

            // Convert SwiftKanbanCard to KanbanCard with column name lookup
            let cards: [KanbanCard] = swiftCards.compactMap { card in
                // Find column name from ID
                let columnName = kanbanColumnMappings[boardId]?.first { $0.value == card.columnId }?.key ?? "backlog"

                return KanbanCard(
                    id: card.id,
                    title: card.title,
                    cardDescription: card.description.isEmpty ? nil : card.description,
                    column: columnName,
                    assignee: card.assigneeIds.first,
                    priority: 1,  // Default priority (Rust backend uses state machine instead)
                    createdAt: Date(timeIntervalSince1970: Double(card.createdAt) / 1000.0),
                    updatedAt: Date(timeIntervalSince1970: Double(card.updatedAt) / 1000.0),
                    sortOrder: Int(card.position),
                    commentCount: Int(card.commentCount)
                )
            }

            kanbanCards[entityId] = cards.sorted { $0.sortOrder < $1.sortOrder }
            print("[Communitas] Loaded \(cards.count) Kanban cards from Rust backend for entity \(entityId)")
        } catch {
            print("[Communitas] Error loading Kanban cards for \(entityId): \(error)")
            kanbanCards[entityId] = []
        }
    }

    /// Get Kanban cards for an entity
    public func getKanbanCards(for entityId: String) -> [KanbanCard] {
        return kanbanCards[entityId] ?? []
    }

    /// Create a new Kanban card using Rust backend
    public func createKanbanCard(in entityId: String, title: String, column: String, description: String? = nil) {
        guard !title.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }

        guard let boardId = ensureKanbanBoard(for: entityId),
              let columnId = getColumnId(for: column, boardId: boardId),
              let client = client else {
            print("[Communitas] Cannot create card: board or column not found")
            return
        }

        do {
            let cardId = try client.kanbanCreateCard(
                boardId: boardId,
                columnId: columnId,
                title: title.trimmingCharacters(in: .whitespacesAndNewlines),
                description: description
            )

            print("[Communitas] Created Kanban card '\(title)' (id: \(cardId)) in column '\(column)' for entity \(entityId)")

            // Reload cards to include the new one
            loadKanbanCards(for: entityId)
        } catch {
            print("[Communitas] Error creating Kanban card: \(error)")
            errorMessage = "Failed to create card: \(error.localizedDescription)"
        }
    }

    /// Move a Kanban card to a new column using Rust backend
    public func moveKanbanCard(cardId: String, entityId: String, toColumn: String) {
        guard let boardId = kanbanBoardIds[entityId],
              let columnId = getColumnId(for: toColumn, boardId: boardId),
              let client = client else {
            print("[Communitas] Cannot move card: board or column not found")
            return
        }

        // Find target position (end of column)
        let cardsInColumn = (kanbanCards[entityId] ?? []).filter { $0.column == toColumn }
        let position = UInt32(cardsInColumn.count)

        do {
            try client.kanbanMoveCard(
                boardId: boardId,
                cardId: cardId,
                toColumnId: columnId,
                position: position
            )

            print("[Communitas] Moved card '\(cardId)' to column '\(toColumn)'")

            // Reload cards to reflect the change
            loadKanbanCards(for: entityId)
        } catch {
            print("[Communitas] Error moving Kanban card: \(error)")
            errorMessage = "Failed to move card: \(error.localizedDescription)"
        }
    }

    /// Update a Kanban card using Rust backend
    public func updateKanbanCard(cardId: String, entityId: String, title: String? = nil, description: String? = nil, priority: Int? = nil) {
        guard let boardId = kanbanBoardIds[entityId],
              let client = client else {
            print("[Communitas] Cannot update card: board not found")
            return
        }

        do {
            try client.kanbanUpdateCard(
                boardId: boardId,
                cardId: cardId,
                title: title,
                description: description
            )

            print("[Communitas] Updated Kanban card '\(cardId)'")

            // Reload cards to reflect the change
            loadKanbanCards(for: entityId)
        } catch {
            print("[Communitas] Error updating Kanban card: \(error)")
            errorMessage = "Failed to update card: \(error.localizedDescription)"
        }
    }

    /// Delete a Kanban card using Rust backend
    public func deleteKanbanCard(cardId: String, entityId: String) {
        guard let boardId = kanbanBoardIds[entityId],
              let client = client else {
            print("[Communitas] Cannot delete card: board not found")
            return
        }

        let cardTitle = kanbanCards[entityId]?.first { $0.id == cardId }?.title ?? "unknown"

        do {
            try client.kanbanDeleteCard(boardId: boardId, cardId: cardId)

            print("[Communitas] Deleted Kanban card '\(cardTitle)'")

            // Reload cards to reflect the change
            loadKanbanCards(for: entityId)
        } catch {
            print("[Communitas] Error deleting Kanban card: \(error)")
            errorMessage = "Failed to delete card: \(error.localizedDescription)"
        }
    }

    /// Get cards for a specific column
    public func getKanbanCards(for entityId: String, column: String) -> [KanbanCard] {
        return (kanbanCards[entityId] ?? [])
            .filter { $0.column == column }
            .sorted { $0.sortOrder < $1.sortOrder }
    }

    /// Get card count per column for an entity
    public func getKanbanColumnCounts(for entityId: String) -> [String: Int] {
        let cards = kanbanCards[entityId] ?? []
        var counts: [String: Int] = [:]
        for card in cards {
            counts[card.column, default: 0] += 1
        }
        return counts
    }

    /// Get CRDT sync update for a Kanban board
    public func getKanbanSyncUpdate(for entityId: String) -> Data? {
        guard let boardId = kanbanBoardIds[entityId],
              let client = client else {
            return nil
        }

        do {
            let data = try client.kanbanGetSyncUpdate(boardId: boardId)
            return data
        } catch {
            print("[Communitas] Error getting Kanban sync update: \(error)")
            return nil
        }
    }

    /// Apply a CRDT sync update to a Kanban board
    public func applyKanbanSyncUpdate(for entityId: String, update: Data) {
        guard let boardId = kanbanBoardIds[entityId],
              let client = client else {
            return
        }

        do {
            try client.kanbanApplySyncUpdate(boardId: boardId, update: update)
            // Reload cards after sync
            loadKanbanCards(for: entityId)
            print("[Communitas] Applied Kanban sync update for entity \(entityId)")
        } catch {
            print("[Communitas] Error applying Kanban sync update: \(error)")
        }
    }

    // MARK: - Card Discussion Methods

    /// Load comments for a card
    public func loadCardComments(boardId: String, cardId: String) -> [SwiftKanbanComment] {
        guard let client = client else {
            return []
        }

        do {
            let comments = try client.kanbanListComments(boardId: boardId, cardId: cardId)
            return comments
        } catch {
            print("[Communitas] Error loading comments for card \(cardId): \(error)")
            return []
        }
    }

    /// Add a comment to a card
    public func addCardComment(boardId: String, cardId: String, content: String, replyToId: String? = nil) -> String? {
        guard let client = client else {
            errorMessage = "Not initialized"
            return nil
        }

        do {
            let commentId = try client.kanbanAddComment(
                boardId: boardId,
                cardId: cardId,
                content: content,
                replyToId: replyToId
            )
            print("[Communitas] Added comment '\(commentId)' to card \(cardId)")
            return commentId
        } catch {
            print("[Communitas] Error adding comment: \(error)")
            errorMessage = "Failed to add comment: \(error.localizedDescription)"
            return nil
        }
    }

    /// Get board ID for an entity (helper for card discussions)
    public func getBoardId(for entityId: String) -> String? {
        return kanbanBoardIds[entityId]
    }

    // MARK: - P2P Networking Methods

    /// Start networking with optional port
    public func startNetworkingWithBootstrap(port: UInt16? = nil) {
        do {
            let identity = try client?.gossipStart(port: port)
            connectionIdentity = identity
            isNetworking = true
            // Start presence beacons automatically
            startPresenceBeacons()
            print("[Communitas] Networking started with identity: \(identity ?? "unknown")")

            // Auto-detect external address in background (non-blocking)
            // This runs with retries to allow peer connections to establish
            // Reset detection failed flag before starting
            externalAddressDetectionFailed = false
            DispatchQueue.global(qos: .background).async { [weak self] in
                do {
                    try self?.client?.gossipAutoRequestExternalAddress()
                    DispatchQueue.main.async {
                        self?.objectWillChange.send()
                        print("[Communitas] External address auto-detected successfully")
                    }
                } catch {
                    DispatchQueue.main.async {
                        self?.externalAddressDetectionFailed = true
                        self?.objectWillChange.send()
                    }
                    print("[Communitas] External address auto-detection failed (non-fatal): \(error.localizedDescription)")
                }
            }
        } catch {
            errorMessage = "Failed to start networking: \(error.localizedDescription)"
        }
    }

    /// Start presence beacons
    public func startPresenceBeacons() {
        do {
            try client?.presenceStartBeacons()
            isPresenceActive = true
        } catch {
            print("Failed to start presence beacons: \(error)")
        }
    }

    /// Stop presence beacons
    public func stopPresenceBeacons() {
        do {
            try client?.presenceStopBeacons()
            isPresenceActive = false
        } catch {
            print("Failed to stop presence beacons: \(error)")
        }
    }

    // MARK: - Contact Methods

    /// Load contacts from local storage and merge with network contacts
    public func loadContacts() {
        // First load local contacts
        loadContactsLocally()

        // If networking is active, merge with network contacts
        if isNetworking {
            do {
                let swiftContacts = try client?.gossipGetContacts() ?? []
                let networkContacts = swiftContacts.map { c in
                    // Convert timestamps (millis) to Date if present
                    let endpointDate: Date? = c.endpointUpdatedAt.map { Date(timeIntervalSince1970: Double($0) / 1000.0) }
                    let linkedAtDate: Date? = c.linkedAt.map { Date(timeIntervalSince1970: Double($0) / 1000.0) }
                    let lastSyncDate: Date? = c.lastSyncAt.map { Date(timeIntervalSince1970: Double($0) / 1000.0) }
                    return ContactItem(
                        id: c.id,
                        fourWords: c.fourWords,
                        displayName: c.displayName,
                        isFavourite: c.isFavourite,
                        isOnline: c.online,
                        lastSeen: nil,
                        isLocalOnly: c.isLocalOnly,
                        linkedAt: linkedAtDate,
                        lastSyncAt: lastSyncDate,
                        lastSeenEndpoint: c.lastSeenEndpoint,
                        endpointUpdatedAt: endpointDate,
                        endpointSuccessCount: c.endpointSuccessCount,
                        endpointFailureCount: c.endpointFailureCount
                    )
                }

                // Merge: keep local display names, update online status and endpoints from network
                for networkContact in networkContacts {
                    if let index = contacts.firstIndex(where: { $0.id == networkContact.id || ($0.fourWords != nil && $0.fourWords == networkContact.fourWords) }) {
                        // Update online status but preserve local display name if set
                        contacts[index].isOnline = networkContact.isOnline
                        contacts[index].isFavourite = networkContact.isFavourite
                        if contacts[index].displayName == nil {
                            contacts[index].displayName = networkContact.displayName
                        }
                        // Update endpoint tracking from network
                        contacts[index].lastSeenEndpoint = networkContact.lastSeenEndpoint
                        contacts[index].endpointUpdatedAt = networkContact.endpointUpdatedAt
                        contacts[index].endpointSuccessCount = networkContact.endpointSuccessCount
                        contacts[index].endpointFailureCount = networkContact.endpointFailureCount
                    } else {
                        // Add network contact that's not local
                        contacts.append(networkContact)
                    }
                }
                print("[Communitas] Merged \(contacts.count) contacts (local + network)")
            } catch {
                print("Error loading network contacts: \(error)")
                // Keep local contacts even if network fails
            }
        }
    }

    /// Find a contact by four-word address
    public func findContact(fourWords: String) -> String? {
        guard isNetworking else {
            errorMessage = "Networking not active"
            return nil
        }

        do {
            let peerId = try client?.gossipFindContact(fourWords: fourWords)
            return peerId
        } catch {
            errorMessage = "Contact not found: \(error.localizedDescription)"
            return nil
        }
    }

    /// Add a contact by four-word address with optional display name
    /// Accepts input with spaces or hyphens (e.g., "bear wolf swift dragon" or "bear-wolf-swift-dragon")
    public func addContact(fourWords: String, displayName: String? = nil) {
        // Normalize input: convert spaces to hyphens for backend
        let normalizedFourWords = fourWords
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .replacingOccurrences(of: " ", with: "-")
            .lowercased()

        guard !normalizedFourWords.isEmpty else {
            errorMessage = "Please enter a four-word address"
            showError = true
            return
        }

        // Create contact item for local storage
        let newContact = ContactItem(
            id: UUID().uuidString,
            fourWords: normalizedFourWords,
            displayName: displayName,
            isFavourite: false,
            isOnline: false,
            lastSeen: nil,
            isLocalOnly: false  // Contact with four-word is network-linked
        )

        // Add to local contacts if not already present (check by fourWords)
        if !contacts.contains(where: { $0.fourWords == normalizedFourWords }) {
            contacts.append(newContact)
        } else {
            // Update display name if contact exists
            if let index = contacts.firstIndex(where: { $0.fourWords == normalizedFourWords }) {
                contacts[index].displayName = displayName
            }
        }

        // Save locally
        saveContactsLocally()

        // If networking is active, also add to gossip network and send contact notification
        if isNetworking {
            do {
                try client?.gossipAddContact(fourWords: normalizedFourWords)
                print("[Communitas] Successfully added contact to network: \(normalizedFourWords)")

                // Send a contact request notification to the peer so they can add us back
                sendContactNotification(to: normalizedFourWords)
            } catch {
                // Non-fatal: contact is saved locally even if network add fails
                print("[Communitas] Failed to add contact to network (saved locally): \(error)")
            }
        }

        print("[Communitas] Contact saved locally: \(normalizedFourWords) (\(displayName ?? "no display name"))")
    }

    /// Create a local-only contact (without a network identity)
    ///
    /// These contacts are placeholders until the user learns their four-word address.
    public func createLocalContact(displayName: String) {
        guard !displayName.isEmpty else { return }

        // Create local-only contact (no four-word address)
        let newContact = ContactItem(
            id: UUID().uuidString,
            fourWords: nil,  // No network identity yet
            displayName: displayName,
            isFavourite: false,
            isOnline: false,
            lastSeen: nil,
            isLocalOnly: true  // Mark as local-only placeholder
        )

        contacts.append(newContact)
        saveContactsLocally()

        // Also save to backend if networking is active
        if isNetworking {
            do {
                _ = try client?.contactCreateLocal(displayName: displayName)
                print("[Communitas] Created local-only contact in backend: \(displayName)")
            } catch {
                print("[Communitas] Failed to create local contact in backend (saved locally): \(error)")
            }
        }

        print("[Communitas] Created local-only contact: \(displayName)")
    }

    /// Link a local-only contact to a network identity
    ///
    /// This converts a placeholder contact to a network-linked contact.
    public func linkContact(contactId: String, fourWords: String) {
        let normalizedFourWords = fourWords
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .replacingOccurrences(of: " ", with: "-")
            .lowercased()

        guard !normalizedFourWords.isEmpty else {
            print("[Communitas] Cannot link contact: empty four-words")
            return
        }

        guard let index = contacts.firstIndex(where: { $0.id == contactId }) else {
            print("[Communitas] Cannot link contact: contact not found")
            return
        }

        // Update the contact with network identity
        var updatedContact = contacts[index]
        updatedContact = ContactItem(
            id: updatedContact.id,
            fourWords: normalizedFourWords,
            displayName: updatedContact.displayName,
            isFavourite: updatedContact.isFavourite,
            isOnline: false,
            lastSeen: nil,
            isLocalOnly: false,  // Now linked to network
            linkedAt: Date(),
            lastSyncAt: nil,
            lastSeenEndpoint: nil,
            endpointUpdatedAt: nil,
            endpointSuccessCount: 0,
            endpointFailureCount: 0
        )
        contacts[index] = updatedContact

        saveContactsLocally()

        // Link in backend and add to gossip network
        if isNetworking {
            do {
                _ = try client?.contactLinkToNetwork(contactId: contactId, fourWords: normalizedFourWords)
                try client?.gossipAddContact(fourWords: normalizedFourWords)
                print("[Communitas] Linked contact to network: \(normalizedFourWords)")
            } catch {
                print("[Communitas] Failed to link contact in backend: \(error)")
            }
        }
    }

    /// Get all local-only contacts
    public var localOnlyContacts: [ContactItem] {
        contacts.filter { $0.isLocalOnly }
    }

    /// Get all network-linked contacts
    public var linkedContacts: [ContactItem] {
        contacts.filter { $0.isLinked }
    }

    // MARK: - Entity Linking

    /// Link a local-only entity to a network identity
    ///
    /// This converts a placeholder entity to a network-linked entity.
    public func linkEntity(entityId: String, fourWords: String) {
        let normalizedFourWords = fourWords
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .replacingOccurrences(of: " ", with: "-")
            .lowercased()

        guard !normalizedFourWords.isEmpty else {
            print("[Communitas] Cannot link entity: empty four-words")
            return
        }

        // Link in backend
        if isNetworking {
            do {
                _ = try client?.entityLinkToNetwork(entityId: entityId, fourWords: normalizedFourWords)
                print("[Communitas] Linked entity to network: \(entityId) -> \(normalizedFourWords)")
                // Refresh entities to get updated data
                loadEntities()
            } catch {
                print("[Communitas] Failed to link entity in backend: \(error)")
                errorMessage = "Failed to link entity: \(error.localizedDescription)"
                showError = true
            }
        } else {
            errorMessage = "Networking must be active to link entities"
            showError = true
        }
    }

    /// Get all local-only entities
    public var localOnlyEntities: [SwiftEntity] {
        entities.filter { $0.isLocalOnly }
    }

    /// Get all network-linked entities
    public var linkedEntities: [SwiftEntity] {
        entities.filter { !$0.isLocalOnly && $0.networkFourWords != nil }
    }

    // MARK: - Local Contact Storage

    private var localContactsURL: URL? {
        // Use fourWords as the vault identifier (matches vault directory structure)
        guard !fourWords.isEmpty, fourWords != "ocean-forest-moon-star" else { return nil }
        let appSupport = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
        return appSupport?.appendingPathComponent("communitas/vaults/\(fourWords)/contacts.json")
    }

    /// Save contacts to local storage
    private func saveContactsLocally() {
        guard let url = localContactsURL else {
            print("[Communitas] No vault ID, cannot save contacts locally")
            return
        }

        do {
            // Ensure directory exists
            try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)

            let encoder = JSONEncoder()
            encoder.outputFormatting = .prettyPrinted
            let data = try encoder.encode(contacts)
            try data.write(to: url)
            print("[Communitas] Saved \(contacts.count) contacts locally")
        } catch {
            print("[Communitas] Failed to save contacts locally: \(error)")
        }
    }

    /// Load contacts from local storage
    public func loadContactsLocally() {
        guard let url = localContactsURL else {
            print("[Communitas] No vault ID, cannot load contacts locally")
            return
        }

        guard FileManager.default.fileExists(atPath: url.path) else {
            print("[Communitas] No local contacts file found")
            return
        }

        do {
            let data = try Data(contentsOf: url)
            let decoder = JSONDecoder()
            let localContacts = try decoder.decode([ContactItem].self, from: data)
            contacts = localContacts
            print("[Communitas] Loaded \(contacts.count) contacts from local storage")
        } catch {
            print("[Communitas] Failed to load contacts locally: \(error)")
        }
    }

    /// Remove a contact from both local storage and network
    public func removeContact(fourWords: String) {
        // Remove from local contacts
        contacts.removeAll { $0.fourWords == fourWords }
        saveContactsLocally()

        // If networking is active, also remove from gossip network
        if isNetworking {
            do {
                try client?.gossipRemoveContact(fourWords: fourWords)
            } catch {
                print("[Communitas] Failed to remove contact from network (removed locally): \(error)")
            }
        }

        print("[Communitas] Contact removed: \(fourWords)")
    }

    /// Toggle favourite status for a contact
    public func toggleFavouriteContact(fourWords: String) {
        // Update locally
        if let index = contacts.firstIndex(where: { $0.fourWords == fourWords }) {
            contacts[index].isFavourite.toggle()
            saveContactsLocally()
        }

        // If networking is active, also update on gossip network
        if isNetworking {
            do {
                try client?.gossipAddFavouriteContact(fourWords: fourWords)
            } catch {
                print("[Communitas] Failed to update favourite on network (saved locally): \(error)")
            }
        }
    }

    /// Toggle favourite status for a contact by ID
    public func toggleFavouriteContactById(id: String) {
        // Update locally
        if let index = contacts.firstIndex(where: { $0.id == id }) {
            contacts[index].isFavourite.toggle()
            saveContactsLocally()

            // If the contact is linked (has four-words), sync to network
            if let fourWords = contacts[index].fourWords, isNetworking {
                do {
                    try client?.gossipAddFavouriteContact(fourWords: fourWords)
                } catch {
                    print("[Communitas] Failed to update favourite on network (saved locally): \(error)")
                }
            }
        }
    }

    /// Remove a contact by ID
    public func removeContactById(id: String) {
        // Find the contact first to get fourWords if available
        let contact = contacts.first { $0.id == id }

        // Remove from local contacts
        contacts.removeAll { $0.id == id }
        saveContactsLocally()

        // If the contact was linked, also remove from network
        if let fourWords = contact?.fourWords, isNetworking {
            do {
                try client?.gossipRemoveContact(fourWords: fourWords)
            } catch {
                print("[Communitas] Failed to remove contact from network: \(error)")
            }
        }

        print("[Communitas] Contact removed: \(id)")
    }

    /// Check if a contact is online
    public func isContactOnline(fourWords: String) -> Bool {
        guard isNetworking else { return false }

        do {
            return try client?.presenceIsOnline(fourWords: fourWords) ?? false
        } catch {
            return false
        }
    }

    /// Connect to a specific peer
    public func connectToPeer(fourWords: String) {
        guard isNetworking else {
            errorMessage = "Networking not active"
            return
        }

        do {
            try client?.gossipConnectToPeer(fourWords: fourWords)
        } catch {
            errorMessage = "Failed to connect to peer: \(error.localizedDescription)"
        }
    }

    /// Dial a specific socket address to establish a connection
    /// Used for local peer discovery or manual connection testing
    public func dialAddress(_ address: String) {
        guard isNetworking else {
            errorMessage = "Networking not active"
            return
        }

        do {
            try client?.gossipDialAddress(address: address)
            print("[Communitas] Successfully dialed address: \(address)")
        } catch {
            errorMessage = "Failed to dial address: \(error.localizedDescription)"
            print("[Communitas] Failed to dial address \(address): \(error)")
        }
    }

    /// Get the bound port for this node's networking
    public func getBoundPort() -> UInt16? {
        return client?.gossipGetBoundPort()
    }

    // MARK: - Permission Methods (Phase 3: Granular Permissions)

    /// Check if the current user can view a resource type in an entity
    ///
    /// Returns true if the user has at least ReadOnly permission level.
    public func canView(entityId: String, resource: SwiftResourceType) -> Bool {
        guard let client = client else { return false }
        do {
            return try client.permissionCanView(entityId: entityId, resourceType: resource)
        } catch {
            print("[Communitas] Permission check failed: \(error)")
            return false
        }
    }

    /// Check if the current user can edit a resource type in an entity
    ///
    /// Returns true if the user has Edit permission level.
    public func canEdit(entityId: String, resource: SwiftResourceType) -> Bool {
        guard let client = client else { return false }
        do {
            return try client.permissionCanEdit(entityId: entityId, resourceType: resource)
        } catch {
            print("[Communitas] Permission check failed: \(error)")
            return false
        }
    }

    /// Get the effective permission level for a resource in an entity
    ///
    /// Returns the computed permission level combining role defaults and any overrides.
    public func getEffectivePermission(entityId: String, resource: SwiftResourceType) -> SwiftAccessLevel {
        guard let client = client else { return .notVisible }
        do {
            return try client.permissionGetEffective(entityId: entityId, resourceType: resource)
        } catch {
            print("[Communitas] Permission check failed: \(error)")
            return .notVisible
        }
    }

    /// Check if current user can access a resource with a required permission level
    public func canAccess(entityId: String, resource: SwiftResourceType, required: SwiftAccessLevel) -> Bool {
        guard let client = client else { return false }
        do {
            return try client.permissionCanAccess(entityId: entityId, resourceType: resource, requiredLevel: required)
        } catch {
            print("[Communitas] Permission check failed: \(error)")
            return false
        }
    }

    /// Get the current user's role in an entity
    public func getMemberRole(entityId: String, memberFourWords: String) -> String? {
        guard let client = client else { return nil }
        do {
            return try client.permissionGetMemberRole(entityId: entityId, memberFourWords: memberFourWords)
        } catch {
            print("[Communitas] Failed to get member role: \(error)")
            return nil
        }
    }

    /// Set a member's role in an entity (requires admin/owner)
    public func setMemberRole(entityId: String, memberFourWords: String, role: String) {
        guard let client = client else {
            errorMessage = "Not initialized"
            showError = true
            return
        }
        do {
            try client.permissionSetMemberRole(entityId: entityId, memberFourWords: memberFourWords, role: role)
            print("[Communitas] Set role '\(role)' for member \(memberFourWords)")
        } catch {
            print("[Communitas] Failed to set member role: \(error)")
            errorMessage = "Failed to set member role: \(error.localizedDescription)"
            showError = true
        }
    }

    /// Set a permission override for a member (requires admin/owner)
    public func setPermissionOverride(entityId: String, memberFourWords: String, resource: SwiftResourceType, level: SwiftAccessLevel) {
        guard let client = client else {
            errorMessage = "Not initialized"
            showError = true
            return
        }
        do {
            try client.permissionSetMemberOverride(entityId: entityId, memberFourWords: memberFourWords, resourceType: resource, level: level)
            print("[Communitas] Set permission override for \(memberFourWords): \(resource) = \(level)")
        } catch {
            print("[Communitas] Failed to set permission override: \(error)")
            errorMessage = "Failed to set permission: \(error.localizedDescription)"
            showError = true
        }
    }

    /// Remove a permission override for a member
    public func removePermissionOverride(entityId: String, memberFourWords: String, resource: SwiftResourceType) {
        guard let client = client else {
            errorMessage = "Not initialized"
            showError = true
            return
        }
        do {
            try client.permissionRemoveMemberOverride(entityId: entityId, memberFourWords: memberFourWords, resourceType: resource)
            print("[Communitas] Removed permission override for \(memberFourWords): \(resource)")
        } catch {
            print("[Communitas] Failed to remove permission override: \(error)")
            errorMessage = "Failed to remove permission: \(error.localizedDescription)"
            showError = true
        }
    }

    /// Get all permission overrides for a member
    public func getMemberPermissionOverrides(entityId: String, memberFourWords: String) -> [SwiftMemberPermission] {
        guard let client = client else { return [] }
        do {
            return try client.permissionGetMemberOverrides(entityId: entityId, memberFourWords: memberFourWords)
        } catch {
            print("[Communitas] Failed to get permission overrides: \(error)")
            return []
        }
    }

    /// Get all permissions for a member (role defaults + overrides)
    public func getAllMemberPermissions(entityId: String, memberFourWords: String) -> [SwiftMemberPermission] {
        guard let client = client else { return [] }
        do {
            return try client.permissionGetAllForMember(entityId: entityId, memberFourWords: memberFourWords)
        } catch {
            print("[Communitas] Failed to get all member permissions: \(error)")
            return []
        }
    }

    /// Get comprehensive network information
    public func getNetworkInfo() -> (
        isActive: Bool,
        connectionIdentity: String?,
        listenAddress: String?,
        externalAddress: String?,
        externalAddressWords: String?,
        port: UInt16?,
        fourWords: String,
        isLocalOnlyMode: Bool
    )? {
        guard let info = client?.gossipGetNetworkInfo() else { return nil }
        return (
            isActive: info.isActive,
            connectionIdentity: info.connectionIdentity,
            listenAddress: info.listenAddress,
            externalAddress: info.externalAddress,
            externalAddressWords: info.externalAddressWords,
            port: info.port,
            fourWords: info.fourWords,
            isLocalOnlyMode: info.isLocalOnlyMode
        )
    }

    /// Request external/public address via NAT reflection
    ///
    /// Queries a connected bootstrap node to determine our public IP address
    /// as seen from the internet. Should be called after networking is active.
    public func requestExternalAddress() {
        do {
            try client?.gossipRequestExternalAddress()
            // Trigger UI refresh by publishing change
            objectWillChange.send()
            print("[Communitas] Requested external address via NAT reflection")
        } catch {
            print("[Communitas] Failed to request external address: \(error.localizedDescription)")
            errorMessage = "Failed to detect external address: \(error.localizedDescription)"
        }
    }

    // MARK: - Navigation Methods

    /// Toggle the network panel view (toggle behavior: if already showing, go home)
    public func toggleNetworkPanel() {
        if activeView == .networkPanel {
            activeView = .home
        } else {
            activeView = .networkPanel
        }
    }

    /// Open a chat with an entity
    public func openChat(entityType: String, entityId: String, entityName: String) {
        activeView = .chat(entityType: entityType, entityId: entityId, entityName: entityName)
        loadMessages(for: entityId)
    }

    /// Open a chat with a contact
    public func openContactChat(fourWords: String, displayName: String?) {
        activeView = .contactChat(fourWords: fourWords, displayName: displayName)
        // Load messages for contact - using fourWords as the entity ID for 1:1 chats
        loadMessages(for: fourWords)
    }

    /// Open a drive view for an entity
    public func openDrive(entityType: String, entityId: String) {
        activeView = .drive(entityType: entityType, entityId: entityId)
        loadFiles(for: entityId)
    }

    /// Navigate back to home
    public func navigateHome() {
        activeView = .home
    }
}
