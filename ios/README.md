# iOS / macOS support

This folder contains the iOS Xcode project files. Building iOS requires a Mac
with Xcode 15+ and the canary Tauri CLI (the iOS subcommand is not in the
stable release yet).

## One-time setup on a Mac

```bash
# 1. Install canary Tauri CLI
cargo install tauri-cli --git https://github.com/tauri-apps/tauri --locked --force

# 2. Generate the Tauri iOS scaffolding (this populates src-tauri/gen/apple)
pnpm tauri ios init

# 3. Generate the Xcode project from project.yml (requires XcodeGen)
brew install xcodegen
cd ios && xcodegen generate && cd ..

# 4. Install CocoaPods deps
cd ios && pod install && cd ..
```

## Build & run (iOS Simulator)

```bash
pnpm tauri ios dev
```

## Build & run (real device)

Open `ios/CalcTimers.xcworkspace` in Xcode, set your development team, and Run.

## Build for App Store

```bash
pnpm tauri ios build
```

## macOS

No extra setup. `pnpm tauri build` on a Mac produces a `.app` bundle and `.dmg`
in `src-tauri/target/release/bundle/`.

## Notes

- Bundle identifier: `com.timedcalc.app`
- Deployment target: iOS 13.0
- Background mode `audio` is enabled so timer audio can finish playing when the
  app is backgrounded.
- The `mobile` capability (`src-tauri/capabilities/mobile.json`) is loaded for
  iOS/Android targets; the `default` capability handles desktop.
