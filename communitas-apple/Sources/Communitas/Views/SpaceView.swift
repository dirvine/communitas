import SwiftUI
import X0xClient

/// Container view for a Space (group) with tabbed sub-views.
struct SpaceView: View {
    @EnvironmentObject var appState: AppState

    var body: some View {
        Group {
            if let group = appState.selectedGroup {
                VStack(spacing: 0) {
                    tabBar
                    Divider()
                    tabContent(group: group)
                }
            } else {
                noSpaceSelected
            }
        }
    }

    private var tabBar: some View {
        HStack(spacing: 0) {
            ForEach(SpaceTab.allCases) { tab in
                Button {
                    appState.selectedSpaceTab = tab
                } label: {
                    HStack(spacing: 4) {
                        Image(systemName: tab.systemImage)
                            .font(.caption)
                        Text(tab.rawValue)
                            .font(.caption)
                            .fontWeight(.medium)
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)
                    .background(
                        appState.selectedSpaceTab == tab
                            ? Color.accentColor.opacity(0.15)
                            : Color.clear,
                        in: RoundedRectangle(cornerRadius: 6)
                    )
                    .foregroundStyle(
                        appState.selectedSpaceTab == tab
                            ? Color.accentColor
                            : .secondary
                    )
                }
                .buttonStyle(.plain)
            }
            Spacer()
            if let group = appState.selectedGroup {
                Text(group.name)
                    .font(.caption)
                    .foregroundStyle(.tertiary)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.secondary.opacity(0.1), in: Capsule())
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(.bar)
    }

    @ViewBuilder
    private func tabContent(group: GroupSummary) -> some View {
        switch appState.selectedSpaceTab {
        case .chat:
            MessagingView()
        case .board:
            BoardView(groupId: group.groupId)
        case .files:
            FilesView()
        case .swarm:
            SwarmView(groupId: group.groupId)
        case .feed:
            FeedView(groupId: group.groupId)
        case .wiki:
            WikiView(groupId: group.groupId)
        case .web:
            WebPublishView(groupId: group.groupId)
        }
    }

    private var noSpaceSelected: some View {
        VStack(spacing: 12) {
            Image(systemName: "building.2")
                .font(.system(size: 48))
                .foregroundStyle(.secondary)
            Text("No Space Selected")
                .font(.title2)
            Text("Select a group and channel from the sidebar.")
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
