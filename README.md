# rustsmith

A Rust compiler fuzzer

# Useful Commands

## Generating LLVMIR, Assembly and Rust MIR

`$ cargo rustc --bin [BIN_NAME] -- --emit=llvm-ir,asm,mir`

Output will be in `target/debug/deps` or `target/release/deps` depending on opt level or opt profile.

####
