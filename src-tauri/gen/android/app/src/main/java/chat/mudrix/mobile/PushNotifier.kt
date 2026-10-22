package chat.mudrix.mobile

import android.Manifest
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.content.ContextCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.ProcessLifecycleOwner
import org.json.JSONObject

/** Turns a Matrix push (as forwarded by a UnifiedPush gateway) into a system notification. */
object PushNotifier {
  private const val CHANNEL = "messages"

  fun show(context: Context, payload: String) {
    // The web client shows its own notifications while the app is on screen.
    if (ProcessLifecycleOwner.get().lifecycle.currentState.isAtLeast(Lifecycle.State.STARTED)) return
    if (Build.VERSION.SDK_INT >= 33 &&
      ContextCompat.checkSelfPermission(context, Manifest.permission.POST_NOTIFICATIONS) !=
      PackageManager.PERMISSION_GRANTED
    ) return

    var title = "Mudrix"
    var body = "New message"
    var roomId: String? = null
    try {
      val n = JSONObject(payload).optJSONObject("notification")
      if (n != null) {
        roomId = n.optString("room_id").ifEmpty { null }
        val room = n.optString("room_name")
        val sender = n.optString("sender_display_name")
        val text = n.optJSONObject("content")?.optString("body").orEmpty()
        if (room.isNotEmpty()) title = room else if (sender.isNotEmpty()) title = sender
        if (text.isNotEmpty()) body = if (room.isNotEmpty() && sender.isNotEmpty()) "$sender: $text" else text
      }
    } catch (_: Exception) {
      // Not JSON: keep the generic text.
    }

    val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
    if (Build.VERSION.SDK_INT >= 26) {
      manager.createNotificationChannel(
        NotificationChannel(CHANNEL, "Messages", NotificationManager.IMPORTANCE_HIGH)
      )
    }
    val open = Intent(context, MainActivity::class.java).apply {
      flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP
      if (roomId != null) putExtra("roomId", roomId)
    }
    val pending = PendingIntent.getActivity(
      context, 0, open, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
    )
    val notification = NotificationCompat.Builder(context, CHANNEL)
      .setSmallIcon(android.R.drawable.stat_notify_chat)
      .setContentTitle(title)
      .setContentText(body)
      .setAutoCancel(true)
      .setContentIntent(pending)
      .setPriority(NotificationCompat.PRIORITY_HIGH)
      .build()
    manager.notify((roomId ?: payload).hashCode(), notification)
  }
}
