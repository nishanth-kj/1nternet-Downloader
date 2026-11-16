# libtorrent's package config calls find_package(Boost). CMake 3.30+ removed the
# FindBoost module (CMP0167) and warns unless the policy is set; NEW makes it use
# the BoostConfig.cmake that vcpkg installs. A policy default only takes effect
# when it is set before cmake_minimum_required().
set(CMAKE_POLICY_DEFAULT_CMP0167 NEW)
cmake_minimum_required(VERSION 3.21)

project(InternetDownloader VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(CURL REQUIRED)
find_package(ZLIB REQUIRED)
find_package(unofficial-sqlite3 CONFIG REQUIRED)
find_package(nlohmann_json CONFIG REQUIRED)
find_package(fmt CONFIG REQUIRED)
find_package(spdlog CONFIG REQUIRED)
find_package(wxWidgets CONFIG REQUIRED)
find_package(SqliteOrm CONFIG REQUIRED)
find_package(LibtorrentRasterbar CONFIG REQUIRED)

file(GLOB_RECURSE CORE_SRCS CONFIGURE_DEPENDS "src/*.cpp")

set(APP_RC "")
if(WIN32)
    set(APP_RC app.rc)
endif()

add_executable(InternetDownloader WIN32
    main.cpp
    ${CORE_SRCS}
    ${APP_RC}
)

target_include_directories(InternetDownloader PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${CMAKE_CURRENT_SOURCE_DIR}/src/service
    ${CMAKE_CURRENT_SOURCE_DIR}/src/gui
    ${CMAKE_CURRENT_SOURCE_DIR}/src/model
    ${CMAKE_CURRENT_SOURCE_DIR}/src/utils
)

target_link_libraries(InternetDownloader PRIVATE
    CURL::libcurl
    ZLIB::ZLIB
    unofficial::sqlite3::sqlite3
    nlohmann_json::nlohmann_json
    fmt::fmt
    spdlog::spdlog
    wx::core
    wx::base
    sqlite_orm::sqlite_orm
    LibtorrentRasterbar::torrent-rasterbar
)

if(WIN32)
    target_link_libraries(InternetDownloader PRIVATE winhttp ws2_32 crypt32 bcrypt shell32 ole32)
endif()

if(MSVC)
    target_compile_options(InternetDownloader PRIVATE /W4 /WX-)
else()
    target_compile_options(InternetDownloader PRIVATE -Wall -Wextra)
endif()

install(TARGETS InternetDownloader RUNTIME DESTINATION bin)
