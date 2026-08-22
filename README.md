# Lokked

Cross-platform study app — per-subject timers, flashcards and statistics.

Target platforms, in order: **Linux** (Fedora 43 / GNOME / Wayland) → Windows → Android → iOS.

Stack: [Tauri 2](https://tauri.app) + Rust backend, TypeScript + React + Vite frontend, SQLite for data.

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

The window opens on the timers screen. Data lives in the per-user app data
directory (`~/.local/share/com.lokked.app/lokked.sqlite3` on Linux).

## Commands

| Command                                        | What it does                                                               |
| ---------------------------------------------- | -------------------------------------------------------------------------- |
| `npm run tauri dev`                            | Run the desktop app with hot reload                                        |
| `npm run tauri build`                          | Build a release bundle                                                     |
| `npm run package:linux`                        | The same, with the AppImage workaround Fedora needs                        |
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
  commands/       thin #[tauri::command] layer — no domain logic
  core/           pure Rust: clock, dayline, timer, scheduler, stats, cli, backup
  db/             SQLite access, migrations and the startup backup
  desktop.rs      command line, suspend/resume, startup backup wiring
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

## The streak

A day counts towards the streak once it has enough study time on it — ten
minutes by default, changed under **Settings → Серия**. The day boundary is
the student's own, so a session that runs past midnight lands on the day it
belonged to.

The streak is never reset at midnight: a day that has only just begun is not
a miss, it is a day that has not happened yet, so the number stays up until
the day is actually over.

**Freezes** are what a missed day costs instead of the whole streak. One is
earned per ten days in a row, at most three in hand, and a missed day spends
one automatically. A frozen day keeps the streak alive but does not lengthen
it — eleven studied days across twelve calendar ones still reads as eleven.
Once there is nothing left to spend, the streak ends and the freezes go with
it.

The page itself is at **Серия**: the current run, the record with the days it
ran between, the freezes in hand, a calendar of the month and the milestones
at 7, 30 and 100 days. «Поделиться серией» draws a 1080×1350 image in the
black screen's own colours and saves it to the pictures directory.

## How cards are picked

Cards are not shuffled evenly. Every card in a deck carries a weight computed
from its own answers in `reviews` — recent accuracy (with the newest answers
counting for more), how long it has been out of sight, what the last answer
was, and how much history there is to judge by. The next card is drawn at
random in proportion to those weights.

Two rules hold whatever the weights say:

- **Nothing leaves the rotation.** A card answered perfectly fifty times sinks
  to a small weight and stays there. It comes round rarely; it never
  disappears, the way a due-date queue would hide it.
- **Nothing repeats immediately.** The last few cards dealt are held back from
  the draw, so a heavy card cannot come up twice in a row.

Because the draw is with replacement, a card may come round more than once
inside one sitting — which is the point: answering «не помню» recomputes that
card's weight straight away and usually brings it back within the next
handful. A marathon is the exception: it runs the whole deck, so there the
weight decides how early a card comes, not whether it comes at all.

**Settings → Карточки** has a slider for how far this leans. All the way left
is a plain shuffle; all the way right is a sitting made almost entirely of
what is not going well.

## Global hotkeys on Wayland

Wayland does not let an application grab a global shortcut for itself, so
Lokked is driven from the outside: a second launch hands its arguments to the
copy that is already running.

| Command           | What it does                                 |
| ----------------- | -------------------------------------------- |
| `lokked --toggle` | Pause a running session, resume a paused one |
| `lokked --zen`    | Open the black screen                        |
| `lokked --stop`   | Stop the session and write it down           |

A launch with no arguments simply brings the window forward.

Bind them in **Settings → Keyboard → View and Customize Shortcuts → Custom
Shortcuts**. Use the absolute path to the binary — GNOME does not read your
shell's `PATH`:

| Name          | Command                    | Suggested key |
| ------------- | -------------------------- | ------------- |
| Lokked: пауза | `/usr/bin/lokked --toggle` | `Super+Alt+P` |
| Lokked: zen   | `/usr/bin/lokked --zen`    | `Super+Alt+Z` |
| Lokked: стоп  | `/usr/bin/lokked --stop`   | `Super+Alt+S` |

Running a development build instead? Point the shortcut at
`src-tauri/target/debug/lokked`.

## Suspend and backups

While a work phase runs, Lokked asks the desktop portal (falling back to
`org.freedesktop.ScreenSaver`) not to blank the screen. It also listens for
`PrepareForSleep` from `logind`: closing the lid pauses the session, and on
waking the app offers to carry on — the time asleep is never counted as study.

Every launch writes one copy of the database into `backups/` next to it,
keeping the newest seven. The copies are made with `VACUUM INTO`, so each one
is a complete database that opens on its own:

```sh
sqlite3 ~/.local/share/com.lokked.app/backups/lokked-20260821-034509.sqlite3
```

## Packaging

```sh
npm run package:linux
```

Produces an `.rpm`, an AppImage and a `.deb` under
`src-tauri/target/release/bundle/`. The `.rpm` installs the binary as
`/usr/bin/lokked`, which is the path the shortcuts above expect.

The script is `tauri build` with `NO_STRIP=true`: the AppImage step runs
`linuxdeploy`, whose bundled `strip` is older than the `.relr.dyn` sections
in current Fedora libraries and fails on every one of them. Skipping the
strip costs a few megabytes in an image that is ~100 MB of GTK and WebKit
anyway. That step also downloads `linuxdeploy` on first use, so it needs
network access; `npm run tauri build -- --bundles rpm` skips it entirely.

## Troubleshooting

**Blank or black window on Wayland**, typically with NVIDIA drivers — WebKitGTK's
DMA-BUF renderer is the usual cause:

```sh
WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run tauri dev
```
