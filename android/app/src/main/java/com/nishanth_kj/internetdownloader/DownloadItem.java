package com.nishanth_kj.internetdownloader;

public class DownloadItem {
    public int id;
    public String filename;
    public String url;
    public String destination;
    public long downloadedBytes;
    public long totalBytes;
    public double speed;
    public int status; // 0=Queued, 1=Downloading, 2=Paused, 3=Completed, 4=Error
    public int threads;
    public boolean isTorrent;

    public DownloadItem(int id, String filename, String url, String destination, long downloadedBytes, long totalBytes, double speed, int status, int threads, boolean isTorrent) {
        this.id = id;
        this.filename = filename;
        this.url = url;
        this.destination = destination;
        this.downloadedBytes = downloadedBytes;
        this.totalBytes = totalBytes;
        this.speed = speed;
        this.status = status;
        this.threads = threads;
        this.isTorrent = isTorrent;
    }

    public int getProgressPercent() {
        if (totalBytes <= 0) return 0;
        return (int) ((downloadedBytes * 100) / totalBytes);
    }

    public String getFormattedSpeed() {
        return formatBytes((long) speed) + "/s";
    }

    public String getFormattedDownloaded() {
        return formatBytes(downloadedBytes);
    }

    public String getFormattedTotal() {
        return totalBytes > 0 ? formatBytes(totalBytes) : "Unknown";
    }

    public static String formatBytes(long bytes) {
        if (bytes <= 0) return "0 B";
        final String[] units = new String[] { "B", "KB", "MB", "GB", "TB" };
        int digitGroups = (int) (Math.log10(bytes) / Math.log10(1024));
        if (digitGroups >= units.length) digitGroups = units.length - 1;
        return String.format("%.2f %s", bytes / Math.pow(1024, digitGroups), units[digitGroups]);
    }
}
