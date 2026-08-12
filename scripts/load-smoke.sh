#!/usr/bin/env bash
set -euo pipefail

base_url=${1:-http://127.0.0.1:8080}
requests=${QERVON_LOAD_REQUESTS:-100}
concurrency=${QERVON_LOAD_CONCURRENCY:-10}

case "$requests:$concurrency" in
  *[!0-9:]*|0:*|*:0) echo 'Load values must be positive integers.' >&2; exit 1 ;;
esac

for _ in $(seq 1 "$requests"); do
  printf '%s\n' "$base_url/ready"
done | xargs -n 1 -P "$concurrency" curl --fail --silent --show-error --output /dev/null
echo "Load smoke passed: $requests readiness requests at concurrency $concurrency."
