import SwiftUI

struct ContentView: View {
    @EnvironmentObject var appState: AppState
    @State private var columnVisibility: NavigationSplitViewVisibility = .all

    var body: some View {
        NavigationSplitView(columnVisibility: $columnVisibility) {
            // Primary sidebar: navigation items
            List(NavigationItem.allCases, selection: $appState.selectedNavigation) { item in
                Label(item.rawValue, systemImage: item.systemImage)
                    .tag(item)
            }
            .navigationSplitViewColumnWidth(min: 160, ideal: 180)
            .listStyle(.sidebar)
        } content: {
            // Secondary column: channel sidebar when messaging, DM contact list, or nothing
            switch appState.selectedNavigation {
            case .messaging:
                ChannelSidebarView()
                    .navigationSplitViewColumnWidth(min: 180, ideal: 220)
            case .directMessages:
                dmSidebar
                    .navigationSplitViewColumnWidth(min: 180, ideal: 220)
            default:
                EmptyView()
            }
        } detail: {
            detailView
                .inspector(isPresented: $appState.showInspector) {
                    DetailPanelView()
                        .inspectorColumnWidth(min: 280, ideal: 320, max: 400)
                        .environmentObject(appState)
                }
        }
        .navigationTitle("CommuniTas")
        .toolbar {
            ToolbarItem(placement: .automatic) {
                Button {
                    appState.showInspector.toggle()
                } label: {
                    Label("Toggle Inspector", systemImage: "sidebar.right")
                }
                .keyboardShortcut("i", modifiers: [.command, .option])
            }
        }
        .sheet(isPresented: $appState.showCreateSpace) {
            CreateSpaceSheet()
                .environmentObject(appState)
        }
    }

    // MARK: - DM Sidebar

    private var dmSidebar: some View {
        List(appState.contacts, selection: Binding(
            get: { appState.selectedDMContact },
            set: { appState.selectedDMContact = $0 }
        )) { contact in
            HStack(spacing: 8) {
                ZStack {
                    Circle()
                        .fill(dmAvatarColor(for: contact.agentId))
                        .frame(width: 28, height: 28)
                    Text(String((contact.label ?? contact.agentId).prefix(1)).uppercased())
                        .font(.caption2)
                        .fontWeight(.semibold)
                        .foregroundStyle(.white)
                }
                VStack(alignment: .leading, spacing: 1) {
                    Text(contact.label ?? dmTruncatedId(contact.agentId))
                        .font(.subheadline)
                    Text(dmTruncatedId(contact.agentId))
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                }
            }
            .padding(.vertical, 2)
            .tag(contact)
        }
        .listStyle(.sidebar)
        .navigationTitle("Direct Messages")
    }

    private func dmAvatarColor(for senderId: String) -> Color {
        let hash = senderId.hashValue
        let colors: [Color] = [.blue, .purple, .orange, .green, .pink, .teal, .indigo, .mint]
        let index = abs(hash) % colors.count
        return colors[index]
    }

    private func dmTruncatedId(_ id: String) -> String {
        if id.count > 16 {
            return String(id.prefix(8)) + "..." + String(id.suffix(6))
        }
        return id
    }

    @ViewBuilder
    private var detailView: some View {
        switch appState.selectedNavigation {
        case .dashboard:
            DashboardView()
        case .status:
            DaemonStatusView()
        case .network:
            NetworkView()
        case .messaging:
            SpaceView()
        case .directMessages:
            DirectMessagesView()
        case .contacts:
            ContactsView()
        case .groups:
            GroupsView()
        case .settings:
            SettingsView()
        case nil:
            Text("Select an item from the sidebar.")
                .foregroundStyle(.secondary)
        }
    }
}
