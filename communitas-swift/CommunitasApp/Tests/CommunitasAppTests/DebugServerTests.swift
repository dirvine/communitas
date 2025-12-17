import XCTest
@testable import CommunitasAppLib

/// Tests for DebugServer
/// Note: Network-level tests are limited to avoid port conflicts and flaky behavior.
/// These tests focus on the testable, non-network aspects of DebugServer.
@MainActor
final class DebugServerTests: XCTestCase {

    // MARK: - Test Fixtures

    var server: DebugServer!

    override func setUp() async throws {
        try await super.setUp()
        server = DebugServer(forTesting: true)
    }

    override func tearDown() async throws {
        server = nil
        try await super.tearDown()
    }

    // MARK: - Initialization Tests

    func testDefaultPort() {
        XCTAssertEqual(server.port, 9999)
    }

    func testTestingInitializer() {
        // Verify that testing initializer creates a valid instance
        XCTAssertNotNil(server)
    }

    // MARK: - HTTP Status Text Tests

    func testHttpStatusText_200() {
        XCTAssertEqual(server.httpStatusText(200), "OK")
    }

    func testHttpStatusText_204() {
        XCTAssertEqual(server.httpStatusText(204), "No Content")
    }

    func testHttpStatusText_400() {
        XCTAssertEqual(server.httpStatusText(400), "Bad Request")
    }

    func testHttpStatusText_404() {
        XCTAssertEqual(server.httpStatusText(404), "Not Found")
    }

    func testHttpStatusText_500() {
        XCTAssertEqual(server.httpStatusText(500), "Internal Server Error")
    }

    func testHttpStatusText_Unknown() {
        XCTAssertEqual(server.httpStatusText(418), "Unknown")
        XCTAssertEqual(server.httpStatusText(999), "Unknown")
        XCTAssertEqual(server.httpStatusText(0), "Unknown")
    }

    // MARK: - Handler Registration Tests

    func testRegisterHandler() async throws {
        var handlerCalled = false

        server.registerHandler("testAction") { _ in
            handlerCalled = true
            return Data()
        }

        // Handler registration doesn't call the handler
        XCTAssertFalse(handlerCalled)
    }

    func testRegisterMultipleHandlers() {
        server.registerHandler("action1") { _ in Data() }
        server.registerHandler("action2") { _ in Data() }
        server.registerHandler("action3") { _ in Data() }

        // Should not crash when registering multiple handlers
        XCTAssertNotNil(server)
    }

    func testOverwriteHandler() {
        var firstHandlerCalled = false
        var secondHandlerCalled = false

        server.registerHandler("sameAction") { _ in
            firstHandlerCalled = true
            return Data()
        }

        server.registerHandler("sameAction") { _ in
            secondHandlerCalled = true
            return Data()
        }

        // Registration should succeed without error
        XCTAssertFalse(firstHandlerCalled)
        XCTAssertFalse(secondHandlerCalled)
    }

    // MARK: - Server Lifecycle Tests (without actual network)

    func testStopBeforeStart() {
        // Stopping without starting should not crash
        server.stop()
        XCTAssertNotNil(server)
    }

    func testMultipleStops() {
        // Multiple stops should not crash
        server.stop()
        server.stop()
        server.stop()
        XCTAssertNotNil(server)
    }

    // MARK: - Port Configuration Tests

    func testPortValueType() {
        // Port should be a valid UInt16
        let port = server.port
        XCTAssertTrue(port > 0)
        XCTAssertTrue(port <= UInt16.max)
    }

    func testDefaultPortValue() {
        // Default port should be 9999
        XCTAssertEqual(server.port, 9999)
    }

    // MARK: - HTTP Status Codes Coverage

    func testAllDefinedStatusCodes() {
        // Test all status codes that have defined text
        let definedCodes: [Int: String] = [
            200: "OK",
            204: "No Content",
            400: "Bad Request",
            404: "Not Found",
            500: "Internal Server Error"
        ]

        for (code, expectedText) in definedCodes {
            XCTAssertEqual(
                server.httpStatusText(code),
                expectedText,
                "Status code \(code) should return '\(expectedText)'"
            )
        }
    }

    func testCommonUndefinedStatusCodes() {
        // Test common HTTP status codes that aren't explicitly defined
        let undefinedCodes = [
            100,  // Continue
            201,  // Created
            301,  // Moved Permanently
            302,  // Found
            401,  // Unauthorized
            403,  // Forbidden
            405,  // Method Not Allowed
            502,  // Bad Gateway
            503   // Service Unavailable
        ]

        for code in undefinedCodes {
            XCTAssertEqual(
                server.httpStatusText(code),
                "Unknown",
                "Undefined status code \(code) should return 'Unknown'"
            )
        }
    }

    // MARK: - Edge Case Tests

    func testNegativeStatusCode() {
        // Negative codes should return Unknown
        XCTAssertEqual(server.httpStatusText(-1), "Unknown")
        XCTAssertEqual(server.httpStatusText(-500), "Unknown")
    }

    func testLargeStatusCode() {
        // Large codes should return Unknown
        XCTAssertEqual(server.httpStatusText(10000), "Unknown")
        XCTAssertEqual(server.httpStatusText(Int.max), "Unknown")
    }
}
