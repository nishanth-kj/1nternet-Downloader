package com.nishanth_kj.internetdownloader;

import android.graphics.Color;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;
import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;
import com.google.android.material.button.MaterialButton;
import com.google.android.material.progressindicator.LinearProgressIndicator;
import java.util.ArrayList;
import java.util.List;

public class DownloadAdapter extends RecyclerView.Adapter<DownloadAdapter.ViewHolder> {

    public interface OnDownloadActionListener {
        void onPause(DownloadItem item);
        void onResume(DownloadItem item);
        void onCancel(DownloadItem item);
        void onOpen(DownloadItem item);
    }

    private List<DownloadItem> items = new ArrayList<>();
    private final OnDownloadActionListener listener;
    private int selectedId = -1;

    public DownloadAdapter(OnDownloadActionListener listener) {
        this.listener = listener;
    }

    public void updateList(List<DownloadItem> newItems) {
        this.items = newItems;
        boolean stillSelected = false;
        for (DownloadItem item : items) {
            if (item.id == selectedId) {
                stillSelected = true;
                break;
            }
        }
        if (!stillSelected && !items.isEmpty()) {
            selectedId = items.get(0).id;
        } else if (items.isEmpty()) {
            selectedId = -1;
        }
        notifyDataSetChanged();
    }

    public DownloadItem getSelectedItem() {
        for (DownloadItem item : items) {
            if (item.id == selectedId) return item;
        }
        return null;
    }

    @NonNull
    @Override
    public ViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View view = LayoutInflater.from(parent.getContext()).inflate(R.layout.item_download, parent, false);
        return new ViewHolder(view);
    }

    @Override
    public void onBindViewHolder(@NonNull ViewHolder holder, int position) {
        DownloadItem item = items.get(position);

        holder.tvFilename.setText(item.filename);
        holder.tvTag.setText(item.isTorrent ? "Torrent" : item.threads + " Threads");
        holder.progressBar.setProgress(item.getProgressPercent());

        String progressText = item.getFormattedDownloaded() + " / " + item.getFormattedTotal() + " (" + item.getProgressPercent() + "%)";
        holder.tvProgress.setText(progressText);

        if (item.status == 1) {
            holder.tvSpeed.setText(item.getFormattedSpeed());
            holder.btnPauseResume.setText("Pause");
        } else if (item.status == 2) {
            holder.tvSpeed.setText("Paused");
            holder.btnPauseResume.setText("Resume");
        } else if (item.status == 3) {
            holder.tvSpeed.setText("Completed");
            holder.btnPauseResume.setText("Open");
        } else if (item.status == 4) {
            holder.tvSpeed.setText("Error");
            holder.btnPauseResume.setText("Resume");
        } else {
            holder.tvSpeed.setText("Queued");
            holder.btnPauseResume.setText("Start");
        }

        holder.itemView.setBackgroundColor(item.id == selectedId ? Color.parseColor("#DCE8F7") : Color.WHITE);

        holder.itemView.setOnClickListener(v -> {
            selectedId = item.id;
            notifyDataSetChanged();
        });

        holder.btnPauseResume.setOnClickListener(v -> {
            selectedId = item.id;
            if (item.status == 1) {
                listener.onPause(item);
            } else if (item.status == 3) {
                listener.onOpen(item);
            } else {
                listener.onResume(item);
            }
        });

        holder.btnCancel.setOnClickListener(v -> {
            selectedId = item.id;
            listener.onCancel(item);
        });
    }

    @Override
    public int getItemCount() {
        return items.size();
    }

    public static class ViewHolder extends RecyclerView.ViewHolder {
        TextView tvFilename, tvTag, tvProgress, tvSpeed;
        LinearProgressIndicator progressBar;
        MaterialButton btnPauseResume, btnCancel;

        public ViewHolder(@NonNull View itemView) {
            super(itemView);
            tvFilename = itemView.findViewById(R.id.tvFilename);
            tvTag = itemView.findViewById(R.id.tvTag);
            tvProgress = itemView.findViewById(R.id.tvProgress);
            tvSpeed = itemView.findViewById(R.id.tvSpeed);
            progressBar = itemView.findViewById(R.id.progressBar);
            btnPauseResume = itemView.findViewById(R.id.btnPauseResume);
            btnCancel = itemView.findViewById(R.id.btnCancel);
        }
    }
}
