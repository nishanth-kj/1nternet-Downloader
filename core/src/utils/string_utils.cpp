#include "utils/string_utils.h"

#include <cctype>
#include <cstdlib>

namespace idr {
namespace utils {

std::string UrlDecode(const std::string& input) {
    std::string out;
    out.reserve(input.size());

    for (size_t i = 0; i < input.size(); ++i) {
        char c = input[i];
        if (c == '%' && i + 2 < input.size() && std::isxdigit(static_cast<unsigned char>(input[i + 1])) &&
            std::isxdigit(static_cast<unsigned char>(input[i + 2]))) {
            std::string hex = input.substr(i + 1, 2);
            out.push_back(static_cast<char>(std::strtol(hex.c_str(), nullptr, 16)));
            i += 2;
        } else if (c == '+') {
            out.push_back(' ');
        } else {
            out.push_back(c);
        }
    }

    return out;
}

} // namespace utils
} // namespace idr
