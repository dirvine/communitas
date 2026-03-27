import SwiftUI

/// A candidate for @mention autocomplete.
struct MentionCandidate: Identifiable, Equatable {
    /// The agent ID (identity) of the contact.
    let id: String
    /// Human-readable display name shown in the autocomplete list and inserted on selection.
    let displayName: String
}

/// Dropdown list of @mention candidates filtered by a query string.
struct MentionAutocomplete: View {
    /// All candidates to filter from.
    let candidates: [MentionCandidate]
    /// The query text after the `@` character (case-insensitive prefix match).
    let query: String
    /// Called when the user taps a candidate.
    let onSelect: (MentionCandidate) -> Void

    private var filtered: [MentionCandidate] {
        let q = query.lowercased()
        if q.isEmpty {
            return Array(candidates.prefix(6))
        }
        return candidates
            .filter { $0.displayName.lowercased().hasPrefix(q) }
            .prefix(6)
            .map { $0 }
    }

    var body: some View {
        if filtered.isEmpty {
            EmptyView()
        } else {
            VStack(alignment: .leading, spacing: 0) {
                ForEach(filtered) { candidate in
                    Button {
                        onSelect(candidate)
                    } label: {
                        HStack(spacing: 8) {
                            // Avatar initial circle
                            ZStack {
                                Circle()
                                    .fill(avatarColor(for: candidate.id))
                                    .frame(width: 28, height: 28)
                                Text(String(candidate.displayName.prefix(1)).uppercased())
                                    .font(.caption)
                                    .fontWeight(.semibold)
                                    .foregroundStyle(.white)
                            }

                            Text(candidate.displayName)
                                .font(.subheadline)
                                .foregroundStyle(.primary)

                            Spacer()
                        }
                        .padding(.horizontal, 10)
                        .padding(.vertical, 6)
                        .contentShape(Rectangle())
                    }
                    .buttonStyle(.plain)
                    .background(Color.clear)

                    if candidate.id != filtered.last?.id {
                        Divider()
                            .padding(.horizontal, 10)
                    }
                }
            }
            .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 8))
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .strokeBorder(Color.secondary.opacity(0.2), lineWidth: 1)
            )
            .shadow(color: .black.opacity(0.12), radius: 8, x: 0, y: 4)
            .frame(maxWidth: 280)
        }
    }

    private func avatarColor(for id: String) -> Color {
        let colors: [Color] = [.blue, .purple, .orange, .green, .pink, .teal, .indigo, .mint]
        return colors[abs(id.hashValue) % colors.count]
    }
}
