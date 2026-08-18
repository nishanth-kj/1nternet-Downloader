cmake_minimum_required(VERSION 3.15)
set(CMAKE_POLICY_VERSION_MINIMUM "3.5" CACHE STRING "" FORCE)
project(InternetDownloader VERSION 1.0.0 LANGUAGES C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Include directories
include_directories(
    ${CMAKE_SOURCE_DIR}/core/include
    ${CMAKE_SOURCE_DIR}/gui/include
)

# Glob all source files
file(GLOB_RECURSE CORE_SRCS "core/src/*.c")
file(GLOB_RECURSE GUI_SRCS "gui/src/*.c")
set(APP_SRCS "app/main.c")

# Create executable
add_executable(InternetDownloader ${APP_SRCS} ${CORE_SRCS} ${GUI_SRCS})

# Fetch Raylib
include(FetchContent)
FetchContent_Declare(
    raylib
    GIT_REPOSITORY https://github.com/raysan5/raylib.git
    GIT_TAG 5.0
)
FetchContent_MakeAvailable(raylib)

# Link
target_link_libraries(InternetDownloader PRIVATE raylib)

# Hide console window in Release builds on Windows
if(WIN32)
    target_link_options(InternetDownloader PRIVATE
        $<$<CONFIG:Release>:/SUBSYSTEM:WINDOWS>
        $<$<CONFIG:Release>:/ENTRY:mainCRTStartup>
    )
endif()
