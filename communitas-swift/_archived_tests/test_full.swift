// Full integration test - Synchronous API
import Foundation

@main
struct TestRunner {
    static func main() {
        print("Starting Communitas FFI test (sync API)...")

        // Sync test - enums
        let entityType = SwiftEntityType.channel
        print("✓ Entity type created: \(entityType)")

        print("Now testing client creation...")

        do {
            let tempDir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
            try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
            print("Storage path: \(tempDir.path)")

            print("Creating client with four-word identity...")
            let client = try CommunitasClient(
                fourWords: "ocean-forest-moon-star",
                displayName: "Swift Test",
                deviceName: "Test Mac",
                storagePath: tempDir.path
            )
            print("✓ Client created successfully!")

            print("Getting profile...")
            let profile = client.getProfile()
            print("✓ Profile: \(profile.fourWords) - \(profile.displayName)")

            print("Checking network status...")
            let isNetworking = client.isNetworkingActive()
            print("✓ Networking active: \(isNetworking)")

            print("Creating entity...")
            let entity = try client.createEntity(
                name: "Test Group",
                entityType: .group,
                description: "A test group"
            )
            print("✓ Created entity: \(entity.name) (id: \(entity.id))")

            print("Listing entities...")
            let entities = try client.listEntities()
            print("✓ Found \(entities.count) entities")

            for ent in entities {
                print("  - \(ent.name) (\(ent.entityType))")
            }

            // Cleanup
            try? FileManager.default.removeItem(at: tempDir)

            print("\n✅ All tests passed!")

        } catch {
            print("❌ Error: \(error)")
            exit(1)
        }

        print("Test complete.")
    }
}
