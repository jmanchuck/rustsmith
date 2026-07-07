#![allow(dead_code)]
// Expression depth caps directly control program size (binary expressions
// branch exponentially). The old values (12/10/10) produced multi-megabyte
// programs that took tens of seconds to compile; these keep programs in the
// tens-of-kilobytes range so fuzzing throughput stays high.
pub const MAX_EXPR_DEPTH: u32 = 6;
pub const MAX_ARITH_EXPR_DEPTH: u32 = 5;
pub const MAX_BOOL_EXPR_DEPTH: u32 = 4;
pub const MAX_STATICS: u32 = 2;
pub const MAX_STRUCTS: u32 = 2;
pub const MAX_FUNCS: u32 = 5;
pub const MAX_FUNC_PARAMS: u32 = 6;

// Statement-tree fan-out: blocks-per-conditional times statements-per-block
// compounds across nesting levels, and dominated file size before it was
// tamed (a single function reached 5000+ lines). Target program size is
// roughly 10-60 KB.
pub const MAX_STMTS_IN_BLOCK: u8 = 5;
pub const MAX_CONDITIONAL_BRANCHES: u8 = 2;
pub const MAX_CONDITIONAL_DEPTH: u32 = 2; // Only refers to conditional statements
pub const MAX_LOOP_DEPTH: u32 = 2;

// Per-loop iteration cap. Every loop gets a stopper; in addition a global
// fuel counter (see the emitted prelude) bounds total loop iterations
// program-wide, since function-calls-inside-loops multiply per-loop caps.
pub const MAX_FOR_LOOP_ITERS: u32 = 100;
