package chat.mudrix.mobile

import org.unifiedpush.android.connector.FailedReason
import org.unifiedpush.android.connector.PushService
import org.unifiedpush.android.connector.data.PushEndpoint
import org.unifiedpush.android.connector.data.PushMessage

/** Receives UnifiedPush events from the user's distributor (ntfy, etc.). */
class MudrixPushService : PushService() {
  override fun onNewEndpoint(endpoint: PushEndpoint, instance: String) {
    PushStore.setEndpoint(this, endpoint.url)
    PushStore.setError(this, null)
  }

  override fun onMessage(message: PushMessage, instance: String) {
    PushNotifier.show(this, String(message.content, Charsets.UTF_8))
  }

  override fun onRegistrationFailed(reason: FailedReason, instance: String) {
    PushStore.setError(this, reason.name)
  }

  override fun onUnregistered(instance: String) {
    PushStore.setEndpoint(this, null)
  }
}
