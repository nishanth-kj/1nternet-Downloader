package com.nishanth_kj.internetdownloader;

import androidx.appcompat.app.AppCompatActivity;
import android.os.Bundle;
import android.util.Log;

public class MainActivity extends AppCompatActivity {

    // Used to load the 'internetdownloader' library on application startup.
    static {
        System.loadLibrary("internetdownloader");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        
        // Test native method call
        Log.d("MainActivity", "Native string: " + stringFromJNI());
    }

    /**
     * A native method that is implemented by the 'internetdownloader' native library,
     * which is packaged with this application.
     */
    public native String stringFromJNI();
}
