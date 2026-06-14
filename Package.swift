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
            url: "https://github.com/Mapeak-com/pmtiles-mobile/releases/download/v0.3.2/PMTilesFFI.xcframework.zip",
            checksum: "af5f5c7653a6d7213fa8137196db535ce75746088e56660b63e4cb5b3b7bce30"
        ),
        .target(
            name: "PMTiles",
            dependencies: ["PMTilesFFI"],
            path: "Sources/PMTiles"
        ),
    ]
)
