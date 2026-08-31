# Internet Downloader

A native C++17 Internet Downloader for Windows using CMake, vcpkg,
libcurl, OpenSSL, SQLite, wxWidgets, spdlog, fmt, and nlohmann-json.

## 1. Project Structure

The project is organized as follows:

``` text
C:\
└── vcpkg\
    ├── vcpkg.exe
    ├── ports\
    ├── scripts\
    └── ...

C:\Projects\1nternet-Downloader\
├── InternetDownloader.exe        # Optional final/copied application
└── core\
    ├── CMakeLists.txt
    ├── CMakePresets.json
    ├── vcpkg.json
    ├── vcpkg_installed\          # Project dependencies
    ├── build\                    # CMake generated build files
    ├── src\
    └── include\
```

### Important

There is one global vcpkg installation:

``` text
C:\vcpkg
```

The project does not contain another copy of vcpkg.

The project-specific packages are installed into:

``` text
core\vcpkg_installed
```

This means the vcpkg tool is shared, while the installed libraries
remain isolated for this project.

------------------------------------------------------------------------

# 2. Requirements

Install the following:

-   Windows 10/11
-   Visual Studio 2026
-   Visual Studio C++ Desktop Development workload
-   CMake 3.20 or newer
-   Git
-   vcpkg

The project is configured for:

``` text
Architecture: x64
Compiler: MSVC
Language: C++17
Generator: Visual Studio
```

------------------------------------------------------------------------

# 3. Visual Studio C++ Workload

Open Visual Studio Installer.

Install:

``` text
Desktop development with C++
```

Make sure the following are available:

-   MSVC C++ compiler
-   Windows SDK
-   CMake tools for Windows
-   MSBuild

Verify from a terminal:

``` cmd
cl
```

You should see the Microsoft C/C++ compiler information.

Verify CMake:

``` cmd
cmake --version
```

------------------------------------------------------------------------

# 4. Install vcpkg

The project uses one global vcpkg installation.

Recommended location:

``` text
C:\vcpkg
```

If vcpkg is not installed:

``` cmd
cd /d C:\
git clone https://github.com/microsoft/vcpkg.git
cd vcpkg
bootstrap-vcpkg.bat
```

After installation, verify:

``` cmd
C:\vcpkg\vcpkg.exe version
```

------------------------------------------------------------------------

# 5. Add vcpkg to PATH

Add:

``` text
C:\vcpkg
```

to the Windows PATH environment variable.

Verify:

``` cmd
where vcpkg
```

Expected:

``` text
C:\vcpkg\vcpkg.exe
```

------------------------------------------------------------------------

# 6. Set VCPKG_ROOT

Set the environment variable:

``` cmd
setx VCPKG_ROOT C:\vcpkg
```

Close and reopen the terminal after running `setx`.

Verify:

``` cmd
echo %VCPKG_ROOT%
```

Expected:

``` text
C:\vcpkg
```

Also verify:

``` cmd
where vcpkg
```

Expected:

``` text
C:\vcpkg\vcpkg.exe
```

------------------------------------------------------------------------

# 7. Project vcpkg Configuration

The project uses:

``` text
CMAKE_TOOLCHAIN_FILE
```

to tell CMake where the global vcpkg CMake integration is located.

It resolves to:

``` text
C:\vcpkg\scripts\buildsystems\vcpkg.cmake
```

The project uses:

``` text
VCPKG_INSTALLED_DIR
```

to tell vcpkg where this project's packages should be installed.

It resolves to:

``` text
C:\Projects\1nternet-Downloader\core\vcpkg_installed
```

Therefore:

``` text
C:\vcpkg
    ↓
vcpkg tool

core\vcpkg_installed
    ↓
project dependencies
```

------------------------------------------------------------------------

# 8. Clone/Open the Project

Example:

``` cmd
cd /d C:\Projects
git clone <YOUR_REPOSITORY_URL> 1nternet-Downloader
cd 1nternet-Downloader\core
```

If the project already exists:

``` cmd
cd /d C:\Projects\1nternet-Downloader\core
```

Verify the project:

``` cmd
dir
```

You should see:

``` text
CMakeLists.txt
CMakePresets.json
vcpkg.json
src
include
```

------------------------------------------------------------------------

# 9. vcpkg Manifest

The project should contain:

``` text
core\vcpkg.json
```

This file defines the project's dependencies.

For example, dependencies can include:

``` text
curl
openssl
zlib
sqlite3
nlohmann-json
fmt
spdlog
wxwidgets
```

Do not manually maintain library paths in `CMakeLists.txt`.

CMake uses the vcpkg toolchain and the project's manifest to locate the
dependencies.

------------------------------------------------------------------------

# 10. CMake Configuration

The project contains:

``` text
core\CMakePresets.json
```

The default preset uses:

``` text
CMAKE_TOOLCHAIN_FILE
    ↓
$env{VCPKG_ROOT}/scripts/buildsystems/vcpkg.cmake

VCPKG_INSTALLED_DIR
    ↓
${sourceDir}/vcpkg_installed
```

The build directory is:

``` text
core\build
```

Configure Debug:

``` cmd
cd /d C:\Projects\1nternet-Downloader\core
cmake --preset default
```

CMake will configure the project and vcpkg will install the required
packages into:

``` text
core\vcpkg_installed
```

------------------------------------------------------------------------

# 11. Build Debug

After configuration:

``` cmd
cmake --build --preset default
```

The Debug executable will normally be:

``` text
core\build\Debug\InternetDownloader.exe
```

Run:

``` cmd
build\Debug\InternetDownloader.exe
```

------------------------------------------------------------------------

# 12. Build Release

For a production build:

``` cmd
cmake --preset release
```

Then:

``` cmd
cmake --build --preset release
```

The executable will normally be:

``` text
core\build\Release\InternetDownloader.exe
```

Run:

``` cmd
build\Release\InternetDownloader.exe
```

------------------------------------------------------------------------

# 13. Clean Build

If the build becomes corrupted or you want to configure from scratch:

``` cmd
cd /d C:\Projects\1nternet-Downloader\core
rmdir /s /q build
cmake --preset default
cmake --build --preset default
```

For Release:

``` cmd
rmdir /s /q build
cmake --preset release
cmake --build --preset release
```

Do not delete `C:\vcpkg` when cleaning the project.

------------------------------------------------------------------------

# 14. Clean Project Dependencies

If you want to reinstall the project's packages from scratch:

``` cmd
cd /d C:\Projects\1nternet-Downloader\core
rmdir /s /q vcpkg_installed
rmdir /s /q build
cmake --preset default
```

This recreates:

``` text
core\vcpkg_installed
```

using the global:

``` text
C:\vcpkg
```

------------------------------------------------------------------------

# 15. CMakeLists.txt

`CMakeLists.txt` describes what the application is and how it is built.

It defines:

-   C++ standard
-   source files
-   executable
-   include directories
-   dependencies
-   libraries
-   compiler warnings
-   installation rules

The application target is:

``` text
InternetDownloader
```

The source files are collected with:

``` cmake
file(GLOB_RECURSE CORE_SRCS "src/*.cpp")
```

The executable is created with:

``` cmake
add_executable(InternetDownloader WIN32
    main.cpp
    ${CORE_SRCS}
)
```

Dependencies are found with:

``` cmake
find_package(CURL REQUIRED)
find_package(OpenSSL REQUIRED)
find_package(ZLIB REQUIRED)
find_package(unofficial-sqlite3 CONFIG REQUIRED)
find_package(nlohmann_json CONFIG REQUIRED)
find_package(fmt CONFIG REQUIRED)
find_package(spdlog CONFIG REQUIRED)
find_package(wxWidgets CONFIG REQUIRED)
```

------------------------------------------------------------------------

# 16. CMakePresets.json

`CMakePresets.json` defines build configurations.

The project has:

``` text
default
release
```

The default configuration uses:

``` text
Debug
```

The release configuration uses:

``` text
Release
```

The important paths are:

``` text
CMAKE_TOOLCHAIN_FILE
    C:\vcpkg\scripts\buildsystems\vcpkg.cmake

VCPKG_INSTALLED_DIR
    core\vcpkg_installed

binaryDir
    core\build
```

------------------------------------------------------------------------

# 17. Why vcpkg_installed Is Inside the Project

The project intentionally keeps dependencies here:

``` text
core\vcpkg_installed
```

rather than:

``` text
C:\vcpkg_installed
```

or:

``` text
core\build\vcpkg_installed
```

This provides a clear separation:

``` text
C:\vcpkg
    = package manager/tool

core\vcpkg_installed
    = packages used by this project

core\build
    = CMake/Visual Studio generated build files
```

------------------------------------------------------------------------

# 18. Source Code Structure

The main source directory is:

``` text
core\src
```

The downloader should be organized by responsibility.

Typical areas include:

``` text
src\
├── download\
├── network\
├── gui\
├── torrent\
├── storage\
└── utils\
```

The general downloader flow is:

``` text
GUI
 ↓
Download Manager
 ↓
Download
 ↓
Network / HTTP
 ↓
Worker
 ↓
Segment
 ↓
File Storage
```

------------------------------------------------------------------------

# 19. Downloader Concepts

When developing the downloader, understand these concepts:

## HTTP

Learn:

-   GET
-   HEAD
-   response status codes
-   headers
-   redirects
-   Content-Length
-   Content-Type
-   cookies
-   authentication

## Range Requests

Range requests allow a large file to be divided into pieces.

Example:

``` http
Range: bytes=0-1048575
```

A downloader can use multiple ranges:

``` text
Worker 1 → bytes 0 - 25 MB
Worker 2 → bytes 25 - 50 MB
Worker 3 → bytes 50 - 75 MB
Worker 4 → bytes 75 - 100 MB
```

## Resume

The application should track downloaded portions so an interrupted
download can continue.

## Retry

Network failures should be recoverable through:

-   timeout
-   retry
-   reconnect
-   failed segment handling

## Queue

Downloads should be managed as jobs:

``` text
Queued
  ↓
Downloading
  ↓
Paused / Failed / Completed
```

## Progress

Track:

``` text
total bytes
downloaded bytes
percentage
download speed
remaining time
```

------------------------------------------------------------------------

# 20. Production Build

Before creating a production build:

``` cmd
cd /d C:\Projects\1nternet-Downloader\core
cmake --preset release
cmake --build --preset release
```

Verify:

``` text
core\build\Release\InternetDownloader.exe
```

For distribution, the executable also requires its runtime DLL
dependencies.

Do not copy only the `.exe` and assume all dependencies are embedded.

Typical runtime dependencies include DLLs from:

``` text
libcurl
OpenSSL
zlib
wxWidgets
libpng
libjpeg
libwebp
SQLite
```

The exact DLL set depends on the final build and linked libraries.

------------------------------------------------------------------------

# 21. Troubleshooting

## vcpkg command not found

Run:

``` cmd
where vcpkg
```

Expected:

``` text
C:\vcpkg\vcpkg.exe
```

If it is not found, check PATH.

## VCPKG_ROOT not set

Run:

``` cmd
echo %VCPKG_ROOT%
```

Expected:

``` text
C:\vcpkg
```

If missing:

``` cmd
setx VCPKG_ROOT C:\vcpkg
```

Then reopen the terminal.

## CMake cannot find packages

Clean the build:

``` cmd
rmdir /s /q build
```

Then configure again:

``` cmd
cmake --preset default
```

## DLL Bad Image error

Make sure the application and native DLLs use the same architecture.

This project uses:

``` text
x64
```

Do not mix:

``` text
x86 DLL + x64 EXE
```

or:

``` text
x64 DLL + x86 EXE
```

A clean rebuild is recommended:

``` cmd
rmdir /s /q build
cmake --preset default
cmake --build --preset default
```

------------------------------------------------------------------------

# 22. Recommended Daily Development Commands

### Start development

``` cmd
cd /d C:\Projects\1nternet-Downloader\core
```

### Configure

``` cmd
cmake --preset default
```

### Build Debug

``` cmd
cmake --build --preset default
```

### Build Release

``` cmd
cmake --preset release
cmake --build --preset release
```

### Run Debug

``` cmd
build\Debug\InternetDownloader.exe
```

### Run Release

``` cmd
build\Release\InternetDownloader.exe
```

------------------------------------------------------------------------

# 23. Important Rules

1.  Keep only one vcpkg installation at:

``` text
C:\vcpkg
```

2.  Keep project dependencies at:

``` text
core\vcpkg_installed
```

3.  Do not put another vcpkg installation inside the project.

4.  Do not hardcode dependency include paths such as:

``` cmake
build/vcpkg_installed/x64-windows/include
```

5.  Let the vcpkg toolchain provide include and library paths.

6.  Keep dependencies in:

``` text
vcpkg.json
```

7.  Use CMake presets for Debug and Release.

8.  Build x64 consistently.

------------------------------------------------------------------------

# 24. Quick Setup

For a machine that already has Visual Studio, CMake, Git, and vcpkg
installed:

``` cmd
where vcpkg
echo %VCPKG_ROOT%
cd /d C:\Projects\1nternet-Downloader\core
cmake --preset default
cmake --build --preset default
```

For Release:

``` cmd
cd /d C:\Projects\1nternet-Downloader\core
cmake --preset release
cmake --build --preset release
```

The resulting application is:

``` text
core\build\Debug\InternetDownloader.exe
```

or:

``` text
core\build\Release\InternetDownloader.exe
```
