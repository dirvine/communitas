import SwiftUI
import X0xClient

/// Website publishing via KvStore.
struct WebPublishView: View {
    let groupId: String
    @EnvironmentObject var appState: AppState

    @State private var pagePaths: [String] = []
    @State private var selectedPath: String?
    @State private var pageContent = ""
    @State private var isEditing = false
    @State private var editBuffer = ""
    @State private var isLoading = false
    @State private var hasLoadedIndex = false

    private var prefix: String {
        appState.groupPrefix(for: groupId)
    }

    private var storeName: String {
        "x0x-web-\(prefix)"
    }

    private func indexKey() -> String {
        "web_index"
    }

    private func webKey(_ path: String) -> String {
        "web:\(path)"
    }

    var body: some View {
        VStack(spacing: 0) {
            webHeader
            Divider()
            HStack(spacing: 0) {
                pathList
                    .frame(width: 220)
                Divider()
                pageDetail
            }
        }
        .onAppear {
            scheduleInitialLoad()
        }
    }

    private var webHeader: some View {
        HStack {
            Image(systemName: "globe")
                .foregroundStyle(.secondary)
            Text("Web")
                .font(.headline)
            Spacer()
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(.bar)
    }

    // MARK: - Path List

    private var pathList: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Pages")
                .font(.caption)
                .fontWeight(.semibold)
                .foregroundStyle(.secondary)

            AppKitPageCreatePanel(
                placeholder: "Path, e.g. index.html",
                buttonTitle: "Create",
                accessibilityPrefix: "web-create-page"
            ) { rawPath in
                Task { await createPage(rawPath: rawPath) }
                return true
            }
            .frame(height: 58)

            Divider()

            if pagePaths.isEmpty {
                Text("No pages yet")
                    .font(.caption)
                    .foregroundStyle(.tertiary)
            } else {
                ScrollView {
                    VStack(alignment: .leading, spacing: 4) {
                        ForEach(pagePaths, id: \.self) { path in
                            Button {
                                selectPath(path)
                            } label: {
                                HStack(spacing: 6) {
                                    Image(systemName: "globe")
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                    Text(path)
                                        .font(.body)
                                    Spacer()
                                }
                                .padding(.horizontal, 8)
                                .padding(.vertical, 6)
                                .background(
                                    selectedPath == path ? Color.accentColor.opacity(0.12) : Color.clear,
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
            ProgressView("Loading pages...")
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if let path = selectedPath {
            if isEditing {
                editorView(path: path)
            } else {
                previewView(path: path)
            }
        } else {
            VStack(spacing: 12) {
                Image(systemName: "globe")
                    .font(.system(size: 36))
                    .foregroundStyle(.secondary)
                Text("Select a page to preview")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
    }

    private func previewView(path: String) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text(path)
                    .font(.title2)
                    .fontWeight(.bold)
                Spacer()
                AppKitInlineButton(
                    title: "Edit",
                    systemSymbolName: "pencil",
                    accessibilityIdentifier: "web-edit-page-button"
                ) {
                    editBuffer = pageContent
                    isEditing = true
                }
                .frame(width: 74, height: 26)
            }

            ScrollView {
                if pageContent.isEmpty {
                    Text("This page is empty. Click Edit to add content.")
                        .font(.body)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                } else {
                    Text(pageContent)
                        .font(.system(.body, design: .monospaced))
                        .textSelection(.enabled)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(12)
                        .background(Color.secondary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8))
                }
            }
        }
        .padding(16)
    }

    private func editorView(path: String) -> some View {
        AppKitTextEditorPanel(
            title: "Editing: \(path)",
            content: editBuffer,
            saveTitle: "Publish",
            accessibilityPrefix: "web-editor",
            isMonospaced: true
        ) {
            isEditing = false
        } onSave: { content in
            Task { await publishPage(path: path, content: content) }
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

    private func selectPath(_ path: String) {
        selectedPath = path
        isEditing = false
        Task { await loadPage(path: path) }
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
               let paths = try? JSONDecoder().decode([String].self, from: data) {
                pagePaths = paths
            }
        } catch {
            pagePaths = []
        }
    }

    private func loadPage(path: String) async {
        do {
            pageContent = try await appState.client.storeGet(storeId: storeName, key: webKey(path))
        } catch {
            pageContent = ""
        }
    }

    private func publishPage(path: String, content: String) async {
        do {
            try await appState.client.storePut(storeId: storeName, key: webKey(path), value: content)
            pageContent = content
            editBuffer = content
            isEditing = false
        } catch {
            appState.errorMessage = "Failed to publish: \(error.localizedDescription)"
        }
    }

    private func createPage(rawPath: String) async {
        let path = rawPath.trimmingCharacters(in: .whitespaces)
        guard !path.isEmpty else { return }

        do {
            try await appState.client.storePut(storeId: storeName, key: webKey(path), value: "")

            var paths = pagePaths
            if !paths.contains(path) {
                paths.append(path)
            }
            let indexData = try JSONEncoder().encode(paths)
            if let indexJson = String(data: indexData, encoding: .utf8) {
                try await appState.client.storePut(storeId: storeName, key: indexKey(), value: indexJson)
            }

            pagePaths = paths
            selectedPath = path
            pageContent = ""
            editBuffer = ""
            isEditing = true
        } catch {
            appState.errorMessage = "Failed to create page: \(error.localizedDescription)"
        }
    }
}
