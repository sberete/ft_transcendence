# SyncWatch real-time contract (WebSocket)

> **Status: implemented.** This document served as the contract before writing
> the `ws` module (phase 3); the code now follows it to the letter (see
> `src/rooms/`, `src/ws/`). Updated 2026-10-29: rooms are identified by a
> randomly generated 6-digit `code` (rather than the sequential internal
> `id`, too easy to guess by incrementing).

## 1. Overview

A **room** groups users watching the same video. The server is the sole
source of truth for playback state: a client never tells the others what
to do, it **asks** the server, which validates and then broadcasts to
everyone.

```
Client A ── playback.pause ──► Server ── playback.state ──► A, B, C
```

The `rooms` and `messages` tables have existed since migration 0001; the
`code` column (migration 0004) was added afterward.

## 2. REST routes to add

Creating and reading a room stay in REST: these are one-off actions, not
real-time.

| Method | Path | Role | Response |
|---|---|---|---|
| `POST` | `/rooms` | Create a room. Body: `{"name"}` | `201` `Room` |
| `GET` | `/rooms/{code}` | Read a room | `200` `Room` · `404` |
| `GET` | `/rooms/{code}/messages?before={id}&limit=50` | Chat history, newest to oldest | `200` `[ChatMessage]` |

No route to delete a room: it deletes itself automatically as soon as the
last member leaves (see section 9, point 3) — nothing else in the process
ever needed one.

`{code}` is the 6-digit code returned on creation, not an internal id —
it's the only way to refer to a room from outside the backend.

```json
// Room
{ "code": "335551", "name": "Chaplin Night", "owner_id": 3, "created_at": "2026-10-10T20:00:00Z" }
```

All private (`Authorization: Bearer`).

## 3. Connection

```
GET /ws        (Upgrade: websocket)
```

### Authentication via the first message

A browser can't add an `Authorization` header to a WebSocket connection.
Two options exist: put the token in the URL (`/ws?token=…`), or send it in
the first message. **We keep the first-message approach**, because a URL
ends up in server logs, proxies, and browser history, and a token has no
business being there.

1. The client opens the connection.
2. It sends `auth` within **5 seconds**.
3. The server replies with `auth.ok`, or closes with code `4401`.

Until `auth.ok` has been received, any other message closes the connection.

The server must also check the `Origin` header of the opening request and
reject unknown origins: WebSockets aren't subject to CORS.

## 4. Message format

**Text** frames, JSON, one object per frame. The `type` field identifies
the message; the other fields depend on the type.

```json
{ "type": "chat.send", "content": "Hi!" }
```

- Names in `domain.action`, lowercase.
- Dates in ISO 8601 UTC. Playback positions in **seconds**, decimal number.
- Server clock (`server_time`) in **milliseconds** since the Unix epoch.
- An unknown field is ignored; an unknown `type` triggers an `error`
  message.
- Maximum frame size: **4 KB**. Beyond that, closed with `4400`.

## 5. Client → server messages

| `type` | Fields | Effect |
|---|---|---|
| `auth` | `token` | Authenticates the connection. |
| `room.join` | `code` | Joins a room. Leaves the previous one: one connection = one room. |
| `room.leave` | — | Leaves the current room. |
| `chat.send` | `content` (1 to 500 characters) | Sends a message in the room. |
| `playback.set_media` | `provider`, `id` | Changes the room's video. Resets position to 0, paused. |
| `playback.play` | `position` | Starts playback from this position. |
| `playback.pause` | `position` | Pauses at this position. |
| `playback.seek` | `position` | Jumps to this position without changing play/pause state. |
| `ping` | `t` (client clock, ms) | Measures latency and clock offset. |

`provider` and `id` follow exactly the same rules as
`GET /media/{provider}/{id}`. The server validates them the same way, and
never accepts a URL.

## 6. Server → client messages

| `type` | Fields | When |
|---|---|---|
| `auth.ok` | `user` (`{id, username, avatar_url}`) | After a valid `auth`. |
| `room.state` | `room`, `members`, `media`, `playback` | Right after `room.join`: the full state. |
| `room.member_joined` | `user` | Someone enters the room. |
| `room.member_left` | `user_id`, `new_owner_id` | Someone leaves or disconnects. `new_owner_id` is present only if the one leaving was the owner and the room continues (transferred to the member present the longest). |
| `chat.message` | `id`, `sender`, `content`, `created_at` | New message, including to its own author. |
| `playback.state` | `media`, `playing`, `position`, `server_time`, `by` | On every playback change, to everyone. |
| `pong` | `t`, `server_time` | Reply to `ping`. |
| `error` | `code`, `message` | A request was rejected. The connection stays open. |

```json
// room.state
{
  "type": "room.state",
  "room": { "code": "335551", "name": "Chaplin Night", "owner_id": 3 },
  "members": [{ "id": 3, "username": "alice", "avatar_url": null }],
  "media": { "provider": "archive", "id": "notld_201610" },
  "playback": { "playing": true, "position": 754.2, "server_time": 1791590000000 }
}
```

`media` is `null` as long as no video has been chosen.

### Error codes (`error.code`)

| Code | Meaning |
|---|---|
| `bad_message` | Invalid JSON, unknown `type`, or missing field. |
| `not_in_room` | A room action sent without having joined a room. |
| `room_not_found` | The room doesn't exist. |
| `forbidden` | Action reserved for the room owner. |
| `invalid_media` | `provider` or `id` in the wrong format. |
| `rate_limited` | Too many messages. |

## 7. Playback synchronization

### Principle

The server doesn't continuously broadcast "where the video is at." It
stores a **reference point**, and each client computes the expected
position:

```
if playing: expected_position = position + (server_now - server_time) / 1000
else:       expected_position = position
```

A single message is therefore enough for a new arrival to sync up, even
mid-playback.

### Clock offset

`server_now` isn't the client's clock. The client estimates it with
`ping` / `pong`:

```
round_trip = reception - t
offset     = server_time + round_trip / 2 - reception
server_now = Date.now() + offset
```

Take 5 measurements on connection and keep the one with the shortest
round trip (the least disturbed), then one measurement every 30 seconds.

### Drift correction

Every 2 seconds, the client compares `video.currentTime` to the expected
position:

| Gap | Action |
|---|---|
| less than 0.25 s | Nothing. |
| 0.25 s to 2 s | Adjust `video.playbackRate` (0.95 or 1.05) until caught up: imperceptible. |
| more than 2 s | `video.currentTime = expected_position` (visible jump). |

Same principle as Movix's WASM engine (*calibrating → adjusting →
perfect* statuses), in plain JavaScript. A WASM module could replace it
later without changing this contract.

### Avoiding loops

When the client applies a `playback.state`, the `<video>` tag emits its
own `play`, `pause`, `seeked` events. These must **not** be sent back to
the server, or every client would re-trigger all the others forever. The
client keeps a "change came from the server" flag and ignores events it
triggered itself.

## 8. Limits and closures

| Rule | Value |
|---|---|
| Time to authenticate | 5 s |
| Frame size | 4 KB |
| Chat messages | 5 per 5 seconds per user |
| Playback commands | 10 per 5 seconds per user |
| Inactivity (no message, no `ping`) | closed after 60 s |

Exceeding a limit returns `error` with `rate_limited`. Persisting closes
the connection.

| Close code | Reason |
|---|---|
| `1000` | Normal closure. |
| `4400` | Unreadable message or frame too large. |
| `4401` | Missing, invalid, or expired token, or authentication deadline exceeded. |
| `4408` | Inactivity. |
| `4429` | Limits exceeded repeatedly. |

Codes 4000 to 4999 are reserved for applications by the WebSocket
standard.

After a closure other than `4401`, the client reconnects with a growing
delay (1 s, 2 s, 4 s… capped at 30 s), resending `auth` then `room.join`.

## 9. Decisions made (2026-10-19)

1. **Who controls playback?** Any member can play, pause, and seek. Only
   the owner can change the video (`playback.set_media` rejected with
   `forbidden` if the caller isn't `owner_id`).
2. **Who can join a room?** Anyone connected who has the 6-digit code. No
   friendship check.
3. **What happens to an empty room?** **Automatic deletion** as soon as
   the last member leaves or disconnects (different from the initial
   proposal): nobody left to broadcast anything to, the server directly
   deletes the `rooms` row (cascading to `messages` via the existing
   constraint). If the owner leaves but the room continues, it isn't
   closed: ownership passes to the member present the longest
   (`room.member_left.new_owner_id`). A room that never had a connected
   member (created then never joined) isn't affected by this as long as
   nobody triggered `room.join` then left.
4. **Is chat kept?** Yes, in the `messages` table. Plain text only, never
   interpreted as HTML (`v-text`/interpolation on the Vue side, never
   `v-html`).
