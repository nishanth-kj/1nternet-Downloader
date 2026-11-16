cmake_minimum_required(VERSION 3.20)

if(POLICY CMP0141)
    cmake_policy(SET CMP0141 NEW)
endif()

# CMake 3.30 removed FindBoost. libtorrent calls find_dependency(Boost) from
# vcpkg's nested wrapper, which does not inherit this directory's cmake_policy().
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

set(CMAKE_MSVC_DEBUG_INFORMATION_FORMAT "$<$<CONFIG:Debug,RelWithDebInfo>:Embedded>")

project(InternetDownloader VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_OPTIMIZE_DEPENDENCIES ON)

find_package(CURL REQUIRED)
find_package(ZLIB REQUIRED)
find_package(unofficial-sqlite3 CONFIG REQUIRED)
find_package(wxWidgets CONFIG REQUIRED)
find_package(SqliteOrm CONFIG REQUIRED)
find_package(spdlog CONFIG REQUIRED)
find_package(Boost CONFIG REQUIRED)
find_package(LibtorrentRasterbar CONFIG REQUIRED)
find_package(nlohmann_json CONFIG REQUIRED)

set(CORE_SRCS
    main.cpp
    src/gui/main_window.cpp
    src/gui/settings_dialog.cpp
    src/gui/tray_icon.cpp
    src/model/sqlite_download_model.cpp
    src/service/download/download.cpp
    src/service/download/manager.cpp
    src/service/download/progress.cpp
    src/service/network/connection.cpp
    src/service/network/cookies.cpp
    src/service/network/headers.cpp
    src/service/network/http.cpp
    src/service/network/https.cpp
    src/service/network/proxy.cpp
    src/service/network/request.cpp
    src/service/network/response.cpp
    src/service/torrent/torrent_manager.cpp
    src/service/torrent/torrent_metainfo.cpp
    src/utils/checksum.cpp
    src/utils/string_utils.cpp
    src/utils/system/config.cpp
    src/utils/system/ipc.cpp
    src/utils/system/logger.cpp
    src/utils/system/platform.cpp
)

set(APP_RC "")
if(WIN32)
    set(APP_RC app.rc)
endif()

add_executable(InternetDownloader WIN32
    ${CORE_SRCS}
    ${APP_RC}
)

target_include_directories(InternetDownloader PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/src
)

if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/src/idr_pch.h")
    target_precompile_headers(InternetDownloader PRIVATE src/idr_pch.h)
endif()

set_source_files_properties(
    main.cpp
    src/model/sqlite_download_model.cpp
    src/service/torrent/torrent_manager.cpp
    src/service/torrent/torrent_metainfo.cpp
    PROPERTIES SKIP_UNITY_BUILD_INCLUSION ON
)
set_source_files_properties(${APP_RC} PROPERTIES SKIP_PRECOMPILE_HEADERS ON)

target_link_libraries(InternetDownloader PRIVATE
    CURL::libcurl
    ZLIB::ZLIB
    unofficial::sqlite3::sqlite3
    wx::core
    wx::base
    sqlite_orm::sqlite_orm
    spdlog::spdlog
    LibtorrentRasterbar::torrent-rasterbar
    nlohmann_json::nlohmann_json
)

if(WIN32)
    target_link_libraries(InternetDownloader PRIVATE winhttp ws2_32 crypt32 bcrypt shell32 ole32)
endif()

if(MSVC)
    target_compile_options(InternetDownloader PRIVATE
        /W4
        /WX-
        /Zc:inline
        /bigobj
    )
    if(CMAKE_GENERATOR MATCHES "Visual Studio")
        target_compile_options(InternetDownloader PRIVATE /MP)
        set_property(TARGET InternetDownloader PROPERTY VS_GLOBAL_UseMultiToolTask true)
        set_property(TARGET InternetDownloader PROPERTY VS_GLOBAL_EnforceProcessCountAcrossBuilds true)
    endif()
else()
    target_compile_options(InternetDownloader PRIVATE -Wall -Wextra)
endif()

install(TARGETS InternetDownloader RUNTIME DESTINATION bin)
