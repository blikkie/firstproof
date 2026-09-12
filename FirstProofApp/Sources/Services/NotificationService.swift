import Foundation
import UserNotifications

/// Handles native macOS UserNotification Center banners for background batch export completion.
public final class NotificationService {
    public static let shared = NotificationService()
    
    private init() {}
    
    /// Requests user authorization for notification banners and sounds.
    public func requestAuthorization() {
        UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound]) { granted, error in
            if let error = error {
                print("Notification authorization error: \(error)")
            }
        }
    }
    
    /// Posts a native macOS Notification Center banner when a batch export completes.
    public func postBatchExportCompletedNotification(count: Int, destination: String) {
        let content = UNMutableNotificationContent()
        content.title = "Batch Export Complete"
        let folderName = URL(fileURLWithPath: destination).lastPathComponent
        content.body = "Successfully exported \(count) photo\(count == 1 ? "" : "s") to \(folderName)."
        content.sound = .default
        
        let trigger = UNTimeIntervalNotificationTrigger(timeInterval: 0.1, repeats: false)
        let request = UNNotificationRequest(identifier: UUID().uuidString, content: content, trigger: trigger)
        UNUserNotificationCenter.current().add(request) { error in
            if let error = error {
                print("Failed to schedule export notification: \(error)")
            }
        }
    }
}
