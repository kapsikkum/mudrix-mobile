// Mudrix Mobile, iOS: keep the app (and its web view's Matrix sync) running in the background, the
// way Android's keep-alive foreground service does. See src/lib.rs for the overview.
//
// Needs `UIBackgroundModes` audio and location in Info.plist (src-tauri/Info.plist).

import AVFoundation
import CoreLocation
import Tauri
import UIKit
import UserNotifications
import WebKit

/// How the app stays alive.
enum KeepAliveStrategy: String {
  case audio
  case location

  static func parse(_ raw: String?) -> KeepAliveStrategy {
    return raw.flatMap(KeepAliveStrategy.init(rawValue:)) ?? .audio
  }
}

/// A silent, looping `AVAudioSession`. `.playback` keeps the process alive in the background and
/// `.mixWithOthers` means music or podcasts from other apps are not interrupted. No permission prompt.
final class SilentAudioKeeper: NSObject {
  private var player: AVAudioPlayer?
  private var wanted = false
  private var watchdog: Timer?

  func start() {
    wanted = true
    observe()
    activateAndPlay()
    if watchdog == nil {
      // Cheap safety net for anything the notifications below miss; runs while the app is alive.
      let timer = Timer(timeInterval: 30, repeats: true) { [weak self] _ in self?.activateAndPlay() }
      RunLoop.main.add(timer, forMode: .common)
      watchdog = timer
    }
  }

  func stop() {
    let wasWanted = wanted
    wanted = false
    NotificationCenter.default.removeObserver(self)
    watchdog?.invalidate()
    watchdog = nil
    player?.stop()
    player = nil
    if wasWanted {
      // Let other apps' audio go back to normal.
      try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
    }
  }

  private func observe() {
    let center = NotificationCenter.default
    center.removeObserver(self)
    let session = AVAudioSession.sharedInstance()
    center.addObserver(
      self, selector: #selector(interruption(_:)),
      name: AVAudioSession.interruptionNotification, object: session)
    center.addObserver(
      self, selector: #selector(routeChange(_:)),
      name: AVAudioSession.routeChangeNotification, object: session)
    center.addObserver(
      self, selector: #selector(mediaServicesReset(_:)),
      name: AVAudioSession.mediaServicesWereResetNotification, object: session)
    center.addObserver(
      self, selector: #selector(appBecameActive(_:)),
      name: UIApplication.didBecomeActiveNotification, object: nil)
  }

  @objc private func interruption(_ notification: Notification) {
    guard wanted,
      let raw = notification.userInfo?[AVAudioSessionInterruptionTypeKey] as? UInt,
      let type = AVAudioSession.InterruptionType(rawValue: raw)
    else { return }
    // On `.began` the system has already paused the player. On `.ended` start again, whether or not
    // it says we should resume: this session exists only to stay alive.
    if type == .ended { activateAndPlay() }
  }

  @objc private func routeChange(_ notification: Notification) {
    guard wanted else { return }
    activateAndPlay()
  }

  @objc private func mediaServicesReset(_ notification: Notification) {
    guard wanted else { return }
    // All audio objects are invalid after a media services reset.
    player = nil
    activateAndPlay()
  }

  @objc private func appBecameActive(_ notification: Notification) {
    guard wanted else { return }
    activateAndPlay()
  }

  private func activateAndPlay() {
    guard wanted else { return }
    let session = AVAudioSession.sharedInstance()
    do {
      try session.setCategory(.playback, mode: .default, options: [.mixWithOthers])
      try session.setActive(true)
    } catch {
      // Another app (a phone call, say) may hold the session; the next event or tick retries.
      return
    }
    if player == nil { player = makePlayer() }
    guard let player = player else { return }
    if !player.isPlaying { player.play() }
  }

  private func makePlayer() -> AVAudioPlayer? {
    guard let player = try? AVAudioPlayer(data: SilentAudioKeeper.silentWav()) else { return nil }
    player.numberOfLoops = -1
    // Not zero: a muted player can be treated as not playing. The data is silent anyway.
    player.volume = 0.01
    player.prepareToPlay()
    return player
  }

  /// One second of 8 kHz, 16-bit mono PCM in a WAV container. The samples alternate between 0 and 1
  /// (one least significant bit, about -90 dBFS and inaudible) so the stream is not literal digital
  /// zero.
  private static func silentWav() -> Data {
    let sampleRate: UInt32 = 8000
    let samples = Int(sampleRate)
    let dataSize = UInt32(samples * 2)
    var wav = Data()
    func append32(_ v: UInt32) {
      var le = v.littleEndian
      wav.append(Data(bytes: &le, count: 4))
    }
    func append16(_ v: UInt16) {
      var le = v.littleEndian
      wav.append(Data(bytes: &le, count: 2))
    }
    wav.append(contentsOf: Array("RIFF".utf8))
    append32(36 + dataSize)
    wav.append(contentsOf: Array("WAVE".utf8))
    wav.append(contentsOf: Array("fmt ".utf8))
    append32(16)
    append16(1)  // PCM
    append16(1)  // mono
    append32(sampleRate)
    append32(sampleRate * 2)
    append16(2)  // block align
    append16(16)  // bits per sample
    wav.append(contentsOf: Array("data".utf8))
    append32(dataSize)
    for i in 0..<samples { append16(i % 2 == 0 ? 0 : 1) }
    return wav
  }
}

/// Coarse, low-power location updates, only to keep the app running in the background. The location
/// itself is ignored.
final class LocationKeeper: NSObject, CLLocationManagerDelegate {
  private var manager: CLLocationManager?
  private var wanted = false
  private var askedForAlways = false

  func start() {
    wanted = true
    askedForAlways = false
    let manager = self.manager ?? CLLocationManager()
    self.manager = manager
    manager.delegate = self
    manager.desiredAccuracy = kCLLocationAccuracyThreeKilometers
    manager.distanceFilter = 5000
    manager.pausesLocationUpdatesAutomatically = false
    manager.activityType = .other
    manager.showsBackgroundLocationIndicator = false
    advance()
  }

  func stop() {
    wanted = false
    manager?.stopUpdatingLocation()
    manager?.allowsBackgroundLocationUpdates = false
    manager?.delegate = nil
    manager = nil
  }

  private var status: CLAuthorizationStatus {
    if #available(iOS 14.0, *), let manager = manager { return manager.authorizationStatus }
    return CLLocationManager.authorizationStatus()
  }

  /// Takes the next permission step, or starts updating when allowed.
  private func advance() {
    guard wanted, let manager = manager else { return }
    switch status {
    case .notDetermined:
      manager.requestWhenInUseAuthorization()
    case .authorizedWhenInUse:
      // Background updates work with when-in-use too (iOS shows its blue indicator); ask once for
      // always so they carry on without it.
      begin()
      if !askedForAlways {
        askedForAlways = true
        manager.requestAlwaysAuthorization()
      }
    case .authorizedAlways:
      begin()
    default:
      break  // denied or restricted: nothing to do until the user changes it in Settings
    }
  }

  private func begin() {
    guard let manager = manager else { return }
    manager.allowsBackgroundLocationUpdates = true
    manager.startUpdatingLocation()
  }

  @available(iOS 14.0, *)
  func locationManagerDidChangeAuthorization(_ manager: CLLocationManager) {
    advance()
  }

  // iOS 13 only; newer systems call locationManagerDidChangeAuthorization instead.
  func locationManager(_ manager: CLLocationManager, didChangeAuthorization status: CLAuthorizationStatus) {
    if #available(iOS 14.0, *) { return }
    advance()
  }

  func locationManager(_ manager: CLLocationManager, didUpdateLocations locations: [CLLocation]) {}

  func locationManager(_ manager: CLLocationManager, didFailWithError error: Error) {
    // Transient errors (no fix yet) are expected; updates keep running.
  }
}

struct SetKeepAliveArgs: Decodable {
  let enabled: Bool
  let strategy: String?
}

class IosKeepAlivePlugin: Plugin {
  private static let enabledKey = "chat.mudrix.keepalive.enabled"
  private static let strategyKey = "chat.mudrix.keepalive.strategy"

  private let audio = SilentAudioKeeper()
  private let location = LocationKeeper()
  private var resumed = false

  private var defaults: UserDefaults { UserDefaults.standard }

  private var enabled: Bool { defaults.bool(forKey: IosKeepAlivePlugin.enabledKey) }
  private var strategy: KeepAliveStrategy {
    KeepAliveStrategy.parse(defaults.string(forKey: IosKeepAlivePlugin.strategyKey))
  }

  /// Called when the web view is created, which is every app launch: resume what was left on.
  override func load(webview: WKWebView) {
    if resumed { return }
    resumed = true
    if enabled {
      DispatchQueue.main.async { self.apply(enabled: true, strategy: self.strategy) }
    }
  }

  /// Starts the chosen strategy and stops the other one. Main thread only.
  private func apply(enabled: Bool, strategy: KeepAliveStrategy) {
    if enabled {
      switch strategy {
      case .audio:
        location.stop()
        audio.start()
      case .location:
        audio.stop()
        location.start()
      }
    } else {
      audio.stop()
      location.stop()
    }
  }

  /// Whether local notifications are allowed; with `ask`, asks first if the user was never asked.
  private func notificationsAllowed(ask: Bool, _ completion: @escaping (Bool) -> Void) {
    let center = UNUserNotificationCenter.current()
    center.getNotificationSettings { settings in
      let status = settings.authorizationStatus
      if status == .authorized || status == .provisional {
        completion(true)
      } else if #available(iOS 14.0, *), status == .ephemeral {
        completion(true)
      } else if status == .notDetermined && ask {
        center.requestAuthorization(options: [.alert, .sound, .badge]) { granted, _ in
          completion(granted)
        }
      } else {
        completion(false)
      }
    }
  }

  @objc public func status(_ invoke: Invoke) {
    notificationsAllowed(ask: false) { allowed in
      invoke.resolve([
        "notificationsAllowed": allowed,
        "keepAlive": self.enabled,
        "keepAliveStrategy": self.strategy.rawValue,
      ])
    }
  }

  @objc public func setKeepAlive(_ invoke: Invoke) {
    let args: SetKeepAliveArgs
    do {
      args = try invoke.parseArgs(SetKeepAliveArgs.self)
    } catch {
      invoke.reject("invalid arguments: \(error.localizedDescription)")
      return
    }
    let strategy = KeepAliveStrategy.parse(args.strategy)
    defaults.set(args.enabled, forKey: IosKeepAlivePlugin.enabledKey)
    defaults.set(strategy.rawValue, forKey: IosKeepAlivePlugin.strategyKey)
    DispatchQueue.main.async {
      self.apply(enabled: args.enabled, strategy: strategy)
      if args.enabled {
        // Fire and forget: the prompt must not hold up the reply. `status` reports the outcome.
        self.notificationsAllowed(ask: true) { _ in }
      }
      invoke.resolve([
        "keepAlive": args.enabled,
        "keepAliveStrategy": strategy.rawValue,
      ])
    }
  }
}

@_cdecl("init_plugin_ios_keepalive")
public func initPlugin() -> Plugin {
  return IosKeepAlivePlugin()
}
