# StudyApp

Cross-platform study app — per-subject timers, flashcards and statistics.

Target platforms, in order: **Linux** (Fedora 43 / GNOME / Wayland) → Windows → Android → iOS.

Stack: [Tauri 2](https://tauri.app) + Rust backend, TypeScript + React + Vite frontend, SQLite for data.

> Current state: **skeleton only**. There is no business logic yet — no timers,
> no database, no flashcards. What exists is the module layout, the tooling, and
> a `ping()` command proving the Rust ↔ TypeScript bridge works.

## Prerequisites

Rust via [rustup](https://rustup.rs) (not the distro package — `rustup target add`
is needed for the Android and iOS targets later), Node.js 20+, and the WebKitGTK
development libraries.

On Fedora:

```sh
sudo dnf install -y webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel \
  librsvg2-devel patchelf openssl-devel curl wget file
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

For other platforms see the [Tauri prerequisites](https://tauri.app/start/prerequisites/).

## Running

```sh
npm install
npm run tauri dev     # opens the app window
```

The window should show **StudyApp** and `ping() → pong`. If it shows an error
instead, the Rust ↔ TypeScript bridge is broken — start at
`src-tauri/src/commands.rs` and `src/lib/tauri.ts`.

## Commands

| Command                                        | What it does                                                               |
| ---------------------------------------------- | -------------------------------------------------------------------------- |
| `npm run tauri dev`                            | Run the desktop app with hot reload                                        |
| `npm run tauri build`                          | Build a release bundle                                                     |
| `npm run dev`                                  | Frontend only, in a browser at `localhost:1420` (Tauri commands will fail) |
| `npm run build`                                | Type-check and build the frontend                                          |
| `npm run lint` / `lint:fix`                    | ESLint + Prettier                                                          |
| `npm test` / `test:watch`                      | Vitest                                                                     |
| `npm run fmt:rust` / `lint:rust` / `test:rust` | `cargo fmt` / `clippy` / `test`                                            |
| `npm run check`                                | Everything above, as one gate                                              |

## Layout

```
src-tauri/src/
  main.rs         thin wrapper around lib.rs::run()
  lib.rs          Tauri builder, module declarations
  commands.rs     thin #[tauri::command] layer — no domain logic
  core/           pure Rust: clock, dayline, timer, scheduler, stats
  db/             SQLite access + migrations
  platform/       PlatformServices trait; linux / windows / mobile / noop backends
src/
  main.tsx        React entry point
  routes/         screens + the hash router
  components/     reusable UI
  lib/            typed wrappers over the Tauri commands
  styles/         Tailwind entry point and theme tokens
```

Two conventions worth knowing before adding code:

- **`crate::core`, never bare `core::`.** The `core` module shadows Rust's
  built-in `core` crate inside this crate.
- **Hash routing, not browser routing.** A release build serves the frontend
  over Tauri's asset protocol, which has no SPA fallback.

## Troubleshooting

**Blank or black window on Wayland**, typically with NVIDIA drivers — WebKitGTK's
DMA-BUF renderer is the usual cause:

```sh
WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run tauri dev
```
