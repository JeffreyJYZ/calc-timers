# CalcTimers

A calm, fast desktop & mobile utility built with **Tauri 2 + React 19 + TypeScript + Tailwind CSS 4 + Vite 7**.

Two tools in one app:

- **Calculator** — full precision shunting-yard parser, history, keyboard shortcuts, sign toggle, percent.
- **Multi-Timer** — unlimited concurrent countdowns with progress rings, labels, per-timer sound/vibration/notification toggles, presets, and persistence across restarts.

Warm-cream pastel theme. Flat surfaces, subtle borders, no glow, no gradient soup. Runs natively on Windows, macOS, Linux, Android, iOS, iPadOS.

## Highlights

- Native shell (Tauri 2 webview) — single tiny binary, no Electron.
- Cross-platform: Windows, macOS, Linux (desktop) + Android, iOS, iPadOS (mobile).
- Calculator engine uses shunting-yard (no `eval`); supports `+ − × ÷ % ( )` and unary minus.
- Timers persist with **Zustand + localStorage**, mirrored to a Tauri store file for reliability.
- Sound, vibration, and OS notifications on timer finish (all opt-out per-timer).
- Strict TS, ESLint flat config, Prettier, full typecheck + lint + format + build pipeline.
- 71 kB JS gzipped, 4 kB CSS gzipped.

## Stack

| Layer       | Tool                                                        |
| ----------- | ----------------------------------------------------------- |
| Shell       | Tauri 2 (Rust)                                              |
| UI          | React 19 + TypeScript 5.8                                   |
| Build       | Vite 7                                                      |
| Styles      | Tailwind CSS 4 (via `@tailwindcss/vite`)                    |
| State       | Zustand 5 (with `persist` middleware)                       |
| Icons       | lucide-react                                                |
| Lint        | ESLint 10 flat config + typescript-eslint                   |
| Format      | Prettier 3 + prettier-plugin-tailwindcss                    |
| Persistence | `@tauri-apps/plugin-store` (Tauri) ↔ localStorage (browser) |

## Project layout

```
calc-timers/
├── index.html
├── package.json
├── tsconfig.json
├── vite.config.ts
├── eslint.config.js
├── .prettierrc
├── public/
│   └── icon.svg
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── index.css
│   ├── components/
│   │   ├── TabBar.tsx
│   │   ├── Calculator.tsx
│   │   ├── TimerList.tsx
│   │   ├── TimerForm.tsx
│   │   └── TimerCard.tsx
│   ├── store/
│   │   ├── calcStore.ts
│   │   └── timerStore.ts
│   └── lib/
│       ├── calc.ts        # shunting-yard parser
│       ├── time.ts        # format + parse durations
│       ├── audio.ts       # WebAudio beep/chime + vibrate
│       ├── notifications.ts
│       └── storage.ts     # Tauri store ↔ localStorage adapter
├── ios/                   # iOS Xcode project (XcodeGen + CocoaPods)
│   ├── project.yml
│   ├── Podfile
│   ├── Runner/
│   │   ├── AppDelegate.swift
│   │   ├── Info.plist
│   │   ├── Runner-Bridging-Header.h
│   │   ├── Base.lproj/LaunchScreen.storyboard
│   │   └── Assets.xcassets/
│   └── README.md          # Mac build instructions
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json    # macOS bundle config included
    ├── capabilities/
    │   ├── default.json   # desktop
    │   └── mobile.json    # iOS / Android
    └── src/
        ├── main.rs
        └── lib.rs
```

## Getting started

Requirements:

- Node 20+
- pnpm 11+
- Rust stable (1.77+)
- Tauri prereqs — see <https://v2.tauri.app/start/prerequisites/>

```bash
pnpm install
```

### Web frontend in a browser

```bash
pnpm dev
# open http://localhost:1420
```

### Native desktop (Win / Mac / Linux)

```bash
pnpm tauri dev          # dev with hot reload
pnpm tauri build        # release → src-tauri/target/release/bundle/
```

On macOS the bundle is `.app` + `.dmg`. On Windows: `.msi` + `.exe`. On Linux: `.deb` / `.AppImage` / `.rpm`.

### Android

```bash
pnpm tauri android init     # one-time
pnpm tauri android dev      # connected device / emulator
pnpm tauri android build    # release APK / AAB
```

Requires Android SDK + NDK. Setup: <https://v2.tauri.app/distribute/google-play/>

### iOS / iPadOS

> Requires a Mac with Xcode 15+. The iOS scaffolding is in `ios/`. Full steps in [`ios/README.md`](ios/README.md).

```bash
# On a Mac:
cargo install tauri-cli --git https://github.com/tauri-apps/tauri --locked
pnpm tauri ios init
brew install xcodegen
cd ios && xcodegen generate && pod install && cd ..
pnpm tauri ios dev
```

The iOS project uses:

- Bundle ID: `com.timedcalc.app`
- Deployment target: iOS 13
- `UIBackgroundModes: audio` so timer chimes finish when backgrounded
- Notification permission keys already in `Info.plist`

## Scripts

| Command             | Purpose                                 |
| ------------------- | --------------------------------------- |
| `pnpm dev`          | Vite dev server                         |
| `pnpm build`        | Typecheck + production frontend build   |
| `pnpm preview`      | Preview the built frontend              |
| `pnpm tauri ...`    | Tauri CLI passthrough                   |
| `pnpm typecheck`    | `tsc --noEmit`                          |
| `pnpm lint`         | ESLint flat config                      |
| `pnpm lint:fix`     | ESLint --fix                            |
| `pnpm format`       | Prettier --write                        |
| `pnpm format:check` | Prettier --check                        |
| `pnpm verify`       | typecheck + lint + format:check + build |

## Keyboard shortcuts (Calculator)

- Digits, `+ - * / % ( )` — input
- `Enter` / `=` — evaluate
- `Backspace` — delete last token
- `Escape` / `c` — clear

## Color tokens

Defined in `src/index.css` as CSS variables; mapped via Tailwind v4 `@theme`.

| Token                   | Value     | Use                   |
| ----------------------- | --------- | --------------------- |
| `--color-bg`            | `#f3efe6` | Page background       |
| `--color-surface`       | `#fbf9f3` | Card background       |
| `--color-border`        | `#e0d9c8` | Hairline divider      |
| `--color-text`          | `#2b2823` | Primary text          |
| `--color-text-muted`    | `#7a766c` | Secondary text        |
| `--color-accent`        | `#7a9b76` | Primary accent (sage) |
| `--color-accent-strong` | `#5e7e5a` | Pressed accent        |
| `--color-accent-soft`   | `#d8e3d4` | Operator button bg    |
| `--color-danger`        | `#c47868` | Errors                |
| `--color-success`       | `#7fa374` | Timer finished        |

## License

MIT.
