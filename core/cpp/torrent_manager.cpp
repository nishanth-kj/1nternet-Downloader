#include "manager.h"

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
#include <libtorrent/alert_types.hpp>
#include <libtorrent/announce_entry.hpp>
#include <libtorrent/peer_info.hpp>
#include <libtorrent/read_resume_data.hpp>
#include <libtorrent/write_resume_data.hpp>
#include <libtorrent/create_torrent.hpp>
#include <libtorrent/bencode.hpp>
#include <libtorrent/entry.hpp>

#include <algorithm>
#include <atomic>
#include <chrono>
#include <fstream>
#include <iterator>
#include <sstream>
#include <filesystem>
#include <mutex>
#include <system_error>

namespace lt = libtorrent;

namespace idr {
namespace torrent {

namespace {

std::mutex g_settingsMutex;
TorrentSessionSettings g_settings;

TorrentSessionSettings CurrentSettings() {
    std::lock_guard<std::mutex> lock(g_settingsMutex);
    return g_settings;
}

lt::settings_pack BuildSettingsPack(const TorrentSessionSettings& s) {
    lt::settings_pack pack;
    const int port = s.randomPort ? 0 : (std::max)(1, (std::min)(65535, s.listenPort));
    const std::string p = std::to_string(port);
    pack.set_str(lt::settings_pack::listen_interfaces, "0.0.0.0:" + p + ",[::]:" + p);
    pack.set_bool(lt::settings_pack::enable_dht, s.dht);
    pack.set_bool(lt::settings_pack::enable_lsd, s.lsd);
    pack.set_bool(lt::settings_pack::enable_upnp, s.upnp);
    pack.set_bool(lt::settings_pack::enable_natpmp, s.natpmp);
    pack.set_bool(lt::settings_pack::enable_incoming_utp, s.utp);
    pack.set_bool(lt::settings_pack::enable_outgoing_utp, s.utp);
    const int policy = s.encryption == 0 ? lt::settings_pack::pe_forced
                     : s.encryption == 2 ? lt::settings_pack::pe_disabled
                                         : lt::settings_pack::pe_enabled;
    pack.set_int(lt::settings_pack::out_enc_policy, policy);
    pack.set_int(lt::settings_pack::in_enc_policy, policy);
    pack.set_int(lt::settings_pack::allowed_enc_level, lt::settings_pack::pe_both);
    pack.set_int(lt::settings_pack::connections_limit, (std::max)(10, s.maxConnections));
    pack.set_int(lt::settings_pack::download_rate_limit, (std::max)(0, s.downloadLimitKBps) * 1024);
    pack.set_int(lt::settings_pack::upload_rate_limit, (std::max)(0, s.uploadLimitKBps) * 1024);
    pack.set_bool(lt::settings_pack::announce_to_all_trackers, s.announceToAllTrackers);
    pack.set_bool(lt::settings_pack::announce_to_all_tiers, s.announceToAllTrackers);
    pack.set_bool(lt::settings_pack::validate_https_trackers, s.validateHttpsTrackers);
    if (!s.userAgent.empty()) pack.set_str(lt::settings_pack::user_agent, s.userAgent);
    return pack;
}

lt::session_params BuildSessionParams() {
    return lt::session_params(BuildSettingsPack(CurrentSettings()));
}

// Set once the session exists, so ApplySettings knows whether to push changes now.
std::atomic<TorrentManager*> g_liveManager{nullptr};

template <typename Hash>
std::string HexOf(const Hash& hash) {
    std::ostringstream os;
    os << hash;
    return os.str();
}

std::filesystem::path ResumeFile(const std::string& dir, const lt::info_hash_t& ih) {
    return std::filesystem::path(dir) / (HexOf(ih.get_best()) + ".fastresume");
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
    if (m_impl && m_impl->handle.is_valid()) {
        // An auto-managed torrent can be resumed by the session's queueing logic,
        // which would undo a pause the user asked for.
        m_impl->handle.unset_flags(lt::torrent_flags::auto_managed);
        m_impl->handle.pause();
    }
}

void TorrentTask::Resume() {
    if (m_impl && m_impl->handle.is_valid()) {
        m_impl->handle.unset_flags(lt::torrent_flags::auto_managed);
        m_impl->handle.resume();
    }
}

std::string TorrentTask::GetError() const {
    if (!m_impl || !m_impl->handle.is_valid()) return "torrent handle is no longer valid";
    lt::torrent_status st = m_impl->handle.status();
    return st.errc ? st.errc.message() : std::string();
}

std::vector<TorrentFileInfo> TorrentTask::GetFiles() const {
    std::vector<TorrentFileInfo> out;
    if (!m_impl || !m_impl->handle.is_valid()) return out;
    auto ti = m_impl->handle.torrent_file();
    if (!ti) return out; // magnet without metadata yet
    const lt::file_storage& fs = ti->layout();
    std::vector<std::int64_t> progress = m_impl->handle.file_progress(lt::torrent_handle::piece_granularity);
    std::vector<lt::download_priority_t> prios = m_impl->handle.get_file_priorities();
    for (lt::file_index_t i : fs.file_range()) {
        if (fs.pad_file_at(i)) continue;
        const int idx = static_cast<int>(i);
        TorrentFileInfo f;
        f.index = idx;
        f.path = fs.file_path(i);
        f.size = static_cast<uint64_t>(fs.file_size(i));
        f.done = idx < static_cast<int>(progress.size()) ? static_cast<uint64_t>(progress[idx]) : 0;
        f.priority = idx < static_cast<int>(prios.size()) ? static_cast<int>(static_cast<std::uint8_t>(prios[idx])) : 4;
        out.push_back(std::move(f));
    }
    return out;
}

void TorrentTask::SetFilePriority(int index, int priority) {
    if (!m_impl || !m_impl->handle.is_valid() || index < 0) return;
    const int p = (std::max)(0, (std::min)(7, priority));
    m_impl->handle.file_priority(lt::file_index_t{index}, lt::download_priority_t{static_cast<std::uint8_t>(p)});
}

std::vector<TorrentTrackerInfo> TorrentTask::GetTrackers() const {
    std::vector<TorrentTrackerInfo> out;
    if (!m_impl || !m_impl->handle.is_valid()) return out;
    for (const lt::announce_entry& ae : m_impl->handle.trackers()) {
        TorrentTrackerInfo t;
        t.url = ae.url;
        t.tier = ae.tier;
        bool updating = false, working = false, failed = false;
        for (const auto& ep : ae.endpoints) {
            for (const auto& ih : ep.info_hashes) {
                if (ih.updating) updating = true;
                if (ih.fails > 0) failed = true;
                if (ih.fails == 0 && ih.start_sent) working = true;
                if (ih.scrape_complete >= 0) t.seeds = (std::max)(t.seeds, ih.scrape_complete);
                if (ih.scrape_incomplete >= 0) t.peers = (std::max)(t.peers, ih.scrape_incomplete);
                if (ih.scrape_downloaded >= 0) t.downloaded = (std::max)(t.downloaded, ih.scrape_downloaded);
                if (t.message.empty()) {
                    if (!ih.message.empty()) t.message = ih.message;
                    else if (ih.last_error) t.message = ih.last_error.message();
                }
            }
        }
        t.status = updating ? 2 : working ? 1 : failed ? 3 : 0;
        out.push_back(std::move(t));
    }
    return out;
}

std::vector<TorrentPeerInfo> TorrentTask::GetPeers() const {
    std::vector<TorrentPeerInfo> out;
    if (!m_impl || !m_impl->handle.is_valid()) return out;
    std::vector<lt::peer_info> peers;
    m_impl->handle.get_peer_info(peers);
    out.reserve(peers.size());
    for (const lt::peer_info& p : peers) {
        TorrentPeerInfo o;
        std::ostringstream addr;
        const auto ep = p.remote_endpoint();
        addr << ep.address().to_string() << ":" << ep.port();
        o.address = addr.str();
        o.client = p.client;
        o.downSpeed = static_cast<double>(p.payload_down_speed);
        o.upSpeed = static_cast<double>(p.payload_up_speed);
        o.progress = p.progress;
        o.downloaded = static_cast<uint64_t>((std::max)(std::int64_t{0}, p.total_download));
        o.uploaded = static_cast<uint64_t>((std::max)(std::int64_t{0}, p.total_upload));
        o.seed = static_cast<bool>(p.flags & lt::peer_info::seed);
        out.push_back(std::move(o));
    }
    return out;
}

TorrentExtraInfo TorrentTask::GetExtra() const {
    TorrentExtraInfo x;
    if (!m_impl || !m_impl->handle.is_valid()) return x;
    lt::torrent_status st = m_impl->handle.status();
    x.allTimeUpload = static_cast<uint64_t>((std::max)(std::int64_t{0}, st.all_time_upload));
    x.allTimeDownload = static_cast<uint64_t>((std::max)(std::int64_t{0}, st.all_time_download));
    x.seedingSeconds = static_cast<int64_t>(st.seeding_duration.count());
    x.activeSeconds = static_cast<int64_t>(st.active_duration.count());
    x.piecesHave = st.num_pieces;
    x.savePath = st.save_path;
    x.infoHash = HexOf(m_impl->handle.info_hashes().get_best());
    if (auto ti = m_impl->handle.torrent_file()) {
        x.numPieces = ti->num_pieces();
        x.pieceLength = ti->piece_length();
    }
    return x;
}

void TorrentTask::SetSequential(bool on) {
    if (!m_impl || !m_impl->handle.is_valid()) return;
    if (on) m_impl->handle.set_flags(lt::torrent_flags::sequential_download);
    else m_impl->handle.unset_flags(lt::torrent_flags::sequential_download);
}

bool TorrentTask::IsSequential() const {
    if (!m_impl || !m_impl->handle.is_valid()) return false;
    return static_cast<bool>(m_impl->handle.flags() & lt::torrent_flags::sequential_download);
}

void TorrentTask::AddTracker(const std::string& url) {
    if (!m_impl || !m_impl->handle.is_valid() || url.empty()) return;
    m_impl->handle.add_tracker(lt::announce_entry(url));
}

void TorrentTask::ForceRecheck() {
    if (m_impl && m_impl->handle.is_valid()) m_impl->handle.force_recheck();
}

void TorrentTask::ForceReannounce() {
    if (m_impl && m_impl->handle.is_valid()) m_impl->handle.force_reannounce();
}

std::vector<uint8_t> TorrentTask::GetPieces() const {
    std::vector<uint8_t> out;
    if (!m_impl || !m_impl->handle.is_valid()) return out;
    lt::torrent_status st = m_impl->handle.status(lt::torrent_handle::query_pieces);
    out.resize(static_cast<size_t>(st.pieces.size()));
    for (int i = 0; i < st.pieces.size(); ++i) {
        out[static_cast<size_t>(i)] = st.pieces.get_bit(lt::piece_index_t{i}) ? 1 : 0;
    }
    return out;
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
    std::mutex resumeMutex; // only one caller may pop alerts at a time
    std::string resumeDir;
    int nextId{1};

    Impl() : session(BuildSessionParams()) {}
};

TorrentManager::TorrentManager() : m_impl(std::make_unique<Impl>()) {
    g_liveManager.store(this);
}
TorrentManager::~TorrentManager() {
    g_liveManager.store(nullptr);
}

void TorrentManager::ApplySettings(const TorrentSessionSettings& settings) {
    {
        std::lock_guard<std::mutex> lock(g_settingsMutex);
        g_settings = settings;
    }
    if (TorrentManager* live = g_liveManager.load()) {
        live->m_impl->session.apply_settings(BuildSettingsPack(settings));
    }
}

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

    // Resume from saved state (pieces, file priorities, totals) when we have it.
    {
        std::string dir;
        {
            std::lock_guard<std::mutex> lock(m_impl->mutex);
            dir = m_impl->resumeDir;
        }
        const lt::info_hash_t ih = atp.ti ? atp.ti->info_hashes() : atp.info_hashes;
        if (!dir.empty() && ih.has_v1() | ih.has_v2()) {
            std::ifstream in(ResumeFile(dir, ih), std::ios::binary);
            if (in) {
                std::vector<char> buf((std::istreambuf_iterator<char>(in)), std::istreambuf_iterator<char>());
                lt::error_code rec;
                lt::add_torrent_params resumed = lt::read_resume_data(buf, rec);
                if (!rec) {
                    if (!resumed.ti && atp.ti) resumed.ti = atp.ti;
                    resumed.flags = atp.flags;
                    atp = std::move(resumed);
                }
            }
        }
    }

    atp.save_path = destinationDir;
    {
        const TorrentSessionSettings s = CurrentSettings();
        if (!s.pex) atp.flags |= lt::torrent_flags::disable_pex;
        atp.max_connections = (std::max)(2, s.maxPeersPerTorrent);
    }
    // libtorrent adds torrents paused + auto-managed and lets its queue start them. We
    // manage start/pause ourselves (so a user pause sticks), so set the state explicitly.
    atp.flags &= ~lt::torrent_flags::auto_managed;
    if (autoStart) {
        atp.flags &= ~lt::torrent_flags::paused;
    } else {
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

void TorrentManager::RemoveTorrent(int id, bool deleteFiles) {
    std::lock_guard<std::mutex> lock(m_impl->mutex);
    auto it = std::find_if(m_impl->tasks.begin(), m_impl->tasks.end(),
        [id](const std::shared_ptr<TorrentTask>& t) { return t->GetId() == id; });

    if (it != m_impl->tasks.end()) {
        if ((*it)->m_impl && (*it)->m_impl->handle.is_valid()) {
            if (!m_impl->resumeDir.empty()) {
                std::error_code rmEc;
                std::filesystem::remove(ResumeFile(m_impl->resumeDir, (*it)->m_impl->handle.info_hashes()), rmEc);
            }
            if (deleteFiles) {
                m_impl->session.remove_torrent((*it)->m_impl->handle, lt::session::delete_files);
            } else {
                m_impl->session.remove_torrent((*it)->m_impl->handle);
            }
        }
        m_impl->tasks.erase(it);
    }
}

void TorrentManager::SetResumeDir(const std::string& dir) {
    std::error_code ec;
    std::filesystem::create_directories(dir, ec);
    std::lock_guard<std::mutex> lock(m_impl->mutex);
    m_impl->resumeDir = dir;
}

int TorrentManager::SaveResumeData(int timeoutMs) {
    std::lock_guard<std::mutex> resumeLock(m_impl->resumeMutex);
    std::string dir;
    std::vector<lt::torrent_handle> handles;
    {
        std::lock_guard<std::mutex> lock(m_impl->mutex);
        dir = m_impl->resumeDir;
        for (auto& t : m_impl->tasks) {
            if (t->m_impl && t->m_impl->handle.is_valid()) handles.push_back(t->m_impl->handle);
        }
    }
    if (dir.empty()) return 0;

    int outstanding = 0;
    for (auto& h : handles) {
        if (h.need_save_resume_data()) {
            h.save_resume_data(lt::torrent_handle::save_info_dict);
            ++outstanding;
        }
    }

    int written = 0;
    const auto deadline = std::chrono::steady_clock::now() + std::chrono::milliseconds(timeoutMs);
    while (outstanding > 0 && std::chrono::steady_clock::now() < deadline) {
        if (!m_impl->session.wait_for_alert(std::chrono::milliseconds(200))) continue;
        std::vector<lt::alert*> alerts;
        m_impl->session.pop_alerts(&alerts);
        for (lt::alert* a : alerts) {
            if (auto* rd = lt::alert_cast<lt::save_resume_data_alert>(a)) {
                --outstanding;
                std::vector<char> buf = lt::write_resume_data_buf(rd->params);
                const lt::info_hash_t ih = rd->params.ti ? rd->params.ti->info_hashes() : rd->params.info_hashes;
                std::ofstream out(ResumeFile(dir, ih), std::ios::binary | std::ios::trunc);
                if (out) {
                    out.write(buf.data(), static_cast<std::streamsize>(buf.size()));
                    ++written;
                }
            } else if (lt::alert_cast<lt::save_resume_data_failed_alert>(a)) {
                --outstanding;
            }
        }
    }
    return written;
}

void TorrentManager::PauseAll() {
    std::lock_guard<std::mutex> lock(m_impl->mutex);
    for (auto& t : m_impl->tasks) t->Pause();
}

void TorrentManager::ResumeAll() {
    std::lock_guard<std::mutex> lock(m_impl->mutex);
    for (auto& t : m_impl->tasks) t->Resume();
}

std::string CreateTorrentFile(const TorrentCreateOptions& o) {
    try {
        namespace fsys = std::filesystem;
        const fsys::path src(o.sourcePath);
        std::error_code ec;
        if (!fsys::exists(src, ec)) return "The file or folder doesn't exist.";

        std::vector<lt::create_file_entry> files = lt::list_files(src.string());
        if (files.empty()) return "There are no files to put in the torrent.";

        const int pieceSize = o.pieceSizeKiB > 0 ? o.pieceSizeKiB * 1024 : 0;
        lt::create_torrent ct(std::move(files), pieceSize);
        int tier = 0;
        for (const auto& t : o.trackers) {
            if (!t.empty()) ct.add_tracker(t, tier++);
        }
        for (const auto& w : o.webSeeds) {
            if (!w.empty()) ct.add_url_seed(w);
        }
        if (!o.comment.empty()) ct.set_comment(o.comment.c_str());
        ct.set_creator("1nternet Downloader");
        ct.set_priv(o.isPrivate);

        lt::error_code hashEc;
        lt::set_piece_hashes(ct, src.parent_path().string(), hashEc);
        if (hashEc) return "Hashing failed: " + hashEc.message();

        std::vector<char> buf = ct.generate_buf();
        std::ofstream out(o.outputFile, std::ios::binary | std::ios::trunc);
        if (!out) return "Can't write " + o.outputFile;
        out.write(buf.data(), static_cast<std::streamsize>(buf.size()));
        return out ? std::string() : "Writing the .torrent file failed.";
    } catch (const std::exception& e) {
        return std::string("Creating the torrent failed: ") + e.what();
    }
}

} // namespace torrent
} // namespace idr
