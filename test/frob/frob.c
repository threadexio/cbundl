// cbundl: bundle
#include "frob.h"

#include <stdio.h>

static void update_frob_count(int *frob_count) {
  *frob_count += 1;
}

void frobinate(struct frobinator* frob) {
  printf("frobbed!\n");
  update_frob_count(&frob->frob_count);
}
