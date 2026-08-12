#!/bin/bash
# ==============================================================================
# File:           mobile/ios/scripts/build-simulator.sh
# Project:        Qervon
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-12
# Version:        0.2.0
#
# Description:
#   Builds both QervonCourierApp (Faz-2.2) and QervonCustomerApp (Faz-2.3)
#   for the iOS Simulator SDK without requiring an installed Simulator
#   *runtime* (only the SDK, which ships with Xcode itself, is needed to
#   compile -- a runtime is only needed to actually boot and run a
#   simulator).
#
#   Why this script exists instead of a plain `xcodebuild build -scheme
#   QervonCourierApp`: an app's product type is "application", and
#   xcodebuild refuses to resolve a build destination for an application
#   scheme unless a concrete Simulator *device* (backed by an installed
#   runtime) exists. Library/package schemes have no such restriction, so
#   this script builds every local Swift package product first via
#   `-scheme`, then builds each app target via `-target` with
#   SYMROOT/OBJROOT pointed at the SAME DerivedData directory the package
#   builds just used -- without that alignment, `-target` writes to a
#   different, unrelated build folder and the app build fails with
#   spurious "unable to resolve module dependency" errors for its own
#   dependencies.
#
# Usage:
#   ./scripts/build-simulator.sh
#
# License:
#   Qervon License v1.0 — see LICENSE in the repository root.
# ==============================================================================

set -euo pipefail
cd "$(dirname "$0")/.."

DERIVED_DATA="$(pwd)/.build-artifacts/DerivedData"
PROJECT="QervonCourierApp.xcodeproj"
COMMON_FLAGS=(-sdk iphonesimulator CODE_SIGNING_ALLOWED=NO ARCHS=arm64 ONLY_ACTIVE_ARCH=YES)

echo "Project.yml'den .xcodeproj üretiliyor (xcodegen generate)..."
xcodegen generate

# Shared infrastructure (QervonKit) + AuthFeature, used by both apps.
LIBRARY_SCHEMES=(
  QervonCore QervonNetworking QervonSecurity QervonLocation QervonDesignSystem
  AuthFeature
  # Courier-only features
  MapsFeature ProofOfDeliveryFeature OrdersFeature EarningsFeature
  ProfileFeature DispatchFeature
  # Customer-only features
  AddressBookFeature CustomerOrderFeature CustomerProfileFeature
)

for scheme in "${LIBRARY_SCHEMES[@]}"; do
  echo "=== Building $scheme ==="
  xcodebuild build -project "$PROJECT" -scheme "$scheme" "${COMMON_FLAGS[@]}" \
    -derivedDataPath "$DERIVED_DATA"
done

APP_TARGETS=(QervonCourierApp QervonCustomerApp)
for app in "${APP_TARGETS[@]}"; do
  echo "=== Building $app (app target) ==="
  xcodebuild build -project "$PROJECT" -target "$app" "${COMMON_FLAGS[@]}" \
    SYMROOT="$DERIVED_DATA/Build/Products" \
    OBJROOT="$DERIVED_DATA/Build/Intermediates.noindex"

  APP_PATH="$DERIVED_DATA/Build/Products/Debug-iphonesimulator/$app.app"
  if [ -d "$APP_PATH" ]; then
    echo "Derleme başarılı: $APP_PATH"
  else
    echo "Derleme tamamlandı ama .app bulunamadı: $APP_PATH" >&2
    exit 1
  fi
done
