// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "PMTiles",
    platforms: [.iOS(.v15)],
    products: [
        .library(name: "PMTiles", targets: ["PMTiles"]),
    ],
    targets: [
        // For a release, switch to .binaryTarget(url:checksum:) pointing at the release asset.
        .binaryTarget(
            name: "PMTilesFFI",
            url: "https://github.com/Mapeak-com/pmtiles-mobile/releases/download/v0.3.5/PMTilesFFI.xcframework.zip",
            checksum: "24a32847d375f87917361d48b017119a7d6b193d52552e496295e23d9b527131"
        ),
        .target(
            name: "PMTiles",
            dependencies: ["PMTilesFFI"],
            path: "Sources/PMTiles"
        ),
    ]
)
