#!/usr/bin/env bash
# Start only disposable smoke services; never uses the development compose volumes.
set -euo pipefail
: "${LEMMY_SMOKE_BACKEND_IMAGE:?Set the locally built backend image}"
: "${LEMMY_SMOKE_UI_IMAGE:?Set the locally built UI image}"
: "${LEMMY_SMOKE_VERSION:?Set the expected backend/UI version}"
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
smoke_dir=$(mktemp -d "${TMPDIR:-/tmp}/lemmy-ui-smoke.XXXXXX")
project="lemmy-ui-smoke-$(date +%s)-$$"
compose=(docker compose --project-name "$project" --file "$smoke_dir/compose.yml")
cleanup() {
  result=$?
  trap - EXIT
  if [ "$result" -ne 0 ]; then "${compose[@]}" logs --no-color || true; fi
  "${compose[@]}" down --volumes --remove-orphans || true
  # Preserve the small generated configuration for diagnosis; data lives only in tmpfs.
  exit "$result"
}
trap cleanup EXIT
cat > "$smoke_dir/lemmy.hjson" <<'CONFIG'
{
  hostname: "localhost"
  bind: "0.0.0.0"
  port: 8536
  tls_enabled: false
  database: {
    host: "postgres"
    user: "lemmy"
    password: "smoke-only-password"
    database: "lemmy"
  }
  setup: {
    admin_username: "smoke_admin"
    admin_password: "smoke-only-password"
    site_name: "Lemmy UI smoke fixture"
  }
}
CONFIG
chmod 644 "$smoke_dir/lemmy.hjson"
cat > "$smoke_dir/compose.yml" <<'COMPOSE'
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: lemmy
      POSTGRES_PASSWORD: smoke-only-password
      POSTGRES_DB: lemmy
    tmpfs:
      - /var/lib/postgresql/data
    healthcheck:
      test: [CMD-SHELL, 'pg_isready -U lemmy -d lemmy']
      interval: 1s
      timeout: 3s
      retries: 60
  backend:
    image: ${LEMMY_SMOKE_BACKEND_IMAGE}
    command: [--disable-scheduled-tasks, --disable-activity-sending]
    pull_policy: never
    environment:
      LEMMY_CONFIG_LOCATION: /config/config.hjson
    volumes:
      - ./lemmy.hjson:/config/config.hjson:ro
    ports:
      - '127.0.0.1::8536'
    depends_on:
      postgres:
        condition: service_healthy
    healthcheck:
      test: [CMD, curl, --fail, --silent, http://localhost:8536/api/v3/site]
      interval: 2s
      timeout: 5s
      retries: 90
  ui:
    image: ${LEMMY_SMOKE_UI_IMAGE}
    pull_policy: never
    environment:
      LEMMY_UI_LEMMY_INTERNAL_HOST: backend:8536
      LEMMY_UI_LEMMY_EXTERNAL_HOST: localhost:8536
      LEMMY_UI_HTTPS: 'false'
    ports:
      - '127.0.0.1::1234'
    depends_on:
      backend:
        condition: service_healthy
    healthcheck:
      test: [CMD, curl, --fail, --silent, http://localhost:1234/]
      interval: 2s
      timeout: 5s
      retries: 30
COMPOSE
"${compose[@]}" up --detach --wait --wait-timeout 240
backend_url="http://$("${compose[@]}" port backend 8536)"
ui_url="http://$("${compose[@]}" port ui 1234)"
node "$script_dir/smoke.mjs" "$backend_url" "$ui_url" "$LEMMY_SMOKE_VERSION"
