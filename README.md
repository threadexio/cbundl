[crates-io]: https://crates.io/crates/cbundl
[gh-actions]: https://github.com/threadexio/cbundl/actions/workflows/ci.yaml
[gh-releases]: https://github.com/threadexio/cbundl/releases
[license]: https://github.com/threadexio/cbundl/blob/master/LICENSE
[art-license]: https://github.com/threadexio/cbundl/blob/master/assets/LICENSE

[license-badge]: https://img.shields.io/github/license/threadexio/cbundl?style=for-the-badge&logo=github&label=license&labelColor=%230e202e&color=%23d6e4ef
[tests-badge]: https://img.shields.io/github/actions/workflow/status/threadexio/cbundl/ci.yaml?branch=master&style=for-the-badge&logo=artifacthub&labelColor=%230e202e
[version-badge]: https://img.shields.io/crates/v/cbundl?style=for-the-badge&logo=buffer&labelColor=%230e202e&color=%23354c5d

[cc-by-nc-4-int]: https://creativecommons.org/licenses/by-nc/4.0

<div align="center">
<img src="https://raw.githubusercontent.com/threadexio/cbundl/refs/heads/master/assets/logo.svg" width="50%" alt="logo">

# cbundl

<p>

[`webpack`](https://webpack.js.org) but for C code.

</p>

[![license-badge]][license]
[![tests-badge]][gh-actions]
[![version-badge]][crates-io]

</div>

<br>

A simple tool that makes self-contained abominations of C code called bundles. It takes many `.c` and `.h` files and ~~concatenates~~ figures out the dependencies between them. Then, using patented (not yet) dependency finding algorithms, it arranges the source code in the files in the correct order so as to produce a single `.c` file that contains everything. In other words, given a bunch of header and implementation files, this tool can produce a single `.c` file that (hopefully) compiles and works the same way as just compiling each translation unit by itself and then linking them together.

## Table of Contents

* [Usage](#usage)
  * [Directives](#directives)
    * [bundle](#bundle)
    * [impl](#impl)
  * [Workflow](#workflow)
* [Installation](#installation)
  * [cargo](#cargo)
  * [nix](#nix)
  * [manually](#manually)
* [Building](#building)
* [License](#license)

## Usage

The tool operates as a simple preprocessor. Like the C preprocessor, the tool has its own directives that instruct it how to bundle code together. Enough words. Here is an example:

Consider the following C code (don't worry about the `// cbundl` comments for now):

---

`frob.h`

```c
#ifndef _FOO_H
#define _FOO_H

struct frobinator {
  int frob_count;
};

void frobinate(struct frobinator* frob);

#endif

// cbundl: impl=frob.c
```

---

`frob.c`

```c
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
```

---

`main.c`

```c
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
```

To compile and run this project you would have to do something along the lines of:

```bash
$ cc main.c frob.c -o frob
```

Which would get you a binary. Now let's just assume that you are in a fictional universe where for whatever reason your code will be compiled with a fixed command along the lines of:

```bash
$ cc main.c -o frob
```

Well... that won't work because the linker complains that we did not give it any implementation for `frobinate()`. We cannot change the compilation command. But we _can_ change what code goes in the compiler. If we insert a pre-processing step on our code _before_ it is even sent to the compiler we could theoretically include the implementation of `frobinate()` directly in `main.c`. This way we would end up with a self-contained translation unit which the compiler (and linker) will be happy to assemble into an executable. We can use `cbundl` for exactly this.

```bash
$ cbundl main.c -o final.c
```

The above command will parse `main.c` and figure out what dependencies it has. In this example, `main.c` wants `stdio.h` and `frob.h`. Notice the comment above the `#include "frob.h"`. Comments that begin with `// cbundl:` are called "directives" and give special instructions to `cbundl`. The directive above `frob.h` tells `cbundl` that, to build the final bundle, it needs to include `frob.h`. The directive at the end of `frob.h` tells `cbundl` that the implementation for at least one of the symbols declared by `frob.h` lives in `frob.c`. This tells `cbundl` to include `frob.c` inside the resulting bundle. That's it. That's the entire tool :clap:. The file `final.c` then contains:

```c
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
 *                cbundl X.X.X-release (XXXXXXX)
 *             https://github.com/threadexio/cbundl
 *
 *      Generated at: XXX XX XXX XXX XX:XX:XX (UTC+XX:XX)
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

static void update_frob_count(int* frob_count) { *frob_count += 1; }

void frobinate(struct frobinator* frob) {
  printf("frobbed!\n");
  update_frob_count(&frob->frob_count);
}  
```

The compiler is now happy to make us our binary. :smiley: Congratulations, you now know everything about this tool. :clap: And because we were good programmer boys, girls and everything in between, `cbundl` will also pass the resulting bundle code through a code formatter of our choice (`clang-format` by default) so its nice and pretty.

<details>
<summary>Command line arguments</summary>

```
Usage: cbundl [OPTIONS] <path>

Arguments:
  <path>
          Path to the entry source file.

Options:
      --no-format
          Don't pass the resulting bundle through the formatter.

      --formatter <exe>
          Code formatter. Must format the code from stdin and write it to stdout.

          [default: clang-format]

      --deterministic
          Output a deterministic bundle.

  -o, --output <path>
          Specify where to write the resulting bundle.

          [default: -]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

</details>

### Directives

Directives are special single-line comments (`//`, not `/* */`) that give instructions to `cbundl`.

The format of directives is as follows:

```c
// cbundl: <body>
```

The directive means different things depending on what `<body>` is. At this time, only 2 different directives exist:

* `bundle`
* `impl`

#### bundle

**Format:** `// cbundl: bundle`

The bundle directive must always appear exactly above a local `#include`, without any other comments or code in between. It informs `cbundl` of a dependency relation between the current file and the `#include`d file. An intuitive way to think about it, is that the current file "wants" the `#include`d file. Any `#include`s annotated with a bundle directive will not appear in the bundle. Additionally, any `#include`s not annotated with a bundle directive will be left as-is. This allows you to create a kind of semi-bundle where even the final bundle includes local files. I can't imagine where that would be useful, but you _can_ do it.

#### impl

**Format:** `// cbundl: impl=<path>`

The `impl` directive, also called an implementation directive, informs `cbundl` that the current file is implemented by the file specified by `<path>`. This directive can appear any number of times in the file (if the implementation is split across many other files). It can also appear anywhere in the file, but convention is that `impl` directives appear only at either the start or the end of the file. Just like `#include`-ing `.c` files, using an implementation directive that points to a `.h` file is generally considered bad practice.

### Workflow

Ok that's all cool and all but how do I integrate it into my workflow? I'm glad you asked. Simple, instead of running just:

```bash
$ cc ...
```

You do:

```bash
$ cbundl main.c > bundle.c
$ cc bundle.c -o main
```

Just make the bundle with `cbundl` and compile the bundle instead of your source files. You can also write a simple `Makefile` that does this:

```make
build:
  cbundl main.c > bundle.c
  cc bundle.c -o main
```

## Installation

`cbundl` provides pre-built release binaries in [Releases][gh-releases] for all 3 major desktop platforms.

> [!NOTE]
> Those binaries are built in [Github Actions][gh-actions]. However if you still don't trust the binaries, I don't blame you. Proceed to the [Building](#building) section.

### cargo

If you happen to have `cargo` installed you can simply do:

```bash
$ cargo install cbundl
```

### nix

If you happen to have `nix` with flakes enabled, you can do:

```bash
$ nix run 'github:threadexio/cbundl/master'
# or
$ nix run 'github:threadexio/cbundl/vX.X.X'
```

> [!NOTE]
> The above will run `cbundl` from the `master` branch. You should generally use the second form to pin down exactly which version you want.

Directly without even cloning the repository. Isn't Nix great?

If you want to install `cbundl` permanently, you can add the flake to your system configuration.

### manually

`cbundl` is a standalone binary. This means you can very easily install it only for your own user. The following will download the latest linux binary from [Releases][releases] into `~/.bin`.

```bash
$ mkdir -p ~/.bin
$ curl --proto '=https' --tlsv1.2 -sSfL 'https://github.com/threadexio/cbundl/releases/latest/download/cbundl-linux' -o ~/.bin/cbundl
$ chmod +x ~/.bin/cbundl
```

You can then add `~/.bin` to `PATH` so you can use the tool like any other command.

* Temporarily

```bash
$ export PATH="$HOME/.bin:$PATH"
```

* Permanently

```bash
$ echo -e '\nexport PATH="$HOME/.bin:$PATH"\n' >> ~/.profile
$ exec bash
```

If all goes well, you should then be able to do `cbundl --version`.

You could however not do any of that and simply download it somewhere and use the full path to run it.

## Building

Ironic that a C source code pre-processing tool is not written in C, isn't it? Anyway, as `cbundl` is written in a modern language called "Rust", you don't have to fiddle with finicky `Makefile`s or an esoteric `cmake` setup to get the damn thing to build. Simply do:

```bash
$ cargo build # for the debug build
# or
$ cargo build --release # for the release build
```

Then you can run `cbundl` through `cargo` with `cargo run` or by running it directly from `target/debug/cbundl` or `target/release/cbundl`, depending on which you built.

## License

* All code and contributions in this repository are licensed under the Apache 2.0 license, a copy of which can be found [here][license].
* All artwork in this repository is licensed under [Creative Commons Attribution-NonCommercial 4.0 International][cc-by-nc-4-int]. A copy of the license can be found [here][art-license].
