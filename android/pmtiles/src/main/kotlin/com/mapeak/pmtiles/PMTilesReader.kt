package com.mapeak.pmtiles

import java.io.Closeable

/**
 * Reads tiles from a local .pmtiles archive.
 *
 *     PMTilesReader(path).use { reader ->
 *         val tile: ByteArray? = reader.getTile(z = 5, x = 10, y = 12)
 *     }
 *
 * Returned bytes are the decompressed tile payload (e.g. an MVT or PNG),
 * or null if that tile is not in the archive.
 */
class PMTilesReader(path: String) : Closeable {

    private var handle: Long = nativeOpen(path)

    init {
        require(handle != 0L) { "Failed to open PMTiles archive: $path" }
    }

    fun getTile(z: Int, x: Int, y: Int): ByteArray? {
        check(handle != 0L) { "PMTiles reader is closed" }
        return nativeGetTile(handle, z, x, y)
    }

    override fun close() {
        if (handle != 0L) {
            nativeClose(handle)
            handle = 0L
        }
    }

    private external fun nativeOpen(path: String): Long
    private external fun nativeClose(handle: Long)
    private external fun nativeGetTile(handle: Long, z: Int, x: Int, y: Int): ByteArray?

    companion object {
        init { System.loadLibrary("pmtiles_jni") }
    }
}
