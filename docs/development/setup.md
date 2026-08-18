# Development Setup

## Prerequisites

- CMake (version 3.15 or higher)
- A C11 compatible compiler (GCC, Clang, or MSVC)

## Workflow

1. Configure the project:
   ```bash
   cmake -B build -S .
   ```
2. Build the project:
   ```bash
   cmake --build build
   ```

## Adding New Files

The `CMakeLists.txt` is configured to use `file(GLOB_RECURSE)` to automatically pick up any new `.c` files added to the `core/src/` or `gui/src/` directories. You do not need to manually add files to the CMake configuration unless you add a new top-level source directory.
