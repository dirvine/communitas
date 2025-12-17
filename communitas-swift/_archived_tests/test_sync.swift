// Synchronous test - just call a simple FFI function
import Foundation

@main
struct SyncTest {
    static func main() {
        print("Starting synchronous test...")

        // Try calling a function that exists in the generated bindings
        // The simplest UniFFI function we can call

        print("Attempting to access SwiftEntityType enum...")
        let entityType = SwiftEntityType.group
        print("Successfully created entity type: \(entityType)")

        print("Creating SwiftUserProfile struct...")
        let profile = SwiftUserProfile(
            fourWords: "test-test-test-test",
            displayName: "Test",
            deviceName: "Mac",
            deviceType: "Desktop"
        )
        print("Created profile: \(profile.fourWords)")

        print("\n✅ Synchronous tests passed!")
    }
}
