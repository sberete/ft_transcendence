# Backup and restore

SyncWatch backs up its PostgreSQL database and its uploads (avatars), and provides a restore script. Run every command from the repository root, with the stack running (`make up`).

## How it works

**`scripts/backup-db.sh`** runs in the `backup` service (PostgreSQL image): at startup, then every 24 h, retrying 5 min after a failure. Each run produces a timestamped pair (Paris time) in `/backups` (volume `backup_data`):

- `postgres_<timestamp>.dump`: the database, in `pg_dump` custom format;
- `uploads_<timestamp>.tar.gz`: the uploads folder.

Files are written as `.tmp`, checked (`pg_restore --list`, `tar -tzf`), then published: a half-written file is never left behind. Pairs older than 7 days are deleted (`BACKUP_RETENTION_DAYS`).

**`scripts/restore.sh`** runs from the host and goes through the `backup` container. It:

1. checks the pair (dump readable and containing tables, archive readable);
2. takes a **safety backup** of the current state, which makes the restore reversible;
3. stops the backend;
4. restores the database **in a single transaction** (on error, the current database stays intact);
5. replaces the uploads;
6. restarts the backend, even if a step failed.

Nothing is modified before step 3.

## 1. Take a backup

```sh
docker compose ps                      # backup must be "Up"
docker compose logs --tail 30 backup   # "Backup successfully created: ..."

docker compose exec backup sh /scripts/backup-db.sh   # immediate manual backup
docker compose up -d backup                           # if backup is not running (make dev); backs up right away
```

## 2. List and compare backups

```sh
docker compose exec backup ls -lht /backups
```

Number of users and uploaded files in each pair:

```sh
docker compose exec backup sh -c 'for f in /backups/postgres_*.dump; do ts=${f#/backups/postgres_}; ts=${ts%.dump}; u=$(pg_restore --data-only --table=users -f - "$f" | awk "/^COPY /{c=1;next} /^\\\\[.]/{c=0} c{n++} END{print n+0}"); a=$(tar -tzf /backups/uploads_$ts.tar.gz | grep -vc "/$"); echo "$ts   users=$u   files=$a"; done'
```

```text
2026-11-21_09-00-02   users=0    files=0
2026-11-22_09-00-03   users=12   files=9
```

Current state, for comparison:

```sh
docker compose exec backup psql -c "SELECT count(*) FROM users;"
docker compose exec backup sh -c 'find /uploads -type f | wc -l'
```

## 3. Choose a backup

- The **last one before the incident** (timestamps are in Paris time).
- Always a **complete pair**: `.dump` and `.tar.gz` with the same timestamp.
- Avoid a pair with `users=0` if the site had accounts (the first backup after a fresh install is empty; the script refuses a dump without tables anyway).

## 4. Read a backup's contents (without changing anything)

Store the chosen timestamp in a shell variable, then run the commands **in the same terminal**:

```sh
TS=2026-11-22_15-41-17    # replace with your timestamp
echo $TS                  # check: prints 2026-11-22_15-41-17
```

In a new terminal the variable no longer exists: set it again.

Database:

```sh
docker compose exec backup pg_restore --list /backups/postgres_$TS.dump                          # table of contents
docker compose exec backup pg_restore --data-only --table=users -f - /backups/postgres_$TS.dump  # contents of one table
docker compose exec -T backup pg_restore --no-owner --no-acl -f - /backups/postgres_$TS.dump > backup_$TS.sql   # everything as SQL, on the host
```

SQL queries on a separate test database:

```sh
docker compose exec backup createdb restore_test
docker compose exec backup pg_restore --no-owner --no-acl -d restore_test /backups/postgres_$TS.dump
docker compose exec backup psql -d restore_test -c "SELECT id, username, email FROM users;"
docker compose exec backup dropdb restore_test
```

Uploads:

```sh
docker compose exec backup tar -tzf /backups/uploads_$TS.tar.gz

mkdir -p /tmp/syncwatch-uploads
docker compose exec -T backup cat /backups/uploads_$TS.tar.gz > /tmp/syncwatch-uploads/uploads.tar.gz
tar -xzf /tmp/syncwatch-uploads/uploads.tar.gz -C /tmp/syncwatch-uploads
ls /tmp/syncwatch-uploads/avatars
```

## 5. Restore

### Prepare a test

Change the site **after** the backup so there is a visible difference (for example, create a user with a picture on the site, and delete one in SQL), then note the state:

```sh
docker compose exec backup psql -c "DELETE FROM users WHERE username = 'ye';"
docker compose exec backup psql -c "SELECT id, username FROM users ORDER BY id;"
docker compose exec backup find /uploads -type f
```

After the restore, the deleted user is back and the new one is gone.

### With the script

```sh
sh scripts/restore.sh                       # lists available backups
sh scripts/restore.sh 2026-11-22_15-41-17   # restores this pair
sh scripts/restore.sh 2026-11-22_16-05-44   # undo: timestamp of the safety backup shown at step 2
```

### Check

```sh
curl -k https://localhost:8080/api/health    # {"status":"ok","database":"ok"}
docker compose exec backup psql -c "SELECT id, username FROM users ORDER BY id;"
docker compose exec backup find /uploads -type f
```

Then open `https://localhost:8080/status`, log in with an account and check that its avatar is displayed.

### By hand (without the script, or with `make dev`)

With `make dev`, stop the backend with Ctrl+C instead of `docker compose stop backend`, and start it again at the end.

```sh
TS=2026-11-22_15-41-17    # timestamp of the chosen pair

docker compose exec backup sh /scripts/backup-db.sh    # safety backup
docker compose stop backend

# Database: drop the schema and replay the dump, in a single transaction
docker compose exec backup sh -c "
  set -e
  pg_restore --no-owner --no-acl -f /tmp/restore.sql /backups/postgres_$TS.dump
  { echo 'DROP SCHEMA public CASCADE; CREATE SCHEMA public;'; cat /tmp/restore.sql; } \
    | psql -v ON_ERROR_STOP=1 --single-transaction -q -f -
  rm -f /tmp/restore.sql"

# Uploads: empty the folder and extract the archive
docker compose exec backup sh -c "
  set -e
  find /uploads -mindepth 1 -delete
  tar -xzf /backups/uploads_$TS.tar.gz -C /uploads"

docker compose start backend
```

### End-to-end demo

1. Create a `drtest` account with an avatar on the site.
2. `docker compose exec backup sh /scripts/backup-db.sh`, note the timestamp of the new pair.
3. Simulate the disaster (logging in as `drtest` then fails):
   ```sh
   docker compose exec backup psql -c "DELETE FROM users WHERE username = 'drtest';"
   docker compose exec backup sh -c "find /uploads -mindepth 1 -delete"
   ```
4. `sh scripts/restore.sh <timestamp noted at step 2>`.
5. Logging in as `drtest` works again and the avatar is displayed.

## If the machine is lost

Backups live on the same machine as the site: export a copy regularly.

```sh
docker compose exec -T backup tar -cf - -C /backups . > backups-export.tar
```

On a new machine: `git clone`, put `.env` back, `make up`, then:

```sh
docker compose exec -T backup tar -xf - -C /backups < backups-export.tar
sh scripts/restore.sh <timestamp>
```

## Limitations

- **Up to 24 h of data lost** (one backup per day).
- **First backup is empty** after a fresh install: `backup` starts before the backend creates the tables.
- **`make clean` also deletes the backups** (volume `backup_data`).
- **`.env` is not backed up**: keep the secrets elsewhere.

## Clean up after a test

```sh
docker compose exec backup dropdb --if-exists restore_test   # test database (section 4)
rm -f backup_*.sql                                            # exported SQL dump (section 4)
rm -rf /tmp/syncwatch-uploads                                 # extracted avatars (section 4)
```

Each restore creates a safety backup, deleted after 7 days. To delete it sooner (**check the timestamp**, this cannot be undone):

```sh
docker compose exec backup ls -lht /backups
docker compose exec backup rm /backups/postgres_<timestamp>.dump /backups/uploads_<timestamp>.tar.gz
```
