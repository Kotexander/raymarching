package kotexander.raymarcher

import android.view.View
import com.google.androidgamesdk.GameActivity

class MainActivity : GameActivity() {
    companion object {
        init {
            System.loadLibrary("raymarcherandroid")
        }
    }
}