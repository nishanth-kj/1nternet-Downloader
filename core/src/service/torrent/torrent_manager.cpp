#include "service/torrent/manager.h"

#include <libtorrent/add_torrent_params.hpp>
#include <libtorrent/load_torrent.hpp>
#include <libtorrent/magnet_uri.hpp>
#include <libtorrent/session.hpp>
#include <libtorrent/session_params.hpp>
#include <libtorrent/settings_pack.hpp>
#include <libtorrent/torrent_flags.hpp>
#include <libtorrent/torrent_handle.hpp>
#include <libtorrent/torrent_info.hpp>
#include <libtorrent/torrent_status.hpp>

#include <algorithm>
#include <filesystem>
#include <mutex>
#include <system_error>

namespace lt = libtorrent;

namespace idr {
namespace torrent {

namespace {

lt::session_params BuildSessionParams() {
    lt::settings_pack pack;
    pack.set_str(lt::settings_pack::listen_interfaces, "0.0.0.0:6881,[::]:6881");
    pack.set_bool(lt::settings_pack::enable_dht, true);
    pack.set_bool(lt::settings_pack::enable_lsd, true);
    pack.set_bool(lt::settings_pack::enable_upnp, true);
    pack.set_bool(lt::settings_pack::enable_natpmp, true);
    return lt::session_params(pack);
}

} // namespace

// ---------------------------------------------------------------------------
// TorrentTask
// ---------------------------------------------------------------------------

struct TorrentTask::Impl {
    lt::torrent_handle handle;
};

TorrentTask::TorrentTask(int id, std::string source, std::string destinationDir)
    : m_impl(std::make_unique<Impl>())
    , m_id(id)
    , m_source(std::move(source))
    , m_destinationDir(std::move(destinationDir))
{
}

TorrentTask::~TorrentTask() = default;

TorrentSnapshot TorrentTask::GetSnapshot() const {
    TorrentSnapshot snap;

    if (!m_impl || !m_impl->handle.is_valid()) {
        snap.status = TorrentStatus::Error;
        return snap;
    }

    lt::torrent_status st = m_impl->handle.status();

    snap.name = st.name;
    snap.totalBytes = st.total_wanted > 0 ? static_cast<uint64_t>(st.total_wanted) : 0;
    snap.downloadedBytes = st.total_wanted_done > 0 ? static_cast<uint64_t>(st.total_wanted_done) : 0;
    snap.uploadedBytes = st.total_payload_upload > 0 ? static_cast<uint64_t>(st.total_payload_upload) : 0;
    snap.downloadSpeed = static_cast<double>(st.download_payload_rate);
    snap.uploadSpeed = static_cast<double>(st.upload_payload_rate);
    snap.peers = st.num_peers;
    snap.seeds = st.num_seeds;

    const bool paused = static_cast<bool>(st.flags & lt::torrent_flags::paused);

    if (st.errc) {
        snap.status = TorrentStatus::Error;
    } else if (paused) {
        snap.status = TorrentStatus::Paused;
    } else if (st.state == lt::torrent_status::seeding) {
        snap.status = TorrentStatus::Seeding;
    } else if (st.is_finished || st.state == lt::torrent_status::finished) {
        snap.status = TorrentStatus::Completed;
    } else {
        snap.status = TorrentStatus::Downloading;
    }

    return snap;
}

void TorrentTask::Pause() {
    if (m_impl && m_impl->handle.is_valid()) m_impl->handle.pause();
}

void TorrentTask::Resume() {
    if (m_impl && m_impl->handle.is_valid()) m_impl->handle.resume();
}

void TorrentTask::Stop() {
    Pause();
}

// ---------------------------------------------------------------------------
// TorrentManager
// ---------------------------------------------------------------------------

struct TorrentManager::Impl {
    lt::session session;
    std::vector<std::shared_ptr<TorrentTask>> tasks;
    std::mutex mutex;
    int nextId{1};

    Impl() : session(BuildSessionParams()) {}
};

TorrentManager::TorrentManager() : m_impl(std::make_unique<Impl>()) {}
TorrentManager::~TorrentManager() = default;

TorrentManager& TorrentManager::GetInstance() {
    static TorrentManager instance;
    return instance;
}

std::shared_ptr<TorrentTask> TorrentManager::AddTorrent(const std::string& source, const std::string& destinationDir, bool autoStart) {
    std::error_code fsEc;
    std::filesystem::create_directories(destinationDir, fsEc);

    lt::add_torrent_params atp;
    lt::error_code ltEc;

    if (source.rfind("magnet:", 0) == 0) {
        atp = lt::parse_magnet_uri(source, ltEc);
        if (ltEc) return nullptr;
    } else {
        try {
            atp = lt::load_torrent_file(source);
        } catch (const std::exception&) {
            return nullptr;
        }
        if (!atp.ti || !atp.ti->is_valid()) return nullptr;
    }

    atp.save_path = destinationDir;
    if (!autoStart) {
        atp.flags |= lt::torrent_flags::paused;
    }

    lt::torrent_handle handle;
    try {
        handle = m_impl->session.add_torrent(std::move(atp));
    } catch (const std::exception&) {
        return nullptr;
    }

    if (!handle.is_valid()) return nullptr;

    std::lock_guard<std::mutex> lock(m_impl->mutex);
    int id = m_impl->nextId++;

    auto task = std::shared_ptr<TorrentTask>(new TorrentTask(id, source, destinationDir));
    task->m_impl->handle = handle;
    m_impl->tasks.push_back(task);

    return task;
}

std::vector<std::shared_ptr<TorrentTask>> TorrentManager::GetTorrents() {
    std::lock_guard<std::mutex> lock(m_impl->mutex);
    return m_impl->tasks;
}

std::shared_ptr<TorrentTask> TorrentManager::FindTorrent(int id) {
    std::lock_guard<std::mutex> lock(m_impl->mutex);
    auto it = std::find_if(m_impl->tasks.begin(), m_impl->tasks.end(),
        [id](const std::shared_ptr<TorrentTask>& t) { return t->GetId() == id; });
    return it != m_impl->tasks.end() ? *it : nullptr;
}

void TorrentManager::RemoveTorrent(int id) {
    std::lock_guard<std::mutex> lock(m_impl->mutex);
    auto it = std::find_if(m_impl->tasks.begin(), m_impl->tasks.end(),
        [id](const std::shared_ptr<TorrentTask>& t) { return t->GetId() == id; });

    if (it != m_impl->tasks.end()) {
        if ((*it)->m_impl && (*it)->m_impl->handle.is_valid()) {
            m_impl->session.remove_torrent((*it)->m_impl->handle);
        }
        m_impl->tasks.erase(it);
    }
}

void TorrentManager::PauseAll() {
    std::lock_guard<std::mutex> lock(m_impl->mutex);
    for (auto& t : m_impl->tasks) t->Pause();
}

void TorrentManager::ResumeAll() {
    std::lock_guard<std::mutex> lock(m_impl->mutex);
    for (auto& t : m_impl->tasks) t->Resume();
}

} // namespace torrent
} // namespace idr
