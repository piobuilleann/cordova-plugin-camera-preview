package com.cordovaplugincamerapreview;

import android.app.Activity;
import android.content.pm.ActivityInfo;
import android.app.Fragment;
import android.content.Context;
import android.content.pm.PackageManager;
import android.graphics.Bitmap;
import android.graphics.Bitmap.CompressFormat;
import android.graphics.ImageFormat;
import android.graphics.Matrix;
import android.graphics.SurfaceTexture;
import android.util.Base64;
import android.graphics.BitmapFactory;
import android.graphics.Canvas;
import android.graphics.ImageFormat;
import android.graphics.PixelFormat;
import android.graphics.Matrix;
import android.graphics.Rect;
import android.graphics.YuvImage;
import android.hardware.Camera;
import android.hardware.Camera.PictureCallback;
import android.hardware.Camera.ShutterCallback;
import android.media.Image;
import android.media.Image.Plane;
import android.os.AsyncTask;
import android.os.Bundle;
import android.os.Handler;
import android.renderscript.RenderScript;
import android.util.Log;
import android.util.DisplayMetrics;
import android.view.GestureDetector;
import android.view.Gravity;
import android.view.LayoutInflater;
import android.view.MotionEvent;
import android.view.View.OnClickListener;
import android.view.TextureView;
import android.view.View;
import android.view.ViewGroup;
import android.view.ViewTreeObserver;
import android.widget.FrameLayout;
import android.widget.ImageView;
import android.widget.RelativeLayout;

import org.apache.cordova.LOG;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.lang.Exception;
import java.lang.Integer;
import java.text.SimpleDateFormat;
import java.util.Date;
import java.util.List;
import java.util.Arrays;


import android.renderscript.RenderScript;
import org.apache.cordova.CordovaInterface;
import org.apache.cordova.CordovaWebView;


public class CameraActivity extends Fragment implements TextureView.SurfaceTextureListener, Camera.PreviewCallback  {

  public interface CameraPreviewListener {
    void onPictureTaken(String originalPicture);
    void onPictureTakenError(String message);
    void onFocusSet(int pointX, int pointY);
    void onFocusSetError(String message);
  }

  private CameraPreviewListener eventListener;
  private static final String TAG = "CameraActivity";
  public FrameLayout mainLayout;
  public FrameLayout frameContainerLayout;

  private Preview mPreview;
  private TextureView mTextureView;
  private TextureView mTextureOverlay;
  private boolean canTakePicture = true;
  SurfaceTexture surface;
  private View view;
  private Camera.Parameters cameraParameters;
  private Camera mCamera;
  private int numberOfCameras;
  private int cameraCurrentlyLocked;
  private Filter filter;
  private Camera.Size previewSize;
  private int surfaceWidth;
  private int surfaceHeight;

  
  
  private static final int STATE_OFF = 0;
  private static final int STATE_PREVIEW = 1;
  private int state = STATE_OFF;
  private boolean isProcessing = false;
  
  
  // The first rear facing camera
  private int defaultCameraId;
  public String defaultCamera;
  public boolean tapToTakePicture;
  public boolean dragEnabled;

  public int width;
  public int height;
  public int x;
  public int y;

  public void setEventListener(CameraPreviewListener listener){
    eventListener = listener;
  }

  private String appResourcesPackage;
  
  
  
  private CordovaInterface cordova;
  private CordovaWebView webView;  
  





    private class ProcessPreviewDataTask extends AsyncTask<byte[], Void, Boolean> {
        @Override
        protected Boolean doInBackground(byte[]... datas) {
            byte[] data = datas[0];
			

			//YuvImage image=new YuvImage(data, ImageFormat.NV21, surfaceWidth, surfaceHeight, null);
			//byte[] newData = image.getYuvData();
			
            filter.execute(data);
            mCamera.addCallbackBuffer(data);
            return true;
        }

        @Override
        protected void onPostExecute(Boolean result) {
            isProcessing = false;
            mTextureOverlay.invalidate();
        }

	}  
  
  
	@Override
	public void onPreviewFrame(byte[] data, Camera c) {
        if (isProcessing || state != STATE_PREVIEW) {
            mCamera.addCallbackBuffer(data);
            return;
		}
        if (data == null) {
            return;
		}
		
		isProcessing = true;		
		
        if (filter == null
                || previewSize.width != filter.getWidth()
                || previewSize.height != filter.getHeight()) {

            filter.reset(previewSize.width, previewSize.height);
        }

		new ProcessPreviewDataTask().execute(data);		
	}
  
  
  
  
  private void startPreview() {
	  if( state != STATE_OFF) {
		  //Stop for a while to drain callbacks
		  mCamera.setPreviewCallbackWithBuffer(null);
		  mCamera.stopPreview();
		  state = STATE_OFF;
		  Handler h = new Handler();
          Runnable mDelayedPreview = new Runnable() {
              @Override
              public void run() {
                  startPreview();
              }
          };
          h.postDelayed(mDelayedPreview, 300);
		  return;		  
	  }
	  
	  state = STATE_PREVIEW;
	  /*
	  Matrix transform = new Matrix();
	  float widthRatio = previewSize.width / (float) surfaceWidth;
	  float heightRatio = previewSize.height / (float) surfaceHeight;
	  
	  transform.setScale(1, heightRatio / widthRatio);
	  transform.postTranslate(0,
		surfaceHeight * (1 - heightRatio / widthRatio) / 2);
		
	  //mTextureView.setTransform(transform);
	  //mTextureOverlay.setTransform(transform);
	  */
	  
	  mCamera.setPreviewCallbackWithBuffer(this);
	  int expectedBytes = previewSize.width * previewSize.height *
		ImageFormat.getBitsPerPixel(ImageFormat.NV21) / 8;
		
	  for(int i =0; i < 4; i++){
		  mCamera.addCallbackBuffer(new byte[expectedBytes]);
		  
	  }
	  
	  try {
            mCamera.setPreviewTexture(surface);
            mCamera.startPreview();				  
	  }catch (IOException t){
		  Log.i("IOException", t.getMessage());
	  }
  }
  
  
  
	 @Override
    public void onCreate(Bundle savedInstanceState)
    {
		//
		super.onCreate(savedInstanceState);

		
	}
    @Override
    public void onSurfaceTextureAvailable(SurfaceTexture s, int width, int height) {
        mCamera = Camera.open();

        previewSize = mCamera.getParameters().getPreviewSize();
        mTextureView.setLayoutParams(new RelativeLayout.LayoutParams(previewSize.width, previewSize.height));
        mTextureOverlay.setLayoutParams(new RelativeLayout.LayoutParams(previewSize.width, previewSize.height));
		
		surfaceWidth = width;
		surfaceHeight = height;
		surface = s;
		
		Camera.Parameters parameters = mCamera.getParameters();
		parameters.set("orientation", "portrait");
		//parameters.setPictureFormat(17);
		mCamera.setDisplayOrientation(90);
		mCamera.setParameters(parameters);		
		

		if(mCamera != null){
			startPreview();
		}
		


	}	
    @Override
    public void onSurfaceTextureSizeChanged(SurfaceTexture surface, int width, int height) {
        // Ignored, Camera does all the work for us
    }

    @Override
    public boolean onSurfaceTextureDestroyed(SurfaceTexture surface) {
        return true;
    }

    @Override
    public void onSurfaceTextureUpdated(SurfaceTexture surface) {
        // Invoked every time there's a new Camera preview frame
	}  
  
  
  
	private void setUpCamera() {
        if (surface != null) {
            startPreview();
		}		
	}
  
  
  

  @Override
  public View onCreateView(LayoutInflater inflater, ViewGroup container, Bundle savedInstanceState) {
    appResourcesPackage = getActivity().getPackageName();

    // Inflate the layout for this fragment
    view = inflater.inflate(getResources().getIdentifier("camera_activity", "layout", appResourcesPackage), container, false);

	
	mTextureView = (TextureView) view.findViewById(getResources().getIdentifier("preview", "id", appResourcesPackage));
	mTextureOverlay = (TextureView) view.findViewById(getResources().getIdentifier("overlay", "id", appResourcesPackage));
	
	filter = new Filter(RenderScript.create( getActivity() ));
	
	//mTextureView.setSurfaceTextureListener( this );
	mTextureOverlay.setSurfaceTextureListener( filter );

		
	Log.d("onCreateView", "create new texture view");	
	
	
	mTextureView.setOnClickListener(new OnClickListener() {
		@Override
		public void onClick(View v) {
			filter.toggleBlending();
		}
	});	
	
	
	
	
	createCameraPreview();			
  
    
    return view;
  }

  public void setRect(int x, int y, int width, int height){
    this.x = x;
    this.y = y;
    this.width = width;
    this.height = height;
  }

  private void createCameraPreview(){
    if(mPreview == null) {

	  
	  

	/*
      setDefaultCameraId();
	  


      //set box position and size
      FrameLayout.LayoutParams layoutParams = new FrameLayout.LayoutParams(width, height);
      layoutParams.setMargins(x, y, 0, 0);
      frameContainerLayout = (FrameLayout) view.findViewById(getResources().getIdentifier("frame_container", "id", appResourcesPackage));
      frameContainerLayout.setLayoutParams(layoutParams);

      //video view
      mPreview = new Preview(getActivity());
	  tView = mPreview.getView();
	  //webView.loadUrl("javascript:console.log('" + mPreview + "');");
      mainLayout = (FrameLayout) view.findViewById(getResources().getIdentifier("video_view", "id", appResourcesPackage));
      mainLayout.setLayoutParams(new RelativeLayout.LayoutParams(RelativeLayout.LayoutParams.MATCH_PARENT, RelativeLayout.LayoutParams.MATCH_PARENT));
      //mainLayout.addView( tView );
      mainLayout.setEnabled(false); 

      final GestureDetector gestureDetector = new GestureDetector(getActivity().getApplicationContext(), new TapGestureDetector());

	 getActivity().runOnUiThread(new Runnable() {
        @Override
        public void run() {
          frameContainerLayout.setClickable(true);
          frameContainerLayout.setOnTouchListener(new View.OnTouchListener() {

            private int mLastTouchX;
            private int mLastTouchY;
            private int mPosX = 0;
            private int mPosY = 0;

            @Override
            public boolean onTouch(View v, MotionEvent event) {
              FrameLayout.LayoutParams layoutParams = (FrameLayout.LayoutParams) frameContainerLayout.getLayoutParams();


              boolean isSingleTapTouch = gestureDetector.onTouchEvent(event);
              if (event.getAction() != MotionEvent.ACTION_MOVE && isSingleTapTouch) {
                if (tapToTakePicture) {
                  takePicture(0, 0, 85);
                }
                return true;
              } else {
                if (dragEnabled) {
                  int x;
                  int y;

                  switch (event.getAction()) {
                    case MotionEvent.ACTION_DOWN:
                      if(mLastTouchX == 0 || mLastTouchY == 0) {
                        mLastTouchX = (int)event.getRawX() - layoutParams.leftMargin;
                        mLastTouchY = (int)event.getRawY() - layoutParams.topMargin;
                      }
                      else{
                        mLastTouchX = (int)event.getRawX();
                        mLastTouchY = (int)event.getRawY();
                      }
                      break;
                    case MotionEvent.ACTION_MOVE:

                      x = (int) event.getRawX();
                      y = (int) event.getRawY();

                      final float dx = x - mLastTouchX;
                      final float dy = y - mLastTouchY;

                      mPosX += dx;
                      mPosY += dy;

                      layoutParams.leftMargin = mPosX;
                      layoutParams.topMargin = mPosY;

                      frameContainerLayout.setLayoutParams(layoutParams);

                      // Remember this touch position for the next move event
                      mLastTouchX = x;
                      mLastTouchY = y;

                      break;
                    default:
                      break;
                  }
                }
              }
              return true;
            }
          });
        }
      });*/
    }
  }

  private void setDefaultCameraId(){
    // Find the total number of cameras available
    numberOfCameras = Camera.getNumberOfCameras();

    int camId = defaultCamera.equals("front") ? Camera.CameraInfo.CAMERA_FACING_FRONT : Camera.CameraInfo.CAMERA_FACING_BACK;

    // Find the ID of the default camera
    Camera.CameraInfo cameraInfo = new Camera.CameraInfo();
    for (int i = 0; i < numberOfCameras; i++) {
      Camera.getCameraInfo(i, cameraInfo);
      if (cameraInfo.facing == camId) {
        defaultCameraId = camId;
        break;
      }
    }
  }

  @Override
  public void onResume() {
    super.onResume();

	setUpCamera();
	
	
   /* mCamera = Camera.open(defaultCameraId);

    if (cameraParameters != null) {
      mCamera.setParameters(cameraParameters);
    }

    cameraCurrentlyLocked = defaultCameraId;


      mCamera.startPreview();

    Log.d(TAG, "cameraCurrentlyLocked:" + cameraCurrentlyLocked);

    final FrameLayout frameContainerLayout = (FrameLayout) view.findViewById(getResources().getIdentifier("frame_container", "id", appResourcesPackage));

    ViewTreeObserver viewTreeObserver = frameContainerLayout.getViewTreeObserver();

    if (viewTreeObserver.isAlive()) {
      viewTreeObserver.addOnGlobalLayoutListener(new ViewTreeObserver.OnGlobalLayoutListener() {
        @Override
        public void onGlobalLayout() {
          frameContainerLayout.getViewTreeObserver().removeGlobalOnLayoutListener(this);
          frameContainerLayout.measure(View.MeasureSpec.UNSPECIFIED, View.MeasureSpec.UNSPECIFIED);
          final RelativeLayout frameCamContainerLayout = (RelativeLayout) view.findViewById(getResources().getIdentifier("frame_camera_cont", "id", appResourcesPackage));

          FrameLayout.LayoutParams camViewLayout = new FrameLayout.LayoutParams(frameContainerLayout.getWidth(), frameContainerLayout.getHeight());
          camViewLayout.gravity = Gravity.CENTER_HORIZONTAL | Gravity.CENTER_VERTICAL;
          frameCamContainerLayout.setLayoutParams(camViewLayout);
        }
      });
    }*/
  }

  @Override
  public void onPause() {
    super.onPause();

    // Because the Camera object is a shared resource, it's very important to release it when the activity is paused.
    if (mCamera != null) {
      setDefaultCameraId();
      //mPreview.setCamera(null, -1);
      mCamera.setPreviewCallback(null);
      mCamera.release();
      mCamera = null;
    }
  }

  public Camera getCamera() {
    return mCamera;
  }

  public void switchCamera() {
    // check for availability of multiple cameras

  }

  public void setCameraParameters(Camera.Parameters params) {
    cameraParameters = params;

    if (mCamera != null && cameraParameters != null) {
      mCamera.setParameters(cameraParameters);
    }
  }

  public boolean hasFrontCamera(){
    return getActivity().getApplicationContext().getPackageManager().hasSystemFeature(PackageManager.FEATURE_CAMERA_FRONT);
  }

  public Bitmap cropBitmap(Bitmap bitmap, Rect rect){
    int w = rect.right - rect.left;
    int h = rect.bottom - rect.top;
    Bitmap ret = Bitmap.createBitmap(w, h, bitmap.getConfig());
    Canvas canvas= new Canvas(ret);
    canvas.drawBitmap(bitmap, -rect.left, -rect.top, null);
    return ret;
  }

  public static Bitmap rotateBitmap(Bitmap source, float angle, boolean mirror) {
    Matrix matrix = new Matrix();
    if (mirror) {
      matrix.preScale(-1.0f, 1.0f);
    }
    matrix.postRotate(angle);
    return Bitmap.createBitmap(source, 0, 0, source.getWidth(), source.getHeight(), matrix, true);
  }

  ShutterCallback shutterCallback = new ShutterCallback(){
    public void onShutter(){
      // do nothing, availabilty of this callback causes default system shutter sound to work
    }
  };

  PictureCallback jpegPictureCallback = new PictureCallback(){
    public void onPictureTaken(byte[] data, Camera arg1){
      Log.d(TAG, "CameraPreview jpegPictureCallback");
      Camera.Parameters params = mCamera.getParameters();

    }
  };

  private Camera.Size getOptimalPictureSize(final int width, final int height, final Camera.Size previewSize, final List<Camera.Size> supportedSizes){
    /*
      get the supportedPictureSize that:
      - has the closest aspect ratio to the preview aspect ratio
      - has picture.width and picture.height closest to width and height
      - has the highest supported picture width and height up to 2 Megapixel if width == 0 || height == 0
    */
    Camera.Size size = mCamera.new Size(width, height);

    // convert to landscape if necessary
    if (size.width < size.height) {
      int temp = size.width;
      size.width = size.height;
      size.height = temp;
    }

    double previewAspectRatio  = (double)previewSize.width / (double)previewSize.height;

    if (previewAspectRatio < 1.0) {
      // reset ratio to landscape
      previewAspectRatio = 1.0 / previewAspectRatio;
    }

    Log.d(TAG, "CameraPreview previewAspectRatio " + previewAspectRatio);

    double aspectTolerance = 0.1;
    double bestDifference = Double.MAX_VALUE;

    for (int i = 0; i < supportedSizes.size(); i++) {
      Camera.Size supportedSize = supportedSizes.get(i);
      double difference = Math.abs(previewAspectRatio - ((double)supportedSize.width / (double)supportedSize.height));

      if (difference < bestDifference - aspectTolerance) {
        // better aspectRatio found
        if ((width != 0 && height != 0) || (supportedSize.width * supportedSize.height < 2048 * 1024)) {
          size.width = supportedSize.width;
          size.height = supportedSize.height;
          bestDifference = difference;
        }
      } else if (difference < bestDifference + aspectTolerance) {
        // same aspectRatio found (within tolerance)
        if (width == 0 || height == 0) {
          // set highest supported resolution below 2 Megapixel
          if ((size.width < supportedSize.width) && (supportedSize.width * supportedSize.height < 2048 * 1024)) {
            size.width = supportedSize.width;
            size.height = supportedSize.height;
          }
        } else {
          // check if this pictureSize closer to requested width and height
          if (Math.abs(width * height - supportedSize.width * supportedSize.height) < Math.abs(width * height - size.width * size.height)) {
            size.width = supportedSize.width;
            size.height = supportedSize.height;
          }
        }
      }
    }
    Log.d(TAG, "CameraPreview optimalPictureSize " + size.width + 'x' + size.height);
    return size;
  }

  public void takePicture(final int width, final int height, final int quality){
    Log.d(TAG, "CameraPreview takePicture width: " + width + ", height: " + height + ", quality: " + quality);

    if(mPreview != null) {
      if(!canTakePicture){
        return;
      }

      canTakePicture = false;

      new Thread() {
        public void run() {
          Camera.Parameters params = mCamera.getParameters();

          Camera.Size size = getOptimalPictureSize(width, height, params.getPreviewSize(), params.getSupportedPictureSizes());
          params.setPictureSize(size.width, size.height);
          params.setJpegQuality(quality);

          mCamera.setParameters(params);
          mCamera.takePicture(shutterCallback, null, jpegPictureCallback);
        }
      }.start();
    } else {
      canTakePicture = true;
    }
  }

  public void setFocusArea(final int pointX, final int pointY) {
    if (mCamera != null) {

      mCamera.cancelAutoFocus();

      Camera.Parameters parameters = mCamera.getParameters();

      Rect focusRect = calculateTapArea(pointX, pointY, 1f);
      parameters.setFocusMode(Camera.Parameters.FOCUS_MODE_AUTO);
      parameters.setFocusAreas(Arrays.asList(new Camera.Area(focusRect, 1000)));

      if (parameters.getMaxNumMeteringAreas() > 0) {
        Rect meteringRect = calculateTapArea(pointX, pointY, 1.5f);
        parameters.setMeteringAreas(Arrays.asList(new Camera.Area(meteringRect, 1000)));
      }

      try {
        setCameraParameters(parameters);

        mCamera.autoFocus(new Camera.AutoFocusCallback() {
          public void onAutoFocus(boolean success, Camera camera) {
            if (success) {
              eventListener.onFocusSet(pointX, pointY);
            } else {
              eventListener.onFocusSetError("Focus set failed");
            }
          }
        });
      } catch (Exception e) {
        Log.d(TAG, e.getMessage());
        eventListener.onFocusSetError("Focus set parameters failed");
      }
    }
  }

  private Rect calculateTapArea(float x, float y, float coefficient) {
    return new Rect(
      Math.round((x - 100) * 2000 / width  - 1000),
      Math.round((y - 100) * 2000 / height - 1000),
      Math.round((x + 100) * 2000 / width  - 1000),
      Math.round((y + 100) * 2000 / height - 1000)
    );
  }
}
