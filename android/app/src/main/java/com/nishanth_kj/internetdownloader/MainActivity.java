package com.nishanth_kj.internetdownloader;

import androidx.appcompat.app.AppCompatActivity;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;
import android.content.ClipboardManager;
import android.content.ClipData;
import android.content.Context;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.view.LayoutInflater;
import android.view.View;
import android.widget.LinearLayout;
import android.widget.Toast;
import com.google.android.material.chip.ChipGroup;
import com.google.android.material.dialog.MaterialAlertDialogBuilder;
import com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton;
import com.google.android.material.slider.Slider;
import com.google.android.material.textfield.TextInputEditText;
import org.json.JSONArray;
import org.json.JSONObject;
import java.io.File;
import java.util.ArrayList;
import java.util.List;

public class MainActivity extends AppCompatActivity implements DownloadAdapter.OnDownloadActionListener {

    static {
        System.loadLibrary("internetdownloader");
    }

    // Native JNI Declarations
    public native String stringFromJNI();
    public native int nativeStartDownload(String url, String destination, int threads);
    public native void nativePauseDownload(int id);
    public native void nativeResumeDownload(int id);
    public native void nativeCancelDownload(int id);
    public native void nativePauseAll();
    public native void nativeResumeAll();
    public native String nativeGetDownloadProgress(int id);
    public native String nativeGetAllDownloads();

    private RecyclerView rvDownloads;
    private DownloadAdapter adapter;
    private LinearLayout layoutEmpty;
    private ChipGroup chipGroup;
    private int selectedFilter = 0; // 0 = All, 1 = Active, 2 = Completed, 3 = Torrents

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

        rvDownloads = findViewById(R.id.rvDownloads);
        layoutEmpty = findViewById(R.id.layoutEmpty);
        chipGroup = findViewById(R.id.chipGroup);
        ExtendedFloatingActionButton fabAdd = findViewById(R.id.fabAdd);

        adapter = new DownloadAdapter(this);
        rvDownloads.setLayoutManager(new LinearLayoutManager(this));
        rvDownloads.setAdapter(adapter);

        fabAdd.setOnClickListener(v -> showAddDownloadDialog());

        chipGroup.setOnCheckedStateChangeListener((group, checkedIds) -> {
            if (!checkedIds.isEmpty()) {
                int id = checkedIds.get(0);
                if (id == R.id.chipAll) selectedFilter = 0;
                else if (id == R.id.chipActive) selectedFilter = 1;
                else if (id == R.id.chipCompleted) selectedFilter = 2;
                else if (id == R.id.chipTorrents) selectedFilter = 3;
                refreshDownloads();
            }
        });
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

                boolean matches = false;
                switch (selectedFilter) {
                    case 0: matches = true; break;
                    case 1: matches = (item.status == 1 || item.status == 0); break;
                    case 2: matches = (item.status == 3); break;
                    case 3: matches = item.isTorrent; break;
                }

                if (matches) {
                    list.add(item);
                }
            }

            adapter.updateList(list);
            layoutEmpty.setVisibility(list.isEmpty() ? View.VISIBLE : View.GONE);
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    private void showAddDownloadDialog() {
        View dialogView = LayoutInflater.from(this).inflate(R.layout.dialog_add_download, null);
        TextInputEditText etUrl = dialogView.findViewById(R.id.etUrl);
        Slider sliderThreads = dialogView.findViewById(R.id.sliderThreads);

        // Auto paste clipboard if URL
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
}
