cmake_minimum_required(VERSION 3.18)
project(pmtiles_core LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

add_library(pmtiles_core STATIC
  src/pmtiles_c.cpp
  src/reader.cpp
)

target_include_directories(pmtiles_core
  PUBLIC  ${CMAKE_CURRENT_SOURCE_DIR}/include       # pmtiles_c.h
  PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/third_party    # pmtiles.hpp
          ${CMAKE_CURRENT_SOURCE_DIR}/src
)

# zlib ships with both the Android NDK and the Apple SDKs.
find_package(ZLIB REQUIRED)
target_link_libraries(pmtiles_core PUBLIC ZLIB::ZLIB)
