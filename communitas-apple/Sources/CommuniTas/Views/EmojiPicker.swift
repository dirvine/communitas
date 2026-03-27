import SwiftUI

/// Full emoji picker with category tabs, search bar, and scrollable grid.
struct EmojiPicker: View {
    let onSelect: (String) -> Void

    @State private var searchQuery = ""
    @State private var selectedCategory: EmojiCategory = .smileys

    private let columns = Array(repeating: GridItem(.flexible(), spacing: 4), count: 8)

    var body: some View {
        VStack(spacing: 0) {
            searchBar
            categoryTabs
            Divider()
            emojiGrid
        }
        .frame(width: 320, height: 360)
        .background(.background)
        .clipShape(RoundedRectangle(cornerRadius: 12))
        .shadow(color: .black.opacity(0.15), radius: 12, x: 0, y: 4)
    }

    // MARK: - Search Bar

    private var searchBar: some View {
        HStack(spacing: 6) {
            Image(systemName: "magnifyingglass")
                .foregroundStyle(.secondary)
                .font(.footnote)
            TextField("Search emoji…", text: $searchQuery)
                .textFieldStyle(.plain)
                .font(.footnote)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 7)
        .background(Color.secondary.opacity(0.08), in: RoundedRectangle(cornerRadius: 8))
        .padding(10)
    }

    // MARK: - Category Tabs

    private var categoryTabs: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 2) {
                ForEach(EmojiCategory.allCases) { category in
                    Button {
                        selectedCategory = category
                        searchQuery = ""
                    } label: {
                        Image(systemName: category.systemImage)
                            .font(.system(size: 14))
                            .frame(width: 32, height: 28)
                            .foregroundStyle(selectedCategory == category ? Color.accentColor : .secondary)
                            .background(
                                selectedCategory == category
                                    ? Color.accentColor.opacity(0.12)
                                    : Color.clear,
                                in: RoundedRectangle(cornerRadius: 6)
                            )
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.horizontal, 10)
            .padding(.bottom, 6)
        }
    }

    // MARK: - Emoji Grid

    private var emojiGrid: some View {
        ScrollView {
            let emojis = displayedEmojis
            if emojis.isEmpty {
                Text("No results")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, minHeight: 80)
            } else {
                LazyVGrid(columns: columns, spacing: 4) {
                    ForEach(emojis, id: \.emoji) { entry in
                        Button {
                            onSelect(entry.emoji)
                        } label: {
                            Text(entry.emoji)
                                .font(.system(size: 22))
                                .frame(width: 32, height: 32)
                        }
                        .buttonStyle(.plain)
                        .help(entry.name)
                    }
                }
                .padding(10)
            }
        }
    }

    // MARK: - Helpers

    private var displayedEmojis: [EmojiEntry] {
        if searchQuery.trimmingCharacters(in: .whitespaces).isEmpty {
            return EmojiData.byCategory(selectedCategory)
        } else {
            return EmojiData.search(searchQuery)
        }
    }
}
