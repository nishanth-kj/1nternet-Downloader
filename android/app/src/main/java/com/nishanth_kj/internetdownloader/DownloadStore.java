package com.nishanth_kj.internetdownloader;

import android.content.ContentValues;
import android.content.Context;
import android.database.Cursor;
import android.database.sqlite.SQLiteDatabase;
import android.database.sqlite.SQLiteOpenHelper;
import java.util.ArrayList;
import java.util.List;

/** App-private SQLite history for the Android project. Not the desktop core/downloads.db. */
public final class DownloadStore extends SQLiteOpenHelper {
    public static final class Record {
        public String url;
        public String destination;
        public String filename;
        public int threads;
        public boolean torrent;
        public int status;
    }

    public DownloadStore(Context context) {
        super(context, "downloads.db", null, 1);
    }

    @Override
    public void onCreate(SQLiteDatabase db) {
        db.execSQL("CREATE TABLE downloads ("
                + "url TEXT PRIMARY KEY NOT NULL,"
                + "destination TEXT,"
                + "filename TEXT,"
                + "threads INTEGER,"
                + "is_torrent INTEGER,"
                + "status INTEGER)");
    }

    @Override
    public void onUpgrade(SQLiteDatabase db, int oldVersion, int newVersion) {}

    public void upsert(String url, String destination, String filename, int threads, boolean torrent, int status) {
        ContentValues values = new ContentValues();
        values.put("url", url);
        values.put("destination", destination);
        values.put("filename", filename);
        values.put("threads", threads);
        values.put("is_torrent", torrent ? 1 : 0);
        values.put("status", status);
        getWritableDatabase().insertWithOnConflict("downloads", null, values, SQLiteDatabase.CONFLICT_REPLACE);
    }

    public void delete(String url) {
        getWritableDatabase().delete("downloads", "url=?", new String[]{url});
    }

    public void deleteCompleted() {
        getWritableDatabase().delete("downloads", "status=?", new String[]{"3"});
    }

    public List<Record> loadAll() {
        List<Record> records = new ArrayList<>();
        try (Cursor cursor = getReadableDatabase().query("downloads", null, null, null, null, null, null)) {
            while (cursor.moveToNext()) {
                Record record = new Record();
                record.url = cursor.getString(cursor.getColumnIndexOrThrow("url"));
                record.destination = cursor.getString(cursor.getColumnIndexOrThrow("destination"));
                record.filename = cursor.getString(cursor.getColumnIndexOrThrow("filename"));
                record.threads = cursor.getInt(cursor.getColumnIndexOrThrow("threads"));
                record.torrent = cursor.getInt(cursor.getColumnIndexOrThrow("is_torrent")) != 0;
                record.status = cursor.getInt(cursor.getColumnIndexOrThrow("status"));
                records.add(record);
            }
        }
        return records;
    }
}
