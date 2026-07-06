#!/usr/bin/env bash
# macOS runner：cargo build 后用 Apple Development 证书签名，稳定 TCC 屏幕录制授权。
# Tauri dev  传入: run --no-default-features --color always --
# Tauri build 传入: build --release ...
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/src-tauri"

MODE="build"
if [[ "${1:-}" == "run" ]]; then
  MODE="run"
  shift
elif [[ "${1:-}" == "build" ]]; then
  shift
fi

cargo build "$@" || exit $?

PROFILE="debug"
for arg in "$@"; do
  if [[ "$arg" == "--release" ]]; then
    PROFILE="release"
    break
  fi
done

BIN="$ROOT/src-tauri/target/$PROFILE/rinova-void"
IDENTITY="${APPLE_SIGNING_IDENTITY:-}"

if [[ -z "$IDENTITY" ]]; then
  IDENTITY="$(security find-identity -v -p codesigning 2>/dev/null \
    | grep 'Apple Development' \
    | head -1 \
    | sed -E 's/^[[:space:]]*[0-9]+[[:space:]]+[0-9A-F]+[[:space:]]+"([^"]+)".*$/\1/' \
    || true)"
fi

if [[ -n "$IDENTITY" ]]; then
  codesign --force --sign "$IDENTITY" --options runtime "$BIN"
  echo "[macos-dev-runner] 已签名: $IDENTITY ($PROFILE)"
else
  codesign --force --sign - --options runtime "$BIN" 2>/dev/null || true
  echo "[macos-dev-runner] 警告: 未找到 Apple Development 证书（请在 Xcode 登录 Apple ID）。"
  echo "[macos-dev-runner] 已 ad-hoc 签名；macOS Sequoia 上屏幕录制权限可能仍不可用。"
fi

if [[ "$MODE" == "run" ]]; then
  exec "$BIN"
fi
