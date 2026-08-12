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
)

rustup target add "${TARGETS[@]}"
# llvm-objcopy, for the bitcode strip below.
rustup component add llvm-tools >/dev/null
OBJCOPY="$(dirname "$(rustc --print target-libdir)")/bin/llvm-objcopy"

for t in "${TARGETS[@]}"; do
  ( cd "$CORE" && cargo build --release --lib --target "$t" )
  # Rust's precompiled std/core/alloc rlibs still carry embedded LLVM bitcode,
  # ~40% of the archive. Apple removed bitcode in Xcode 14, and its own
  # bitcode_strip is a no-op on these objects, so drop the sections directly.
  # Nothing local to fix: -C embed-bitcode=no is already cargo's release default.
  "$OBJCOPY" --remove-section=__LLVM,__bitcode --remove-section=__LLVM,__cmdline \
    "$CORE/target/$t/release/$LIB"
done

HEADERS="$BUILD/headers"
rm -rf "$HEADERS"; mkdir -p "$HEADERS/$FFI_MODULE"
( cd "$CORE" && cargo run --quiet --features uniffi-bindgen --bin uniffi-bindgen -- generate \
    --library "target/aarch64-apple-ios/release/$LIB" \
    --language swift --out-dir "$BUILD/swift" --no-format )
cp "$BUILD/swift/pmtiles_core.swift" "$ROOT/Sources/PMTiles/pmtiles_core.swift"
cp "$BUILD/swift/$FFI_MODULE.h" "$HEADERS/$FFI_MODULE/"
cp "$BUILD/swift/$FFI_MODULE.modulemap" "$HEADERS/$FFI_MODULE/module.modulemap"

mkdir -p "$BUILD/ios-sim"
lipo -create \
  "$CORE/target/aarch64-apple-ios-sim/release/$LIB" \
  "$CORE/target/x86_64-apple-ios/release/$LIB" \
  -output "$BUILD/ios-sim/$LIB"

rm -rf "$ARTIFACTS/$FRAMEWORK.xcframework"; mkdir -p "$ARTIFACTS"
xcodebuild -create-xcframework \
  -library "$CORE/target/aarch64-apple-ios/release/$LIB" -headers "$HEADERS" \
  -library "$BUILD/ios-sim/$LIB"                          -headers "$HEADERS" \
  -output "$ARTIFACTS/$FRAMEWORK.xcframework"

echo "Wrote $ARTIFACTS/$FRAMEWORK.xcframework"
