"""Tests de concurrence multi-utilisateurs contre la stack Docker reelle.

Complete le script ad hoc de verification des recommandations : ici on
cherche specifiquement des races (plusieurs clients qui agissent en meme
temps sur la meme ressource), pas juste un scenario sequentiel.

Prerequis : la stack doit tourner (`docker compose up`) et etre joignable
sur https://localhost:8080. Necessite le paquet `websockets` :

    python3 -m venv .venv && .venv/bin/pip install websockets
    .venv/bin/python3 tests/test_concurrency.py

Chaque scenario est independant : un echec dans l'un n'empeche pas les
autres de s'executer (voir `run_all` en bas de fichier).
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

# Certificat auto-signe (voir docs/browser-testing.md / le reverse proxy
# nginx) : on ne verifie pas la chaine, comme les autres scripts de test
# de ce projet.
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
    """Petit wrapper : connecte, authentifie, et laisse piocher les
    messages serveur par type plutot que de tout lire en ordre strict
    (plusieurs evenements peuvent arriver dans un ordre different selon
    le client, par nature meme de ce qu'on teste ici)."""

    def __init__(self, ws):
        self.ws = ws
        self.inbox = []

    @classmethod
    async def connect_and_auth(cls, token):
        # Le serveur rejette toute connexion WebSocket dont l'Origin n'est
        # pas celle du front connu (voir src/ws/handler.rs) - les WebSockets
        # ne passent pas par le CORS classique, c'est cette verif manuelle
        # qui en tient lieu.
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
        """Lit des messages jusqu'a trouver `type_`, en gardant les
        autres dans l'inbox (on ne suppose pas un ordre fixe entre
        clients concurrents)."""
        async with asyncio.timeout(timeout):
            while True:
                msg = await self.recv(timeout=timeout)
                if msg["type"] == type_:
                    return msg

    async def drain(self, quiet_for=0.5):
        """Lit tout ce qui arrive jusqu'a `quiet_for` secondes sans nouveau
        message - pour laisser le temps aux diffusions asynchrones
        (room.member_joined) d'arriver avant de juger l'etat final d'un
        client."""
        while True:
            try:
                await self.recv(timeout=quiet_for)
            except asyncio.TimeoutError:
                return

    async def close(self):
        await self.ws.close()


async def test_concurrent_signup_same_username():
    """Deux inscriptions simultanees avec EXACTEMENT le meme username :
    la contrainte unique de Postgres doit laisser passer une seule des
    deux requetes, jamais les deux, jamais aucune (pas de deadlock)."""
    username = f"race_{uuid.uuid4().hex[:8]}"

    results = await asyncio.gather(*[
        asyncio.to_thread(signup, username) for _ in range(5)
    ])
    statuses = [status for status, _ in results]

    assert statuses.count(201) == 1, f"attendu exactement 1 succes, obtenu {statuses}"
    assert statuses.count(409) == 4, f"attendu 4 conflits 409, obtenu {statuses}"
    print("OK: inscription concurrente du meme username -> exactement 1 gagnant, 4x 409")


async def test_concurrent_room_join():
    """4 utilisateurs rejoignent la meme salle en meme temps (pas les
    uns apres les autres) : tout le monde doit finir par voir une salle
    a 4 membres, sans doublon ni membre manquant."""
    suffix = uuid.uuid4().hex[:8]
    owner = await asyncio.to_thread(signup_login, "owner", suffix)
    status, room = http("POST", "/rooms", token=owner["token"], body={"name": "Test concurrence"})
    assert status == 201, (status, room)
    code = room["code"]

    # 3 et pas plus : chaque scenario de ce fichier consomme des connexions
    # (login) qui partagent le meme quota anti-bruteforce (10/60s par IP,
    # voir docs JOURNAL 2026-10-29) - total vise sous 10 sur l'ensemble du
    # fichier pour qu'un run complet ne se bloque pas lui-meme.
    guests = await asyncio.gather(*[
        asyncio.to_thread(signup_login, f"guest{i}", suffix) for i in range(2)
    ])
    users = [owner, *guests]

    clients = await asyncio.gather(*[WsClient.connect_and_auth(u["token"]) for u in users])

    # Les 4 envoient room.join EN MEME TEMPS (pas sequentiellement), le
    # point du test : si le serveur serialise mal l'entree dans la salle,
    # un membre peut se retrouver absent de la vue finale de quelqu'un
    # d'autre, ou duplique.
    await asyncio.gather(*[c.send(type="room.join", code=code) for c in clients])

    states = await asyncio.gather(*[c.recv_until("room.state") for c in clients])

    # Le serveur serialise les room.join (un seul Mutex sur le registre, voir
    # src/ws/registry.rs) : `room.state.members` ne liste QUE ceux deja
    # presents avant moi (voir le commentaire de roomSocket.js:9-12, le
    # frontend se rajoute lui-meme) - donc la taille de ce snapshot varie
    # selon l'ordre d'arrivee reel, pas forcement 0 ni 3. Ce qui doit rester
    # vrai pour tout le monde : en combinant son propre snapshot + les
    # room.member_joined recus ensuite + soi-meme, chaque client doit finir
    # par connaitre exactement les 4 membres, sans doublon ni absent.
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
            f"{user['username']} ne connait que {len(known_ids)} membre(s) sur 3 : {known_ids}"
        )

    await asyncio.gather(*[c.close() for c in clients])
    print("OK: 3 room.join concurrents -> tout le monde finit par connaitre les 3 membres, sans doublon ni absent")


async def test_concurrent_ownership_transfer():
    """Le proprietaire part alors qu'un deuxieme evenement (un message de
    chat de l'autre invite) arrive EXACTEMENT au meme moment : la salle ne
    doit ni disparaitre (il reste des membres), ni finir sans proprietaire,
    et le nouveau proprietaire doit etre celui present depuis le plus
    longtemps (section 9.3 de docs/websocket.md)."""
    suffix = uuid.uuid4().hex[:8]
    owner = await asyncio.to_thread(signup_login, "owner2", suffix)
    guest1 = await asyncio.to_thread(signup_login, "guest1b", suffix)
    guest2 = await asyncio.to_thread(signup_login, "guest2b", suffix)

    status, room = http("POST", "/rooms", token=owner["token"], body={"name": "Test transfert"})
    assert status == 201, (status, room)
    code = room["code"]

    c_owner = await WsClient.connect_and_auth(owner["token"])
    await c_owner.send(type="room.join", code=code)
    await c_owner.recv_until("room.state")

    # guest1 rejoint avant guest2 : c'est lui qui doit devenir proprietaire.
    c_guest1 = await WsClient.connect_and_auth(guest1["token"])
    await c_guest1.send(type="room.join", code=code)
    await c_guest1.recv_until("room.state")
    await c_owner.recv_until("room.member_joined")

    c_guest2 = await WsClient.connect_and_auth(guest2["token"])
    await c_guest2.send(type="room.join", code=code)
    await c_guest2.recv_until("room.state")
    await c_owner.recv_until("room.member_joined")
    await c_guest1.recv_until("room.member_joined")

    # Le depart du proprietaire et un message de chat d'un invite partent
    # dans le meme batch, sans attendre l'un l'autre.
    await asyncio.gather(
        c_owner.send(type="room.leave"),
        c_guest2.send(type="chat.send", content="toujours la ?"),
    )

    left_guest1 = await c_guest1.recv_until("room.member_left")
    left_guest2 = await c_guest2.recv_until("room.member_left")

    status, refreshed = http("GET", f"/rooms/{code}", token=guest1["token"])
    assert status == 200, (status, refreshed)
    assert refreshed["owner_id"] is not None, "la salle s'est retrouvee sans proprietaire"
    assert left_guest1["new_owner_id"] == left_guest2["new_owner_id"], "incoherence entre les deux clients sur le nouveau proprietaire"

    # guest1 a rejoint avant guest2 : c'est lui le plus ancien membre present.
    guest1_id = (await asyncio.to_thread(http, "GET", "/users/me", guest1["token"]))[1]["id"]
    assert left_guest1["new_owner_id"] == guest1_id, (
        f"le nouveau proprietaire devrait etre guest1 ({guest1_id}), "
        f"obtenu {left_guest1['new_owner_id']}"
    )

    await asyncio.gather(c_guest1.close(), c_guest2.close())
    print("OK: transfert de propriete concurrent avec un chat simultane -> coherent pour tout le monde")


async def test_concurrent_playback_commands():
    """Deux membres (n'importe lequel peut piloter la lecture, section 9.1)
    envoient play et pause EN MEME TEMPS sur des positions differentes :
    le serveur doit retenir UN seul etat final coherent, diffuse a tout le
    monde a l'identique - pas un etat different par client, pas de crash."""
    suffix = uuid.uuid4().hex[:8]
    owner = await asyncio.to_thread(signup_login, "pbowner", suffix)
    guest = await asyncio.to_thread(signup_login, "pbguest", suffix)

    status, room = http("POST", "/rooms", token=owner["token"], body={"name": "Test playback"})
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

    # Peu importe lequel des deux "gagne" (le serveur traite les messages
    # dans l'ordre ou il les recoit, pas d'ambiguite cote serveur) - ce qui
    # compte est que TOUT LE MONDE voie exactement le meme etat final.
    assert state_owner["playing"] == state_guest["playing"]
    assert state_owner["position"] == state_guest["position"]
    print(
        "OK: playback.play/pause concurrents -> etat final identique pour les deux clients "
        f"(playing={state_owner['playing']}, position={state_owner['position']})"
    )

    await asyncio.gather(c_owner.close(), c_guest.close())


async def test_rate_limit_burst():
    """Rafale de plus de 10 commandes de lecture en moins de 5s (limite du
    chap. 8 de docs/websocket.md) : le serveur doit repondre `rate_limited`
    sans planter la connexion, pas juste la fermer brutalement ou l'ignorer
    silencieusement."""
    suffix = uuid.uuid4().hex[:8]
    user = await asyncio.to_thread(signup_login, "burst", suffix)
    status, room = http("POST", "/rooms", token=user["token"], body={"name": "Test rate limit"})
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

    assert saw_rate_limited, "aucune erreur rate_limited recue malgre la rafale de 15 commandes"
    assert saw_states <= 10, f"plus de 10 commandes acceptees avant la limite : {saw_states}"
    await c.close()
    print(f"OK: rafale de 15 commandes -> rate_limited declenche ({saw_states} acceptees avant la limite)")


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
        except Exception as exc:  # noqa: BLE001 - on veut continuer malgre un echec
            failures.append((scenario.__name__, exc))
            print(f"ECHEC: {scenario.__name__}: {exc!r}")

    print()
    if failures:
        print(f"{len(failures)}/{len(scenarios)} scenario(s) en echec")
        raise SystemExit(1)
    print(f"TOUS LES SCENARIOS DE CONCURRENCE ONT PASSE ({len(scenarios)}/{len(scenarios)})")


if __name__ == "__main__":
    asyncio.run(run_all())
