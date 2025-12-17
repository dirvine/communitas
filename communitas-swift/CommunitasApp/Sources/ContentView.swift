import SwiftUI
import CommunitasKit

// CommunitasKit bindings are now working with sync API
// Async FFI issue resolved by using tokio runtime with block_on wrappers
// AppState, types, and ActiveView routing are now in AppState.swift


public struct ContentView: View {
    @EnvironmentObject var state: AppState
    @State private var selectedEntity: SwiftEntity?
    @State private var selectedSection: SidebarSection?
    @State private var columnVisibility: NavigationSplitViewVisibility = .all
    @State private var showingContactsView = false
    @State private var showingNetworkSettings = false

    public init() {}

    public var body: some View {
        Group {
            if state.isAuthenticated && state.client != nil {
                // Main app with sidebar navigation
                NavigationSplitView(columnVisibility: $columnVisibility) {
                    // Sidebar
                    VStack(spacing: 0) {
                        // Profile header
                        ProfileHeader(showingNetworkSettings: $showingNetworkSettings, showingContactsView: $showingContactsView)
                            .padding(.horizontal)
                            .padding(.vertical, 8)

                        Divider()

                        // Entity sidebar
                        SidebarView(
                            selectedEntity: $selectedEntity,
                            selectedSection: $selectedSection
                        )
                    }
                    .navigationSplitViewColumnWidth(min: 240, ideal: 280, max: 350)
                } detail: {
                    // Detail view - responds to activeView routing
                    detailPane
                }
                .navigationSplitViewStyle(.balanced)
                .sheet(isPresented: $showingContactsView) {
                    ContactsView()
                        .environmentObject(state)
                }
                .sheet(isPresented: $showingNetworkSettings) {
                    NetworkSettingsView()
                        .environmentObject(state)
                }
                .alert("Error", isPresented: $state.showError) {
                    Button("OK") {
                        state.showError = false
                    }
                } message: {
                    Text(state.errorMessage ?? "An unknown error occurred")
                }
            } else if let error = state.errorMessage {
                ErrorView(error: error) {
                    state.initialize()
                }
            } else {
                LoadingView()
            }
        }
        .onAppear {
            if state.isAuthenticated && state.client != nil {
                state.loadEntities()
                state.isNetworking = state.client?.isNetworkingActive() ?? false
                if state.isNetworking {
                    state.loadContacts()
                }
            }
        }
    }

    // MARK: - Detail Pane Routing

    @ViewBuilder
    private var detailPane: some View {
        switch state.activeView {
        case .home:
            // Default home view - show selected entity or welcome
            if let entity = selectedEntity {
                EntityDetailPane(entity: entity)
            } else {
                WelcomePane()
            }

        case .contactChat(let fourWords, let displayName):
            // Contact direct message view
            ContactChatView(
                fourWords: fourWords,
                displayName: displayName
            )

        case .chat(_, let entityId, let entityName):
            // Entity chat view (for groups/channels/etc)
            if let entity = state.entities.first(where: { $0.id == entityId }) {
                ChatView(entity: entity)
            } else {
                // Fallback: show a simple chat pane
                VStack {
                    Text("Chat: \(entityName)")
                        .font(.title2)
                    Text("Entity not loaded")
                        .foregroundColor(.secondary)
                }
            }

        case .drive(let entityType, let entityId):
            // Drive view - placeholder for now
            VStack {
                Image(systemName: "externaldrive")
                    .font(.system(size: 48))
                    .foregroundColor(.secondary)
                Text("Drive")
                    .font(.title2)
                Text("\(entityType): \(entityId)")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

        case .call(let peerFourWords):
            // Call view - full WebRTC call interface
            CallView(
                callId: UUID().uuidString,  // TODO: Get actual call ID from WebRTC layer
                peerFourWords: peerFourWords,
                displayName: nil  // TODO: Look up display name from contacts
            )

        case .project(let projectId):
            // Project view - placeholder for now
            VStack {
                Image(systemName: "folder.fill")
                    .font(.system(size: 48))
                    .foregroundColor(.blue)
                Text("Project")
                    .font(.title2)
                Text(projectId)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

        case .networkPanel:
            // Network status panel - shows P2P network info
            NetworkPanelView()
        }
    }
}

// MARK: - Profile Header

struct ProfileHeader: View {
    @EnvironmentObject var state: AppState
    @Binding var showingNetworkSettings: Bool
    @Binding var showingContactsView: Bool
    @State private var showFullFourWords = false

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 10) {
                // Avatar
                Image(systemName: "person.circle.fill")
                    .font(.system(size: 36))
                    .foregroundColor(.blue)

                // User info - format: "DisplayName : short four words"
                VStack(alignment: .leading, spacing: 2) {
                    // Combined format: "DisplayName : four words"
                    HStack(spacing: 4) {
                        Text("\(state.displayName) : \(shortFourWords(state.fourWords))")
                            .font(.headline)
                            .lineLimit(1)
                            .accessibilityIdentifier("displayName")

                        Image(systemName: showFullFourWords ? "chevron.up" : "chevron.down")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                    }
                    .onTapGesture {
                        withAnimation(.easeInOut(duration: 0.2)) {
                            showFullFourWords.toggle()
                        }
                    }

                    // Show full four words when expanded
                    if showFullFourWords {
                        Text(formatFourWords(state.fourWords))
                            .font(.system(.caption, design: .monospaced))
                            .foregroundColor(.secondary)
                            .accessibilityIdentifier("fourWords")
                    }
                }

                Spacer()

                // Network status
                Button {
                    if state.isNetworking {
                        state.stopNetworking()
                    } else {
                        state.startNetworkingWithBootstrap()
                    }
                } label: {
                    HStack(spacing: 4) {
                        Circle()
                            .fill(state.isNetworking ? Color.green : Color.orange)
                            .frame(width: 8, height: 8)
                        Text(state.isNetworking ? "Online" : "Local")
                            .font(.caption2)
                    }
                }
                .buttonStyle(.plain)
                .accessibilityIdentifier("networkStatus")

                // Network panel toggle button (globe icon)
                Button {
                    state.toggleNetworkPanel()
                } label: {
                    Image(systemName: state.activeView == .networkPanel
                        ? "globe.americas.fill"
                        : "globe")
                        .font(.system(size: 16))
                        .foregroundColor(state.activeView == .networkPanel
                            ? .accentColor
                            : .secondary)
                }
                .buttonStyle(.plain)
                .help("Network Status Panel")
                .accessibilityIdentifier("networkPanelButton")

                // Menu
                Menu {
                    Button {
                        showingContactsView = true
                    } label: {
                        Label("Contacts", systemImage: "person.2.fill")
                    }

                    Button {
                        showingNetworkSettings = true
                    } label: {
                        Label("Network Settings", systemImage: "gear")
                    }

                    Divider()

                    Button(role: .destructive) {
                        state.logout()
                    } label: {
                        Label("Sign Out", systemImage: "rectangle.portrait.and.arrow.right")
                    }
                } label: {
                    Image(systemName: "ellipsis.circle")
                        .font(.title3)
                        .foregroundColor(.secondary)
                }
                .menuStyle(.borderlessButton)
                .accessibilityIdentifier("profileMenu")
            }

            // Expanded four-word with copy button
            if showFullFourWords {
                HStack {
                    Text(formatFourWords(state.fourWords))
                        .font(.system(.caption2, design: .monospaced))
                        .foregroundColor(.secondary)
                        .textSelection(.enabled)

                    Button {
                        NSPasteboard.general.clearContents()
                        NSPasteboard.general.setString(state.fourWords, forType: .string)
                    } label: {
                        Image(systemName: "doc.on.doc")
                            .font(.caption2)
                            .foregroundColor(.blue)
                    }
                    .buttonStyle(.plain)
                    .help("Copy four-word address")

                    Spacer()
                }
                .padding(.leading, 46)  // Align with user info
            }
        }
    }

    /// Short four-word format with spaces (e.g., "bear wolf...")
    private func shortFourWords(_ fourWords: String) -> String {
        let words = fourWords.split(separator: "-")
        if words.count >= 2 {
            return "\(words[0]) \(words[1])..."
        }
        return fourWords.replacingOccurrences(of: "-", with: " ")
    }

    /// Format four words with spaces instead of hyphens
    private func formatFourWords(_ fourWords: String) -> String {
        return fourWords.replacingOccurrences(of: "-", with: " ")
    }
}

// MARK: - Entity Detail Pane

struct EntityDetailPane: View {
    @EnvironmentObject var state: AppState
    let entity: SwiftEntity

    @State private var selectedTab: DetailTab = .chat

    enum DetailTab: String, CaseIterable {
        case chat = "Chat"
        case drive = "Drive"
        case documents = "Documents"
        case details = "Details"

        var icon: String {
            switch self {
            case .chat: return "bubble.left.and.bubble.right.fill"
            case .drive: return "externaldrive.fill"
            case .documents: return "doc.text.fill"
            case .details: return "info.circle.fill"
            }
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            // Entity header
            HStack {
                Image(systemName: iconFor(entity.entityType))
                    .font(.title2)
                    .foregroundColor(colorFor(entity.entityType))

                VStack(alignment: .leading, spacing: 2) {
                    Text(entity.name)
                        .font(.title3)
                        .fontWeight(.semibold)

                    if let desc = entity.description {
                        Text(desc)
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }

                Spacer()

                // Media control buttons for calls
                MediaControlButtons(
                    entityId: entity.id,
                    entityType: entityTypeName(entity.entityType),
                    displayName: entity.name
                )
                .padding(.trailing, 8)

                Text("\(entity.members.count) members")
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.gray.opacity(0.1))
                    .cornerRadius(8)
            }
            .padding()
            .background(Color.gray.opacity(0.05))

            // Tab picker
            Picker("View", selection: $selectedTab) {
                ForEach(DetailTab.allCases, id: \.self) { tab in
                    Label(tab.rawValue, systemImage: tab.icon)
                        .tag(tab)
                }
            }
            .pickerStyle(.segmented)
            .padding(.horizontal)
            .padding(.vertical, 8)

            Divider()

            // Tab content
            Group {
                switch selectedTab {
                case .chat:
                    EmbeddedChatView(entity: entity)
                case .drive:
                    EmbeddedDriveView(entity: entity)
                case .documents:
                    EmbeddedDocumentsView(entity: entity)
                case .details:
                    EmbeddedEntityDetails(entity: entity)
                }
            }
        }
        .onChange(of: entity.id) {
            // Reset to chat when entity changes
            selectedTab = .chat
        }
    }

    private func iconFor(_ type: SwiftEntityType) -> String {
        switch type {
        case .organisation: return "building.2.fill"
        case .project: return "folder.fill"
        case .channel: return "number"
        case .group: return "person.3.fill"
        case .person: return "person.fill"
        }
    }

    private func colorFor(_ type: SwiftEntityType) -> Color {
        switch type {
        case .organisation: return .blue
        case .project: return .yellow
        case .channel: return .green
        case .group: return .purple
        case .person: return .orange
        }
    }

    private func entityTypeName(_ type: SwiftEntityType) -> String {
        switch type {
        case .organisation: return "organisation"
        case .project: return "project"
        case .channel: return "channel"
        case .group: return "group"
        case .person: return "person"
        }
    }
}

// MARK: - Embedded Chat View

struct EmbeddedChatView: View {
    @EnvironmentObject var state: AppState
    let entity: SwiftEntity
    @State private var messageText = ""

    // Only show parent messages (not thread replies) in main view
    var messages: [SwiftMessage] {
        state.getMessages(for: entity.id).filter { $0.replyToId == nil }
    }

    var body: some View {
        HStack(spacing: 0) {
            // Main chat area
            VStack(spacing: 0) {
                if messages.isEmpty {
                    VStack(spacing: 16) {
                        Spacer()
                        Image(systemName: "bubble.left.and.bubble.right")
                            .font(.system(size: 48))
                            .foregroundColor(.secondary)
                        Text("No messages yet")
                            .font(.headline)
                            .foregroundColor(.secondary)
                        Text("Send a message to start the conversation")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        Spacer()
                    }
                } else {
                    ScrollViewReader { proxy in
                        ScrollView {
                            LazyVStack(spacing: 8) {
                                ForEach(messages, id: \.id) { message in
                                    MessageBubble(
                                        message: message,
                                        isFromCurrentUser: message.author == state.fourWords,
                                        replyCount: state.getReplyCount(for: message.id),
                                        onStartThread: {
                                            state.openThread(for: message)
                                        }
                                    )
                                    .id(message.id)
                                }
                            }
                            .padding()
                        }
                        .onChange(of: messages.count) { _, _ in
                            if let lastMessage = messages.last {
                                withAnimation {
                                    proxy.scrollTo(lastMessage.id, anchor: .bottom)
                                }
                            }
                        }
                    }
                }

                Divider()

                // Message composer
                MessageComposer(text: $messageText, onSend: sendMessage)
                    .padding()
            }

            // Thread panel (Slack-style slide-in from right)
            if state.isThreadPanelOpen, let threadMessage = state.selectedThreadMessage {
                Divider()
                ThreadPanelView(parentMessage: threadMessage)
                    .transition(.move(edge: .trailing))
            }
        }
        .onAppear {
            state.loadMessages(for: entity.id)
            // Update thread reply counts
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                state.updateThreadReplyCounts(for: entity.id)
            }
        }
        .animation(.easeInOut(duration: 0.2), value: state.isThreadPanelOpen)
    }

    private func sendMessage() {
        guard !messageText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        let text = messageText
        messageText = ""
        state.sendMessage(to: entity.id, text: text)
    }
}

// MARK: - Embedded Drive View

struct EmbeddedDriveView: View {
    @EnvironmentObject var state: AppState
    let entity: SwiftEntity
    @State private var showingCreateFolder = false
    @State private var newFolderName = ""

    var files: [FileItem] {
        state.getFiles(for: entity.id)
    }

    var storageStats: (totalBytes: UInt64, fileCount: Int) {
        state.getStorageStats(for: entity.id)
    }

    var body: some View {
        VStack(spacing: 0) {
            // Stats header
            HStack {
                Image(systemName: "externaldrive.fill")
                    .foregroundColor(.blue)
                Text(formatBytes(storageStats.totalBytes))
                    .font(.caption)
                    .foregroundColor(.secondary)
                Text("\(storageStats.fileCount) files")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Spacer()
                Button {
                    showingCreateFolder = true
                } label: {
                    Label("New Folder", systemImage: "folder.badge.plus")
                        .font(.caption)
                }
                .buttonStyle(.bordered)
            }
            .padding()
            .background(Color.gray.opacity(0.05))

            Divider()

            if files.isEmpty {
                VStack(spacing: 16) {
                    Spacer()
                    Image(systemName: "folder")
                        .font(.system(size: 48))
                        .foregroundColor(.secondary)
                    Text("No files yet")
                        .font(.headline)
                        .foregroundColor(.secondary)
                    Text("Drag files here or create folders")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Spacer()
                }
            } else {
                List(files) { file in
                    FileRowView(file: file)
                }
            }
        }
        .onAppear {
            state.loadFiles(for: entity.id)
        }
        .alert("New Folder", isPresented: $showingCreateFolder) {
            TextField("Folder name", text: $newFolderName)
            Button("Cancel", role: .cancel) { newFolderName = "" }
            Button("Create") {
                if !newFolderName.isEmpty {
                    state.createFolder(in: entity.id, name: newFolderName)
                    newFolderName = ""
                }
            }
        }
        .onDrop(of: [.fileURL], isTargeted: nil) { providers in
            for provider in providers {
                _ = provider.loadObject(ofClass: URL.self) { url, _ in
                    guard let url = url else { return }
                    DispatchQueue.main.async {
                        if let data = try? Data(contentsOf: url) {
                            state.writeFile(in: entity.id, name: url.lastPathComponent, data: data)
                        }
                    }
                }
            }
            return true
        }
    }

    private func formatBytes(_ bytes: UInt64) -> String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .file
        return formatter.string(fromByteCount: Int64(bytes))
    }
}

// MARK: - Embedded Documents View

struct EmbeddedDocumentsView: View {
    @EnvironmentObject var state: AppState
    let entity: SwiftEntity
    @State private var showingCreateDocument = false
    @State private var newDocumentName = ""
    @State private var selectedDocument: DocumentItem?
    @State private var showingEditor = false

    var documents: [DocumentItem] {
        state.getDocuments(for: entity.id)
    }

    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                Image(systemName: "doc.text.fill")
                    .foregroundColor(.blue)
                Text("\(documents.count) documents")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Spacer()
                Button {
                    showingCreateDocument = true
                } label: {
                    Label("New Document", systemImage: "doc.badge.plus")
                        .font(.caption)
                }
                .buttonStyle(.bordered)
            }
            .padding()
            .background(Color.gray.opacity(0.05))

            Divider()

            if documents.isEmpty {
                VStack(spacing: 16) {
                    Spacer()
                    Image(systemName: "doc.text")
                        .font(.system(size: 48))
                        .foregroundColor(.secondary)
                    Text("No documents yet")
                        .font(.headline)
                        .foregroundColor(.secondary)
                    Text("Create a new document to get started")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Spacer()
                }
            } else {
                List(documents, id: \.id) { document in
                    Button {
                        selectedDocument = document
                        showingEditor = true
                    } label: {
                        DocumentRowView(document: document)
                    }
                    .buttonStyle(.plain)
                }
            }
        }
        .onAppear {
            state.loadDocuments(for: entity.id)
        }
        .alert("New Document", isPresented: $showingCreateDocument) {
            TextField("Document name", text: $newDocumentName)
            Button("Cancel", role: .cancel) { newDocumentName = "" }
            Button("Create") {
                if !newDocumentName.isEmpty {
                    state.createDocument(in: entity.id, name: newDocumentName)
                    newDocumentName = ""
                }
            }
        }
        .sheet(isPresented: $showingEditor) {
            if let document = selectedDocument {
                DocumentEditorView(document: document, entityId: entity.id)
                    .environmentObject(state)
            }
        }
    }
}

// MARK: - Embedded Entity Details

struct EmbeddedEntityDetails: View {
    @EnvironmentObject var state: AppState
    let entity: SwiftEntity

    var body: some View {
        Form {
            Section(header: Text("Information")) {
                LabeledContent("Name") {
                    Text(entity.name)
                }

                LabeledContent("Type") {
                    Text(displayName(for: entity.entityType))
                }

                if let desc = entity.description {
                    LabeledContent("Description") {
                        Text(desc)
                            .foregroundColor(.secondary)
                    }
                }

                LabeledContent("ID") {
                    Text(entity.id)
                        .font(.system(.caption, design: .monospaced))
                        .foregroundColor(.secondary)
                        .lineLimit(1)
                        .truncationMode(.middle)
                }

                if let parentId = entity.parentOrgId {
                    LabeledContent("Parent Org") {
                        Text(parentId)
                            .font(.system(.caption, design: .monospaced))
                            .foregroundColor(.secondary)
                            .lineLimit(1)
                            .truncationMode(.middle)
                    }
                }
            }

            if !entity.members.isEmpty {
                Section(header: Text("Members (\(entity.members.count))")) {
                    ForEach(entity.members, id: \.self) { member in
                        HStack {
                            Image(systemName: "person.fill")
                                .foregroundColor(.secondary)
                            Text(member)
                                .font(.system(.body, design: .monospaced))
                                .lineLimit(1)
                                .truncationMode(.middle)
                        }
                    }
                }
            }

            Section {
                Button(role: .destructive) {
                    state.deleteEntity(id: entity.id)
                } label: {
                    Label("Delete \(displayName(for: entity.entityType))", systemImage: "trash")
                }
            }
        }
        .formStyle(.grouped)
    }

    private func displayName(for type: SwiftEntityType) -> String {
        switch type {
        case .organisation: return "Organisation"
        case .project: return "Project"
        case .channel: return "Channel"
        case .group: return "Group"
        case .person: return "Person"
        }
    }
}

// MARK: - Welcome Pane

struct WelcomePane: View {
    @EnvironmentObject var state: AppState

    var body: some View {
        VStack(spacing: 24) {
            Spacer()

            Image(systemName: "bubble.left.and.bubble.right.fill")
                .font(.system(size: 64))
                .foregroundColor(.blue.opacity(0.5))

            Text("Welcome to Communitas")
                .font(.largeTitle)
                .fontWeight(.bold)

            Text("Select an organisation, project, channel, or group from the sidebar to get started.")
                .font(.body)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .frame(maxWidth: 400)

            HStack(spacing: 32) {
                VStack {
                    Image(systemName: "building.2.fill")
                        .font(.title)
                        .foregroundColor(.blue)
                    Text("Organisations")
                        .font(.caption)
                }

                VStack {
                    Image(systemName: "folder.fill")
                        .font(.title)
                        .foregroundColor(.yellow)
                    Text("Projects")
                        .font(.caption)
                }

                VStack {
                    Image(systemName: "number")
                        .font(.title)
                        .foregroundColor(.green)
                    Text("Channels")
                        .font(.caption)
                }

                VStack {
                    Image(systemName: "person.3.fill")
                        .font(.title)
                        .foregroundColor(.purple)
                    Text("Groups")
                        .font(.caption)
                }
            }
            .foregroundColor(.secondary)

            Spacer()

            // Quick stats
            HStack(spacing: 32) {
                StatItem(value: "\(state.entities.filter { $0.entityType == .organisation }.count)", label: "Orgs")
                StatItem(value: "\(state.entities.filter { $0.entityType == .project }.count)", label: "Projects")
                StatItem(value: "\(state.entities.filter { $0.entityType == .channel }.count)", label: "Channels")
                StatItem(value: "\(state.entities.filter { $0.entityType == .group }.count)", label: "Groups")
            }
            .padding()
            .background(Color.gray.opacity(0.1))
            .cornerRadius(12)

            Spacer()
        }
        .padding()
    }
}

struct StatItem: View {
    let value: String
    let label: String

    var body: some View {
        VStack(spacing: 4) {
            Text(value)
                .font(.title)
                .fontWeight(.bold)
            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
        }
    }
}

// MARK: - Error View

struct ErrorView: View {
    let error: String
    let onRetry: () -> Void

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "exclamationmark.triangle")
                .font(.largeTitle)
                .foregroundColor(.red)
            Text("Error")
                .font(.headline)
            Text(error)
                .multilineTextAlignment(.center)
                .foregroundColor(.secondary)
                .accessibilityIdentifier("errorMessage")
            Button("Retry", action: onRetry)
                .buttonStyle(.borderedProminent)
        }
        .padding()
    }
}

// MARK: - Loading View

struct LoadingView: View {
    var body: some View {
        VStack {
            ProgressView()
                .scaleEffect(1.5)
            Text("Loading...")
                .padding(.top)
                .accessibilityIdentifier("loadingIndicator")
        }
    }
}

struct EntityDetailView: View {
    let entity: SwiftEntity
    var onDelete: () -> Void
    @Environment(\.dismiss) var dismiss
    @State private var showingDeleteConfirmation = false

    var body: some View {
        NavigationView {
            Form {
                Section(header: Text("Entity Information")) {
                    HStack {
                        Image(systemName: iconFor(entity.entityType))
                            .foregroundColor(.blue)
                            .font(.title2)
                        VStack(alignment: .leading) {
                            Text(entity.name)
                                .font(.headline)
                            Text(displayName(for: entity.entityType))
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }
                    .padding(.vertical, 4)

                    if let desc = entity.description {
                        LabeledContent("Description") {
                            Text(desc)
                                .foregroundColor(.secondary)
                        }
                    }
                }

                Section(header: Text("Details")) {
                    LabeledContent("ID") {
                        Text(entity.id)
                            .font(.system(.caption, design: .monospaced))
                            .foregroundColor(.secondary)
                            .lineLimit(1)
                            .truncationMode(.middle)
                    }

                    LabeledContent("Members") {
                        Text("\(entity.members.count)")
                    }

                    if let parentId = entity.parentOrgId {
                        LabeledContent("Parent Org") {
                            Text(parentId)
                                .font(.system(.caption, design: .monospaced))
                                .foregroundColor(.secondary)
                                .lineLimit(1)
                                .truncationMode(.middle)
                        }
                    }
                }

                if !entity.members.isEmpty {
                    Section(header: Text("Members (\(entity.members.count))")) {
                        ForEach(entity.members, id: \.self) { member in
                            HStack {
                                Image(systemName: "person.fill")
                                    .foregroundColor(.secondary)
                                Text(member)
                                    .font(.system(.body, design: .monospaced))
                                    .lineLimit(1)
                                    .truncationMode(.middle)
                            }
                        }
                    }
                }

                Section {
                    Button(role: .destructive) {
                        showingDeleteConfirmation = true
                    } label: {
                        HStack {
                            Image(systemName: "trash")
                            Text("Delete \(displayName(for: entity.entityType))")
                        }
                        .frame(maxWidth: .infinity)
                    }
                    .accessibilityIdentifier("deleteEntityButton")
                }
            }
            .navigationTitle(entity.name)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
            .alert("Delete \(entity.name)?", isPresented: $showingDeleteConfirmation) {
                Button("Cancel", role: .cancel) { }
                Button("Delete", role: .destructive) {
                    onDelete()
                    dismiss()
                }
            } message: {
                Text("This action cannot be undone. All data associated with this \(displayName(for: entity.entityType).lowercased()) will be permanently removed.")
            }
        }
        .frame(minWidth: 400, minHeight: 400)
    }

    private func iconFor(_ type: SwiftEntityType) -> String {
        switch type {
        case .group: return "person.3.fill"
        case .channel: return "number"
        case .project: return "folder.fill"
        case .organisation: return "building.2.fill"
        case .person: return "person.fill"
        }
    }

    private func displayName(for type: SwiftEntityType) -> String {
        switch type {
        case .group: return "Group"
        case .channel: return "Channel"
        case .project: return "Project"
        case .organisation: return "Organisation"
        case .person: return "Person"
        }
    }
}

struct CreateEntityView: View {
    @Binding var name: String
    @Binding var description: String
    @Binding var selectedType: SwiftEntityType
    var onCreate: () -> Void
    @Environment(\.dismiss) var dismiss

    var body: some View {
        NavigationView {
            Form {
                Section(header: Text("Entity Type")) {
                    Picker("Type", selection: $selectedType) {
                        Label("Group", systemImage: "person.3.fill")
                            .tag(SwiftEntityType.group)
                        Label("Channel", systemImage: "number")
                            .tag(SwiftEntityType.channel)
                        Label("Project", systemImage: "folder.fill")
                            .tag(SwiftEntityType.project)
                        Label("Organisation", systemImage: "building.2.fill")
                            .tag(SwiftEntityType.organisation)
                    }
                    .pickerStyle(.inline)
                    .labelsHidden()
                }

                Section(header: Text("Details")) {
                    TextField("Name", text: $name)
                        .accessibilityIdentifier("entityNameField")
                    TextField("Description (optional)", text: $description)
                        .accessibilityIdentifier("entityDescriptionField")
                }

                Section(footer: Text(entityTypeDescription)) {
                    Button(action: onCreate) {
                        HStack {
                            Image(systemName: iconFor(selectedType))
                            Text("Create \(displayName(for: selectedType))")
                        }
                        .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(name.isEmpty)
                    .accessibilityIdentifier("createEntityButton")
                }
            }
            .navigationTitle("New Entity")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
            }
        }
        .frame(minWidth: 400, minHeight: 450)
    }

    private var entityTypeDescription: String {
        switch selectedType {
        case .group:
            return "Groups are private spaces for team collaboration with shared files and messaging."
        case .channel:
            return "Channels are topic-based discussions within a group or organisation."
        case .project:
            return "Projects organize work with tasks, files, and team members around a specific goal."
        case .organisation:
            return "Organisations are top-level containers that can hold multiple groups and projects."
        case .person:
            return "Individual identity for direct messaging and personal files."
        }
    }

    private func iconFor(_ type: SwiftEntityType) -> String {
        switch type {
        case .group: return "person.3.fill"
        case .channel: return "number"
        case .project: return "folder.fill"
        case .organisation: return "building.2.fill"
        case .person: return "person.fill"
        }
    }

    private func displayName(for type: SwiftEntityType) -> String {
        switch type {
        case .group: return "Group"
        case .channel: return "Channel"
        case .project: return "Project"
        case .organisation: return "Organisation"
        case .person: return "Person"
        }
    }
}

// MARK: - Chat View

struct ChatView: View {
    @EnvironmentObject var state: AppState
    let entity: SwiftEntity
    @Environment(\.dismiss) var dismiss
    @State private var messageText = ""
    @State private var isLoading = false

    // Only show parent messages (not thread replies) in main view
    var messages: [SwiftMessage] {
        state.getMessages(for: entity.id).filter { $0.replyToId == nil }
    }

    var body: some View {
        NavigationView {
            HStack(spacing: 0) {
                // Main chat area
                VStack(spacing: 0) {
                    // Messages list
                    if messages.isEmpty && !isLoading {
                        VStack(spacing: 16) {
                            Spacer()
                            Image(systemName: "bubble.left.and.bubble.right")
                                .font(.system(size: 48))
                                .foregroundColor(.secondary)
                            Text("No messages yet")
                                .font(.headline)
                                .foregroundColor(.secondary)
                            Text("Send a message to start the conversation")
                                .font(.caption)
                                .foregroundColor(.secondary)
                            Spacer()
                        }
                        .frame(maxWidth: .infinity)
                    } else {
                        ScrollViewReader { proxy in
                            ScrollView {
                                LazyVStack(spacing: 8) {
                                    ForEach(messages, id: \.id) { message in
                                        MessageBubble(
                                            message: message,
                                            isFromCurrentUser: message.author == state.fourWords,
                                            replyCount: state.getReplyCount(for: message.id),
                                            onStartThread: {
                                                state.openThread(for: message)
                                            }
                                        )
                                        .id(message.id)
                                    }
                                }
                                .padding()
                            }
                            .onChange(of: messages.count) { _, _ in
                                if let lastMessage = messages.last {
                                    withAnimation {
                                        proxy.scrollTo(lastMessage.id, anchor: .bottom)
                                    }
                                }
                            }
                            .onAppear {
                                if let lastMessage = messages.last {
                                    proxy.scrollTo(lastMessage.id, anchor: .bottom)
                                }
                            }
                        }
                    }

                    Divider()

                    // Message composer
                    MessageComposer(text: $messageText, onSend: sendMessage)
                        .padding()
                }

                // Thread panel (Slack-style slide-in from right)
                if state.isThreadPanelOpen, let threadMessage = state.selectedThreadMessage {
                    Divider()
                    ThreadPanelView(parentMessage: threadMessage)
                        .transition(.move(edge: .trailing))
                }
            }
            .navigationTitle(entity.name)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Close") {
                        state.closeThread() // Close thread when closing chat
                        dismiss()
                    }
                }
                ToolbarItem(placement: .principal) {
                    // Media control buttons for calls in entity chat
                    MediaControlButtons(
                        entityId: entity.id,
                        entityType: entityTypeName(entity.entityType),
                        displayName: entity.name
                    )
                }
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        state.loadMessages(for: entity.id)
                        state.updateThreadReplyCounts(for: entity.id)
                    } label: {
                        Image(systemName: "arrow.clockwise")
                    }
                }
            }
        }
        .frame(minWidth: 500, minHeight: 500)
        .onAppear {
            state.loadMessages(for: entity.id)
            // Update thread reply counts
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                state.updateThreadReplyCounts(for: entity.id)
            }
        }
        .animation(.easeInOut(duration: 0.2), value: state.isThreadPanelOpen)
    }

    private func entityTypeName(_ type: SwiftEntityType) -> String {
        switch type {
        case .organisation: return "organisation"
        case .project: return "project"
        case .channel: return "channel"
        case .group: return "group"
        case .person: return "person"
        }
    }

    private func sendMessage() {
        guard !messageText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        let text = messageText
        messageText = ""
        state.sendMessage(to: entity.id, text: text)
    }
}

// MARK: - Contact Chat View (for direct messages with contacts)

struct ContactChatView: View {
    @EnvironmentObject var state: AppState
    let fourWords: String
    let displayName: String?
    @State private var messageText = ""

    var title: String {
        displayName ?? shortFourWords(fourWords)
    }

    var messages: [SwiftMessage] {
        state.getDirectMessages(for: fourWords)
    }

    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                Button {
                    state.navigateHome()
                } label: {
                    Image(systemName: "chevron.left")
                        .font(.title3)
                }
                .buttonStyle(.plain)
                .padding(.leading)
                .accessibilityIdentifier("backButton")

                VStack(alignment: .leading, spacing: 2) {
                    Text(title)
                        .font(.headline)
                    Text(fourWords)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .padding(.leading, 8)

                Spacer()

                // Media control buttons for calls
                MediaControlButtons(
                    peerFourWords: fourWords,
                    displayName: displayName
                )
                .padding(.trailing)
            }
            .padding(.vertical, 12)
            .background(Color(.windowBackgroundColor))

            Divider()

            // Messages list
            if messages.isEmpty {
                VStack(spacing: 16) {
                    Spacer()
                    Image(systemName: "bubble.left.and.bubble.right")
                        .font(.system(size: 48))
                        .foregroundColor(.secondary)
                    Text("No messages yet")
                        .font(.headline)
                        .foregroundColor(.secondary)
                    Text("Send a message to start chatting with \(title)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Spacer()
                }
                .frame(maxWidth: .infinity)
            } else {
                ScrollViewReader { proxy in
                    ScrollView {
                        LazyVStack(spacing: 8) {
                            ForEach(messages, id: \.id) { message in
                                MessageBubble(
                                    message: message,
                                    isFromCurrentUser: message.author == state.fourWords
                                )
                                .id(message.id)
                            }
                        }
                        .padding()
                    }
                    .onChange(of: messages.count) {
                        if let lastMessage = messages.last {
                            withAnimation {
                                proxy.scrollTo(lastMessage.id, anchor: .bottom)
                            }
                        }
                    }
                    .onAppear {
                        if let lastMessage = messages.last {
                            proxy.scrollTo(lastMessage.id, anchor: .bottom)
                        }
                    }
                }
            }

            Divider()

            // Message composer
            MessageComposer(text: $messageText, onSend: sendMessage)
                .padding()
        }
        .accessibilityIdentifier("contactChatView_\(fourWords)")
        .onAppear {
            state.loadDirectMessages(for: fourWords)
        }
    }

    private func sendMessage() {
        guard !messageText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        let text = messageText
        messageText = ""
        state.sendDirectMessage(to: fourWords, text: text)
    }

    private func shortFourWords(_ fourWords: String) -> String {
        let words = fourWords.split(separator: "-")
        if words.count >= 2 {
            return "\(words[0].capitalized) \(words[1].capitalized)"
        }
        return fourWords.replacingOccurrences(of: "-", with: " ").capitalized
    }
}

// MARK: - Message Bubble

struct MessageBubble: View {
    let message: SwiftMessage
    let isFromCurrentUser: Bool
    var replyCount: Int = 0
    var onStartThread: (() -> Void)?
    @State private var isHovered: Bool = false

    var body: some View {
        HStack {
            if isFromCurrentUser { Spacer() }

            VStack(alignment: isFromCurrentUser ? .trailing : .leading, spacing: 4) {
                // Author name (for non-current user)
                if !isFromCurrentUser {
                    Text(shortFourWords(message.author))
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }

                // Message content
                Text(message.text)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)
                    .background(isFromCurrentUser ? Color.blue : Color.gray.opacity(0.2))
                    .foregroundColor(isFromCurrentUser ? .white : .primary)
                    .cornerRadius(16)

                // Thread indicator and reply button row
                HStack(spacing: 8) {
                    // Timestamp
                    Text(formatTimestamp(message.createdAt))
                        .font(.caption2)
                        .foregroundColor(.secondary)

                    // Thread reply count badge (Slack-style)
                    if replyCount > 0 {
                        Button(action: { onStartThread?() }) {
                            HStack(spacing: 4) {
                                Image(systemName: "bubble.left.and.bubble.right.fill")
                                    .font(.caption2)
                                Text("\(replyCount) \(replyCount == 1 ? "reply" : "replies")")
                                    .font(.caption2)
                            }
                            .foregroundColor(.blue)
                        }
                        .buttonStyle(.plain)
                    }

                    // Reply in thread button (shows on hover)
                    if isHovered && onStartThread != nil && replyCount == 0 {
                        Button(action: { onStartThread?() }) {
                            HStack(spacing: 2) {
                                Image(systemName: "arrowshape.turn.up.left")
                                    .font(.caption2)
                                Text("Reply")
                                    .font(.caption2)
                            }
                            .foregroundColor(.secondary)
                        }
                        .buttonStyle(.plain)
                        .transition(.opacity)
                    }
                }
            }
            .frame(maxWidth: 280, alignment: isFromCurrentUser ? .trailing : .leading)
            .onHover { hovering in
                withAnimation(.easeInOut(duration: 0.15)) {
                    isHovered = hovering
                }
            }

            if !isFromCurrentUser { Spacer() }
        }
        .accessibilityIdentifier("messageBubble_\(message.id)")
    }

    private func shortFourWords(_ fourWords: String) -> String {
        let words = fourWords.split(separator: "-")
        if words.count >= 2 {
            return "\(words[0]) \(words[1])"
        }
        return fourWords.replacingOccurrences(of: "-", with: " ")
    }

    private func formatTimestamp(_ timestamp: Int64) -> String {
        let date = Date(timeIntervalSince1970: TimeInterval(timestamp) / 1000)
        let formatter = DateFormatter()
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }
}

// MARK: - Message Composer

struct MessageComposer: View {
    @Binding var text: String
    var onSend: () -> Void

    var body: some View {
        HStack(spacing: 12) {
            TextField("Type a message...", text: $text)
                .textFieldStyle(.roundedBorder)
                .onSubmit {
                    if !text.isEmpty {
                        onSend()
                    }
                }
                .accessibilityIdentifier("messageTextField")

            Button(action: onSend) {
                Image(systemName: "arrow.up.circle.fill")
                    .font(.title2)
                    .foregroundColor(text.isEmpty ? .gray : .blue)
            }
            .disabled(text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
            .buttonStyle(.plain)
            .accessibilityIdentifier("sendMessageButton")
        }
    }
}

// MARK: - Thread Panel View (Slack-style)

struct ThreadPanelView: View {
    @EnvironmentObject var state: AppState
    let parentMessage: SwiftMessage
    @State private var replyText: String = ""

    var threadReplies: [SwiftMessage] {
        state.getThreadMessages(for: parentMessage.id)
    }

    var body: some View {
        VStack(spacing: 0) {
            // Header with close button
            HStack {
                Text("Thread")
                    .font(.headline)
                Spacer()
                Button(action: { state.closeThread() }) {
                    Image(systemName: "xmark")
                        .foregroundColor(.secondary)
                }
                .buttonStyle(.plain)
            }
            .padding()
            .background(Color.gray.opacity(0.1))

            Divider()

            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(spacing: 12) {
                        // Parent message (highlighted)
                        VStack(alignment: .leading, spacing: 8) {
                            HStack {
                                Text(shortFourWords(parentMessage.author))
                                    .font(.subheadline)
                                    .fontWeight(.semibold)
                                Spacer()
                                Text(formatTimestamp(parentMessage.createdAt))
                                    .font(.caption2)
                                    .foregroundColor(.secondary)
                            }
                            Text(parentMessage.text)
                                .font(.body)
                        }
                        .padding()
                        .background(Color.blue.opacity(0.1))
                        .cornerRadius(8)
                        .padding(.horizontal)
                        .padding(.top, 8)

                        // Replies count divider
                        if !threadReplies.isEmpty {
                            HStack {
                                Rectangle()
                                    .fill(Color.gray.opacity(0.3))
                                    .frame(height: 1)
                                Text("\(threadReplies.count) \(threadReplies.count == 1 ? "reply" : "replies")")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                                Rectangle()
                                    .fill(Color.gray.opacity(0.3))
                                    .frame(height: 1)
                            }
                            .padding(.horizontal)
                            .padding(.vertical, 8)
                        }

                        // Thread replies
                        ForEach(threadReplies, id: \.id) { reply in
                            ThreadReplyBubble(message: reply, isFromCurrentUser: reply.author == state.fourWords)
                                .id(reply.id)
                        }
                    }
                }
                .onChange(of: threadReplies.count) { _, _ in
                    if let lastReply = threadReplies.last {
                        withAnimation {
                            proxy.scrollTo(lastReply.id, anchor: .bottom)
                        }
                    }
                }
            }

            Divider()

            // Thread reply composer
            HStack(spacing: 12) {
                TextField("Reply in thread...", text: $replyText)
                    .textFieldStyle(.roundedBorder)
                    .onSubmit {
                        sendReply()
                    }

                Button(action: sendReply) {
                    Image(systemName: "arrow.up.circle.fill")
                        .font(.title2)
                        .foregroundColor(replyText.isEmpty ? .gray : .blue)
                }
                .disabled(replyText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                .buttonStyle(.plain)
            }
            .padding()
        }
        .frame(width: 350)
        .background(Color(NSColor.windowBackgroundColor))
    }

    private func sendReply() {
        guard !replyText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        state.sendThreadReply(to: parentMessage, text: replyText)
        replyText = ""
    }

    private func shortFourWords(_ fourWords: String) -> String {
        let words = fourWords.split(separator: "-")
        if words.count >= 2 {
            return "\(words[0]) \(words[1])"
        }
        return fourWords.replacingOccurrences(of: "-", with: " ")
    }

    private func formatTimestamp(_ timestamp: Int64) -> String {
        let date = Date(timeIntervalSince1970: TimeInterval(timestamp) / 1000)
        let formatter = DateFormatter()
        formatter.dateStyle = .short
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }
}

// MARK: - Thread Reply Bubble

struct ThreadReplyBubble: View {
    let message: SwiftMessage
    let isFromCurrentUser: Bool

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            // Avatar placeholder
            Circle()
                .fill(isFromCurrentUser ? Color.blue : Color.gray.opacity(0.3))
                .frame(width: 28, height: 28)
                .overlay(
                    Text(String(message.author.prefix(1)).uppercased())
                        .font(.caption)
                        .foregroundColor(isFromCurrentUser ? .white : .primary)
                )

            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text(shortFourWords(message.author))
                        .font(.caption)
                        .fontWeight(.medium)
                    Text(formatTimestamp(message.createdAt))
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }
                Text(message.text)
                    .font(.body)
            }
            Spacer()
        }
        .padding(.horizontal)
        .padding(.vertical, 4)
    }

    private func shortFourWords(_ fourWords: String) -> String {
        let words = fourWords.split(separator: "-")
        if words.count >= 2 {
            return "\(words[0]) \(words[1])"
        }
        return fourWords.replacingOccurrences(of: "-", with: " ")
    }

    private func formatTimestamp(_ timestamp: Int64) -> String {
        let date = Date(timeIntervalSince1970: TimeInterval(timestamp) / 1000)
        let formatter = DateFormatter()
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }
}

// MARK: - Drive View

struct DriveView: View {
    @EnvironmentObject var state: AppState
    let entity: SwiftEntity
    @Environment(\.dismiss) var dismiss
    @State private var showingCreateFolder = false
    @State private var newFolderName = ""
    @State private var selectedFile: FileItem?
    @State private var showingDeleteConfirmation = false

    var files: [FileItem] {
        state.getFiles(for: entity.id)
    }

    var storageStats: (totalBytes: UInt64, fileCount: Int) {
        state.getStorageStats(for: entity.id)
    }

    var body: some View {
        NavigationView {
            VStack(spacing: 0) {
                // Storage stats header
                HStack {
                    Image(systemName: "externaldrive.fill")
                        .foregroundColor(.blue)
                    Text("\(formatBytes(storageStats.totalBytes)) used")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text("\(storageStats.fileCount) files")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Spacer()
                    Button {
                        showingCreateFolder = true
                    } label: {
                        Image(systemName: "folder.badge.plus")
                    }
                    .buttonStyle(.plain)
                }
                .padding()
                .background(Color.gray.opacity(0.1))

                Divider()

                // File list
                if files.isEmpty {
                    VStack(spacing: 16) {
                        Spacer()
                        Image(systemName: "folder")
                            .font(.system(size: 48))
                            .foregroundColor(.secondary)
                        Text("No files yet")
                            .font(.headline)
                            .foregroundColor(.secondary)
                        Text("Right-click to create folders or drag files here")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        Spacer()
                    }
                    .frame(maxWidth: .infinity)
                } else {
                    List(files, selection: $selectedFile) { file in
                        FileRowView(file: file)
                            .contextMenu {
                                if !file.isDirectory {
                                    Button {
                                        if let data = state.readFile(in: entity.id, path: file.path) {
                                            let savePanel = NSSavePanel()
                                            savePanel.nameFieldStringValue = file.name
                                            if savePanel.runModal() == .OK, let url = savePanel.url {
                                                try? data.write(to: url)
                                            }
                                        }
                                    } label: {
                                        Label("Export", systemImage: "square.and.arrow.up")
                                    }
                                }
                                Divider()
                                Button(role: .destructive) {
                                    selectedFile = file
                                    showingDeleteConfirmation = true
                                } label: {
                                    Label("Delete", systemImage: "trash")
                                }
                            }
                    }
                }
            }
            .navigationTitle("\(entity.name) Drive")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Close") {
                        dismiss()
                    }
                }
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        state.loadFiles(for: entity.id)
                    } label: {
                        Image(systemName: "arrow.clockwise")
                    }
                }
            }
            .alert("Create Folder", isPresented: $showingCreateFolder) {
                TextField("Folder name", text: $newFolderName)
                Button("Cancel", role: .cancel) {
                    newFolderName = ""
                }
                Button("Create") {
                    if !newFolderName.isEmpty {
                        state.createFolder(in: entity.id, name: newFolderName)
                        newFolderName = ""
                    }
                }
            }
            .alert("Delete \(selectedFile?.name ?? "")?", isPresented: $showingDeleteConfirmation) {
                Button("Cancel", role: .cancel) { }
                Button("Delete", role: .destructive) {
                    if let file = selectedFile {
                        state.deleteFile(in: entity.id, path: file.path)
                    }
                }
            } message: {
                Text("This action cannot be undone.")
            }
            .onDrop(of: [.fileURL], isTargeted: nil) { providers in
                for provider in providers {
                    _ = provider.loadObject(ofClass: URL.self) { url, _ in
                        guard let url = url else { return }
                        DispatchQueue.main.async {
                            if let data = try? Data(contentsOf: url) {
                                state.writeFile(in: entity.id, name: url.lastPathComponent, data: data)
                            }
                        }
                    }
                }
                return true
            }
        }
        .frame(minWidth: 500, minHeight: 400)
        .onAppear {
            state.loadFiles(for: entity.id)
        }
    }

    private func formatBytes(_ bytes: UInt64) -> String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .file
        return formatter.string(fromByteCount: Int64(bytes))
    }
}

// MARK: - File Row View

struct FileRowView: View {
    let file: FileItem

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: file.isDirectory ? "folder.fill" : iconForFile(file.name))
                .foregroundColor(file.isDirectory ? .blue : .secondary)
                .font(.title3)
                .frame(width: 24)

            VStack(alignment: .leading, spacing: 2) {
                Text(file.name)
                    .font(.body)
                    .lineLimit(1)

                HStack(spacing: 8) {
                    if !file.isDirectory {
                        Text(formatBytes(file.sizeBytes))
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    Text(formatDate(file.modifiedAt))
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }

            Spacer()

            if file.isDirectory {
                Image(systemName: "chevron.right")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .padding(.vertical, 4)
    }

    private func iconForFile(_ name: String) -> String {
        let ext = (name as NSString).pathExtension.lowercased()
        switch ext {
        case "pdf": return "doc.fill"
        case "txt", "md", "rtf": return "doc.text.fill"
        case "doc", "docx": return "doc.richtext.fill"
        case "xls", "xlsx", "csv": return "tablecells.fill"
        case "ppt", "pptx", "key": return "rectangle.on.rectangle.fill"
        case "jpg", "jpeg", "png", "gif", "heic", "webp": return "photo.fill"
        case "mp4", "mov", "avi", "mkv": return "film.fill"
        case "mp3", "wav", "aac", "m4a": return "music.note"
        case "zip", "tar", "gz", "rar": return "archivebox.fill"
        case "swift", "rs", "py", "js", "ts", "html", "css": return "chevron.left.forwardslash.chevron.right"
        case "json", "xml", "yaml", "toml": return "curlybraces"
        default: return "doc.fill"
        }
    }

    private func formatBytes(_ bytes: UInt64) -> String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .file
        return formatter.string(fromByteCount: Int64(bytes))
    }

    private func formatDate(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .short
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }
}

// MARK: - Document List View

struct DocumentListView: View {
    @EnvironmentObject var state: AppState
    let entity: SwiftEntity
    @Environment(\.dismiss) var dismiss
    @State private var showingCreateDocument = false
    @State private var newDocumentName = ""
    @State private var selectedDocument: DocumentItem?
    @State private var showingEditor = false
    @State private var showingDeleteConfirmation = false
    @State private var showingRenameAlert = false
    @State private var renameText = ""

    var documents: [DocumentItem] {
        state.getDocuments(for: entity.id)
    }

    var body: some View {
        NavigationView {
            VStack(spacing: 0) {
                // Header with document count
                HStack {
                    Image(systemName: "doc.text.fill")
                        .foregroundColor(.blue)
                    Text("\(documents.count) documents")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Spacer()
                    Button {
                        showingCreateDocument = true
                    } label: {
                        Image(systemName: "doc.badge.plus")
                    }
                    .buttonStyle(.plain)
                }
                .padding()
                .background(Color.gray.opacity(0.1))

                Divider()

                // Document list
                if documents.isEmpty {
                    VStack(spacing: 16) {
                        Spacer()
                        Image(systemName: "doc.text")
                            .font(.system(size: 48))
                            .foregroundColor(.secondary)
                        Text("No documents yet")
                            .font(.headline)
                            .foregroundColor(.secondary)
                        Text("Tap + to create a new document")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        Spacer()
                    }
                    .frame(maxWidth: .infinity)
                } else {
                    List(documents, id: \.id) { document in
                        Button {
                            selectedDocument = document
                            showingEditor = true
                        } label: {
                            DocumentRowView(document: document)
                        }
                        .buttonStyle(.plain)
                        .contextMenu {
                            Button {
                                selectedDocument = document
                                renameText = document.name
                                showingRenameAlert = true
                            } label: {
                                Label("Rename", systemImage: "pencil")
                            }
                            Divider()
                            Button(role: .destructive) {
                                selectedDocument = document
                                showingDeleteConfirmation = true
                            } label: {
                                Label("Delete", systemImage: "trash")
                            }
                        }
                    }
                }
            }
            .navigationTitle("\(entity.name) Documents")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Close") {
                        dismiss()
                    }
                }
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        state.loadDocuments(for: entity.id)
                    } label: {
                        Image(systemName: "arrow.clockwise")
                    }
                }
            }
            .alert("New Document", isPresented: $showingCreateDocument) {
                TextField("Document name", text: $newDocumentName)
                Button("Cancel", role: .cancel) {
                    newDocumentName = ""
                }
                Button("Create") {
                    if !newDocumentName.isEmpty {
                        state.createDocument(in: entity.id, name: newDocumentName)
                        newDocumentName = ""
                    }
                }
            }
            .alert("Rename Document", isPresented: $showingRenameAlert) {
                TextField("New name", text: $renameText)
                Button("Cancel", role: .cancel) {
                    renameText = ""
                }
                Button("Rename") {
                    if let doc = selectedDocument, !renameText.isEmpty {
                        state.renameDocument(documentId: doc.id, entityId: entity.id, newName: renameText)
                        renameText = ""
                    }
                }
            }
            .alert("Delete \(selectedDocument?.name ?? "")?", isPresented: $showingDeleteConfirmation) {
                Button("Cancel", role: .cancel) { }
                Button("Delete", role: .destructive) {
                    if let doc = selectedDocument {
                        state.deleteDocument(documentId: doc.id, entityId: entity.id)
                    }
                }
            } message: {
                Text("This action cannot be undone.")
            }
            .sheet(isPresented: $showingEditor) {
                if let document = selectedDocument {
                    DocumentEditorView(
                        document: document,
                        entityId: entity.id
                    )
                    .environmentObject(state)
                }
            }
        }
        .frame(minWidth: 500, minHeight: 400)
        .onAppear {
            state.loadDocuments(for: entity.id)
        }
    }
}

// MARK: - Document Row View

struct DocumentRowView: View {
    let document: DocumentItem

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: "doc.text.fill")
                .foregroundColor(.blue)
                .font(.title3)
                .frame(width: 24)

            VStack(alignment: .leading, spacing: 2) {
                Text(document.name)
                    .font(.body)
                    .lineLimit(1)

                HStack(spacing: 8) {
                    Text(formatWordCount(document.content))
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text(formatDate(document.modifiedAt))
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }

            Spacer()

            Text(shortFourWords(document.authorFourWords))
                .font(.caption2)
                .foregroundColor(.secondary)
        }
        .padding(.vertical, 4)
    }

    private func formatWordCount(_ content: String) -> String {
        let wordCount = content.split(separator: " ").count
        return "\(wordCount) words"
    }

    private func formatDate(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .short
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }

    private func shortFourWords(_ fourWords: String) -> String {
        let words = fourWords.split(separator: "-")
        if words.count >= 2 {
            return "\(words[0]) \(words[1])"
        }
        return fourWords.replacingOccurrences(of: "-", with: " ")
    }
}

// MARK: - Document Editor View

struct DocumentEditorView: View {
    @EnvironmentObject var state: AppState
    let document: DocumentItem
    let entityId: String
    @Environment(\.dismiss) var dismiss
    @State private var content: String = ""
    @State private var hasChanges = false
    @State private var showingDiscardAlert = false

    var body: some View {
        NavigationView {
            VStack(spacing: 0) {
                // Document info header
                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text(document.name)
                            .font(.headline)
                        HStack(spacing: 8) {
                            Text("by \(shortFourWords(document.authorFourWords))")
                                .font(.caption)
                                .foregroundColor(.secondary)
                            Text("Modified \(formatDate(document.modifiedAt))")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }
                    Spacer()
                    if hasChanges {
                        Text("Unsaved")
                            .font(.caption)
                            .foregroundColor(.orange)
                            .padding(.horizontal, 8)
                            .padding(.vertical, 2)
                            .background(Color.orange.opacity(0.2))
                            .cornerRadius(4)
                    }
                }
                .padding()
                .background(Color.gray.opacity(0.1))

                Divider()

                // Text editor
                TextEditor(text: $content)
                    .font(.system(.body, design: .monospaced))
                    .padding(8)
                    .onChange(of: content) {
                        hasChanges = content != document.content
                    }

                Divider()

                // Status bar
                HStack {
                    Text("\(content.split(separator: " ").count) words")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text("•")
                        .foregroundColor(.secondary)
                    Text("\(content.count) characters")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Spacer()
                    Button("Save") {
                        saveDocument()
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(!hasChanges)
                }
                .padding()
            }
            .navigationTitle("Edit Document")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Close") {
                        if hasChanges {
                            showingDiscardAlert = true
                        } else {
                            dismiss()
                        }
                    }
                }
                ToolbarItem(placement: .primaryAction) {
                    Button("Save") {
                        saveDocument()
                    }
                    .disabled(!hasChanges)
                }
            }
            .alert("Discard Changes?", isPresented: $showingDiscardAlert) {
                Button("Cancel", role: .cancel) { }
                Button("Discard", role: .destructive) {
                    dismiss()
                }
                Button("Save & Close") {
                    saveDocument()
                    dismiss()
                }
            } message: {
                Text("You have unsaved changes that will be lost.")
            }
        }
        .frame(minWidth: 600, minHeight: 500)
        .onAppear {
            content = document.content
        }
    }

    private func saveDocument() {
        state.updateDocumentContent(documentId: document.id, entityId: entityId, content: content)
        hasChanges = false
    }

    private func shortFourWords(_ fourWords: String) -> String {
        let words = fourWords.split(separator: "-")
        if words.count >= 2 {
            return "\(words[0]) \(words[1])"
        }
        return fourWords.replacingOccurrences(of: "-", with: " ")
    }

    private func formatDate(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }
}

// MARK: - Contacts View

struct ContactsView: View {
    @EnvironmentObject var state: AppState
    @Environment(\.dismiss) var dismiss
    @State private var showingAddContact = false
    @State private var newContactFourWords = ""
    @State private var searchText = ""
    @State private var isSearching = false
    @State private var searchResult: String?

    var filteredContacts: [ContactItem] {
        if searchText.isEmpty {
            return state.contacts
        } else {
            return state.contacts.filter { contact in
                (contact.fourWords?.localizedCaseInsensitiveContains(searchText) ?? false) ||
                (contact.displayName?.localizedCaseInsensitiveContains(searchText) ?? false)
            }
        }
    }

    var body: some View {
        NavigationView {
            VStack(spacing: 0) {
                // Network status header
                HStack {
                    Circle()
                        .fill(state.isNetworking ? Color.green : Color.orange)
                        .frame(width: 8, height: 8)
                    Text(state.isNetworking ? "Online" : "Offline")
                        .font(.caption)
                        .foregroundColor(.secondary)

                    if let identity = state.connectionIdentity {
                        Text(identity)
                            .font(.caption2)
                            .foregroundColor(.secondary)
                            .lineLimit(1)
                            .truncationMode(.middle)
                    }

                    Spacer()

                    Text("\(state.contacts.count) contacts")
                        .font(.caption)
                        .foregroundColor(.secondary)

                    Button {
                        showingAddContact = true
                    } label: {
                        Image(systemName: "person.badge.plus")
                    }
                    .buttonStyle(.plain)
                    .disabled(!state.isNetworking)
                    .accessibilityIdentifier("addContactButton")
                }
                .padding()
                .background(Color.gray.opacity(0.1))

                Divider()

                // Search bar
                HStack {
                    Image(systemName: "magnifyingglass")
                        .foregroundColor(.secondary)
                    TextField("Search contacts by four-word address...", text: $searchText)
                        .textFieldStyle(.plain)
                        .accessibilityIdentifier("contactSearchField")
                }
                .padding(.horizontal)
                .padding(.vertical, 8)
                .background(Color.gray.opacity(0.05))

                Divider()

                // Contacts list
                if !state.isNetworking {
                    VStack(spacing: 16) {
                        Spacer()
                        Image(systemName: "wifi.slash")
                            .font(.system(size: 48))
                            .foregroundColor(.secondary)
                        Text("Networking not active")
                            .font(.headline)
                            .foregroundColor(.secondary)
                        Text("Start networking to discover contacts")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        Button("Start Networking") {
                            state.startNetworkingWithBootstrap()
                        }
                        .buttonStyle(.borderedProminent)
                        Spacer()
                    }
                    .frame(maxWidth: .infinity)
                } else if filteredContacts.isEmpty {
                    VStack(spacing: 16) {
                        Spacer()
                        Image(systemName: "person.2.slash")
                            .font(.system(size: 48))
                            .foregroundColor(.secondary)
                        Text("No contacts yet")
                            .font(.headline)
                            .foregroundColor(.secondary)
                        Text("Add contacts by their four-word address")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        Spacer()
                    }
                    .frame(maxWidth: .infinity)
                } else {
                    List(filteredContacts, id: \.id) { contact in
                        ContactRowView(contact: contact)
                            .contextMenu {
                                // Favourite toggle - works for all contacts by ID
                                Button {
                                    state.toggleFavouriteContactById(id: contact.id)
                                } label: {
                                    Label(
                                        contact.isFavourite ? "Remove from Favourites" : "Add to Favourites",
                                        systemImage: contact.isFavourite ? "star.slash" : "star"
                                    )
                                }
                                // Network actions - only for linked contacts
                                if let contactFourWords = contact.fourWords {
                                    Button {
                                        state.connectToPeer(fourWords: contactFourWords)
                                    } label: {
                                        Label("Connect", systemImage: "antenna.radiowaves.left.and.right")
                                    }
                                }
                                // Local-only contacts can be linked
                                if contact.isLocalOnly {
                                    Button {
                                        // TODO: Show linking sheet
                                    } label: {
                                        Label("Link to Network", systemImage: "link.badge.plus")
                                    }
                                }
                                Divider()
                                Button(role: .destructive) {
                                    state.removeContactById(id: contact.id)
                                } label: {
                                    Label("Remove Contact", systemImage: "trash")
                                }
                            }
                    }
                }
            }
            .navigationTitle("Contacts")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Close") {
                        dismiss()
                    }
                }
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        state.loadContacts()
                    } label: {
                        Image(systemName: "arrow.clockwise")
                    }
                }
            }
            .alert("Add Contact", isPresented: $showingAddContact) {
                TextField("Four-word address (e.g. ocean forest moon star)", text: $newContactFourWords)
                Button("Cancel", role: .cancel) {
                    newContactFourWords = ""
                }
                Button("Add") {
                    if !newContactFourWords.isEmpty {
                        state.addContact(fourWords: newContactFourWords)
                        newContactFourWords = ""
                    }
                }
            } message: {
                Text("Enter the four-word address of the contact you want to add. You can use spaces or hyphens between words.")
            }
        }
        .frame(minWidth: 450, minHeight: 400)
        .onAppear {
            if state.isNetworking {
                state.loadContacts()
            }
        }
    }
}

// MARK: - Contact Row View

struct ContactRowView: View {
    let contact: ContactItem

    var body: some View {
        HStack(spacing: 12) {
            // Avatar with online status
            ZStack(alignment: .bottomTrailing) {
                Image(systemName: "person.circle.fill")
                    .font(.title)
                    .foregroundColor(contact.isOnline ? .blue : .gray)

                if contact.isOnline {
                    Circle()
                        .fill(Color.green)
                        .frame(width: 10, height: 10)
                        .overlay(
                            Circle()
                                .stroke(Color.white, lineWidth: 2)
                        )
                }
            }

            VStack(alignment: .leading, spacing: 2) {
                HStack {
                    if let displayName = contact.displayName {
                        Text(displayName)
                            .font(.body)
                    } else if let fourWords = contact.fourWords {
                        Text(fourWords)
                            .font(.body)
                    } else {
                        Text("Unknown")
                            .font(.body)
                            .foregroundColor(.secondary)
                    }

                    if contact.isFavourite {
                        Image(systemName: "star.fill")
                            .font(.caption)
                            .foregroundColor(.yellow)
                    }

                    // Local-only badge
                    if contact.isLocalOnly {
                        Text("Local")
                            .font(.system(size: 9))
                            .foregroundColor(.orange)
                            .padding(.horizontal, 5)
                            .padding(.vertical, 2)
                            .background(Color.orange.opacity(0.15))
                            .cornerRadius(4)
                    }
                }

                // Show four-words or local indicator
                if let fourWords = contact.fourWords {
                    Text(fourWords)
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .lineLimit(1)
                } else {
                    Text("Not linked to network")
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .italic()
                }
            }

            Spacer()

            // Online/Offline indicator
            Text(contact.isOnline ? "Online" : "Offline")
                .font(.caption2)
                .foregroundColor(contact.isOnline ? .green : .secondary)
                .padding(.horizontal, 8)
                .padding(.vertical, 2)
                .background(contact.isOnline ? Color.green.opacity(0.1) : Color.gray.opacity(0.1))
                .cornerRadius(4)
        }
        .padding(.vertical, 4)
        .accessibilityIdentifier("contactRow_\(contact.id)")
    }
}

// MARK: - Network Settings View

struct NetworkSettingsView: View {
    @EnvironmentObject var state: AppState
    @Environment(\.dismiss) var dismiss
    @State private var bootstrapAddress: String = ""
    @State private var customPort: String = ""
    @State private var useCustomPort = false

    var body: some View {
        NavigationView {
            Form {
                Section(header: Text("Network Status")) {
                    HStack {
                        Circle()
                            .fill(state.isNetworking ? Color.green : Color.orange)
                            .frame(width: 10, height: 10)
                        Text(state.isNetworking ? "Connected" : "Disconnected")
                        Spacer()
                        if state.isNetworking {
                            Button("Disconnect") {
                                state.stopNetworking()
                            }
                            .buttonStyle(.bordered)
                        } else {
                            Button("Connect") {
                                let port: UInt16? = useCustomPort ? UInt16(customPort) : nil
                                state.startNetworkingWithBootstrap(port: port)
                            }
                            .buttonStyle(.borderedProminent)
                        }
                    }

                    if let identity = state.connectionIdentity {
                        LabeledContent("Connection Identity") {
                            Text(identity)
                                .font(.system(.caption, design: .monospaced))
                                .foregroundColor(.secondary)
                        }
                    }

                    LabeledContent("Presence Beacons") {
                        Toggle("", isOn: Binding(
                            get: { state.isPresenceActive },
                            set: { newValue in
                                if newValue {
                                    state.startPresenceBeacons()
                                } else {
                                    state.stopPresenceBeacons()
                                }
                            }
                        ))
                        .disabled(!state.isNetworking)
                    }
                }

                Section(header: Text("Bootstrap Node")) {
                    TextField("Bootstrap Address", text: $bootstrapAddress)
                        .font(.system(.body, design: .monospaced))

                    Text("Default: \(NetworkConfig.defaultBootstrapAddress)")
                        .font(.caption)
                        .foregroundColor(.secondary)

                    Button("Reset to Default") {
                        bootstrapAddress = NetworkConfig.defaultBootstrapAddress
                    }
                    .font(.caption)
                }

                Section(header: Text("Port Configuration")) {
                    Toggle("Use Custom Port", isOn: $useCustomPort)

                    if useCustomPort {
                        TextField("Port (e.g. 4433)", text: $customPort)
                            .font(.system(.body, design: .monospaced))
                    }

                    Text("Leave empty for automatic port selection")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Section(header: Text("Digital Ocean Bootstrap")) {
                    HStack {
                        Image(systemName: "server.rack")
                            .foregroundColor(.blue)
                        VStack(alignment: .leading, spacing: 2) {
                            Text("DO Bootstrap Node")
                                .font(.body)
                            Text("138.197.29.195:4433")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                        Spacer()
                        Button("Use") {
                            bootstrapAddress = "138.197.29.195:4433"
                        }
                        .buttonStyle(.bordered)
                    }
                }
            }
            .navigationTitle("Network Settings")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Close") {
                        dismiss()
                    }
                }
            }
        }
        .frame(minWidth: 450, minHeight: 400)
        .onAppear {
            bootstrapAddress = state.bootstrapAddress
        }
    }
}

#Preview {
    ContentView()
        .environmentObject(AppState())
}
