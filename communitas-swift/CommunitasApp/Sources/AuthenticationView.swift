import SwiftUI
import CommunitasKit
import LocalAuthentication
import Security
#if os(macOS)
import AppKit
#else
import UIKit
#endif

// MARK: - Keychain Helper for Passkey Storage
struct KeychainHelper {
    static let serviceName = "com.communitas.passkey"

    /// Generate a cryptographically secure random password for vault encryption
    static func generateSecurePassword(length: Int = 32) -> String {
        var bytes = [UInt8](repeating: 0, count: length)
        let result = SecRandomCopyBytes(kSecRandomDefault, bytes.count, &bytes)
        if result == errSecSuccess {
            return Data(bytes).base64EncodedString()
        }
        // Fallback to UUID-based if SecRandom fails
        return UUID().uuidString + UUID().uuidString
    }

    /// Store password in Keychain with biometric protection
    static func storePasskeyPassword(fourWords: String, password: String) -> Bool {
        let account = fourWords

        // Delete any existing entry first
        let deleteQuery: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: serviceName,
            kSecAttrAccount as String: account
        ]
        SecItemDelete(deleteQuery as CFDictionary)

        // Create access control requiring biometric authentication
        // Note: .biometryCurrentSet requires re-enrollment if biometrics change
        // Using .biometryAny for better UX (persists across biometric updates)
        var error: Unmanaged<CFError>?
        guard let accessControl = SecAccessControlCreateWithFlags(
            kCFAllocatorDefault,
            kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
            .biometryAny,
            &error
        ) else {
            print("Failed to create access control: \(error?.takeRetainedValue().localizedDescription ?? "unknown")")
            // Fallback: store without biometric requirement (still secure, device-locked)
            return storePasskeyPasswordWithoutBiometric(fourWords: fourWords, password: password)
        }

        // Add new entry with biometric protection
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: serviceName,
            kSecAttrAccount as String: account,
            kSecValueData as String: password.data(using: .utf8)!,
            kSecAttrAccessControl as String: accessControl
        ]

        let status = SecItemAdd(query as CFDictionary, nil)
        if status == errSecSuccess {
            return true
        }

        // If biometric storage fails, try without biometric constraint
        print("Biometric keychain storage failed with status: \(status), falling back to standard storage")
        return storePasskeyPasswordWithoutBiometric(fourWords: fourWords, password: password)
    }

    /// Fallback storage without biometric constraint (still secure, device-locked)
    private static func storePasskeyPasswordWithoutBiometric(fourWords: String, password: String) -> Bool {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: serviceName,
            kSecAttrAccount as String: fourWords,
            kSecValueData as String: password.data(using: .utf8)!,
            kSecAttrAccessible as String: kSecAttrAccessibleWhenUnlockedThisDeviceOnly
        ]

        let status = SecItemAdd(query as CFDictionary, nil)
        return status == errSecSuccess
    }

    /// Retrieve password from Keychain (requires biometric auth)
    static func retrievePasskeyPassword(fourWords: String, context: LAContext? = nil) -> String? {
        let account = fourWords

        var query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: serviceName,
            kSecAttrAccount as String: account,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne
        ]

        // Use provided LAContext if available (for pre-authenticated access)
        if let context = context {
            query[kSecUseAuthenticationContext as String] = context
        }

        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)

        guard status == errSecSuccess,
              let data = result as? Data,
              let password = String(data: data, encoding: .utf8) else {
            return nil
        }

        return password
    }

    /// Check if passkey exists for a four-word identity
    static func hasPasskey(fourWords: String) -> Bool {
        // Use LAContext with interactionNotAllowed to check without prompting
        let context = LAContext()
        context.interactionNotAllowed = true

        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: serviceName,
            kSecAttrAccount as String: fourWords,
            kSecUseAuthenticationContext as String: context
        ]

        let status = SecItemCopyMatching(query as CFDictionary, nil)
        // errSecSuccess: item exists and accessible
        // errSecInteractionNotAllowed: item exists but needs biometric auth
        return status == errSecSuccess || status == errSecInteractionNotAllowed
    }

    /// Delete passkey from Keychain
    static func deletePasskey(fourWords: String) -> Bool {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: serviceName,
            kSecAttrAccount as String: fourWords
        ]

        let status = SecItemDelete(query as CFDictionary)
        return status == errSecSuccess || status == errSecItemNotFound
    }
}

// MARK: - Bundle Image Helper
#if os(macOS)
func loadBundleImage(_ name: String) -> NSImage? {
    if let url = Bundle.main.url(forResource: name, withExtension: "png") {
        return NSImage(contentsOf: url)
    }
    return nil
}
#else
func loadBundleImage(_ name: String) -> UIImage? {
    if let url = Bundle.main.url(forResource: name, withExtension: "png"),
       let data = try? Data(contentsOf: url) {
        return UIImage(data: data)
    }
    return nil
}
#endif

// MARK: - Brand Colors
extension Color {
    static let communitasCyan = Color(red: 0.29, green: 0.89, blue: 0.89)
    static let communitasDark = Color(red: 0.05, green: 0.10, blue: 0.15)
    static let communitasDarkAlt = Color(red: 0.08, green: 0.15, blue: 0.20)
}

// MARK: - Four Words Generator
/// Simple four-word generator using a sample word list
/// In production, this would use the four-word-networking dictionary
struct FourWordsGenerator {
    /// Sample words (subset of four-word-networking dictionary)
    private static let words = [
        "ocean", "forest", "moon", "star", "river", "mountain", "wind", "cloud",
        "dawn", "dusk", "rain", "snow", "leaf", "bloom", "wave", "stone",
        "eagle", "tiger", "wolf", "bear", "falcon", "raven", "phoenix", "dragon",
        "amber", "coral", "jade", "pearl", "ruby", "silver", "gold", "bronze",
        "swift", "brave", "calm", "bold", "wise", "keen", "true", "free",
        "north", "south", "east", "west", "zenith", "horizon", "apex", "nexus"
    ]

    /// Generate a random four-word identity
    static func generate() -> String {
        var selected: [String] = []
        var available = words
        for _ in 0..<4 {
            let index = Int.random(in: 0..<available.count)
            selected.append(available[index])
            available.remove(at: index)
        }
        return selected.joined(separator: "-")
    }
}

// MARK: - Auth State
enum AuthMode {
    case welcome
    case login
    case createIdentity
    case vaultSelection
    case vaultManagement
}

// MARK: - Auth Method
enum AuthMethod: String, CaseIterable {
    case password = "Password"
    case passkey = "Passkey"

    var icon: String {
        switch self {
        case .password: return "key.fill"
        case .passkey: return "faceid"
        }
    }

    var description: String {
        switch self {
        case .password: return "Create a strong password"
        case .passkey: return "Use Touch ID / Face ID"
        }
    }
}

// MARK: - Password Strength
enum PasswordStrength: String {
    case weak = "Weak"
    case fair = "Fair"
    case good = "Good"
    case strong = "Strong"

    var color: Color {
        switch self {
        case .weak: return .red
        case .fair: return .orange
        case .good: return .yellow
        case .strong: return .green
        }
    }

    static func calculate(_ password: String) -> PasswordStrength {
        var score = 0
        if password.count >= 8 { score += 1 }
        if password.count >= 12 { score += 1 }
        if password.contains(where: { $0.isUppercase }) { score += 1 }
        if password.contains(where: { $0.isNumber }) { score += 1 }
        if password.contains(where: { "!@#$%^&*()_+-=[]{}|;':\",./<>?".contains($0) }) { score += 1 }

        switch score {
        case 0...1: return .weak
        case 2: return .fair
        case 3...4: return .good
        default: return .strong
        }
    }
}

// MARK: - Authentication View
public struct AuthenticationView: View {
    @EnvironmentObject var appState: AppState
    @State private var authMode: AuthMode = .welcome

    public init() {}
    @State private var fourWords: String = ""
    @State private var password: String = ""
    @State private var confirmPassword: String = ""
    @State private var displayName: String = ""
    @State private var errorMessage: String?
    @State private var isLoading: Bool = false
    @State private var availableVaults: [SwiftVaultInfo] = []
    @State private var canUseTouchID: Bool = false
    @State private var vaultToDelete: SwiftVaultInfo?
    @State private var deletePassword: String = ""
    @State private var showDeleteConfirmation: Bool = false
    @State private var selectedAuthMethod: AuthMethod = .password
    @State private var passkeySetupComplete: Bool = false
    @State private var vaultToEnableTouchID: SwiftVaultInfo?
    @State private var enableTouchIDPassword: String = ""
    @State private var showEnableTouchIDSheet: Bool = false

    public var body: some View {
        ZStack {
            // Background - solid dark color
            Color.communitasDark
                .ignoresSafeArea()

            // Content - centered in window
            VStack {
                Spacer()

                VStack(spacing: 0) {
                    switch authMode {
                    case .welcome:
                        welcomeView
                    case .login:
                        loginView
                    case .createIdentity:
                        createIdentityView
                    case .vaultSelection:
                        vaultSelectionView
                    case .vaultManagement:
                        vaultManagementView
                    }
                }
                .frame(maxWidth: 450)
                .padding(.horizontal, 40)

                Spacer()
            }
        }
        .onAppear {
            checkTouchIDAvailability()
            loadAvailableVaults()

            // Always show identity picker if vaults exist (no auto-login)
            if !availableVaults.isEmpty {
                print("[Communitas] Vaults exist - showing identity picker")
                authMode = .vaultSelection
            }
        }
    }

    // MARK: - Welcome View
    private var welcomeView: some View {
        VStack(spacing: 32) {
            // Logo - Use SF Symbol for reliability
            Image(systemName: "network")
                .font(.system(size: 80))
                .foregroundColor(.communitasCyan)
                .frame(width: 120, height: 120)
                .accessibilityIdentifier("appLogo")

            // Title
            VStack(spacing: 8) {
                Text("COMMUNITAS")
                    .font(.system(size: 36, weight: .bold, design: .rounded))
                    .foregroundColor(.communitasCyan)

                Text("Decentralized. Secure. Collaborative.")
                    .font(.subheadline)
                    .foregroundColor(.communitasCyan.opacity(0.7))
            }

            // Action buttons
            VStack(spacing: 16) {
                // Login button
                Button(action: {
                    withAnimation(.easeInOut(duration: 0.3)) {
                        if availableVaults.isEmpty {
                            authMode = .login
                        } else {
                            authMode = .vaultSelection
                        }
                    }
                }) {
                    HStack {
                        Image(systemName: "person.circle.fill")
                        Text(availableVaults.isEmpty ? "Sign In" : "Continue")
                    }
                    .font(.headline)
                    .foregroundColor(.communitasDark)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 16)
                    .background(Color.communitasCyan)
                    .cornerRadius(12)
                }
                .buttonStyle(.plain)
                .accessibilityIdentifier("signInButton")

                // Create identity button
                Button(action: {
                    withAnimation(.easeInOut(duration: 0.3)) {
                        authMode = .createIdentity
                    }
                }) {
                    HStack {
                        Image(systemName: "plus.circle.fill")
                        Text("Create New Identity")
                    }
                    .font(.headline)
                    .foregroundColor(.communitasCyan)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 16)
                    .background(Color.communitasCyan.opacity(0.15))
                    .overlay(
                        RoundedRectangle(cornerRadius: 12)
                            .stroke(Color.communitasCyan.opacity(0.5), lineWidth: 1)
                    )
                    .cornerRadius(12)
                }
                .buttonStyle(.plain)
                .accessibilityIdentifier("createIdentityButton")

                // Touch ID (if available)
                if canUseTouchID && !availableVaults.isEmpty {
                    Button(action: authenticateWithTouchID) {
                        HStack {
                            Image(systemName: "touchid")
                            Text("Use Touch ID")
                        }
                        .font(.headline)
                        .foregroundColor(.communitasCyan.opacity(0.8))
                    }
                    .buttonStyle(.plain)
                    .padding(.top, 8)
                    .accessibilityIdentifier("touchIDButton")
                }
            }

            // Version info
            Text("Version 1.0.0 • Post-Quantum Secure")
                .font(.caption)
                .foregroundColor(.gray)
                .padding(.top, 40)
        }
    }

    // MARK: - Login View
    private var loginView: some View {
        VStack(spacing: 24) {
            // Back button
            HStack {
                Button(action: {
                    // Reload vaults and go back to vault selection if vaults exist
                    loadAvailableVaults()
                    withAnimation {
                        authMode = availableVaults.isEmpty ? .welcome : .vaultSelection
                    }
                }) {
                    HStack(spacing: 4) {
                        Image(systemName: "chevron.left")
                        Text("Back")
                    }
                    .foregroundColor(.communitasCyan)
                }
                .buttonStyle(.plain)
                Spacer()
            }

            // Header
            VStack(spacing: 8) {
                Image(systemName: "person.badge.key.fill")
                    .font(.system(size: 48))
                    .foregroundColor(.communitasCyan)

                Text("Sign In")
                    .font(.title)
                    .fontWeight(.bold)
                    .foregroundColor(.white)

                Text("Enter your four-word identity")
                    .font(.subheadline)
                    .foregroundColor(.gray)
            }
            .padding(.bottom, 16)

            // Four words input
            VStack(alignment: .leading, spacing: 8) {
                Text("Four-Word Identity")
                    .font(.caption)
                    .foregroundColor(.gray)

                TextField("ocean-forest-moon-star", text: $fourWords)
                    .textFieldStyle(.plain)
                    .font(.system(.body, design: .monospaced))
                    .padding()
                    .background(Color.white.opacity(0.1))
                    .cornerRadius(8)
                    .foregroundColor(.white)
                    .autocorrectionDisabled()
                    .accessibilityIdentifier("fourWordsField")
            }

            // Password input
            VStack(alignment: .leading, spacing: 8) {
                Text("Password")
                    .font(.caption)
                    .foregroundColor(.gray)

                SecureField("Enter password", text: $password)
                    .textFieldStyle(.plain)
                    .padding()
                    .background(Color.white.opacity(0.1))
                    .cornerRadius(8)
                    .foregroundColor(.white)
                    .accessibilityIdentifier("passwordField")
            }

            // Error message
            if let error = errorMessage {
                Text(error)
                    .font(.caption)
                    .foregroundColor(.red)
                    .padding(.vertical, 4)
                    .accessibilityIdentifier("errorMessage")
            }

            // Login button
            Button(action: performLogin) {
                if isLoading {
                    ProgressView()
                        .progressViewStyle(CircularProgressViewStyle(tint: .communitasDark))
                } else {
                    Text("Sign In")
                        .fontWeight(.semibold)
                }
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 16)
            .background(fourWords.isEmpty || password.isEmpty ? Color.gray : Color.communitasCyan)
            .foregroundColor(.communitasDark)
            .cornerRadius(12)
            .disabled(fourWords.isEmpty || password.isEmpty || isLoading)
            .buttonStyle(.plain)
            .accessibilityIdentifier("loginButton")
        }
    }

    // MARK: - Create Identity View
    private var createIdentityView: some View {
        ScrollView {
            VStack(spacing: 24) {
                // Back button
                HStack {
                    Button(action: {
                        withAnimation { authMode = .welcome }
                    }) {
                        HStack(spacing: 4) {
                            Image(systemName: "chevron.left")
                            Text("Back")
                        }
                        .foregroundColor(.communitasCyan)
                    }
                    .buttonStyle(.plain)
                    Spacer()
                }

                // Header
                VStack(spacing: 8) {
                    Image(systemName: "person.badge.plus")
                        .font(.system(size: 48))
                        .foregroundColor(.communitasCyan)

                    Text("Create Identity")
                        .font(.title)
                        .fontWeight(.bold)
                        .foregroundColor(.white)

                    Text("Your unique four-word address will be generated")
                        .font(.subheadline)
                        .foregroundColor(.gray)
                        .multilineTextAlignment(.center)
                }
                .padding(.bottom, 8)

                // Display name input
                VStack(alignment: .leading, spacing: 8) {
                    Text("Display Name")
                        .font(.caption)
                        .foregroundColor(.gray)

                    TextField("Your Name", text: $displayName)
                        .textFieldStyle(.plain)
                        .padding()
                        .background(Color.white.opacity(0.1))
                        .cornerRadius(8)
                        .foregroundColor(.white)
                        .accessibilityIdentifier("displayNameField")
                }

                // Authentication Method Selector
                VStack(alignment: .leading, spacing: 12) {
                    Text("Authentication Method")
                        .font(.caption)
                        .foregroundColor(.gray)

                    HStack(spacing: 12) {
                        ForEach(AuthMethod.allCases, id: \.self) { method in
                            Button(action: {
                                withAnimation(.easeInOut(duration: 0.2)) {
                                    selectedAuthMethod = method
                                    // Reset passkey state when switching
                                    if method == .passkey {
                                        passkeySetupComplete = false
                                    }
                                }
                            }) {
                                VStack(spacing: 8) {
                                    Image(systemName: method.icon)
                                        .font(.title2)
                                    Text(method.rawValue)
                                        .font(.caption)
                                        .fontWeight(.medium)
                                }
                                .frame(maxWidth: .infinity)
                                .padding(.vertical, 16)
                                .background(
                                    selectedAuthMethod == method
                                        ? Color.communitasCyan.opacity(0.2)
                                        : Color.white.opacity(0.05)
                                )
                                .foregroundColor(
                                    selectedAuthMethod == method
                                        ? Color.communitasCyan
                                        : Color.gray
                                )
                                .overlay(
                                    RoundedRectangle(cornerRadius: 10)
                                        .stroke(
                                            selectedAuthMethod == method
                                                ? Color.communitasCyan
                                                : Color.white.opacity(0.1),
                                            lineWidth: selectedAuthMethod == method ? 2 : 1
                                        )
                                )
                                .cornerRadius(10)
                            }
                            .buttonStyle(.plain)
                            .disabled(method == .passkey && !canUseTouchID)
                            .opacity(method == .passkey && !canUseTouchID ? 0.5 : 1)
                        }
                    }

                    // Passkey availability note
                    if !canUseTouchID {
                        HStack(spacing: 4) {
                            Image(systemName: "info.circle")
                            Text("Passkey requires Touch ID or Face ID hardware")
                        }
                        .font(.caption2)
                        .foregroundColor(.orange)
                    }
                }

                // Password fields (shown only when password method selected)
                if selectedAuthMethod == .password {
                    VStack(spacing: 16) {
                        // Password input
                        VStack(alignment: .leading, spacing: 8) {
                            Text("Password")
                                .font(.caption)
                                .foregroundColor(.gray)

                            SecureField("Create a strong password", text: $password)
                                .textFieldStyle(.plain)
                                .padding()
                                .background(Color.white.opacity(0.1))
                                .cornerRadius(8)
                                .foregroundColor(.white)
                                .accessibilityIdentifier("newPasswordField")

                            // Password strength indicator
                            if !password.isEmpty {
                                let strength = PasswordStrength.calculate(password)
                                HStack {
                                    ForEach(0..<4, id: \.self) { i in
                                        Rectangle()
                                            .fill(i < strengthLevel(strength) ? strength.color : Color.gray.opacity(0.3))
                                            .frame(height: 4)
                                            .cornerRadius(2)
                                    }
                                    Text(strength.rawValue)
                                        .font(.caption)
                                        .foregroundColor(strength.color)
                                }
                            }
                        }

                        // Confirm password
                        VStack(alignment: .leading, spacing: 8) {
                            Text("Confirm Password")
                                .font(.caption)
                                .foregroundColor(.gray)

                            SecureField("Confirm password", text: $confirmPassword)
                                .textFieldStyle(.plain)
                                .padding()
                                .background(Color.white.opacity(0.1))
                                .cornerRadius(8)
                                .foregroundColor(.white)
                                .accessibilityIdentifier("confirmPasswordField")

                            if !confirmPassword.isEmpty && password != confirmPassword {
                                Text("Passwords don't match")
                                    .font(.caption)
                                    .foregroundColor(.red)
                            }
                        }
                    }
                }

                // Passkey setup (shown only when passkey method selected)
                if selectedAuthMethod == .passkey {
                    VStack(spacing: 16) {
                        if passkeySetupComplete {
                            // Passkey configured successfully
                            HStack(spacing: 12) {
                                Image(systemName: "checkmark.circle.fill")
                                    .font(.title)
                                    .foregroundColor(.green)
                                VStack(alignment: .leading, spacing: 4) {
                                    Text("Passkey Ready")
                                        .font(.headline)
                                        .foregroundColor(.white)
                                    Text("Touch ID / Face ID configured")
                                        .font(.caption)
                                        .foregroundColor(.gray)
                                }
                                Spacer()
                            }
                            .padding()
                            .background(Color.green.opacity(0.1))
                            .cornerRadius(10)
                        } else {
                            // Setup passkey button
                            VStack(spacing: 12) {
                                Image(systemName: "faceid")
                                    .font(.system(size: 60))
                                    .foregroundColor(.communitasCyan.opacity(0.7))

                                Text("Use biometric authentication to secure your identity")
                                    .font(.subheadline)
                                    .foregroundColor(.gray)
                                    .multilineTextAlignment(.center)

                                Button(action: setupPasskey) {
                                    HStack {
                                        Image(systemName: "touchid")
                                        Text("Set Up Passkey")
                                    }
                                    .font(.headline)
                                    .foregroundColor(.communitasDark)
                                    .frame(maxWidth: .infinity)
                                    .padding(.vertical, 14)
                                    .background(Color.communitasCyan)
                                    .cornerRadius(10)
                                }
                                .buttonStyle(.plain)
                            }
                            .padding()
                            .background(Color.white.opacity(0.05))
                            .cornerRadius(12)
                        }

                        // Info about passkey
                        HStack(alignment: .top, spacing: 8) {
                            Image(systemName: "info.circle")
                                .foregroundColor(.communitasCyan)
                            Text("Your passkey is securely stored in the system keychain and never leaves your device.")
                                .font(.caption)
                                .foregroundColor(.gray)
                        }
                    }
                }

                // Error message
                if let error = errorMessage {
                    Text(error)
                        .font(.caption)
                        .foregroundColor(.red)
                        .padding(.vertical, 4)
                }

                // Create button
                Button(action: performCreateIdentity) {
                    if isLoading {
                        ProgressView()
                            .progressViewStyle(CircularProgressViewStyle(tint: .communitasDark))
                    } else {
                        Text("Create Identity")
                            .fontWeight(.semibold)
                    }
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, 16)
                .background(canCreate ? Color.communitasCyan : Color.gray)
                .foregroundColor(.communitasDark)
                .cornerRadius(12)
                .disabled(!canCreate || isLoading)
                .buttonStyle(.plain)
                .accessibilityIdentifier("createButton")
            }
            .padding(.bottom, 20)
        }
    }

    // MARK: - Vault Selection View
    private var vaultSelectionView: some View {
        VStack(spacing: 24) {
            // Back button and manage button
            HStack {
                Button(action: {
                    withAnimation { authMode = .welcome }
                }) {
                    HStack(spacing: 4) {
                        Image(systemName: "chevron.left")
                        Text("Back")
                    }
                    .foregroundColor(.communitasCyan)
                }
                .buttonStyle(.plain)
                Spacer()

                Button(action: {
                    withAnimation { authMode = .vaultManagement }
                }) {
                    HStack(spacing: 4) {
                        Image(systemName: "gear")
                        Text("Manage")
                    }
                    .foregroundColor(.communitasCyan.opacity(0.8))
                    .font(.caption)
                }
                .buttonStyle(.plain)
            }

            // Header
            VStack(spacing: 8) {
                Image(systemName: "person.2.circle.fill")
                    .font(.system(size: 48))
                    .foregroundColor(.communitasCyan)

                Text("Choose Identity")
                    .font(.title)
                    .fontWeight(.bold)
                    .foregroundColor(.white)

                Text("Select an identity to continue")
                    .font(.subheadline)
                    .foregroundColor(.gray)
            }
            .padding(.bottom, 16)

            // Vault list - Display name is primary, four-word ID is secondary
            ScrollView {
                VStack(spacing: 12) {
                    ForEach(availableVaults, id: \.fourWords) { vault in
                        VaultSelectionRow(
                            vault: vault,
                            hasPasskey: KeychainHelper.hasPasskey(fourWords: vault.fourWords),
                            onTap: {
                                // If passkey available, try biometric auth directly
                                if KeychainHelper.hasPasskey(fourWords: vault.fourWords) {
                                    performPasskeyLogin(for: vault.fourWords)
                                } else {
                                    // Otherwise go to password login
                                    fourWords = vault.fourWords
                                    withAnimation { authMode = .login }
                                }
                            },
                            onPasswordLogin: {
                                // Force password login (for when biometric fails)
                                fourWords = vault.fourWords
                                withAnimation { authMode = .login }
                            }
                        )
                    }
                }
            }
            .frame(maxHeight: 300)

            // Divider
            HStack {
                Rectangle()
                    .fill(Color.gray.opacity(0.3))
                    .frame(height: 1)
                Text("or")
                    .font(.caption)
                    .foregroundColor(.gray)
                Rectangle()
                    .fill(Color.gray.opacity(0.3))
                    .frame(height: 1)
            }
            .padding(.vertical, 8)

            // Sign in with four-word identity (for new device / recovery)
            Button(action: {
                fourWords = ""
                withAnimation { authMode = .login }
            }) {
                HStack {
                    Image(systemName: "key.fill")
                    Text("Sign in with Four-Word Identity")
                }
                .font(.subheadline)
                .foregroundColor(.communitasCyan.opacity(0.8))
            }
            .buttonStyle(.plain)

            Text("Use this on a new device or for account recovery")
                .font(.caption2)
                .foregroundColor(.gray.opacity(0.7))
        }
    }

    // MARK: - Vault Management View
    private var vaultManagementView: some View {
        VStack(spacing: 24) {
            // Back button
            HStack {
                Button(action: {
                    withAnimation { authMode = .vaultSelection }
                }) {
                    HStack(spacing: 4) {
                        Image(systemName: "chevron.left")
                        Text("Back")
                    }
                    .foregroundColor(.communitasCyan)
                }
                .buttonStyle(.plain)
                Spacer()
            }

            // Header
            VStack(spacing: 8) {
                Image(systemName: "gearshape.2.fill")
                    .font(.system(size: 48))
                    .foregroundColor(.communitasCyan)

                Text("Manage Identities")
                    .font(.title)
                    .fontWeight(.bold)
                    .foregroundColor(.white)

                Text("View, select, or remove stored identities")
                    .font(.subheadline)
                    .foregroundColor(.gray)
                    .multilineTextAlignment(.center)
            }
            .padding(.bottom, 16)

            // Error message
            if let error = errorMessage {
                Text(error)
                    .font(.caption)
                    .foregroundColor(.red)
                    .padding(.vertical, 4)
            }

            // Vault list with delete option
            ScrollView {
                VStack(spacing: 12) {
                    ForEach(availableVaults, id: \.fourWords) { vault in
                        VaultManagementRow(
                            vault: vault,
                            hasPasskey: KeychainHelper.hasPasskey(fourWords: vault.fourWords),
                            canUseTouchID: canUseTouchID,
                            onSelect: {
                                fourWords = vault.fourWords
                                withAnimation { authMode = .login }
                            },
                            onEnableTouchID: {
                                vaultToEnableTouchID = vault
                                showEnableTouchIDSheet = true
                            },
                            onDisableTouchID: {
                                _ = KeychainHelper.deletePasskey(fourWords: vault.fourWords)
                                // Trigger UI refresh
                                loadAvailableVaults()
                            },
                            onDelete: {
                                vaultToDelete = vault
                                showDeleteConfirmation = true
                            }
                        )
                    }
                }
            }
            .frame(maxHeight: 350)

            if availableVaults.isEmpty {
                VStack(spacing: 12) {
                    Image(systemName: "tray")
                        .font(.system(size: 40))
                        .foregroundColor(.gray)
                    Text("No identities stored")
                        .foregroundColor(.gray)
                }
                .padding()
            }

            // Create new identity button
            Button(action: {
                withAnimation { authMode = .createIdentity }
            }) {
                HStack {
                    Image(systemName: "plus.circle.fill")
                    Text("Create New Identity")
                }
                .font(.headline)
                .foregroundColor(.communitasCyan)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 14)
                .background(Color.communitasCyan.opacity(0.15))
                .overlay(
                    RoundedRectangle(cornerRadius: 12)
                        .stroke(Color.communitasCyan.opacity(0.5), lineWidth: 1)
                )
                .cornerRadius(12)
            }
            .buttonStyle(.plain)
        }
        .sheet(isPresented: $showDeleteConfirmation) {
            deleteConfirmationSheet
        }
        .sheet(isPresented: $showEnableTouchIDSheet) {
            enableTouchIDSheet
        }
    }

    // MARK: - Enable Touch ID Sheet
    private var enableTouchIDSheet: some View {
        VStack(spacing: 24) {
            // Touch ID icon
            Image(systemName: "touchid")
                .font(.system(size: 48))
                .foregroundColor(.communitasCyan)

            Text("Enable Touch ID")
                .font(.title2)
                .fontWeight(.bold)
                .foregroundColor(.white)

            if let vault = vaultToEnableTouchID {
                VStack(spacing: 8) {
                    Text(vault.displayName.isEmpty ? "Identity" : vault.displayName)
                        .font(.headline)
                        .foregroundColor(.white)
                    Text(vault.fourWords)
                        .font(.system(.body, design: .monospaced))
                        .foregroundColor(.communitasCyan)
                }
                .padding()
                .background(Color.white.opacity(0.05))
                .cornerRadius(8)
            }

            Text("Enter your password to enable Touch ID login for this identity.")
                .font(.caption)
                .foregroundColor(.gray)
                .multilineTextAlignment(.center)
                .padding(.horizontal)

            // Password field
            VStack(alignment: .leading, spacing: 8) {
                Text("Password")
                    .font(.caption)
                    .foregroundColor(.gray)

                SecureField("Enter your password", text: $enableTouchIDPassword)
                    .textFieldStyle(.plain)
                    .padding()
                    .background(Color.white.opacity(0.1))
                    .cornerRadius(8)
                    .foregroundColor(.white)
            }

            // Error message
            if let error = errorMessage {
                Text(error)
                    .font(.caption)
                    .foregroundColor(.red)
            }

            // Buttons
            HStack(spacing: 16) {
                Button(action: {
                    enableTouchIDPassword = ""
                    errorMessage = nil
                    showEnableTouchIDSheet = false
                }) {
                    Text("Cancel")
                        .fontWeight(.medium)
                        .foregroundColor(.gray)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 14)
                        .background(Color.white.opacity(0.1))
                        .cornerRadius(10)
                }
                .buttonStyle(.plain)

                Button(action: performEnableTouchID) {
                    if isLoading {
                        ProgressView()
                            .progressViewStyle(CircularProgressViewStyle(tint: .communitasDark))
                    } else {
                        HStack {
                            Image(systemName: "touchid")
                            Text("Enable")
                        }
                        .fontWeight(.semibold)
                    }
                }
                .foregroundColor(.communitasDark)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 14)
                .background(enableTouchIDPassword.isEmpty ? Color.communitasCyan.opacity(0.5) : Color.communitasCyan)
                .cornerRadius(10)
                .disabled(enableTouchIDPassword.isEmpty || isLoading)
                .buttonStyle(.plain)
            }
        }
        .padding(32)
        .background(Color.communitasDark)
        .frame(width: 400)
    }

    // MARK: - Delete Confirmation Sheet
    private var deleteConfirmationSheet: some View {
        VStack(spacing: 24) {
            // Warning icon
            Image(systemName: "exclamationmark.triangle.fill")
                .font(.system(size: 48))
                .foregroundColor(.orange)

            Text("Delete Identity?")
                .font(.title2)
                .fontWeight(.bold)
                .foregroundColor(.white)

            if let vault = vaultToDelete {
                VStack(spacing: 8) {
                    Text(vault.displayName.isEmpty ? "Identity" : vault.displayName)
                        .font(.headline)
                        .foregroundColor(.white)
                    Text(vault.fourWords)
                        .font(.system(.body, design: .monospaced))
                        .foregroundColor(.communitasCyan)
                }
                .padding()
                .background(Color.white.opacity(0.05))
                .cornerRadius(8)
            }

            Text("This action cannot be undone. All data associated with this identity will be permanently deleted.")
                .font(.caption)
                .foregroundColor(.gray)
                .multilineTextAlignment(.center)
                .padding(.horizontal)

            // Password confirmation
            VStack(alignment: .leading, spacing: 8) {
                Text("Enter password to confirm")
                    .font(.caption)
                    .foregroundColor(.gray)

                SecureField("Password", text: $deletePassword)
                    .textFieldStyle(.plain)
                    .padding()
                    .background(Color.white.opacity(0.1))
                    .cornerRadius(8)
                    .foregroundColor(.white)
            }

            // Error message
            if let error = errorMessage {
                Text(error)
                    .font(.caption)
                    .foregroundColor(.red)
            }

            // Buttons
            HStack(spacing: 16) {
                Button(action: {
                    deletePassword = ""
                    errorMessage = nil
                    showDeleteConfirmation = false
                }) {
                    Text("Cancel")
                        .fontWeight(.medium)
                        .foregroundColor(.gray)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 14)
                        .background(Color.white.opacity(0.1))
                        .cornerRadius(10)
                }
                .buttonStyle(.plain)

                Button(action: performDeleteVault) {
                    if isLoading {
                        ProgressView()
                            .progressViewStyle(CircularProgressViewStyle(tint: .white))
                    } else {
                        Text("Delete")
                            .fontWeight(.semibold)
                    }
                }
                .foregroundColor(.white)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 14)
                .background(deletePassword.isEmpty ? Color.red.opacity(0.5) : Color.red)
                .cornerRadius(10)
                .disabled(deletePassword.isEmpty || isLoading)
                .buttonStyle(.plain)
            }
        }
        .padding(32)
        .background(Color.communitasDark)
        .frame(width: 400)
    }

    // MARK: - Helper Methods

    private var canCreate: Bool {
        guard !displayName.isEmpty else { return false }

        switch selectedAuthMethod {
        case .password:
            return password.count >= 8 && password == confirmPassword
        case .passkey:
            return passkeySetupComplete
        }
    }

    private func setupPasskey() {
        let context = LAContext()
        context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, localizedReason: "Set up passkey for Communitas") { success, error in
            DispatchQueue.main.async {
                if success {
                    passkeySetupComplete = true
                    errorMessage = nil
                } else {
                    errorMessage = error?.localizedDescription ?? "Passkey setup failed"
                }
            }
        }
    }

    private func strengthLevel(_ strength: PasswordStrength) -> Int {
        switch strength {
        case .weak: return 1
        case .fair: return 2
        case .good: return 3
        case .strong: return 4
        }
    }

    private func checkTouchIDAvailability() {
        let context = LAContext()
        var error: NSError?
        canUseTouchID = context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error)
    }

    private func loadAvailableVaults() {
        // Load vaults using the discovery client (works before login)
        availableVaults = appState.listAllVaults()
        print("[Communitas] AuthenticationView loaded \(availableVaults.count) vault(s)")
        for vault in availableVaults {
            print("[Communitas] Vault: fourWords='\(vault.fourWords)' displayName='\(vault.displayName)'")
        }
    }

    private func formatDate(_ timestamp: UInt64) -> String {
        let date = Date(timeIntervalSince1970: Double(timestamp))
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .abbreviated
        return formatter.localizedString(for: date, relativeTo: Date())
    }

    private func formatBytes(_ bytes: UInt64) -> String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .file
        return formatter.string(fromByteCount: Int64(bytes))
    }

    private func performEnableTouchID() {
        guard let vault = vaultToEnableTouchID else { return }

        isLoading = true
        errorMessage = nil

        DispatchQueue.global(qos: .userInitiated).async {
            // Ensure client is initialized
            if appState.client == nil {
                DispatchQueue.main.sync {
                    appState.initializeClientWithCredentials(fourWords: vault.fourWords, displayName: vault.displayName)
                }
            }

            do {
                guard let client = appState.client else {
                    DispatchQueue.main.async {
                        isLoading = false
                        errorMessage = "Failed to initialize client"
                    }
                    return
                }

                // Verify the password by attempting a login
                _ = try client.authLogin(
                    fourWords: vault.fourWords,
                    password: enableTouchIDPassword
                )

                // Password verified! Store it in the keychain with biometric protection
                let stored = KeychainHelper.storePasskeyPassword(
                    fourWords: vault.fourWords,
                    password: enableTouchIDPassword
                )

                DispatchQueue.main.async {
                    isLoading = false

                    if stored {
                        // Success - close sheet and refresh
                        enableTouchIDPassword = ""
                        vaultToEnableTouchID = nil
                        showEnableTouchIDSheet = false
                        loadAvailableVaults()
                    } else {
                        errorMessage = "Failed to store passkey in Keychain"
                    }
                }
            } catch {
                DispatchQueue.main.async {
                    isLoading = false
                    errorMessage = "Invalid password"
                }
            }
        }
    }

    private func performDeleteVault() {
        guard let vault = vaultToDelete else { return }

        isLoading = true
        errorMessage = nil

        DispatchQueue.global(qos: .userInitiated).async {
            do {
                guard let client = appState.client else {
                    DispatchQueue.main.async {
                        isLoading = false
                        errorMessage = "Client not initialized"
                    }
                    return
                }

                try client.authDeleteVault(fourWords: vault.fourWords, password: deletePassword)

                DispatchQueue.main.async {
                    isLoading = false
                    deletePassword = ""
                    vaultToDelete = nil
                    showDeleteConfirmation = false
                    loadAvailableVaults()

                    // If no more vaults, go back to welcome
                    if availableVaults.isEmpty {
                        withAnimation { authMode = .welcome }
                    }
                }
            } catch {
                DispatchQueue.main.async {
                    isLoading = false
                    errorMessage = "Invalid password or deletion failed"
                }
            }
        }
    }

    private func performLogin() {
        isLoading = true
        errorMessage = nil

        DispatchQueue.global(qos: .userInitiated).async {
            // Ensure client is initialized
            if appState.client == nil {
                DispatchQueue.main.sync {
                    appState.initializeClientWithCredentials(fourWords: fourWords, displayName: "Logging in")
                }
            }

            do {
                guard let client = appState.client else {
                    DispatchQueue.main.async {
                        isLoading = false
                        errorMessage = "Failed to initialize client"
                    }
                    return
                }

                let session = try client.authLogin(
                    fourWords: fourWords,
                    password: password
                )

                DispatchQueue.main.async {
                    isLoading = false
                    appState.fourWords = session.fourWords
                    appState.displayName = session.displayName
                    appState.isAuthenticated = true
                    appState.isInitialized = true
                }
            } catch {
                DispatchQueue.main.async {
                    isLoading = false
                    errorMessage = "Invalid credentials or identity not found"
                }
            }
        }
    }

    private func performCreateIdentity() {
        isLoading = true
        errorMessage = nil

        DispatchQueue.global(qos: .userInitiated).async {
            // Generate random four words using Swift-side generator
            let generatedFourWords = FourWordsGenerator.generate()

            // Determine the password to use based on auth method
            let vaultPassword: String
            switch selectedAuthMethod {
            case .password:
                vaultPassword = password
            case .passkey:
                // Generate a secure random password for passkey-based auth
                vaultPassword = KeychainHelper.generateSecurePassword()
            }

            // Ensure client is initialized with the new four words
            if appState.client == nil {
                DispatchQueue.main.sync {
                    appState.initializeClientWithCredentials(fourWords: generatedFourWords, displayName: displayName)
                }
            }

            do {
                guard let client = appState.client else {
                    DispatchQueue.main.async {
                        isLoading = false
                        errorMessage = "Failed to initialize client"
                    }
                    return
                }

                // Create the vault - returns the four-word identity string
                let resultFourWords = try client.authCreateVault(
                    fourWords: generatedFourWords,
                    password: vaultPassword,
                    displayName: displayName
                )

                // If using passkey, store the password in Keychain with biometric protection
                if selectedAuthMethod == .passkey {
                    let stored = KeychainHelper.storePasskeyPassword(fourWords: resultFourWords, password: vaultPassword)
                    if !stored {
                        // Vault was created but Keychain storage failed - this is a critical error
                        // We should delete the vault or warn the user
                        DispatchQueue.main.async {
                            isLoading = false
                            errorMessage = "Identity created but passkey storage failed. Please delete and recreate with password."
                        }
                        return
                    }
                }

                DispatchQueue.main.async {
                    isLoading = false
                    appState.fourWords = resultFourWords
                    appState.displayName = displayName
                    appState.isAuthenticated = true
                    appState.isInitialized = true
                }
            } catch {
                DispatchQueue.main.async {
                    isLoading = false
                    errorMessage = "Failed to create identity: \(error.localizedDescription)"
                }
            }
        }
    }

    private func authenticateWithTouchID() {
        let context = LAContext()
        context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, localizedReason: "Sign in to Communitas") { success, authError in
            if success {
                // Try to find a vault with a stored passkey
                for vault in availableVaults {
                    if KeychainHelper.hasPasskey(fourWords: vault.fourWords) {
                        // Retrieve the password using the authenticated context
                        if let storedPassword = KeychainHelper.retrievePasskeyPassword(fourWords: vault.fourWords, context: context) {
                            // Perform login with the retrieved password
                            DispatchQueue.main.async {
                                fourWords = vault.fourWords
                                password = storedPassword
                                performLogin()
                            }
                            return
                        }
                    }
                }
                // No passkey found, go to login screen
                DispatchQueue.main.async {
                    if let firstVault = availableVaults.first {
                        fourWords = firstVault.fourWords
                        authMode = .login
                    }
                }
            } else {
                DispatchQueue.main.async {
                    errorMessage = authError?.localizedDescription ?? "Authentication failed"
                }
            }
        }
    }

    /// Login using passkey (biometric authentication to retrieve stored password)
    private func performPasskeyLogin(for vaultFourWords: String) {
        isLoading = true
        errorMessage = nil

        let context = LAContext()
        context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, localizedReason: "Sign in to Communitas") { success, error in
            if success {
                // Retrieve the password from Keychain
                if let storedPassword = KeychainHelper.retrievePasskeyPassword(fourWords: vaultFourWords, context: context) {
                    DispatchQueue.main.async {
                        fourWords = vaultFourWords
                        password = storedPassword
                        performLogin()
                    }
                } else {
                    DispatchQueue.main.async {
                        isLoading = false
                        errorMessage = "Passkey not found. Please use password login."
                    }
                }
            } else {
                DispatchQueue.main.async {
                    isLoading = false
                    errorMessage = error?.localizedDescription ?? "Authentication failed"
                }
            }
        }
    }
}

// MARK: - Vault Management Row
struct VaultManagementRow: View {
    let vault: SwiftVaultInfo
    let hasPasskey: Bool
    let canUseTouchID: Bool
    let onSelect: () -> Void
    let onEnableTouchID: () -> Void
    let onDisableTouchID: () -> Void
    let onDelete: () -> Void

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                // Select button (main tap area)
                Button(action: onSelect) {
                    HStack {
                        // Avatar with Touch ID badge
                        ZStack(alignment: .bottomTrailing) {
                            Image(systemName: "person.circle.fill")
                                .font(.title2)
                                .foregroundColor(.communitasCyan)

                            if hasPasskey {
                                Image(systemName: "touchid")
                                    .font(.system(size: 10))
                                    .foregroundColor(.green)
                                    .background(
                                        Circle()
                                            .fill(Color.communitasDark)
                                            .frame(width: 16, height: 16)
                                    )
                            }
                        }

                        VStack(alignment: .leading, spacing: 4) {
                            Text(vault.displayName.isEmpty ? "Identity" : vault.displayName)
                                .font(.headline)
                                .foregroundColor(.white)
                            Text(vault.fourWords)
                                .font(.system(.caption, design: .monospaced))
                                .foregroundColor(.gray)
                        }

                        Spacer()

                        VStack(alignment: .trailing, spacing: 2) {
                            Text(formatDate(vault.lastAccessed))
                                .font(.caption2)
                                .foregroundColor(.gray)
                            Text(formatBytes(vault.sizeBytes))
                                .font(.caption2)
                                .foregroundColor(.gray.opacity(0.7))
                        }
                    }
                }
                .buttonStyle(.plain)

                // Delete button
                Button(action: onDelete) {
                    Image(systemName: "trash")
                        .foregroundColor(.red.opacity(0.8))
                        .padding(8)
                }
                .buttonStyle(.plain)
            }
            .padding()

            // Touch ID toggle row
            if canUseTouchID {
                Divider()
                    .background(Color.gray.opacity(0.3))

                HStack {
                    Image(systemName: "touchid")
                        .foregroundColor(hasPasskey ? .green : .gray)
                    Text("Touch ID")
                        .font(.subheadline)
                        .foregroundColor(.white)

                    Spacer()

                    if hasPasskey {
                        Button(action: onDisableTouchID) {
                            Text("Disable")
                                .font(.caption)
                                .foregroundColor(.red.opacity(0.8))
                                .padding(.horizontal, 12)
                                .padding(.vertical, 6)
                                .background(Color.red.opacity(0.15))
                                .cornerRadius(6)
                        }
                        .buttonStyle(.plain)
                    } else {
                        Button(action: onEnableTouchID) {
                            Text("Enable")
                                .font(.caption)
                                .foregroundColor(.communitasCyan)
                                .padding(.horizontal, 12)
                                .padding(.vertical, 6)
                                .background(Color.communitasCyan.opacity(0.15))
                                .cornerRadius(6)
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(.horizontal)
                .padding(.vertical, 10)
            }
        }
        .background(Color.white.opacity(0.05))
        .overlay(
            RoundedRectangle(cornerRadius: 12)
                .stroke(hasPasskey ? Color.green.opacity(0.3) : Color.clear, lineWidth: 1)
        )
        .cornerRadius(12)
    }

    private func formatDate(_ timestamp: UInt64) -> String {
        let date = Date(timeIntervalSince1970: Double(timestamp))
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .abbreviated
        return formatter.localizedString(for: date, relativeTo: Date())
    }

    private func formatBytes(_ bytes: UInt64) -> String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .file
        return formatter.string(fromByteCount: Int64(bytes))
    }
}

// MARK: - Vault Selection Row (Display Name Primary)
/// Shows display name prominently with biometric indicator
/// Four-word identity shown small for reference, expandable on tap
struct VaultSelectionRow: View {
    let vault: SwiftVaultInfo
    let hasPasskey: Bool
    let onTap: () -> Void
    let onPasswordLogin: () -> Void

    @State private var showFullFourWords = false

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Button(action: onTap) {
                HStack(spacing: 16) {
                    // Avatar with biometric indicator
                    ZStack(alignment: .bottomTrailing) {
                        Image(systemName: "person.circle.fill")
                            .font(.system(size: 44))
                            .foregroundColor(.communitasCyan)

                        if hasPasskey {
                            Image(systemName: "touchid")
                                .font(.system(size: 14))
                                .foregroundColor(.green)
                                .background(
                                    Circle()
                                        .fill(Color.communitasDark)
                                        .frame(width: 20, height: 20)
                                )
                        }
                    }

                    // Name and identity info
                    VStack(alignment: .leading, spacing: 4) {
                        // Display name : four-word-address format
                        HStack(spacing: 0) {
                            Text(extractDisplayName(vault.displayName))
                                .font(.title3)
                                .fontWeight(.semibold)
                                .foregroundColor(.white)
                            Text(" : ")
                                .font(.title3)
                                .foregroundColor(.gray)
                            Text(showFullFourWords ? vault.fourWords : shortFourWords(vault.fourWords))
                                .font(.system(.subheadline, design: .monospaced))
                                .foregroundColor(.communitasCyan.opacity(0.9))
                        }

                        // Expand/collapse hint
                        HStack(spacing: 4) {
                            Image(systemName: "key")
                                .font(.system(size: 9))
                            Text(showFullFourWords ? "Hide full address" : "Show full address")
                                .font(.caption2)
                            Image(systemName: showFullFourWords ? "chevron.up" : "chevron.down")
                                .font(.system(size: 8))
                        }
                        .foregroundColor(.gray.opacity(0.6))
                        .onTapGesture {
                            withAnimation(.easeInOut(duration: 0.2)) {
                                showFullFourWords.toggle()
                            }
                        }
                    }

                    Spacer()

                    // Auth method indicator
                    VStack(alignment: .trailing, spacing: 4) {
                        if hasPasskey {
                            HStack(spacing: 4) {
                                Image(systemName: "touchid")
                                Text("Touch ID")
                            }
                            .font(.caption)
                            .foregroundColor(.green)
                        } else {
                            HStack(spacing: 4) {
                                Image(systemName: "key.fill")
                                Text("Password")
                            }
                            .font(.caption)
                            .foregroundColor(.communitasCyan.opacity(0.7))
                        }

                        Text(formatDate(vault.lastAccessed))
                            .font(.caption2)
                            .foregroundColor(.gray.opacity(0.6))
                    }
                }
                .padding()
            }
            .buttonStyle(.plain)

            // Expanded four-word with copy button
            if showFullFourWords {
                HStack {
                    Text(vault.fourWords)
                        .font(.system(.caption, design: .monospaced))
                        .foregroundColor(.communitasCyan.opacity(0.8))
                        .textSelection(.enabled)

                    Button {
                        #if os(macOS)
                        NSPasteboard.general.clearContents()
                        NSPasteboard.general.setString(vault.fourWords, forType: .string)
                        #endif
                    } label: {
                        Image(systemName: "doc.on.doc")
                            .font(.caption)
                            .foregroundColor(.communitasCyan)
                    }
                    .buttonStyle(.plain)
                    .help("Copy four-word address")

                    Spacer()
                }
                .padding(.horizontal)
                .padding(.bottom, 12)
                .transition(.opacity.combined(with: .move(edge: .top)))
            }
        }
        .background(Color.white.opacity(0.05))
        .overlay(
            RoundedRectangle(cornerRadius: 12)
                .stroke(hasPasskey ? Color.green.opacity(0.3) : Color.clear, lineWidth: 1)
        )
        .cornerRadius(12)
        .contextMenu {
            if hasPasskey {
                Button(action: onPasswordLogin) {
                    Label("Use Password Instead", systemImage: "key.fill")
                }
            }
            Button {
                #if os(macOS)
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(vault.fourWords, forType: .string)
                #endif
            } label: {
                Label("Copy Four-Word Address", systemImage: "doc.on.doc")
            }
        }
    }

    private func shortFourWords(_ fourWords: String) -> String {
        let words = fourWords.split(separator: "-")
        if words.count >= 2 {
            return "\(words[0])-\(words[1])..."
        }
        return fourWords
    }

    private func formatDate(_ timestamp: UInt64) -> String {
        let date = Date(timeIntervalSince1970: Double(timestamp))
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .abbreviated
        return formatter.localizedString(for: date, relativeTo: Date())
    }

    /// Extract just the display name from a string that might contain "(four-word-id)"
    /// e.g. "User (bear-wolf-swift-dragon)" -> "User"
    /// e.g. "Alice" -> "Alice"
    /// e.g. "" -> "My Identity"
    private func extractDisplayName(_ displayName: String) -> String {
        let trimmed = displayName.trimmingCharacters(in: .whitespaces)
        if trimmed.isEmpty {
            return "My Identity"
        }
        // Check if it has a pattern like "Name (word-word-word-word)"
        if let parenRange = trimmed.range(of: " (", options: .backwards) {
            let name = String(trimmed[..<parenRange.lowerBound]).trimmingCharacters(in: .whitespaces)
            return name.isEmpty ? "My Identity" : name
        }
        return trimmed
    }
}

#Preview {
    AuthenticationView()
        .environmentObject(AppState())
}
