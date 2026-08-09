#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCHEMA_FILE="${SCRIPT_DIR}/supabase_schema.sql"

if [ -z "$1" ]; then
  echo "Usage: ./push_schema.sh <SUPABASE_DB_PASSWORD_OR_ACCESS_TOKEN>"
  exit 1
fi

if [[ "$1" == sbp_* ]]; then
  export SUPABASE_ACCESS_TOKEN="$1"
  echo "Linking project jwswpryozfxzaocimadp..."
  npx supabase link --project-ref jwswpryozfxzaocimadp
  echo "Pushing schema..."
  npx supabase db query -f "${SCHEMA_FILE}" --linked
else
  echo "Executing schema against database URL..."
  ENCODED_PASS=$(python3 -c "import urllib.parse; print(urllib.parse.quote('$1'))")
  npx supabase db query -f "${SCHEMA_FILE}" --db-url "postgresql://postgres:${ENCODED_PASS}@db.jwswpryozfxzaocimadp.supabase.co:5432/postgres"
fi

echo "Schema push completed successfully!"
