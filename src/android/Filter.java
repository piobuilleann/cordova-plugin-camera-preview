package com.cordovaplugincamerapreview;

import android.graphics.ImageFormat;
import android.graphics.SurfaceTexture;
import android.renderscript.Allocation;
import android.renderscript.Element;
import android.renderscript.RenderScript;
import android.renderscript.Script.LaunchOptions;
import android.renderscript.ScriptIntrinsicHistogram;
import android.renderscript.Type;
import android.view.TextureView;

import android.renderscript.ScriptIntrinsicYuvToRGB;

public class Filter implements TextureView.SurfaceTextureListener {
    private int mWidth;
    private int mHeight;
    private int mSize;
    private RenderScript mRS;
    private Allocation mAllocationIn;
    private Allocation mAllocationMagnitude;
    private Allocation mAllocationBlurred;
    private Allocation mAllocationDirection;
    private Allocation mAllocationEdge;
    private Allocation mAllocationTest;
    private Allocation mAllocationOut;
    private Allocation mAllocationKmeans;
    private ScriptC_effects mEffects;
    private boolean mHaveSurface;
    private SurfaceTexture mSurface;
    private LaunchOptions sc;
    private ScriptIntrinsicHistogram mHistogram;
    private ScriptIntrinsicYuvToRGB yuvToRgbIntrinsic ;
    private Allocation mAllocationHistogram;
    private int[] histo;
    private int blending = 0;

	
	//
	private ScriptIntrinsicYuvToRGB yuvToRgbScript;
    private Allocation mAllocationRgb;
	//
	
	
    public Filter(RenderScript rs) {
        mRS = rs;
        mEffects = new ScriptC_effects(mRS);
        mHistogram = ScriptIntrinsicHistogram.create(mRS, Element.U8(mRS));
		yuvToRgbScript = ScriptIntrinsicYuvToRGB.create(mRS, Element.U8_4(mRS));
    }

    private void setupSurface() {
        if (mSurface != null) {
            if (mAllocationOut != null) {
                // hidden API
                //mAllocationOut.setSurfaceTexture(mSurface);
                setSurfaceTexture(mAllocationOut, mSurface);
            }
            mHaveSurface = true;
        } else {
            mHaveSurface = false;
        }
    }

	
	/*
	 *
	 *		This is called in onPreviewFrame within CameraActivity.java
	 *		it is fired when the filter is null OR if the camera size != the filter dimentions
	 *
	*/
    public void reset(int width, int height) {
        if (mAllocationOut != null) {
            mAllocationOut.destroy();
        }

        mWidth = width;
        mHeight = height;
        mSize = width * height;

        Type.Builder tb;

		yuvToRgbIntrinsic = ScriptIntrinsicYuvToRGB.create(mRS, Element.U8_4(mRS));
		
        tb = new Type.Builder(mRS, Element.createPixel(mRS, 
              Element.DataType.UNSIGNED_8, Element.DataKind.PIXEL_YUV)).setX(mWidth).setY(mHeight).setYuvFormat(ImageFormat.NV21);
        mAllocationIn = Allocation.createTyped(mRS, tb.create(), Allocation.USAGE_SCRIPT);

       /* tb = new Type.Builder(mRS, Element.F32(mRS)).setX(mWidth).setY(mHeight);
        mAllocationBlurred = Allocation.createTyped(mRS, tb.create(), Allocation.USAGE_SCRIPT);
        mAllocationMagnitude = Allocation.createTyped(mRS, tb.create(), Allocation.USAGE_SCRIPT);

        tb = new Type.Builder(mRS, Element.I32(mRS)).setX(mWidth).setY(mHeight);
        mAllocationDirection = Allocation.createTyped(mRS, tb.create(), Allocation.USAGE_SCRIPT);
        mAllocationEdge = Allocation.createTyped(mRS, tb.create(), Allocation.USAGE_SCRIPT);

        tb = new Type.Builder(mRS, Element.I32(mRS)).setX(256);
        mAllocationHistogram = Allocation.createTyped(mRS, tb.create(), Allocation.USAGE_SCRIPT);
		*/
		
        tb = new Type.Builder(mRS, Element.RGBA_8888(mRS)).setX(mWidth).setY(mHeight);
        mAllocationKmeans = Allocation.createTyped(mRS, tb.create(), Allocation.USAGE_SCRIPT);

        tb = new Type.Builder(mRS, Element.RGBA_8888(mRS)).setX(mWidth).setY(mHeight);
        mAllocationTest = Allocation.createTyped(mRS, tb.create(), Allocation.USAGE_SCRIPT);

		tb = new Type.Builder(mRS, Element.RGBA_8888(mRS)).setX(mWidth).setY(mHeight);
        mAllocationOut = Allocation.createTyped(mRS, tb.create(), Allocation.USAGE_SCRIPT |
                Allocation.USAGE_IO_OUTPUT);

		setupSurface();
        /*

        mHistogram.setOutput(mAllocationHistogram);
        mEffects.invoke_set_histogram(mAllocationHistogram);
        mEffects.invoke_set_blur_input(mAllocationIn);
        mEffects.invoke_set_compute_gradient_input(mAllocationBlurred);
        mEffects.invoke_set_suppress_input(mAllocationMagnitude, mAllocationDirection);
        mEffects.invoke_set_hysteresis_input(mAllocationEdge);
        mEffects.invoke_set_thresholds(0.2f, 0.6f);

        histo = new int[256];
		*/
        sc = new LaunchOptions();
        sc.setX(2, mWidth - 3);
        sc.setY(2, mHeight - 3);
    }

    public int getWidth() {
        return mWidth;
    }

    public int getHeight() {
        return mHeight;
    }

    public void execute(byte[] yuv) {
        if (mHaveSurface) {
            //mAllocationIn.copy1DRangeFrom(0, mSize, yuv);
			//mAllocationOut.copyFrom(mAllocationIn);

            if (blending == 0) {
				//mEffects.forEach_copy(mAllocationIn, mAllocationOut);
				//mAllocationIn.copyFrom(yuv);
				//mAllocationOut.copyFrom(mAllocationIn);
            } else {
               /* mHistogram.forEach_Dot(mAllocationIn);
                mAllocationHistogram.copyTo(histo);
                setThresholds();
                mEffects.forEach_blur(mAllocationBlurred, sc);
                mEffects.forEach_compute_gradient(mAllocationMagnitude, sc);
                mEffects.forEach_suppress(mAllocationEdge, sc);
                mEffects.forEach_hysteresis(mAllocationOut, sc);
                if (blending == 2) {
                    mEffects.forEach_blend(mAllocationOut, mAllocationOut);
                }*/
            }
			
			//yuvToRgbScript.setInput(mAllocationIn);
			//yuvToRgbScript.forEach(mAllocationRgb);
			
			mAllocationIn.copyFrom(yuv);
			
			yuvToRgbIntrinsic.setInput(mAllocationIn);
			yuvToRgbIntrinsic.forEach(mAllocationKmeans);
			
			//mEffects.set_kmeans_in(mAllocationKmeans);
			
            mEffects.forEach_kMeans(mAllocationKmeans, mAllocationTest);
			
			mAllocationOut.copyFrom(mAllocationTest);
			
			
			/*
			mAllocationIn.copyFrom(yuv);
			mEffects.set_yuv_in(mAllocationIn);
			mEffects.set_width(mWidth);
			mEffects.set_offset_to_u(mWidth * mHeight);
			mEffects.set_offset_to_v( (mWidth * mHeight) + ( (mWidth/2) * (mHeight/2) ) );
			mEffects.forEach_yuv_to_rgba(mAllocationOut);
			*/
            //mEffects.forEach_copy(mAllocationIn, mAllocationOut);
            ioSendOutput(mAllocationOut);
        }
    }

    private static final float THRESHOLD_MULT_LOW = 0.66f * 0.00390625f;
    private static final float THRESHOLD_MULT_HIGH = 1.33f * 0.00390625f;

    private void setThresholds() {
        int median = mSize / 2;
        for (int i = 0; i < 256; ++i) {
            median -= histo[i];
            if (median < 1) {
                mEffects.invoke_set_thresholds(i * THRESHOLD_MULT_LOW, i * THRESHOLD_MULT_HIGH);
                break;
            }
        }
    }

    @Override
    public void onSurfaceTextureAvailable(SurfaceTexture surface, int width, int height) {
        mSurface = surface;
        setupSurface();
    }

    @Override
    public void onSurfaceTextureSizeChanged(SurfaceTexture surface, int width, int height) {
        mSurface = surface;
        setupSurface();
    }

    @Override
    public boolean onSurfaceTextureDestroyed(SurfaceTexture surface) {
        mSurface = null;
        setupSurface();
        return true;
    }

    @Override
    public void onSurfaceTextureUpdated(SurfaceTexture surface) {
    }

    private static void setSurfaceTexture(Allocation allocation, SurfaceTexture surface) {
        try {
            Allocation.class.getMethod("setSurfaceTexture",
                    SurfaceTexture.class).invoke(allocation, surface);
        } catch (ReflectiveOperationException e) {
            e.printStackTrace();
        }
    }

    private static void ioSendOutput(Allocation allocation) {
        try {
            Allocation.class.getMethod("ioSendOutput").invoke(allocation);
        } catch (ReflectiveOperationException e) {
            e.printStackTrace();
        }
    }

    public void toggleBlending() {
        blending = (blending + 1) % 3;
    }
}
