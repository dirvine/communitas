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
        #expect(cases.contains(.blocked))
        #expect(cases.contains(.unknown))
        #expect(cases.contains(.known))
        #expect(cases.contains(.trusted))
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

    @Test("HealthStatus decodes flat response")
    func healthStatusDecoding() throws {
        let json = """
        {
            "ok": true,
            "status": "healthy",
            "version": "0.10.0",
            "peers": 4,
            "uptime_secs": 300
        }
        """
        let data = Data(json.utf8)
        let health = try JSONDecoder().decode(HealthStatus.self, from: data)
        #expect(health.ok == true)
        #expect(health.status == "healthy")
        #expect(health.version == "0.10.0")
        #expect(health.peers == 4)
        #expect(health.uptimeSecs == 300)
    }

    @Test("ApiEnvelope decodes error response")
    func apiErrorDecoding() throws {
        let json = """
        {
            "ok": false,
            "error": "not found"
        }
        """
        let data = Data(json.utf8)
        let envelope = try JSONDecoder().decode(ApiEnvelope.self, from: data)
        #expect(!envelope.ok)
        #expect(envelope.error == "not found")
    }

    @Test("DaemonStatus decodes flat response")
    func daemonStatusDecoding() throws {
        let json = """
        {
            "ok": true,
            "status": "connected",
            "version": "0.10.0",
            "uptime_secs": 300,
            "api_address": "127.0.0.1:12700",
            "external_addrs": ["203.0.113.5:5483"],
            "agent_id": "8a3f0000",
            "peers": 4,
            "warnings": []
        }
        """
        let data = Data(json.utf8)
        let status = try JSONDecoder().decode(DaemonStatus.self, from: data)
        #expect(status.status == "connected")
        #expect(status.uptimeSecs == 300)
        #expect(status.apiAddress == "127.0.0.1:12700")
        #expect(status.agentId == "8a3f0000")
        #expect(status.peers == 4)
    }

    @Test("AgentIdentity decodes flat response")
    func agentIdentityDecoding() throws {
        let json = """
        {
            "ok": true,
            "agent_id": "hex64abc",
            "machine_id": "hex64def",
            "user_id": null
        }
        """
        let data = Data(json.utf8)
        let agent = try JSONDecoder().decode(AgentIdentity.self, from: data)
        #expect(agent.agentId == "hex64abc")
        #expect(agent.machineId == "hex64def")
        #expect(agent.userId == nil)
    }

    @Test("ContactListResponse decodes wrapped list")
    func contactListDecoding() throws {
        let json = """
        {
            "ok": true,
            "contacts": [
                {"agent_id": "abc", "trust_level": "known", "label": "Alice", "added_at": 1234, "last_seen": null}
            ]
        }
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(ContactListResponse.self, from: data)
        #expect(resp.contacts.count == 1)
        #expect(resp.contacts[0].agentId == "abc")
    }

    @Test("GroupListResponse decodes wrapped list")
    func groupListDecoding() throws {
        let json = """
        {
            "ok": true,
            "groups": [
                {"group_id": "grp1", "name": "Team", "description": "", "creator": "abc", "created_at": 1234, "member_count": 3}
            ]
        }
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(GroupListResponse.self, from: data)
        #expect(resp.groups.count == 1)
        #expect(resp.groups[0].creator == "abc")
    }

    @Test("NetworkStatus decodes flat response")
    func networkStatusDecoding() throws {
        let json = """
        {
            "ok": true,
            "avg_rtt_ms": 76.5,
            "can_receive_direct": true,
            "connected_peers": 4,
            "direct_connections": 11,
            "external_addrs": ["203.0.113.5:5483"],
            "hole_punch_success_rate": 0.0,
            "nat_type": "FullCone"
        }
        """
        let data = Data(json.utf8)
        let status = try JSONDecoder().decode(NetworkStatus.self, from: data)
        #expect(status.connectedPeers == 4)
        #expect(status.natType == "FullCone")
        #expect(status.externalAddrs?.count == 1)
    }

    @Test("PeerListResponse decodes wrapped list")
    func peerListDecoding() throws {
        let json = """
        {
            "ok": true,
            "peers": [{"id": "peer1"}, {"id": "peer2"}, {"id": "peer3"}]
        }
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(PeerListResponse.self, from: data)
        #expect(resp.peers.count == 3)
        #expect(resp.peerInfos[0].peerId == "peer1")
    }

    @Test("GossipStatsResponse decodes diagnostics wrapper")
    func gossipStatsDecoding() throws {
        let json = """
        {
            "ok": true,
            "stats": {
                "publish_total": 13,
                "publish_failed": 0,
                "incoming_total": 12,
                "incoming_decoded": 12,
                "incoming_decode_failed": 0,
                "delivered_to_subscriber": 12,
                "subscriber_channel_closed": 0,
                "in_flight_decode": 0,
                "decode_to_delivery_drops": 0
            }
        }
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(GossipStatsResponse.self, from: data)
        #expect(resp.stats.publishTotal == 13)
        #expect(resp.stats.deliveredToSubscriber == 12)
        #expect(resp.stats.decodeToDeliveryDrops == 0)
    }

    @Test("Peer diagnostics decode flat responses")
    func peerDiagnosticsDecoding() throws {
        let probeJson = """
        {
            "ok": true,
            "rtt_ms": 17,
            "rtt_us": 17321,
            "timeout_ms": 1000
        }
        """
        let healthJson = """
        {
            "ok": true,
            "peer_id": "peer1",
            "health": "ConnectionHealth { rtt: 17ms }"
        }
        """
        let probe = try JSONDecoder().decode(ProbePeerResult.self, from: Data(probeJson.utf8))
        let health = try JSONDecoder().decode(PeerHealth.self, from: Data(healthJson.utf8))
        #expect(probe.rttMs == 17)
        #expect(probe.timeoutMs == 1000)
        #expect(health.peerId == "peer1")
        #expect(health.health?.contains("ConnectionHealth") == true)
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
            "total_size": 1024,
            "bytes_transferred": 512,
            "direction": "Sending",
            "status": "InProgress",
            "remote_agent_id": "agent-x",
            "sha256": "abc123",
            "started_at": 1700000000
        }
        """
        let data = Data(json.utf8)
        let transfer = try JSONDecoder().decode(FileTransfer.self, from: data)
        #expect(transfer.transferId == "tx-001")
        #expect(transfer.direction == .sending)
        #expect(transfer.status == .inProgress)
        #expect(transfer.progress == 0.5)
        #expect(transfer.remoteAgentId == "agent-x")
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

    // MARK: - New Model Type Tests (API Parity)

    @Test("AgentCard decodes from JSON")
    func agentCardDecoding() throws {
        let json = """
        {
            "ok": true,
            "card": {
                "display_name": "Alice",
                "agent_id": "hex64aaa",
                "machine_id": "hex64bbb",
                "user_id": null,
                "addresses": ["203.0.113.5:12000"],
                "groups": [{"name": "Team", "invite_link": "x0x://invite/abc"}],
                "stores": [{"name": "Data", "topic": "store-topic"}],
                "created_at": 1700000000
            },
            "link": "x0x://agent/base64data"
        }
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(AgentCardResponse.self, from: data)
        #expect(resp.card.displayName == "Alice")
        #expect(resp.card.agentId == "hex64aaa")
        #expect(resp.card.addresses?.count == 1)
        #expect(resp.card.externalAddresses?.first == "203.0.113.5:12000")
        #expect(resp.card.groups?.count == 1)
        #expect(resp.card.groups?[0].inviteLink == "x0x://invite/abc")
        #expect(resp.card.stores?.count == 1)
        #expect(resp.link == "x0x://agent/base64data")
    }

    @Test("ImportCardResponse decodes from JSON")
    func importCardResponseDecoding() throws {
        let json = """
        {"ok": true, "agent_id": "hex64", "display_name": "Bob", "trust_level": "known", "groups": 2, "stores": 1}
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(ImportCardResponse.self, from: data)
        #expect(resp.agentId == "hex64")
        #expect(resp.displayName == "Bob")
        #expect(resp.groups == 2)
        #expect(resp.stores == 1)
    }

    @Test("AnnounceRequest encodes correctly")
    func announceRequestEncoding() throws {
        let request = AnnounceRequest(includeUserIdentity: true, humanConsent: false)
        let data = try JSONEncoder().encode(request)
        let dict = try JSONDecoder().decode([String: Bool].self, from: data)
        #expect(dict["include_user_identity"] == true)
        #expect(dict["human_consent"] == false)
    }

    @Test("DirectConnection decodes from JSON")
    func directConnectionDecoding() throws {
        let json = """
        {"ok": true, "connections": [{"agent_id": "abc", "machine_id": "def", "connected_at": 1700000000}]}
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(DirectConnectionList.self, from: data)
        #expect(resp.connections.count == 1)
        #expect(resp.connections[0].agentId == "abc")
        #expect(resp.connections[0].connectedAt == 1700000000)
    }

    @Test("MlsGroup decodes from JSON")
    func mlsGroupDecoding() throws {
        let json = """
        {"ok": true, "group_id": "grp1", "epoch": 5, "members": ["agent-a", "agent-b"], "member_count": 2}
        """
        let data = Data(json.utf8)
        let group = try JSONDecoder().decode(MlsGroup.self, from: data)
        #expect(group.groupId == "grp1")
        #expect(group.epoch == 5)
        #expect(group.members?.count == 2)
        #expect(group.memberCount == 2)
    }

    @Test("MlsGroupList decodes from JSON")
    func mlsGroupListDecoding() throws {
        let json = """
        {"ok": true, "groups": [{"group_id": "g1", "epoch": 1, "member_count": 3}]}
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(MlsGroupList.self, from: data)
        #expect(resp.groups.count == 1)
        #expect(resp.groups[0].groupId == "g1")
    }

    @Test("AddMlsMemberResponse decodes from JSON")
    func addMlsMemberResponseDecoding() throws {
        let json = """
        {"ok": true, "epoch": 3, "members": ["a", "b", "c"]}
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(AddMlsMemberResponse.self, from: data)
        #expect(resp.epoch == 3)
        #expect(resp.members?.count == 3)
    }

    @Test("WelcomeResponse decodes from JSON")
    func welcomeResponseDecoding() throws {
        let json = """
        {"ok": true, "welcome": "aGVsbG8=", "group_id": "grp1", "epoch": 2}
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(WelcomeResponse.self, from: data)
        #expect(resp.welcome == "aGVsbG8=")
        #expect(resp.groupId == "grp1")
        #expect(resp.epoch == 2)
    }

    @Test("EncryptResponse decodes from JSON")
    func encryptResponseDecoding() throws {
        let json = """
        {"ok": true, "ciphertext": "Y2lwaGVydGV4dA==", "epoch": 7}
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(EncryptResponse.self, from: data)
        #expect(resp.ciphertext == "Y2lwaGVydGV4dA==")
        #expect(resp.epoch == 7)
    }

    @Test("IntroductionCard decodes trust-gated payload")
    func introductionCardDecoding() throws {
        let json = """
        {
            "ok": true,
            "agent_id": "hex64aaa",
            "display_name": "Alice",
            "identity_words": "river meadow lantern star",
            "services": [
                {"name": "presence", "description": "Online/offline presence visibility", "min_trust": "unknown"}
            ]
        }
        """
        let data = Data(json.utf8)
        let card = try JSONDecoder().decode(IntroductionCard.self, from: data)
        #expect(card.agentId == "hex64aaa")
        #expect(card.displayName == "Alice")
        #expect(card.identityWords == "river meadow lantern star")
        #expect(card.services.count == 1)
        #expect(card.services[0].minTrust == "unknown")
    }

    @Test("TrustEvaluation decodes from JSON")
    func trustEvaluationDecoding() throws {
        let json = """
        {"ok": true, "decision": "Allow"}
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(TrustEvaluation.self, from: data)
        #expect(resp.decision == "Allow")
    }

    @Test("BootstrapCacheStatus decodes from JSON")
    func bootstrapCacheDecoding() throws {
        let json = """
        {"ok": true, "connected_peers": ["peer-1", "peer-2"], "connection_count": 2}
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(BootstrapCacheStatus.self, from: data)
        #expect(resp.connectedPeers?.count == 2)
        #expect(resp.connectionCount == 2)
    }

    @Test("TaskListIndex decodes from JSON")
    func taskListIndexDecoding() throws {
        let json = """
        {"ok": true, "task_lists": [{"id": "list-1", "topic": "sync-topic"}]}
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(TaskListIndex.self, from: data)
        #expect(resp.taskLists.count == 1)
        #expect(resp.taskLists[0].id == "list-1")
        #expect(resp.taskLists[0].topic == "sync-topic")
    }

    @Test("WsSessionList decodes from JSON")
    func wsSessionListDecoding() throws {
        let json = """
        {"ok": true, "sessions": [{"session_id": "uuid-1", "subscribed_topics": ["chat"], "receives_direct": true}], "shared_subscriptions": {"chat": 2}}
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(WsSessionList.self, from: data)
        #expect(resp.sessions.count == 1)
        #expect(resp.sessions[0].sessionId == "uuid-1")
        #expect(resp.sessions[0].receivesDirect == true)
        #expect(resp.sharedSubscriptions?["chat"] == 2)
    }

    @Test("Revocation decodes from JSON")
    func revocationDecoding() throws {
        let json = """
        {"ok": true, "revocations": [{"agent_id": "abc", "reason": "compromised", "timestamp": 1700000000, "revoker_id": "def"}]}
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(RevocationList.self, from: data)
        #expect(resp.revocations.count == 1)
        #expect(resp.revocations[0].reason == "compromised")
        #expect(resp.revocations[0].revokerId == "def")
    }

    @Test("UpgradeStatus decodes from JSON")
    func upgradeStatusDecoding() throws {
        let json = """
        {"ok": true, "update_available": true, "version": "0.12.3", "current_version": "0.11.1"}
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(UpgradeStatus.self, from: data)
        #expect(resp.updateAvailable == true)
        #expect(resp.version == "0.12.3")
        #expect(resp.currentVersion == "0.11.1")
    }

    @Test("DiscoveredAgentWrapper decodes nested agent payload")
    func discoveredAgentWrapperDecoding() throws {
        let json = """
        {"ok": true, "agent": {"agent_id": "hex64", "machine_id": "mhex64", "user_id": null, "addresses": ["1.2.3.4:12000"], "announced_at": 1700000000, "last_seen": 1700000100}}
        """
        let data = Data(json.utf8)
        let wrapper = try JSONDecoder().decode(DiscoveredAgentWrapper.self, from: data)
        let agent = wrapper.agent
        #expect(agent.agentId == "hex64")
        #expect(agent.machineId == "mhex64")
        #expect(agent.addresses.count == 1)
        #expect(agent.lastSeen == 1700000100)
    }

    @Test("FileTransferWrapper decodes nested transfer payload")
    func fileTransferWrapperDecoding() throws {
        let json = """
        {
            "ok": true,
            "transfer": {
                "transfer_id": "tx-001",
                "direction": "Sending",
                "remote_agent_id": "agent-x",
                "filename": "doc.pdf",
                "total_size": 1024,
                "bytes_transferred": 512,
                "status": "InProgress"
            }
        }
        """
        let data = Data(json.utf8)
        let wrapper = try JSONDecoder().decode(FileTransferWrapper.self, from: data)
        #expect(wrapper.transfer.transferId == "tx-001")
        #expect(wrapper.transfer.remoteAgentId == "agent-x")
        #expect(wrapper.transfer.status == .inProgress)
    }

    @Test("UpdateContactRequest encodes correctly")
    func updateContactRequestEncoding() throws {
        let request = UpdateContactRequest(trustLevel: .trusted, identityType: "pinned")
        let data = try JSONEncoder().encode(request)
        let jsonStr = String(data: data, encoding: .utf8)!
        #expect(jsonStr.contains("\"trust_level\":\"trusted\""))
        #expect(jsonStr.contains("\"identity_type\":\"pinned\""))
    }

    @Test("EvaluateTrustRequest encodes correctly")
    func evaluateTrustRequestEncoding() throws {
        let request = EvaluateTrustRequest(agentId: "abc", machineId: "def")
        let data = try JSONEncoder().encode(request)
        let jsonStr = String(data: data, encoding: .utf8)!
        #expect(jsonStr.contains("\"agent_id\":\"abc\""))
        #expect(jsonStr.contains("\"machine_id\":\"def\""))
    }

    @Test("SendFileRequest encodes with optional path")
    func sendFileRequestWithPath() throws {
        let request = SendFileRequest(agentId: "agent1", filename: "doc.pdf", size: 1024, sha256: "abc", path: "/tmp/doc.pdf")
        let data = try JSONEncoder().encode(request)
        let jsonStr = String(data: data, encoding: .utf8)!
        #expect(jsonStr.contains("\"path\":\"\\/tmp\\/doc.pdf\""))
    }

    @Test("CreateMlsGroupRequest encodes with optional group_id")
    func createMlsGroupRequestEncoding() throws {
        let withId = CreateMlsGroupRequest(groupId: "custom-id")
        let data1 = try JSONEncoder().encode(withId)
        let jsonStr1 = String(data: data1, encoding: .utf8)!
        #expect(jsonStr1.contains("\"group_id\":\"custom-id\""))

        let withoutId = CreateMlsGroupRequest(groupId: nil)
        let data2 = try JSONEncoder().encode(withoutId)
        let jsonStr2 = String(data: data2, encoding: .utf8)!
        // Swift's JSONEncoder skips nil optionals by default, so group_id is absent
        #expect(!jsonStr2.contains("\"group_id\""))
    }

    @Test("DecryptRequest encodes correctly")
    func decryptRequestEncoding() throws {
        let request = DecryptRequest(ciphertext: "Y2lwaGVy", epoch: 5)
        let data = try JSONEncoder().encode(request)
        let jsonStr = String(data: data, encoding: .utf8)!
        #expect(jsonStr.contains("\"ciphertext\":\"Y2lwaGVy\""))
        #expect(jsonStr.contains("\"epoch\":5"))
    }

    // MARK: - 0.27.x peer-lifecycle wire shapes (x0xd ≥ 0.19.6 / 0.19.7)

    @Test("PeerHealth decodes legacy health-only response")
    func peerHealthLegacyDecoding() throws {
        // Daemons < 0.19.7 emit only the Debug-string `health` field.
        let json = """
        {
            "ok": true,
            "peer_id": "57ae036829ecbcb5d851b0554de7841a7a5232337172471d9c2e04871582440f",
            "health": "ConnectionHealth { connected: true, generation: Some(4), reader_task_active: Some(true), idle_for: Some(31.24875ms), close_reason: None }"
        }
        """
        let data = Data(json.utf8)
        let health = try JSONDecoder().decode(PeerHealth.self, from: data)
        #expect(health.ok == true)
        #expect(health.health?.contains("connected: true") == true)
        #expect(health.snapshot == nil)
    }

    @Test("PeerHealth decodes structured snapshot response")
    func peerHealthSnapshotDecoding() throws {
        // Daemons ≥ 0.19.7 emit both `health` and the structured `snapshot`.
        let json = """
        {
            "ok": true,
            "peer_id": "57ae036829ecbcb5d851b0554de7841a7a5232337172471d9c2e04871582440f",
            "health": "ConnectionHealth { connected: true, ... }",
            "snapshot": {
                "connected": true,
                "generation": 4,
                "reader_task_active": true,
                "last_received_ms_ago": 31,
                "last_sent_ms_ago": 14,
                "idle_ms": 31,
                "close_reason": null
            }
        }
        """
        let data = Data(json.utf8)
        let health = try JSONDecoder().decode(PeerHealth.self, from: data)
        #expect(health.snapshot?.connected == true)
        #expect(health.snapshot?.generation == 4)
        #expect(health.snapshot?.readerTaskActive == true)
        #expect(health.snapshot?.idleMs == 31)
        #expect(health.snapshot?.closeReason == nil)
        #expect(health.health?.contains("connected: true") == true)
    }

    @Test("PeerHealth snapshot decodes a closed-with-reason response")
    func peerHealthSnapshotClosedDecoding() throws {
        let json = """
        {
            "ok": true,
            "peer_id": "deadbeef",
            "snapshot": {
                "connected": false,
                "generation": 5,
                "reader_task_active": false,
                "last_received_ms_ago": null,
                "last_sent_ms_ago": null,
                "idle_ms": null,
                "close_reason": "Superseded"
            }
        }
        """
        let data = Data(json.utf8)
        let health = try JSONDecoder().decode(PeerHealth.self, from: data)
        #expect(health.snapshot?.connected == false)
        #expect(health.snapshot?.closeReason == "Superseded")
        #expect(health.snapshot?.idleMs == nil)
    }

    @Test("ProbePeerResult decodes successful probe")
    func probePeerResultDecoding() throws {
        let json = """
        {
            "ok": true,
            "rtt_ms": 0,
            "rtt_us": 209,
            "timeout_ms": 3000
        }
        """
        let data = Data(json.utf8)
        let probe = try JSONDecoder().decode(ProbePeerResult.self, from: data)
        #expect(probe.ok == true)
        #expect(probe.rttUs == 209)
        #expect(probe.timeoutMs == 3000)
    }

    @Test("ProbePeerResult decodes a failed probe")
    func probePeerResultErrorDecoding() throws {
        let json = """
        {
            "ok": false,
            "error": "no live connection",
            "timeout_ms": 1000
        }
        """
        let data = Data(json.utf8)
        let probe = try JSONDecoder().decode(ProbePeerResult.self, from: data)
        #expect(probe.ok == false)
        #expect(probe.error == "no live connection")
    }

    @Test("DirectSendResponse decodes ACK round-trip")
    func directSendResponseAckDecoding() throws {
        // POST /direct/send with require_ack_ms=3000 — peer ACK round-trip
        // arrives in `require_ack`.
        let json = """
        {
            "ok": true,
            "path": "gossip_inbox",
            "request_id": "614d0f48011c0cf1df6f97f9f9658705",
            "require_ack": {
                "ok": true,
                "rtt_ms": 0,
                "rtt_us": 411
            },
            "retries_used": 0
        }
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(DirectSendResponse.self, from: data)
        #expect(resp.ok == true)
        #expect(resp.requestId == "614d0f48011c0cf1df6f97f9f9658705")
        #expect(resp.requireAck?.ok == true)
        #expect(resp.requireAck?.rttUs == 411)
    }

    @Test("DirectSendResponse decodes legacy fire-and-forget response")
    func directSendResponseLegacyDecoding() throws {
        // Without require_ack_ms the daemon omits the `require_ack` block.
        let json = """
        {
            "ok": true,
            "path": "gossip_inbox",
            "request_id": "abc",
            "retries_used": 0
        }
        """
        let data = Data(json.utf8)
        let resp = try JSONDecoder().decode(DirectSendResponse.self, from: data)
        #expect(resp.ok == true)
        #expect(resp.requireAck == nil)
    }

    @Test("DirectMessageRequest encodes require_ack_ms when set")
    func directMessageRequestAckEncoding() throws {
        let req = DirectMessageRequest(
            agentId: "abc",
            payload: "aGVsbG8=",
            requireAckMs: 3000
        )
        let data = try JSONEncoder().encode(req)
        let jsonStr = String(data: data, encoding: .utf8)!
        #expect(jsonStr.contains("\"agent_id\":\"abc\""))
        #expect(jsonStr.contains("\"payload\":\"aGVsbG8=\""))
        #expect(jsonStr.contains("\"require_ack_ms\":3000"))
    }

    @Test("PeerLifecycleEvent decodes Established frame payload")
    func peerLifecycleEventDecoding() throws {
        // Daemon emits these on /peers/events SSE with event=peer-lifecycle.
        let json = """
        {
            "peer_id": "57ae036829ecbcb5d851b0554de7841a7a5232337172471d9c2e04871582440f",
            "event": "Established { generation: 5 }",
            "at_ms": 1777370802198
        }
        """
        let data = Data(json.utf8)
        let event = try JSONDecoder().decode(PeerLifecycleEvent.self, from: data)
        #expect(event.peerId.count == 64)
        #expect(event.event.contains("Established"))
        #expect(event.atMs == 1_777_370_802_198)
    }

    @Test("PeerLifecycleEvent surfaces supersede transitions")
    func peerLifecycleEventSupersedeDecoding() throws {
        // ant-quic 0.27.3+ emits a Replaced/Closing/Closed sequence on
        // connection supersede. Each lands as a separate frame.
        let json = """
        {
            "peer_id": "abc",
            "event": "Replaced { old_generation: 5, new_generation: 6 }",
            "at_ms": 1777370802199
        }
        """
        let data = Data(json.utf8)
        let event = try JSONDecoder().decode(PeerLifecycleEvent.self, from: data)
        #expect(event.event.contains("Replaced"))
        #expect(event.event.contains("old_generation: 5"))
        #expect(event.event.contains("new_generation: 6"))
    }

    // MARK: - SSE frame parsing

    @Test("SseFrame decodes JSON data payload")
    func sseFrameJsonDecoding() throws {
        let frame = SseFrame(
            event: "peer-lifecycle",
            id: nil,
            data: """
            {"peer_id":"abc","event":"Established { generation: 5 }","at_ms":1234}
            """
        )
        let event: PeerLifecycleEvent = try frame.json()
        #expect(event.peerId == "abc")
        #expect(event.event.contains("Established"))
        #expect(event.atMs == 1234)
    }
}
