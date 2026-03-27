import Foundation

/// Sidebar navigation items.
enum NavigationItem: String, CaseIterable, Identifiable, Hashable {
    case dashboard = "Dashboard"
    case status = "Status"
    case network = "Network"
    case messaging = "Spaces"
    case directMessages = "Direct Messages"
    case contacts = "Contacts"
    case groups = "Groups"
    case settings = "Settings"

    var id: String { rawValue }

    var systemImage: String {
        switch self {
        case .dashboard: return "square.grid.2x2"
        case .status: return "circle.fill"
        case .network: return "network"
        case .messaging: return "building.2"
        case .directMessages: return "envelope"
        case .contacts: return "person.2"
        case .groups: return "bubble.left.and.bubble.right"
        case .settings: return "gear"
        }
    }
}
