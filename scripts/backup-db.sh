#!/bin/sh

set -eu

# Configuration
BACKUP_DIR="/backups"
RETENTION_DAYS="${BACKUP_RETENTION_DAYS:-7}"

# Date et heure de la sauvegarde, affichées dans le fuseau de Paris
# (le décalage CET/CEST est appliqué automatiquement).
TIMESTAMP=$(TZ=Europe/Paris date +"%Y-%m-%d_%H-%M-%S")

BACKUP_FILE="$BACKUP_DIR/postgres_$TIMESTAMP.dump"
TEMP_FILE="$BACKUP_FILE.tmp"
UPLOADS_FILE="$BACKUP_DIR/uploads_$TIMESTAMP.tar.gz"
UPLOADS_TEMP_FILE="$UPLOADS_FILE.tmp"

# Remove incomplete temporary files if any backup step fails or is interrupted.
cleanup() {
    rm -f "$TEMP_FILE" "$UPLOADS_TEMP_FILE"
}
trap cleanup 0
trap 'exit 1' HUP INT TERM

# Création du dossier si nécessaire
mkdir -p "$BACKUP_DIR"

# Permissions restrictives sur les fichiers
umask 077

echo "Starting PostgreSQL backup..."

# Export de la base de données
pg_dump -Fc \
    --no-owner \
    --no-acl \
    -f "$TEMP_FILE"

# Vérification de l'intégrité du dump
pg_restore --list "$TEMP_FILE" > /dev/null

# Archive uploaded files (avatars) with the same timestamp as the database.
tar -czf "$UPLOADS_TEMP_FILE" -C /uploads .
tar -tzf "$UPLOADS_TEMP_FILE" > /dev/null

# Publish both files only after both temporary backups have been validated.
mv "$UPLOADS_TEMP_FILE" "$UPLOADS_FILE"
mv "$TEMP_FILE" "$BACKUP_FILE"

echo "Backup successfully created: $BACKUP_FILE"
echo "Uploads backup successfully created: $UPLOADS_FILE"

# Suppression des anciennes sauvegardes
find "$BACKUP_DIR" \
    -type f \
    -name "postgres_*.dump" \
    -mtime +"$RETENTION_DAYS" \
    -delete

find "$BACKUP_DIR" \
    -type f \
    -name "uploads_*.tar.gz" \
    -mtime +"$RETENTION_DAYS" \
    -delete

echo "Backup cleanup completed."
