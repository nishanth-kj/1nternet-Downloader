#pragma once

#include <functional>
#include <string>

namespace idr {
namespace system {

// Single-instance + inter-process handoff for the "idr://" protocol handler.
// When a browser extension or the registered idr:// URI handler launches a second
// InternetDownloader.exe, that second process forwards its URL to the already-running
// instance over a named pipe instead of opening a second window.
class SingleInstanceGuard {
public:
    // Returns true if another instance already holds the app-wide lock. Must be
    // called once, before StartServer(), and the guard must outlive the process.
    static bool AnotherInstanceIsRunning();

    // Sends a URL/magnet link to the already-running instance. Only meaningful when
    // AnotherInstanceIsRunning() returned true.
    static bool SendUrlToRunningInstance(const std::string& url);

    // Starts a background thread that listens for URLs forwarded by future
    // second-instance launches and invokes the callback (from that background
    // thread — the callback is responsible for marshalling to the GUI thread).
    static void StartServer(std::function<void(std::string)> onUrlReceived);
};

} // namespace system
} // namespace idr
