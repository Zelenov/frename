#!/usr/bin/env bash
# Install apt packages on a GitHub runner without hanging on a stalled mirror.
# Usage: .github/scripts/apt-install.sh <package>...
#
# The runner's apt mirror sometimes stops sending mid-download and apt-get then waits forever.
# Acquire::*::Timeout makes a silent connection fail after 30 s and Acquire::Retries re-fetches it;
# if the whole update or install still fails, it is retried up to 3 times. The download is ~300 MB
# and a slow (not stalled) mirror can take several minutes, so install has no wall-clock limit of
# its own: the step's timeout-minutes is the hard cap.
set -euo pipefail

opts=(-o Acquire::Retries=3 -o Acquire::http::Timeout=30 -o Acquire::https::Timeout=30
      -o DPkg::Lock::Timeout=120)

retry() {
  local attempt
  for attempt in 1 2 3; do
    if "$@"; then return 0; fi
    if [ "$attempt" -lt 3 ]; then
      echo "::warning::'$*' failed (attempt $attempt of 3), retrying in 15 s"
      sleep 15
      sudo dpkg --configure -a || true
    fi
  done
  echo "::error::'$*' failed 3 times"
  return 1
}

retry timeout --kill-after=10 180 sudo apt-get "${opts[@]}" update
retry sudo apt-get "${opts[@]}" install -y "$@"
