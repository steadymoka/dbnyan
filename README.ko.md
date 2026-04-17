# dbnyan

> English: [README.md](./README.md)

손으로 만든 작은 MySQL 관리 도구. 로컬 우선·단일 바이너리 지향이고, **Claude Code 구독**을 그대로 활용하는 자연어 → SQL 생성기를 내장.

TablePlus / SequelAce에서 영감을 받되 더 차분하고 의도된 디자인. 모든 동작은 단일 Rust 프로세스 위의 브라우저 탭 스타일로 진행됨.

---

## 기능

- **커넥션** — 중첩 폴더(`prod/api/staging`), 컬러 라벨, 터널 옵션(**SSH** key/agent 또는 **AWS SSM** 포트 포워딩), 드래그 폴더 이동, 원클릭 clone
- **탭** — 브라우저 탭 스타일. 상태는 `localStorage` + URL(`?cid=…&db=…&t=…&v=q`)에 영속화. 새로고침·공유 링크 모두 같은 화면 복원
- **Browse** — DB 드롭다운 picker + 테이블 목록 + 스키마 + 행 200개 미리보기
- **Query** — **CodeMirror 6** 신택스 하이라이트 SQL 에디터(`⌘⏎` 실행), 커넥션별 히스토리 (시간/성공 표시)
- **AI SQL 생성기** — 자연어 한 번 → SQL 블록. `Use →` 클릭하면 에디터로 paste. 로컬 `claude` CLI(구독 인증) 경유, 활성 DB의 테이블 목록을 system context로 자동 주입
- **Editorial UI** — 따뜻한 cream/paper 톤, Fraunces serif italic 디스플레이, JetBrains Mono 코드, Plus Jakarta Sans UI, 단일 rust 액센트

---

## 빠르게 시작

```bash
./bin/start
```

웹 빌드 → Rust 서버 기동 (단일 포트, 기본 `3939`). 브라우저에서 <http://127.0.0.1:3939>.

> 첫 실행은 3~5분 (release 컴파일 + 초기 bun install). 이후엔 캐시되어 몇 초.

### Dev 모드 (HMR)

```bash
./bin/dev
```

Rust :3939 + SvelteKit dev 서버 :5173 (Vite가 `/api/*`를 백엔드로 프록시). <http://localhost:5173>.

### `make` 단축

```bash
make            # 타겟 목록
make dev        # = ./bin/dev
make start      # = ./bin/start
make portless   # ./bin/start를 portless 아래로 (https://dbnyan.localhost:1355)
make build      # 웹 빌드 + cargo release
make check      # cargo check + svelte-check
make fmt        # cargo fmt + prettier
make clean      # 빌드 산출물 삭제
```

### portless ([vercel-labs/portless](https://github.com/vercel-labs/portless))

`*.localhost` HTTPS 프록시 셋업이 있으면:

```bash
make portless                                # → https://dbnyan.localhost:1355
make portless PORTLESS_NAME=admin.dbnyan     # → https://admin.dbnyan.localhost:1355
```

`bin/start`는 portless가 넘긴 `PORT` env를 받아 그 포트에 서버를 띄움 (우선순위: `DBNYAN_PORT` > `PORT` > `3939`). 끝의 `:1355`는 portless 프록시의 기본 포트로, `portless proxy start -p 443` (sudo 필요)로 띄우면 생략 가능.

### 메뉴바 (SwiftBar)

선택사항: 맥 메뉴바에 portless 서버 상태 + Start / Stop / Open / View logs 메뉴. 플러그인은 레포 안에 그대로 들어 있음 — [`bin/swiftbar/dbnyan.10s.sh`](./bin/swiftbar/dbnyan.10s.sh).

```bash
brew install --cask swiftbar
open -a SwiftBar
```

첫 실행 다이얼로그에서 **Plugin Folder**를 `<이 레포>/bin/swiftbar`로 지정. 10초마다 갱신 (파일명 `.10s.sh` → `.30s.sh` 식으로 주기 변경). `PORTLESS_NAME`을 기본값과 다르게 쓰는 경우 플러그인 상단 `NAME=` 줄을 수정.

> 참고: SwiftBar는 폴더 하나만 감시함. `bin/swiftbar`로 지정하면 다른 SwiftBar 플러그인도 같은 폴더에 둬야 함.

---

## 요구사항

- **macOS** (Linux는 동작 가능성 높지만 미검증)
- [Rust](https://rustup.rs/) ≥ 1.80
- [Bun](https://bun.sh/) — SvelteKit 빌드/dev 서버
- 테스트할 **MySQL** 인스턴스
- AI 생성기 쓰려면: [Claude Code](https://claude.com/claude-code) 설치 + `claude login`. 서버는 `claude -p`를 spawn 하면서 `ANTHROPIC_API_KEY`를 자식 env에서 제거 → 구독 인증 강제.
- AWS SSM 포워드 커넥션 쓰려면: `aws` CLI v2 (`aws configure` 완료) + [Session Manager Plugin](https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-working-with-install-plugin.html) 설치 필요.

---

## 아키텍처

```
[Browser]
   │  HTTP (REST + JSON)
   ▼
[Rust axum :3939]
   ├─ /api/*           → crates/server 핸들러 (crates/core 호출)
   │     ├─ SQLite     (앱 상태 — 커넥션, 히스토리)
   │     └─ MySQL pool (커넥션별 활성. SSH 터널은 `ssh -N -L` 서브프로세스)
   └─ /(static)        → tower-http ServeDir로 web/build/ 서빙 (SvelteKit SPA)
```

```
dbnyan/
├── Cargo.toml                         # workspace
├── bin/{dev,start}                    # 진입 스크립트
├── Makefile
├── crates/
│   ├── core/
│   │   ├── migrations/*.sql           # sqlx 마이그레이션
│   │   └── src/
│   │       ├── connection.rs          # 저장된 커넥션 CRUD
│   │       ├── ssh.rs                 # SshConfig / SshAuth 모델
│   │       ├── tunnel.rs              # `ssh -N -L` 서브프로세스 래퍼
│   │       ├── session.rs             # 커넥션별 활성 MySQL pool
│   │       ├── mysql.rs               # SHOW DBs/TABLES, 스키마, 행 프리뷰
│   │       ├── query.rs               # 임의 SQL 실행
│   │       └── history.rs             # 쿼리 히스토리 (SQLite)
│   └── server/
│       └── src/
│           ├── main.rs                # router + state + static fallback
│           ├── connections.rs         # /api/connections CRUD
│           ├── runtime.rs             # /api/connections/:id/{databases,tables,query,history,…}
│           └── chat.rs                # /api/connections/:id/chat — `claude -p` spawn
└── web/                               # SvelteKit 5 + Tailwind v4 + adapter-static
    └── src/
        ├── app.html                   # Google Fonts import
        ├── routes/
        │   ├── +page.svelte           # 탭 바, empty 상태, 모달 마운트
        │   └── layout.css             # design tokens (@theme), base layer
        └── lib/
            ├── api.ts                 # 타입 + fetch 클라이언트
            ├── stores/tabs.svelte.ts  # 탭 + 탭별 상태, localStorage
            └── components/
                ├── TabContent.svelte  # 사이드바 + 메인 browse 뷰
                ├── QueryView.svelte   # 에디터 + 히스토리 + 생성기 컨테이너
                ├── SqlEditor.svelte   # CodeMirror 6 래퍼, editorial 테마
                ├── SqlGenerator.svelte# AI 생성기 카드
                ├── RowGrid.svelte     # 데이터 테이블
                ├── NewTabModal.svelte # 커넥션 picker / form 호스트
                └── ConnectionForm.svelte
```

### AI 흐름

```
[Browser] ──POST /api/connections/:id/chat──▶ [Rust]
                                                │
                                                ▼
                                  spawn `claude -p "<스키마 컨텍스트 + 유저 메시지>"
                                          --output-format json`
                                  (env_remove ANTHROPIC_API_KEY → 구독)
                                                │
                                                ▼
                                  JSON 파싱 → { text, session_id, … }
```

생성기는 의도적으로 **단발성** — 매 프롬프트마다 활성 DB의 테이블 목록을 system context로 새로 붙임. 멀티턴 / 채팅 스레드 없음. 가치는 **프롬프트 → SQL 블록 → 에디터로 paste**, 대화가 아님.

### 왜 Rust + SvelteKit?

추후 Tauri 데스크톱 앱으로 옮기는 길을 열어두려고 분리했음. SvelteKit 번들은 완전 정적(`adapter-static`)이고 Rust 백엔드는 그대로 Tauri의 `invoke()` IPC 뒤에 들어감. 지금은 `127.0.0.1` HTTP로 통신.

---

## 저장소

- **앱 상태** (저장된 커넥션, 쿼리 히스토리) — 단일 SQLite 파일 (`<data_dir>/dbnyan/app.db`, WAL 모드)
  - macOS: `~/Library/Application Support/dbnyan/app.db`
  - Override: `DBNYAN_DATA_DIR=./data ./bin/start`
- **탭 UI 상태** (열린 탭, 선택 db/table, draft SQL, 뷰 모드) — `localStorage` (`dbnyan.tabs.v1`). 서버 재시작·새로고침 모두 안전. URL `?cid=…&db=…&t=…&v=q`로도 반영되어 딥링크 가능.
- **비밀번호** — 현재 SQLite 평문 저장 (MVP — UI에 노란 경고). Keychain 연동은 후순위.

---

## 환경변수

| Var | 기본값 | 비고 |
|-----|--------|-----|
| `DBNYAN_PORT` | `3939` | 최우선 |
| `PORT` | — | `DBNYAN_PORT` 미설정 시 사용 (portless 등이 셋업) |
| `DBNYAN_DATA_DIR` | 플랫폼 `data_dir/dbnyan/` | `app.db` 위치 |
| `RUST_LOG` | `info,sqlx=warn,tower_http=info` | tracing-subscriber 필터 (예: `RUST_LOG=info,sqlx=info`로 백엔드가 실행하는 SQL 전부 로그) |

---

## 디버깅

- **백엔드 로그** — `RUST_LOG=debug ./bin/start` 전부; `RUST_LOG=info,sqlx=info`면 실행되는 SQL 출력
- **API 직접 호출** — UI의 모든 동작은 그냥 REST: `curl localhost:3939/api/connections | jq` 등
- **앱 SQLite** — `sqlite3 "$HOME/Library/Application Support/dbnyan/app.db"` → `.tables`, `SELECT * FROM connections;`, `SELECT * FROM query_history ORDER BY executed_at DESC LIMIT 10;`
- **AI 단독** — `unset ANTHROPIC_API_KEY && claude -p "your prompt" --output-format json | jq`로 CLI 단독 검증
- **프론트** — DevTools Network에서 `/api/*` / Console에서 `localStorage.getItem('dbnyan.tabs.v1')`
- **전체 리셋** — `rm "$HOME/Library/Application Support/dbnyan/app.db"*` 후 재기동

---

## 한계 / TODO

- MySQL만 지원 (PostgreSQL / SQLite 드라이버는 후순위)
- SSH 비번 인증 미지원 — 키 또는 `ssh-agent` 사용
- AI 응답 비스트리밍 (5~15초 대기)
- 비번 평문 저장 (Keychain 연동 예정)
- 자동 테스트 없음 — 백엔드 단위 테스트 후보 명확 (`query::is_select_like`, `connection` CRUD, `history` CRUD); UI는 수동 스모크 테스트

---

## 기술 스택

- **백엔드** — Rust, [axum](https://github.com/tokio-rs/axum), [sqlx](https://github.com/launchbadge/sqlx), [tower-http](https://github.com/tower-rs/tower-http), [chrono](https://github.com/chronotope/chrono), tokio
- **프론트엔드** — [SvelteKit 5](https://svelte.dev) (runes 모드), [Tailwind CSS v4](https://tailwindcss.com), [CodeMirror 6](https://codemirror.net) + `@codemirror/lang-sql`
- **타입 / 빌드** — TypeScript, Bun
- **폰트** — Fraunces (디스플레이), Plus Jakarta Sans (UI), JetBrains Mono (코드)

---

## 라이선스

개인 프로젝트, 라이선스 미선언.
