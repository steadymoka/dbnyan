# dbnyan

> 한국어 버전: [README.ko.md](./README.ko.md)

A small, hand-built MySQL admin tool. Local-first, single-binary friendly, and shipped with a built-in natural-language → SQL generator that uses your **Claude Code subscription** instead of an API key.

Built as a calmer, more opinionated alternative to TablePlus / SequelAce — everything happens in browser-style tabs on top of one Rust process.

---

## Features

- **Connections** — nested folders (`prod/api/staging`), color labels, optional tunnel (**SSH** key/agent or **AWS SSM** port-forwarding), drag-to-reorganize, one-click clone
- **Tabs** — browser-style; state persists in `localStorage` and the URL (`?cid=…&db=…&t=…&v=q`) so refreshes and shared links land on the same view
- **Browse** — collapsed database picker, table list, schema view, 200-row preview
- **Query** — SQL editor with **CodeMirror 6** syntax highlighting (`⌘⏎` to run), per-connection history with timing & success status
- **AI SQL generator** — single-shot natural-language input that returns a SQL block; `Use →` pushes it straight into the editor. Wired through the local `claude` CLI (subscription auth), schema context auto-injected from the active database
- **Editorial UI** — warm cream/paper palette, italic Fraunces serif display, JetBrains Mono for code, Plus Jakarta Sans for UI, single rust accent color

---

## Quick start

```bash
./bin/start
```

Builds the web bundle and serves it from the Rust backend on a single port (default `3939`). Open <http://127.0.0.1:3939>.

> The first run takes 3–5 minutes (release compile + initial bun install). Subsequent starts are cached and take a few seconds.

### Dev mode (HMR)

```bash
./bin/dev
```

Rust on `:3939`, SvelteKit dev server on `:5173` with Vite proxying `/api/*` to the backend. Open <http://localhost:5173>.

### `make` shortcuts

```bash
make            # list targets
make dev        # = ./bin/dev
make start      # = ./bin/start
make portless   # spawn ./bin/start under portless (https://dbnyan.localhost:1355)
make build      # web build + cargo release
make check      # cargo check + svelte-check
make fmt        # cargo fmt + prettier
make clean      # remove build artifacts
```

### portless ([vercel-labs/portless](https://github.com/vercel-labs/portless))

If you have the `*.localhost` HTTPS proxy set up:

```bash
make portless                                # → https://dbnyan.localhost:1355
make portless PORTLESS_NAME=admin.dbnyan     # → https://admin.dbnyan.localhost:1355
```

`bin/start` honors the `PORT` env var that portless sets (priority: `DBNYAN_PORT` > `PORT` > `3939`). The `:1355` suffix is the portless proxy's default — set up `portless proxy start -p 443` (requires sudo) to drop it.

### Menu bar (SwiftBar)

Optional: show the portless server state in the macOS menu bar with Start / Stop / Open / View logs actions. The plugin lives in-tree at [`bin/swiftbar/dbnyan.10s.sh`](./bin/swiftbar/dbnyan.10s.sh).

```bash
brew install --cask swiftbar
open -a SwiftBar
```

In SwiftBar's first-launch dialog, set **Plugin Folder** to `<this repo>/bin/swiftbar`. The icon updates every 10 s (rename the file suffix to change, e.g. `dbnyan.30s.sh`). If you run portless under a custom name (`make portless PORTLESS_NAME=admin.dbnyan`), edit the `NAME=` line at the top of the plugin.

> Note: SwiftBar can only watch one folder. Pointing it at `bin/swiftbar` means other SwiftBar plugins would also need to live there.

---

## Requirements

- **macOS** (Linux probably works, untested)
- [Rust](https://rustup.rs/) ≥ 1.80
- [Bun](https://bun.sh/) — used for the SvelteKit build / dev server
- A **MySQL** instance to point at
- For the AI generator: [Claude Code](https://claude.com/claude-code) installed and `claude login`’d. The server spawns `claude -p` and removes `ANTHROPIC_API_KEY` from the child env so subscription auth is used.
- For AWS SSM port-forward connections: `aws` CLI v2 (configured via `aws configure`) and the [Session Manager Plugin](https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-working-with-install-plugin.html) on this machine.

---

## Architecture

```
[Browser]
   │  HTTP (REST + JSON)
   ▼
[Rust axum :3939]
   ├─ /api/*           → handlers in crates/server (calls into crates/core)
   │     ├─ SQLite     (app state — connections, history)
   │     └─ MySQL pools per active connection (optional SSH tunnel via `ssh -N -L`)
   └─ /(static)        → tower-http ServeDir of web/build/  (SvelteKit SPA)
```

```
dbnyan/
├── Cargo.toml                         # workspace
├── bin/{dev,start}                    # entry-point scripts
├── Makefile
├── crates/
│   ├── core/
│   │   ├── migrations/*.sql           # sqlx migrations
│   │   └── src/
│   │       ├── connection.rs          # saved connections + CRUD
│   │       ├── ssh.rs                 # SshConfig / SshAuth model
│   │       ├── tunnel.rs              # `ssh -N -L` subprocess wrapper
│   │       ├── session.rs             # active MySQL pools per connection
│   │       ├── mysql.rs               # SHOW DBs/TABLES, schema, row preview
│   │       ├── query.rs               # arbitrary SQL execution
│   │       └── history.rs             # query history (SQLite)
│   └── server/
│       └── src/
│           ├── main.rs                # router + state + static fallback
│           ├── connections.rs         # /api/connections CRUD
│           ├── runtime.rs             # /api/connections/:id/{databases,tables,query,history,…}
│           └── chat.rs                # /api/connections/:id/chat — spawns `claude -p`
└── web/                               # SvelteKit 5 + Tailwind v4 + adapter-static
    └── src/
        ├── app.html                   # Google Fonts import
        ├── routes/
        │   ├── +page.svelte           # tab bar, empty state, modal mount
        │   └── layout.css             # design tokens (@theme), base layer
        └── lib/
            ├── api.ts                 # typed fetch client
            ├── stores/tabs.svelte.ts  # tab + per-tab state, localStorage
            └── components/
                ├── TabContent.svelte  # sidebar + main browse view
                ├── QueryView.svelte   # editor + history + generator container
                ├── SqlEditor.svelte   # CodeMirror 6 wrapper, editorial theme
                ├── SqlGenerator.svelte# AI generator card
                ├── RowGrid.svelte     # data table
                ├── NewTabModal.svelte # connection picker / form host
                └── ConnectionForm.svelte
```

### AI flow

```
[Browser] ──POST /api/connections/:id/chat──▶ [Rust]
                                                │
                                                ▼
                                  spawn `claude -p "<schema context + user message>"
                                          --output-format json`
                                  (env_remove ANTHROPIC_API_KEY → subscription)
                                                │
                                                ▼
                                  parse JSON → { text, session_id, … }
```

The generator is intentionally one-shot — each prompt re-attaches the active database’s table list as system context. No multi-turn / no chat thread; the value is **prompt → SQL block → push into the editor**, not conversation.

### Why Rust + SvelteKit

The split keeps the door open for shipping as a Tauri desktop app later: the SvelteKit bundle is fully static (`adapter-static`) and the Rust backend is the same crate that would back Tauri’s `invoke()` IPC. Today everything talks over local HTTP on `127.0.0.1`.

---

## Storage

- **App state** (saved connections, query history) — single SQLite file (`<data_dir>/dbnyan/app.db`, WAL mode)
  - macOS: `~/Library/Application Support/dbnyan/app.db`
  - Override with `DBNYAN_DATA_DIR=./data ./bin/start`
- **Tab UI state** (which tabs are open, selected db/table, draft SQL, view mode) — `localStorage` (`dbnyan.tabs.v1`). Survives server restart, refresh, and the URL `?cid=…&db=…&t=…&v=q` reflects the active tab so deep links work.
- **Passwords** — currently plaintext in SQLite (MVP — there’s a yellow notice in the UI). Keychain integration is on the to-do list.

---

## Environment

| Var | Default | Notes |
|-----|---------|-------|
| `DBNYAN_PORT` | `3939` | Highest priority |
| `PORT` | — | Used if `DBNYAN_PORT` is unset (e.g. set by portless) |
| `DBNYAN_DATA_DIR` | platform `data_dir/dbnyan/` | Where `app.db` lives |
| `RUST_LOG` | `info,sqlx=warn,tower_http=info` | tracing-subscriber filter (e.g. `RUST_LOG=info,sqlx=info` to log every SQL the backend runs) |

---

## Debugging

- **Backend logs** — `RUST_LOG=debug ./bin/start` for everything; `RUST_LOG=info,sqlx=info` to see executed SQL
- **API directly** — anything in the UI is a normal REST call: `curl localhost:3939/api/connections | jq`, etc.
- **App SQLite** — `sqlite3 "$HOME/Library/Application Support/dbnyan/app.db"` then `.tables`, `SELECT * FROM connections;`, `SELECT * FROM query_history ORDER BY executed_at DESC LIMIT 10;`
- **AI standalone** — `unset ANTHROPIC_API_KEY && claude -p "your prompt" --output-format json | jq` to confirm the CLI side independently
- **Frontend** — Browser DevTools (Network for `/api/*`, Console for `localStorage.getItem('dbnyan.tabs.v1')`)
- **Reset everything** — `rm "$HOME/Library/Application Support/dbnyan/app.db"*` then restart

---

## Limitations / TODO

- MySQL only (PostgreSQL / SQLite drivers later)
- SSH password auth not supported — use a key or `ssh-agent`
- AI responses are not streamed (5–15 s wait for the full text)
- Plaintext password storage (Keychain integration pending)
- No automated tests — backend has obvious unit-test candidates (`query::is_select_like`, `connection` CRUD, `history` CRUD); UI relies on manual smoke testing

---

## Tech stack

- **Backend** — Rust, [axum](https://github.com/tokio-rs/axum), [sqlx](https://github.com/launchbadge/sqlx), [tower-http](https://github.com/tower-rs/tower-http), [chrono](https://github.com/chronotope/chrono), tokio
- **Frontend** — [SvelteKit 5](https://svelte.dev) (runes mode), [Tailwind CSS v4](https://tailwindcss.com), [CodeMirror 6](https://codemirror.net) with `@codemirror/lang-sql`
- **Type / build** — TypeScript, Bun
- **Fonts** — Fraunces (display), Plus Jakarta Sans (UI), JetBrains Mono (code)

---

## License

Personal project, no license declared.
