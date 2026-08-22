# Internet Downloader

A high-performance, modular, and extensible download manager built in modern C++.

## Features

- **Multi-Protocol Support**: Supports HTTP and HTTPS via `libcurl`.
- **Advanced Download Management**: Pause, resume, and schedule downloads concurrently.
- **Robust Storage**: Efficient file and metadata management.
- **Native GUI**: Modern, native user interface built with `wxWidgets` (retained-mode) rather than a game engine loop, ensuring low CPU usage and proper desktop integration.

## Architecture

The project is structured entirely within the `core/` directory:

- `core/src/download/` & `core/include/download/`: The core multithreaded download engine utilizing libcurl.
- `core/src/gui/` & `core/include/gui/`: The graphical user interface utilizing `wxWidgets`.
- `core/src/system/`, `network/`, `storage/`: Backend scaffolding.

## Build Instructions

This project uses **CMake** and **vcpkg** for dependency management. Dependencies (like wxWidgets and curl) are fetched and built automatically.

### Prerequisites
- [CMake](https://cmake.org/) (3.15+)
- Visual Studio / MSVC toolchain (on Windows)
- `vcpkg` (Environment variable `VCPKG_ROOT` must be set)

### Building the Project (Windows)

The simplest way to build the project is by running the included PowerShell script from the `core` directory:

```powershell
cd core
./scripts/build.ps1
```

*(Note: The build script will automatically detect and load your Visual Studio `vcvars64.bat` environment to ensure the compiler and linker can find the Windows SDK).*

### Building Manually via CMake

You can also build the project manually via standard CMake commands in the `core` directory:

```bash
cd core
cmake -B out/build/x64-Debug -S . -DCMAKE_TOOLCHAIN_FILE="$env:VCPKG_ROOT/scripts/buildsystems/vcpkg.cmake"
cmake --build out/build/x64-Debug
```

After a successful build, the main executable will be located in `core/out/build/x64-Debug/InternetDownloader.exe`.

## License

See the `LICENSE` file for details.
