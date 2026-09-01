#pragma once

#include <string>
#include <cstdint>
#include <chrono>

namespace idr {
namespace model {

/**
 * CustomModel - represents custom user configurations, rules, or metadata.
 * Can be used for extensible features like custom download rules, plugin settings, etc.
 */
struct CustomModel {
    int64_t     id          = 0;
    std::string key;                       // Unique key or category
    std::string value;                     // Payload (e.g. JSON string)
    std::string type;                      // Type identifier (e.g. "Rule", "Setting")
    std::chrono::system_clock::time_point createdAt;
    std::chrono::system_clock::time_point updatedAt;
};

} // namespace model
} // namespace idr
