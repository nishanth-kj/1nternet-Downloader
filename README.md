# Internet Downloader

A high-performance, modular, and extensible download manager built in C.

## Features

- **Multi-Protocol Support**: Supports HTTP, HTTPS, and BitTorrent out of the box.
- **Advanced Download Management**: Pause, resume, and schedule downloads.
- **Parallel Downloading**: Segmented chunk downloads for maximum speed.
- **Robust Storage**: Efficient file and metadata management.
- **Clean GUI**: Modern, responsive user interface.

## Build Instructions

This project uses CMake for dependency management. Dependencies are fetched automatically during configure.

### Prerequisites
- [CMake](https://cmake.org/) (3.15+)

### Building the Project (Development/Debug)
To automatically fetch dependencies and compile the project using the default preset:

```bash
# Configure the project
cmake --preset default

# Build the project
cmake --build --preset default
```

After a successful build, the main executable will be located in `build/Debug/InternetDownloader.exe` (on Windows).

### Building for Production (Release)
To build a highly optimized, production-level version of the app:

```bash
# Configure the release build
cmake --preset release

# Build the release executable
cmake --build --preset release
```
The production executable will be located in `build/Release/InternetDownloader.exe`.

## Project Structure

- `app/`: Contains the main application entry point.
- `core/`: The core download engine and systems (network, torrent, storage, security).
- `gui/`: The graphical user interface components and views.
- `docs/`: Technical documentation and development guides.
- `tests/`: Automated unit and integration tests.

## License

See the `LICENSE` file for details.
