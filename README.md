*This project has been created as part of the 42 curriculum by sberete.*

# SyncWatch

## Description

SyncWatch is a platform for watching videos together, in real time, from
anywhere: shared rooms with synchronized playback, a live chat, and a
social layer (friends, profiles, online status). It is this project's
answer to the `ft_transcendence` subject's open brief — instead of Pong,
the real-time core is a shared watch-room.

SyncWatch never hosts or streams copyrighted content itself. It only
aggregates videos from sources that explicitly allow it through a public
API — the Internet Archive (public-domain films) and PeerTube (freely
licensed videos) — and hands the browser a direct link to the source. The
server never stores or proxies the video stream.

Key features:
- Account system (email/username + password, or Google/GitHub OAuth)
- Friends (requests, accept/decline, online status), with notifications
  for friend activity and room events
- A video catalogue aggregated live from legal sources, with search,
  filtering/sorting/pagination, and playback (MP4 and HLS)
- A hybrid (content + collaborative) recommendation system based on
  watch history
- Watch rooms with a real-time chat (WebSocket), synchronized playback
  (play/pause/seek shared across a room, clock sync and drift
  correction), and automatic cleanup when a room empties
- French, English and Spanish, fully translated including both legal
  pages
- Privacy Policy and Terms of Service pages
- The whole site served over HTTPS behind an nginx reverse proxy

See "Modules" below for the exact, verified state of every claimed
module.

## Instructions

### Prerequisites

- Docker and Docker Compose
- `make`
- For native backend development only (optional): Rust/Cargo, Node.js

### Setup

```bash
git clone <this-repository>
cd ft_transcendence
cp .env.example .env
```

Fill in `.env` with real values. At minimum, `POSTGRES_PASSWORD` should be
changed from the example. `GOOGLE_CLIENT_ID`/`GOOGLE_CLIENT_SECRET` and
`GITHUB_CLIENT_ID`/`GITHUB_CLIENT_SECRET` are optional — the OAuth buttons
stay visible but inactive without them. See the comments in `.env.example`
for where to obtain each value.

### Run everything in Docker (single command)

```bash
make up
```

Opens the site at **https://localhost:8080**. The certificate is
self-signed (generated at build time, not committed) — the browser will
show a security warning on first load; this is expected, accept it to
continue. `make down` stops everything, `make logs` tails every service,
`make clean` also removes the volumes (database and avatars are lost).

### Development mode

```bash
make dev
```

Runs PostgreSQL and the frontend in Docker, and the backend natively via
`cargo run` for fast recompiles. The site stays on the same address,
`https://localhost:8080` — nginx routes to the native backend instead of
the Docker one. Database migrations are applied automatically on backend
startup, in both modes.

`make dev` and `make up` cannot run at the same time in different
terminals: `make dev` stops the Docker backend container every time it
runs, so a `make up` left running elsewhere gets cut without warning.

### Tests

```bash
cargo test                 # unit tests, no database needed
cargo test -- --ignored    # + integration tests (needs the database running)
```

## Resources

Classic references used while building this project (full annotated list
in [`docs/resources.md`](docs/resources.md)):

- Rust: [The Rust Book](https://doc.rust-lang.org/book/), [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- Tokio: [official tutorial](https://tokio.rs/tokio/tutorial)
- Axum: [repository and examples](https://github.com/tokio-rs/axum), [API docs](https://docs.rs/axum)
- SQLx + PostgreSQL: [SQLx repository](https://github.com/launchbadge/sqlx), [PostgreSQL docs](https://www.postgresql.org/docs/)
- WebSockets: [RFC 6455](https://www.rfc-editor.org/info/rfc6455), [MDN WebSocket API](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
- Security: [MDN CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS), [OWASP Cheat Sheet Series](https://github.com/OWASP/CheatSheetSeries), [RFC 9106 (Argon2)](https://www.rfc-editor.org/rfc/rfc9106)
- OAuth 2.0: [RFC 6749](https://www.rfc-editor.org/rfc/rfc6749), [Google OAuth docs](https://developers.google.com/identity/protocols/oauth2/web-server), [GitHub OAuth docs](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps)
- Frontend: [Vue 3 docs](https://vuejs.org/guide/introduction), [Vite docs](https://vite.dev), [Vue Router docs](https://router.vuejs.org/introduction)
- Playback: [hls.js](https://github.com/video-dev/hls.js)
- Catalogue sources: [Internet Archive Advanced Search API](https://archive.org/advancedsearch.php), [PeerTube REST API](https://docs.joinpeertube.org/api/rest-reference.html)
- Docker: [Compose file reference](https://docs.docker.com/reference/compose-file/)

### Use of AI

AI (Claude, through Claude Code) was used throughout this project as a
pair-programming assistant, not as an autonomous author:

- **Design**: discussing the concept, technology choices (Rust/Axum,
  PostgreSQL, Vue), and product decisions (e.g. the 4 room-behaviour
  choices in `docs/websocket.md` section 9) — always made or validated by
  me, never decided unilaterally by the AI.
- **Learning Rust**: a full set of warm-up exercises (not part of this
  repository) was used to learn the language before starting the project;
  those exercises were written and coded entirely by me, with the AI only
  providing the prompts, never the solutions.

## Team Information

| Member | Roles | Responsibilities |
|---|---|---|
| sberete | Product Owner, Project Manager, Technical Lead, Developer | All of them — this project is currently being built solo. The subject expects a 4-5 person team; roles will be reassigned and this section rewritten honestly if/when teammates join. |

## Project Management

With no team yet, there is no formal coordination process to describe.
Work is organized as:

- **Feature branches**, not person-based ones, so that a real teammate
  could pick up a specific piece of work later without inheriting a
  fabricated commit history: `rooms-rest`, `ws-core`, `frontend-chat`,
  `frontend-sync`, `compliance`, merged into `main` as they stabilize.
- **Git history** as the record of what was done and when — commit
  messages explain the *why*, not just the *what*.
- A private working journal (outside this repository) tracking decisions,
  open questions, and a day-by-day log, kept up to date as the project
  evolves.

## Technical Stack

- **Backend**: Rust + [Axum](https://github.com/tokio-rs/axum) 0.8 (switched from Actix-web early on), on the
  [Tokio](https://tokio.rs) async runtime. Chosen to build real depth in a systems language
  rather than defaulting to something more familiar.
- **Database**: PostgreSQL 16, accessed through [SQLx](https://github.com/launchbadge/sqlx) with hand-written SQL
  (`sqlx::query_as`, `#[derive(sqlx::FromRow)]`) and compile-time query
  checking — a deliberate choice over an ORM, for direct control over the
  exact queries run. The "ORM for the database" module was dropped as a
  result, rather than retrofitting one into already-stable, tested
  queries for a single point; see "Modules" for its replacement.
- **Frontend**: [Vue 3](https://vuejs.org) (Composition API, `<script setup>`) + [Vue Router](https://router.vuejs.org) 4 +
  [Vite](https://vite.dev). [hls.js](https://github.com/video-dev/hls.js) for HLS playback in browsers that don't support
  it natively.
- **Styling**: [Tailwind CSS](https://tailwindcss.com) (via `@tailwindcss/vite`) alongside the existing
  hand-written design system (CSS custom properties for color/spacing
  tokens, scoped component styles) — utility classes for layout, and
  arbitrary-value classes (`bg-[var(--bg-hover)]`) to stay on the same
  design tokens rather than duplicating them in Tailwind's own theme.
- **Real time**: native WebSockets on Axum/Tokio (no Socket.IO or similar
  library) — connection handling, rate limiting, and custom close codes
  implemented directly against [RFC 6455](https://www.rfc-editor.org/info/rfc6455).
- **Infrastructure**: Docker Compose (`db`, `backend`, `frontend`
  services), nginx as a reverse proxy terminating HTTPS (self-signed
  certificate generated at image build time) and routing to the backend
  over the internal Docker network — the backend publishes no port of its
  own. A `Makefile` wraps the common commands.

## Database Schema

PostgreSQL 16, schema defined in `migrations/`, applied automatically on
backend startup.

```mermaid
erDiagram
  users ||--o{ sessions : "has"
  users ||--o{ oauth_accounts : "links"
  users ||--o{ friends : "sends"
  users ||--o{ rooms : "owns"
  rooms ||--o{ messages : "contains"
  users ||--o{ messages : "writes"
  users ||--o{ notifications : "receives"
  users ||--o{ watch_history : "watches"
  users {
    bigserial id PK
    varchar32 username UK
    varchar255 email UK
    text password_hash "NULL for OAuth-only accounts"
    text avatar_url
    timestamptz last_seen_at
    timestamptz created_at
  }
  oauth_accounts {
    varchar16 provider PK
    text provider_user_id PK
    bigint user_id FK
  }
  sessions {
    bigserial id PK
    bigint user_id FK
    text token_hash UK
    timestamptz expires_at
  }
  friends {
    bigint user_id PK
    bigint friend_id PK
    varchar16 status
  }
  rooms {
    bigserial id PK
    varchar64 name
    bigint owner_id FK
  }
  messages {
    bigserial id PK
    bigint room_id FK
    bigint sender_id FK
    text content
  }
  notifications {
    bigserial id PK
    bigint user_id FK
    varchar32 kind
    jsonb payload "denormalized at creation time"
    timestamptz read_at
    timestamptz created_at
  }
  watch_history {
    bigint user_id PK_FK
    varchar16 provider PK
    varchar100 media_id PK
    varchar255 title
    int year
    text thumbnail_url
    timestamptz watched_at
  }
```

Notes:
- A friendship is stored as two rows, one per direction, which keeps "my
  friends" a simple `WHERE user_id = me AND status = 'accepted'` query.
- `rooms` is deleted (cascading to its `messages`) as soon as its last
  member leaves — rooms are not meant to persist empty.
- A `videos` table exists in the first migration but is unused: uploading
  personal videos was considered and then dropped in favor of relying
  entirely on the external catalogue (see "Modules").
- `watch_history`'s primary key is `(user_id, provider, media_id)`, not a
  row id: one row per (user, video) pair, re-watching just bumps
  `watched_at` — it's the set the recommendation system reasons over, not
  an event log.
- `notifications.payload` is denormalized at creation time (e.g. the
  other user's username/avatar at that moment) so the notification list
  never needs a second query per row to render.

## Features List

All features below were designed and implemented by sberete (see "Team
Information").

| Feature | Description |
|---|---|
| Account system | Sign up / log in with email or username + password (Argon2, salted), log out, DB-backed sessions |
| OAuth login | Google and GitHub, popup-based flow, account linking on a verified email |
| Profile | View and edit username, upload an avatar (with progress bar and delete) |
| Friends | Send/accept/decline requests, friends list, online status |
| Notifications | Friend requests/accepts/removals, a friend starting a session, room ownership transfer — navbar bell, unread count, polling-based |
| Video catalogue | Search and browse videos aggregated from Internet Archive and PeerTube, with filter by source, sort, and pagination |
| Recommendations | Hybrid content (TF-IDF) + collaborative filtering based on watch history, shown on the home page |
| Playback | MP4 and HLS playback in-browser (hls.js), no server-side transcoding |
| Watch rooms | Create or join a room, WebSocket connection, live member list, synchronized playback (play/pause/seek, clock sync, drift correction, ownership transfer) |
| Chat | Real-time messages in a room, persisted history with pagination |
| Multi-language | French (default), English, Spanish — switcher in the navbar, full coverage including legal pages |
| Privacy Policy / Terms of Service | Accessible from the footer on every page |
| HTTPS | Entire site served over TLS through an nginx reverse proxy |

## Modules

Target: 14 points (major = 2, minor = 1). Status reflects what is
genuinely functional today, not what is planned — a module claimed here
that turns out broken or incomplete is worth 0 points per the subject, so
this table is kept honest rather than aspirational.

| Module | Category | Points | Status |
|---|---|---|---|
| Real-time features (WebSockets) | Web, Major | 2 | **Done** — rooms, presence, chat and synchronized playback all implemented, verified both end-to-end on the protocol and visually across two real browser sessions (clock sync, drift correction, ownership transfer on owner departure) |
| Allow users to interact (chat, profile, friends) | Web, Major | 2 | **Done** |
| Standard user management | User Management, Major | 2 | **Done** |
| Recommendation system (ML) | Artificial Intelligence, Major | 2 | **Done** — a hybrid of collaborative filtering (co-watch overlap between accounts) and content-based filtering (TF-IDF + cosine similarity over titles, implemented from scratch and unit-tested), fed by a single instrumented endpoint shared by solo and in-room playback. Zero-signal candidates are dropped rather than padded with a "most watched" fallback |
| File upload and management | Web, Minor | 1 | **Done** — avatar upload (type/size validated server-side by real file signature), a live progress indicator during upload, a delete action, and the previous file is cleaned up from disk on both replace and delete |
| Remote authentication (OAuth 2.0) | User Management, Minor | 1 | **Done** |
| Notification system | Web, Minor | 1 | **Done** — persistent notifications (friend requests/accepts/removals, a friend starting a session, room ownership transfer), surfaced via a navbar bell with an unread count, fetched by polling (the WebSocket connection is scoped to a room, not global to the app) |
| Support for additional browsers | Accessibility, Minor | 1 | **Done** — replaces the ORM module (dropped, see "Technical Stack"). A latent crash on browsers without `BroadcastChannel` support was fixed in the OAuth popup flow. Systematic pass run on real Firefox and Safari (signup/login, OAuth popup, search/filters, avatar upload with progress bar, joining a room by code, chat, and playback sync including mid-session join) — see `docs/browser-testing.md` for the filled-in grid |
| Advanced search | Web, Minor | 1 | **Done** — filter by source, sort by title/year/duration, and client-side pagination (12 results per page) on the search page |
| Support for multiple languages | Accessibility, Minor | 1 | **Done** — French (default), English and Spanish (3 complete translations, meeting the subject's minimum), via vue-i18n; a switcher in the navbar, the choice persisted in `localStorage`, and the `lang` attribute kept in sync. Covers the whole app including both legal pages, not just a partial pass. Replaces 2FA (dropped as a module choice) |

All 14 mandatory points are implemented and verified, including the
manual Firefox/Safari pass for "Support for additional browsers" (see
`docs/browser-testing.md`).

## Bonus

Per the subject (chap. VII), bonus is only evaluated once the mandatory
14 points are solid — true as of this submission — and each extra module
must be fully functional and justified here, in the README.

| Module | Category | Points | Status |
|---|---|---|---|
| Use a framework for both the frontend and backend | Web, Major | 2 | **Done** |

**Why this counts**: Vue is explicitly listed as a frontend framework by
the subject (III.3), so that half isn't in question. Axum isn't in the
subject's example list for the backend half (Express, Fastify, NestJS,
Django, Flask, Rails) but meets the subject's own three-part definition
of a framework — structured architecture with conventions, built-in
functionality for common tasks, a complete ecosystem — not just a
different name for the same examples:
- **Structured architecture**: a handler must be `async`, take extractors
  as parameters, and return a type implementing `IntoResponse` — this
  isn't a convention you can ignore, non-conforming code doesn't compile.
  Sub-routers are assembled via `.nest()` (`src/routes/mod.rs`), not an
  ad hoc dispatch table — this is what let the backend stay split into 8
  independent modules (`auth`, `users`, `friends`, `media`, `rooms`, `ws`,
  `notifications`, `recommendations`), each testable on its own.
- **Built-in functionality for common tasks** (the subject's own examples
  — routing, database access, session management): `Router`/`.route()`
  for routing, `State<T>` for the shared connection pool, a custom
  `AuthUser` extractor (`src/auth/extractor.rs`) that turns any handler
  parameter `auth: AuthUser` into an authenticated-only route without
  repeating the check, `WebSocketUpgrade` for native WebSocket handling.
- **Inversion of control**, the property that actually separates a
  framework from a library per the subject's own definition: the Axum
  router decides when and with what arguments each handler gets called —
  the handler never calls Axum, Axum calls the handler.

Full argument, with more code examples, in `docs/axum-as-framework.md`.

## Individual Contributions

Everything in this repository was built by **sberete**, working solo (see
"Team Information" for why). Breakdown by area:

- **Backend foundations**: Axum server setup, PostgreSQL pool, migrations,
  modular routing (`src/routes`), the `ApiError` → HTTP response mapping.
- **Authentication**: password auth (Argon2, DB-backed sessions), OAuth
  2.0 for Google and GitHub (authorization-code flow, account linking on
  verified email, CSRF-safe `state` parameter).
- **Social features**: user profiles, avatar upload, the friends system
  (bidirectional rows, request/accept flow).
- **Video catalogue**: the aggregator (`src/media`), querying Internet
  Archive and PeerTube in parallel, normalizing their responses into one
  shared format.
- **Real-time core**: the WebSocket protocol (`docs/websocket.md`), its
  implementation (`src/ws`, `src/rooms`), and the frontend chat/room UI
  (`roomSocket.js` and related views), including synchronized playback
  (clock sync, drift correction, ownership transfer).
- **Recommendation system**: the hybrid scoring pipeline (`src/recommendations`),
  a hand-written TF-IDF + cosine similarity implementation
  (`scoring.rs`, unit-tested), and the single instrumentation point
  (`GET /media/{provider}/{id}`) feeding `watch_history` from both solo
  and in-room playback.
- **Notifications**: the persistent notification system (`src/notifications`),
  polling-based delivery on the frontend (no global WebSocket in this
  app), and the navbar bell UI.
- **Internationalization**: the vue-i18n setup (`frontend/src/i18n.js`),
  French/English/Spanish translation of the entire app including both
  legal pages, and the locale switcher.
- **Search**: filter by source, sort, and client-side pagination on top
  of the existing catalogue aggregator.
- **Styling**: the hand-written design system (CSS custom properties,
  scoped component styles) plus Tailwind CSS, added specifically to meet
  the subject's CSS-framework requirement (III.3) without rewriting what
  already worked.
- **Testing**: a formalized multi-user concurrency test suite
  (`tests/test_concurrency.py` — concurrent signups, room joins,
  ownership transfer, playback commands, rate-limit bursts), replacing
  ad hoc one-off scripts.
- **Infrastructure**: Docker Compose setup, the nginx reverse proxy and
  HTTPS termination, unifying the Docker and native-backend development
  workflows behind one URL; later hardened for Podman compatibility (see
  "Challenges faced").
- **Compliance**: Privacy Policy and Terms of Service pages, this README.

**Challenges faced and how they were resolved**:
- *Choosing Rust mid-piscine without prior experience in the language*:
  resolved by completing a dedicated set of warm-up exercises covering
  ownership, borrowing, traits, and error handling before writing any
  project code.
- *A silently stale backend IP cached by nginx after a container
  restart*, surfacing as an infinite loading screen with no clear error:
  found by comparing `docker inspect`'s `OOMKilled` flag (false) against
  the assumption of a crash, then tracing it to nginx's one-time DNS
  resolution of the Docker service name; fixed with dynamic resolution
  through Docker's embedded DNS.
- *A page/API path collision* (`/friends` existing as both a frontend
  route and a backend endpoint) once the frontend and the API were merged
  behind a single HTTPS origin: resolved by prefixing every API call with
  `/api`.
- *The project wouldn't come up on the school's machines*, which run
  Podman instead of Docker: unqualified image names (`postgres:16-alpine`)
  made Podman prompt interactively for a registry, which blocks a
  detached `docker compose up -d` indefinitely — fixed by fully
  qualifying every image with `docker.io/library/`. Separately, nginx's
  `resolver` directive was hardcoded to Docker's embedded DNS address
  (`127.0.0.11`); Podman's own DNS listens elsewhere, so every proxied
  request (including both OAuth popups) hung until a timeout, then failed
  with 502 — fixed by switching to the official nginx image's
  `NGINX_ENTRYPOINT_LOCAL_RESOLVERS` mechanism, which reads the
  container's actual `/etc/resolv.conf` instead of assuming a fixed
  address, portable between Docker and Podman.
