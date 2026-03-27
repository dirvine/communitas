import Foundation

/// System-level navigation pages shown in the SYSTEM section of the sidebar.
enum SystemPage: String, CaseIterable, Identifiable, Hashable {
    case people = "People"
    case network = "Network"
    case settings = "Settings"
    case about = "About"

    var id: String { rawValue }

    var systemImage: String {
        switch self {
        case .people: return "person.2"
        case .network: return "network"
        case .settings: return "gear"
        case .about: return "info.circle"
        }
    }
}
