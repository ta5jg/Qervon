#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
android_root="$repo_root/mobile/android"
release_root="$HOME/Library/Application Support/Qervon/Android"

export QERVON_ANDROID_KEYSTORE_FILE="$release_root/qervon-upload.jks"
export QERVON_ANDROID_KEY_ALIAS="qervon-upload"
export QERVON_ANDROID_STORE_PASSWORD
export QERVON_ANDROID_KEY_PASSWORD

QERVON_ANDROID_STORE_PASSWORD=$(security find-generic-password \
    -a qervon-android-upload \
    -s qervon.android.upload.store-password \
    -w)
QERVON_ANDROID_KEY_PASSWORD=$(security find-generic-password \
    -a qervon-android-upload \
    -s qervon.android.upload.key-password \
    -w)

: "${JAVA_HOME:?JAVA_HOME must point to JDK 17}"

cd "$android_root"
exec ./gradlew :app-courier:bundleRelease :app-customer:bundleRelease
