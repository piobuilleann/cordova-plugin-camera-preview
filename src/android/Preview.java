package com.cordovaplugincamerapreview;

import java.io.IOException;

import android.annotation.SuppressLint;
import android.app.Activity;
import android.content.Context;
import android.graphics.SurfaceTexture;
import android.hardware.Camera;
import android.os.Bundle;
import android.view.Gravity;
import android.view.Menu;
import android.view.TextureView;
import android.view.TextureView.SurfaceTextureListener;
import android.view.View;
import android.widget.FrameLayout;
import android.widget.RelativeLayout;

class Preview extends Activity implements TextureView.SurfaceTextureListener {
  private final String TAG = "Preview";

  //CustomSurfaceView mTextureView;
  //SurfaceHolder mHolder;
  private TextureView mTextureView = null;
  SurfaceTexture surface;
  
  Preview(Context context) {
    //super(context);
	
    //mTextureView = new CustomSurfaceView(context);
	//mTextureView.setSurfaceTextureListener(context);
	//setContentView(mTextureView);	
  }
  
  
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
