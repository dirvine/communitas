import CryptoKit
import Foundation
import Testing

@testable import X0xClient

/// Live x0xd round-trip tests for the **Direct / file transfer** row of
/// the parity matrix.
///
/// Two-daemon file transfer requires both peers to be established on
/// the gossip layer. The localhost ephemeral fixtures boot with empty
/// bootstrap lists by design and never establish a direct connection,
/// so the actual transfer-of-bytes step is out of scope for the live
/// round-trip — it's covered by `tests/e2e_live_network.sh` against a
/// real bootstrap fleet.
///
/// What the matrix asks of Apple is that the surface decodes both the
/// happy and the negative paths:
/// - `POST /files/send` is reachable and returns a structured error
///   (HTTP 500 + `{"ok":false,"error":"…"}`) when the recipient is
///   not connected — proving the wire envelope round-trips through
///   the Swift error path.
/// - `GET /files/transfers` lists the (possibly empty) queue.
@Suite("File transfer round-trip (live x0xd)")
struct FileTransferRoundTripTests {

    @Test("listTransfers decodes the empty-queue wire shape on a fresh daemon")
    func listTransfersDecodesEmpty() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "filex-list")
        defer { daemon.terminate() }

        let transfers = try await daemon.client.listTransfers()
        // Fresh daemon — list is empty but the wire shape must decode.
        #expect(transfers.count == 0)
    }

    @Test("sendFile to disconnected peer surfaces a structured error")
    func sendFileToDisconnectedPeerErrors() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "filex-err")
        defer { daemon.terminate() }

        // Build a temp file with deterministic content so the SHA-256
        // is stable across runs.
        let bytes = Data("communitas-file-rt-\(UUID().uuidString)".utf8)
        let expectedHash = SHA256.hash(data: bytes)
            .map { String(format: "%02x", $0) }
            .joined()
        let tmp = FileManager.default.temporaryDirectory
            .appendingPathComponent("communitas-rt-\(UUID().uuidString).bin")
        try bytes.write(to: tmp)
        defer { try? FileManager.default.removeItem(at: tmp) }

        // Fabricated 64-char hex peer id — daemon validates the
        // shape but cannot reach it.
        let phantom = String(repeating: "ab", count: 32)

        do {
            _ = try await daemon.client.sendFile(
                agentId: phantom,
                filename: tmp.lastPathComponent,
                size: UInt64(bytes.count),
                sha256: expectedHash,
                path: tmp.path
            )
            Issue.record("Expected sendFile to throw against an unreachable phantom peer")
        } catch let error as X0xError {
            switch error {
            case .httpError(let statusCode, let body):
                // The daemon emits 500 with a structured JSON body
                // explaining why — that's the matrix-required round-trip.
                #expect(statusCode >= 400)
                #expect(body.lowercased().contains("recipient")
                    || body.lowercased().contains("not connected")
                    || body.lowercased().contains("agent"))
            case .apiError(let msg):
                #expect(msg.lowercased().contains("recipient")
                    || msg.lowercased().contains("not connected"))
            default:
                Issue.record("Unexpected X0xError variant: \(error)")
            }
        }
    }

    @Test("Reject helper accepts both reason and bare-rejection paths")
    func rejectFileMethodVariants() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "filex-rej")
        defer { daemon.terminate() }

        // No transfers exist — both reject paths should surface a
        // structured 404 / not-found error rather than silently
        // pretending to succeed.
        do {
            try await daemon.client.rejectFile(transferId: "phantom-id")
            Issue.record("Expected rejectFile against missing id to throw")
        } catch let error as X0xError {
            switch error {
            case .httpError(let statusCode, _):
                #expect(statusCode >= 400)
            case .apiError:
                () // ok — daemon emits {"ok": false, ...}
            default:
                Issue.record("Unexpected X0xError variant: \(error)")
            }
        }

        do {
            try await daemon.client.rejectFile(transferId: "phantom-id", reason: "test")
            Issue.record("Expected rejectFile-with-reason against missing id to throw")
        } catch let error as X0xError {
            switch error {
            case .httpError(let statusCode, _):
                #expect(statusCode >= 400)
            case .apiError:
                ()
            default:
                Issue.record("Unexpected X0xError variant: \(error)")
            }
        }
    }
}
