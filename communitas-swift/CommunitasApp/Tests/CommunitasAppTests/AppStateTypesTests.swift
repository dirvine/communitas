import XCTest
@testable import CommunitasAppLib

/// Tests for AppState types and models
final class AppStateTypesTests: XCTestCase {

    // MARK: - ActiveView Equality Tests

    func testActiveViewHome_Equality() {
        let view1 = ActiveView.home
        let view2 = ActiveView.home

        XCTAssertEqual(view1, view2)
    }

    func testActiveViewChat_Equality() {
        let view1 = ActiveView.chat(entityType: "org", entityId: "123", entityName: "Test Org")
        let view2 = ActiveView.chat(entityType: "org", entityId: "123", entityName: "Different Name")

        // Names are ignored in equality - only type and ID matter
        XCTAssertEqual(view1, view2)
    }

    func testActiveViewChat_Inequality() {
        let view1 = ActiveView.chat(entityType: "org", entityId: "123", entityName: "Test")
        let view2 = ActiveView.chat(entityType: "org", entityId: "456", entityName: "Test")

        XCTAssertNotEqual(view1, view2)
    }

    func testActiveViewContactChat_Equality() {
        let view1 = ActiveView.contactChat(fourWords: "ocean-forest-moon-star", displayName: "Alice")
        let view2 = ActiveView.contactChat(fourWords: "ocean-forest-moon-star", displayName: nil)

        // Display names are ignored in equality - only fourWords matter
        XCTAssertEqual(view1, view2)
    }

    func testActiveViewContactChat_Inequality() {
        let view1 = ActiveView.contactChat(fourWords: "ocean-forest-moon-star", displayName: "Alice")
        let view2 = ActiveView.contactChat(fourWords: "bear-wolf-swift-dragon", displayName: "Bob")

        XCTAssertNotEqual(view1, view2)
    }

    func testActiveViewDrive_Equality() {
        let view1 = ActiveView.drive(entityType: "org", entityId: "123")
        let view2 = ActiveView.drive(entityType: "org", entityId: "123")

        XCTAssertEqual(view1, view2)
    }

    func testActiveViewDrive_Inequality() {
        let view1 = ActiveView.drive(entityType: "org", entityId: "123")
        let view2 = ActiveView.drive(entityType: "group", entityId: "123")

        XCTAssertNotEqual(view1, view2)
    }

    func testActiveViewCall_Equality() {
        let view1 = ActiveView.call(peerFourWords: "ocean-forest-moon-star")
        let view2 = ActiveView.call(peerFourWords: "ocean-forest-moon-star")

        XCTAssertEqual(view1, view2)
    }

    func testActiveViewCall_Inequality() {
        let view1 = ActiveView.call(peerFourWords: "ocean-forest-moon-star")
        let view2 = ActiveView.call(peerFourWords: "bear-wolf-swift-dragon")

        XCTAssertNotEqual(view1, view2)
    }

    func testActiveViewProject_Equality() {
        let view1 = ActiveView.project(projectId: "proj-123")
        let view2 = ActiveView.project(projectId: "proj-123")

        XCTAssertEqual(view1, view2)
    }

    func testActiveViewProject_Inequality() {
        let view1 = ActiveView.project(projectId: "proj-123")
        let view2 = ActiveView.project(projectId: "proj-456")

        XCTAssertNotEqual(view1, view2)
    }

    func testActiveView_DifferentTypes_Inequality() {
        let home = ActiveView.home
        let chat = ActiveView.chat(entityType: "org", entityId: "123", entityName: "Test")
        let contact = ActiveView.contactChat(fourWords: "test-words", displayName: nil)
        let drive = ActiveView.drive(entityType: "org", entityId: "123")
        let call = ActiveView.call(peerFourWords: "test-words")
        let project = ActiveView.project(projectId: "123")

        // All different types should not be equal
        XCTAssertNotEqual(home, chat)
        XCTAssertNotEqual(home, contact)
        XCTAssertNotEqual(home, drive)
        XCTAssertNotEqual(home, call)
        XCTAssertNotEqual(home, project)
        XCTAssertNotEqual(chat, contact)
        XCTAssertNotEqual(chat, drive)
        XCTAssertNotEqual(drive, call)
    }

    // MARK: - FileItem Tests

    func testFileItem_Initialization() {
        let now = Date()
        let item = FileItem(
            id: "file-123",
            name: "test.txt",
            path: "/documents/test.txt",
            isDirectory: false,
            sizeBytes: 1024,
            modifiedAt: now
        )

        XCTAssertEqual(item.id, "file-123")
        XCTAssertEqual(item.name, "test.txt")
        XCTAssertEqual(item.path, "/documents/test.txt")
        XCTAssertFalse(item.isDirectory)
        XCTAssertEqual(item.sizeBytes, 1024)
        XCTAssertEqual(item.modifiedAt, now)
    }

    func testFileItem_DirectoryFlag() {
        let directory = FileItem(
            id: "dir-1",
            name: "Documents",
            path: "/Documents",
            isDirectory: true,
            sizeBytes: 0,
            modifiedAt: Date()
        )

        let file = FileItem(
            id: "file-1",
            name: "readme.md",
            path: "/readme.md",
            isDirectory: false,
            sizeBytes: 500,
            modifiedAt: Date()
        )

        XCTAssertTrue(directory.isDirectory)
        XCTAssertFalse(file.isDirectory)
    }

    func testFileItem_Equality() {
        let item1 = FileItem(
            id: "file-123",
            name: "test.txt",
            path: "/test.txt",
            isDirectory: false,
            sizeBytes: 1024,
            modifiedAt: Date()
        )

        let item2 = FileItem(
            id: "file-123",
            name: "different.txt",
            path: "/different.txt",
            isDirectory: true,
            sizeBytes: 2048,
            modifiedAt: Date()
        )

        // Equality is based on ID only
        XCTAssertEqual(item1, item2)
    }

    func testFileItem_Hashable() {
        let item1 = FileItem(
            id: "file-123",
            name: "test.txt",
            path: "/test.txt",
            isDirectory: false,
            sizeBytes: 1024,
            modifiedAt: Date()
        )

        let item2 = FileItem(
            id: "file-123",
            name: "different.txt",
            path: "/different.txt",
            isDirectory: true,
            sizeBytes: 2048,
            modifiedAt: Date()
        )

        // Same ID = same hash
        XCTAssertEqual(item1.hashValue, item2.hashValue)
    }

    func testFileItem_Identifiable() {
        let item = FileItem(
            id: "unique-id-456",
            name: "test.txt",
            path: "/test.txt",
            isDirectory: false,
            sizeBytes: 0,
            modifiedAt: Date()
        )

        XCTAssertEqual(item.id, "unique-id-456")
    }

    // MARK: - DocumentItem Tests

    func testDocumentItem_Initialization() {
        let created = Date()
        let modified = Date()
        let doc = DocumentItem(
            id: "doc-123",
            name: "Meeting Notes",
            content: "# Meeting Notes\n\nDiscussion points...",
            createdAt: created,
            modifiedAt: modified,
            authorFourWords: "ocean-forest-moon-star"
        )

        XCTAssertEqual(doc.id, "doc-123")
        XCTAssertEqual(doc.name, "Meeting Notes")
        XCTAssertEqual(doc.content, "# Meeting Notes\n\nDiscussion points...")
        XCTAssertEqual(doc.createdAt, created)
        XCTAssertEqual(doc.modifiedAt, modified)
        XCTAssertEqual(doc.authorFourWords, "ocean-forest-moon-star")
    }

    func testDocumentItem_Mutability() {
        var doc = DocumentItem(
            id: "doc-1",
            name: "Draft",
            content: "Initial content",
            createdAt: Date(),
            modifiedAt: Date(),
            authorFourWords: "test-words"
        )

        doc.name = "Updated Title"
        doc.content = "Updated content"
        doc.modifiedAt = Date()

        XCTAssertEqual(doc.name, "Updated Title")
        XCTAssertEqual(doc.content, "Updated content")
    }

    func testDocumentItem_Equality() {
        let doc1 = DocumentItem(
            id: "doc-123",
            name: "Title 1",
            content: "Content 1",
            createdAt: Date(),
            modifiedAt: Date(),
            authorFourWords: "author-1"
        )

        let doc2 = DocumentItem(
            id: "doc-123",
            name: "Title 2",
            content: "Content 2",
            createdAt: Date(),
            modifiedAt: Date(),
            authorFourWords: "author-2"
        )

        // Equality is based on ID only
        XCTAssertEqual(doc1, doc2)
    }

    func testDocumentItem_Hashable() {
        let doc1 = DocumentItem(
            id: "doc-123",
            name: "Title 1",
            content: "Content 1",
            createdAt: Date(),
            modifiedAt: Date(),
            authorFourWords: "author-1"
        )

        let doc2 = DocumentItem(
            id: "doc-123",
            name: "Different",
            content: "Different",
            createdAt: Date(),
            modifiedAt: Date(),
            authorFourWords: "different"
        )

        XCTAssertEqual(doc1.hashValue, doc2.hashValue)
    }

    func testDocumentItem_Codable() throws {
        let original = DocumentItem(
            id: "doc-codable",
            name: "Codable Test",
            content: "Test content",
            createdAt: Date(timeIntervalSince1970: 1000000),
            modifiedAt: Date(timeIntervalSince1970: 1000100),
            authorFourWords: "test-author"
        )

        let encoder = JSONEncoder()
        let data = try encoder.encode(original)

        let decoder = JSONDecoder()
        let decoded = try decoder.decode(DocumentItem.self, from: data)

        XCTAssertEqual(decoded.id, original.id)
        XCTAssertEqual(decoded.name, original.name)
        XCTAssertEqual(decoded.content, original.content)
        XCTAssertEqual(decoded.authorFourWords, original.authorFourWords)
    }

    // MARK: - ContactItem Tests

    func testContactItem_Initialization() {
        let lastSeen = Date()
        let contact = ContactItem(
            id: "contact-123",
            fourWords: "bear-wolf-swift-dragon",
            displayName: "Bob",
            isFavourite: true,
            isOnline: true,
            lastSeen: lastSeen
        )

        XCTAssertEqual(contact.id, "contact-123")
        XCTAssertEqual(contact.fourWords, "bear-wolf-swift-dragon")
        XCTAssertEqual(contact.displayName, "Bob")
        XCTAssertTrue(contact.isFavourite)
        XCTAssertTrue(contact.isOnline)
        XCTAssertEqual(contact.lastSeen, lastSeen)
    }

    func testContactItem_NilDisplayName() {
        let contact = ContactItem(
            id: "contact-456",
            fourWords: "ocean-forest-moon-star",
            displayName: nil,
            isFavourite: false,
            isOnline: false,
            lastSeen: nil
        )

        XCTAssertNil(contact.displayName)
        XCTAssertNil(contact.lastSeen)
    }

    func testContactItem_Mutability() {
        var contact = ContactItem(
            id: "contact-1",
            fourWords: "test-words",
            displayName: nil,
            isFavourite: false,
            isOnline: false,
            lastSeen: nil
        )

        contact.displayName = "New Name"
        contact.isFavourite = true
        contact.isOnline = true
        contact.lastSeen = Date()

        XCTAssertEqual(contact.displayName, "New Name")
        XCTAssertTrue(contact.isFavourite)
        XCTAssertTrue(contact.isOnline)
        XCTAssertNotNil(contact.lastSeen)
    }

    func testContactItem_Equality() {
        let contact1 = ContactItem(
            id: "contact-123",
            fourWords: "four-words-1",
            displayName: "Alice",
            isFavourite: true,
            isOnline: true,
            lastSeen: Date()
        )

        let contact2 = ContactItem(
            id: "contact-123",
            fourWords: "different-words",
            displayName: "Bob",
            isFavourite: false,
            isOnline: false,
            lastSeen: nil
        )

        // Equality is based on ID only
        XCTAssertEqual(contact1, contact2)
    }

    func testContactItem_Hashable() {
        let contact1 = ContactItem(
            id: "contact-123",
            fourWords: "words-1",
            displayName: "A",
            isFavourite: true,
            isOnline: true,
            lastSeen: Date()
        )

        let contact2 = ContactItem(
            id: "contact-123",
            fourWords: "words-2",
            displayName: "B",
            isFavourite: false,
            isOnline: false,
            lastSeen: nil
        )

        XCTAssertEqual(contact1.hashValue, contact2.hashValue)
    }

    func testContactItem_Codable() throws {
        let original = ContactItem(
            id: "contact-codable",
            fourWords: "test-four-words-here",
            displayName: "Test User",
            isFavourite: true,
            isOnline: false,
            lastSeen: Date(timeIntervalSince1970: 1000000)
        )

        let encoder = JSONEncoder()
        let data = try encoder.encode(original)

        let decoder = JSONDecoder()
        let decoded = try decoder.decode(ContactItem.self, from: data)

        XCTAssertEqual(decoded.id, original.id)
        XCTAssertEqual(decoded.fourWords, original.fourWords)
        XCTAssertEqual(decoded.displayName, original.displayName)
        XCTAssertEqual(decoded.isFavourite, original.isFavourite)
        XCTAssertEqual(decoded.isOnline, original.isOnline)
    }

    // MARK: - NetworkConfig Tests

    func testNetworkConfig_DefaultValues() {
        XCTAssertEqual(NetworkConfig.defaultBootstrapAddress, "138.197.29.195:4433")
        XCTAssertEqual(NetworkConfig.defaultPort, 0)
    }

    // MARK: - Set Membership Tests

    func testFileItem_SetMembership() {
        let item1 = FileItem(id: "1", name: "a", path: "/a", isDirectory: false, sizeBytes: 0, modifiedAt: Date())
        let item2 = FileItem(id: "2", name: "b", path: "/b", isDirectory: false, sizeBytes: 0, modifiedAt: Date())
        let item3 = FileItem(id: "1", name: "c", path: "/c", isDirectory: true, sizeBytes: 100, modifiedAt: Date())

        let set: Set<FileItem> = [item1, item2]

        XCTAssertEqual(set.count, 2)
        XCTAssertTrue(set.contains(item1))
        XCTAssertTrue(set.contains(item3)) // Same ID as item1
    }

    func testDocumentItem_SetMembership() {
        let doc1 = DocumentItem(id: "1", name: "A", content: "", createdAt: Date(), modifiedAt: Date(), authorFourWords: "a")
        let doc2 = DocumentItem(id: "2", name: "B", content: "", createdAt: Date(), modifiedAt: Date(), authorFourWords: "b")
        let doc3 = DocumentItem(id: "1", name: "C", content: "x", createdAt: Date(), modifiedAt: Date(), authorFourWords: "c")

        let set: Set<DocumentItem> = [doc1, doc2]

        XCTAssertEqual(set.count, 2)
        XCTAssertTrue(set.contains(doc3)) // Same ID as doc1
    }

    func testContactItem_SetMembership() {
        let c1 = ContactItem(id: "1", fourWords: "a", displayName: nil, isFavourite: false, isOnline: false, lastSeen: nil)
        let c2 = ContactItem(id: "2", fourWords: "b", displayName: nil, isFavourite: false, isOnline: false, lastSeen: nil)
        let c3 = ContactItem(id: "1", fourWords: "c", displayName: "X", isFavourite: true, isOnline: true, lastSeen: Date())

        let set: Set<ContactItem> = [c1, c2]

        XCTAssertEqual(set.count, 2)
        XCTAssertTrue(set.contains(c3)) // Same ID as c1
    }
}
