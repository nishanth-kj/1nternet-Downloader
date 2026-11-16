#pragma once

// STL-only precompiled header. Keep third-party libraries (wx, curl,
// sqlite_orm, libtorrent) out of this file so torrent/DB/GUI TUs do not
// pay for headers they never include.

#include <algorithm>
#include <atomic>
#include <chrono>
#include <cstdint>
#include <filesystem>
#include <fstream>
#include <functional>
#include <memory>
#include <mutex>
#include <string>
#include <thread>
#include <utility>
#include <vector>
