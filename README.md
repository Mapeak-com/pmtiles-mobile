# @mapeak/pmtiles

A small cross-platform library for reading tiles from a local `.pmtiles`
archive on **Android** and **iOS**, built from a single shared C++ core.

- **Android:** AAR served by [JitPack](https://jitpack.io) — no auth required
- **iOS:** `PMTiles` SwiftPM package (consumed from this repo by git tag)

## How it's structured

```
core/        Shared C++ core + the extern "C" public API (pmtiles_c.h).
             This is the only place the reading logic lives.
             core/third_party/pmtiles.hpp is vendored from protomaps/PMTiles.
Sources/     Swift wrapper (iOS) — compiled together with core/ by SwiftPM.
android/     Gradle library module — compiles core/ via the NDK into an AAR.
scripts/     XCFramework build script (binary-distribution fallback for iOS).
tests/       Desktop smoke test.
.github/     CI: build/test the core, Android AAR, and SwiftPM on every push.
jitpack.yml  JitPack build config (installs NDK/CMake, builds the AAR per tag).
```

The layering is deliberate: **C++ core → C ABI (`pmtiles_c.h`) → thin native
binding per platform**. Only the pure-C header crosses the language boundary,
which is what lets both JNI and Swift call the same code.

## Vendored upstream header

The reader is built on protomaps' header-only C++ implementation, vendored at
[core/third_party/pmtiles.hpp](core/third_party/pmtiles.hpp) (BSD-3-Clause; the
commit it was taken from is recorded in a comment at the top of that file).

To update it, re-download and bump the recorded commit:

```sh
curl -L -o core/third_party/pmtiles.hpp \
  https://raw.githubusercontent.com/protomaps/PMTiles/main/cpp/pmtiles.hpp
```

`core/src/reader.cpp` is written against these symbols from that header
(`deserialize_header`, `deserialize_directory`, `find_tile`, `zxy_to_tileid`,
`entryv3`, `COMPRESSION_*`) — sanity-check them after an update.

## Releasing

Versions are driven by git tags. Create a **GitHub Release** (or just push a tag)
like `v0.1.0`, and:

- **Android** — JitPack builds the AAR from that tag on first request and serves
  it at `com.github.Mapeak-com:pmtiles-mobile:v0.1.0`. (Optionally warm it
  up by opening `https://jitpack.io/#Mapeak-com/pmtiles-mobile` and clicking
  *Get it* on the tag.)
- **iOS** — the same tag is the SwiftPM version; nothing to publish.

No repository secrets are needed. The on-push CI in `.github/workflows/ci.yml`
just validates that everything still builds.

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
    implementation("com.github.Mapeak-com:pmtiles-mobile:v0.1.0")
}
```

```kotlin
import com.mapeak.pmtiles.PMTiles

PMTiles(file.path).use { reader ->
    val tile: ByteArray? = reader.getTile(z = 5, x = 10, y = 12)
}
```

> The Kotlin package / API is `com.mapeak.pmtiles`; the JitPack *coordinate*
> (`com.github.Mapeak-com:pmtiles-mobile`) is just how JitPack namespaces the repo
> (group = `com.github.<owner>`, artifact = repo name).

### iOS (in your app repo)

Add the package in Xcode (File → Add Packages → this repo URL), or in
`Package.swift`:

```swift
.package(url: "https://github.com/Mapeak-com/pmtiles-mobile.git", from: "0.1.0")
```

SwiftPM compiles `core/` from source. If the repo is private, make sure Xcode/
SwiftPM has access (SSH key or a token in `~/.netrc`).

```swift
import PMTiles

let reader = try PMTilesReader(path: url.path)
let tile: Data? = reader.tile(z: 5, x: 10, y: 12)
```

If you'd rather ship a binary (or hit C++/Swift interop friction), run
`scripts/build-xcframework.sh` and switch the package to a `.binaryTarget`.

---

## Building & testing locally

### Android

```sh
cd android
./gradlew :pmtiles:assembleRelease      # produces the AAR
./gradlew :pmtiles:publishToMavenLocal   # publish to ~/.m2 for local testing
```

Requires the Android SDK (`ANDROID_HOME`), plus NDK and CMake (the build
installs/uses `cmake;3.22.1`).

### iOS

```sh
swift build
```

### Desktop smoke test

```sh
cmake -S core -B build && cmake --build build
c++ -std=c++17 -Icore/include tests/test_reader.cpp build/libpmtiles_core.a -lz -o test_reader
./test_reader some.pmtiles 5 10 12
```

## License

This project is [MIT](LICENSE) licensed. The vendored
`core/third_party/pmtiles.hpp` is BSD-3-Clause (Copyright 2021 Protomaps LLC) —
its terms are reproduced in [THIRD_PARTY_LICENSES](THIRD_PARTY_LICENSES).
