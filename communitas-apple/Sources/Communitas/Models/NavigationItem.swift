import Foundation

/// System-level navigation pages shown in the SYSTEM section of the sidebar.
enum SystemPage: String, CaseIterable, Identifiable, Hashable {
    case people = "People"
    case groups = "Groups"
    case network = "Network"
    case liveFeed = "Live Feed"
    case kvStores = "KV Stores"
    case fourWord = "Four-Word Bootstrap"
    case constitution = "Constitution"
    case settings = "Settings"
    case about = "About"

    var id: String { rawValue }

    var systemImage: String {
        switch self {
        case .people: return "person.2"
        case .groups: return "person.3"
        case .network: return "network"
        case .liveFeed: return "antenna.radiowaves.left.and.right"
        case .kvStores: return "tray.full"
        case .fourWord: return "text.word.spacing"
        case .constitution: return "scroll"
        case .settings: return "gear"
        case .about: return "info.circle"
        }
    }
}
