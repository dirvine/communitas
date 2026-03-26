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
            // Secondary column: channel sidebar when messaging, or nothing
            if appState.selectedNavigation == .messaging {
                ChannelSidebarView()
                    .navigationSplitViewColumnWidth(min: 180, ideal: 220)
            } else {
                EmptyView()
            }
        } detail: {
            detailView
        }
        .navigationTitle("CommuniTas")
    }

    @ViewBuilder
    private var detailView: some View {
        switch appState.selectedNavigation {
        case .status:
            DaemonStatusView()
        case .messaging:
            MessagingView()
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
