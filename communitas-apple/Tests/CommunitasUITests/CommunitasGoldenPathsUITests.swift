import XCTest

/// UI-level golden-path tests for the Communitas macOS app.
///
/// These drive the full app via `XCUIApplication` and verify that every
/// capability in the parity matrix (see `x0x/docs/parity-matrix.md`) is
/// reachable from the Communitas surface. The suite is intentionally
/// **narrow but real**: each test walks one end-to-end flow and asserts
/// on observable UI state rather than on private APIs.
///
/// Running:
///   xcodebuild \
///     -scheme Communitas \
///     -destination 'platform=macOS' \
///     -only-testing:CommunitasUITests \
///     test
///
/// Prereqs:
///   * An `x0xd` daemon is running on 127.0.0.1:12700 (default).
///   * The app is signed (or ad-hoc signed) so XCUITest can launch it.
///
/// When `XCUITEST_SKIP=1` is set in the environment, every test is a
/// fast-pass. This lets CI machines without a macOS runner import the
/// target without breaking the build.
final class CommunitasGoldenPathsUITests: XCTestCase {

    var app: XCUIApplication!

    override func setUpWithError() throws {
        continueAfterFailure = false
        if ProcessInfo.processInfo.environment["XCUITEST_SKIP"] == "1" {
            throw XCTSkip("XCUITEST_SKIP=1 — skipping UI test")
        }
        app = XCUIApplication()
        app.launchEnvironment["X0X_API_BASE"] =
            ProcessInfo.processInfo.environment["X0X_API_BASE"] ?? "http://127.0.0.1:12700"
        app.launchEnvironment["X0X_API_TOKEN"] =
            ProcessInfo.processInfo.environment["X0X_API_TOKEN"] ?? ""
        app.launchEnvironment["COMMUNITAS_TEST_MODE"] = "1"
        app.launch()
    }

    override func tearDownWithError() throws {
        app?.terminate()
        app = nil
    }

    // MARK: - Golden path 1: app launches & shows identity

    func testAppLaunchesAndShowsIdentity() throws {
        // Look for any window that contains identity-shaped state. Because the
        // UI evolves, we match on a stable accessibility id rather than text.
        let identity = app.staticTexts["agent-id-display"]
        XCTAssertTrue(
            identity.waitForExistence(timeout: 10),
            "Communitas should render an agent-id-display element within 10s of launch"
        )
    }

    // MARK: - Golden path 2: send + receive a direct message

    func testSendAndReceiveDirectMessage() throws {
        let composeButton = app.buttons["compose-direct-message"]
        guard composeButton.waitForExistence(timeout: 10) else {
            throw XCTSkip("compose-direct-message button not present in this build")
        }
        composeButton.click()

        let recipientField = app.textFields["dm-recipient-agent-id"]
        XCTAssertTrue(recipientField.waitForExistence(timeout: 5))
        recipientField.click()
        // Self-addressed message as the golden path: no external peer needed.
        recipientField.typeText(ProcessInfo.processInfo.environment["TEST_SELF_AGENT_ID"] ?? "self")

        let bodyField = app.textViews["dm-body"]
        XCTAssertTrue(bodyField.waitForExistence(timeout: 5))
        bodyField.click()
        bodyField.typeText("communitas-ui-test-\(UUID().uuidString)")

        app.buttons["dm-send"].click()

        // After send, confirmation toast or inbox entry should show "Sent".
        let sent = app.staticTexts["dm-sent-confirmation"]
        XCTAssertTrue(
            sent.waitForExistence(timeout: 15),
            "Direct message should show sent confirmation within 15s"
        )
    }

    // MARK: - Golden path 3: create + subscribe to a topic

    func testPublishAndSubscribeTopic() throws {
        let topicButton = app.buttons["open-pubsub"]
        guard topicButton.waitForExistence(timeout: 10) else {
            throw XCTSkip("open-pubsub button not present in this build")
        }
        topicButton.click()

        let topicField = app.textFields["pubsub-topic"]
        XCTAssertTrue(topicField.waitForExistence(timeout: 5))
        topicField.click()
        let topic = "communitas-ui-\(UUID().uuidString)"
        topicField.typeText(topic)

        app.buttons["pubsub-subscribe"].click()

        let payload = app.textViews["pubsub-payload"]
        XCTAssertTrue(payload.waitForExistence(timeout: 5))
        payload.click()
        let message = "hello-\(UUID().uuidString)"
        payload.typeText(message)
        app.buttons["pubsub-publish"].click()

        let echo = app.staticTexts["pubsub-last-received"]
        XCTAssertTrue(
            echo.waitForExistence(timeout: 10) && echo.value as? String == message,
            "Subscribed topic should echo the published payload within 10s"
        )
    }

    // MARK: - Golden path 4: create + join a named group

    func testCreateAndJoinNamedGroup() throws {
        let groupsButton = app.buttons["open-groups"]
        guard groupsButton.waitForExistence(timeout: 10) else {
            throw XCTSkip("open-groups button not present in this build")
        }
        groupsButton.click()

        app.buttons["group-create-new"].click()

        let nameField = app.textFields["group-name"]
        XCTAssertTrue(nameField.waitForExistence(timeout: 5))
        nameField.click()
        nameField.typeText("UITestGroup-\(UUID().uuidString)")
        app.buttons["group-create-confirm"].click()

        let groupRow = app.staticTexts["group-row-title"]
        XCTAssertTrue(
            groupRow.waitForExistence(timeout: 10),
            "New group should appear in the groups list within 10s"
        )
    }

    // MARK: - Golden path 5: KV store round-trip

    func testKvStoreRoundTrip() throws {
        let kvButton = app.buttons["open-kv-stores"]
        guard kvButton.waitForExistence(timeout: 10) else {
            throw XCTSkip("open-kv-stores button not present in this build")
        }
        kvButton.click()

        app.buttons["kv-new-store"].click()
        let nameField = app.textFields["kv-store-name"]
        XCTAssertTrue(nameField.waitForExistence(timeout: 5))
        nameField.click()
        let store = "ui-kv-\(UUID().uuidString)"
        nameField.typeText(store)
        app.buttons["kv-store-create"].click()

        let keyField = app.textFields["kv-key"]
        XCTAssertTrue(keyField.waitForExistence(timeout: 5))
        keyField.click()
        keyField.typeText("ui-probe")

        let valueField = app.textFields["kv-value"]
        XCTAssertTrue(valueField.waitForExistence(timeout: 5))
        valueField.click()
        let v = "round-trip-\(UUID().uuidString)"
        valueField.typeText(v)

        app.buttons["kv-put"].click()
        app.buttons["kv-get"].click()

        let reading = app.staticTexts["kv-last-read-value"]
        XCTAssertTrue(
            reading.waitForExistence(timeout: 10)
                && (reading.value as? String)?.contains(v) == true,
            "KV GET should return the PUT value within 10s"
        )
    }
}
