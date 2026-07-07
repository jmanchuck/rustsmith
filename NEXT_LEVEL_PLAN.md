# RustSmith: Post-Mortem and Next-Level Plan

This document reviews why the fuzzing campaign found no rustc bugs, and lays out a
concrete, phased plan to turn RustSmith into a fuzzer that can realistically find them.

## Where the project stands

RustSmith is a Csmith-style random program generator for Rust plus a differential-testing
harness (`generated/runtest.py`):

- `smith` generates deterministic, panic-free safe-Rust programs from a seed: integer and
  bool typed variables, structs (incl. nested), references (`&`/`&mut`) with a
  borrow-tracking scope, `if`/`else if`/`else`, bounded-ish `for` loops over ranges,
  functions with parameters, compound assignments.
- `runtime` provides `safe_add`/`safe_div`/… wrappers (checked ops that fall back to
  `self` on overflow/division-by-zero) so generated programs never panic.
- `main` serializes one "global struct" to JSON on stdout; the harness compiles each
  program at opt levels `0`, `3`, `s`, runs all three, and compares checksums.

The architecture is sound in outline — deterministic generation, UB-free programs, a
serialized observable state, differential comparison. The reasons it found nothing are
specific and fixable.

## Why it found no bugs

### 1. The harness never actually varied the optimization level (fatal)

`generated/runtest.py`, `compile()`:

```python
os.system(f"CARGO_PROFILE_RELEASE_OPT_LEVEL={opt_level}")   # sets var in a subshell that exits immediately
os.system(f"cargo build --bin {name} --release --target-dir executables/{opt_level}")  # never sees it
```

`os.system` spawns a fresh shell per call. The environment variable dies with the first
shell and the `cargo build` on the next line never sees it. All three "opt levels" were
built with the default release profile (opt-level 3). **The campaign differential-tested
three identical binaries against each other — zero differentials was guaranteed no matter
how buggy the compiler was.** The `unset` on the following line is a no-op for the same
reason.

Fix (also needed: pass the env through `subprocess.run(..., env=...)`):

```python
env = os.environ.copy()
env["CARGO_PROFILE_RELEASE_OPT_LEVEL"] = opt_level
subprocess.run(["cargo", "build", "--bin", name, "--release",
                "--target-dir", f"executables/{opt_level}"], env=env, check=False)
```

### 2. Compiler crashes were never checked (fatal for the easiest bug class)

`compile()` ignores cargo's exit status entirely. Internal compiler errors (ICEs) are by
far the most abundant and most findable class of rustc bug — grep-able as exit code 101 /
"internal compiler error" in stderr — and the harness would have silently skipped every
one. Any generator soundness bug (invalid program emitted) is likewise indistinguishable
from a compiler bug or from success.

### 3. The oracle observes almost nothing (severe sensitivity loss)

Only `main`'s global struct is serialized at exit. Computation that doesn't flow into
that one struct — most of it, including every non-`main` function whose return value
feeds a dead expression — is dead code that the optimizer deletes. A miscompilation must
land in the narrow live slice to be observable. Csmith wins by checksumming *all* global
state; RustSmith needs the equivalent (see Phase 2).

### 4. The generated language is too small and too tame

- All arithmetic is routed through opaque `safe_*`/`bit_*` runtime calls. This shields
  exactly the surface where integer bugs live: overflow checks, wrapping/saturating
  semantics, `as` casts, shifts, unary neg, const-folding of literal arithmetic. It also
  hands the optimizer tidy, uniform call-shaped IR instead of the messy operator soup
  that stresses instruction selection and mid-end passes.
- No `match`/enums, no arrays/slices/indexing (AST nodes for arrays, `Rc`, `RefCell`
  exist in `smith/src/program/expr/` but are never generated), no tuples, no generics,
  no traits, no closures, no iterators/`Vec`, no `while`, no casts, no shadowing games,
  no `Drop`, no recursion. rustc/LLVM bugs cluster around layout (niche optimization,
  enum discriminants), monomorphization, MIR inlining, pattern-match lowering, and
  iterator chains — none of which are exercised at all.

### 5. Loops mostly time out or don't run

Loop bounds are arbitrary random expressions (e.g. `0..u128-sized-value`); the
iteration-capping "loop stopper" is injected with only 20% probability
(`PROB_MAX_FOR_LOOP_ITERS = 0.2`). Most hot loops hit the 5s timeout, and `run()` treats
a timeout as *missing data* (the binary is simply excluded from comparison) rather than
as a signal — a real opt-level-dependent divergence in loop behavior would be silently
dropped.

### 6. Throughput is orders of magnitude too low

Each program costs a full `cargo build` per opt level (multiple seconds each, no
parallelism, and lock-stepped through Python `os.system`). The campaign was ~500
programs; productive Csmith-style campaigns run 10⁵–10⁷. At this throughput even a
perfect oracle rarely gets a lottery ticket.

### 7. Weak comparison metric

The "checksum" is the *sum* of the output JSON's field values — two wrongs that cancel
(a +1 here, a −1 there) compare equal. Just compare raw stdout bytes; there is no reason
to lose information.

---

## The plan

### Phase 0 — Make the harness honest (days)

Goal: a harness where, if a bug happens, we see it.

1. **Fix the opt-level bug** (above). Verify with `strings`/`objdump` or build-plan
   output that the three binaries actually differ.
2. **Check every exit code.** Classify: compile error (generator bug — log and count),
   ICE (jackpot — save program + rustc version + stderr), runtime non-zero exit or
   signal (jackpot), timeout (signal, not missing data), output mismatch (jackpot).
3. **Compare raw stdout bytes**, not summed checksums.
4. **Drop cargo from the hot path.** Emit each program as a single self-contained file
   (inline the `runtime` helpers as a module at the top — it's ~100 lines) and invoke
   `rustc` directly: `rustc -O prog.rs`, `rustc -C opt-level=0 prog.rs`, etc. This turns
   seconds per build into tenths.
5. **Parallelize** with a worker pool; record results in SQLite/JSONL (seed, rustc
   version, flags, outcome, stderr signature) instead of a flat `results` file.
6. **Always bound loops**: make the loop-stopper unconditional (or bound ranges by
   construction). Timeouts should be rare enough to investigate individually.

Exit criterion: rerun the original 500-seed campaign and confirm binaries differ per opt
level and every outcome is classified. Sanity-check the pipeline by planting a known
miscompile (e.g. build one binary from a mutated program) and confirming it's flagged.

### Phase 1 — Multiply the oracles (1–2 weeks)

Differential testing across opt levels of one stable rustc is the *hardest* setting to
find bugs in — that path is the most hardened. Add cheap, independent oracles:

- **Opt-level matrix**: `0,1,2,3,s,z`, plus `debug` vs `release`, plus
  `-C overflow-checks=on/off` (safe: generated programs never overflow),
  `lto=on/off`, `codegen-units=1/16`, `-C panic=abort/unwind`.
- **Toolchain matrix**: stable vs beta vs nightly. Same program, same flags, different
  compiler — a divergence is a *regression*, the class of report the rustc project
  values most and triages fastest.
- **Backend differential**: nightly `-Z codegen-backend=cranelift` vs LLVM. Cranelift
  is far less battle-tested; this is one of today's most productive rustc fuzzing
  seams.
- **MIR optimization levels**: nightly `-Z mir-opt-level=0..=4` (+ `-Z inline-mir`).
  MIR opts are young and have a real miscompilation history.
- **Miri as reference interpreter** on a subsample (it's ~1000× slower): Miri's result
  is the language-semantics ground truth, and disagreement with compiled output is a
  strong report either way.
- **ICE hunting for free**: every compile in the matrix doubles as an ICE probe. Track
  known ICE signatures (hash of the panic message + query stack) to dedup, and consider
  feeding interesting programs through `icemaker`-style flag permutation.

### Phase 2 — Make programs worth compiling (the core work, 4–8 weeks)

Two thrusts: *observability* and *expressivity*.

**Observability — kill the dead code.**
- Thread a mutable checksum accumulator (a `u64` hash state, or the global struct
  itself) through every function; fold every local variable into it at scope exit
  (Csmith's trick). Alternatively/additionally, hash function return values at every
  call site. Target: a miscompiled value *anywhere* perturbs final output.
- Print the accumulator, plus the global struct, at exit.

**Expressivity — grow the subset toward where rustc bugs live**, roughly in order of
(bug-yield ÷ implementation cost):

1. **Native operators**: replace `safe_*` with `wrapping_*`/`checked_*`/`saturating_*`
   mixed, plain `+`/`*` where operands are constrained by construction, `as` casts
   between all int widths (a classic miscompile source), shifts masked by bit width,
   unary `-`/`!`.
2. **Arrays and slices**: fixed-size arrays, in-bounds indexing (`idx % LEN`), slice
   patterns, `iter().sum()`. Exercises bounds-check elision — a high-value optimization
   with a bug history.
3. **Enums + `match`**: C-like and data-carrying enums, nested patterns, guards,
   or-patterns. Exercises niche layout and match lowering — top-tier rustc-specific
   surface that C fuzzers can never reach.
4. **Tuples, `Option`/`Result`** with `?`-free explicit matching.
5. **Generics + traits**: small generic functions with trait bounds, a couple of local
   traits with default methods, trait objects (`dyn`) for dynamic dispatch.
   Monomorphization and vtable layout stress.
6. **Closures and iterator chains**: `map/filter/fold` over ranges and arrays — heavy
   MIR inlining stress, very idiomatic, very unlike what C fuzzers feed LLVM.
7. **`Vec`, `Box`, recursion with bounded depth, `Drop` impls with observable side
   effects** (drop order is a rich, rustc-specific semantic surface).
8. Wire up (or delete) the existing dead `array`/`Rc`/`RefCell` AST nodes as part of
   this — `RefCell` with runtime-checked borrows is a nice panic-free interior-mutability
   story since generated code can be structured to never double-borrow.

Every feature must preserve the two invariants: *deterministic* (no addresses, no
`HashMap` iteration order, no floats at first) and *panic-free by construction*
(or panic-deterministic: a `panic!` with a fixed message is itself a valid observable).

**Engineering hygiene while in there**: the generator string-concatenates the AST
(`RawExpr::new(format!(...))` in places); keep the typed-AST discipline as the subset
grows, add golden-file tests per feature (seed → exact program text), and keep the
same-seed determinism test green.

### Phase 3 — Scale and automate (2–3 weeks, overlaps Phase 2)

- **Campaign runner**: long-running parallel driver (this is worth writing in Rust as
  part of the workspace, replacing `runtest.py`): generate → compile matrix → run →
  classify → record, with graceful resume and per-hour stats. Target ≥10k programs/hour
  on a desktop for the small subset.
- **Automatic reduction**: an interestingness script (`program still ICEs with same
  signature` / `outputs still differ between config A and B`) plugged into
  `creduce`/`cvise` (they work fine on Rust) or `treereduce-rust`. Unreduced test cases
  are unfileable; this is what turns a campaign into actual bug reports.
- **Triage and dedup**: cluster by ICE signature or by first-divergent config pair;
  auto-check against open rust-lang/rust issues (search the panic message) before
  getting excited.
- **Continuous mode**: run nightly-vs-stable differential daily against fresh nightlies
  — regressions surface within a day of landing, which is exactly when they're easiest
  to bisect (`cargo-bisect-rustc` automates the bisection).

### Phase 4 — Research-grade directions (pick one)

To be novel rather than just bigger, aim at surface no one else covers well:

- **Semantics-rich differential**: generate programs where *drop order*, *panic
  unwinding*, or *trait resolution* determine output; today's Rust fuzzers
  (RustSmith-Kotlin, rustlantis) barely touch these.
- **Borrow-checker fuzzing as its own oracle**: generate programs at the edge of NLL
  (two-phase borrows, reborrows, closures capturing by ref) and differential-test
  *acceptance* across compiler versions (`-Z polonius` vs NLL). Bugs here are
  soundness-critical.
- **Const-eval differential**: evaluate the same expression at compile time (`const`)
  and at runtime; any disagreement is a bug by definition. Cheap to add on top of the
  existing arithmetic generator and surprisingly productive historically (CTFE has its
  own interpreter — it's an oracle you get for free).
- **Coverage-guided generation**: instrument rustc (profile-guided build) and bias
  generator weights toward grammar productions that light up new compiler coverage.

### Prior art to study and position against

- **Csmith / YARPGen** — the oracle-sensitivity and generation-scheduling playbook.
- **RustSmith (ISSTA '23 tool paper, Kotlin implementation)** — same name, same idea,
  bigger subset; study what it covers to differentiate.
- **rustlantis (ETH Zurich)** — generates *custom MIR* directly and uses a purpose-built
  MIR interpreter as the oracle; it found real miscompilations in LLVM, Cranelift, and
  MIR opts. The clearest proof that backend/MIR differential (Phase 1) is where the bugs
  are; also a warning that source-level generation competes on crowded ground unless it
  exploits source-only features (traits, borrows, drops — Phase 2/4).
- **fuzz-rustc, icemaker** — ICE-hunting harnesses; adopt their signature dedup.
- **Miri, cargo-bisect-rustc, cvise/treereduce** — the triage toolchain.

## Expected outcomes, honestly

- **ICEs**: near-certain within days of Phase 1 running at scale on nightly with an
  expanded subset — ICEs in exotic-but-valid programs remain plentiful.
- **Cranelift / MIR-opt / const-eval divergences**: realistic within weeks at Phase 2–3
  scale; these components are young.
- **Stable-rustc LLVM miscompilations from safe integer code**: rare for anyone; treat
  as a lottery ticket, not the success metric. Define success as *confirmed, filed,
  minimized upstream reports*, of any class.

## Quick-wins checklist (do these first)

- [ ] Fix `CARGO_PROFILE_RELEASE_OPT_LEVEL` never reaching cargo (`runtest.py compile()`)
- [ ] Record compiler exit codes; save any ICE (program + stderr + version)
- [ ] Compare raw stdout instead of summed checksum
- [ ] Treat timeouts as findings, not missing data
- [ ] Make loop bounding unconditional
- [ ] Single-file emission + direct `rustc`, parallel workers
- [ ] Add nightly + Cranelift + `-Z mir-opt-level` configs to the matrix
- [ ] Re-run the 500-seed campaign and confirm the matrix binaries genuinely differ
