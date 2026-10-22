package chat.mudrix.mobile

import android.content.Context

/** Small persistent state shared between the push service (no UI) and the Tauri plugin. */
object PushStore {
  private const val FILE = "mudrix_push"

  private fun prefs(context: Context) = context.getSharedPreferences(FILE, Context.MODE_PRIVATE)

  fun endpoint(context: Context): String? = prefs(context).getString("endpoint", null)

  fun setEndpoint(context: Context, url: String?) {
    prefs(context).edit().putString("endpoint", url).apply()
  }

  fun error(context: Context): String? = prefs(context).getString("error", null)

  fun setError(context: Context, reason: String?) {
    prefs(context).edit().putString("error", reason).apply()
  }
}
