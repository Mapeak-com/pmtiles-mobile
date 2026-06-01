// reader.cpp — directory walk + decompression over pmtiles.hpp.
//
// IMPORTANT: the exact symbol/field names below (deserialize_header,
// headerv3, entryv3, find_tile, zxy_to_tileid, COMPRESSION_*) come from
// protomaps' pmtiles.hpp. Verify them against the version of the header
// you actually vendor into core/third_party/ — minor naming differences
// across versions are the most likely thing to need a tweak here.

#include "reader.hpp"
#include "pmtiles.hpp"

#include <zlib.h>
#include <cstring>

namespace {

// Decompress a gzip/zlib stream (windowBits 15+32 auto-detects gzip vs zlib).
std::vector<uint8_t> gunzip(const std::vector<uint8_t> &in) {
  z_stream zs{};
  if (inflateInit2(&zs, 15 + 32) != Z_OK) return {};

  zs.next_in = const_cast<Bytef *>(in.data());
  zs.avail_in = static_cast<uInt>(in.size());

  std::vector<uint8_t> out;
  uint8_t buf[16384];
  int ret;
  do {
    zs.next_out = buf;
    zs.avail_out = sizeof(buf);
    ret = inflate(&zs, Z_NO_FLUSH);
    if (ret != Z_OK && ret != Z_STREAM_END) { inflateEnd(&zs); return {}; }
    out.insert(out.end(), buf, buf + (sizeof(buf) - zs.avail_out));
  } while (ret != Z_STREAM_END);

  inflateEnd(&zs);
  return out;
}

std::vector<uint8_t> maybe_decompress(std::vector<uint8_t> bytes, uint8_t compression) {
  // pmtiles::COMPRESSION_NONE == 1, COMPRESSION_GZIP == 2 in v3.
  if (compression == pmtiles::COMPRESSION_GZIP) return gunzip(bytes);
  return bytes; // NONE (or already-handled). Add zstd here if you need it.
}

} // namespace

namespace pmt {

Reader::Reader(const std::string &path) {
  f_.open(path, std::ios::binary);
  if (!f_) return;

  auto header_bytes = read_range(0, 127);
  if (header_bytes.size() != 127) return;

  auto h = pmtiles::deserialize_header(
      std::string(header_bytes.begin(), header_bytes.end()));

  root_dir_offset_      = h.root_dir_offset;
  root_dir_bytes_       = h.root_dir_bytes;
  leaf_dirs_offset_     = h.leaf_dirs_offset;
  tile_data_offset_     = h.tile_data_offset;
  internal_compression_ = h.internal_compression;
  tile_compression_     = h.tile_compression;
  ok_ = true;
}

std::vector<uint8_t> Reader::read_range(uint64_t offset, uint64_t length) {
  std::vector<uint8_t> buf(length);
  f_.clear();
  f_.seekg(static_cast<std::streamoff>(offset));
  f_.read(reinterpret_cast<char *>(buf.data()), static_cast<std::streamsize>(length));
  buf.resize(static_cast<size_t>(f_.gcount()));
  return buf;
}

std::vector<uint8_t> Reader::get_tile(uint8_t z, uint32_t x, uint32_t y) {
  if (!ok_) return {};

  uint64_t tile_id = pmtiles::zxy_to_tileid(z, x, y);

  uint64_t dir_offset = root_dir_offset_;
  uint64_t dir_length = root_dir_bytes_;

  // Up to 3 levels: root dir -> leaf dir -> (rarely) deeper. 4 is safe.
  for (int depth = 0; depth < 4; ++depth) {
    auto raw = read_range(dir_offset, dir_length);
    auto dir_bytes = maybe_decompress(std::move(raw), internal_compression_);

    auto entries = pmtiles::deserialize_directory(
        std::string(dir_bytes.begin(), dir_bytes.end()));

    auto entry = pmtiles::find_tile(entries, tile_id);
    if (entry.length == 0) return {}; // not found

    if (entry.run_length == 0) {
      // Pointer to a leaf directory.
      dir_offset = leaf_dirs_offset_ + entry.offset;
      dir_length = entry.length;
    } else {
      // Actual tile.
      auto tile = read_range(tile_data_offset_ + entry.offset, entry.length);
      return maybe_decompress(std::move(tile), tile_compression_);
    }
  }
  return {};
}

} // namespace pmt
