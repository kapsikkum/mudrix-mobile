package chat.mudrix.mobile

import android.Manifest
import android.app.Activity
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import org.unifiedpush.android.connector.UnifiedPush

@InvokeArg
class RegisterArgs {
  var distributor: String? = null
}

/** Exposes UnifiedPush registration to the web client (through the Rust `push_*` commands). */
@TauriPlugin
class PushPlugin(private val activity: Activity) : Plugin(activity) {
  private val context get() = activity.applicationContext

  private fun notificationsAllowed(): Boolean =
    Build.VERSION.SDK_INT < 33 ||
      ContextCompat.checkSelfPermission(context, Manifest.permission.POST_NOTIFICATIONS) ==
      PackageManager.PERMISSION_GRANTED

  @Command
  fun status(invoke: Invoke) {
    val result = JSObject()
    result.put("supported", true)
    result.put("distributors", JSArray(UnifiedPush.getDistributors(context)))
    result.put("distributor", UnifiedPush.getSavedDistributor(context))
    result.put("endpoint", PushStore.endpoint(context))
    result.put("error", PushStore.error(context))
    result.put("notificationsAllowed", notificationsAllowed())
    invoke.resolve(result)
  }

  @Command
  fun register(invoke: Invoke) {
    val args = invoke.parseArgs(RegisterArgs::class.java)
    if (!notificationsAllowed()) {
      ActivityCompat.requestPermissions(activity, arrayOf(Manifest.permission.POST_NOTIFICATIONS), 4711)
    }
    val distributor = args.distributor ?: UnifiedPush.getDistributors(context).firstOrNull()
    if (distributor == null) {
      invoke.reject("No UnifiedPush distributor installed")
      return
    }
    PushStore.setError(context, null)
    UnifiedPush.saveDistributor(context, distributor)
    UnifiedPush.register(context)
    invoke.resolve()
  }

  @Command
  fun unregister(invoke: Invoke) {
    UnifiedPush.unregister(context)
    PushStore.setEndpoint(context, null)
    invoke.resolve()
  }
}
