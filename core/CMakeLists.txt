cmake_minimum_required(VERSION 3.20)
set(CMAKE_POLICY_VERSION_MINIMUM "3.5" CACHE STRING "" FORCE)

project(InternetDownloader VERSION 1.0.0 LANGUAGES CXX)

# ── language standard ────────────────────────────────────────────
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# ── vcpkg packages ───────────────────────────────────────────────

find_package(CURL REQUIRED)
find_package(OpenSSL              REQUIRED)
find_package(ZLIB                 REQUIRED)
find_package(unofficial-sqlite3  CONFIG REQUIRED)
find_package(nlohmann_json CONFIG REQUIRED)
find_package(fmt           CONFIG REQUIRED)
find_package(spdlog        CONFIG REQUIRED)
find_package(wxWidgets     CONFIG REQUIRED)
find_package(SqliteOrm     CONFIG REQUIRED)

# ── source files ─────────────────────────────────────────────────
file(GLOB_RECURSE CORE_SRCS "src/*.cpp")

# ── executable ───────────────────────────────────────────────────
add_executable(InternetDownloader WIN32
    main.cpp
    ${CORE_SRCS}
)

target_include_directories(InternetDownloader PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# ── link all dependencies ────────────────────────────────────────
target_link_libraries(InternetDownloader PRIVATE
    CURL::libcurl
    OpenSSL::SSL
    OpenSSL::Crypto
    ZLIB::ZLIB
    unofficial::sqlite3::sqlite3
    nlohmann_json::nlohmann_json
    fmt::fmt
    spdlog::spdlog
    wx::core
    wx::base
    sqlite_orm::sqlite_orm
)

# ── Windows-only ─────────────────────────────────────────────────
if(WIN32)
    # WinHTTP as a fallback / supplement to libcurl
    target_link_libraries(InternetDownloader PRIVATE winhttp ws2_32)
endif()

# ── compiler warnings ────────────────────────────────────────────
if(MSVC)
    target_compile_options(InternetDownloader PRIVATE /W4 /WX-)
else()
    target_compile_options(InternetDownloader PRIVATE -Wall -Wextra)
endif()

# ── install ──────────────────────────────────────────────────────
install(TARGETS InternetDownloader RUNTIME DESTINATION bin)
