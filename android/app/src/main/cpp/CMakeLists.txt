cmake_minimum_required(VERSION 3.22.1)

project("internetdownloader")

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(
        internetdownloader
        SHARED
        native-lib.cpp)

find_library(
        log-lib
        log)

target_link_libraries(
        internetdownloader
        ${log-lib})
