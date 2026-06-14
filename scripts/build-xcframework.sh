#!/usr/bin/env bash
# Build artifacts/PMTilesFFI.xcframework and Sources/PMTiles/pmtiles_core.swift
# from the Rust core. Requires rustup, cargo, Xcode.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CORE="$ROOT/core"
BUILD="$ROOT/build/apple"
ARTIFACTS="$ROOT/artifacts"
LIB="libpmtiles_core.a"
FRAMEWORK="PMTilesFFI"
FFI_MODULE="pmtiles_coreFFI"

TARGETS=(
  aarch64-apple-ios
  aarch64-apple-ios-sim
  x86_64-apple-ios
  aarch64-apple-darwin
  x86_64-apple-darwin
)

rustup target add "${TARGETS[@]}"

for t in "${TARGETS[@]}"; do
  ( cd "$CORE" && cargo build --release --lib --target "$t" )
done

HEADERS="$BUILD/headers"
rm -rf "$HEADERS"; mkdir -p "$HEADERS/$FFI_MODULE"
( cd "$CORE" && cargo run --quiet --bin uniffi-bindgen -- generate \
    --library "target/aarch64-apple-ios/release/$LIB" \
    --language swift --out-dir "$BUILD/swift" --no-format )
cp "$BUILD/swift/pmtiles_core.swift" "$ROOT/Sources/PMTiles/pmtiles_core.swift"
cp "$BUILD/swift/$FFI_MODULE.h" "$HEADERS/$FFI_MODULE/"
cp "$BUILD/swift/$FFI_MODULE.modulemap" "$HEADERS/$FFI_MODULE/module.modulemap"

mkdir -p "$BUILD/ios-sim" "$BUILD/macos"
lipo -create \
  "$CORE/target/aarch64-apple-ios-sim/release/$LIB" \
  "$CORE/target/x86_64-apple-ios/release/$LIB" \
  -output "$BUILD/ios-sim/$LIB"
lipo -create \
  "$CORE/target/aarch64-apple-darwin/release/$LIB" \
  "$CORE/target/x86_64-apple-darwin/release/$LIB" \
  -output "$BUILD/macos/$LIB"

rm -rf "$ARTIFACTS/$FRAMEWORK.xcframework"; mkdir -p "$ARTIFACTS"
xcodebuild -create-xcframework \
  -library "$CORE/target/aarch64-apple-ios/release/$LIB" -headers "$HEADERS" \
  -library "$BUILD/ios-sim/$LIB"                          -headers "$HEADERS" \
  -library "$BUILD/macos/$LIB"                            -headers "$HEADERS" \
  -output "$ARTIFACTS/$FRAMEWORK.xcframework"

echo "Wrote $ARTIFACTS/$FRAMEWORK.xcframework"
