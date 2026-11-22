#!/bin/sh
# Creates (or updates) the read-only PostgreSQL role used by
# postgres_exporter. Run once by the `monitoring-db-init` service of
# docker-compose.yml, on every `make up`: safe to run again, it only
# creates the role if it's missing and resets its password from .env.
#
# The role only gets `pg_monitor` (PostgreSQL's built-in role for reading
# statistics): it can't read or modify the application's tables.

set -eu

: "${MONITORING_DB_PASSWORD:?MONITORING_DB_PASSWORD must be set in .env}"

# \getenv reads the password from the environment, so it never appears
# on psql's command line (visible in the process list).
psql -v ON_ERROR_STOP=1 <<'SQL'
\getenv monitoring_password MONITORING_DB_PASSWORD
SELECT 'CREATE ROLE monitoring LOGIN'
WHERE NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'monitoring')\gexec
ALTER ROLE monitoring WITH LOGIN PASSWORD :'monitoring_password';
GRANT pg_monitor TO monitoring;
SQL

echo "Monitoring role ready."
