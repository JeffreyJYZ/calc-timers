import UIKit
import WebKit
import Tauri

@main
@objc class AppDelegate: UIResponder, UIApplicationDelegate {
  var window: UIWindow?

  func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    let window = UIWindow(frame: UIScreen.main.bounds)
    let tauri = Tauri.shared
    tauri.setup()
    window.rootViewController = tauri.rootViewController
    self.window = window
    window.makeKeyAndVisible()
    return true
  }
}
