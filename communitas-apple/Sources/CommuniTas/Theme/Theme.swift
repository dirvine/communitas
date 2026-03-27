import SwiftUI

enum DeepSpace {
    // Base
    static let bg = Color(hex: 0x0A0C14)
    static let surface1 = Color(hex: 0x10131E)
    static let surface2 = Color(hex: 0x161A2A)
    static let surface3 = Color(hex: 0x1C2036)

    // Borders
    static let border = Color(hex: 0x252940)
    static let borderLight = Color(hex: 0x353A52)

    // Text
    static let textPrimary = Color(hex: 0xE4E6F0)
    static let textSecondary = Color(hex: 0xB0B3CC)
    static let textMuted = Color(hex: 0x747896)

    // Accent
    static let cyan = Color(hex: 0x00D4FF)
    static let cyanDim = Color(hex: 0x00D4FF).opacity(0.12)

    // Status
    static let green = Color(hex: 0x10B981)
    static let amber = Color(hex: 0xF59E0B)
    static let red = Color(hex: 0xFF4466)
    static let violet = Color(hex: 0x8B5CF6)
    static let lavender = Color(hex: 0xA78BFA)

    // Trust colors
    static func trustColor(_ level: String) -> Color {
        switch level.lowercased() {
        case "trusted": return green
        case "known": return cyan
        case "unknown": return amber
        case "blocked": return red
        default: return textMuted
        }
    }
}

extension Color {
    init(hex: UInt, opacity: Double = 1.0) {
        self.init(
            .sRGB,
            red: Double((hex >> 16) & 0xFF) / 255.0,
            green: Double((hex >> 8) & 0xFF) / 255.0,
            blue: Double(hex & 0xFF) / 255.0,
            opacity: opacity
        )
    }
}
