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
            url: "https://github.com/Mapeak-com/pmtiles-mobile/releases/download/v0.2.1/PMTilesFFI.xcframework.zip",
            checksum: "bf65801d834bfb3bd5bcacf9c3d383c8b019d75625de4f22f2f02d2959e8fd21"
        ),
        .target(
            name: "PMTiles",
            dependencies: ["PMTilesFFI"],
            path: "Sources/PMTiles"
        ),
    ]
)
