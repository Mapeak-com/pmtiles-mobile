// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "PMTiles",
    platforms: [.iOS(.v13), .macOS(.v11)],
    products: [
        .library(name: "PMTiles", targets: ["PMTiles"]),
    ],
    targets: [
        // For a release, switch to .binaryTarget(url:checksum:) pointing at the release asset.
        .binaryTarget(
            name: "PMTilesFFI",
            url: "https://github.com/Mapeak-com/pmtiles-mobile/releases/download/v0.2.2/PMTilesFFI.xcframework.zip",
            checksum: "cb77924df06077c050e42b70e018e28c233f7e27462c496a2ca1250cf63ef1f4"
        ),
        .target(
            name: "PMTiles",
            dependencies: ["PMTilesFFI"],
            path: "Sources/PMTiles"
        ),
    ]
)
