package chat.mudrix.mobile

import android.graphics.Canvas
import android.graphics.Color
import android.graphics.ColorFilter
import android.graphics.Paint
import android.graphics.PixelFormat
import android.graphics.drawable.Drawable
import android.graphics.drawable.ColorDrawable
import android.graphics.Rect
import android.os.Bundle
import android.view.View
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.activity.SystemBarStyle
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  private companion object {
    const val EDGE_DP = 48
    const val EXCLUDE_HEIGHT_DP = 200

    // What the web client last reported. The activity can be recreated while the page stays loaded
    // (and the page only reports a colour when it changes), so the colours are kept here to put
    // back behind the system bars instead of falling to the black default.
    var lastTop: Int? = null
    var lastBottom: Int? = null
    var lastLightTop = false
    var lastLightBottom = false

    // The dark theme's bar colour, for before the page has reported anything.
    const val DEFAULT_BACKGROUND = 0xFF1A1B1E.toInt()
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    // The web client is dark by default; light system-bar icons keep the clock and battery legible.
    enableEdgeToEdge(
      statusBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
      navigationBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
    )
    super.onCreate(savedInstanceState)

    if (PushStore.keepAlive(this)) KeepAliveService.apply(this, true)

    val top = lastTop
    val bottom = lastBottom
    if (top != null && bottom != null) {
      window.setBackgroundDrawable(TwoToneDrawable(top, bottom))
      WindowCompat.getInsetsController(window, window.decorView).apply {
        isAppearanceLightStatusBars = lastLightTop
        isAppearanceLightNavigationBars = lastLightBottom
      }
    } else {
      window.setBackgroundDrawable(ColorDrawable(DEFAULT_BACKGROUND))
    }

    // Keep the web view out of the status bar, the camera cutout, the navigation/gesture bar and the
    // keyboard instead of drawing the app header underneath them.
    val content = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(content) { view, insets ->
      val bars = insets.getInsets(
        WindowInsetsCompat.Type.systemBars() or
          WindowInsetsCompat.Type.displayCutout() or
          WindowInsetsCompat.Type.ime()
      )
      view.setPadding(bars.left, bars.top, bars.right, bars.bottom)
      WindowInsetsCompat.CONSUMED
    }

    // The side panels open by swiping in from the screen edges, which the system back gesture would
    // otherwise take. Android lets an app claim at most 200dp of each edge, so the middle of both
    // edges is kept for the app.
    val decor = window.decorView
    decor.addOnLayoutChangeListener { _, left, top, right, bottom, _, _, _, _ ->
      val density = decor.resources.displayMetrics.density
      val edge = (EDGE_DP * density).toInt()
      val half = (EXCLUDE_HEIGHT_DP * density / 2).toInt()
      val mid = (top + bottom) / 2
      ViewCompat.setSystemGestureExclusionRects(
        decor,
        listOf(
          Rect(left, mid - half, left + edge, mid + half),
          Rect(right - edge, mid - half, right, mid + half),
        )
      )
    }
  }

  override fun onWebViewCreate(webView: WebView) {
    webView.addJavascriptInterface(ChromeBridge(), "MudrixChrome")
  }

  /**
   * The web client reports the colours at the top and bottom of the page. The window shows through
   * the padding left around the web view for the system bars, so painting the top half and the
   * bottom half those colours makes the app reach from the top of the screen to the bottom, and the
   * bar icons follow how light each colour is.
   */
  private inner class ChromeBridge {
    @JavascriptInterface
    fun setBackground(top: String, bottom: String, lightTop: Boolean, lightBottom: Boolean) {
      val topColor: Int
      val bottomColor: Int
      try {
        topColor = Color.parseColor(top)
        bottomColor = Color.parseColor(bottom)
      } catch (_: IllegalArgumentException) {
        return
      }
      lastTop = topColor
      lastBottom = bottomColor
      lastLightTop = lightTop
      lastLightBottom = lightBottom
      runOnUiThread {
        window.setBackgroundDrawable(TwoToneDrawable(topColor, bottomColor))
        WindowCompat.getInsetsController(window, window.decorView).apply {
          isAppearanceLightStatusBars = lightTop
          isAppearanceLightNavigationBars = lightBottom
        }
      }
    }
  }

  /** The top half of the window in one colour and the bottom half in another. */
  private class TwoToneDrawable(private val top: Int, private val bottom: Int) : Drawable() {
    private val paint = Paint()

    override fun draw(canvas: Canvas) {
      val b = bounds
      val mid = (b.top + b.bottom) / 2f
      paint.color = top
      canvas.drawRect(b.left.toFloat(), b.top.toFloat(), b.right.toFloat(), mid, paint)
      paint.color = bottom
      canvas.drawRect(b.left.toFloat(), mid, b.right.toFloat(), b.bottom.toFloat(), paint)
    }

    override fun setAlpha(alpha: Int) {}

    override fun setColorFilter(colorFilter: ColorFilter?) {}

    @Deprecated("Required by Drawable")
    override fun getOpacity(): Int = PixelFormat.OPAQUE
  }
}
