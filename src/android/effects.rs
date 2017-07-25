#pragma version(1)
#pragma rs java_package_name(com.cordovaplugincamerapreview)
#pragma rs_fp_relaxed
#include "rs_debug.rsh" 

rs_allocation yuv_in;
rs_allocation kmeans_in;
rs_allocation mAllocationOut;
rs_allocation mAllocationKmeans;


//
int k;
int width;
int height;
//


uint32_t offset_to_u;
uint32_t offset_to_v;

static rs_allocation raw, magnitude, blurred, direction, candidates;
static float low, high;
static const uint32_t zero = 0;

void set_blur_input(rs_allocation u8_buf) {
	raw = u8_buf;
}

void set_compute_gradient_input(rs_allocation f_buf) {
	blurred = f_buf;
}

void set_suppress_input(rs_allocation f_buf, rs_allocation i_buf) {
	magnitude = f_buf;
	direction = i_buf;
}

void set_hysteresis_input(rs_allocation i_buf) {
	candidates = i_buf;
}

void set_thresholds(float l, float h) {
	low = l;
	high = h;
}

inline static float getElementAt_uchar_to_float(rs_allocation a, uint32_t x,
		uint32_t y) {
	return rsGetElementAt_uchar(a, x, y) / 255.0f;
}

static rs_allocation histogram;

void set_histogram(rs_allocation h) {
	histogram = h;
}

uchar4 __attribute__((kernel)) addhisto(uchar in, uint32_t x, uint32_t y) {
	int px = (x - 100) / 2;
	if (px > -1 && px < 256) {
		int v = log((float) rsGetElementAt_int(histogram, (uint32_t) px)) * 30;
		int py = (400 - y);
		if (py > -1 && v > py) {
			in = 255;
		}
		if (py == -1) {
			in = 255;
		}
	}
	uchar4 out = { in, in, in, 255 };
	return out;
}

uchar4 __attribute__((kernel)) copy(uchar in) {
	uchar4 out = { in, in, in, 255 };
	return out;
}


uchar4 __attribute__((kernel)) yuv_to_rgba(uint32_t x, uint32_t y) {
    uint32_t index = y * width + x;
    uint32_t uv_index = (y >> 1) * width + (x >> 1);
    float Y = (float)rsGetElementAt_uchar(yuv_in, index);
    float U = (float)rsGetElementAt_uchar(yuv_in, uv_index + offset_to_u);
    float V = (float)rsGetElementAt_uchar(yuv_in, uv_index + offset_to_v);
    float3 f_out;
    f_out.r = Y + 1.403f * V;
    f_out.g = Y - 0.344f * U - 0.714f * V;
    f_out.b = Y + 1.770f * U;
    f_out = clamp(f_out, 0.f, 255.f);
    uchar4 out;
    out.rgb = convert_uchar3(f_out);
    out.a = 255;
    return out;	
}






uchar4 __attribute__((kernel)) blend(uchar4 in, uint32_t x, uint32_t y) {
	uchar r = rsGetElementAt_uchar(raw, x, y);
	uchar4 out = { r, r, r, 255 };
	return max(out, in);
}

float __attribute__((kernel)) blur(uint32_t x, uint32_t y) {
	float pixel = 0;

	pixel += 2 * getElementAt_uchar_to_float(raw, x - 2, y - 2);
	pixel += 4 * getElementAt_uchar_to_float(raw, x - 1, y - 2);
	pixel += 5 * getElementAt_uchar_to_float(raw, x, y - 2);
	pixel += 4 * getElementAt_uchar_to_float(raw, x + 1, y - 2);
	pixel += 2 * getElementAt_uchar_to_float(raw, x + 2, y - 2);

	pixel += 4 * getElementAt_uchar_to_float(raw, x - 2, y - 1);
	pixel += 9 * getElementAt_uchar_to_float(raw, x - 1, y - 1);
	pixel += 12 * getElementAt_uchar_to_float(raw, x, y - 1);
	pixel += 9 * getElementAt_uchar_to_float(raw, x + 1, y - 1);
	pixel += 4 * getElementAt_uchar_to_float(raw, x + 2, y - 1);

	pixel += 5 * getElementAt_uchar_to_float(raw, x - 2, y);
	pixel += 12 * getElementAt_uchar_to_float(raw, x - 1, y);
	pixel += 15 * getElementAt_uchar_to_float(raw, x, y);
	pixel += 12 * getElementAt_uchar_to_float(raw, x + 1, y);
	pixel += 5 * getElementAt_uchar_to_float(raw, x + 2, y);

	pixel += 4 * getElementAt_uchar_to_float(raw, x - 2, y + 1);
	pixel += 9 * getElementAt_uchar_to_float(raw, x - 1, y + 1);
	pixel += 12 * getElementAt_uchar_to_float(raw, x, y + 1);
	pixel += 9 * getElementAt_uchar_to_float(raw, x + 1, y + 1);
	pixel += 4 * getElementAt_uchar_to_float(raw, x + 2, y + 1);

	pixel += 2 * getElementAt_uchar_to_float(raw, x - 2, y + 2);
	pixel += 4 * getElementAt_uchar_to_float(raw, x - 1, y + 2);
	pixel += 5 * getElementAt_uchar_to_float(raw, x, y + 2);
	pixel += 4 * getElementAt_uchar_to_float(raw, x + 1, y + 2);
	pixel += 2 * getElementAt_uchar_to_float(raw, x + 2, y + 2);

	pixel /= 159;

	return pixel;
}

float __attribute__((kernel)) compute_gradient(uint32_t x, uint32_t y) {
	float gx = 0;

	gx -= rsGetElementAt_float(blurred, x - 1, y - 1);
	gx -= rsGetElementAt_float(blurred, x - 1, y) * 2;
	gx -= rsGetElementAt_float(blurred, x - 1, y + 1);
	gx += rsGetElementAt_float(blurred, x + 1, y - 1);
	gx += rsGetElementAt_float(blurred, x + 1, y) * 2;
	gx += rsGetElementAt_float(blurred, x + 1, y + 1);

	float gy = 0;

	gy += rsGetElementAt_float(blurred, x - 1, y - 1);
	gy += rsGetElementAt_float(blurred, x, y - 1) * 2;
	gy += rsGetElementAt_float(blurred, x + 1, y - 1);
	gy -= rsGetElementAt_float(blurred, x - 1, y + 1);
	gy -= rsGetElementAt_float(blurred, x, y + 1) * 2;
	gy -= rsGetElementAt_float(blurred, x + 1, y + 1);

	int d = ((int) round(atan2pi(gy, gx) * 4.0f) + 4) % 4;
	rsSetElementAt_int(direction, d, (uint32_t)x, (uint32_t)y);
	return hypot(gx, gy);
}

int __attribute__((kernel)) suppress(uint32_t x, uint32_t y) {
	int d = rsGetElementAt_int(direction, x, y);
	float g = rsGetElementAt_float(magnitude, x, y);
	if (d == 0) {
		// horizontal, check left and right
		float a = rsGetElementAt_float(magnitude, x - 1, y);
		float b = rsGetElementAt_float(magnitude, x + 1, y);
		return a < g && b < g ? 1 : 0;
	} else if (d == 2) {
		// vertical, check above and below
		float a = rsGetElementAt_float(magnitude, x, y - 1);
		float b = rsGetElementAt_float(magnitude, x, y + 1);
		return a < g && b < g ? 1 : 0;
	} else if (d == 1) {
		// NW-SE
		float a = rsGetElementAt_float(magnitude, x - 1, y - 1);
		float b = rsGetElementAt_float(magnitude, x + 1, y + 1);
		return a < g && b < g ? 1 : 0;
	} else {
		// NE-SW
		float a = rsGetElementAt_float(magnitude, x + 1, y - 1);
		float b = rsGetElementAt_float(magnitude, x - 1, y + 1);
		return a < g && b < g ? 1 : 0;
	}
}

static const int NON_EDGE = 0b000;
static const int LOW_EDGE = 0b001;
static const int MED_EDGE = 0b010;
static const int HIG_EDGE = 0b100;

inline static int getEdgeType(uint32_t x, uint32_t y) {
	int e = rsGetElementAt_int(candidates, x, y);
	float g = rsGetElementAt_float(magnitude, x, y);
	if (e == 1) {
		if (g < low)
			return LOW_EDGE;
		if (g > high)
			return HIG_EDGE;
		return MED_EDGE;
	}
	return NON_EDGE;
}

uchar4 __attribute__((kernel)) hysteresis(uint32_t x, uint32_t y) {
	uchar4 white = { 255, 255, 255, 255 };
	uchar4 red = { 255, 0, 0, 255 };
	uchar4 black = { 0, 0, 0, 255 };
	int type = getEdgeType(x, y);
	if (type) {
		if (type & LOW_EDGE)
			return black;
		if (type & HIG_EDGE)
			return white;

		// it's medium, check nearest neighbours
		type = getEdgeType(x - 1, y - 1);
		type |= getEdgeType(x, y - 1);
		type |= getEdgeType(x + 1, y - 1);
		type |= getEdgeType(x - 1, y);
		type |= getEdgeType(x + 1, y);
		type |= getEdgeType(x - 1, y + 1);
		type |= getEdgeType(x, y + 1);
		type |= getEdgeType(x + 1, y + 1);

		if (type & HIG_EDGE)
			return white;

		if (type & MED_EDGE) {
			// check further
			type = getEdgeType(x - 2, y - 2);
			type |= getEdgeType(x - 1, y - 2);
			type |= getEdgeType(x, y - 2);
			type |= getEdgeType(x + 1, y - 2);
			type |= getEdgeType(x + 2, y - 2);
			type |= getEdgeType(x - 2, y - 1);
			type |= getEdgeType(x + 2, y - 1);
			type |= getEdgeType(x - 2, y);
			type |= getEdgeType(x + 2, y);
			type |= getEdgeType(x - 2, y + 1);
			type |= getEdgeType(x + 2, y + 1);
			type |= getEdgeType(x - 2, y + 2);
			type |= getEdgeType(x - 1, y + 2);
			type |= getEdgeType(x, y + 2);
			type |= getEdgeType(x + 1, y + 2);
			type |= getEdgeType(x + 2, y + 2);

			if (type & HIG_EDGE)
				return white;
		}
	}
	return black;
}









typedef struct cluster {
	int id; 
	int pixelCount; 
	int red; 
	int green; 
	int blue; 
	int reds; 
	int greens; 
	int blues; 	
	int (*getId)();
} cluster;

int getId(cluster){
	return 1;
}

cluster Cluster() {
	struct cluster nCluster;
	nCluster.getId = &getId;
	return nCluster;
}










void kMeans(const uchar4* in, uchar4* out, uint32_t x, uint32_t y) {
   uchar4 modifiedData;  
   
   //Get item from input allocation  
   modifiedData = rsGetElementAt_uchar4(kmeans_in, x, y);  
	//rsDebug("float4: ", modifiedData);
   
   uchar addVal = 0;  
   //Increment all values by addVal  
   modifiedData.r += addVal;  
   modifiedData.g += addVal;  
   modifiedData.b += addVal;  
   //modifiedData.a += addVal;  
   
   //Place modified data in output allocation  
   rsSetElementAt_uchar4(mAllocationOut, modifiedData, x, y);  	
}




void createClusters() {
	// Here the clusters are taken with specific steps, 
	// so the result looks always same with same image. 
	// You can randomize the cluster centers, if you like. 	
	//Cluster[] result = new Cluster[k]; 
	int x = 0; 
	int y = 0; 
	int dx = width/k; 
	int dy = height/k; 
	struct cluster nCluster = Cluster();
	for (int i=0;i<k;i++) { 
		//nCluster.
		//result[i] = new Cluster(i,image.getRGB(x, y)); 
		x+=dx;
		y+=dy; 
	} 
	//return result; 	
}








































