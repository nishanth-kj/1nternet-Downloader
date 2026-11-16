#pragma once

#include <chrono>
#include <cstdint>
#include <mutex>

namespace idr {
namespace download {

// Smoothed transfer-rate estimator. Feed it the cumulative byte count as often as
// you like (from any thread); it re-samples at most every `interval` and applies an
// exponential moving average so the displayed speed doesn't jitter per chunk.
class SpeedMeter {
public:
    explicit SpeedMeter(std::chrono::milliseconds interval = std::chrono::milliseconds(500))
        : m_interval(interval) {}

    // Starts a new measurement from `baselineBytes` (e.g. bytes already on disk).
    void Reset(uint64_t baselineBytes);

    // Records the current cumulative total and returns the smoothed bytes/second.
    double Update(uint64_t totalBytes);

private:
    using Clock = std::chrono::steady_clock;

    std::mutex m_mutex;
    std::chrono::milliseconds m_interval;
    Clock::time_point m_lastTime{Clock::now()};
    uint64_t m_lastBytes{0};
    double m_speed{0.0};
    bool m_hasSample{false};
};

} // namespace download
} // namespace idr
