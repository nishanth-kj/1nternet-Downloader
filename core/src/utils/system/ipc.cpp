#include "utils/system/ipc.h"

#ifdef _WIN32
#include <windows.h>
#include <thread>
#endif

namespace idr {
namespace system {

#ifdef _WIN32

namespace {
const wchar_t* kMutexName = L"InternetDownloader_SingleInstance_Mutex";
const wchar_t* kPipeName = L"\\\\.\\pipe\\InternetDownloader_IPC";
} // namespace

bool SingleInstanceGuard::AnotherInstanceIsRunning() {
    // Intentionally leaked: the handle must live for the process's lifetime so the
    // mutex stays held until we exit, and Windows closes it automatically at that point.
    HANDLE mutex = CreateMutexW(nullptr, FALSE, kMutexName);
    return mutex != nullptr && GetLastError() == ERROR_ALREADY_EXISTS;
}

bool SingleInstanceGuard::SendUrlToRunningInstance(const std::string& url) {
    HANDLE pipe = CreateFileW(kPipeName, GENERIC_WRITE, 0, nullptr, OPEN_EXISTING, 0, nullptr);
    if (pipe == INVALID_HANDLE_VALUE) return false;

    DWORD written = 0;
    BOOL ok = WriteFile(pipe, url.data(), static_cast<DWORD>(url.size()), &written, nullptr);
    FlushFileBuffers(pipe);
    CloseHandle(pipe);
    return ok && static_cast<size_t>(written) == url.size();
}

void SingleInstanceGuard::StartServer(std::function<void(std::string)> onUrlReceived) {
    std::thread([onUrlReceived]() {
        for (;;) {
            HANDLE pipe = CreateNamedPipeW(
                kPipeName,
                PIPE_ACCESS_INBOUND,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                PIPE_UNLIMITED_INSTANCES,
                0, 4096, 0, nullptr);

            if (pipe == INVALID_HANDLE_VALUE) return;

            BOOL connected = ConnectNamedPipe(pipe, nullptr) ? TRUE : (GetLastError() == ERROR_PIPE_CONNECTED);
            if (connected) {
                std::string buffer;
                char chunk[512];
                DWORD bytesRead = 0;
                while (ReadFile(pipe, chunk, sizeof(chunk), &bytesRead, nullptr) && bytesRead > 0) {
                    buffer.append(chunk, bytesRead);
                }
                if (!buffer.empty() && onUrlReceived) {
                    onUrlReceived(buffer);
                }
            }

            DisconnectNamedPipe(pipe);
            CloseHandle(pipe);
        }
    }).detach();
}

#else

bool SingleInstanceGuard::AnotherInstanceIsRunning() { return false; }
bool SingleInstanceGuard::SendUrlToRunningInstance(const std::string&) { return false; }
void SingleInstanceGuard::StartServer(std::function<void(std::string)>) {}

#endif

} // namespace system
} // namespace idr
