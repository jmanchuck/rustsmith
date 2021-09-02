# rustsmith

A Rust compiler fuzzer

# Installing and running rustsmith

## Prerequisites

An installation of Rust is required to compile the source code. This can be done by following the instructions [here](https://www.rust-lang.org/tools/install). This will install the latest stable version of Rust.

Python 3.8 is also required to run the script for automated program generation, compilation, running and differential testing. `numpy`, `pandas` and `json` is required to run the Python script.

## Automated Run

Currently the easiest way to run rustsmith is from the `runtest.py` script in the generated directory.

`cd generated`

The `test` command is followed by a count.
`python3 runtest.py test 500`
This generates 500 programs, and differential tests them one by one. At the end, a `results` file is generated in the same directory summarising the test results.

To delete all generated artifacts:
`python3 runtest.py clean`

## Manually Generating Programs

The programs can also be manually generated by using the `generated` crate.

`cd generated`

To generate 50 programs, starting from seed 10:
`cargo run --release -- -c 50 --s 10`

`--release` is required to generate programs faster, since cargo doesn't turn on all optimisations by default.

# Structure of the source code

The source code is split into 3 separate crates under a single workspace.

The `generated` crate contains the generated programs and is used for generating programs, compiling them, running them and differential testing. All generated artifacts are generated into this crate. It depends on the `runtime` and `smith` crate.s

The `runtime` crate contains safe arithmetic functions that the generated programs. The generated programs contain import statements that import these functions. Moving this directory may cause issues with the generated programs.

The `smith` crate contains the program generator and the code used to represent the abstract syntax tree of the program.

See the `README.md` in the `smith` crate for a breakdown of the source code.

# Useful Commands

## Generating LLVMIR, Assembly and Rust MIR

`cargo rustc --bin [BIN_NAME] -- --emit=llvm-ir,asm,mir`

Output will be in `target/debug/deps` or `target/release/deps` depending on opt level or opt profile.

####
