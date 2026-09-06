package com.nishanth_kj.internetdownloader;

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
    }

    private List<DownloadItem> items = new ArrayList<>();
    private final OnDownloadActionListener listener;

    public DownloadAdapter(OnDownloadActionListener listener) {
        this.listener = listener;
    }

    public void updateList(List<DownloadItem> newItems) {
        this.items = newItems;
        notifyDataSetChanged();
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

        if (item.status == 1) { // Downloading
            holder.tvSpeed.setText(item.getFormattedSpeed());
            holder.btnPauseResume.setText("Pause");
        } else if (item.status == 2) { // Paused
            holder.tvSpeed.setText("Paused");
            holder.btnPauseResume.setText("Resume");
        } else if (item.status == 3) { // Completed
            holder.tvSpeed.setText("Completed");
            holder.btnPauseResume.setText("Open");
        } else {
            holder.tvSpeed.setText("Queued");
            holder.btnPauseResume.setText("Start");
        }

        holder.btnPauseResume.setOnClickListener(v -> {
            if (item.status == 1) {
                listener.onPause(item);
            } else {
                listener.onResume(item);
            }
        });

        holder.btnCancel.setOnClickListener(v -> listener.onCancel(item));
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

