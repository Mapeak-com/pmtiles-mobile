#!/usr/bin/env bash
# Build a prebuilt XCFramework from the shared core (device + simulator).
# Use this only if you prefer shipping a binary over building from source.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUILD="$ROOT/build/ios"
rm -rf "$BUILD"; mkdir -p "$BUILD"

build_slice () {     # $1 = sysroot, $2 = arch(es), $3 = output dir
  cmake -S "$ROOT/core" -B "$BUILD/$3" -G Xcode \
    -DCMAKE_SYSTEM_NAME=iOS \
    -DCMAKE_OSX_SYSROOT="$1" \
    -DCMAKE_OSX_ARCHITECTURES="$2" \
    -DCMAKE_BUILD_TYPE=Release
  cmake --build "$BUILD/$3" --config Release
}

build_slice iphoneos          arm64        device
build_slice iphonesimulator   "arm64;x86_64" sim

xcodebuild -create-xcframework \
  -library "$BUILD/device/Release-iphoneos/libpmtiles_core.a"      -headers "$ROOT/core/include" \
  -library "$BUILD/sim/Release-iphonesimulator/libpmtiles_core.a"  -headers "$ROOT/core/include" \
  -output "$ROOT/PMTilesCore.xcframework"

echo "Wrote $ROOT/PMTilesCore.xcframework"
