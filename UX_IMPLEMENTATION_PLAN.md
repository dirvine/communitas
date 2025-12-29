# Communitas UX Improvement Implementation Plan

## Executive Summary

This document outlines the complete implementation plan for improving the Communitas macOS app's user experience, specifically addressing the organizational structure issues identified in the UX review.

## Problem Statement

The current sidebar structure creates semantic confusion:
- "Organisations" mixes orgs you own AND orgs you're a member of
- "Personal" section mixes groups, contacts, and organizational references
- No clear ownership distinction or permission visibility
- Contact management is buried and hard to access

## Solution Overview

### New Four-Section Structure
1. **My Organizations** - Orgs you own/admin
2. **My Communities** - Orgs you're a member of (but don't own)  
3. **Personal** - Personal groups & spaces
4. **Direct Messages** - 1:1 contacts only

### Enhanced Permission System
- Role badges: Owner (crown), Admin (shield), Member (person), Guest (eye)
- Permission-aware UI controls
- Visual indicators for read-only vs editable content

## Phase 1: Core Navigation Restructure

### 1.1 Update SidebarSection Enum

**File:** `SidebarView.swift` (lines 7-32)

```swift
enum SidebarSection: String, CaseIterable, Identifiable {
    case myOrganizations = "My Organizations"
    case myCommunities = "My Communities"
    case personal = "Personal"
    case directMessages = "Direct Messages"

    var id: String { rawValue }

    var icon: String {
        switch self {
        case .myOrganizations: return "building.2.fill"
        case .myCommunities: return "building.2"
        case .personal: return "person.3.fill"
        case .directMessages: return "message.fill"
        }
    }
    
    var priority: Int {
        switch self {
        case .myOrganizations: return 1
        case .myCommunities: return 2
        case .personal: return 3
        case .directMessages: return 4
        }
    }
}
```

### 1.2 Add User Role System

**File:** `SidebarView.swift` (add after line 32)

```swift
enum UserRole: String, CaseIterable {
    case owner = "Owner"
    case admin = "Admin"
    case member = "Member"
    case guest = "Guest"
    
    var icon: String {
        switch self {
        case .owner: return "crown.fill"
        case .admin: return "shield.fill"
        case .member: return "person.fill"
        case .guest: return "eye"
        }
    }
    
    var color: Color {
        switch self {
        case .owner: return .orange
        case .admin: return .blue
        case .member: return .gray
        case .guest: return .secondary
        }
    }
}
```

### 1.3 Enhanced Entity Model

**File:** `SidebarView.swift` (add after UserRole)

```swift
struct EnhancedEntity: Identifiable {
    let id: String
    let entity: SwiftEntity
    let userRole: UserRole
    let memberCount: Int
    let isActive: Bool
    let unreadCount: Int
    
    init(entity: SwiftEntity, state: AppState) {
        self.id = entity.id
        self.entity = entity
        self.memberCount = entity.members.count
        self.isActive = true // TODO: Implement from presence
        self.unreadCount = 0 // TODO: Implement from messaging
        
        // Determine user role
        if entity.fourWords == state.fourWords {
            self.userRole = .owner
        } else {
            self.userRole = .member // TODO: Check actual permissions
        }
    }
}
```

### 1.4 Update Organization Categorization Logic

**File:** `SidebarView.swift` (update computed properties around line 120)

```swift
// Replace existing organization categorization
private var myOrganizations: [EnhancedEntity] {
    state.entities
        .filter { $0.entityType == .organisation && $0.fourWords == state.fourWords }
        .map { EnhancedEntity(entity: $0, state: state) }
        .sorted { $0.entity.name < $1.entity.name }
}

private var myCommunities: [EnhancedEntity] {
    state.entities
        .filter { $0.entityType == .organisation && $0.fourWords != state.fourWords }
        .map { EnhancedEntity(entity: $0, state: state) }
        .sorted { $0.entity.name < $1.entity.name }
}

private var personalGroups: [SwiftEntity] {
    state.entities
        .filter { $0.entityType == .group && $0.parentOrgId == nil }
        .sorted { $0.name < $1.name }
}

private var directMessages: [SwiftContact] {
    state.contacts
        .filter { $0.fourWords != nil }
        .sorted { ($0.displayName ?? "") < ($1.displayName ?? "") }
}
```

## Phase 2: Enhanced UI Components

### 2.1 Role Badge Component

**Create new file:** `RoleBadge.swift`

```swift
import SwiftUI

struct RoleBadge: View {
    let role: UserRole
    let compact: Bool = false
    
    var body: some View {
        HStack(spacing: compact ? 2 : 4) {
            Image(systemName: role.icon)
                .font(compact ? .caption2 : .caption)
            if !compact {
                Text(role.rawValue)
                    .font(.caption2)
                    .fontWeight(.medium)
            }
        }
        .foregroundColor(.white)
        .padding(.horizontal, compact ? 4 : 6)
        .padding(.vertical, compact ? 1 : 2)
        .background(role.color)
        .clipShape(Capsule())
    }
}
```

### 2.2 Enhanced Entity Row

**Update file:** `SidebarView.swift` (modify EntityRow component)

```swift
struct EntityRow: View {
    let entity: EnhancedEntity
    @Binding var selectedEntity: SwiftEntity?
    
    var body: some View {
        HStack(spacing: 8) {
            // Entity icon
            Image(systemName: SidebarItem.iconFor(entity.entity.entityType))
                .font(.system(size: 14))
                .foregroundColor(.primary)
                .frame(width: 16)
            
            // Entity name
            Text(entity.entity.name)
                .font(.body)
                .foregroundColor(.primary)
                .lineLimit(1)
            
            Spacer()
            
            // Role badge (for organizations)
            if entity.entity.entityType == .organisation {
                RoleBadge(role: entity.userRole, compact: true)
            }
            
            // Member count
            if entity.memberCount > 0 {
                Text("\(entity.memberCount)")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            // Unread indicator
            if entity.unreadCount > 0 {
                Text("\(entity.unreadCount)")
                    .font(.caption2)
                    .foregroundColor(.white)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 1)
                    .background(.red)
                    .clipShape(Capsule())
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(
            RoundedRectangle(cornerRadius: 6)
                .fill(selectedEntity?.id == entity.id ? Color.accentColor.opacity(0.2) : Color.clear)
        )
        .contentShape(Rectangle())
        .onTapGesture {
            selectedEntity = entity.entity
        }
    }
}
```

### 2.3 Enhanced Contact Row

**Create new file:** `ContactRow.swift`

```swift
import SwiftUI

struct ContactRow: View {
    let contact: SwiftContact
    @Binding var selectedContact: SwiftContact?
    let onTap: () -> Void
    
    var body: some View {
        HStack(spacing: 10) {
            // Avatar
            ZStack {
                Circle()
                    .fill(contact.isOnline ? .green : .gray)
                    .frame(width: 32, height: 32)
                
                Image(systemName: "person.fill")
                    .font(.system(size: 16))
                    .foregroundColor(.white)
            }
            
            // Contact info
            VStack(alignment: .leading, spacing: 2) {
                Text(contact.displayName ?? "Unknown")
                    .font(.body)
                    .foregroundColor(.primary)
                
                if let fourWords = contact.fourWords {
                    Text(shortFourWords(fourWords))
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
            
            Spacer()
            
            // Status indicator
            Circle()
                .fill(contact.isOnline ? .green : .gray)
                .frame(width: 8, height: 8)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(
            RoundedRectangle(cornerRadius: 6)
                .fill(selectedContact?.id == contact.id ? Color.accentColor.opacity(0.2) : Color.clear)
        )
        .contentShape(Rectangle())
        .onTapGesture {
            selectedContact = contact
            onTap()
        }
    }
}

private func shortFourWords(_ fourWords: String) -> String {
    let words = fourWords.split(separator: "-")
    return words.map { String($0.prefix(3)) }.joined(separator: "-")
}
```

## Phase 3: Permission-Aware UI Updates

### 3.1 Permission Check Extensions

**Update file:** `AppState.swift` (add permission checking methods)

```swift
extension AppState {
    func getUserRole(for entityId: String) -> UserRole {
        guard let entity = entities.first(where: { $0.id == entityId }) else {
            return .guest
        }
        
        if entity.fourWords == fourWords {
            return .owner
        }
        
        // TODO: Check actual permissions from backend
        return .member
    }
    
    func canEdit(_ entity: SwiftEntity) -> Bool {
        let role = getUserRole(for: entity.id)
        return role == .owner || role == .admin
    }
    
    func canCreateProjects(in orgId: String) -> Bool {
        let role = getUserRole(for: orgId)
        return role == .owner || role == .admin
    }
    
    func canManageMembers(in orgId: String) -> Bool {
        let role = getUserRole(for: orgId)
        return role == .owner || role == .admin
    }
}
```

### 3.2 Permission-Aware View Modifiers

**Update file:** `ContentView.swift` (enhance conditional modifier)

```swift
extension View {
    /// Conditionally applies modifiers based on entity permissions
    @ViewBuilder
    func ifCanEdit<T: View>(_ entity: SwiftEntity, state: AppState, transform: (Self) -> T) -> some View {
        if state.canEdit(entity) {
            transform(self)
        } else {
            self
                .disabled(true)
                .opacity(0.6)
        }
    }
    
    /// Shows read-only indicator for non-editable content
    @ViewBuilder
    func readOnlyIfCannotEdit<T: View>(_ entity: SwiftEntity, state: AppState, transform: (Self) -> T) -> some View {
        if state.canEdit(entity) {
            transform(self)
        } else {
            VStack(spacing: 8) {
                HStack {
                    Image(systemName: "lock.fill")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text("Read-only")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Spacer()
                }
                transform(self)
                    .disabled(true)
                    .opacity(0.8)
            }
        }
    }
}
```

## Phase 4: Enhanced Sidebar Structure

### 4.1 Update Main Sidebar Body

**Update file:** `SidebarView.swift` (replace main body around line 150)

```swift
var body: some View {
    List(selection: $selectedSection) {
        // MARK: - My Organizations Section
        Section {
            DisclosureGroup(
                isExpanded: Binding(
                    get: { expandedSections.contains("myOrganizations") },
                    set: { if $0 { expandedSections.insert("myOrganizations") } else { expandedSections.remove("myOrganizations") } }
                )
            ) {
                if myOrganizations.isEmpty {
                    emptyStateView(
                        icon: "building.2",
                        text: "No organizations created yet",
                        action: ("Create Organization", { createContext = .organisation })
                    )
                } else {
                    ForEach(myOrganizations) { entity in
                        OrganisationRow(
                            entity: entity,
                            isExpanded: expandedOrgs.contains(entity.id),
                            selectedEntity: $selectedEntity,
                            onToggleExpanded: { toggleExpanded(entity.id) },
                            onCreateProject: { createContext = .project(parentOrgId: entity.id) },
                            onCreateChannel: { createContext = .channel(parentOrgId: entity.id) },
                            onCreateGroup: { createContext = .group(parentOrgId: entity.id) }
                        )
                    }
                }
            } label: {
                sectionHeader(
                    title: "My Organizations",
                    icon: "building.2.fill",
                    count: myOrganizations.count,
                    action: ("+", { createContext = .organisation })
                )
            }
        }

        // MARK: - My Communities Section
        Section {
            DisclosureGroup(
                isExpanded: Binding(
                    get: { expandedSections.contains("myCommunities") },
                    set: { if $0 { expandedSections.insert("myCommunities") } else { expandedSections.remove("myCommunities") } }
                )
            ) {
                if myCommunities.isEmpty {
                    emptyStateView(
                        icon: "building.2",
                        text: "No communities joined yet",
                        action: nil
                    )
                } else {
                    ForEach(myCommunities) { entity in
                        EntityRow(
                            entity: entity,
                            selectedEntity: $selectedEntity
                        )
                    }
                }
            } label: {
                sectionHeader(
                    title: "My Communities",
                    icon: "building.2",
                    count: myCommunities.count,
                    action: nil
                )
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
                        emptyStateView(
                            icon: "person.3",
                            text: "No personal groups yet",
                            action: ("Create Group", { createContext = .group(parentOrgId: nil) })
                        )
                    } else {
                        ForEach(personalGroups, id: \.id) { group in
                            EntityRow(
                                entity: EnhancedEntity(entity: group, state: state),
                                selectedEntity: $selectedEntity
                            )
                        }
                    }
                } label: {
                    subSectionHeader(
                        title: "Groups",
                        icon: "person.3.fill",
                        count: personalGroups.count,
                        action: ("+", { createContext = .group(parentOrgId: nil) })
                    )
                }
            } label: {
                sectionHeader(
                    title: "Personal",
                    icon: "person.3.fill",
                    count: personalGroups.count,
                    action: nil
                )
            }
        }

        // MARK: - Direct Messages Section
        Section {
            DisclosureGroup(
                isExpanded: Binding(
                    get: { expandedSections.contains("directMessages") },
                    set: { if $0 { expandedSections.insert("directMessages") } else { expandedSections.remove("directMessages") } }
                )
            ) {
                if directMessages.isEmpty {
                    emptyStateView(
                        icon: "message",
                        text: state.isNetworking ? "No contacts yet" : "Go online to see contacts",
                        action: state.isNetworking ? ("Add Contact", { showingAddContact = true }) : nil
                    )
                } else {
                    ForEach(directMessages, id: \.id) { contact in
                        ContactRow(
                            contact: contact,
                            selectedContact: .constant(nil)
                        ) {
                            if let fourWords = contact.fourWords {
                                state.openContactChat(fourWords: fourWords, displayName: contact.displayName)
                            }
                        }
                    }
                }
            } label: {
                sectionHeader(
                    title: "Direct Messages",
                    icon: "message.fill",
                    count: directMessages.count,
                    action: state.isNetworking ? ("+", { showingAddContact = true }) : nil
                )
            }
        }
    }
    .listStyle(.sidebar)
    .navigationSplitViewColumnWidth(min: 240, ideal: 280, max: 350)
    .sheet(item: $createContext) { context in
        CreateEntityView(context: context)
            .environmentObject(state)
    }
    .sheet(isPresented: $showingAddContact) {
        AddContactView()
            .environmentObject(state)
    }
}
```

## Phase 5: Helper Components

### 5.1 Section Header Component

**Add to SidebarView.swift**

```swift
@ViewBuilder
private func sectionHeader(title: String, icon: String, count: Int, action: (String, () -> Void)?) -> some View {
    HStack {
        Label(title, systemImage: icon)
            .font(.headline)
        Spacer()
        
        HStack(spacing: 8) {
            if count > 0 {
                Text("\(count)")
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(.secondary.opacity(0.2))
                    .clipShape(Capsule())
            }
            
            if let action = action {
                Button(action: action.1) {
                    Image(systemName: action.0)
                        .font(.system(size: 14))
                        .foregroundColor(.blue)
                }
                .buttonStyle(.plain)
            }
        }
    }
}

@ViewBuilder
private func subSectionHeader(title: String, icon: String, count: Int, action: (String, () -> Void)?) -> some View {
    HStack {
        Image(systemName: icon)
            .foregroundColor(.purple)
            .font(.system(size: 14))
        Text(title)
        Spacer()
        
        HStack(spacing: 8) {
            if count > 0 {
                Text("\(count)")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
            
            if let action = action {
                Button(action: action.1) {
                    Image(systemName: action.0)
                        .font(.caption2)
                        .foregroundColor(.blue)
                }
                .buttonStyle(.plain)
            }
        }
    }
}

@ViewBuilder
private func emptyStateView(icon: String, text: String, action: (String, () -> Void)?) -> some View {
    VStack(spacing: 8) {
        HStack {
            Image(systemName: icon)
                .foregroundColor(.secondary)
            Text(text)
                .foregroundColor(.secondary)
                .font(.caption)
            Spacer()
        }
        
        if let action = action {
            Button(action: action.1) {
                HStack {
                    Image(systemName: action.0)
                        .font(.caption)
                    Text(text.contains("Create") || text.contains("Add") ? "New" : "Action")
                        .font(.caption)
                }
                .foregroundColor(.blue)
            }
            .buttonStyle(.plain)
            .padding(.vertical, 2)
        }
    }
    .padding(.vertical, 4)
}
```

## Phase 6: Testing Implementation

### 6.1 UI Test Structure

**Create:** `SidebarNavigationTests.swift`

```swift
import XCTest
import SwiftUI

class SidebarNavigationTests: XCTestCase {
    var app: XCUIApplication!
    
    override func setUp() {
        super.setUp()
        app = XCUIApplication()
        app.launchArguments = ["--uitesting"]
        app.launch()
    }
    
    func testSidebarSectionsExist() {
        XCTAssertTrue.app.navigationBars["My Organizations"].exists
        XCTAssertTrue.app.navigationBars["My Communities"].exists
        XCTAssertTrue.app.navigationBars["Personal"].exists
        XCTAssertTrue.app.navigationBars["Direct Messages"].exists
    }
    
    func testOrganizationOwnershipSeparation() {
        // Test that owned orgs appear in correct section
        let myOrgsSection = app.buttons["My Organizations"]
        myOrgsSection.tap()
        
        // Should have owner badges
        XCTAssertTrue.app.images["crown.fill"].exists
        
        // Test communities section
        let myCommunitiesSection = app.buttons["My Communities"]
        myCommunitiesSection.tap()
        
        // Should have member badges
        XCTAssertTrue.app.images["person.fill"].exists
    }
    
    func testDirectMessagesSection() {
        let dmSection = app.buttons["Direct Messages"]
        dmSection.tap()
        
        // Should only have individual contacts
        XCTAssertTrue.app.buttons["Add Contact"].exists
    }
}
```

## Implementation Timeline

### Week 1: Core Structure
- [ ] Update SidebarSection enum
- [ ] Add UserRole system
- [ ] Create EnhancedEntity model
- [ ] Update categorization logic

### Week 2: UI Components  
- [ ] Create RoleBadge component
- [ ] Create enhanced EntityRow
- [ ] Create ContactRow component
- [ ] Add permission-aware modifiers

### Week 3: Sidebar Implementation
- [ ] Implement new sidebar body structure
- [ ] Add section header components
- [ ] Implement permission checking
- [ ] Add creation context handling

### Week 4: Testing & Polish
- [ ] Write comprehensive UI tests
- [ ] Add accessibility testing
- [ ] Performance optimization
- [ ] User acceptance testing

## Success Metrics

### Technical Metrics
- Build success rate: 100%
- UI test coverage: >90%
- Performance: Sidebar renders <100ms
- Memory usage: No increase beyond 10%

### User Experience Metrics
- Task completion rate: >95%
- User satisfaction: >4.5/5
- Learnability: New users navigate successfully within 2 minutes
- Error rate: <2% for navigation tasks

## Conclusion

This implementation plan provides a comprehensive solution to the UX issues identified in the Communitas app. The new structure will:

1. **Provide Clear Mental Models** - Users understand ownership vs membership
2. **Improve Navigation** - Separate sections for different contexts
3. **Enhance Permission Clarity** - Visual role indicators throughout
4. **Streamline Contact Management** - Dedicated Direct Messages section
5. **Maintain Performance** - Efficient rendering with large datasets

The phased approach ensures iterative improvement with testing at each stage, resulting in a polished, user-friendly interface that addresses the core UX problems while maintaining the app's powerful collaboration features.