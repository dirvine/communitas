import SwiftUI
import X0xClient

/// Cmd+K fuzzy-search overlay for quickly jumping to spaces or channels.
struct QuickSwitcherView: View {
    @EnvironmentObject var appState: AppState
    @State private var query = ""
    @State private var selectedIndex = 0
    @FocusState private var isFocused: Bool

    var body: some View {
        VStack(spacing: 0) {
            // Search field
            HStack(spacing: 8) {
                Image(systemName: "magnifyingglass")
                    .foregroundStyle(.secondary)
                TextField("Jump to space or channel...", text: $query)
                    .textFieldStyle(.plain)
                    .font(.title3)
                    .focused($isFocused)
                    .onSubmit {
                        selectCurrent()
                    }

                if !query.isEmpty {
                    Button {
                        query = ""
                    } label: {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundStyle(.secondary)
                    }
                    .buttonStyle(.plain)
                }

                Text("esc")
                    .font(.caption2)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(.quaternary, in: RoundedRectangle(cornerRadius: 4))
            }
            .padding(12)

            Divider()

            // Results
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 0) {
                    ForEach(Array(filteredItems.enumerated()), id: \.element.id) { index, item in
                        Button {
                            select(item)
                        } label: {
                            HStack(spacing: 8) {
                                Image(systemName: item.icon)
                                    .foregroundStyle(item.iconColor)
                                    .frame(width: 20)
                                VStack(alignment: .leading, spacing: 1) {
                                    Text(item.title)
                                        .font(.body)
                                    if let subtitle = item.subtitle {
                                        Text(subtitle)
                                            .font(.caption)
                                            .foregroundStyle(.secondary)
                                    }
                                }
                                Spacer()
                                if index < 9 {
                                    Text("\(index + 1)")
                                        .font(.caption2)
                                        .foregroundStyle(.tertiary)
                                        .padding(.horizontal, 5)
                                        .padding(.vertical, 2)
                                        .background(.quaternary, in: RoundedRectangle(cornerRadius: 4))
                                }
                            }
                            .padding(.horizontal, 12)
                            .padding(.vertical, 8)
                            .background(index == selectedIndex ? Color.accentColor.opacity(0.12) : .clear)
                            .contentShape(Rectangle())
                        }
                        .buttonStyle(.plain)
                    }

                    if filteredItems.isEmpty {
                        Text("No results")
                            .foregroundStyle(.secondary)
                            .padding(20)
                            .frame(maxWidth: .infinity)
                    }
                }
            }
            .frame(maxHeight: 300)
        }
        .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 12))
        .overlay(RoundedRectangle(cornerRadius: 12).stroke(.quaternary))
        .shadow(color: .black.opacity(0.3), radius: 20, y: 10)
        .frame(width: 500)
        .onAppear {
            isFocused = true
            selectedIndex = 0
        }
        .onChange(of: query) { _, _ in
            selectedIndex = 0
        }
        .onKeyPress(.upArrow) {
            if selectedIndex > 0 { selectedIndex -= 1 }
            return .handled
        }
        .onKeyPress(.downArrow) {
            if selectedIndex < filteredItems.count - 1 { selectedIndex += 1 }
            return .handled
        }
        .onKeyPress(.escape) {
            appState.showQuickSwitcher = false
            return .handled
        }
    }

    // MARK: - Data

    private struct SwitcherItem: Identifiable {
        let id: String
        let title: String
        let subtitle: String?
        let icon: String
        let iconColor: Color
        let action: () -> Void
    }

    private var allItems: [SwitcherItem] {
        var items: [SwitcherItem] = []

        // Spaces
        for group in appState.groups {
            items.append(SwitcherItem(
                id: "space-\(group.groupId)",
                title: group.name,
                subtitle: "Space",
                icon: "circle.fill",
                iconColor: colorForId(group.groupId)
            ) {
                Task {
                    await appState.selectGroupAndChannel(group: group, channel: "general")
                    appState.selectedSpaceTab = .chat
                    appState.selectedSystemPage = nil
                    appState.selectedDMContact = nil
                }
            })

            // Channels within this space
            let manager = appState.channelManager(for: group)
            for channel in manager.channels {
                items.append(SwitcherItem(
                    id: "channel-\(group.groupId)-\(channel.name)",
                    title: "#\(channel.name)",
                    subtitle: group.name,
                    icon: "number",
                    iconColor: .secondary
                ) {
                    Task {
                        await appState.selectGroupAndChannel(group: group, channel: channel.name)
                        appState.selectedSpaceTab = .chat
                        appState.selectedSystemPage = nil
                        appState.selectedDMContact = nil
                    }
                })
            }
        }

        // System pages
        for page in SystemPage.allCases {
            items.append(SwitcherItem(
                id: "system-\(page.rawValue)",
                title: page.rawValue,
                subtitle: "System",
                icon: page.systemImage,
                iconColor: .accentColor
            ) {
                appState.selectedSystemPage = page
                appState.selectedDMContact = nil
            })
        }

        return items
    }

    private var filteredItems: [SwitcherItem] {
        guard !query.isEmpty else { return allItems }
        let lowered = query.lowercased()
        return allItems.filter { item in
            item.title.lowercased().contains(lowered)
                || (item.subtitle?.lowercased().contains(lowered) ?? false)
        }
    }

    private func selectCurrent() {
        guard selectedIndex < filteredItems.count else { return }
        select(filteredItems[selectedIndex])
    }

    private func select(_ item: SwitcherItem) {
        item.action()
        appState.showQuickSwitcher = false
    }

    private func colorForId(_ id: String) -> Color {
        let palette: [Color] = [.blue, .purple, .orange, .green, .pink, .teal, .indigo, .mint]
        let index = abs(id.hashValue) % palette.count
        return palette[index]
    }
}
