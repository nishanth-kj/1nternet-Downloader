#pragma once

#include <memory>

#include <spdlog/spdlog.h>

namespace idr {
namespace system {

// Process-wide logger writing to AppDataDir()/logs/idr.log (rotating, 5 MiB x 3).
// Falls back to a no-op logger if the file can't be opened, so callers never need
// to null-check. Thread-safe.
std::shared_ptr<spdlog::logger> Log();

} // namespace system
} // namespace idr
