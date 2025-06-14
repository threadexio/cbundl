// My amazing header text!
// Another part of my awesome banner!

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
 *                cbundl 0.1.4-debug (1045a5d+)
 *             https://github.com/threadexio/cbundl
 *
 *      Generated at: Thu 01 Jan 1970 00:00:00 (UTC+00:00)
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
