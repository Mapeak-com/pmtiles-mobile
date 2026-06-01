// pmtiles_c.cpp — implements the extern "C" API in terms of pmt::Reader.
#include "pmtiles_c.h"
#include "reader.hpp"

#include <cstdlib>
#include <cstring>
#include <new>

struct pmtiles_reader {
  pmt::Reader impl;
  explicit pmtiles_reader(const char *path) : impl(path) {}
};

extern "C" {

pmtiles_reader *pmtiles_open(const char *path) {
  if (!path) return nullptr;
  auto *r = new (std::nothrow) pmtiles_reader(path);
  if (!r || !r->impl.ok()) { delete r; return nullptr; }
  return r;
}

void pmtiles_close(pmtiles_reader *reader) { delete reader; }

int pmtiles_get_tile(pmtiles_reader *reader, int z, int x, int y,
                     uint8_t **out_data, size_t *out_len) {
  if (!reader || !out_data || !out_len) return 1;
  *out_data = nullptr;
  *out_len = 0;

  auto bytes = reader->impl.get_tile(static_cast<uint8_t>(z),
                                     static_cast<uint32_t>(x),
                                     static_cast<uint32_t>(y));
  if (bytes.empty()) return 0; // success, tile simply absent

  auto *buf = static_cast<uint8_t *>(std::malloc(bytes.size()));
  if (!buf) return 1;
  std::memcpy(buf, bytes.data(), bytes.size());
  *out_data = buf;
  *out_len = bytes.size();
  return 0;
}

void pmtiles_free(uint8_t *data) { std::free(data); }

} // extern "C"
