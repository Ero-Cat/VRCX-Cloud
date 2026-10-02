#!/usr/bin/env bash
# Server-side half of the automated deploy: check GHCR for a new image and
# recreate the container when it changes. Run by the vrcx-cloud-update
# systemd timer (deploy/) or by hand from the clone root.
#
# Requires: .env next to docker-compose.yml, and `docker login ghcr.io`
# when the package is private (a public package pulls anonymously).
set -euo pipefail

cd "$(dirname "$0")/.."

# Manifest check only — downloads nothing when the digest is unchanged.
docker compose pull --quiet app

container_id="$(docker compose ps -q app)"
running_image=""
if [[ -n "$container_id" ]]; then
    running_image="$(docker inspect --format '{{.Image}}' "$container_id")"
fi
latest_image="$(docker image inspect ghcr.io/ero-cat/vrcx-cloud:latest --format '{{.Id}}')"

if [[ "$running_image" == "$latest_image" ]]; then
    exit 0
fi

docker compose up -d --remove-orphans
docker image prune --force >/dev/null
echo "deployed $latest_image"
