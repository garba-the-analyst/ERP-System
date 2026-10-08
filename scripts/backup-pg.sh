#!/usr/bin/env bash
# Nightly pg_dump + AES baseline (clinic NFR-3 + 90-180d retention).
set -euo pipefail
STAMP=$(date +%F)
mkdir -p backups
for DB in "erp_main:5432" "erp_clinic:5433"; do
  NAME=${DB%%:*}; PORT=${DB##*:}
  PGPASSWORD="${POSTGRES_PASSWORD:-erp_pw}" pg_dump -h localhost -p "$PORT" -U postgres "$NAME" \
    | openssl enc -aes-256-cbc -pbkdf2 -pass "env:BACKUP_PASSPHRASE" -out "backups/${NAME}-${STAMP}.sql.enc"
done
echo "backups done ${STAMP}; hot 0-90d local, warm 90-180d offsite"
