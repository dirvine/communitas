import AppKit
import UserNotifications

/// Manages dock badge counts and native macOS notifications with actionable categories.
final class NotificationService: NSObject, UNUserNotificationCenterDelegate {
    static let shared = NotificationService()

    /// Notification category identifiers.
    static let messageCategoryId = "MESSAGE"
    /// Action identifiers.
    static let replyActionId = "REPLY_ACTION"
    static let markReadActionId = "MARK_READ_ACTION"
    /// UserInfo keys for routing notification taps and replies.
    static let groupIdKey = "groupId"
    static let channelNameKey = "channelName"
    static let agentIdKey = "agentId"

    /// Called when the user taps a notification or performs an inline reply.
    /// Set by AppState to route interactions back to the messaging layer.
    var onReply: ((_ groupId: String, _ channelName: String, _ text: String) -> Void)?
    var onTap: ((_ groupId: String, _ channelName: String) -> Void)?

    private var unreadCount = 0

    private override init() {
        super.init()
    }

    // MARK: - Badge Management

    func incrementUnread() {
        unreadCount += 1
        updateBadge()
    }

    func resetUnread() {
        unreadCount = 0
        updateBadge()
    }

    func setUnreadCount(_ count: Int) {
        unreadCount = count
        updateBadge()
    }

    private func updateBadge() {
        DispatchQueue.main.async {
            NSApp.dockTile.badgeLabel = self.unreadCount > 0 ? "\(self.unreadCount)" : nil
        }
    }

    // MARK: - Bundle Check

    /// UNUserNotificationCenter requires a proper app bundle.
    /// When running as a bare SPM executable, skip notification APIs.
    private var hasBundleIdentifier: Bool {
        Bundle.main.bundleIdentifier != nil
    }

    // MARK: - Notification Permissions & Categories

    func requestPermission() {
        guard hasBundleIdentifier else { return }

        // Register actionable notification categories
        let replyAction = UNTextInputNotificationAction(
            identifier: Self.replyActionId,
            title: "Reply",
            textInputButtonTitle: "Send",
            textInputPlaceholder: "Type a reply..."
        )
        let markReadAction = UNNotificationAction(
            identifier: Self.markReadActionId,
            title: "Mark Read"
        )
        let messageCategory = UNNotificationCategory(
            identifier: Self.messageCategoryId,
            actions: [replyAction, markReadAction],
            intentIdentifiers: []
        )

        let center = UNUserNotificationCenter.current()
        center.setNotificationCategories([messageCategory])
        center.delegate = self
        center.requestAuthorization(options: [.alert, .badge, .sound]) { _, _ in }
    }

    // MARK: - Send Notifications

    func sendNotification(title: String, body: String, categoryIdentifier: String? = nil, userInfo: [String: String] = [:]) {
        guard hasBundleIdentifier else { return }
        let content = UNMutableNotificationContent()
        content.title = title
        content.body = body
        content.sound = .default
        if let category = categoryIdentifier {
            content.categoryIdentifier = category
        }
        if !userInfo.isEmpty {
            content.userInfo = userInfo
        }

        let request = UNNotificationRequest(
            identifier: UUID().uuidString,
            content: content,
            trigger: nil
        )
        UNUserNotificationCenter.current().add(request)
    }

    func sendMessageNotification(from sender: String, in channel: String, preview: String, groupId: String? = nil) {
        let truncated = preview.count > 100 ? String(preview.prefix(100)) + "..." : preview
        var userInfo: [String: String] = [Self.channelNameKey: channel]
        if let groupId {
            userInfo[Self.groupIdKey] = groupId
        }
        sendNotification(
            title: "\(sender) in #\(channel)",
            body: truncated,
            categoryIdentifier: Self.messageCategoryId,
            userInfo: userInfo
        )
    }

    // MARK: - UNUserNotificationCenterDelegate

    /// Handle notification taps (foreground and background).
    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        didReceive response: UNNotificationResponse,
        withCompletionHandler completionHandler: @escaping () -> Void
    ) {
        let userInfo = response.notification.request.content.userInfo
        let groupId = userInfo[Self.groupIdKey] as? String ?? ""
        let channelName = userInfo[Self.channelNameKey] as? String ?? ""

        switch response.actionIdentifier {
        case Self.replyActionId:
            if let textResponse = response as? UNTextInputNotificationResponse {
                onReply?(groupId, channelName, textResponse.userText)
            }
        case Self.markReadActionId:
            // Mark read is handled by simply not incrementing unread
            break
        case UNNotificationDefaultActionIdentifier:
            // User tapped the notification — navigate to the conversation
            onTap?(groupId, channelName)
        default:
            break
        }
        completionHandler()
    }

    /// Show notifications even when app is in foreground.
    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification,
        withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
    ) {
        completionHandler([.banner, .sound])
    }
}
