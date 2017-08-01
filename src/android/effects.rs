#pragma version(1)
#pragma rs java_package_name(com.cordovaplugincamerapreview)
#pragma rs_fp_relaxed
#include "rs_debug.rsh" 


rs_allocation yuv_in;
rs_allocation kmeans_in;
rs_allocation lut;
rs_allocation mAllocationOut;
rs_allocation mAllocationKmeans;


//
int k;
int width;
int height;
int clusterInt;
int imageDimenstion;
bool pixelChangedCluster;
char *Clusters;
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
} cluster;






void addPixel(int i, uchar4 pixel) {
	struct cluster Clusters[i];
	Clusters[i].reds+=pixel.r;
	Clusters[i].greens+=pixel.g;
	Clusters[i].blues+=pixel.b;
	Clusters[i].pixelCount++;
	Clusters[i].red = Clusters[i].reds/Clusters[i].pixelCount;
	Clusters[i].green = Clusters[i].greens/Clusters[i].pixelCount;
	Clusters[i].blue = Clusters[i].blues/Clusters[i].pixelCount;
}

void removePixel(int i, uchar4 pixel) {
	struct cluster Clusters[i];
	Clusters[i].reds-=pixel.r;
	Clusters[i].greens-=pixel.g;
	Clusters[i].blues-=pixel.b;
	Clusters[i].pixelCount--;
	Clusters[i].red = Clusters[i].reds/Clusters[i].pixelCount;
	Clusters[i].green = Clusters[i].greens/Clusters[i].pixelCount;
	Clusters[i].blue = Clusters[i].blues/Clusters[i].pixelCount;
}


void clear(int i) {
	struct cluster Clusters[i];
	
	Clusters[i].red = 0;
	Clusters[i].green = 0;
	Clusters[i].blue = 0;
	Clusters[i].reds = 0;
	Clusters[i].greens = 0;
	Clusters[i].blues = 0;
	Clusters[i].pixelCount = 0;
}

int static getDistance(int i, uchar4 pixel) {
	struct cluster Clusters[i];

	int rx = abs(Clusters[i].red - pixel.r);
	int gx = abs(Clusters[i].green - pixel.g);
	int bx = abs(Clusters[i].blue - pixel.b);
	int d = (rx+gx+bx) / 3;
	//rsDebug("Clusters[i].red: ", Clusters[i].red);
	//rsDebug("pixel.r: ", pixel.r);
	return d;
}


int static findMinimalCluster(uchar4 pixel) {
	// min defined the max value of an int
	int min = 2147483647;
	//int clusterInt;
	
	for (int i=0;i<clusterInt;i++) { 
		struct cluster Clusters[i];
		
		uchar4 cPixel;
		cPixel.r = Clusters[i].red;
		cPixel.g = Clusters[i].green;
		cPixel.b = Clusters[i].blue;
		
		int distance = getDistance(i, cPixel);
		//rsDebug("distance: ", distance);
		if (distance<min) { 
			min  = distance;
			clusterInt = i;
		}
		
	}
	return clusterInt;
}

void addClusterInt(){
		clusterInt++;
}

void createClusters() {
	// Here the clusters are taken with specific steps, 
	// so the result looks always same with same image. 
	// You can randomize the cluster centers, if you like. 	
	int x = 0; 
	int y = 0; 
	int dx = width/k; 
	int dy = height/k; 
	clusterInt = 0;
	
	imageDimenstion = width*height;
	//int* lut[imageDimenstion];
	


	for (int i=0;i<k;i++) { 
		struct cluster Clusters[i];
		
		Clusters[i].id = i;
		Clusters[i].pixelCount = 0;
		addClusterInt();
		
		uchar4 pixel = rsGetElementAt_uchar4(kmeans_in, x, y);
		
		Clusters[i].red = pixel.r;
		Clusters[i].green = pixel.g;
		Clusters[i].blue = pixel.b;
		
		addPixel(i, pixel);

		x+=dx;
		y+=dy; 
		
		rsDebug("cluster id: ", Clusters[i].id);
		rsDebug("cluster id: ", Clusters[i].red);
	} 
	//return result; 	
}






void kMeans(const uchar4* in, uchar4* out, uint32_t x, uint32_t y) {
   uchar4 pixel;  
   uchar4* lut;  
   
   //Get item from input allocation  
   pixel = rsGetElementAt_uchar4(kmeans_in, x, y);  
   lut    = rsGetElementAt_uchar4(lut, x, y);
   
   /*uchar addVal = 0;  
   //Increment all values by addVal  
   pixel.r += addVal;  
   pixel.g += addVal;  
   pixel.b += addVal;  
   //pixel.a += addVal;  */
   
   //Place modified data in output allocation  
   
   int cInt = findMinimalCluster(pixel);
   struct cluster Clusters[cInt];
   
   //rsDebug("fuck int: ", cInt);
   //rsDebug("fuck: ", Clusters[cInt].red);
   
   int clusterId = width*y+x;
   //rsDebug("lut->red: ", lut.r);
   //rsDebug("x: ", x);
   //rsDebug("y: ", y);
   
   /*if (lut[clusterId]!=Clusters[cInt].id) { 
		//int pixelInt = width*y+x;
		
		if (lut[clusterId]!=0) {			
			rsDebug("remove pixel from cluster id: ", clusterId);
			removePixel(clusterId, pixel);
		}

		rsDebug("add pixel from cluster id: ", clusterId);
		addPixel(cInt, pixel);
		pixelChangedCluster = true;
		
		//update lut
		lut[clusterId] = Clusters[cInt].id;
   }*/
	rsSetElementAt_uchar4(mAllocationOut, pixel, x, y);  	
   
   
}

































