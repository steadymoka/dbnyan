# dbnyan

Local-first MySQL admin tool — TablePlus/SequelAce 스타일을 단순화. Rust 백엔드 + SvelteKit 프론트, 단일 바이너리로 동작 가능. Claude Code 구독을 활용한 자연어 → SQL 생성기 내장.

## Features

- **Connections**: 폴더/라벨 + SSH 터널(키/agent 인증) + 평문 비번(MVP) 저장
- **Tabs**: 브라우저 탭 스타일 — 새로고침/공유 URL로 상태 복원 (localStorage + URL 쿼리)
- **Browse**: 데이터베이스/테이블 목록, 스키마 뷰, 행 200개 미리보기
- **Query**: SQL 에디터 (⌘⏎ 실행) + 히스토리 (성공/실패/시간/행수)
- **AI SQL Generator**: Claude Code 구독 경유 — 자연어 입력 → SQL 출력 → "Use" 버튼으로 에디터에 paste

## Requirements

- macOS (Linux는 동작할 가능성 높지만 미검증)
- [Rust](https://rustup.rs/) ≥ 1.80
- [Bun](https://bun.sh/) (frontend 빌드/dev 서버용)
- MySQL 인스턴스 (테스트할 대상)
- AI 챗 기능 쓰려면 [Claude Code](https://claude.com/claude-code) 설치 + `claude login`

## Quick start

```bash
./bin/start
```

웹 빌드 → Rust 서버 기동 → 브라우저로 http://127.0.0.1:3939 열면 됨.

### Dev mode (HMR)

```bash
./bin/dev
```

Rust 서버(:3939)는 백그라운드, SvelteKit dev 서버(:5173)가 포어그라운드. Vite 프록시가 `/api/*` 를 Rust로 흘려보냄. 프론트 수정 시 즉시 반영.

### Environment

- `DBNYAN_PORT` — 서버 포트 (기본 3939)
- `DBNYAN_DATA_DIR` — 앱 상태 SQLite 파일 위치 override (기본 macOS: `~/Library/Application Support/dbnyan/`)

## Architecture

```
dbnyan/
├── Cargo.toml                 # Rust workspace
├── crates/
│   ├── core/                  # lib: 커넥션 저장(SQLite) + MySQL 드라이버 + SSH 터널
│   │   ├── migrations/
│   │   └── src/
│   │       ├── connection.rs   # 저장된 커넥션 CRUD
│   │       ├── ssh.rs          # SSH 터널 설정 모델
│   │       ├── tunnel.rs       # `ssh -N -L` 서브프로세스 관리
│   │       ├── session.rs      # 활성 MySQL pool 매니저
│   │       ├── mysql.rs        # 메타 쿼리 + 행 프리뷰
│   │       ├── query.rs        # 임의 SQL 실행
│   │       └── history.rs      # 쿼리 히스토리
│   └── server/                # bin: axum HTTP 서버
│       └── src/
│           ├── main.rs
│           ├── connections.rs  # /api/connections CRUD
│           ├── runtime.rs      # /api/connections/:id/{databases,tables,query,history,...}
│           └── chat.rs         # /api/connections/:id/chat (Claude Code 서브프로세스)
└── web/                       # SvelteKit + Tailwind v4, adapter-static (SPA)
    └── src/
        ├── lib/
        │   ├── api.ts          # 타입 + fetch 클라이언트
        │   ├── stores/tabs.svelte.ts   # 탭 상태 + localStorage
        │   └── components/
        └── routes/+page.svelte
```

### Data flow

```
[Browser]
  ↓ HTTP (REST + JSON)
[Rust axum :3939]
  ├─ /api/* routes → core/
  │     ├── SQLite (app state: connections, history)
  │     └── MySQL pools (per active connection, with optional SSH tunnel)
  └─ /(static) → web/build/  (SvelteKit SPA)
```

AI SQL 생성:
```
[Browser] → POST /api/connections/:id/chat → [Rust]
  → spawn `claude -p "<schema context + user message>" --output-format json`
    (uses local Claude Code subscription, env_remove ANTHROPIC_API_KEY)
  → parse JSON, return { text, session_id, … }
```

### Why Rust + SvelteKit (not Tauri / Electron)

향후 Tauri 데스크톱 앱으로 전환할 수 있도록 백엔드를 Rust로 두고 프론트를 정적 SvelteKit 번들로 분리. 현재는 로컬 HTTP로 통신하지만, Tauri 전환 시 `invoke()` IPC로 자연스럽게 갈아끼울 수 있음.

## Storage

- **앱 상태** (등록된 커넥션, 쿼리 히스토리): SQLite 파일 (`<data_dir>/dbnyan/app.db`, WAL 모드)
- **비밀번호**: 현재는 평문 (MVP) — UI에 경고 배너. macOS Keychain 연동은 후순위

## Limitations / TODO

- MySQL only (Postgres/SQLite 지원은 후속)
- SSH password 인증 미지원 (key/agent 만)
- AI 응답 비스트리밍 (5~15초 대기)
- SQL 에디터 신택스 하이라이팅 없음 (textarea — CodeMirror 후속)
- 비번 평문 저장
- 셀 클릭 시 전체 값 모달 없음 (truncate + title tooltip)

## License

Personal project, no license declared.
