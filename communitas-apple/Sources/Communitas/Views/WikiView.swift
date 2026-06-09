import SwiftUI
import X0xClient

/// Collaborative wiki pages via KvStore.
struct WikiView: View {
    let groupId: String
    @EnvironmentObject var appState: AppState

    @State private var pageSlugs: [String] = []
    @State private var selectedSlug: String?
    @State private var pageContent = ""
    @State private var isEditing = false
    @State private var editBuffer = ""
    @State private var isLoading = false
    @State private var hasLoadedIndex = false

    private var prefix: String {
        appState.groupPrefix(for: groupId)
    }

    private var storeName: String {
        "x0x-wiki-\(prefix)"
    }

    private func indexKey() -> String {
        "wiki_index"
    }

    private func pageKey(_ slug: String) -> String {
        "wiki:\(slug)"
    }

    var body: some View {
        VStack(spacing: 0) {
            wikiHeader
            Divider()
            HStack(spacing: 0) {
                pageList
                    .frame(width: 220)
                Divider()
                pageDetail
            }
        }
        .onAppear {
            scheduleInitialLoad()
        }
    }

    private var wikiHeader: some View {
        HStack {
            Image(systemName: "book")
                .foregroundStyle(.secondary)
            Text("Wiki")
                .font(.headline)
            Spacer()
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(.bar)
    }

    // MARK: - Page List

    private var pageList: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Pages")
                .font(.caption)
                .fontWeight(.semibold)
                .foregroundStyle(.secondary)

            AppKitPageCreatePanel(
                placeholder: "Page slug",
                buttonTitle: "Create",
                accessibilityPrefix: "wiki-create-page"
            ) { rawSlug in
                Task { await createPage(rawSlug: rawSlug) }
                return true
            }
            .frame(height: 58)

            Divider()

            if pageSlugs.isEmpty {
                Text("No pages yet")
                    .font(.caption)
                    .foregroundStyle(.tertiary)
            } else {
                ScrollView {
                    VStack(alignment: .leading, spacing: 4) {
                        ForEach(pageSlugs, id: \.self) { slug in
                            Button {
                                selectPage(slug)
                            } label: {
                                HStack(spacing: 6) {
                                    Image(systemName: "doc.text")
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                    Text(slug)
                                        .font(.body)
                                    Spacer()
                                }
                                .padding(.horizontal, 8)
                                .padding(.vertical, 6)
                                .background(
                                    selectedSlug == slug ? Color.accentColor.opacity(0.12) : Color.clear,
                                    in: RoundedRectangle(cornerRadius: 6)
                                )
                            }
                            .buttonStyle(.plain)
                        }
                    }
                }
            }
        }
        .padding(12)
    }

    // MARK: - Page Detail

    @ViewBuilder
    private var pageDetail: some View {
        if isLoading {
            ProgressView("Loading wiki...")
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if let slug = selectedSlug {
            if isEditing {
                editorView(slug: slug)
            } else {
                readerView(slug: slug)
            }
        } else {
            VStack(spacing: 12) {
                Image(systemName: "book.closed")
                    .font(.system(size: 36))
                    .foregroundStyle(.secondary)
                Text("Select a page")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
    }

    private func readerView(slug: String) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text(slug)
                    .font(.title2)
                    .fontWeight(.bold)
                Spacer()
                AppKitInlineButton(
                    title: "Edit",
                    systemSymbolName: "pencil",
                    accessibilityIdentifier: "wiki-edit-page-button"
                ) {
                    editBuffer = pageContent
                    isEditing = true
                }
                .frame(width: 74, height: 26)
            }

            ScrollView {
                Text(pageContent.isEmpty ? "This page is empty. Click Edit to add content." : pageContent)
                    .font(.body)
                    .foregroundStyle(pageContent.isEmpty ? .secondary : .primary)
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .padding(16)
    }

    private func editorView(slug: String) -> some View {
        AppKitTextEditorPanel(
            title: "Editing: \(slug)",
            content: editBuffer,
            saveTitle: "Save",
            accessibilityPrefix: "wiki-editor",
            isMonospaced: false
        ) {
            isEditing = false
        } onSave: { content in
            Task { await savePage(slug: slug, content: content) }
            return true
        }
        .padding(16)
    }

    // MARK: - Actions

    private func scheduleInitialLoad() {
        guard !hasLoadedIndex else { return }
        hasLoadedIndex = true
        DispatchQueue.main.async {
            Task { @MainActor in
                await loadIndex()
                isLoading = false
            }
        }
    }

    private func selectPage(_ slug: String) {
        selectedSlug = slug
        isEditing = false
        Task { await loadPage(slug: slug) }
    }

    private func ensureStore() async {
        do {
            let stores = try await appState.client.listStores()
            if !stores.contains(where: { $0.id == storeName }) {
                _ = try await appState.client.createStore(name: storeName, topic: storeName)
            }
        } catch { /* store may already exist */ }
    }

    private func loadIndex() async {
        await ensureStore()
        do {
            let json = try await appState.client.storeGet(storeId: storeName, key: indexKey())
            if let data = json.data(using: .utf8),
               let slugs = try? JSONDecoder().decode([String].self, from: data) {
                pageSlugs = slugs
            }
        } catch {
            pageSlugs = []
        }
    }

    private func loadPage(slug: String) async {
        do {
            pageContent = try await appState.client.storeGet(storeId: storeName, key: pageKey(slug))
        } catch {
            pageContent = ""
        }
    }

    private func savePage(slug: String, content: String) async {
        do {
            try await appState.client.storePut(storeId: storeName, key: pageKey(slug), value: content)
            pageContent = content
            editBuffer = content
            isEditing = false
        } catch {
            appState.errorMessage = "Failed to save: \(error.localizedDescription)"
        }
    }

    private func createPage(rawSlug: String) async {
        let slug = rawSlug
            .trimmingCharacters(in: .whitespaces)
            .lowercased()
            .replacingOccurrences(of: " ", with: "-")
        guard !slug.isEmpty else { return }

        do {
            // Save empty page
            try await appState.client.storePut(storeId: storeName, key: pageKey(slug), value: "")

            // Update index
            var slugs = pageSlugs
            if !slugs.contains(slug) {
                slugs.append(slug)
            }
            let indexData = try JSONEncoder().encode(slugs)
            if let indexJson = String(data: indexData, encoding: .utf8) {
                try await appState.client.storePut(storeId: storeName, key: indexKey(), value: indexJson)
            }

            pageSlugs = slugs
            selectedSlug = slug
            pageContent = ""
            editBuffer = ""
            isEditing = true
        } catch {
            appState.errorMessage = "Failed to create page: \(error.localizedDescription)"
        }
    }
}
