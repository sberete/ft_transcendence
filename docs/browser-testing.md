# Support for additional browsers — test plan

For the **"Support for additional browsers"** module (Accessibility,
minor, 1 pt, chap. IV). The subject asks for compatibility that is
**tested and documented** with at least two browsers besides the main
one — not just "it should work." This document lists what to test, why
each point matters for this specific project, and a grid to fill in
while actually doing it.

I can't open a browser myself: this pass has to be done by hand. This
document serves as a checklist so nothing gets missed, and as evidence
for the README/oral defense once filled in.

## Target browsers

| Browser | Engine | Status before this pass |
|---|---|---|
| Chrome | Blink | Reference, used throughout development |
| Brave | Blink (same engine as Chrome) | Already used several times while debugging playback sync |
| Safari | WebKit | Already used several times (OAuth debugging, sync debugging) — but never a systematic pass |
| Firefox | Gecko | **Never tested** on this project |

Since Brave is Chromium, testing it adds almost nothing new compared to
Chrome (same rendering engine, same APIs) — Safari and Firefox are the
two that really matter for this module, because they're different
engines with different behavior on the sensitive points below.

## Why these specific points (not a generic list)

Each line corresponds to something specific to this project's code, not
a generic "the site renders" test:

- **HLS video playback**: `VideoPlayer.vue` uses hls.js everywhere
  *except* on Safari, which plays HLS natively (`canPlayType`). Two
  different code paths depending on the browser — must be checked
  separately, not assumed equivalent.
- **Autoplay with sound**: browsers block `video.play()` until there has
  been a real click on THIS specific element. This is the direct cause
  of two bugs already found and fixed this session (see the manual,
  "Rooms and real-time" section) — the exact policy varies from one
  engine to another, so the "Join the ongoing video" button's behavior
  must be re-checked per browser.
- **OAuth popup + `BroadcastChannel`**: `OAuthButtons.vue` opens a popup
  and communicates with it via `BroadcastChannel`. A specific Safari bug
  was already observed here (popup silent in normal browsing, never
  reproduced since without a confirmed cause — see the JOURNAL).
  `BroadcastChannel` was just made tolerant of missing support (direct
  full-page fallback): need to check this fallback never triggers by
  mistake on a browser that supports it fine.
- **`navigator.clipboard`** (copying a room's invite link): already has
  a fallback (silent `catch`, the link stays selectable), but never seen
  actually fail or succeed on Firefox/Safari.
- **WebSocket + playback sync**: the sync engine (`playbackSync.js`)
  directly manipulates `<video>` (play/pause/seek, `playbackRate`). Each
  browser has its own implementation of the `play`/`pause`/`seeked`
  events and of `currentTime` precision — the grace and correction
  window (see the manual) was calibrated by observing Chrome/Brave/
  Safari, never Firefox.
- **Avatar upload with progress bar**: `XMLHttpRequest.upload.onprogress`,
  added 2026-10-30, never visually checked anywhere.
- **Design (CSS)**: CSS variables, `aspect-ratio`, possible
  `backdrop-filter` — by eye, no automated check possible here.

## Grid to fill in

One row per browser tested. Put ✅ / ❌ / ⚠️ (partial, note details) in
each column.

| Browser (version) | Signup/Login | OAuth popup | Search + filters | Avatar upload + bar | Room: join by code | Chat | Video sync (ongoing playback) | Video sync (joining mid-playback) | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Firefox | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Pass done 2026-11-04, confirmed by the user for real on their machine |
| Safari | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Pass done 2026-11-04, confirmed by the user for real on their machine |

## How to test video sync (the easiest point to test wrong)

A single tab isn't enough: you need two sessions watching the same room
at the same time, otherwise there's nothing to synchronize.

1. Open the same room in **two different browsers** (or the same
   browser in normal + private browsing, for two accounts), one owner
   account and one guest.
2. Owner: pick a video, start playback.
3. Guest: should see the "Join the ongoing video" button (no
   autoplay) — click it, check that playback starts at the right
   position.
4. Pause on the owner's side: the guest should follow within a few
   seconds, with no action on their part.
5. Seek manually on the guest's side (drag the progress bar): the owner
   should follow too.
6. Owner leaves the room (guest stays): re-check that the guest becomes
   owner (can change the video) and that nothing restarts from zero.

## Once the grid is filled in

- Update the README's "Modules" table (`Support for additional
  browsers`: In progress → Done, with a one-sentence summary).
- If a real browser-specific bug is found: fix it first, then only mark
  the module as acquired — an incomplete or buggy module is worth 0
  points, not half credit.
