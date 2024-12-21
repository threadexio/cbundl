#ifndef _FOO_H
#define _FOO_H

struct frobinator {
  int frob_count;
};

void frobinate(struct frobinator* frob);

#endif

// cbundl: impl=frob.c
