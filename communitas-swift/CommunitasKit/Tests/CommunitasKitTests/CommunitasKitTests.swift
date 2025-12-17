import XCTest
@testable import CommunitasKit

final class CommunitasKitTests: XCTestCase {
    
    func testInitialization() async throws {
        let tempDir = FileManager.default.temporaryDirectory
        
        let client = try await CommunitasClient(
            fourWords: "ocean-forest-moon-star",
            displayName: "Swift Tester",
            deviceName: "Virtual iPhone",
            storagePath: tempDir.path
        )
        
        let profile = await client.getProfile()
        
        XCTAssertEqual(profile.fourWords, "ocean-forest-moon-star")
        XCTAssertEqual(profile.displayName, "Swift Tester")
        XCTAssertEqual(profile.deviceName, "Virtual iPhone")
    }
    
    func testEntityWorkflow() async throws {
        let tempDir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try? FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
        
        let client = try await CommunitasClient(
            fourWords: "ocean-forest-moon-star",
            displayName: "Swift Tester",
            deviceName: "Virtual iPhone",
            storagePath: tempDir.path
        )
        
        // 1. List empty
        var entities = try await client.listEntities()
        XCTAssertTrue(entities.isEmpty)
        
        // 2. Create Group
        let group = try await client.createEntity(
            name: "Swift Group",
            entityType: .group,
            description: "A test group from XCTest"
        )
        
        XCTAssertEqual(group.name, "Swift Group")
        XCTAssertEqual(group.entityType, .group)
        
        // 3. Verify list
        entities = try await client.listEntities()
        XCTAssertEqual(entities.count, 1)
        XCTAssertEqual(entities.first?.id, group.id)
    }
    
    func testMessaging() async throws {
        let tempDir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try? FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
        
        let client = try await CommunitasClient(
            fourWords: "ocean-forest-moon-star",
            displayName: "Swift Tester",
            deviceName: "Virtual iPhone",
            storagePath: tempDir.path
        )
        
        // Create channel
        let channel = try await client.createEntity(
            name: "General",
            entityType: .channel,
            description: nil
        )
        
        // Send message
        let msgId = try await client.sendMessage(
            entityId: channel.id,
            text: "Hello from Swift!",
            replyToId: nil
        )
        
        XCTAssertFalse(msgId.isEmpty)
        
        // Retrieve
        let messages = try await client.getMessages(entityId: channel.id)
        XCTAssertEqual(messages.count, 1)
        XCTAssertEqual(messages.first?.text, "Hello from Swift!")
        XCTAssertEqual(messages.first?.author, "ocean-forest-moon-star")
    }
}
