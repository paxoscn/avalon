#!/bin/bash
# Database restore script
# Requirement 1.2: Implement database recovery

set -e

# Configuration
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-3306}"
DB_NAME="${DB_NAME:-agent_platform}"
DB_USER="${DB_USER:-agent_user}"
DB_PASSWORD="${DB_PASSWORD:-agent_password}"

# Check if backup file is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <backup_file.sql.gz>"
    echo ""
    echo "Available backups:"
    ls -lh ./backups/agent_platform_*.sql.gz 2>/dev/null || echo "No backups found"
    exit 1
fi

BACKUP_FILE="$1"

# Check if backup file exists
if [ ! -f "$BACKUP_FILE" ]; then
    echo "Error: Backup file not found: $BACKUP_FILE"
    exit 1
fi

echo "WARNING: This will restore the database from backup and overwrite existing data!"
echo "Database: $DB_NAME"
echo "Backup file: $BACKUP_FILE"
echo ""
read -p "Are you sure you want to continue? (yes/no): " CONFIRM

if [ "$CONFIRM" != "yes" ]; then
    echo "Restore cancelled"
    exit 0
fi

echo "Starting database restore..."

# Create a backup of current database before restore
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
PRE_RESTORE_BACKUP="./backups/pre_restore_$TIMESTAMP.sql.gz"
echo "Creating pre-restore backup: $PRE_RESTORE_BACKUP"

mysqldump \
    --host="$DB_HOST" \
    --port="$DB_PORT" \
    --user="$DB_USER" \
    --password="$DB_PASSWORD" \
    --single-transaction \
    "$DB_NAME" | gzip > "$PRE_RESTORE_BACKUP"

# Restore from backup
echo "Restoring database from backup..."
gunzip < "$BACKUP_FILE" | mysql \
    --host="$DB_HOST" \
    --port="$DB_PORT" \
    --user="$DB_USER" \
    --password="$DB_PASSWORD" \
    "$DB_NAME"

# Check if restore was successful
if [ $? -eq 0 ]; then
    echo "Database restored successfully!"
    echo "Pre-restore backup saved to: $PRE_RESTORE_BACKUP"
    exit 0
else
    echo "Restore failed!"
    echo "You can restore the pre-restore backup if needed: $PRE_RESTORE_BACKUP"
    exit 1
fi
