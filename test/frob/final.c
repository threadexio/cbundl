// My amazing header text!

/**
 *
 *                        )                (    (
 *                    ( /(    (           )\ ) )\
 *                (   )\())  ))\   (     (()/(((_)
 *                 )\ ((_)\  /((_)  )\ )   ((_))_
 *               ((_)| |(_)(_))(  _(_/(   _| || |
 *              / _| | '_ \| || || ' \))/ _` || |
 *              \__| |_.__/ \_,_||_||_| \__,_||_|
 *
 *                cbundl 0.1.2-debug (c38dfd5+)
 *             https://github.com/threadexio/cbundl
 *
 *      Generated at: Thu 01 Jan 1970 00:00:00 (UTC+00:00)
 *
 *
 * Use a gun. And if that don't work...
 *                                       use more gun.
 *   - Dr. Dell Conagher
 *
 */

/**
 * bundled from "frob.h"
 */

#ifndef _FOO_H
#define _FOO_H

struct frobinator {
	int frob_count;
};

void frobinate(struct frobinator* frob);

#endif

/**
 * bundled from "main.c"
 */

#include <stdio.h>

int main() {
	struct frobinator f = {0};

	for (int i = 0; i < 10; i++) frobinate(&f);

	printf("enough...\n");
	return 0;
}

/**
 * bundled from "frob.c"
 */

#include <stdio.h>

static void update_frob_count(int* frob_count) {
	*frob_count += 1;
}

void frobinate(struct frobinator* frob) {
	printf("frobbed!\n");
	update_frob_count(&frob->frob_count);
}
