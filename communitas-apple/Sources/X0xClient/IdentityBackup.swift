import CommonCrypto
import CryptoKit
import Foundation

/// Portable backup of the private identity material that defines a local x0x agent.
///
/// Unlike an agent card, this bundle contains private key files. Callers must only
/// create it after explicit user consent and should store it in a user-selected,
/// protected location.
public struct IdentityBackupBundle: Codable, Sendable, Equatable {
    /// Stable schema marker for future migrations.
    public let schema: String
    /// Unix timestamp at which the bundle was created.
    public let exportedAtUnixSeconds: UInt64
    /// Agent id reported by the daemon at export time.
    public let agentId: String
    /// Machine id reported by the daemon at export time, when available.
    public let machineId: String?
    /// Backed-up key/certificate files.
    public let files: [IdentityBackupFile]
}

/// An encrypted wrapper around `IdentityBackupBundle`.
public struct EncryptedIdentityBackupBundle: Codable, Sendable, Equatable {
    public let schema: String
    public let saltBase64: String
    public let ciphertextBase64: String

    public init(schema: String, saltBase64: String, ciphertextBase64: String) {
        self.schema = schema
        self.saltBase64 = saltBase64
        self.ciphertextBase64 = ciphertextBase64
    }
}

/// One file inside an ``IdentityBackupBundle``.
public struct IdentityBackupFile: Codable, Sendable, Equatable {
    /// Logical role of this file.
    public let kind: IdentityBackupFileKind
    /// Original absolute path. This is informational only and must not be
    /// blindly trusted during restore.
    public let originalPath: String
    /// Raw file bytes, base64 encoded.
    public let base64: String
    /// Raw byte length before base64 encoding.
    public let byteCount: Int
    /// SHA-256 of the raw bytes as lowercase hex.
    public let sha256Hex: String
}

/// File roles understood by the identity-backup exporter.
public enum IdentityBackupFileKind: String, Codable, Sendable, CaseIterable {
    /// Portable agent ML-DSA keypair (`agent.key`).
    case agentKey = "agent_key"
    /// Machine-pinned ML-DSA keypair (`machine.key`).
    case machineKey = "machine_key"
    /// Optional human/operator keypair (`user.key`).
    case userKey = "user_key"
    /// Optional user→agent certificate (`agent.cert`).
    case agentCertificate = "agent_certificate"
    /// Optional per-agent ML-KEM keypair (`agent_kem.key`).
    case agentKemKey = "agent_kem_key"
}

/// Errors raised while exporting private key material.
public enum IdentityBackupError: Error, CustomStringConvertible, Equatable {
    /// The required `agent.key` file was missing.
    case missingAgentKey(String)
    /// The required `machine.key` file was missing.
    case missingMachineKey(String)
    /// A file existed but could not be read.
    case unreadableFile(String)
    /// The generated bundle did not contain any private key files.
    case emptyBundle

    public var description: String {
        switch self {
        case .missingAgentKey(let path):
            return "required agent key is missing at \(path)"
        case .missingMachineKey(let path):
            return "required machine key is missing at \(path)"
        case .unreadableFile(let path):
            return "could not read identity backup file at \(path)"
        case .emptyBundle:
            return "identity backup contained no private key files"
        }
    }
}

/// Consent-gated identity-key backup helper.
///
/// The daemon currently exposes agent cards over REST, but cards are public
/// metadata and are not backups. This helper backs up the local key files that
/// x0xd itself uses (`agent.key`, `machine.key`, optional `user.key`, optional
/// `agent.cert`, optional `agent_kem.key`) into an explicit JSON bundle.
public enum IdentityBackupExporter {
    /// Default x0x identity directory (`~/.x0x`).
    public static func defaultIdentityDirectory() -> URL {
        URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent(".x0x", isDirectory: true)
    }

    /// Default x0x data directory (`Application Support/x0x`) when the platform
    /// exposes an application-support location.
    public static func defaultDataDirectory() -> URL? {
        FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)
            .first?
            .appendingPathComponent("x0x", isDirectory: true)
    }

    /// Build a backup bundle from local key files.
    ///
    /// - Parameters:
    ///   - agentId: Agent id to record in the bundle metadata.
    ///   - machineId: Machine id to record in the bundle metadata.
    ///   - identityDir: Directory containing `agent.key` and `machine.key`.
    ///   - dataDir: Directory containing optional daemon data key material such
    ///     as `agent_kem.key`.
    public static func exportBundle(
        agentId: String,
        machineId: String?,
        identityDir: URL = defaultIdentityDirectory(),
        dataDir: URL? = defaultDataDirectory()
    ) throws -> IdentityBackupBundle {
        let agentKey = identityDir.appendingPathComponent("agent.key")
        let machineKey = identityDir.appendingPathComponent("machine.key")

        guard FileManager.default.isReadableFile(atPath: agentKey.path) else {
            throw IdentityBackupError.missingAgentKey(agentKey.path)
        }
        guard FileManager.default.isReadableFile(atPath: machineKey.path) else {
            throw IdentityBackupError.missingMachineKey(machineKey.path)
        }

        var files: [IdentityBackupFile] = []
        try files.append(readFile(kind: .agentKey, url: agentKey))
        try files.append(readFile(kind: .machineKey, url: machineKey))

        let optionalIdentityFiles: [(IdentityBackupFileKind, URL)] = [
            (.userKey, identityDir.appendingPathComponent("user.key")),
            (.agentCertificate, identityDir.appendingPathComponent("agent.cert")),
        ]
        for (kind, url) in optionalIdentityFiles where FileManager.default.isReadableFile(atPath: url.path) {
            try files.append(readFile(kind: kind, url: url))
        }

        if let dataDir {
            let kem = dataDir.appendingPathComponent("agent_kem.key")
            if FileManager.default.isReadableFile(atPath: kem.path) {
                try files.append(readFile(kind: .agentKemKey, url: kem))
            }
        }

        guard !files.isEmpty else {
            throw IdentityBackupError.emptyBundle
        }

        return IdentityBackupBundle(
            schema: "x0x.identity-backup.v1",
            exportedAtUnixSeconds: UInt64(Date().timeIntervalSince1970),
            agentId: agentId,
            machineId: machineId,
            files: files
        )
    }

    /// Derive a 256-bit AES key from a passphrase using PBKDF2 with SHA-256 and a salt.
    private static func deriveKey(passphrase: String, salt: Data) throws -> SymmetricKey {
        var keyData = Data(count: 32)
        let passwordBytes = Array(passphrase.utf8)
        let saltBytes = Array(salt)
        let keyLength = keyData.count

        let status = keyData.withUnsafeMutableBytes { keyBytes in
            CCKeyDerivationPBKDF(
                CCPBKDFAlgorithm(kCCPBKDF2),
                passphrase,
                passwordBytes.count,
                saltBytes,
                saltBytes.count,
                CCPseudoRandomAlgorithm(kCCPRFHmacAlgSHA256),
                10000, // iterations
                keyBytes.baseAddress?.assumingMemoryBound(to: UInt8.self),
                keyLength
            )
        }
        guard status == kCCSuccess else {
            throw NSError(domain: "IdentityBackupError", code: -2, userInfo: [NSLocalizedDescriptionKey: "Key derivation failed"])
        }
        return SymmetricKey(data: keyData)
    }

    /// Encrypt a bundle using a passphrase and return the JSON-encoded EncryptedIdentityBackupBundle data.
    public static func encryptBundle(_ bundle: IdentityBackupBundle, with passphrase: String) throws -> Data {
        // Generate random 16-byte salt
        var salt = Data(count: 16)
        let result = salt.withUnsafeMutableBytes { SecRandomCopyBytes(kSecRandomDefault, 16, $0.baseAddress!) }
        guard result == errSecSuccess else {
            throw NSError(domain: "IdentityBackupError", code: -1, userInfo: [NSLocalizedDescriptionKey: "Failed to generate random salt"])
        }

        let symmetricKey = try deriveKey(passphrase: passphrase, salt: salt)

        let encoder = JSONEncoder()
        let bundleData = try encoder.encode(bundle)
        let sealedBox = try AES.GCM.seal(bundleData, using: symmetricKey)

        guard let combined = sealedBox.combined else {
            throw NSError(domain: "IdentityBackupError", code: -5, userInfo: [NSLocalizedDescriptionKey: "Encryption packing failed"])
        }

        let encryptedBundle = EncryptedIdentityBackupBundle(
            schema: "x0x.encrypted-identity-backup.v1",
            saltBase64: salt.base64EncodedString(),
            ciphertextBase64: combined.base64EncodedString()
        )
        return try JSONEncoder().encode(encryptedBundle)
    }

    /// Decrypt an encrypted bundle data using a passphrase and return the decrypted IdentityBackupBundle.
    public static func decryptBundle(data: Data, with passphrase: String) throws -> IdentityBackupBundle {
        let encryptedBundle = try JSONDecoder().decode(EncryptedIdentityBackupBundle.self, from: data)
        guard encryptedBundle.schema == "x0x.encrypted-identity-backup.v1" else {
            throw NSError(domain: "IdentityBackupError", code: -3, userInfo: [NSLocalizedDescriptionKey: "Unsupported backup schema"])
        }
        guard let salt = Data(base64Encoded: encryptedBundle.saltBase64),
              let combined = Data(base64Encoded: encryptedBundle.ciphertextBase64) else {
            throw NSError(domain: "IdentityBackupError", code: -4, userInfo: [NSLocalizedDescriptionKey: "Malformed encrypted backup data"])
        }

        let symmetricKey = try deriveKey(passphrase: passphrase, salt: salt)
        let sealedBox = try AES.GCM.SealedBox(combined: combined)
        let decryptedData = try AES.GCM.open(sealedBox, using: symmetricKey)
        return try JSONDecoder().decode(IdentityBackupBundle.self, from: decryptedData)
    }

    /// Encode, encrypt, and write a backup bundle to disk.
    public static func writeBundle(_ bundle: IdentityBackupBundle, to url: URL, with passphrase: String) throws {
        let encryptedData = try encryptBundle(bundle, with: passphrase)
        try encryptedData.write(to: url, options: .atomic)
    }

    /// Read, decrypt, and decode a backup bundle from disk.
    public static func readBundle(from url: URL, with passphrase: String) throws -> IdentityBackupBundle {
        let data = try Data(contentsOf: url)
        return try decryptBundle(data: data, with: passphrase)
    }

    private static func readFile(kind: IdentityBackupFileKind, url: URL) throws -> IdentityBackupFile {
        let data: Data
        do {
            data = try Data(contentsOf: url)
        } catch {
            throw IdentityBackupError.unreadableFile(url.path)
        }
        return IdentityBackupFile(
            kind: kind,
            originalPath: url.path,
            base64: data.base64EncodedString(),
            byteCount: data.count,
            sha256Hex: SHA256.hash(data: data).map { String(format: "%02x", $0) }.joined()
        )
    }
}
