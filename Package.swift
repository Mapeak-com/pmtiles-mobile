// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "PMTiles",
    platforms: [.iOS(.v13), .macOS(.v11)],
    products: [
        .library(name: "PMTiles", targets: ["PMTiles"]),
    ],
    targets: [
        // C++ core compiled straight from the shared core/ directory.
        // Its PUBLIC header (pmtiles_c.h) is pure C, so Swift can import it.
        .target(
            name: "CPMTilesCore",
            path: "core",
            exclude: ["CMakeLists.txt"],
            sources: ["src"],
            publicHeadersPath: "include",
            cxxSettings: [
                .headerSearchPath("third_party"),
                .headerSearchPath("src"),
            ],
            linkerSettings: [
                .linkedLibrary("z"),   // system zlib on Apple platforms
            ]
        ),
        // Idiomatic Swift wrapper.
        .target(
            name: "PMTiles",
            dependencies: ["CPMTilesCore"],
            path: "Sources/PMTiles"
        ),
    ],
    cxxLanguageStandard: .cxx17
)

// Note: this builds the core from source on the consumer's machine — the
// simplest setup to maintain. If you hit Swift/C++ interop friction or want
// to ship a closed binary, switch the product to a prebuilt .xcframework via
// a `.binaryTarget` and run scripts/build-xcframework.sh to produce it.
