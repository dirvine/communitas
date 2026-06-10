import SwiftUI
import AppKit
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
            SpaceTabButtonBar(selection: $appState.selectedSpaceTab)
                .frame(width: 440, height: 28)

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

private struct SpaceTabButtonBar: NSViewRepresentable {
    @Binding var selection: SpaceTab

    func makeCoordinator() -> Coordinator {
        Coordinator(selection: $selection)
    }

    func makeNSView(context: Context) -> NSStackView {
        let stack = NSStackView()
        stack.orientation = .horizontal
        stack.alignment = .centerY
        stack.distribution = .fillEqually
        stack.spacing = 4

        for (index, tab) in SpaceTab.allCases.enumerated() {
            let button = NSButton(
                title: tab.rawValue,
                target: context.coordinator,
                action: #selector(Coordinator.selectionChanged(_:))
            )
            button.tag = index
            button.setButtonType(.momentaryPushIn)
            button.bezelStyle = .rounded
            button.controlSize = .small
            button.font = .systemFont(ofSize: NSFont.smallSystemFontSize)
            button.setAccessibilityIdentifier("SpaceTab-\(tab.rawValue)")
            button.setAccessibilityLabel(tab.rawValue)
            stack.addArrangedSubview(button)
        }

        return stack
    }

    func updateNSView(_ stack: NSStackView, context: Context) {
        context.coordinator.selection = $selection
        for (index, view) in stack.arrangedSubviews.enumerated() {
            guard let button = view as? NSButton else { continue }
            let isSelected = SpaceTab.allCases[index] == selection
            button.state = isSelected ? .on : .off
            button.contentTintColor = isSelected ? .controlAccentColor : .labelColor
        }
    }

    final class Coordinator: NSObject {
        var selection: Binding<SpaceTab>

        init(selection: Binding<SpaceTab>) {
            self.selection = selection
        }

        @objc func selectionChanged(_ sender: NSButton) {
            guard SpaceTab.allCases.indices.contains(sender.tag) else { return }
            selection.wrappedValue = SpaceTab.allCases[sender.tag]
        }
    }
}
