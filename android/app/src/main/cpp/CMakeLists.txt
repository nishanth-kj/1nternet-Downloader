cmake_minimum_required(VERSION 3.22.1)

project("internetdownloader")

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# libcurl comes from vcpkg's Android community triplet (see app/build.gradle, which
# chainloads the NDK toolchain through vcpkg's so this find_package resolves).
find_package(CURL REQUIRED)

add_library(
        internetdownloader
        SHARED
        native-lib.cpp)

find_library(
        log-lib
        log)

target_link_libraries(
        internetdownloader
        CURL::libcurl
        ${log-lib})
