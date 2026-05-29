#define ZERO_TO_HUNDRED_COUNT 100
#define ZERO_TO_HUNDRED_EXPECTED 630665u

extern void setStats(int enable);

int main(void) {
  volatile unsigned int zero_to_hundred_data[ZERO_TO_HUNDRED_COUNT];
  volatile unsigned int expected = ZERO_TO_HUNDRED_EXPECTED;
  unsigned int sum = 0;

  zero_to_hundred_data[0] = 9564u;
  zero_to_hundred_data[1] = 1051u;
  zero_to_hundred_data[2] = 8455u;
  zero_to_hundred_data[3] = 9088u;
  zero_to_hundred_data[4] = 3100u;
  zero_to_hundred_data[5] = 5184u;
  zero_to_hundred_data[6] = 10410u;
  zero_to_hundred_data[7] = 11713u;
  zero_to_hundred_data[8] = 10019u;
  zero_to_hundred_data[9] = 5266u;
  zero_to_hundred_data[10] = 8574u;
  zero_to_hundred_data[11] = 924u;
  zero_to_hundred_data[12] = 505u;
  zero_to_hundred_data[13] = 1219u;
  zero_to_hundred_data[14] = 7500u;
  zero_to_hundred_data[15] = 4372u;
  zero_to_hundred_data[16] = 10911u;
  zero_to_hundred_data[17] = 11375u;
  zero_to_hundred_data[18] = 3393u;
  zero_to_hundred_data[19] = 2737u;
  zero_to_hundred_data[20] = 8188u;
  zero_to_hundred_data[21] = 11926u;
  zero_to_hundred_data[22] = 10729u;
  zero_to_hundred_data[23] = 7866u;
  zero_to_hundred_data[24] = 1459u;
  zero_to_hundred_data[25] = 991u;
  zero_to_hundred_data[26] = 2263u;
  zero_to_hundred_data[27] = 4380u;
  zero_to_hundred_data[28] = 582u;
  zero_to_hundred_data[29] = 2145u;
  zero_to_hundred_data[30] = 11891u;
  zero_to_hundred_data[31] = 4458u;
  zero_to_hundred_data[32] = 154u;
  zero_to_hundred_data[33] = 6975u;
  zero_to_hundred_data[34] = 11829u;
  zero_to_hundred_data[35] = 10103u;
  zero_to_hundred_data[36] = 9363u;
  zero_to_hundred_data[37] = 10717u;
  zero_to_hundred_data[38] = 12288u;
  zero_to_hundred_data[39] = 1600u;
  zero_to_hundred_data[40] = 9622u;
  zero_to_hundred_data[41] = 521u;
  zero_to_hundred_data[42] = 4783u;
  zero_to_hundred_data[43] = 10266u;
  zero_to_hundred_data[44] = 3174u;
  zero_to_hundred_data[45] = 7827u;
  zero_to_hundred_data[46] = 1912u;
  zero_to_hundred_data[47] = 4200u;
  zero_to_hundred_data[48] = 4522u;
  zero_to_hundred_data[49] = 9994u;
  zero_to_hundred_data[50] = 12304u;
  zero_to_hundred_data[51] = 2710u;
  zero_to_hundred_data[52] = 9040u;
  zero_to_hundred_data[53] = 5330u;
  zero_to_hundred_data[54] = 3984u;
  zero_to_hundred_data[55] = 4255u;
  zero_to_hundred_data[56] = 8463u;
  zero_to_hundred_data[57] = 7160u;
  zero_to_hundred_data[58] = 4888u;
  zero_to_hundred_data[59] = 5716u;
  zero_to_hundred_data[60] = 3163u;
  zero_to_hundred_data[61] = 6580u;
  zero_to_hundred_data[62] = 12251u;
  zero_to_hundred_data[63] = 8701u;
  zero_to_hundred_data[64] = 11287u;
  zero_to_hundred_data[65] = 6908u;
  zero_to_hundred_data[66] = 3878u;
  zero_to_hundred_data[67] = 5266u;
  zero_to_hundred_data[68] = 6983u;
  zero_to_hundred_data[69] = 1203u;
  zero_to_hundred_data[70] = 9468u;
  zero_to_hundred_data[71] = 4145u;
  zero_to_hundred_data[72] = 1726u;
  zero_to_hundred_data[73] = 9812u;
  zero_to_hundred_data[74] = 3366u;
  zero_to_hundred_data[75] = 2432u;
  zero_to_hundred_data[76] = 336u;
  zero_to_hundred_data[77] = 641u;
  zero_to_hundred_data[78] = 1652u;
  zero_to_hundred_data[79] = 1549u;
  zero_to_hundred_data[80] = 10818u;
  zero_to_hundred_data[81] = 9984u;
  zero_to_hundred_data[82] = 4123u;
  zero_to_hundred_data[83] = 10268u;
  zero_to_hundred_data[84] = 39u;
  zero_to_hundred_data[85] = 11140u;
  zero_to_hundred_data[86] = 8551u;
  zero_to_hundred_data[87] = 5730u;
  zero_to_hundred_data[88] = 6316u;
  zero_to_hundred_data[89] = 10988u;
  zero_to_hundred_data[90] = 8001u;
  zero_to_hundred_data[91] = 9476u;
  zero_to_hundred_data[92] = 8511u;
  zero_to_hundred_data[93] = 4819u;
  zero_to_hundred_data[94] = 1497u;
  zero_to_hundred_data[95] = 7899u;
  zero_to_hundred_data[96] = 6146u;
  zero_to_hundred_data[97] = 4867u;
  zero_to_hundred_data[98] = 12055u;
  zero_to_hundred_data[99] = 12152u;

  setStats(1);
  for (unsigned int index = 0; index < ZERO_TO_HUNDRED_COUNT; ++index) {
    sum += zero_to_hundred_data[index];
  }
  setStats(0);

  if (sum != expected) {
    return 1;
  }
  return (int)sum;
}
