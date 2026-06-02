# @mapeak/pmtiles

A small cross-platform library for reading tiles from a local `.pmtiles`
archive on **Android** and **iOS**, built from a single shared **Rust** core
with [UniFFI](https://mozilla.github.io/uniffi-rs/)-generated bindings.

[![Release](https://jitpack.io/v/mapeak-com/pmtiles-mobile.svg)](https://jitpack.io/#mapeak-com/pmtiles-mobile)

- **Android:** AAR served by [JitPack](https://jitpack.io) — no auth required
- **iOS:** `PMTiles` SwiftPM package (consumed from this repo by git tag)

## How it's structured

```
core/        Rust crate — the reader/writer logic + the UniFFI surface.
  src/pmtiles_reader.rs   PMTiles v3 reader (header, dir walk, gzip).
  src/pmtiles_writer.rs   basic writer (single root directory).
  src/lib.rs              module root + the shared error.
  tests/roundtrip.rs      full-cycle write→read integration tests.
  uniffi.toml             binding config (Kotlin package = com.mapeak.pmtiles).
Sources/     Generated UniFFI Swift bindings (no hand-written wrapper).
android/     Gradle library module — cross-compiles core/ + generates Kotlin.
scripts/     build-xcframework.sh — builds the iOS XCFramework + Swift bindings.
.github/     CI: cargo test, Android AAR, iOS XCFramework + SwiftPM.
jitpack.yml  JitPack build config (installs Rust + NDK, builds the AAR per tag).
```

The layering is: **Rust core → UniFFI → generated Kotlin / Swift**. The Kotlin
`PmTilesReader` and Swift `PmTilesReader` classes are *generated* from the Rust
signatures in [core/src/lib.rs](core/src/lib.rs) — there is no hand-written JNI
bridge or Swift wrapper. `Vec<u8>` maps natively to `ByteArray?` (Kotlin) and
`Data?` (Swift).

## The Rust core

The reader lives in [core/src/pmtiles_reader.rs](core/src/pmtiles_reader.rs) (a
small, dependency-light PMTiles v3 reader using `flate2` for gzip) and a basic
writer in [core/src/pmtiles_writer.rs](core/src/pmtiles_writer.rs). Both are
exposed to Kotlin/Swift via UniFFI. Tests are full-cycle: write an archive with
the writer, read it back with the reader ([core/tests/roundtrip.rs](core/tests/roundtrip.rs)).

```sh
cd core && cargo test
```

> `PmTilesWriter` emits a single root directory (no leaf directories), so it's
> intended for small archives, not large general-purpose tilesets.

## Releasing

Versions are driven by git tags (`vX.Y.Z`) — there's no version file to edit.

To cut a release, run the **Release (bump version)** workflow:
**Actions → Release (bump version) → Run workflow**, then pick `patch`, `minor`,
or `major` from the dropdown. It computes the next version from the latest tag,
pushes the new tag, creates a GitHub Release, and pings JitPack to build the AAR.

Each new tag then flows to consumers automatically:

- **Android** — JitPack builds the AAR from the tag on first request (it
  installs Rust + the NDK and cross-compiles the core) and serves it at
  `com.github.mapeak-com:pmtiles-mobile:<tag>`.
- **iOS** — because SwiftPM can't compile Rust, each release must attach the
  prebuilt `PMTilesFFI.xcframework.zip` (from `scripts/build-xcframework.sh`),
  and `Package.swift` references it via `.binaryTarget(url:checksum:)` for that
  tag. (The release workflow needs a step to build, upload, and pin it.)

The on-push CI in `.github/workflows/ci.yml` validates that the Rust core
(`cargo test`), the Android AAR, and the iOS XCFramework + SwiftPM all build.

---

## Consuming the package

### Android (in your app repo) — no authentication required

```kotlin
// settings.gradle.kts (dependencyResolutionManagement) or build.gradle.kts
repositories {
    google()
    mavenCentral()
    maven { url = uri("https://jitpack.io") }
}

dependencies {
    // Use the latest tag shown in the badge at the top of this README.
    implementation("com.github.mapeak-com:pmtiles-mobile:<version>")
}
```

```kotlin
import com.mapeak.pmtiles.PmTilesReader

PmTilesReader(file.path).use { reader ->
    val tile: ByteArray? = reader.getTile(z = 5, x = 10, y = 12)
}
```

> The Kotlin package / API is `com.mapeak.pmtiles`; the JitPack *coordinate*
> (`com.github.mapeak-com:pmtiles-mobile`) is just how JitPack namespaces the repo
> (group = `com.github.<owner>`, artifact = repo name).

### iOS (in your app repo)

Add the package in Xcode (File → Add Packages → this repo URL), or in
`Package.swift`:

```swift
// Use the latest tag shown in the badge at the top of this README. `from:` is
// a minimum, so SwiftPM also picks up any newer release in the same major.
.package(url: "https://github.com/mapeak-com/pmtiles-mobile.git", from: "<version>")
```

The package pulls a prebuilt `PMTilesFFI.xcframework` (the Rust static lib) plus
the generated Swift. If the repo is private, make sure Xcode/SwiftPM has access
(SSH key or a token in `~/.netrc`).

```swift
import PMTiles

let reader = try PmTilesReader(path: url.path)
let tile: Data? = try reader.getTile(z: 5, x: 10, y: 12)
```

---

## Building & testing locally

### Rust core

```sh
cd core && cargo test
```

### Android

```sh
# one-time: rust + the Android targets
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

cd android
./gradlew :pmtiles:assembleRelease      # cross-compiles Rust + generates Kotlin → AAR
./gradlew :pmtiles:publishToMavenLocal   # publish to ~/.m2 for local testing
```

Requires the Android SDK (`ANDROID_HOME`) + NDK (`ndkVersion` in
[android/pmtiles/build.gradle.kts](android/pmtiles/build.gradle.kts)) and a Rust
toolchain. The `rust-android-gradle` plugin drives the cross-compilation.

### iOS

```sh
./scripts/build-xcframework.sh   # builds the XCFramework + generates Swift bindings
swift build
```

## License

This project is [MIT](LICENSE) licensed. It depends on third-party Rust crates
(`uniffi`, `flate2`, `thiserror`, …), each under permissive licenses
(MIT/Apache-2.0); see their crates for details.
