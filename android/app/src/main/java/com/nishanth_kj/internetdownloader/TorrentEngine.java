package com.nishanth_kj.internetdownloader;

import android.util.Log;
import java.io.File;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import org.libtorrent4j.SessionManager;
import org.libtorrent4j.SessionParams;
import org.libtorrent4j.SettingsPack;
import org.libtorrent4j.Sha1Hash;
import org.libtorrent4j.TorrentFlags;
import org.libtorrent4j.TorrentHandle;
import org.libtorrent4j.TorrentInfo;
import org.libtorrent4j.TorrentStatus;
import org.libtorrent4j.swig.torrent_flags_t;

/**
 * Android BitTorrent engine. Independent from core/; wraps libtorrent4j
 * (same libtorrent 2.x family the desktop app uses).
 */
public final class TorrentEngine {
    private static final String TAG = "IDR_Torrent";
    private static final int ID_BASE = 100000;
    private static final TorrentEngine INSTANCE = new TorrentEngine();

    public static TorrentEngine get() {
        return INSTANCE;
    }

    public static final class Snapshot {
        public int id;
        public String source;
        public String saveDir;
        public String name;
        public long downloaded;
        public long total;
        public double speed;
        public int status;
        public int peers;
        public int seeds;
    }

    private static final class Task {
        int id;
        String source;
        String saveDir;
        String name;
        Sha1Hash hash;
    }

    private final SessionManager session = new SessionManager();
    private final Map<Integer, Task> tasks = new LinkedHashMap<>();
    private int nextId = ID_BASE;
    private boolean started;

    private TorrentEngine() {}

    public synchronized void start() {
        if (started && session.isRunning()) return;
        SettingsPack pack = new SettingsPack();
        pack.setEnableDht(true);
        pack.listenInterfaces("0.0.0.0:6881");
        session.start(new SessionParams(pack));
        started = true;
    }

    public synchronized int add(String source, File saveDir) {
        start();
        if (saveDir != null) {
            //noinspection ResultOfMethodCallIgnored
            saveDir.mkdirs();
        }

        Task task = new Task();
        task.id = ++nextId;
        task.source = source;
        task.saveDir = saveDir.getAbsolutePath();
        task.name = nameFromSource(source);

        try {
            if (source.toLowerCase(Locale.US).startsWith("magnet:")) {
                task.hash = hashFromMagnet(source);
                session.download(source, saveDir, new torrent_flags_t());
            } else {
                TorrentInfo info = new TorrentInfo(new File(source));
                if (!info.isValid()) return -1;
                task.hash = info.infoHash();
                task.name = info.name();
                session.download(info, saveDir);
            }
        } catch (Exception e) {
            Log.e(TAG, "Failed to add torrent", e);
            return -1;
        }

        tasks.put(task.id, task);
        return task.id;
    }

    public synchronized void pause(int id) {
        TorrentHandle handle = handleFor(id);
        if (handle != null && handle.isValid()) handle.pause();
    }

    public synchronized void resume(int id) {
        TorrentHandle handle = handleFor(id);
        if (handle != null && handle.isValid()) handle.resume();
    }

    public synchronized void remove(int id) {
        Task task = tasks.remove(id);
        if (task == null) return;
        TorrentHandle handle = handleFor(task);
        if (handle != null && handle.isValid()) session.remove(handle);
    }

    public synchronized void pauseAll() {
        for (Task task : tasks.values()) {
            TorrentHandle handle = handleFor(task);
            if (handle != null && handle.isValid()) handle.pause();
        }
    }

    public synchronized void resumeAll() {
        for (Task task : tasks.values()) {
            TorrentHandle handle = handleFor(task);
            if (handle != null && handle.isValid()) handle.resume();
        }
    }

    public synchronized List<Snapshot> snapshots() {
        List<Snapshot> out = new ArrayList<>();
        for (Task task : tasks.values()) {
            Snapshot snap = new Snapshot();
            snap.id = task.id;
            snap.source = task.source;
            snap.saveDir = task.saveDir;
            snap.name = task.name;
            snap.status = 1;

            TorrentHandle handle = handleFor(task);
            if (handle != null && handle.isValid()) {
                TorrentStatus st = handle.status();
                if (st.name() != null && !st.name().isEmpty()) {
                    task.name = st.name();
                    snap.name = st.name();
                }
                snap.total = st.totalWanted();
                snap.downloaded = st.totalWantedDone();
                snap.speed = st.downloadPayloadRate();
                snap.peers = st.numPeers();
                snap.seeds = st.numSeeds();
                boolean paused = !st.flags().and_(TorrentFlags.PAUSED).eq(new torrent_flags_t());
                if (paused) {
                    snap.status = 2;
                } else if (st.isFinished() || st.isSeeding()) {
                    snap.status = 3;
                    snap.speed = 0;
                } else {
                    snap.status = 1;
                }
            }
            out.add(snap);
        }
        return out;
    }

    public static boolean isTorrentSource(String source) {
        if (source == null) return false;
        String lower = source.toLowerCase(Locale.US);
        return lower.startsWith("magnet:") || lower.contains(".torrent");
    }

    private TorrentHandle handleFor(int id) {
        Task task = tasks.get(id);
        return task == null ? null : handleFor(task);
    }

    private TorrentHandle handleFor(Task task) {
        if (task.hash == null) return null;
        return session.find(task.hash);
    }

    private static Sha1Hash hashFromMagnet(String magnet) {
        String lower = magnet.toLowerCase(Locale.US);
        int idx = lower.indexOf("btih:");
        if (idx < 0) return null;
        String token = magnet.substring(idx + 5);
        int amp = token.indexOf('&');
        if (amp >= 0) token = token.substring(0, amp);
        if (token.length() == 40) return new Sha1Hash(token);
        return null;
    }

    private static String nameFromSource(String source) {
        if (source == null) return "torrent";
        if (source.toLowerCase(Locale.US).startsWith("magnet:")) return "magnet_download";
        int slash = source.lastIndexOf('/');
        String name = slash >= 0 ? source.substring(slash + 1) : source;
        if (name.toLowerCase(Locale.US).endsWith(".torrent")) {
            name = name.substring(0, name.length() - 8);
        }
        return name.isEmpty() ? "torrent" : name;
    }
}
