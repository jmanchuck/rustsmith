#![allow(dead_code)]
pub const MAX_EXPR_DEPTH: u32 = 12;
pub const MAX_ARITH_EXPR_DEPTH: u32 = 10;
pub const MAX_BOOL_EXPR_DEPTH: u32 = 10;
pub const MAX_STATICS: u32 = 2;
pub const MAX_STRUCTS: u32 = 2;
pub const MAX_FUNCS: u32 = 5;
pub const MAX_FUNC_PARAMS: u32 = 6;

pub const MAX_STMTS_IN_BLOCK: u8 = 8;
pub const MAX_CONDITIONAL_BRANCHES: u8 = 2;
pub const MAX_CONDITIONAL_DEPTH: u32 = 2; // Only refers to conditional statements
pub const MAX_LOOP_DEPTH: u32 = 1;

pub const MAX_FOR_LOOP_ITERS: u32 = 1000;
pub const PROB_MAX_FOR_LOOP_ITERS: f32 = 0.2;
