import Foundation

/// Sidebar navigation items.
enum NavigationItem: String, CaseIterable, Identifiable, Hashable {
    case status = "Status"
    case messaging = "Messaging"
    case contacts = "Contacts"
    case groups = "Groups"
    case settings = "Settings"

    var id: String { rawValue }

    var systemImage: String {
        switch self {
        case .status: return "circle.fill"
        case .messaging: return "message"
        case .contacts: return "person.2"
        case .groups: return "bubble.left.and.bubble.right"
        case .settings: return "gear"
        }
    }
}
