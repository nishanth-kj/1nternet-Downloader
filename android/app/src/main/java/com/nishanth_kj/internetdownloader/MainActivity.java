package com.nishanth_kj.internetdownloader;

import androidx.activity.OnBackPressedCallback;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.view.GravityCompat;
import androidx.drawerlayout.widget.DrawerLayout;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;
import android.content.ClipboardManager;
import android.content.ClipData;
import android.content.Context;
import android.content.Intent;
import android.net.Uri;
import androidx.core.content.FileProvider;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.view.KeyEvent;
import android.view.LayoutInflater;
import android.view.View;
import android.view.inputmethod.EditorInfo;
import android.webkit.DownloadListener;
import android.webkit.URLUtil;
import android.webkit.WebChromeClient;
import android.webkit.WebResourceRequest;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.widget.EditText;
import android.widget.LinearLayout;
import android.widget.TextView;
import android.widget.Toast;
import com.google.android.material.chip.ChipGroup;
import com.google.android.material.dialog.MaterialAlertDialogBuilder;
import com.google.android.material.slider.Slider;
import com.google.android.material.textfield.TextInputEditText;
import org.json.JSONArray;
import org.json.JSONObject;
import java.io.File;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

public class MainActivity extends AppCompatActivity implements DownloadAdapter.OnDownloadActionListener {

    static {
        System.loadLibrary("internetdownloader");
    }

    public native String stringFromJNI();
    public native int nativeStartDownload(String url, String destination, int threads);
    public native void nativePauseDownload(int id);
    public native void nativeResumeDownload(int id);
    public native void nativeCancelDownload(int id);
    public native void nativePauseAll();
    public native void nativeResumeAll();
    public native String nativeGetDownloadProgress(int id);
    public native String nativeGetAllDownloads();

    private DrawerLayout drawerLayout;
    private RecyclerView rvDownloads;
    private DownloadAdapter adapter;
    private LinearLayout layoutEmpty;
    private LinearLayout browserPane;
    private ChipGroup chipGroup;
    private View filterScroll;
    private WebView webView;
    private EditText etBrowserUrl;
    private TextView tabDownloads;
    private TextView tabBrowser;
    private int selectedFilter = 0;
    private boolean browserVisible = false;

    private final Handler updateHandler = new Handler(Looper.getMainLooper());
    private final Runnable updateRunnable = new Runnable() {
        @Override
        public void run() {
            refreshDownloads();
            updateHandler.postDelayed(this, 500);
        }
    };

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        drawerLayout = findViewById(R.id.drawerLayout);
        rvDownloads = findViewById(R.id.rvDownloads);
        layoutEmpty = findViewById(R.id.layoutEmpty);
        browserPane = findViewById(R.id.browserPane);
        chipGroup = findViewById(R.id.chipGroup);
        filterScroll = findViewById(R.id.filterScroll);
        webView = findViewById(R.id.webView);
        etBrowserUrl = findViewById(R.id.etBrowserUrl);
        tabDownloads = findViewById(R.id.tabDownloads);
        tabBrowser = findViewById(R.id.tabBrowser);

        adapter = new DownloadAdapter(this);
        rvDownloads.setLayoutManager(new LinearLayoutManager(this));
        rvDownloads.setAdapter(adapter);

        findViewById(R.id.btnToolbarAdd).setOnClickListener(v -> showAddDownloadDialog());
        findViewById(R.id.btnOpenMenu).setOnClickListener(v -> drawerLayout.openDrawer(GravityCompat.END));

        findViewById(R.id.menuAddUrl).setOnClickListener(v -> { closeMenu(); showAddDownloadDialog(); });
        findViewById(R.id.menuAddTorrent).setOnClickListener(v -> { closeMenu(); showAddDownloadDialog(); });
        findViewById(R.id.menuPause).setOnClickListener(v -> { closeMenu(); actOnSelected(true, false); });
        findViewById(R.id.menuResume).setOnClickListener(v -> { closeMenu(); actOnSelected(false, true); });
        findViewById(R.id.menuRestart).setOnClickListener(v -> { closeMenu(); restartSelected(); });
        findViewById(R.id.menuDelete).setOnClickListener(v -> { closeMenu(); deleteSelected(); });
        findViewById(R.id.menuOpenFile).setOnClickListener(v -> { closeMenu(); openSelected(); });
        findViewById(R.id.menuOpenFolder).setOnClickListener(v -> { closeMenu(); openDownloadsFolder(); });
        findViewById(R.id.menuCopyUrl).setOnClickListener(v -> { closeMenu(); copySelectedUrl(); });
        findViewById(R.id.menuViewSegments).setOnClickListener(v -> { closeMenu(); viewSelectedSegments(); });
        findViewById(R.id.menuPauseAll).setOnClickListener(v -> { closeMenu(); nativePauseAll(); refreshDownloads(); });
        findViewById(R.id.menuResumeAll).setOnClickListener(v -> { closeMenu(); nativeResumeAll(); refreshDownloads(); });
        findViewById(R.id.menuClearCompleted).setOnClickListener(v -> { closeMenu(); clearCompleted(); });
        findViewById(R.id.menuAbout).setOnClickListener(v -> { closeMenu(); showAbout(); });

        tabDownloads.setOnClickListener(v -> showDownloadsTab());
        tabBrowser.setOnClickListener(v -> showBrowserTab());

        findViewById(R.id.btnBrowserBack).setOnClickListener(v -> {
            if (webView.canGoBack()) webView.goBack();
        });
        findViewById(R.id.btnBrowserGo).setOnClickListener(v -> goBrowser());
        etBrowserUrl.setOnEditorActionListener((v, actionId, event) -> {
            if (actionId == EditorInfo.IME_ACTION_GO || (event != null && event.getKeyCode() == KeyEvent.KEYCODE_ENTER)) {
                goBrowser();
                return true;
            }
            return false;
        });

        setupWebView();

        chipGroup.setOnCheckedStateChangeListener((group, checkedIds) -> {
            if (!checkedIds.isEmpty()) {
                int id = checkedIds.get(0);
                if (id == R.id.chipAll) selectedFilter = 0;
                else if (id == R.id.chipActive) selectedFilter = 1;
                else if (id == R.id.chipCompleted) selectedFilter = 2;
                else if (id == R.id.chipTorrents) selectedFilter = 3;
                else if (id == R.id.chipVideos) selectedFilter = 4;
                else if (id == R.id.chipCompressed) selectedFilter = 5;
                else if (id == R.id.chipPrograms) selectedFilter = 6;
                else if (id == R.id.chipDocuments) selectedFilter = 7;
                refreshDownloads();
            }
        });

        getOnBackPressedDispatcher().addCallback(this, new OnBackPressedCallback(true) {
            @Override
            public void handleOnBackPressed() {
                if (drawerLayout.isDrawerOpen(GravityCompat.END)) {
                    drawerLayout.closeDrawer(GravityCompat.END);
                } else if (browserVisible && webView.canGoBack()) {
                    webView.goBack();
                } else if (browserVisible) {
                    showDownloadsTab();
                } else {
                    setEnabled(false);
                    getOnBackPressedDispatcher().onBackPressed();
                }
            }
        });

        handleIncomingIntent(getIntent());
    }

    private void closeMenu() {
        drawerLayout.closeDrawer(GravityCompat.END);
    }

    private void showDownloadsTab() {
        browserVisible = false;
        browserPane.setVisibility(View.GONE);
        rvDownloads.setVisibility(View.VISIBLE);
        filterScroll.setVisibility(View.VISIBLE);
        tabDownloads.setBackgroundColor(getColor(R.color.classic_blue));
        tabDownloads.setTextColor(getColor(R.color.classic_white));
        tabBrowser.setBackgroundColor(getColor(R.color.classic_chrome));
        tabBrowser.setTextColor(getColor(R.color.classic_text));
        refreshDownloads();
    }

    private void showBrowserTab() {
        browserVisible = true;
        browserPane.setVisibility(View.VISIBLE);
        rvDownloads.setVisibility(View.GONE);
        layoutEmpty.setVisibility(View.GONE);
        filterScroll.setVisibility(View.GONE);
        tabBrowser.setBackgroundColor(getColor(R.color.classic_blue));
        tabBrowser.setTextColor(getColor(R.color.classic_white));
        tabDownloads.setBackgroundColor(getColor(R.color.classic_chrome));
        tabDownloads.setTextColor(getColor(R.color.classic_text));
        if (webView.getUrl() == null) {
            webView.loadUrl("https://");
        }
    }

    private void setupWebView() {
        webView.getSettings().setJavaScriptEnabled(true);
        webView.getSettings().setDomStorageEnabled(true);
        webView.setWebChromeClient(new WebChromeClient());
        webView.setWebViewClient(new WebViewClient() {
            @Override
            public boolean shouldOverrideUrlLoading(WebView view, WebResourceRequest request) {
                Uri uri = request.getUrl();
                String url = uri.toString();
                if (shouldGrabUrl(url)) {
                    startDownloadFromUrl(url);
                    return true;
                }
                return false;
            }

            @Override
            public void onPageFinished(WebView view, String url) {
                if (url != null) etBrowserUrl.setText(url);
            }
        });
        webView.setDownloadListener(new DownloadListener() {
            @Override
            public void onDownloadStart(String url, String userAgent, String contentDisposition, String mimeType, long contentLength) {
                startDownloadFromUrl(url);
            }
        });
    }

    private boolean shouldGrabUrl(String url) {
        if (url == null) return false;
        String lower = url.toLowerCase(Locale.US);
        if (lower.startsWith("magnet:") || lower.startsWith("idr:")) return true;
        return lower.contains(".torrent")
                || lower.endsWith(".zip") || lower.endsWith(".rar") || lower.endsWith(".7z")
                || lower.endsWith(".mp4") || lower.endsWith(".mkv") || lower.endsWith(".iso")
                || lower.endsWith(".exe") || lower.endsWith(".msi") || lower.endsWith(".apk")
                || lower.endsWith(".pdf");
    }

    private void goBrowser() {
        String url = etBrowserUrl.getText() != null ? etBrowserUrl.getText().toString().trim() : "";
        if (url.isEmpty()) return;
        if (shouldGrabUrl(url) && !url.startsWith("http")) {
            startDownloadFromUrl(url);
            return;
        }
        if (!url.startsWith("http://") && !url.startsWith("https://") && !url.startsWith("magnet:")) {
            url = "https://" + url;
        }
        if (shouldGrabUrl(url) && (url.toLowerCase(Locale.US).startsWith("magnet:") || looksLikeDirectFile(url))) {
            startDownloadFromUrl(url);
            return;
        }
        webView.loadUrl(url);
    }

    private boolean looksLikeDirectFile(String url) {
        String path = url.toLowerCase(Locale.US);
        int q = path.indexOf('?');
        if (q >= 0) path = path.substring(0, q);
        return path.endsWith(".zip") || path.endsWith(".rar") || path.endsWith(".7z")
                || path.endsWith(".mp4") || path.endsWith(".mkv") || path.endsWith(".iso")
                || path.endsWith(".exe") || path.endsWith(".msi") || path.endsWith(".apk")
                || path.endsWith(".pdf") || path.endsWith(".torrent");
    }

    @Override
    protected void onNewIntent(Intent intent) {
        super.onNewIntent(intent);
        setIntent(intent);
        handleIncomingIntent(intent);
    }

    private void handleIncomingIntent(Intent intent) {
        if (intent == null) return;
        Uri data = intent.getData();
        if (data == null) return;

        String url = data.toString();
        String scheme = data.getScheme();
        if (scheme != null && scheme.equalsIgnoreCase("idr")) {
            String encoded = url.length() > 6 ? url.substring(6) : "";
            try {
                url = java.net.URLDecoder.decode(encoded, "UTF-8");
            } catch (Exception e) {
                url = encoded;
            }
        }
        if (url.isEmpty()) return;
        startDownloadFromUrl(url);
        Toast.makeText(this, "Download added from link", Toast.LENGTH_SHORT).show();
        refreshDownloads();
    }

    private void startDownloadFromUrl(String url) {
        File destFolder = getExternalFilesDir(null);
        String filename = URLUtil.guessFileName(url, null, null);
        if (filename == null || filename.isEmpty()) {
            filename = "download_" + System.currentTimeMillis();
        }
        String destPath = new File(destFolder, filename).getAbsolutePath();
        nativeStartDownload(url, destPath, 4);
        Toast.makeText(this, "Download added", Toast.LENGTH_SHORT).show();
        showDownloadsTab();
        refreshDownloads();
    }

    @Override
    protected void onResume() {
        super.onResume();
        updateHandler.post(updateRunnable);
    }

    @Override
    protected void onPause() {
        super.onPause();
        updateHandler.removeCallbacks(updateRunnable);
    }

    private void refreshDownloads() {
        try {
            String jsonStr = nativeGetAllDownloads();
            if (jsonStr == null || jsonStr.isEmpty()) return;

            JSONArray arr = new JSONArray(jsonStr);
            List<DownloadItem> list = new ArrayList<>();

            for (int i = 0; i < arr.length(); i++) {
                JSONObject obj = arr.getJSONObject(i);
                DownloadItem item = new DownloadItem(
                        obj.getInt("id"),
                        obj.getString("filename"),
                        obj.getString("url"),
                        obj.getString("destination"),
                        obj.getLong("downloaded"),
                        obj.getLong("total"),
                        obj.getDouble("speed"),
                        obj.getInt("status"),
                        obj.getInt("threads"),
                        obj.getBoolean("isTorrent")
                );

                String fname = item.filename == null ? "" : item.filename.toLowerCase(Locale.US);
                boolean matches;
                switch (selectedFilter) {
                    case 1: matches = (item.status == 1 || item.status == 0); break;
                    case 2: matches = (item.status == 3); break;
                    case 3: matches = item.isTorrent; break;
                    case 4: matches = fname.endsWith(".mp4") || fname.endsWith(".mkv") || fname.endsWith(".avi"); break;
                    case 5: matches = fname.endsWith(".zip") || fname.endsWith(".rar") || fname.endsWith(".7z") || fname.endsWith(".tar"); break;
                    case 6: matches = fname.endsWith(".exe") || fname.endsWith(".msi") || fname.endsWith(".apk") || fname.endsWith(".iso"); break;
                    case 7: matches = fname.endsWith(".pdf") || fname.endsWith(".doc") || fname.endsWith(".txt"); break;
                    default: matches = true; break;
                }

                if (matches) list.add(item);
            }

            adapter.updateList(list);
            if (!browserVisible) {
                layoutEmpty.setVisibility(list.isEmpty() ? View.VISIBLE : View.GONE);
            }
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    private void showAddDownloadDialog() {
        View dialogView = LayoutInflater.from(this).inflate(R.layout.dialog_add_download, null);
        TextInputEditText etUrl = dialogView.findViewById(R.id.etUrl);
        Slider sliderThreads = dialogView.findViewById(R.id.sliderThreads);

        ClipboardManager clipboard = (ClipboardManager) getSystemService(Context.CLIPBOARD_SERVICE);
        if (clipboard != null && clipboard.hasPrimaryClip()) {
            ClipData.Item clipItem = clipboard.getPrimaryClip().getItemAt(0);
            if (clipItem != null && clipItem.getText() != null) {
                String text = clipItem.getText().toString().trim();
                if (text.startsWith("http://") || text.startsWith("https://") || text.startsWith("magnet:?")) {
                    etUrl.setText(text);
                }
            }
        }

        new MaterialAlertDialogBuilder(this)
                .setView(dialogView)
                .setPositiveButton("Download", (dialog, which) -> {
                    String url = etUrl.getText() != null ? etUrl.getText().toString().trim() : "";
                    if (!url.isEmpty()) {
                        int threads = (int) sliderThreads.getValue();
                        File destFolder = getExternalFilesDir(null);
                        String destPath = new File(destFolder, "download_" + System.currentTimeMillis()).getAbsolutePath();
                        nativeStartDownload(url, destPath, threads);
                        Toast.makeText(this, "Accelerated download started (" + threads + " streams)", Toast.LENGTH_SHORT).show();
                        refreshDownloads();
                    }
                })
                .setNegativeButton("Cancel", null)
                .show();
    }

    private void actOnSelected(boolean pause, boolean resume) {
        DownloadItem item = adapter.getSelectedItem();
        if (item == null) {
            Toast.makeText(this, "Select a download first", Toast.LENGTH_SHORT).show();
            return;
        }
        if (pause) nativePauseDownload(item.id);
        if (resume) nativeResumeDownload(item.id);
        refreshDownloads();
    }

    private void restartSelected() {
        DownloadItem item = adapter.getSelectedItem();
        if (item == null) {
            Toast.makeText(this, "Select a download first", Toast.LENGTH_SHORT).show();
            return;
        }
        nativeCancelDownload(item.id);
        nativeStartDownload(item.url, item.destination, item.threads);
        refreshDownloads();
    }

    private void deleteSelected() {
        DownloadItem item = adapter.getSelectedItem();
        if (item == null) {
            Toast.makeText(this, "Select a download first", Toast.LENGTH_SHORT).show();
            return;
        }
        nativeCancelDownload(item.id);
        refreshDownloads();
    }

    private void openSelected() {
        DownloadItem item = adapter.getSelectedItem();
        if (item == null) {
            Toast.makeText(this, "Select a download first", Toast.LENGTH_SHORT).show();
            return;
        }
        onOpen(item);
    }

    private void openDownloadsFolder() {
        File destFolder = getExternalFilesDir(null);
        if (destFolder == null) return;
        Intent intent = new Intent(Intent.ACTION_VIEW);
        intent.setDataAndType(Uri.parse(destFolder.getAbsolutePath()), "resource/folder");
        try {
            startActivity(Intent.createChooser(intent, "Open downloads folder"));
        } catch (Exception e) {
            Toast.makeText(this, destFolder.getAbsolutePath(), Toast.LENGTH_LONG).show();
        }
    }

    private void copySelectedUrl() {
        DownloadItem item = adapter.getSelectedItem();
        if (item == null) {
            Toast.makeText(this, "Select a download first", Toast.LENGTH_SHORT).show();
            return;
        }
        ClipboardManager clipboard = (ClipboardManager) getSystemService(Context.CLIPBOARD_SERVICE);
        if (clipboard != null) {
            clipboard.setPrimaryClip(ClipData.newPlainText("download url", item.url));
            Toast.makeText(this, "URL copied", Toast.LENGTH_SHORT).show();
        }
    }

    private void viewSelectedSegments() {
        DownloadItem item = adapter.getSelectedItem();
        if (item == null) {
            Toast.makeText(this, "Select a download first", Toast.LENGTH_SHORT).show();
            return;
        }
        String msg = "File: " + item.filename
                + "\nConnections: " + item.threads + " parallel threads"
                + "\nSpeed: " + item.getFormattedSpeed()
                + "\nStatus: " + item.status;
        new MaterialAlertDialogBuilder(this)
                .setTitle("Segment & Connection Details")
                .setMessage(msg)
                .setPositiveButton("OK", null)
                .show();
    }

    private void clearCompleted() {
        try {
            String jsonStr = nativeGetAllDownloads();
            if (jsonStr == null || jsonStr.isEmpty()) return;
            JSONArray arr = new JSONArray(jsonStr);
            for (int i = 0; i < arr.length(); i++) {
                JSONObject obj = arr.getJSONObject(i);
                if (obj.getInt("status") == 3) {
                    nativeCancelDownload(obj.getInt("id"));
                }
            }
            refreshDownloads();
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    private void showAbout() {
        new MaterialAlertDialogBuilder(this)
                .setTitle("About Internet Downloader")
                .setMessage("Internet Downloader v1.0.0\n\nClassic simple mobile layout with the same tasks as desktop: add URL/torrent, pause, resume, browser grabber, and multi-stream downloads.")
                .setPositiveButton("OK", null)
                .show();
    }

    @Override
    public void onPause(DownloadItem item) {
        nativePauseDownload(item.id);
        refreshDownloads();
    }

    @Override
    public void onResume(DownloadItem item) {
        nativeResumeDownload(item.id);
        refreshDownloads();
    }

    @Override
    public void onCancel(DownloadItem item) {
        nativeCancelDownload(item.id);
        refreshDownloads();
    }

    @Override
    public void onOpen(DownloadItem item) {
        try {
            File file = new File(item.destination);
            if (!file.exists()) {
                Toast.makeText(this, "File not found: " + item.destination, Toast.LENGTH_LONG).show();
                return;
            }
            Uri uri = FileProvider.getUriForFile(this, getPackageName() + ".fileprovider", file);
            Intent intent = new Intent(Intent.ACTION_VIEW);
            intent.setDataAndType(uri, "*/*");
            intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION);
            startActivity(Intent.createChooser(intent, "Open file"));
        } catch (Exception e) {
            Toast.makeText(this, item.destination, Toast.LENGTH_LONG).show();
        }
    }
}
