// reader.hpp — C++ core. Not exposed across the FFI boundary.
#pragma once

#include <cstdint>
#include <fstream>
#include <string>
#include <vector>

namespace pmt {

class Reader {
public:
  explicit Reader(const std::string &path);
  bool ok() const { return ok_; }

  // Returns the decompressed tile bytes, or an empty vector if the tile
  // is not present in the archive.
  std::vector<uint8_t> get_tile(uint8_t z, uint32_t x, uint32_t y);

private:
  std::vector<uint8_t> read_range(uint64_t offset, uint64_t length);

  std::ifstream f_;
  bool ok_ = false;

  // Cached header fields (populated in the constructor).
  uint64_t root_dir_offset_ = 0, root_dir_bytes_ = 0;
  uint64_t leaf_dirs_offset_ = 0;
  uint64_t tile_data_offset_ = 0;
  uint8_t internal_compression_ = 0; // directory compression
  uint8_t tile_compression_ = 0;
};

} // namespace pmt
