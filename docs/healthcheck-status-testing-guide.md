# Health check and status page

## How it works

- **PostgreSQL Docker health check**: `pg_isready` every 5 s (5 s timeout), `unhealthy` after 5 consecutive failures. The backend waits for `db` to be `healthy` before starting.
- **Backend `/health` endpoint**: runs `SELECT 1` through the application's PostgreSQL pool. Reachable at `/api/health`, because nginx strips the `/api` prefix.

| Case | Code | Body |
|---|---|---|
| `SELECT 1` succeeds | `200` | `{"status":"ok","database":"ok"}` |
| `SELECT 1` fails | `503` | `{"status":"degraded","database":"unavailable"}` |
| Backend unreachable (through nginx) | `502` | nginx HTML error page |

This check does not test nginx, the frontend, migrations or video sources: only the HTTP path to the handler and one simple PostgreSQL query.

**`/status` page** (`https://localhost:8080/status`, view `frontend/src/views/StatusView.vue`, text in `frontend/src/locales/`): calls `/api/health` without cache and shows "Checking…", then "The service is up and running." (successful HTTP response and `status: "ok"`) or "The service is unavailable." (any other case, including an HTML `502`). **Retry** button, no automatic refresh.

## Testing

From the repository root. `-k` because the local certificate is self-signed. Outage tests **locally only**; restart services with `docker compose start`, **never `make clean`** (it deletes the volumes, and so the data).

### With `make up`

The backend port is not published on the host: everything goes through nginx.

```sh
make up
docker compose ps                                                                 # db healthy, backend and frontend Up
docker inspect --format '{{.State.Health.Status}}' "$(docker compose ps -q db)"   # healthy
curl -k -i https://localhost:8080/api/health                                      # 200 {"status":"ok","database":"ok"}
```

`https://localhost:8080/status` must show "The service is up and running.".

PostgreSQL outage:

```sh
docker compose stop db
curl -k -i https://localhost:8080/api/health    # 503; /status: "unavailable"
docker compose start db
docker compose ps                               # wait for healthy, then /api/health returns 200 again
```

Backend outage:

```sh
docker compose stop backend
curl -k -i https://localhost:8080/api/health    # 502; /status stays reachable and shows "unavailable"
docker compose start backend
docker compose ps                               # then call /api/health again and click Retry
```

### With `make dev`

The backend runs on the host (`cargo run`, port `3000`); `db` and `frontend` stay in containers. Leave `make dev` running and test from a second terminal. The Makefile stops the Compose `backend` service on purpose: this is expected in `docker compose ps`.

```sh
curl -i http://localhost:3000/health                                              # 200: native backend + PostgreSQL
curl -k -i https://localhost:8080/api/health                                      # same result through nginx
docker compose ps
docker inspect --format '{{.State.Health.Status}}' "$(docker compose ps -q db)"   # healthy
```

PostgreSQL outage:

```sh
docker compose stop db
curl -i http://localhost:3000/health    # 503 {"status":"degraded","database":"unavailable"}
docker compose start db
docker compose ps                       # wait for healthy, then try again
```

## Troubleshooting

**`Address already in use` when starting `make dev`**: an old `cargo run` instance holds port 3000 (`make down` does not stop it). Stop it with Ctrl+C in its terminal, then run `make dev` again.

```sh
lsof -nP -iTCP:3000 -sTCP:LISTEN
```

**`:3000/health` responds but `/api/health` returns `502`**: backend and database work, the problem is between nginx and the native backend. Under Podman, `host.docker.internal could not be resolved` in the logs means `BACKEND_UPSTREAM` (dev config) does not point to an address nginx can reach.

```sh
docker compose logs --tail 50 frontend
```

**`/api/health` returns `503`**: the backend responds, but its `SELECT 1` fails. Check that the database is running and `healthy`:

```sh
docker compose ps
docker compose logs --tail 50 db
```

**`curl` returns `200` but the page says "unavailable"**: check it is the same URL (`https://localhost:8080/api/health`), reload the bundle, and inspect the request in the browser's **Network** tab.

## URL summary

| Mode | Backend directly | Through nginx | Page |
|---|---|---|---|
| `make up` | not published | `https://localhost:8080/api/health` | `https://localhost:8080/status` |
| `make dev` | `http://localhost:3000/health` | `https://localhost:8080/api/health` (if the upstream resolves) | `https://localhost:8080/status` (same condition) |
