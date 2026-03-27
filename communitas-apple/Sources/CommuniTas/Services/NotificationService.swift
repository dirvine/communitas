import AppKit
import UserNotifications

/// Manages dock badge counts and native macOS notifications.
final class NotificationService {
    static let shared = NotificationService()

    private var unreadCount = 0

    private init() {}

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

    // MARK: - Notification Permissions

    func requestPermission() {
        UNUserNotificationCenter.current().requestAuthorization(
            options: [.alert, .badge, .sound]
        ) { _, _ in }
    }

    // MARK: - Send Notifications

    func sendNotification(title: String, body: String, categoryIdentifier: String? = nil) {
        let content = UNMutableNotificationContent()
        content.title = title
        content.body = body
        content.sound = .default
        if let category = categoryIdentifier {
            content.categoryIdentifier = category
        }

        let request = UNNotificationRequest(
            identifier: UUID().uuidString,
            content: content,
            trigger: nil
        )
        UNUserNotificationCenter.current().add(request)
    }

    func sendMessageNotification(from sender: String, in channel: String, preview: String) {
        let truncated = preview.count > 100 ? String(preview.prefix(100)) + "..." : preview
        sendNotification(
            title: "\(sender) in #\(channel)",
            body: truncated,
            categoryIdentifier: "MESSAGE"
        )
    }
}
