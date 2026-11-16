#include "utils/system/logger.h"

#include "utils/system/platform.h"

#include <spdlog/sinks/null_sink.h>
#include <spdlog/sinks/rotating_file_sink.h>

namespace idr {
namespace system {

std::shared_ptr<spdlog::logger> Log() {
    static std::shared_ptr<spdlog::logger> logger = []() {
        std::shared_ptr<spdlog::logger> l;
        try {
            const auto logDir = AppDataDir() / "logs";
            std::error_code ec;
            std::filesystem::create_directories(logDir, ec);
            auto sink = std::make_shared<spdlog::sinks::rotating_file_sink_mt>(
                (logDir / "idr.log").string(), 5 * 1024 * 1024, 3);
            l = std::make_shared<spdlog::logger>("idr", sink);
            l->set_pattern("%Y-%m-%d %H:%M:%S.%e [%l] [t%t] %v");
            l->set_level(spdlog::level::info);
            // Low-volume log (lifecycle events only): flush every line so the file
            // is complete even if the process is killed.
            l->flush_on(spdlog::level::info);
        } catch (...) {
            l = std::make_shared<spdlog::logger>(
                "idr", std::make_shared<spdlog::sinks::null_sink_mt>());
        }
        return l;
    }();
    return logger;
}

} // namespace system
} // namespace idr
