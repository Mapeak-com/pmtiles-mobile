# @mapeak/pmtiles

A small cross-platform library for reading tiles from a local `.pmtiles`
archive on **Android** and **iOS**.

This repo is a thin **[UniFFI](https://mozilla.github.io/uniffi-rs/) wrapper
around the [`pmtiles2`](https://crates.io/crates/pmtiles2) crate** — all the
PMTiles format logic (parsing, directories, gzip/brotli/zstd) lives in
`pmtiles2`. We just expose a small, ergonomic read API to Kotlin and Swift;
there is no hand-written format code, JNI bridge, or Swift wrapper here.

[![Release](https://jitpack.io/v/mapeak-com/pmtiles-mobile.svg)](https://jitpack.io/#mapeak-com/pmtiles-mobile)

- **Android:** AAR served by [JitPack](https://jitpack.io) — no auth required
- **iOS:** `PMTiles` SwiftPM package (consumed from this repo by git tag)

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

PmTilesReader.open(file.path).use { reader ->
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

> **Add it by version (a tag), not the `main` branch.** Tagged releases pin the
> prebuilt `PMTilesFFI.xcframework` via `.binaryTarget(url:checksum:)`; `main`
> keeps a local path target for development and is not resolvable by consumers
> ("does not contain a binary artifact").

The package pulls a prebuilt `PMTilesFFI.xcframework` (the Rust static lib) plus
the generated Swift. If the repo is private, make sure Xcode/SwiftPM has access
(SSH key or a token in `~/.netrc`).

```swift
import PMTiles

let reader = try PmTilesReader.open(path: url.path)
let tile: Data? = try reader.getTile(z: 5, x: 10, y: 12)
```

---

## Building locally

### Rust core

```sh
cd core && cargo build
```

### Android

```sh
# one-time: Android targets + cargo-ndk
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo install cargo-ndk

cd android
./gradlew :pmtiles:assembleRelease      # cross-compiles Rust + generates Kotlin → AAR
./gradlew :pmtiles:publishToMavenLocal   # publish to ~/.m2 for local testing
```

Requires the Android SDK (`ANDROID_HOME`) + NDK (`ndkVersion` in
[android/pmtiles/build.gradle.kts](android/pmtiles/build.gradle.kts)), a Rust
toolchain, and `cargo-ndk` (a Gradle task invokes it to cross-compile the core
into `jniLibs`).

### iOS

```sh
./scripts/build-xcframework.sh   # builds the XCFramework + generates Swift bindings
swift build
```

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
- **iOS** — the release workflow (on a macOS runner) builds
  `PMTilesFFI.xcframework`, attaches it to the release, and pins it into
  `Package.swift` via `.binaryTarget(url:checksum:)` on the tagged commit, so
  SwiftPM consumers resolve the binary from the same tag. (`main` keeps the
  path-based target for local dev.)

The on-push CI in `.github/workflows/ci.yml` validates that the Rust core, the
Android AAR, and the iOS XCFramework + SwiftPM all build.

---

## License

This project is [MIT](LICENSE) licensed. It depends on third-party Rust crates
(`pmtiles2`, `uniffi`, `thiserror`, …), each under permissive licenses
(MIT/Apache-2.0); see their crates for details.
