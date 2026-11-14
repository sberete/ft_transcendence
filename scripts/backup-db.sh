
#!/bin/sh

set -eu

# Configuration
BACKUP_DIR="/backups"
RETENTION_DAYS="${BACKUP_RETENTION_DAYS:-7}"

# Date et heure de la sauvegarde
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")

BACKUP_FILE="$BACKUP_DIR/postgres_$TIMESTAMP.dump"
TEMP_FILE="$BACKUP_FILE.tmp"

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

# Publication du fichier final
mv "$TEMP_FILE" "$BACKUP_FILE"

echo "Backup successfully created: $BACKUP_FILE"

# Suppression des anciennes sauvegardes
find "$BACKUP_DIR" \
    -type f \
    -name "postgres_*.dump" \
    -mtime +"$RETENTION_DAYS" \
    -delete

echo "Backup cleanup completed."
