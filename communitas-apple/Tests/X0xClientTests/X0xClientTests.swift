import Foundation
import Testing

@testable import X0xClient

@Suite("X0xClient Tests")
struct X0xClientTests {

    @Test("Client initializes with default base URL")
    func defaultBaseURL() {
        let client = X0xClient()
        #expect(client.baseURL.absoluteString == "http://127.0.0.1:12700")
    }

    @Test("Client initializes with custom base URL")
    func customBaseURL() {
        let url = URL(string: "http://localhost:9999")!
        let client = X0xClient(baseURL: url)
        #expect(client.baseURL.absoluteString == "http://localhost:9999")
    }

    @Test("DaemonManager detects not-installed when no binary exists")
    func daemonNotInstalled() {
        let manager = DaemonManager(client: X0xClient(baseURL: URL(string: "http://127.0.0.1:1")!))
        // The binary likely doesn't exist in test environments at standard paths,
        // but this test verifies the method doesn't crash.
        _ = manager.isInstalled()
        _ = manager.binaryPath()
    }

    @Test("TrustLevel covers all cases")
    func trustLevelCases() {
        let cases = TrustLevel.allCases
        #expect(cases.count == 4)
        #expect(cases.contains(.untrusted))
        #expect(cases.contains(.known))
        #expect(cases.contains(.trusted))
        #expect(cases.contains(.verified))
    }

    @Test("Contact model decodes from JSON")
    func contactDecoding() throws {
        let json = """
        {
            "agent_id": "abc123",
            "label": "Alice",
            "trust_level": "trusted",
            "added_at": 1700000000
        }
        """
        let data = Data(json.utf8)
        let contact = try JSONDecoder().decode(Contact.self, from: data)
        #expect(contact.agentId == "abc123")
        #expect(contact.label == "Alice")
        #expect(contact.trustLevel == .trusted)
        #expect(contact.addedAt == 1700000000)
        #expect(contact.id == "abc123")
    }

    @Test("GroupSummary model decodes from JSON")
    func groupSummaryDecoding() throws {
        let json = """
        {
            "group_id": "grp-001",
            "name": "Test Group",
            "member_count": 5,
            "description": "A test group"
        }
        """
        let data = Data(json.utf8)
        let group = try JSONDecoder().decode(GroupSummary.self, from: data)
        #expect(group.groupId == "grp-001")
        #expect(group.name == "Test Group")
        #expect(group.memberCount == 5)
        #expect(group.description == "A test group")
    }

    @Test("PublishRequest encodes correctly")
    func publishRequestEncoding() throws {
        let request = PublishRequest(topic: "chat", payload: "aGVsbG8=")
        let data = try JSONEncoder().encode(request)
        let dict = try JSONDecoder().decode([String: String].self, from: data)
        #expect(dict["topic"] == "chat")
        #expect(dict["payload"] == "aGVsbG8=")
    }

    @Test("ApiResponse decodes envelope with data")
    func apiResponseDecoding() throws {
        let json = """
        {
            "ok": true,
            "data": {
                "status": "healthy",
                "version": "0.1.0"
            }
        }
        """
        let data = Data(json.utf8)
        let response = try JSONDecoder().decode(ApiResponse<HealthStatus>.self, from: data)
        #expect(response.ok)
        #expect(response.data?.status == "healthy")
        #expect(response.data?.version == "0.1.0")
    }

    @Test("ApiResponse decodes error envelope")
    func apiErrorDecoding() throws {
        let json = """
        {
            "ok": false,
            "error": "not found"
        }
        """
        let data = Data(json.utf8)
        let response = try JSONDecoder().decode(ApiResponse<HealthStatus>.self, from: data)
        #expect(!response.ok)
        #expect(response.error == "not found")
        #expect(response.data == nil)
    }

    @Test("ChatMessage creates from gossip payload")
    func chatMessageFromGossip() {
        let payload = Data("Hello, world!".utf8).base64EncodedString()
        let gossip = GossipMessage(
            messageId: "msg-1",
            topic: "chat",
            sender: "agent-a",
            payload: payload,
            timestamp: 1700000000
        )
        let msg = ChatMessage.fromGossip(gossip, myAgentId: "agent-b")
        #expect(msg != nil)
        #expect(msg?.content == "Hello, world!")
        #expect(msg?.isOutgoing == false)

        let outgoing = ChatMessage.fromGossip(gossip, myAgentId: "agent-a")
        #expect(outgoing?.isOutgoing == true)
    }

    @Test("FileTransfer model decodes from JSON")
    func fileTransferDecoding() throws {
        let json = """
        {
            "transfer_id": "tx-001",
            "filename": "doc.pdf",
            "size": 1024,
            "direction": "upload",
            "status": "in_progress",
            "peer_agent_id": "agent-x",
            "progress": 0.5
        }
        """
        let data = Data(json.utf8)
        let transfer = try JSONDecoder().decode(FileTransfer.self, from: data)
        #expect(transfer.transferId == "tx-001")
        #expect(transfer.direction == .upload)
        #expect(transfer.status == .inProgress)
        #expect(transfer.progress == 0.5)
    }

    @Test("DaemonState has expected raw values")
    func daemonStateValues() {
        #expect(DaemonState.notInstalled.rawValue == "notInstalled")
        #expect(DaemonState.running.rawValue == "running")
    }

    @Test("X0xError provides descriptions")
    func errorDescriptions() {
        let errors: [X0xError] = [
            .daemonUnreachable,
            .httpError(statusCode: 404, body: "not found"),
            .apiError(message: "bad request"),
            .daemonNotInstalled,
            .daemonStartFailed(reason: "timeout"),
            .webSocketError(reason: "closed"),
            .invalidURL(path: "/bad"),
            .unexpected(message: "oops"),
        ]
        for error in errors {
            #expect(error.errorDescription != nil)
            #expect(!error.errorDescription!.isEmpty)
        }
    }
}
