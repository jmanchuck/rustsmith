use crate::rng::StdRng;

use crate::generator::main_gen;

pub mod generator;
pub mod program;
pub mod rng;

/// The runtime inlined into every generated program, making programs
/// self-contained single files compilable with plain `rustc`.
///
/// CHECKSUM accumulates every observable value via `cs()` — an
/// order-sensitive mix, so swapped or wrong intermediate values change the
/// final output. FUEL bounds total loop iterations program-wide; it is
/// ordinary program semantics (single-threaded, deterministic), so a fuel
/// divergence between compiler configurations is itself a miscompilation
/// signal. Atomics are used only to get safe global mutable state.
const PRELUDE: &str = "\
#![allow(warnings)]
use std::sync::atomic::{AtomicU64, Ordering};
static CHECKSUM: AtomicU64 = AtomicU64::new(0);
static FUEL: AtomicU64 = AtomicU64::new(0);
fn cs(v: u128) {
    let prev = CHECKSUM.load(Ordering::Relaxed);
    let mixed = prev
        .rotate_left(5)
        .wrapping_add((v as u64).wrapping_mul(0x9E3779B97F4A7C15))
        .wrapping_add(((v >> 64) as u64).wrapping_mul(0xC2B2AE3D27D4EB4F));
    CHECKSUM.store(mixed, Ordering::Relaxed);
}
fn fuel_exhausted() -> bool {
    FUEL.fetch_add(1, Ordering::Relaxed) > 1_000_000
}
";

pub fn generate_from_seed(seed: u64) -> String {
    let mut rng = StdRng::seed_from_u64(seed);

    let main = main_gen::gen_main(&mut rng);

    format!("{}{}", PRELUDE, main)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn same_seed_generates_same_program() {
        for i in 0..50 {
            let mut rng1 = StdRng::seed_from_u64(i);
            let mut rng2 = StdRng::seed_from_u64(i);

            let main1 = main_gen::gen_main(&mut rng1);
            let main2 = main_gen::gen_main(&mut rng2);

            assert_eq!(main1, main2);
        }
    }
}
