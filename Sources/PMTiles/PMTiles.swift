import Foundation
import CPMTilesCore

/// Reads tiles from a local `.pmtiles` archive.
///
///     let reader = try PMTilesReader(path: url.path)
///     if let tile = reader.tile(z: 5, x: 10, y: 12) {
///         // `tile` is the decompressed payload (e.g. MVT or PNG)
///     }
public final class PMTilesReader {

    private let handle: OpaquePointer

    public enum Error: Swift.Error { case open(String) }

    public init(path: String) throws {
        guard let h = pmtiles_open(path) else { throw Error.open(path) }
        self.handle = h
    }

    deinit { pmtiles_close(handle) }

    /// Returns the decompressed tile bytes, or `nil` if absent.
    public func tile(z: Int, x: Int, y: Int) -> Data? {
        var ptr: UnsafeMutablePointer<UInt8>? = nil
        var len: Int = 0
        let rc = pmtiles_get_tile(handle, Int32(z), Int32(x), Int32(y), &ptr, &len)
        guard rc == 0, let p = ptr, len > 0 else { return nil }
        defer { pmtiles_free(p) }
        return Data(bytes: p, count: len)
    }
}
