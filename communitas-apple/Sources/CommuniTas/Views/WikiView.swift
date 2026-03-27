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
    @State private var isLoading = true
    @State private var newSlug = ""
    @State private var showCreatePage = false

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
            HSplitView {
                pageList
                    .frame(minWidth: 180, maxWidth: 240)
                pageDetail
            }
        }
        .task {
            await loadIndex()
            isLoading = false
        }
    }

    private var wikiHeader: some View {
        HStack {
            Image(systemName: "book")
                .foregroundStyle(.secondary)
            Text("Wiki")
                .font(.headline)
            Spacer()
            Button {
                showCreatePage = true
            } label: {
                Label("New Page", systemImage: "plus")
                    .font(.caption)
            }
            .buttonStyle(.bordered)
            .controlSize(.small)
            .popover(isPresented: $showCreatePage) {
                createPagePopover
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(.bar)
    }

    // MARK: - Page List

    private var pageList: some View {
        List(pageSlugs, id: \.self, selection: $selectedSlug) { slug in
            HStack {
                Image(systemName: "doc.text")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Text(slug)
                    .font(.body)
            }
            .tag(slug)
        }
        .listStyle(.sidebar)
        .onChange(of: selectedSlug) {
            isEditing = false
            if let slug = selectedSlug {
                Task { await loadPage(slug: slug) }
            }
        }
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
                Button {
                    editBuffer = pageContent
                    isEditing = true
                } label: {
                    Label("Edit", systemImage: "pencil")
                        .font(.caption)
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
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
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Editing: \(slug)")
                    .font(.title2)
                    .fontWeight(.bold)
                Spacer()
                Button("Cancel") {
                    isEditing = false
                }
                .buttonStyle(.bordered)
                .controlSize(.small)

                Button("Save") {
                    Task { await savePage(slug: slug) }
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
            }

            TextEditor(text: $editBuffer)
                .font(.body)
                .scrollContentBackground(.hidden)
                .padding(8)
                .background(Color.secondary.opacity(0.06), in: RoundedRectangle(cornerRadius: 8))
        }
        .padding(16)
    }

    // MARK: - Create Page

    private var createPagePopover: some View {
        VStack(spacing: 12) {
            Text("Create Page")
                .font(.headline)

            TextField("Page slug", text: $newSlug)
                .textFieldStyle(.plain)
                .padding(8)
                .background(Color.secondary.opacity(0.06), in: RoundedRectangle(cornerRadius: 6))
                .help("Lowercase, hyphens for spaces")

            HStack {
                Button("Cancel") {
                    newSlug = ""
                    showCreatePage = false
                }
                .buttonStyle(.bordered)
                .controlSize(.small)

                Button("Create") {
                    Task { await createPage() }
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
                .disabled(newSlug.trimmingCharacters(in: .whitespaces).isEmpty)
            }
        }
        .padding(16)
        .frame(width: 280)
    }

    // MARK: - Actions

    private func ensureStore() async {
        do {
            let stores = try await appState.client.listStores()
            if !stores.contains(where: { $0.id == storeName }) {
                _ = try await appState.client.createStore(name: storeName, topic: "x0x.wiki.\(prefix)")
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

    private func savePage(slug: String) async {
        do {
            try await appState.client.storePut(storeId: storeName, key: pageKey(slug), value: editBuffer)
            pageContent = editBuffer
            isEditing = false
        } catch {
            appState.errorMessage = "Failed to save: \(error.localizedDescription)"
        }
    }

    private func createPage() async {
        let slug = newSlug
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
            newSlug = ""
            showCreatePage = false
        } catch {
            appState.errorMessage = "Failed to create page: \(error.localizedDescription)"
        }
    }
}
