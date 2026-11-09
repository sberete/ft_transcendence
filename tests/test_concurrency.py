"""Multi-user concurrency tests against the real Docker stack.

Complements the ad hoc recommendation-verification script: here we
specifically look for races (several clients acting at the same time on
the same resource), not just a sequential scenario.

Prerequisites: the stack must be running (`docker compose up`) and
reachable at https://localhost:8080. Requires the `websockets` package:

    python3 -m venv .venv && .venv/bin/pip install websockets
    .venv/bin/python3 tests/test_concurrency.py

Each scenario is independent: a failure in one doesn't stop the others
from running (see `run_all` at the bottom of the file).
"""

import asyncio
import json
import ssl
import urllib.error
import urllib.request
import uuid

import websockets

BASE = "https://localhost:8080/api"
WS_BASE = "wss://localhost:8080/api/ws"

# Self-signed certificate (see docs/browser-testing.md / the nginx reverse
# proxy): we don't verify the chain, same as the project's other test
# scripts.
SSL_CTX = ssl.create_default_context()
SSL_CTX.check_hostname = False
SSL_CTX.verify_mode = ssl.CERT_NONE


def http(method, path, token=None, body=None):
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(BASE + path, data=data, method=method)
    req.add_header("Content-Type", "application/json")
    if token:
        req.add_header("Authorization", f"Bearer {token}")
    try:
        with urllib.request.urlopen(req, context=SSL_CTX) as res:
            raw = res.read()
            return res.status, (json.loads(raw) if raw else None)
    except urllib.error.HTTPError as e:
        raw = e.read()
        return e.code, (json.loads(raw) if raw else None)


def signup(username, password="Sup3rSecret!"):
    return http("POST", "/auth/signup", body={
        "username": username, "email": f"{username}@example.com", "password": password,
    })


def login(username, password="Sup3rSecret!"):
    status, body = http("POST", "/auth/login", body={"identifier": username, "password": password})
    assert status == 200, (status, body)
    return body["token"]


def signup_login(prefix, suffix, password="Sup3rSecret!"):
    username = f"{prefix}_{suffix}"
    status, body = signup(username, password)
    assert status == 201, (status, body)
    return {"username": username, "token": login(username, password)}


class WsClient:
    """Small wrapper: connects, authenticates, and lets you pick server
    messages by type rather than reading everything in strict order
    (several events can arrive in a different order depending on the
    client, by the very nature of what's being tested here)."""

    def __init__(self, ws):
        self.ws = ws
        self.inbox = []

    @classmethod
    async def connect_and_auth(cls, token):
        # The server rejects any WebSocket connection whose Origin isn't
        # the known frontend's (see src/ws/handler.rs) - WebSockets don't
        # go through regular CORS, this manual check stands in for it.
        ws = await websockets.connect(WS_BASE, ssl=SSL_CTX, origin="https://localhost:8080")
        await ws.send(json.dumps({"type": "auth", "token": token}))
        raw = await asyncio.wait_for(ws.recv(), timeout=5)
        msg = json.loads(raw)
        assert msg["type"] == "auth.ok", msg
        return cls(ws)

    async def send(self, **payload):
        await self.ws.send(json.dumps(payload))

    async def recv(self, timeout=5):
        raw = await asyncio.wait_for(self.ws.recv(), timeout=timeout)
        msg = json.loads(raw)
        self.inbox.append(msg)
        return msg

    async def recv_until(self, type_, timeout=5):
        """Reads messages until `type_` is found, keeping the others in
        the inbox (we don't assume a fixed order between concurrent
        clients)."""
        async with asyncio.timeout(timeout):
            while True:
                msg = await self.recv(timeout=timeout)
                if msg["type"] == type_:
                    return msg

    async def drain(self, quiet_for=0.5):
        """Reads everything that arrives until `quiet_for` seconds pass
        with no new message - to give async broadcasts (room.member_joined)
        time to arrive before judging a client's final state."""
        while True:
            try:
                await self.recv(timeout=quiet_for)
            except asyncio.TimeoutError:
                return

    async def close(self):
        await self.ws.close()


async def test_concurrent_signup_same_username():
    """Two simultaneous signups with EXACTLY the same username: Postgres's
    unique constraint must let exactly one of the two requests through,
    never both, never neither (no deadlock)."""
    username = f"race_{uuid.uuid4().hex[:8]}"

    results = await asyncio.gather(*[
        asyncio.to_thread(signup, username) for _ in range(5)
    ])
    statuses = [status for status, _ in results]

    assert statuses.count(201) == 1, f"expected exactly 1 success, got {statuses}"
    assert statuses.count(409) == 4, f"expected 4 409 conflicts, got {statuses}"
    print("OK: concurrent signup with the same username -> exactly 1 winner, 4x 409")


async def test_concurrent_room_join():
    """4 users join the same room at the same time (not one after
    another): everyone must end up seeing a room with 4 members, with no
    duplicate and no one missing."""
    suffix = uuid.uuid4().hex[:8]
    owner = await asyncio.to_thread(signup_login, "owner", suffix)
    status, room = http("POST", "/rooms", token=owner["token"], body={"name": "Concurrency test"})
    assert status == 201, (status, room)
    code = room["code"]

    # 3, not more: every scenario in this file consumes logins that share
    # the same anti-bruteforce quota (10/60s per IP, see the JOURNAL docs
    # from 2026-10-29) - aiming to stay under 10 across the whole file so a
    # full run doesn't block itself.
    guests = await asyncio.gather(*[
        asyncio.to_thread(signup_login, f"guest{i}", suffix) for i in range(2)
    ])
    users = [owner, *guests]

    clients = await asyncio.gather(*[WsClient.connect_and_auth(u["token"]) for u in users])

    # All 4 send room.join AT THE SAME TIME (not sequentially), the whole
    # point of this test: if the server serializes joining the room
    # incorrectly, a member could end up missing from someone else's final
    # view, or duplicated.
    await asyncio.gather(*[c.send(type="room.join", code=code) for c in clients])

    states = await asyncio.gather(*[c.recv_until("room.state") for c in clients])

    # The server serializes room.join (a single Mutex on the registry, see
    # src/ws/registry.rs): `room.state.members` lists ONLY those already
    # present before me (see the comment in roomSocket.js:9-12, the
    # frontend adds itself) - so this snapshot's size varies depending on
    # the actual arrival order, not necessarily 0 or 3. What must stay true
    # for everyone: combining one's own snapshot + the room.member_joined
    # events received afterward + oneself, every client must end up
    # knowing exactly the 4 members, with no duplicate and none missing.
    await asyncio.gather(*[c.drain(quiet_for=1.0) for c in clients])
    for user, client, state in zip(users, clients, states):
        known_ids = {m["id"] for m in state["members"]}
        known_ids.update(
            msg["user"]["id"] for msg in client.inbox if msg["type"] == "room.member_joined"
        )
        status, me = http("GET", "/users/me", token=user["token"])
        assert status == 200, (status, me)
        known_ids.add(me["id"])
        assert len(known_ids) == 3, (
            f"{user['username']} only knows {len(known_ids)} member(s) out of 3: {known_ids}"
        )

    await asyncio.gather(*[c.close() for c in clients])
    print("OK: 3 concurrent room.join -> everyone ends up knowing all 3 members, no duplicate, none missing")


async def test_concurrent_ownership_transfer():
    """The owner leaves while a second event (a chat message from the
    other guest) arrives at EXACTLY the same moment: the room must neither
    disappear (members remain) nor end up without an owner, and the new
    owner must be the one present the longest (section 9.3 of
    docs/websocket.md)."""
    suffix = uuid.uuid4().hex[:8]
    owner = await asyncio.to_thread(signup_login, "owner2", suffix)
    guest1 = await asyncio.to_thread(signup_login, "guest1b", suffix)
    guest2 = await asyncio.to_thread(signup_login, "guest2b", suffix)

    status, room = http("POST", "/rooms", token=owner["token"], body={"name": "Ownership transfer test"})
    assert status == 201, (status, room)
    code = room["code"]

    c_owner = await WsClient.connect_and_auth(owner["token"])
    await c_owner.send(type="room.join", code=code)
    await c_owner.recv_until("room.state")

    # guest1 joins before guest2: they're the one who should become owner.
    c_guest1 = await WsClient.connect_and_auth(guest1["token"])
    await c_guest1.send(type="room.join", code=code)
    await c_guest1.recv_until("room.state")
    await c_owner.recv_until("room.member_joined")

    c_guest2 = await WsClient.connect_and_auth(guest2["token"])
    await c_guest2.send(type="room.join", code=code)
    await c_guest2.recv_until("room.state")
    await c_owner.recv_until("room.member_joined")
    await c_guest1.recv_until("room.member_joined")

    # The owner leaving and a guest's chat message are sent in the same
    # batch, without waiting on each other.
    await asyncio.gather(
        c_owner.send(type="room.leave"),
        c_guest2.send(type="chat.send", content="still there?"),
    )

    left_guest1 = await c_guest1.recv_until("room.member_left")
    left_guest2 = await c_guest2.recv_until("room.member_left")

    status, refreshed = http("GET", f"/rooms/{code}", token=guest1["token"])
    assert status == 200, (status, refreshed)
    assert refreshed["owner_id"] is not None, "the room ended up without an owner"
    assert left_guest1["new_owner_id"] == left_guest2["new_owner_id"], "mismatch between the two clients on the new owner"

    # guest1 joined before guest2: they're the oldest member present.
    guest1_id = (await asyncio.to_thread(http, "GET", "/users/me", guest1["token"]))[1]["id"]
    assert left_guest1["new_owner_id"] == guest1_id, (
        f"the new owner should be guest1 ({guest1_id}), "
        f"got {left_guest1['new_owner_id']}"
    )

    await asyncio.gather(c_guest1.close(), c_guest2.close())
    print("OK: concurrent ownership transfer with a simultaneous chat message -> consistent for everyone")


async def test_concurrent_playback_commands():
    """Two members (anyone can control playback, section 9.1) send play
    and pause AT THE SAME TIME at different positions: the server must
    settle on a SINGLE consistent final state, broadcast identically to
    everyone - not a different state per client, no crash."""
    suffix = uuid.uuid4().hex[:8]
    owner = await asyncio.to_thread(signup_login, "pbowner", suffix)
    guest = await asyncio.to_thread(signup_login, "pbguest", suffix)

    status, room = http("POST", "/rooms", token=owner["token"], body={"name": "Playback test"})
    assert status == 201, (status, room)
    code = room["code"]

    c_owner = await WsClient.connect_and_auth(owner["token"])
    await c_owner.send(type="room.join", code=code)
    await c_owner.recv_until("room.state")

    c_guest = await WsClient.connect_and_auth(guest["token"])
    await c_guest.send(type="room.join", code=code)
    await c_guest.recv_until("room.state")
    await c_owner.recv_until("room.member_joined")

    await asyncio.gather(
        c_owner.send(type="playback.play", position=10.0),
        c_guest.send(type="playback.pause", position=20.0),
    )

    state_owner = await c_owner.recv_until("playback.state")
    state_guest = await c_guest.recv_until("playback.state")

    # It doesn't matter which of the two "wins" (the server processes
    # messages in the order it receives them, no ambiguity server-side) -
    # what matters is that EVERYONE sees exactly the same final state.
    assert state_owner["playing"] == state_guest["playing"]
    assert state_owner["position"] == state_guest["position"]
    print(
        "OK: concurrent playback.play/pause -> identical final state for both clients "
        f"(playing={state_owner['playing']}, position={state_owner['position']})"
    )

    await asyncio.gather(c_owner.close(), c_guest.close())


async def test_rate_limit_burst():
    """A burst of more than 10 playback commands in under 5s (the limit
    from chap. 8 of docs/websocket.md): the server must reply with
    `rate_limited` without crashing the connection, not just close it
    abruptly or silently ignore it."""
    suffix = uuid.uuid4().hex[:8]
    user = await asyncio.to_thread(signup_login, "burst", suffix)
    status, room = http("POST", "/rooms", token=user["token"], body={"name": "Rate limit test"})
    assert status == 201, (status, room)

    c = await WsClient.connect_and_auth(user["token"])
    await c.send(type="room.join", code=room["code"])
    await c.recv_until("room.state")

    for i in range(15):
        await c.send(type="playback.seek", position=float(i))

    saw_rate_limited = False
    saw_states = 0
    for _ in range(15):
        msg = await c.recv(timeout=5)
        if msg["type"] == "error" and msg.get("code") == "rate_limited":
            saw_rate_limited = True
        elif msg["type"] == "playback.state":
            saw_states += 1

    assert saw_rate_limited, "no rate_limited error received despite the burst of 15 commands"
    assert saw_states <= 10, f"more than 10 commands accepted before the limit kicked in: {saw_states}"
    await c.close()
    print(f"OK: burst of 15 commands -> rate_limited triggered ({saw_states} accepted before the limit)")


async def run_all():
    scenarios = [
        test_concurrent_signup_same_username,
        test_concurrent_room_join,
        test_concurrent_ownership_transfer,
        test_concurrent_playback_commands,
        test_rate_limit_burst,
    ]
    failures = []
    for scenario in scenarios:
        try:
            await scenario()
        except Exception as exc:  # noqa: BLE001 - we want to keep going despite a failure
            failures.append((scenario.__name__, exc))
            print(f"FAILED: {scenario.__name__}: {exc!r}")

    print()
    if failures:
        print(f"{len(failures)}/{len(scenarios)} scenario(s) failed")
        raise SystemExit(1)
    print(f"ALL CONCURRENCY SCENARIOS PASSED ({len(scenarios)}/{len(scenarios)})")


if __name__ == "__main__":
    asyncio.run(run_all())
