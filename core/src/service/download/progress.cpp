#include "service/download/progress.h"

namespace idr {
namespace download {

void SpeedMeter::Reset(uint64_t baselineBytes) {
    std::lock_guard<std::mutex> lock(m_mutex);
    m_lastTime = Clock::now();
    m_lastBytes = baselineBytes;
    m_speed = 0.0;
    m_hasSample = false;
}

double SpeedMeter::Update(uint64_t totalBytes) {
    std::lock_guard<std::mutex> lock(m_mutex);
    const auto now = Clock::now();
    const auto elapsed = now - m_lastTime;
    if (elapsed < m_interval) return m_speed;

    const double seconds = std::chrono::duration<double>(elapsed).count();
    const uint64_t delta = totalBytes >= m_lastBytes ? totalBytes - m_lastBytes : 0;
    const double instant = static_cast<double>(delta) / seconds;

    m_speed = m_hasSample ? (0.5 * instant + 0.5 * m_speed) : instant;
    m_hasSample = true;
    m_lastTime = now;
    m_lastBytes = totalBytes;
    return m_speed;
}

} // namespace download
} // namespace idr
