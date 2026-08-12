// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "PMTiles",
    platforms: [.iOS(.v13)],
    products: [
        .library(name: "PMTiles", targets: ["PMTiles"]),
    ],
    targets: [
        // For a release, switch to .binaryTarget(url:checksum:) pointing at the release asset.
        .binaryTarget(
            name: "PMTilesFFI",
            url: "https://github.com/Mapeak-com/pmtiles-mobile/releases/download/v0.3.3/PMTilesFFI.xcframework.zip",
            checksum: "c57455fce1a7184921fd3abe608a452cfc00dcc17340da8cb4cc4808ebc73412"
        ),
        .target(
            name: "PMTiles",
            dependencies: ["PMTilesFFI"],
            path: "Sources/PMTiles"
        ),
    ]
)
