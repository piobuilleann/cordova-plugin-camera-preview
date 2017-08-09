#pragma version(1)
#pragma rs java_package_name(com.cordovaplugincamerapreview)
#pragma rs_fp_relaxed
#include "rs_debug.rsh" 


rs_allocation yuv_in;
rs_allocation kmeans_in;
//rs_allocation lut;
rs_allocation mAllocationOut;
rs_allocation mAllocationKmeans;


//
int k;
int width;
int height;
int clusterInt;
int imageDimenstion;
bool pixelChangedCluster;
//char *Clusters;



int32_t *red;
int32_t redLen;
int32_t *green;
int32_t greenLen;
int32_t *blue;
int32_t blueLen;
int32_t *reds;
int32_t redsLen;
int32_t *greens;
int32_t greensLen;
int32_t *blues;
int32_t bluesLen;
int32_t *pixelCount;
int32_t pixelCountLen;


int32_t *lut;
int32_t lutLen;
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









/*typedef struct cluster {
	int id; 
	int pixelCount; 
	int red; 
	int green; 
	int blue; 
	int reds; 
	int greens; 
	int blues;
} cluster;*/






void addPixel(int i, uchar4 pixel) {
	
	
	int r = pixel.r>>16&0x000000FF;
	int g = pixel.g>>8&0x000000FF;
	int b = pixel.b>>0&0x000000FF;
	

	reds[i] = reds[i] + r;
	greens[i] = greens[i] + g;
	blues[i] = blues[i] + b;
	pixelCount[i]++;
	red[i] = reds[i]/pixelCount[i];
	green[i] = greens[i]/pixelCount[i];
	blue[i] = blues[i]/pixelCount[i];
}

void removePixel(int i, uchar4 pixel) {
	
	int r = pixel.r>>16&0x000000FF;
	int g = pixel.g>>8&0x000000FF;
	int b = pixel.b>>0&0x000000FF;	
	
	reds[i] = reds[i] - r;
	greens[i] = greens[i] - g;
	blues[i] = blues[i] - b;
	pixelCount[i]--;
	red[i] = reds[i]/pixelCount[i];
	green[i] = greens[i]/pixelCount[i];
	blue[i] = blues[i]/pixelCount[i];
}


void clear(int i) {
	
	red[i] = 0;
	green[i] = 0;
	blue[i] = 0;
	reds[i] = 0;
	greens[i] = 0;
	blues[i] = 0;
	pixelCount[i] = 0;
}

int static getDistance(int i, uchar4 pixel) {
	
	int r = pixel.r>>16&0x000000FF;
	int g = pixel.g>>8&0x000000FF;
	int b = pixel.b>>0&0x000000FF;

	int rx = abs(red[i] - r);
	int gx = abs(green[i] - g);
	int bx = abs(blue[i] - b);
	int d = (rx+gx+bx) / 3;
	//rsDebug("schmorgishbourg: ", (rx+gx+bx));
	//rsDebug("Clusters[i].red: ", Clusters[i].red);
	//rsDebug("Distance ", d);
	return d;
}


int static findMinimalCluster(uchar4 pixel) {
	// min defined the max value of an int
	int min = 2147483647;
	int clInt;
	
	for (int i=0;i<k;i++) { 
		
		uchar4 cPixel;
		cPixel.r = red[i];
		cPixel.g = green[i];
		cPixel.b = blue[i];
		
		int distance = getDistance(i, cPixel);
		//rsDebug("distance: ", distance);
		if (distance<min) { 
			min  = distance;
			clInt = i;
		}
		
	}
	return clInt;
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
	
	for (int i=0;i<k;i++) { 
	
	
		uchar4 pixel = rsGetElementAt_uchar4(kmeans_in, x, y);
		int clusterId = width*y+x;
		
		//clear(i);		
		
		red[i] = pixel.r;
		green[i] = pixel.g;
		blue[i] = pixel.b;
		addPixel(i, pixel);
		lut[clusterId] = -1;
		addClusterInt();
		x+=dx;
		y+=dy; 
		
		
		
		/*rsDebug("int: ", i);
		rsDebug("i -> red ", red[i]);
		rsDebug("i -> green ", green[i]);
		rsDebug("i -> blue ", blue[i]);
		rsDebug("lut Id ", clusterId);
		rsDebug("clusterId ", lut[clusterId]);
		rsDebug("cluster below ", lut[clusterId-1]);*/
	} 
}



uchar4 static getRGB(int i){
	uchar4 pixel;
	
	pixel.r = red[i] / pixelCount[i];
	pixel.g = green[i] / pixelCount[i];
	pixel.b = blue[i] / pixelCount[i];
	//pixel.a = 136;
	
	return pixel;
	//return 0xff000000|pixel.r<<16|pixel.g<<8|pixel.b; 
}


void __attribute__((kernel)) kMeans(uchar4 in, uint32_t x, uint32_t y) {
   uchar4 pixel;  
   uchar4 currentLUT;  
   int32_t clusterId;
   
   //Get item from input allocation  
   pixel = rsGetElementAt_uchar4(kmeans_in, x, y);  
   clusterId = width*y+x;
   //currentLUT    = rsGetElementAt_uchar4(lut, x, y);
   
	//int lutTest = rsGetElementAt(lut, clusterId);
	int32_t lutTest = lut[clusterId];
   //rsDebug("lutTest: ", lutTest);
   
   /*uchar addVal = 0;  
   //Increment all values by addVal  
   pixel.r += addVal;  
   pixel.g += addVal;  
   pixel.b += addVal;  
   //pixel.a += addVal;  */
   
   //Place modified data in output allocation  
   
   int cInt = findMinimalCluster(pixel);
   //struct cluster Clusters[cInt];
   
   //rsDebug("fuck int: ", cInt);
   //rsDebug("fuck: ", Clusters[cInt].red);
   
   //clusterId = width*y+x;
   
   //rsDebug("Cluster [2]->red: ", red[2]);
   //rsDebug("lut->red: ", currentLUT.r);
   //rsDebug("x: ", x);
   //rsDebug("y: ", y);
   
   if (lut[clusterId]!=cInt) { 
		//int pixelInt = width*y+x;
		
		//rsDebug("cluster id from lut: ", lut[clusterId]);
		if (lut[clusterId]!=-1) {			
			//rsDebug("remove pixel from cluster id: ", clusterId);
			removePixel(cInt, pixel);
		}

		//rsDebug("add pixel from cluster id: ", clusterId);
		addPixel(cInt, pixel);
		pixelChangedCluster = true;
		
		//update lut
		lut[clusterId] = cInt;
   }
	pixel = getRGB(cInt);
	//pixel = rsGetElementAt_uchar4(kmeans_in, x, y); 
	rsSetElementAt_uchar4(mAllocationOut, pixel, x, y);  	
   
   
}

































