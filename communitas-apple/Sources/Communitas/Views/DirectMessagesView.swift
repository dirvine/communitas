import SwiftUI
import X0xClient

/// Direct messages view that wraps DirectMessageView with full conversation UI.
struct DirectMessagesView: View {
    @EnvironmentObject var appState: AppState

    var body: some View {
        DirectMessageView()
            .navigationTitle("Direct Messages")
    }
}
