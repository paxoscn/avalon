#!/bin/bash
# Database backup script
# Requirement 1.2: Implement database backup

set -e

# Configuration
BACKUP_DIR="${BACKUP_DIR:-./backups}"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-3306}"
DB_NAME="${DB_NAME:-agent_platform}"
DB_USER="${DB_USER:-agent_user}"
DB_PASSWORD="${DB_PASSWORD:-agent_password}"
RETENTION_DAYS="${RETENTION_DAYS:-7}"

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Generate backup filename with timestamp
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BACKUP_FILE="$BACKUP_DIR/agent_platform_$TIMESTAMP.sql"
COMPRESSED_FILE="$BACKUP_FILE.gz"

echo "Starting database backup..."
echo "Database: $DB_NAME"
echo "Backup file: $COMPRESSED_FILE"

# Perform backup
mysqldump \
    --host="$DB_HOST" \
    --port="$DB_PORT" \
    --user="$DB_USER" \
    --password="$DB_PASSWORD" \
    --single-transaction \
    --routines \
    --triggers \
    --events \
    "$DB_NAME" | gzip > "$COMPRESSED_FILE"

# Check if backup was successful
if [ $? -eq 0 ]; then
    echo "Backup completed successfully: $COMPRESSED_FILE"
    
    # Get backup file size
    BACKUP_SIZE=$(du -h "$COMPRESSED_FILE" | cut -f1)
    echo "Backup size: $BACKUP_SIZE"
    
    # Remove old backups
    echo "Cleaning up old backups (older than $RETENTION_DAYS days)..."
    find "$BACKUP_DIR" -name "agent_platform_*.sql.gz" -type f -mtime +$RETENTION_DAYS -delete
    
    echo "Backup process completed successfully"
    exit 0
else
    echo "Backup failed!"
    exit 1
fi
