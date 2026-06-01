/*
 * pmtiles_c.h — pure C public API for the PMTiles reader core.
 *
 * This is the ONLY header exposed across the FFI boundary. It must stay
 * pure C (no C++ types) so that both JNI (Android) and Swift (iOS) can
 * call it directly. All C++ lives behind this in reader.hpp / *.cpp.
 */
#ifndef PMTILES_C_H
#define PMTILES_C_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque handle to an open archive. */
typedef struct pmtiles_reader pmtiles_reader;

/* Open a local .pmtiles file. Returns NULL on failure. */
pmtiles_reader *pmtiles_open(const char *path);

/* Close and free a reader. Safe to call with NULL. */
void pmtiles_close(pmtiles_reader *reader);

/*
 * Fetch a single tile, decompressed.
 * On success: returns 0, sets *out_data to a malloc'd buffer and *out_len
 *             to its length. Caller MUST free it with pmtiles_free().
 * Tile missing: returns 0 with *out_data == NULL and *out_len == 0.
 * Error:        returns non-zero.
 */
int pmtiles_get_tile(pmtiles_reader *reader,
                     int z, int x, int y,
                     uint8_t **out_data, size_t *out_len);

/* Free a buffer returned by pmtiles_get_tile. Safe to call with NULL. */
void pmtiles_free(uint8_t *data);

#ifdef __cplusplus
}
#endif

#endif /* PMTILES_C_H */
