#include <stdio.h>
#include <stdint.h>

extern int64_t ny_main(void);

int main(void) {
  int64_t v = ny_main();
  printf("Result: %lld\n", (long long)v);
  return (int)(v & 0xFF);
}

