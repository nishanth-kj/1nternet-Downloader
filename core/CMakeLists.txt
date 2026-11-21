cmake_minimum_required(VERSION 3.20)

if(POLICY CMP0141)
    cmake_policy(SET CMP0141 NEW)
endif()

set(CMAKE_POLICY_DEFAULT_CMP0167 NEW)
if(POLICY CMP0167)
    cmake_policy(SET CMP0167 NEW)
endif()

if(NOT DEFINED CMAKE_TOOLCHAIN_FILE AND DEFINED ENV{VCPKG_ROOT})
    set(_vcpkg_toolchain "$ENV{VCPKG_ROOT}/scripts/buildsystems/vcpkg.cmake")
    if(EXISTS "${_vcpkg_toolchain}")
        set(CMAKE_TOOLCHAIN_FILE "${_vcpkg_toolchain}" CACHE FILEPATH "vcpkg toolchain")
    endif()
    unset(_vcpkg_toolchain)
endif()

project(idr_torrent_native VERSION 1.0.0 LANGUAGES CXX C)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

find_package(Boost CONFIG REQUIRED)
find_package(LibtorrentRasterbar CONFIG REQUIRED)

set(TORRENT_SRCS
    cpp/torrent_manager.cpp
    cpp/torrent_metainfo.cpp
    cpp/torrent_c_api.cpp
    c/fast_ops.c
)

add_library(idr_torrent_native STATIC ${TORRENT_SRCS})

target_include_directories(idr_torrent_native PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/c
)

target_link_libraries(idr_torrent_native PUBLIC
    LibtorrentRasterbar::torrent-rasterbar
)
