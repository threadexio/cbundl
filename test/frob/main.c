#include <stdio.h>

// cbundl: bundle
#include "frob.h"

int main() {
  struct frobinator f = {0};

  for (int i = 0; i < 10; i++)
    frobinate(&f);

  printf("enough...\n");
  return 0;
}
