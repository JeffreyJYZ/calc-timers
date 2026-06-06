# CalcTimers

A fast, beautiful desktop & mobile utility app built with **Tauri 2 + React 19 + TypeScript + Tailwind CSS 4 + Vite 7**.

Two tools in one app:

- **Calculator** — full precision shunting-yard parser, history, keyboard shortcuts, sign toggle, percent.
- **Multi-Timer** — run unlimited concurrent countdowns with progress rings, labels, per-timer sound/vibration/notification toggles, presets, and persistence across restarts.

Ice-blue dark theme tuned for OLED; runs at native speed on Windows, macOS, Linux, Android.

## Highlights

- Native window (Tauri 2 webview) — single tiny binary, no Electron.
- Cross-platform desktop (Win/Mac/Linux) and Android. iOS/macOS native bundle is not enabled in this repo.
- Calculator engine uses shunting-yard (no `eval`); supports `+ − × ÷ % ( )` and unary minus.
- Timers persist with **Zustand + localStorage** and are also mirrored to a Tauri store file for reliability across webview clears.
- Sound, vibration, and OS notifications on timer finish (all opt-out per-timer).
- Glass UI, animated progress rings, snowflake ambient background.
- Strict TS, ESLint flat config, Prettier, full typecheck + lint + format + build pipeline.

## Stack

| Layer | Tool |
| --- | --- |
| Shell | Tauri 2 (Rust) |
| UI | React 19 + TypeScript 5.8 |
| Build | Vite 7 |
| Styles | Tailwind CSS 4 (via `@tailwindcss/vite`) |
| State | Zustand 5 (with `persist` middleware) |
| Icons | lucide-react |
| Lint | ESLint 10 flat config + typescript-eslint |
| Format | Prettier 3 + prettier-plugin-tailwindcss |
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
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── capabilities/default.json
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

### Run the web frontend in a browser

```bash
pnpm dev
# open http://localhost:1420
```

### Run the native desktop app

```bash
pnpm tauri dev
```

### Build a release bundle (Windows MSI / NSIS)

```bash
pnpm tauri build
```

Artifacts land in `src-tauri/target/release/bundle/`.

### Android

```bash
pnpm tauri android init     # one-time
pnpm tauri android dev      # connected device / emulator
pnpm tauri android build    # release APK / AAB
```

> iOS is not configured in this repo. Adding it requires macOS + Xcode. See Tauri docs.

## Scripts

| Command | Purpose |
| --- | --- |
| `pnpm dev` | Vite dev server |
| `pnpm build` | Typecheck + production frontend build |
| `pnpm preview` | Preview the built frontend |
| `pnpm tauri ...` | Tauri CLI passthrough |
| `pnpm typecheck` | `tsc --noEmit` |
| `pnpm lint` | ESLint flat config |
| `pnpm lint:fix` | ESLint --fix |
| `pnpm format` | Prettier --write |
| `pnpm format:check` | Prettier --check |
| `pnpm verify` | typecheck + lint + format:check + build |

## Keyboard shortcuts (Calculator)

- Digits, `+ - * / % ( )` — input
- `Enter` / `=` — evaluate
- `Backspace` — delete last token
- `Escape` / `c` — clear

## License

MIT.
