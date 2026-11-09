# Defending Axum as a backend framework

Notes for the oral defense and to justify the bonus module **"Use a
framework for both the frontend and backend"** (Web, major, 2 pts, chap.
VII). Vue covers the frontend without any possible debate (explicitly
cited as a framework in III.3 of the subject). Axum, on the other hand,
doesn't appear in the subject's list of examples (Express, Fastify,
NestJS, Django, Flask, Rails) — this document explains why it still
meets the given definition, with precise examples drawn from the code
rather than generalities.

## The subject's exact criterion (III.3)

> *A framework must have the following features: a structured
> architecture, with conventions for organizing code; built-in
> functionalities for common tasks (e.g. routing, database access,
> session management...); a complete ecosystem of tools and libraries.*

Three criteria, not a closed list of names — the subject says "such as"
before Express/Django/Rails, so those are examples, not an exhaustive
list. Axum is checked against these three criteria, one by one.

## 1. Structured architecture, with conventions

Axum enforces a precise code shape, not just a library of functions
called however you like:

- **A handler is an `async` function, its parameters are extractors, its
  return type implements `IntoResponse`.** You don't choose this shape,
  Axum requires it — a handler that doesn't conform doesn't compile.
- **The router assembles sub-routers via `.nest()`**, not through an ad
  hoc dispatch system. Real example (`src/routes/mod.rs:9-15`): each
  module (`auth`, `users`, `friends`, `media`, `rooms`, `ws`) exposes its
  own `Router`, assembled by the application at startup. This convention
  is what let this project keep 8 separate backend modules, each
  testable and readable on its own.
- **Inversion of control is the point that really distinguishes a
  framework from a library** (see the online manual, "Framework vs.
  library" section): a library is code that *mine* calls; a framework is
  a structure that calls *mine* at the right time. With Axum, it's the
  router that decides when and with what arguments to call each handler
  — never the other way around.

## 2. Built-in functionalities for common tasks

The subject explicitly cites *routing*, *database access*, *session
management* as examples of what a framework must provide out of the box.

| Common task | What Axum provides | Example in the code |
|---|---|---|
| Routing | `Router`, `.route()`, `.nest()`, path parameter extraction (`Path<T>`) | `src/routes/mod.rs` |
| Shared state / session | `State<T>` (shared PostgreSQL pool), **custom** `AuthUser` extractor that checks the token on every private route | `src/auth/extractor.rs` (`impl FromRequestParts<PgPool> for AuthUser`) |
| Data access | Body/request extractors (`Json<T>`, `Query<T>`), composed with sqlx for the rest | every handler |
| WebSocket | `WebSocketUpgrade`, native handling of the connection upgrade | `src/ws/handler.rs:26-36` |
| Static files | `ServeDir` (tower-http), wired in one line | `src/server/mod.rs` |

The most telling point is the **custom extractor** `AuthUser`
(`src/auth/extractor.rs`): writing `auth: AuthUser` as a handler
parameter is enough to make it private, without rewriting the check
every time. This is exactly the kind of "built-in functionality for a
common task" the criterion asks for — except Axum lets you extend it
cleanly instead of imposing a ready-made one, which is consistent with a
low-level rather than high-level framework (see section 4).

## 3. Complete ecosystem of tools and libraries

Axum isn't isolated: it's designed around **Tower** (middleware, called
*layers*) and integrates natively with Tokio (async runtime) and Serde
(serialization). In this project:

```rust
// src/server/mod.rs
let app = routes::router(pool.clone(), rooms, frontend_origins)
    .nest_service("/uploads", ServeDir::new("uploads"))   // tower-http
    .layer(Extension(MediaClient::new(&config.peertube_url)))
    .layer(Extension(OAuthConfig::new(&config)))
    .layer(Extension(LoginRateLimiter::new()))
    .layer(cors)                                           // tower-http CorsLayer
    .with_state(pool);
```

Each `.layer()` is a Tower middleware stacked on top of every route
without touching the handlers (CORS, additional shared state). It's the
same mechanism that would let you add compression, metrics, or a generic
rate limiter tomorrow — not an isolated library, an ecosystem that
composes.

## 4. Why Axum isn't in the subject's list (and why that excludes nothing)

Anticipated reviewer pushback: *"Axum is minimalist, not complete like
Rails or Django."* Response:

- The subject doesn't ask to be "as complete as Rails" — it lists three
  precise criteria (above), not a given level of abstraction. Axum meets
  all three, documented with real examples from the code, not vague
  claims.
- Express — cited as an example in the subject — is itself minimalist
  (no ORM, no built-in session management, everything comes from
  third-party middleware). If Express counts as a framework despite
  that, "minimalist" isn't a disqualifying criterion.
- The historical difference for Axum and Rust: before crates like
  Axum/Actix, writing an HTTP server in Rust meant handling low-level
  TCP/HTTP parsing yourself. Axum brings exactly what the subject
  expects from a framework compared to "writing without a framework":
  declarative routing, extractors, composable middleware.

## What to be able to say at the oral, in one sentence

*"Axum meets the subject's three criteria — architecture with
conventions, built-in functionality for routing/state/sessions, a full
Tower/Tokio ecosystem — even though it's not in the list of examples,
which isn't closed. The `AuthUser` extractor is the most concrete
example: it shows the inversion of control that distinguishes a
framework from a library."*
