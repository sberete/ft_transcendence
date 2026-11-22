#!/bin/sh
# Disaster recovery: restores a backup pair made by scripts/backup-db.sh
# (postgres_<TS>.dump + uploads_<TS>.tar.gz), through the backup container.
#
#   sh scripts/restore.sh               list available backups
#   sh scripts/restore.sh <TIMESTAMP>   restore one, e.g. 2026-11-22_13-50-31

set -eu
cd "$(dirname "$0")/.."

in_backup() { docker compose exec -T backup sh -c "$1" 2>/dev/null; }

if [ $# -eq 0 ]; then
    in_backup 'ls -1 /backups'
    exit 0
fi

TS="$1"
# Strict format: the value is inserted in the commands below.
echo "$TS" | grep -Eqx '[0-9]{4}-[0-9]{2}-[0-9]{2}_[0-9]{2}-[0-9]{2}-[0-9]{2}' \
    || { echo "Invalid timestamp: $TS" >&2; exit 1; }
DUMP="/backups/postgres_$TS.dump"
ARCHIVE="/backups/uploads_$TS.tar.gz"

echo "==> 1. Checking backup $TS"
in_backup "pg_restore --list $DUMP | grep -q ' TABLE public '" \
    || { echo "Dump missing, unreadable or empty: $DUMP" >&2; exit 1; }
in_backup "tar -tzf $ARCHIVE >/dev/null" \
    || { echo "Archive missing or unreadable: $ARCHIVE" >&2; exit 1; }

echo "==> 2. Safety backup of the current state"
# Retention disabled for this run, so it cannot delete the pair we restore.
in_backup 'BACKUP_RETENTION_DAYS=36500 sh /scripts/backup-db.sh' | grep 'postgres_'

echo "==> 3. Stopping the backend"
docker compose stop backend >/dev/null 2>&1
trap 'docker compose start backend >/dev/null 2>&1' EXIT   # restart even on failure

echo "==> 4. Restoring the database (single transaction: all or nothing)"
in_backup "set -e; exec 2>&1
    pg_restore --no-owner --no-acl -f /tmp/restore.sql $DUMP
    { echo 'SET client_min_messages = warning; DROP SCHEMA public CASCADE; CREATE SCHEMA public;'
      cat /tmp/restore.sql; } \
        | psql -v ON_ERROR_STOP=1 --single-transaction -q -f - >/dev/null
    rm -f /tmp/restore.sql"

echo "==> 5. Restoring uploads"
in_backup "set -e; exec 2>&1
    rm -rf /tmp/uploads && mkdir /tmp/uploads
    tar -xzf $ARCHIVE -C /tmp/uploads
    find /uploads -mindepth 1 -delete
    cp -a /tmp/uploads/. /uploads/ && rm -rf /tmp/uploads"

echo "==> 6. Restarting the backend"
echo "Done. Check https://localhost:8080/status"
