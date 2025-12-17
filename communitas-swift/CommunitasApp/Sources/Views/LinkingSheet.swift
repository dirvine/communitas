// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

import SwiftUI
import CommunitasKit

// MARK: - Linking Sheet

/// A sheet for linking local-only contacts or entities to network identities
/// via four-word addresses.
struct LinkingSheet: View {
    enum LinkTarget {
        case contact(ContactItem)
        case entity(SwiftEntity)

        var name: String {
            switch self {
            case .contact(let contact):
                return contact.effectiveName
            case .entity(let entity):
                return entity.name
            }
        }

        var id: String {
            switch self {
            case .contact(let contact):
                return contact.id
            case .entity(let entity):
                return entity.id
            }
        }

        var typeName: String {
            switch self {
            case .contact:
                return "Contact"
            case .entity(let entity):
                switch entity.entityType {
                case .organisation:
                    return "Organisation"
                case .group:
                    return "Group"
                case .channel:
                    return "Channel"
                case .project:
                    return "Project"
                case .person:
                    return "Person"
                }
            }
        }

        var icon: String {
            switch self {
            case .contact:
                return "person.fill"
            case .entity(let entity):
                switch entity.entityType {
                case .organisation:
                    return "building.2.fill"
                case .group:
                    return "person.3.fill"
                case .channel:
                    return "number"
                case .project:
                    return "folder.fill"
                case .person:
                    return "person.fill"
                }
            }
        }
    }

    let target: LinkTarget
    let onLink: (String) -> Void
    let onCancel: () -> Void

    @Environment(\.dismiss) var dismiss
    @State private var fourWords = ""
    @State private var isValidating = false
    @State private var validationError: String?

    /// Basic four-word format validation
    private var isValidFormat: Bool {
        let words = fourWords
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .replacingOccurrences(of: " ", with: "-")
            .split(separator: "-")
        return words.count == 4 && words.allSatisfy { !$0.isEmpty }
    }

    var body: some View {
        VStack(spacing: 0) {
            // Header
            header

            Divider()

            // Content
            VStack(alignment: .leading, spacing: 20) {
                // Current item info
                currentItemSection

                // Four-word input
                fourWordInputSection

                // Validation error
                if let error = validationError {
                    errorSection(error)
                }

                // Info about linking
                linkingInfoSection

                Spacer()

                // Action buttons
                actionButtons
            }
            .padding()
        }
        .frame(width: 450, height: 400)
    }

    // MARK: - Header

    private var header: some View {
        HStack {
            Image(systemName: "link.badge.plus")
                .font(.system(size: 28))
                .foregroundColor(.blue)

            VStack(alignment: .leading, spacing: 2) {
                Text("Link to Network")
                    .font(.title2)
                    .fontWeight(.semibold)
                Text("Connect \(target.typeName.lowercased()) to a network identity")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
            }

            Spacer()

            Button {
                onCancel()
                dismiss()
            } label: {
                Image(systemName: "xmark.circle.fill")
                    .font(.title2)
                    .foregroundColor(.secondary)
            }
            .buttonStyle(.plain)
        }
        .padding()
        .background(Color.gray.opacity(0.05))
    }

    // MARK: - Current Item Section

    private var currentItemSection: some View {
        HStack(spacing: 12) {
            Image(systemName: target.icon)
                .font(.title2)
                .foregroundColor(.orange)
                .frame(width: 40, height: 40)
                .background(Color.orange.opacity(0.15))
                .cornerRadius(8)

            VStack(alignment: .leading, spacing: 4) {
                Text(target.name)
                    .font(.headline)
                HStack(spacing: 4) {
                    Image(systemName: "lock.fill")
                        .font(.caption2)
                        .foregroundColor(.orange)
                    Text("Local-only \(target.typeName.lowercased())")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }

            Spacer()

            // Local badge
            Text("Local")
                .font(.caption2)
                .fontWeight(.medium)
                .foregroundColor(.orange)
                .padding(.horizontal, 8)
                .padding(.vertical, 4)
                .background(Color.orange.opacity(0.15))
                .cornerRadius(4)
        }
        .padding()
        .background(Color.gray.opacity(0.05))
        .cornerRadius(8)
    }

    // MARK: - Four Word Input Section

    private var fourWordInputSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text("Four-Word Address")
                    .font(.headline)
                if isValidating {
                    ProgressView()
                        .scaleEffect(0.7)
                }
            }

            TextField("e.g. ocean forest moon star", text: $fourWords)
                .textFieldStyle(.roundedBorder)
                .onChange(of: fourWords) { _, _ in
                    validationError = nil
                }

            Text("Enter the network identity to link this \(target.typeName.lowercased()) to")
                .font(.caption)
                .foregroundColor(.secondary)
        }
    }

    // MARK: - Error Section

    private func errorSection(_ error: String) -> some View {
        HStack(spacing: 8) {
            Image(systemName: "exclamationmark.triangle.fill")
                .foregroundColor(.red)
            Text(error)
                .font(.caption)
                .foregroundColor(.red)
        }
        .padding()
        .background(Color.red.opacity(0.1))
        .cornerRadius(6)
    }

    // MARK: - Linking Info Section

    private var linkingInfoSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("What happens when you link:")
                .font(.subheadline)
                .fontWeight(.medium)

            VStack(alignment: .leading, spacing: 8) {
                infoRow(icon: "arrow.triangle.2.circlepath", text: "Data will sync bidirectionally with the network")
                infoRow(icon: "person.2.fill", text: "Other peers can discover and interact")
                infoRow(icon: "lock.open.fill", text: "Local-only restriction is removed")
            }
        }
        .padding()
        .background(Color.blue.opacity(0.05))
        .cornerRadius(8)
    }

    private func infoRow(icon: String, text: String) -> some View {
        HStack(spacing: 10) {
            Image(systemName: icon)
                .font(.caption)
                .foregroundColor(.blue)
                .frame(width: 16)
            Text(text)
                .font(.caption)
                .foregroundColor(.secondary)
        }
    }

    // MARK: - Action Buttons

    private var actionButtons: some View {
        HStack(spacing: 12) {
            Button("Cancel") {
                onCancel()
                dismiss()
            }
            .buttonStyle(.bordered)

            Spacer()

            Button {
                performLink()
            } label: {
                HStack {
                    if isValidating {
                        ProgressView()
                            .scaleEffect(0.7)
                    } else {
                        Image(systemName: "link")
                    }
                    Text("Link to Network")
                }
            }
            .buttonStyle(.borderedProminent)
            .disabled(!isValidFormat || isValidating)
        }
    }

    // MARK: - Actions

    private func performLink() {
        guard isValidFormat else { return }

        isValidating = true
        validationError = nil

        let normalizedFourWords = fourWords
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .replacingOccurrences(of: " ", with: "-")
            .lowercased()

        // Perform the link via callback
        onLink(normalizedFourWords)
        dismiss()
    }
}

// MARK: - Contact Linking Sheet Wrapper

/// Convenience wrapper for linking a contact
struct ContactLinkingSheet: View {
    let contact: ContactItem
    @EnvironmentObject var state: AppState
    @Environment(\.dismiss) var dismiss

    var body: some View {
        LinkingSheet(
            target: .contact(contact),
            onLink: { fourWords in
                state.linkContact(contactId: contact.id, fourWords: fourWords)
            },
            onCancel: { }
        )
    }
}

// MARK: - Entity Linking Sheet Wrapper

/// Convenience wrapper for linking an entity
struct EntityLinkingSheet: View {
    let entity: SwiftEntity
    @EnvironmentObject var state: AppState
    @Environment(\.dismiss) var dismiss

    var body: some View {
        LinkingSheet(
            target: .entity(entity),
            onLink: { fourWords in
                state.linkEntity(entityId: entity.id, fourWords: fourWords)
            },
            onCancel: { }
        )
    }
}

// MARK: - Preview

#Preview("Contact Linking") {
    let sampleContact = ContactItem(
        id: "test-id",
        fourWords: nil,
        displayName: "Alice from Work",
        isFavourite: false,
        isOnline: false,
        lastSeen: nil,
        isLocalOnly: true
    )

    return LinkingSheet(
        target: .contact(sampleContact),
        onLink: { fourWords in
            print("Linking to: \(fourWords)")
        },
        onCancel: { }
    )
}
