#define HISTOGRAM_INPUTS 128
#define HISTOGRAM_BINS 16

static volatile unsigned int histogram_inputs[HISTOGRAM_INPUTS];
static volatile unsigned int histogram_bins[HISTOGRAM_BINS];

static unsigned int histogram_seed(unsigned int index) {
  return (index * 17u + (index >> 1) + 3u) & 15u;
}

int main(void) {
  unsigned int checksum = 0;

  for (unsigned int i = 0; i < HISTOGRAM_BINS; ++i) {
    histogram_bins[i] = 0;
  }
  for (unsigned int i = 0; i < HISTOGRAM_INPUTS; ++i) {
    unsigned int bin = histogram_seed(i);
    histogram_inputs[i] = bin;
    histogram_bins[bin] = histogram_bins[bin] + 1u;
  }
  for (unsigned int i = 0; i < HISTOGRAM_BINS; ++i) {
    checksum += histogram_bins[i] * (i + 1u);
  }

  return checksum == 1088u ? 0 : 1;
}
