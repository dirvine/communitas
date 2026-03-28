import SwiftUI
import X0xClient

/// Sheet for creating or joining a Space (group).
struct CreateSpaceSheet: View {
    @EnvironmentObject var appState: AppState
    @Environment(\.dismiss) private var dismiss

    enum Tab: String, CaseIterable {
        case create = "Create"
        case join = "Join"
    }

    @State private var selectedTab: Tab = .create

    // Create fields
    @State private var name = ""
    @State private var description = ""

    // Join fields
    @State private var inviteLink = ""

    // Shared
    @State private var displayName = ""
    @State private var isSubmitting = false
    @State private var error: String?

    var body: some View {
        VStack(spacing: 20) {
            // Header
            Text("Space")
                .font(.title2)
                .fontWeight(.semibold)
                .foregroundStyle(DeepSpace.textPrimary)

            // Tab picker
            Picker("", selection: $selectedTab) {
                ForEach(Tab.allCases, id: \.self) { tab in
                    Text(tab.rawValue).tag(tab)
                }
            }
            .pickerStyle(.segmented)

            // Form content
            Form {
                switch selectedTab {
                case .create:
                    Section {
                        TextField("Space Name", text: $name)
                        TextField("Description (optional)", text: $description)
                    }
                case .join:
                    Section {
                        TextField("Invite Link or Token", text: $inviteLink)
                            .font(.system(.body, design: .monospaced))
                    }
                }

                Section("Your Identity") {
                    TextField("Display Name (optional)", text: $displayName)
                }
            }
            .formStyle(.grouped)

            // Error
            if let error {
                Text(error)
                    .foregroundStyle(DeepSpace.red)
                    .font(.caption)
            }

            // Actions
            HStack {
                Button("Cancel") { dismiss() }
                    .keyboardShortcut(.cancelAction)
                Spacer()
                Button(selectedTab == .create ? "Create" : "Join") {
                    submit()
                }
                .buttonStyle(.borderedProminent)
                .tint(DeepSpace.cyan)
                .disabled(isSubmitDisabled)
                .keyboardShortcut(.defaultAction)
            }
        }
        .padding(20)
        .frame(width: 440, height: 400)
    }

    private var isSubmitDisabled: Bool {
        if isSubmitting { return true }
        switch selectedTab {
        case .create:
            return name.trimmingCharacters(in: .whitespaces).isEmpty
        case .join:
            return inviteLink.trimmingCharacters(in: .whitespaces).isEmpty
        }
    }

    private func submit() {
        isSubmitting = true
        error = nil
        let trimmedDisplay = displayName.trimmingCharacters(in: .whitespaces)

        Task {
            defer { isSubmitting = false }
            do {
                switch selectedTab {
                case .create:
                    let trimmedDesc = description.trimmingCharacters(in: .whitespaces)
                    _ = try await appState.client.createGroup(
                        name: name.trimmingCharacters(in: .whitespaces),
                        description: trimmedDesc.isEmpty ? nil : trimmedDesc,
                        displayName: trimmedDisplay.isEmpty ? nil : trimmedDisplay
                    )
                case .join:
                    _ = try await appState.client.joinGroup(
                        invite: inviteLink.trimmingCharacters(in: .whitespaces),
                        displayName: trimmedDisplay.isEmpty ? nil : trimmedDisplay
                    )
                }
                await appState.refresh()
                dismiss()
            } catch {
                self.error = error.localizedDescription
            }
        }
    }
}
