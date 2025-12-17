import SwiftUI
import CommunitasKit
import AppKit

// MARK: - Sidebar Section Model

enum SidebarSection: String, CaseIterable, Identifiable {
    case organisations = "Organisations"
    case personal = "Personal"

    var id: String { rawValue }

    var icon: String {
        switch self {
        case .organisations: return "building.2.fill"
        case .personal: return "person.fill"
        }
    }
}

// MARK: - Sidebar Item Model

struct SidebarItem: Identifiable, Hashable {
    let id: String
    let name: String
    let icon: String
    let entityType: SwiftEntityType?
    let parentId: String?
    let isSection: Bool
    let childCount: Int

    init(entity: SwiftEntity) {
        self.id = entity.id
        self.name = entity.name
        self.icon = Self.iconFor(entity.entityType)
        self.entityType = entity.entityType
        self.parentId = entity.parentOrgId
        self.isSection = false
        self.childCount = entity.members.count
    }

    init(id: String, name: String, icon: String, isSection: Bool = false, childCount: Int = 0) {
        self.id = id
        self.name = name
        self.icon = icon
        self.entityType = nil
        self.parentId = nil
        self.isSection = isSection
        self.childCount = childCount
    }

    static func iconFor(_ type: SwiftEntityType) -> String {
        switch type {
        case .organisation: return "building.2.fill"
        case .project: return "folder.fill"
        case .channel: return "number"
        case .group: return "person.3.fill"
        case .person: return "person.fill"
        }
    }
}

// MARK: - Organisation Tree Node

struct OrganisationNode: Identifiable {
    let id: String
    let organisation: SwiftEntity
    var projects: [SwiftEntity]
    var channels: [SwiftEntity]
    var groups: [SwiftEntity]
    var members: [String]

    init(organisation: SwiftEntity, allEntities: [SwiftEntity]) {
        self.id = organisation.id
        self.organisation = organisation
        self.members = organisation.members

        // Filter children by parent org
        self.projects = allEntities.filter {
            $0.entityType == .project && $0.parentOrgId == organisation.id
        }
        self.channels = allEntities.filter {
            $0.entityType == .channel && $0.parentOrgId == organisation.id
        }
        self.groups = allEntities.filter {
            $0.entityType == .group && $0.parentOrgId == organisation.id
        }
    }
}

// MARK: - Main Sidebar View

struct SidebarView: View {
    @EnvironmentObject var state: AppState
    @Binding var selectedEntity: SwiftEntity?
    @Binding var selectedSection: SidebarSection?

    @State private var expandedOrgs: Set<String> = []
    @State private var expandedSections: Set<String> = ["organisations", "personal"]
    @State private var showingCreateSheet = false
    @State private var createContext: CreateContext?
    @State private var showingAddContact = false

    enum CreateContext {
        case organisation
        case project(parentOrgId: String)
        case channel(parentOrgId: String)
        case group(parentOrgId: String?)
        case personal
    }

    var organisations: [OrganisationNode] {
        let orgs = state.entities.filter { $0.entityType == .organisation }
        return orgs.map { OrganisationNode(organisation: $0, allEntities: state.entities) }
    }

    var personalGroups: [SwiftEntity] {
        state.entities.filter { $0.entityType == .group && $0.parentOrgId == nil }
    }

    var personalContacts: [ContactItem] {
        state.contacts
    }

    var body: some View {
        List(selection: $selectedEntity) {
            // MARK: - Organisations Section
            Section {
                DisclosureGroup(
                    isExpanded: Binding(
                        get: { expandedSections.contains("organisations") },
                        set: { if $0 { expandedSections.insert("organisations") } else { expandedSections.remove("organisations") } }
                    )
                ) {
                    if organisations.isEmpty {
                        HStack {
                            Image(systemName: "building.2")
                                .foregroundColor(.secondary)
                            Text("No organisations yet")
                                .foregroundColor(.secondary)
                                .font(.caption)
                        }
                        .padding(.vertical, 4)
                    } else {
                        ForEach(organisations) { node in
                            OrganisationRow(
                                node: node,
                                isExpanded: expandedOrgs.contains(node.id),
                                selectedEntity: $selectedEntity,
                                onToggleExpanded: {
                                    if expandedOrgs.contains(node.id) {
                                        expandedOrgs.remove(node.id)
                                    } else {
                                        expandedOrgs.insert(node.id)
                                    }
                                },
                                onCreateProject: {
                                    createContext = .project(parentOrgId: node.id)
                                    showingCreateSheet = true
                                },
                                onCreateChannel: {
                                    createContext = .channel(parentOrgId: node.id)
                                    showingCreateSheet = true
                                },
                                onCreateGroup: {
                                    createContext = .group(parentOrgId: node.id)
                                    showingCreateSheet = true
                                }
                            )
                        }
                    }

                    // Add Organisation button
                    Button {
                        createContext = .organisation
                        showingCreateSheet = true
                    } label: {
                        Label("New Organisation", systemImage: "plus.circle")
                            .foregroundColor(.blue)
                    }
                    .buttonStyle(.plain)
                    .padding(.vertical, 4)
                } label: {
                    HStack {
                        Label("Organisations", systemImage: "building.2.fill")
                            .font(.headline)
                        Spacer()
                        Button {
                            createContext = .organisation
                            showingCreateSheet = true
                        } label: {
                            Image(systemName: "plus.circle.fill")
                                .foregroundColor(.blue)
                        }
                        .buttonStyle(.plain)
                        .help("Add Organisation")
                    }
                }
            }

            // MARK: - Personal Section
            Section {
                DisclosureGroup(
                    isExpanded: Binding(
                        get: { expandedSections.contains("personal") },
                        set: { if $0 { expandedSections.insert("personal") } else { expandedSections.remove("personal") } }
                    )
                ) {
                    // Personal Groups
                    DisclosureGroup {
                        if personalGroups.isEmpty {
                            HStack {
                                Image(systemName: "person.3")
                                    .foregroundColor(.secondary)
                                Text("No groups yet")
                                    .foregroundColor(.secondary)
                                    .font(.caption)
                            }
                            .padding(.vertical, 2)
                        } else {
                            ForEach(personalGroups, id: \.id) { group in
                                EntityRow(entity: group, selectedEntity: $selectedEntity)
                            }
                        }

                        Button {
                            createContext = .group(parentOrgId: nil)
                            showingCreateSheet = true
                        } label: {
                            Label("New Group", systemImage: "plus.circle")
                                .font(.caption)
                                .foregroundColor(.blue)
                        }
                        .buttonStyle(.plain)
                        .padding(.vertical, 2)
                    } label: {
                        HStack {
                            Image(systemName: "person.3.fill")
                                .foregroundColor(.purple)
                            Text("Groups")
                            Spacer()
                            Text("\(personalGroups.count)")
                                .font(.caption)
                                .foregroundColor(.secondary)
                            Button {
                                createContext = .group(parentOrgId: nil)
                                showingCreateSheet = true
                            } label: {
                                Image(systemName: "plus.circle.fill")
                                    .font(.caption)
                                    .foregroundColor(.blue)
                            }
                            .buttonStyle(.plain)
                            .help("Add Group")
                        }
                    }

                    // Individuals (Contacts)
                    DisclosureGroup {
                        if personalContacts.isEmpty {
                            HStack {
                                Image(systemName: "person")
                                    .foregroundColor(.secondary)
                                Text(state.isNetworking ? "No contacts yet" : "Go online to see contacts")
                                    .foregroundColor(.secondary)
                                    .font(.caption)
                            }
                            .padding(.vertical, 2)
                        } else {
                            ForEach(personalContacts, id: \.id) { contact in
                                ContactSidebarRow(contact: contact)
                                    .contentShape(Rectangle())
                                    .onTapGesture {
                                        // Only open chat for contacts with network identity
                                        if let contactFourWords = contact.fourWords {
                                            state.openContactChat(fourWords: contactFourWords, displayName: contact.displayName)
                                        }
                                        // TODO: For local-only contacts, show linking sheet
                                    }
                            }
                        }
                    } label: {
                        HStack {
                            Image(systemName: "person.fill")
                                .foregroundColor(.orange)
                            Text("Individuals")
                            Spacer()
                            Text("\(personalContacts.count)")
                                .font(.caption)
                                .foregroundColor(.secondary)
                            Button {
                                showingAddContact = true
                            } label: {
                                Image(systemName: "plus.circle.fill")
                                    .font(.caption)
                                    .foregroundColor(.blue)
                            }
                            .buttonStyle(.plain)
                            .help("Add Contact")
                        }
                    }
                } label: {
                    Label("Personal", systemImage: "person.fill")
                        .font(.headline)
                }
            }
        }
        .listStyle(.sidebar)
        .sheet(isPresented: $showingCreateSheet) {
            if let context = createContext {
                CreateEntitySheet(context: context) { name, description, type, networkFourWords in
                    createEntity(name: name, description: description, type: type, context: context, networkFourWords: networkFourWords)
                    showingCreateSheet = false
                }
            }
        }
        .onAppear {
            state.loadEntities()
            if state.isNetworking {
                state.loadContacts()
            }
        }
        .sheet(isPresented: $showingAddContact) {
            AddContactSheet { fourWords, displayName in
                if let fourWords = fourWords {
                    // Network-linked contact with four-word address
                    state.addContact(fourWords: fourWords, displayName: displayName)
                } else {
                    // Local-only contact (no four-word address yet)
                    state.createLocalContact(displayName: displayName)
                }
                showingAddContact = false
            }
        }
    }

    private func createEntity(name: String, description: String?, type: SwiftEntityType, context: CreateContext, networkFourWords: String?) {
        var parentOrgId: String?

        switch context {
        case .project(let orgId), .channel(let orgId):
            parentOrgId = orgId
        case .group(let orgId):
            parentOrgId = orgId
        default:
            break
        }

        if let fourWords = networkFourWords {
            // Network-linked entity - create with four-word address
            state.createEntity(name: name, type: type, description: description, parentOrgId: parentOrgId, networkFourWords: fourWords)
        } else {
            // Local-only entity - create without network identity
            state.createLocalEntity(name: name, type: type, description: description, parentOrgId: parentOrgId)
        }
    }
}

// MARK: - Organisation Row

struct OrganisationRow: View {
    let node: OrganisationNode
    let isExpanded: Bool
    @Binding var selectedEntity: SwiftEntity?
    let onToggleExpanded: () -> Void
    let onCreateProject: () -> Void
    let onCreateChannel: () -> Void
    let onCreateGroup: () -> Void

    var body: some View {
        DisclosureGroup(isExpanded: .constant(isExpanded)) {
            // Projects Section
            if !node.projects.isEmpty || true {
                DisclosureGroup {
                    ForEach(node.projects, id: \.id) { project in
                        EntityRow(entity: project, selectedEntity: $selectedEntity)
                            .padding(.leading, 8)
                    }

                    Button {
                        onCreateProject()
                    } label: {
                        Label("New Project", systemImage: "plus.circle")
                            .font(.caption)
                            .foregroundColor(.blue)
                    }
                    .buttonStyle(.plain)
                    .padding(.leading, 8)
                    .padding(.vertical, 2)
                } label: {
                    HStack {
                        Image(systemName: "folder.fill")
                            .foregroundColor(.yellow)
                        Text("Projects")
                        Spacer()
                        Text("\(node.projects.count)")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        Button {
                            onCreateProject()
                        } label: {
                            Image(systemName: "plus.circle.fill")
                                .font(.caption)
                                .foregroundColor(.blue)
                        }
                        .buttonStyle(.plain)
                        .help("Add Project")
                    }
                }
            }

            // Channels Section
            if !node.channels.isEmpty || true {
                DisclosureGroup {
                    ForEach(node.channels, id: \.id) { channel in
                        EntityRow(entity: channel, selectedEntity: $selectedEntity)
                            .padding(.leading, 8)
                    }

                    Button {
                        onCreateChannel()
                    } label: {
                        Label("New Channel", systemImage: "plus.circle")
                            .font(.caption)
                            .foregroundColor(.blue)
                    }
                    .buttonStyle(.plain)
                    .padding(.leading, 8)
                    .padding(.vertical, 2)
                } label: {
                    HStack {
                        Image(systemName: "number")
                            .foregroundColor(.green)
                        Text("Channels")
                        Spacer()
                        Text("\(node.channels.count)")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        Button {
                            onCreateChannel()
                        } label: {
                            Image(systemName: "plus.circle.fill")
                                .font(.caption)
                                .foregroundColor(.blue)
                        }
                        .buttonStyle(.plain)
                        .help("Add Channel")
                    }
                }
            }

            // Groups Section
            if !node.groups.isEmpty || true {
                DisclosureGroup {
                    ForEach(node.groups, id: \.id) { group in
                        EntityRow(entity: group, selectedEntity: $selectedEntity)
                            .padding(.leading, 8)
                    }

                    Button {
                        onCreateGroup()
                    } label: {
                        Label("New Group", systemImage: "plus.circle")
                            .font(.caption)
                            .foregroundColor(.blue)
                    }
                    .buttonStyle(.plain)
                    .padding(.leading, 8)
                    .padding(.vertical, 2)
                } label: {
                    HStack {
                        Image(systemName: "person.3.fill")
                            .foregroundColor(.purple)
                        Text("Groups")
                        Spacer()
                        Text("\(node.groups.count)")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        Button {
                            onCreateGroup()
                        } label: {
                            Image(systemName: "plus.circle.fill")
                                .font(.caption)
                                .foregroundColor(.blue)
                        }
                        .buttonStyle(.plain)
                        .help("Add Group")
                    }
                }
            }

            // Members Section
            if !node.members.isEmpty {
                DisclosureGroup {
                    ForEach(node.members, id: \.self) { member in
                        HStack {
                            Image(systemName: "person.fill")
                                .foregroundColor(.secondary)
                                .font(.caption)
                            Text(shortFourWords(member))
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                        .padding(.leading, 8)
                        .padding(.vertical, 2)
                    }
                } label: {
                    HStack {
                        Image(systemName: "person.2.fill")
                            .foregroundColor(.blue)
                        Text("Members")
                        Spacer()
                        Text("\(node.members.count)")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
            }
        } label: {
            Button {
                onToggleExpanded()
            } label: {
                HStack {
                    Image(systemName: "building.2.fill")
                        .foregroundColor(.blue)
                    Text(node.organisation.name)
                        .fontWeight(.medium)
                }
            }
            .buttonStyle(.plain)
        }
        .contextMenu {
            Button {
                onCreateProject()
            } label: {
                Label("New Project", systemImage: "folder.badge.plus")
            }
            Button {
                onCreateChannel()
            } label: {
                Label("New Channel", systemImage: "number.circle")
            }
            Button {
                onCreateGroup()
            } label: {
                Label("New Group", systemImage: "person.3")
            }
            Divider()
            Button(role: .destructive) {
                // Delete organisation
            } label: {
                Label("Delete Organisation", systemImage: "trash")
            }
        }
    }

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

// MARK: - Entity Row

struct EntityRow: View {
    let entity: SwiftEntity
    @Binding var selectedEntity: SwiftEntity?

    var isSelected: Bool {
        selectedEntity?.id == entity.id
    }

    var body: some View {
        Button {
            selectedEntity = entity
        } label: {
            HStack {
                Image(systemName: iconFor(entity.entityType))
                    .foregroundColor(colorFor(entity.entityType))
                    .font(.caption)

                Text(entity.name)
                    .lineLimit(1)

                Spacer()

                if entity.members.count > 0 {
                    Text("\(entity.members.count)")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                        .padding(.horizontal, 4)
                        .padding(.vertical, 1)
                        .background(Color.gray.opacity(0.2))
                        .cornerRadius(4)
                }
            }
            .padding(.vertical, 2)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .background(isSelected ? Color.accentColor.opacity(0.2) : Color.clear)
        .cornerRadius(4)
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
}

// MARK: - Expandable Four-Word Row

/// A reusable component that shows a four-word identity with tap-to-expand functionality
struct ExpandableFourWordRow: View {
    let fourWords: String?
    let displayName: String?
    let showCopyButton: Bool
    let isLocalOnly: Bool

    @State private var isExpanded = false

    init(fourWords: String?, displayName: String? = nil, showCopyButton: Bool = true, isLocalOnly: Bool = false) {
        self.fourWords = fourWords
        self.displayName = displayName
        self.showCopyButton = showCopyButton
        self.isLocalOnly = isLocalOnly
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            // Main row - always visible
            HStack {
                if let name = displayName, !name.isEmpty {
                    if let fw = fourWords {
                        // Show "DisplayName : short-four-words"
                        Text("\(name) : \(shortFourWords(fw))")
                            .font(.caption)
                            .lineLimit(1)
                    } else {
                        // Local-only: just show display name with badge
                        HStack(spacing: 4) {
                            Text(name)
                                .font(.caption)
                                .lineLimit(1)
                            if isLocalOnly {
                                Text("Local")
                                    .font(.system(size: 8))
                                    .foregroundColor(.orange)
                                    .padding(.horizontal, 4)
                                    .padding(.vertical, 1)
                                    .background(Color.orange.opacity(0.15))
                                    .cornerRadius(3)
                            }
                        }
                    }
                } else if let fw = fourWords {
                    // Just show four words with spaces
                    Text(isExpanded ? formatFourWords(fw) : shortFourWords(fw))
                        .font(.caption)
                        .lineLimit(isExpanded ? nil : 1)
                } else {
                    // No name, no four-words - show placeholder
                    Text("Unknown contact")
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .italic()
                }

                Spacer()

                if fourWords != nil {
                    Image(systemName: isExpanded ? "chevron.up" : "chevron.down")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }
            }
            .contentShape(Rectangle())
            .onTapGesture {
                if fourWords != nil {
                    withAnimation(.easeInOut(duration: 0.2)) {
                        isExpanded.toggle()
                    }
                }
            }

            // Expanded content - shows full four-word name with spaces
            if isExpanded, let fw = fourWords {
                HStack {
                    Text(formatFourWords(fw))
                        .font(.system(.caption2, design: .monospaced))
                        .foregroundColor(.secondary)
                        .textSelection(.enabled)

                    if showCopyButton {
                        Button {
                            copyToClipboard(fw)
                        } label: {
                            Image(systemName: "doc.on.doc")
                                .font(.caption2)
                                .foregroundColor(.blue)
                        }
                        .buttonStyle(.plain)
                        .help("Copy four-word address")
                    }
                }
                .padding(.top, 2)
            }
        }
    }

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

    private func copyToClipboard(_ text: String) {
        #if os(macOS)
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(text, forType: .string)
        #endif
    }
}

// MARK: - Contact Sidebar Row

struct ContactSidebarRow: View {
    let contact: ContactItem

    var body: some View {
        HStack(alignment: .top) {
            ZStack(alignment: .bottomTrailing) {
                Image(systemName: "person.circle.fill")
                    .foregroundColor(contact.isOnline ? .blue : .gray)
                    .font(.caption)

                if contact.isOnline {
                    Circle()
                        .fill(Color.green)
                        .frame(width: 6, height: 6)
                }
            }
            .padding(.top, 2)

            ExpandableFourWordRow(
                fourWords: contact.fourWords,
                displayName: contact.displayName,
                showCopyButton: true,
                isLocalOnly: contact.isLocalOnly
            )

            if contact.isFavourite {
                Image(systemName: "star.fill")
                    .font(.caption2)
                    .foregroundColor(.yellow)
                    .padding(.top, 2)
            }
        }
        .padding(.vertical, 2)
    }
}

// MARK: - Create Entity Sheet

struct CreateEntitySheet: View {
    let context: SidebarView.CreateContext
    /// Callback with (name, description, entityType, networkFourWords)
    /// networkFourWords is nil for local-only entities
    let onCreate: (String, String?, SwiftEntityType, String?) -> Void

    @Environment(\.dismiss) var dismiss
    @State private var name = ""
    @State private var description = ""
    @State private var hasNetworkIdentity = false
    @State private var fourWords = ""

    var entityType: SwiftEntityType {
        switch context {
        case .organisation: return .organisation
        case .project: return .project
        case .channel: return .channel
        case .group: return .group
        case .personal: return .group
        }
    }

    var title: String {
        switch context {
        case .organisation: return "New Organisation"
        case .project: return "New Project"
        case .channel: return "New Channel"
        case .group: return "New Group"
        case .personal: return "New Personal Group"
        }
    }

    var icon: String {
        switch context {
        case .organisation: return "building.2.fill"
        case .project: return "folder.fill"
        case .channel: return "number"
        case .group, .personal: return "person.3.fill"
        }
    }

    /// Whether the form is valid for submission
    private var isValid: Bool {
        let hasName = !name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
        if hasNetworkIdentity {
            let hasFourWords = !fourWords.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            return hasName && hasFourWords
        }
        return hasName
    }

    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                Image(systemName: icon)
                    .font(.system(size: 32))
                    .foregroundColor(hasNetworkIdentity ? .blue : .orange)
                VStack(alignment: .leading, spacing: 4) {
                    Text(title)
                        .font(.title2)
                        .fontWeight(.semibold)
                    Text(hasNetworkIdentity ? contextDescription : "Create a local placeholder \(entityTypeName.lowercased())")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
                Spacer()
                Button {
                    dismiss()
                } label: {
                    Image(systemName: "xmark.circle.fill")
                        .font(.title2)
                        .foregroundColor(.secondary)
                }
                .buttonStyle(.plain)
            }
            .padding()
            .background(Color.gray.opacity(0.05))

            Divider()

            // Form content
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    // Network identity toggle
                    HStack {
                        Toggle(isOn: $hasNetworkIdentity) {
                            HStack(spacing: 8) {
                                Image(systemName: hasNetworkIdentity ? "network" : "lock.fill")
                                    .foregroundColor(hasNetworkIdentity ? .blue : .orange)
                                VStack(alignment: .leading, spacing: 2) {
                                    Text("I have a network identity")
                                        .font(.subheadline)
                                        .fontWeight(.medium)
                                    Text(hasNetworkIdentity ? "Entity will sync with the network" : "Local placeholder until you get an identity")
                                        .font(.caption)
                                        .foregroundColor(.secondary)
                                }
                            }
                        }
                        .toggleStyle(.switch)
                    }
                    .padding(.vertical, 4)

                    // Name field
                    VStack(alignment: .leading, spacing: 8) {
                        HStack {
                            Text("Name")
                                .font(.headline)
                            Text("Required")
                                .font(.caption)
                                .foregroundColor(.red)
                        }
                        TextField("Enter name", text: $name)
                            .textFieldStyle(.roundedBorder)
                            .accessibilityIdentifier("entityNameField")
                    }

                    // Description field
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Description")
                            .font(.headline)
                        TextField("Optional description", text: $description)
                            .textFieldStyle(.roundedBorder)
                            .accessibilityIdentifier("entityDescriptionField")
                    }

                    // Four-word address (only shown when toggle is on)
                    if hasNetworkIdentity {
                        VStack(alignment: .leading, spacing: 8) {
                            HStack {
                                Text("Four-Word Address")
                                    .font(.headline)
                                Text("Required")
                                    .font(.caption)
                                    .foregroundColor(.red)
                            }
                            TextField("e.g. ocean forest moon star", text: $fourWords)
                                .textFieldStyle(.roundedBorder)
                                .accessibilityIdentifier("entityFourWordsField")
                            Text("The network identity for this \(entityTypeName.lowercased())")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                        .transition(.opacity.combined(with: .move(edge: .top)))
                    }

                    // Local-only info box
                    if !hasNetworkIdentity {
                        HStack(spacing: 12) {
                            Image(systemName: "info.circle.fill")
                                .foregroundColor(.orange)
                            VStack(alignment: .leading, spacing: 4) {
                                Text("Local-Only \(entityTypeName)")
                                    .font(.caption)
                                    .fontWeight(.semibold)
                                Text("You can link this \(entityTypeName.lowercased()) to a network identity later.")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                        }
                        .padding()
                        .background(Color.orange.opacity(0.1))
                        .cornerRadius(8)
                    }
                }
                .padding()
            }

            Divider()

            // Create button
            HStack {
                Spacer()
                Button {
                    let trimmedFourWords = fourWords.trimmingCharacters(in: .whitespacesAndNewlines)
                    let networkIdentity = hasNetworkIdentity && !trimmedFourWords.isEmpty ? trimmedFourWords : nil
                    onCreate(name, description.isEmpty ? nil : description, entityType, networkIdentity)
                } label: {
                    HStack {
                        Image(systemName: hasNetworkIdentity ? "plus.circle.fill" : "lock.badge.clock")
                        Text(hasNetworkIdentity ? "Create \(entityTypeName)" : "Create Local \(entityTypeName)")
                    }
                    .padding(.horizontal, 16)
                    .padding(.vertical, 8)
                }
                .buttonStyle(.borderedProminent)
                .tint(hasNetworkIdentity ? .blue : .orange)
                .disabled(!isValid)
                Spacer()
            }
            .padding()
        }
        .frame(width: 480, height: hasNetworkIdentity ? 480 : 420)
        .animation(.easeInOut(duration: 0.2), value: hasNetworkIdentity)
    }

    private var entityTypeName: String {
        switch entityType {
        case .organisation: return "Organisation"
        case .project: return "Project"
        case .channel: return "Channel"
        case .group: return "Group"
        case .person: return "Person"
        }
    }

    private var contextDescription: String {
        switch context {
        case .organisation:
            return "Organisations contain projects, channels, and groups"
        case .project:
            return "Project within the organisation"
        case .channel:
            return "Channel for topic-based discussions"
        case .group(let parentOrgId):
            if parentOrgId != nil {
                return "Group within the organisation"
            } else {
                return "Personal group for private collaboration"
            }
        case .personal:
            return "Personal group for private collaboration"
        }
    }
}

// MARK: - Add Contact Sheet

struct AddContactSheet: View {
    /// Callback with (fourWords, displayName) - fourWords is nil for local-only contacts
    let onAdd: (String?, String) -> Void

    @Environment(\.dismiss) var dismiss
    @State private var fourWords = ""
    @State private var displayName = ""
    @State private var hasNetworkIdentity = false  // Toggle for network-linked vs local-only

    /// Whether the form is valid for submission
    private var isValid: Bool {
        let hasDisplayName = !displayName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
        if hasNetworkIdentity {
            let hasFourWords = !fourWords.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            return hasDisplayName && hasFourWords
        }
        return hasDisplayName
    }

    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                Image(systemName: "person.badge.plus")
                    .font(.system(size: 32))
                    .foregroundColor(.blue)
                VStack(alignment: .leading, spacing: 4) {
                    Text("Add Contact")
                        .font(.title2)
                        .fontWeight(.semibold)
                    Text(hasNetworkIdentity ? "Enter their four-word address" : "Create a local placeholder contact")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
                Spacer()
                Button {
                    dismiss()
                } label: {
                    Image(systemName: "xmark.circle.fill")
                        .font(.title2)
                        .foregroundColor(.secondary)
                }
                .buttonStyle(.plain)
            }
            .padding()
            .background(Color.gray.opacity(0.05))

            Divider()

            // Form content
            VStack(alignment: .leading, spacing: 16) {
                // Network identity toggle
                HStack {
                    Toggle(isOn: $hasNetworkIdentity) {
                        HStack(spacing: 8) {
                            Image(systemName: hasNetworkIdentity ? "network" : "lock.fill")
                                .foregroundColor(hasNetworkIdentity ? .blue : .orange)
                            VStack(alignment: .leading, spacing: 2) {
                                Text("I have their four-word address")
                                    .font(.subheadline)
                                    .fontWeight(.medium)
                                Text(hasNetworkIdentity ? "Contact can be synced over the network" : "Local placeholder until you get their address")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                        }
                    }
                    .toggleStyle(.switch)
                }
                .padding(.vertical, 4)

                // Display Name (always shown, required)
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        Text("Display Name")
                            .font(.headline)
                        Text("Required")
                            .font(.caption)
                            .foregroundColor(.red)
                    }
                    TextField("e.g. Alice, Bob, Work Contact", text: $displayName)
                        .textFieldStyle(.roundedBorder)
                        .accessibilityIdentifier("contactDisplayNameField")
                    Text("A friendly name to identify this contact")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                // Four-word address (only shown when toggle is on)
                if hasNetworkIdentity {
                    VStack(alignment: .leading, spacing: 8) {
                        HStack {
                            Text("Four-Word Address")
                                .font(.headline)
                            Text("Required")
                                .font(.caption)
                                .foregroundColor(.red)
                        }
                        TextField("e.g. ocean forest moon star", text: $fourWords)
                            .textFieldStyle(.roundedBorder)
                            .accessibilityIdentifier("contactFourWordsField")
                        Text("Use spaces or hyphens between words")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    .transition(.opacity.combined(with: .move(edge: .top)))
                }

                // Local-only info box
                if !hasNetworkIdentity {
                    HStack(spacing: 12) {
                        Image(systemName: "info.circle.fill")
                            .foregroundColor(.orange)
                        VStack(alignment: .leading, spacing: 4) {
                            Text("Local-Only Contact")
                                .font(.caption)
                                .fontWeight(.semibold)
                            Text("You can link this contact to a network identity later when you have their four-word address.")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }
                    .padding()
                    .background(Color.orange.opacity(0.1))
                    .cornerRadius(8)
                }

                Spacer()

                // Add button
                Button {
                    let trimmedDisplayName = displayName.trimmingCharacters(in: .whitespacesAndNewlines)
                    let trimmedFourWords = fourWords.trimmingCharacters(in: .whitespacesAndNewlines)
                    if hasNetworkIdentity && !trimmedFourWords.isEmpty {
                        onAdd(trimmedFourWords, trimmedDisplayName)
                    } else {
                        onAdd(nil, trimmedDisplayName)
                    }
                } label: {
                    HStack {
                        Image(systemName: hasNetworkIdentity ? "person.badge.plus" : "person.badge.clock")
                        Text(hasNetworkIdentity ? "Add Contact" : "Add Local Contact")
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 8)
                }
                .buttonStyle(.borderedProminent)
                .tint(hasNetworkIdentity ? .blue : .orange)
                .disabled(!isValid)
            }
            .padding()
            .animation(.easeInOut(duration: 0.2), value: hasNetworkIdentity)
        }
        .frame(width: 420, height: hasNetworkIdentity ? 420 : 380)
        .animation(.easeInOut(duration: 0.2), value: hasNetworkIdentity)
    }
}

// MARK: - Preview

#Preview {
    SidebarView(
        selectedEntity: .constant(nil),
        selectedSection: .constant(nil)
    )
    .environmentObject(AppState())
    .frame(width: 280)
}
