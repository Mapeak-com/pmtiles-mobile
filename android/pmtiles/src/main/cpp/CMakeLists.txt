cmake_minimum_required(VERSION 3.18)
project(pmtiles_jni LANGUAGES CXX)

# Pull in the shared core. Path is relative to this file:
# android/pmtiles/src/main/cpp -> repo root /core
add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/../../../../../core
                 ${CMAKE_BINARY_DIR}/core_build)

add_library(pmtiles_jni SHARED jni_bridge.cpp)

target_link_libraries(pmtiles_jni
  pmtiles_core
  log          # Android logging, optional
)

# Align ELF load segments to 16 KB so the library loads on Android 15+ devices
# that use 16 KB memory pages. NDK r26 defaults to 4 KB (r27+ does this by
# default); the flag is harmless on 4 KB devices.
target_link_options(pmtiles_jni PRIVATE "-Wl,-z,max-page-size=16384")
