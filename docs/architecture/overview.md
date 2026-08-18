# Architecture Overview

Internet Downloader is divided into three main layers:

1. **App Layer (`app/`)**: The main entry point that initializes the application and starts the GUI.
2. **GUI Layer (`gui/`)**: A component-based graphical interface that interacts with the core engine. It manages themes, views, and dialogs.
3. **Core Engine (`core/`)**: The backend system responsible for handling the actual networking and storage.

## Core Modules

- **Network**: Handles raw HTTP/HTTPS requests, connections, proxies, and cookies.
- **Download**: Manages segments, workers, the download queue, scheduling, and progress tracking.
- **Torrent**: A full BitTorrent implementation handling DHT, peers, pieces, trackers, and metadata.
- **Storage**: Manages disk writes, file allocation, metadata, and history tracking.
- **Security**: Handles checksums and hashing for file validation.
- **System**: Platform-specific abstractions, logging, config, and filesystem utilities.
