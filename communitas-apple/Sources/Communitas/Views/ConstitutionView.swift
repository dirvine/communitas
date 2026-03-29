import SwiftUI
import X0xClient

/// Displays the x0x Constitution fetched from the local daemon.
///
/// The constitution is a foundational document defining the rights,
/// responsibilities, and governance of all Intelligent Entities on the
/// x0x network. It is embedded in every x0x binary at compile time.
struct ConstitutionView: View {
    @EnvironmentObject var appState: AppState

    @State private var constitutionInfo: ConstitutionInfo?
    @State private var errorMessage: String?
    @State private var isLoading = true

    var body: some View {
        Group {
            if isLoading {
                ProgressView("Loading constitution…")
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if let info = constitutionInfo {
                constitutionContent(info)
            } else {
                errorView
            }
        }
        .navigationTitle("Constitution")
        .toolbar {
            ToolbarItem(placement: .automatic) {
                Button {
                    Task { await fetchConstitution() }
                } label: {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .disabled(isLoading)
            }
        }
        .task {
            await fetchConstitution()
        }
    }

    // MARK: - Content

    private func constitutionContent(_ info: ConstitutionInfo) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                // Version badge
                HStack(spacing: 8) {
                    Label("v\(info.version)", systemImage: "doc.text")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    Text("•")
                        .foregroundStyle(.secondary)
                    Text(info.status)
                        .font(.caption)
                        .foregroundStyle(.tint)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 2)
                        .background(.tint.opacity(0.12))
                        .clipShape(Capsule())
                }
                .padding(.bottom, 4)

                // Render the markdown content
                Text(markdownAttributedString(info.content))
                    .textSelection(.enabled)
                    .font(.body)
                    .lineSpacing(4)
            }
            .padding(32)
            .frame(maxWidth: 720, alignment: .leading)
            .frame(maxWidth: .infinity)
        }
    }

    private var errorView: some View {
        VStack(spacing: 16) {
            Image(systemName: "exclamationmark.triangle")
                .font(.system(size: 40))
                .foregroundStyle(.secondary)
            Text("Could not load constitution")
                .font(.headline)
            if let errorMessage {
                Text(errorMessage)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
            }
            Text("The constitution is embedded in the x0x daemon.\nMake sure x0xd is running.")
                .font(.caption)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
            Button("Retry") {
                Task { await fetchConstitution() }
            }
            .buttonStyle(.bordered)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding(40)
    }

    // MARK: - Data Fetching

    private func fetchConstitution() async {
        isLoading = true
        errorMessage = nil

        do {
            constitutionInfo = try await appState.client.constitutionJSON()
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }

    // MARK: - Markdown Rendering

    /// Convert raw markdown to an `AttributedString` for SwiftUI `Text`.
    private func markdownAttributedString(_ markdown: String) -> AttributedString {
        do {
            return try AttributedString(
                markdown: markdown,
                options: .init(interpretedSyntax: .inlineOnlyPreservingWhitespace)
            )
        } catch {
            return AttributedString(markdown)
        }
    }
}
