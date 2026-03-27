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
    @State private var isLoading = true
    @State private var newPath = ""
    @State private var showCreatePage = false

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
            HSplitView {
                pathList
                    .frame(minWidth: 180, maxWidth: 240)
                pageDetail
            }
        }
        .task {
            await loadIndex()
            isLoading = false
        }
    }

    private var webHeader: some View {
        HStack {
            Image(systemName: "globe")
                .foregroundStyle(.secondary)
            Text("Web")
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

    // MARK: - Path List

    private var pathList: some View {
        List(pagePaths, id: \.self, selection: $selectedPath) { path in
            HStack {
                Image(systemName: "globe")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Text(path)
                    .font(.body)
            }
            .tag(path)
        }
        .listStyle(.sidebar)
        .onChange(of: selectedPath) {
            isEditing = false
            if let path = selectedPath {
                Task { await loadPage(path: path) }
            }
        }
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
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Editing: \(path)")
                    .font(.title2)
                    .fontWeight(.bold)
                Spacer()
                Button("Cancel") {
                    isEditing = false
                }
                .buttonStyle(.bordered)
                .controlSize(.small)

                Button("Publish") {
                    Task { await publishPage(path: path) }
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
            }

            TextEditor(text: $editBuffer)
                .font(.system(.body, design: .monospaced))
                .scrollContentBackground(.hidden)
                .padding(8)
                .background(Color.secondary.opacity(0.06), in: RoundedRectangle(cornerRadius: 8))
        }
        .padding(16)
    }

    // MARK: - Create Page

    private var createPagePopover: some View {
        VStack(spacing: 12) {
            Text("New Web Page")
                .font(.headline)

            TextField("Path (e.g. index.html)", text: $newPath)
                .textFieldStyle(.plain)
                .padding(8)
                .background(Color.secondary.opacity(0.06), in: RoundedRectangle(cornerRadius: 6))

            HStack {
                Button("Cancel") {
                    newPath = ""
                    showCreatePage = false
                }
                .buttonStyle(.bordered)
                .controlSize(.small)

                Button("Create") {
                    Task { await createPage() }
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
                .disabled(newPath.trimmingCharacters(in: .whitespaces).isEmpty)
            }
        }
        .padding(16)
        .frame(width: 280)
    }

    // MARK: - Actions

    private func loadIndex() async {
        do {
            let json = try await appState.client.kvGet(key: indexKey())
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
            pageContent = try await appState.client.kvGet(key: webKey(path))
        } catch {
            pageContent = ""
        }
    }

    private func publishPage(path: String) async {
        do {
            try await appState.client.kvSet(key: webKey(path), value: editBuffer)
            pageContent = editBuffer
            isEditing = false
        } catch {
            appState.errorMessage = "Failed to publish: \(error.localizedDescription)"
        }
    }

    private func createPage() async {
        let path = newPath.trimmingCharacters(in: .whitespaces)
        guard !path.isEmpty else { return }

        do {
            try await appState.client.kvSet(key: webKey(path), value: "")

            var paths = pagePaths
            if !paths.contains(path) {
                paths.append(path)
            }
            let indexData = try JSONEncoder().encode(paths)
            if let indexJson = String(data: indexData, encoding: .utf8) {
                try await appState.client.kvSet(key: indexKey(), value: indexJson)
            }

            pagePaths = paths
            selectedPath = path
            newPath = ""
            showCreatePage = false
        } catch {
            appState.errorMessage = "Failed to create page: \(error.localizedDescription)"
        }
    }
}
