package chat.mudrix.mobile

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import androidx.core.app.ServiceCompat
import androidx.core.content.ContextCompat

/**
 * Optional foreground service for people without a UnifiedPush distributor. It does no work itself:
 * its ongoing notification keeps the process (and so the web client's sync connection) alive while the
 * app is in the background, and the web client raises message notifications as it does on screen.
 */
class KeepAliveService : Service() {
  override fun onBind(intent: Intent?): IBinder? = null

  override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
    val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
    if (Build.VERSION.SDK_INT >= 26) {
      manager.createNotificationChannel(
        NotificationChannel(CHANNEL, "Background connection", NotificationManager.IMPORTANCE_MIN)
      )
    }
    val open = PendingIntent.getActivity(
      this, 0, Intent(this, MainActivity::class.java),
      PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
    )
    val notification: Notification = NotificationCompat.Builder(this, CHANNEL)
      .setSmallIcon(android.R.drawable.stat_notify_sync_noanim)
      .setContentTitle("Mudrix")
      .setContentText("Connected for message notifications")
      .setOngoing(true)
      .setSilent(true)
      .setPriority(NotificationCompat.PRIORITY_MIN)
      .setContentIntent(open)
      .build()
    val type = if (Build.VERSION.SDK_INT >= 34) ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE else 0
    ServiceCompat.startForeground(this, ID, notification, type)
    return START_STICKY
  }

  companion object {
    private const val CHANNEL = "keepalive"
    private const val ID = 4712

    fun apply(context: Context, enabled: Boolean) {
      val intent = Intent(context, KeepAliveService::class.java)
      if (enabled) ContextCompat.startForegroundService(context, intent) else context.stopService(intent)
    }
  }
}
