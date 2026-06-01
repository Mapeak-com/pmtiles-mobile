// Minimal desktop smoke test. Build with the root CMakeLists (see README),
// point it at a real .pmtiles file, and check a known tile comes back.
#include "pmtiles_c.h"
#include <cstdio>
#include <cstdlib>  // atoi

int main(int argc, char **argv) {
  if (argc < 5) { std::printf("usage: %s file.pmtiles z x y\n", argv[0]); return 2; }
  pmtiles_reader *r = pmtiles_open(argv[1]);
  if (!r) { std::printf("open failed\n"); return 1; }
  uint8_t *data = nullptr; size_t len = 0;
  int rc = pmtiles_get_tile(r, atoi(argv[2]), atoi(argv[3]), atoi(argv[4]), &data, &len);
  std::printf("rc=%d len=%zu\n", rc, len);
  pmtiles_free(data);
  pmtiles_close(r);
  return rc;
}
