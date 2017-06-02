package com.cordovaplugincamerapreview;

import android.app.Activity;
import android.content.Context;
import android.graphics.ImageFormat;
import android.graphics.Rect;
import android.graphics.YuvImage;
import android.hardware.Camera;
//import android.support.v8.renderscript.RenderScript;
import android.util.DisplayMetrics;
import android.util.Log;

//import android.view.Surface;
//import android.view.SurfaceHolder;
import android.graphics.SurfaceTexture;
import android.view.TextureView;

import android.view.View;
import android.widget.RelativeLayout;
import org.apache.cordova.LOG;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.util.List;



class Preview extends RelativeLayout implements TextureView.SurfaceTextureListener {
  private final String TAG = "Preview";

  CustomSurfaceView mTextureView;
  //SurfaceHolder mHolder;
  SurfaceTexture surface;
  
 @Override
    protected void onCreate(Bundle savedInstanceState)
    {
        super.onCreate(savedInstanceState);

        mTextureView = new TextureView(this);
        mTextureView.setSurfaceTextureListener(this);

        setContentView(mTextureView);
    }

    @Override
    public void onSurfaceTextureAvailable(SurfaceTexture surface, int width,
            int height)
    {
        Log.i("onSurfaceTextureAvailable", "onSurfaceTextureAvailable");

        mCamera = Camera.open();

        Camera.Size previewSize = mCamera.getParameters().getPreviewSize();
        mTextureView.setLayoutParams(new FrameLayout.LayoutParams(
                previewSize.width, previewSize.height, Gravity.CENTER));

        try
        {
            mCamera.setPreviewTexture(surface);
        }
        catch (IOException t)
        {
        }

        mCamera.startPreview();

    }

    @Override
    public void onSurfaceTextureSizeChanged(SurfaceTexture surface, int width,
            int height)
    {
        // Ignored, the Camera does all the work for us
    }

    @Override
    public boolean onSurfaceTextureDestroyed(SurfaceTexture surface)
    {
        Log.i("onSurfaceTextureDestroyed", "onSurfaceTextureDestroyed");
        mCamera.stopPreview();
        mCamera.release();
        return true;
    }

    @Override
    public void onSurfaceTextureUpdated(SurfaceTexture surface)
    {
        // Update your view here!
    }  
 
}
