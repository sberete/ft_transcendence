# Monitoring with Prometheus and Grafana

What was set up to monitor SyncWatch (major DevOps Monitoring module).

| Subject requirement | Implementation |
|---|---|
| Prometheus collects metrics | backend, PostgreSQL exporter and Prometheus scraped, targets `UP` |
| Exporters / integrations | Axum metrics endpoint + `postgres_exporter` |
| Custom Grafana dashboard | versioned *SyncWatch* dashboard, with room metrics |
| Alerting rules | 6 rules, tested at build time, visible during a simulated outage |
| Secured Grafana | login required, secrets in `.env`, local port only |

## Overview

```text
Rust backend ──(:9091/metrics)──┐
                                ├──> Prometheus ──> Grafana (dashboards)
postgres_exporter ──(/metrics)──┘        │
                                         └──> alerting rules
```

| Service | Internal address | From the host | Role |
|---|---|---|---|
| `backend` | `backend:9091/metrics` | not published | HTTP and room metrics |
| `postgres-exporter` | `postgres-exporter:9187/metrics` | not published | PostgreSQL statistics |
| `monitoring-db-init` | — | — | one-shot: creates the `monitoring` role, then exits |
| `prometheus` | `prometheus:9090` | http://localhost:9090 | scrapes every 15 s, 7-day retention, alerts |
| `grafana` | `grafana:3000` | http://localhost:3001 | dashboard, login required |

Pinned image versions: `prom/prometheus:v3.5.0`, `grafana/grafana:12.2.0`, `quay.io/prometheuscommunity/postgres-exporter:v0.17.1`.

## Configuration (`.env`)

| Variable | Role |
|---|---|
| `METRICS_PORT` | backend metrics port (9091 by default) |
| `MONITORING_DB_PASSWORD` | password of the `monitoring` role, different from `POSTGRES_PASSWORD` |
| `GRAFANA_ADMIN_USER`, `GRAFANA_ADMIN_PASSWORD` | Grafana admin account |

For a teammate pulling the project:

```sh
echo "MONITORING_DB_PASSWORD=$(openssl rand -hex 24)" >> .env
printf 'GRAFANA_ADMIN_USER=admin\nGRAFANA_ADMIN_PASSWORD=%s\n' "$(openssl rand -hex 16)" >> .env
```

Grafana only applies `GRAFANA_ADMIN_PASSWORD` when the `grafana_data` volume is created. To change the password afterwards (then copy the value into `.env`):

```sh
docker compose exec grafana grafana cli admin reset-admin-password 'new_password'
```

## Metrics

| Metric | Source | Measures |
|---|---|---|
| `http_requests_total` | backend | request counter by `method`, `route`, `status` |
| `http_request_duration_seconds` | backend | duration histogram (buckets from 5 ms to 10 s, for p95) |
| `http_requests_in_flight` | backend | requests in progress |
| `syncwatch_active_rooms` | backend | rooms with at least one connected person |
| `syncwatch_room_members` | backend | connected people, across all rooms |
| `pg_up` | exporter | `1` if the exporter reaches the database |
| `pg_stat_activity_count`, `pg_stat_database_numbackends` | exporter | connections per database and state |
| `pg_stat_database_xact_commit` / `_rollback` | exporter | committed / rolled back transactions |
| `pg_database_size_bytes`, `pg_settings_max_connections` | exporter | database size, maximum connections |
| `up{job=...}` | Prometheus | `1` if scraping the target (`prometheus`, `backend`, `postgres`) succeeds |

## Key decisions

- **Separate metrics port (9091), not published.** nginx forwards all of `/api/*` to the main port: a `/metrics` route there would be public. On a separate port nginx cannot reach it, and its config did not change.
- **Bounded labels.** `route` is the route template (`/rooms/{room_id}`), never the actual URL. 404s are not counted, non-standard methods are grouped as `OTHER`, `/uploads/...` as `/uploads`. No per-room or per-user label: no series explosion and no leaked identifiers.
- **Room gauges updated in `RoomRegistry::join` / `leave`**, while the registry is still locked: they are always exact. Published as `0` at startup ("0" instead of "No data").
- **Read-only PostgreSQL `monitoring` role**: only `pg_monitor`, no superuser, no table access. It is created by `monitoring-db-init`, because `/docker-entrypoint-initdb.d` only runs when the volume is created. The script is idempotent and resets the password to the `.env` value (read with `\getenv`, never on the command line). Roles are not in `pg_dump`: a restore does not touch them.
- **Config copied into the images, not mounted as a volume.** Prometheus (`nobody`) and Grafana (user `472`) cannot traverse the home directory, which is `drwx------` and must not be opened up. The build runs `promtool check config` and `promtool test rules`: an error makes `make up` fail. Any change requires a rebuild (`make up` does it).
- **Ports published on `127.0.0.1` only**, nothing in nginx. Prometheus has no authentication.
- **Secured Grafana**: `GF_AUTH_ANONYMOUS_ENABLED=false`, `GF_USERS_ALLOW_SIGN_UP=false`, unique password from `.env` (`admin`/`admin` rejected), no outbound calls (usage stats, update checks, Gravatar). Port 3001 because 3000 is taken by `make dev`.
- **Provisioned data source**: `uid: prometheus`, `editable: false`, identical on every machine.
- **No `depends_on` for Prometheus**: a missing target is simply `DOWN`, and podman can recreate the backend without removing Prometheus.
- **If port 9091 is taken**, the backend stops at startup with a clear error.

## Dashboard

`docker/grafana/dashboards/syncwatch.json`, loaded at startup into the *SyncWatch* folder. Refreshes every 10 s, last 30 minutes by default, one description (ⓘ) per panel. Panel titles are in French.

- **Vue d'ensemble** (overview): Backend (`up{job="backend"}`), Base de données (`pg_up`) and Exporter PostgreSQL (`up{job="postgres"}`) tiles in green/red; requests/s; 5xx errors (orange from 1 %, red from 5 %); p95 latency (orange from 0.5 s, red from 1 s); active rooms; people in rooms; **Alertes en cours** (current alerts) table.
- **Backend HTTP**: requests per route, responses per status class, p50/p95/p99 latency, rooms and in-flight requests.
- **PostgreSQL**: connections against the maximum, transactions per second, database size, share of reads served from cache.

The JSON in the repository is the source of truth (`allowUiUpdates: false`). To keep a change made in the UI: **Export → Export as JSON**, replace the file, rebuild Grafana.

## Alerts

`docker/prometheus/alerts.yml`; each rule has a severity, a summary and a description saying what to check.

| Alert | Condition | Delay | Severity |
|---|---|---|---|
| `BackendDown` | backend scrape fails | 1 min | critical |
| `PostgresExporterDown` | exporter scrape fails | 1 min | warning |
| `DatabaseDown` | `pg_up == 0` | 1 min | critical |
| `HighErrorRate` | more than 5 % 5xx over 5 min, with at least 3 requests/min | 5 min | warning |
| `HighLatencyP95` | p95 above 1 s | 5 min | warning |
| `PostgresConnectionsHigh` | more than 80 % of `max_connections` | 5 min | warning |

States: `inactive` → `pending` (condition true for less than the delay) → `firing`. The delay ignores short restarts; the minimum traffic stops a single error from counting as a 100 % error rate. Alerts are visible in Prometheus (**Alerts**) and on the dashboard, with no notifications (that would require Alertmanager).

`docker/prometheus/alerts.test.yml` checks that `BackendDown` is `pending` at 1 min then `firing` at 2 min, and that `HighErrorRate` fires at 14 % errors, but neither at 1 % nor for a single error on an almost idle server. **The test compares `summary` and `description` verbatim**: any change to these texts in `alerts.yml` must be copied into the test file.

## Files

| File | Contents |
|---|---|
| `src/monitoring/mod.rs` | `install()`, `serve()` (port 9091), `track()` middleware, room gauges, `set_live_rooms()` |
| `src/server/mod.rs` | `install()` and `serve()` at startup, `.route_layer(axum::middleware::from_fn(monitoring::track))` |
| `src/config/mod.rs` | `metrics_port` field (`METRICS_PORT`) |
| `src/ws/registry.rs` | `publish_live_counts()` after each `join` and `leave` |
| `src/main.rs`, `Cargo.toml`, `Cargo.lock` | `monitoring` module; `metrics` and `metrics-exporter-prometheus` crates (without its HTTP server) |
| `docker/backend/Dockerfile` | `EXPOSE 9091` (documentation only, nothing is published) |
| `scripts/monitoring-db-init.sh` | creates / updates the `monitoring` role |
| `docker/prometheus/` | `prometheus.yml` (targets, `rule_files`), `alerts.yml`, `alerts.test.yml`, `Dockerfile` |
| `docker/grafana/` | `provisioning/datasources/prometheus.yml`, `provisioning/dashboards/syncwatch.yml`, `dashboards/syncwatch.json`, `Dockerfile` |
| `docker-compose.yml` | the 4 services above; volumes `prometheus_data`, `grafana_data` |
| `.env.example` | variables from the Configuration section |

## Checking

`make up` starts everything. To (re)start a single component:

```sh
docker compose up -d --build backend
docker compose up -d --no-deps monitoring-db-init postgres-exporter
docker compose build prometheus grafana && docker compose up -d --no-deps prometheus grafana
```

The backend image has no `curl`: internal metrics are read from the `backup` container (`wget`).

```sh
# Backend
curl -k https://localhost:8080/api/health                                          # generates a request
docker compose exec backup wget -qO- http://backend:9091/metrics | grep http_requests_total
curl -k -o /dev/null -w "%{http_code}\n" https://localhost:8080/api/metrics       # 404: not public

# PostgreSQL
docker compose logs monitoring-db-init                                             # "Monitoring role ready."
podman ps -a --format "{{.Names}}\t{{.Status}}" | grep monitoring-db-init         # Exited (0)
docker compose exec backup wget -qO- http://postgres-exporter:9187/metrics | grep '^pg_up'   # pg_up 1
docker compose exec backup wget -qO- http://postgres-exporter:9187/metrics | grep '^pg_database_size_bytes'
docker compose exec backup psql -c "SELECT has_table_privilege('monitoring','users','SELECT') AS can_read_users, rolsuper FROM pg_roles WHERE rolname='monitoring';"   # f | f

# Prometheus
curl -s http://localhost:9090/api/v1/targets | grep -o '"health":"[a-z]*"'         # "up" 3 times
curl -s --get http://localhost:9090/api/v1/query --data-urlencode 'query=up'

# Grafana
curl -s http://localhost:3001/api/health                                            # "database": "ok"
curl -s -o /dev/null -w '%{http_code}\n' http://localhost:3001/api/search           # 401 without login
curl -s -o /dev/null -w '%{http_code}\n' -u admin:admin http://localhost:3001/api/search   # 401

# Alert tests, without rebuilding
docker compose run --rm --no-deps --entrypoint promtool prometheus test rules /etc/prometheus/alerts.test.yml
```

**Prometheus** (http://localhost:9090): **Status → Target health** shows 3 `UP` targets. In **Query**: `up` (3 rows at 1), `sum by (route) (http_requests_total)`, `pg_database_size_bytes{datname="transcendence"}`, `histogram_quantile(0.95, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))`.

**Grafana** (http://localhost:3001): **Connections → Data sources → Prometheus → Test** shows "Successfully queried the Prometheus API". In **Dashboards → SyncWatch → SyncWatch**, tiles are green and "Aucune alerte en cours" (no current alerts) is shown; browsing the site moves the request graphs, and opening a room sets "Salles actives" and "Personnes en salle" to 1.

## Demo: controlled outage

1. Show the normal state: green tiles, no alerts, graphs moving while browsing and opening a room.
2. `docker compose stop backend`: after ~15 s the Backend tile turns red and `BackendDown` is `pending` (http://localhost:9090/alerts); after ~75 s it is `firing` and appears in "Alertes en cours".
3. `docker compose start backend`: at the next scrape, the alert goes back to `inactive` and the tile to UP.

The same demo works with `docker compose stop postgres-exporter` (`PostgresExporterDown`).

## Troubleshooting

**Podman keeps the old image after a rebuild.** If `docker compose logs backend | grep "Serving metrics"` returns nothing, remove the containers (frontend first, it depends on the backend; none of them hold data) and start them again:

```sh
docker compose build backend prometheus grafana
podman rm -f ft_transcendence_good_frontend_1 ft_transcendence_good_backend_1 ft_transcendence_good_prometheus_1 ft_transcendence_good_grafana_1
docker compose up -d --no-deps backend frontend prometheus grafana
```

**`pg_up 0`**: read `docker compose logs postgres-exporter`.

- `password authentication failed`: the `.env` password changed. Run `docker compose up -d --no-deps monitoring-db-init`, then `docker compose restart postgres-exporter`.
- `connection refused`: the database is not running.
- The `Error loading config ... postgres_exporter.yml` warning at startup is harmless.

**With `make dev`**, only `db` and `frontend` run: start the exporter with `docker compose up -d --no-deps monitoring-db-init postgres-exporter`. The native backend serves `http://localhost:9091/metrics`, but the `backend:9091` target stays `DOWN`: full monitoring is tested with `make up`.
