# Resources

Reference document to understand each piece of tech used in SyncWatch —
useful for you, for a future teammate picking up a branch, and as a base
for the final README's "Resources" section (required by the subject).

Every link was checked before being added here (no guessed links). For
one specific YouTube video where I couldn't confirm the exact URL, the
link points to the channel itself with the title to search for, rather
than risking a dead link.

---

## Rust (language)

- **The official book ("The Book")** — https://doc.rust-lang.org/book/
  The essential starting point. Ownership, borrowing, structs, enums,
  traits, generics, error handling, concurrency. Also available offline
  via `rustup doc --book`.
- **Rust by Example** — https://doc.rust-lang.org/rust-by-example/
  Same material as the book, but through code examples rather than
  theory — a good complement if you learn better by reading code.
- **YouTube channel "Let's Get Rusty"** — https://www.youtube.com/@letsgetrusty
  Playlist that follows the official book chapter by chapter, on video.
  A good complement when reading alone isn't enough for a given concept
  (ownership and lifetimes, in particular).

## Tokio (async runtime)

- **Official tutorial** — https://tokio.rs/tokio/tutorial
  Explains `async`/`.await`, tasks (`tokio::spawn`), shared state,
  channels (`mpsc`) — exactly the building blocks used in `src/ws/`.
  Built around progressively building a mini-Redis.

## Axum (backend framework)

- **Official repo + examples** — https://github.com/tokio-rs/axum
  The repo's `examples/` folder has complete, up-to-date mini-projects,
  including `examples/chat` (WebSocket chat with a `broadcast` channel) —
  a useful comparison point for `src/ws/handler.rs`.
- **API documentation (docs.rs)** — https://docs.rs/axum
  Reference for every type/function (`State`, `Router`, `WebSocketUpgrade`,
  extractors...). Axum builds on the Tower ecosystem rather than having
  its own middleware system — also see `docs.rs/tower`.

## SQLx + PostgreSQL

- **Official SQLx repo** — https://github.com/launchbadge/sqlx
  Pure-Rust async SQL, compile-time checked queries, no ORM/DSL — exactly
  the project's approach (`sqlx::query_as`, `#[derive(sqlx::FromRow)]`
  everywhere in `src/*/handlers.rs`).
- **PostgreSQL documentation** — https://www.postgresql.org/docs/
  The tutorial (Part I) for relational/SQL basics; the rest of the manual
  serves as reference (types, constraints, `INTERVAL`, etc. — used in the
  migrations and in `src/ws/registry.rs` for computing "online" status).

## WebSockets (real-time — Phase 3)

- **RFC 6455 — The WebSocket Protocol** — https://www.rfc-editor.org/info/rfc6455
  The standard itself: handshake, frames, close codes. Useful to
  understand why custom `4400`-`4429` codes are used in
  `docs/websocket.md` (the 4000-4999 range is reserved for applications
  by the standard).
- **MDN — WebSocket API (browser)** — https://developer.mozilla.org/en-US/docs/Web/API/WebSocket
  Client side: `new WebSocket(url)`, `open`/`message`/`close` events,
  useful for the `frontend-chat`/`frontend-sync` branch.

## Web security

- **MDN — CORS** — https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS
  Explains why WebSockets do NOT go through CORS (hence the manual
  `Origin` header check in `src/ws/handler.rs`) and why the `*` wildcard
  doesn't cover `Authorization` (seen earlier during the Actix → Axum
  switch).
- **OWASP Cheat Sheet Series** — https://github.com/OWASP/CheatSheetSeries
  Practical sheets, notably *CSRF Prevention* and *XSS Prevention* —
  relevant for OAuth's anti-CSRF `state` and for the choice to never do
  `v-html` on external content (Archive.org descriptions).
- **RFC 9106 — Argon2** — https://www.rfc-editor.org/rfc/rfc9106
  The reference behind `argon2` (crate used in `src/auth/password.rs`).

## OAuth 2.0 (Google / GitHub)

- **RFC 6749 — The OAuth 2.0 Authorization Framework** — https://www.rfc-editor.org/rfc/rfc6749
  Section 4.1: the "Authorization Code" flow, the one used in
  `src/auth/oauth/`. Dense but THE normative reference.
- **Google Identity documentation (OAuth 2.0, Web Server Apps)** — https://developers.google.com/identity/protocols/oauth2/web-server
- **GitHub documentation — Authorizing OAuth Apps** — https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps
  Both explain each provider's specifics (scopes, endpoints, response
  format) on top of the generic RFC.

## Frontend: Vue 3 / Vite / Vue Router

- **Vue 3 documentation** — https://vuejs.org/guide/introduction
  Composition API (`<script setup>`), reactivity (`ref`/`reactive`),
  components — the whole style used in `frontend/src/`.
- **Vite documentation** — https://vite.dev
  The dev server and build tool used (`npm run dev`, `npm run build`).
- **Vue Router documentation** — https://router.vuejs.org/introduction
  Routes, navigation guards (`router.beforeEach`, used in
  `frontend/src/router/index.js` for `requiresAuth`/`guestOnly`).

## Video playback: hls.js

- **Official repo** — https://github.com/video-dev/hls.js
  Pure-JS HLS player on top of `<video>`. The repo's README covers
  getting started (`new Hls()`, `attachMedia`) — used by
  `frontend/src/components/VideoPlayer.vue`.

## Catalog sources (`media` module)

- **Internet Archive — Advanced Search API** — https://archive.org/advancedsearch.php
  Keyword + filter search (`mediatype:movies`, etc.), Lucene-style syntax.
  Pagination capped at 10,000 results (no impact here given
  `MAX_RESULTS_PER_PROVIDER`).
- **PeerTube — REST API** — https://docs.joinpeertube.org/api/rest-reference.html
  Full OpenAPI spec. OAuth2 authentication is per-instance (not used
  here: the project only does public reads, no upload).

## Docker / Docker Compose

- **Compose file reference** — https://docs.docker.com/reference/compose-file/
  The full spec (`services`, `volumes`, `healthcheck`, `depends_on`...)
  used in `docker-compose.yml`.
- **NetworkChuck YouTube channel** — https://www.youtube.com/@NetworkChuck
  Video to search for on the channel: *"Docker Compose will BLOW your
  MIND!!"* (~16 min, couldn't confirm the exact video URL so I'm giving
  the channel) — focused on Compose specifically rather than Docker in
  general.
- **TechWorld with Nana YouTube channel** — https://www.youtube.com/@TechWorldwithNana
  A more complete course (images, containers, volumes, Compose) if the
  general concept of containerization isn't clear yet.

---

## Use of AI on this project

*Draft for the corresponding README section — to review/complete before
final submission, especially for the video catalog/OAuth part done in a
separate session (see the journal in `idea.txt`, entry from 2026-10-10).*

AI (Claude, via Claude Code) was used throughout the project as a
pair-programming assistant, not as an autonomous author:

- **Design**: discussing the concept (SyncWatch), technical choices
  (Rust/Axum, Postgres, Vue), and product trade-offs (e.g. the 4
  decisions in `docs/websocket.md` section 9) — always validated or
  decided by me, never decided alone by the AI.
- **Implementation**: assisted writing of the backend modules (`auth`,
  `users`, `friends`, `rooms`, `ws`, `media`) and frontend, with
  systematic review and testing after each step (`curl`, WebSocket test
  scripts, Docker) before committing.
- **Debugging**: several real bugs found and fixed together with the AI
  (e.g. missing `media: null` in `playback.state` after `set_media`,
  browser cache on `index.html`, a Docker build cache bug).
- **Learning Rust**: the Rust piscine (`rust-piscine/`) was used to
  learn the language before starting the project — exercises done and
  coded entirely by me, the AI only provided the prompts and never the
  solutions.
- **What I can explain**: the backend architecture (modules, the
  `AuthUser` extractor, connection pool, migrations), the WebSocket
  protocol and its decisions, how Docker Compose works for this project
  — I need to be able to defend every part orally, not just have made it
  work.
