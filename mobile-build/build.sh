#!/usr/bin/env bash
# Reproducible Android APK build for FRANK Scanlation.
# Run inside the mobile-build Docker image (see Dockerfile).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}/desktop"

echo "==> Installing frontend dependencies"
bun install --frozen-lockfile

echo "==> Building Android debug APK"
bun run mobile:build -- --debug

echo
echo "APK output:"
find "${ROOT}/desktop/src-tauri/gen/android/app/build/outputs/apk" \
    -name '*.apk' -print -exec ls -lh {} \;
