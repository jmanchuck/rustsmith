#!/usr/bin/env python3
"""Parallel differential-testing driver for the rustsmith generator.

Generates deterministic Rust programs with the generator binary, compiles each
one under a matrix of (toolchain x config) rustc invocations, runs every
resulting binary, and compares (exit code, sha256(stdout)) across the matrix.
Any disagreement, ICE, compile error, crash, or timeout is recorded as a
finding in a sqlite database with artifacts on disk.

Usage examples:

    # Fuzz 1000 seeds with all built-in configs on the default toolchain:
    python3 fuzz.py run --count 1000

    # A subset of configs, custom seed range and parallelism:
    python3 fuzz.py run --count 200 --start 5000 --jobs 4 --configs O0,O2,O3

    # Override the generator and rustc commands (e.g. for testing):
    python3 fuzz.py run --count 10 --generator "python3 fake_gen.py" --rustc rustc

    # Differential-test two toolchains:
    python3 fuzz.py run --count 100 \
        --toolchains "rustc,rustup run nightly rustc"

    # Summarize all campaigns recorded so far:
    python3 fuzz.py report

    # Delete the work dir (results.db, findings, everything):
    python3 fuzz.py clean

Layout of the work dir (default fuzzout/):
    results.db                    sqlite database (campaigns/results/findings)
    tmp/seed_<N>/                 per-seed scratch, removed after each seed
    findings/<class>/seed_<N>/    retained artifacts for findings
    programs/seed_<N>.rs          generated programs (only with --keep-programs)
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import re
import shlex
import shutil
import signal
import sqlite3
import subprocess
import sys
import time
from collections import Counter
from concurrent.futures import ProcessPoolExecutor, as_completed
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional

# Built-in compile config matrix: name -> extra rustc flags.
CONFIGS: dict[str, str] = {
    "O0": "-C opt-level=0",
    "O1": "-C opt-level=1",
    "O2": "-C opt-level=2",
    "O3": "-C opt-level=3",
    "Os": "-C opt-level=s",
    "Oz": "-C opt-level=z",
    "O3-cu1": "-C opt-level=3 -C codegen-units=1",
    "O2-ovf": "-C opt-level=2 -C overflow-checks=on",
}

DEFAULT_WORK_DIR = "fuzzout"
DEFAULT_GENERATOR = "target/release/generated"

# Preference order for choosing which findings/<class>/ dir gets the artifacts
# when a single seed produced findings of several classes.
CLASS_PRIORITY = ["divergence", "ice", "run_crash", "run_timeout", "compile_error"]

# Keep artifacts for at most this many seeds per ICE signature.
ICE_ARTIFACT_QUOTA = 3


# ---------------------------------------------------------------------------
# Small subprocess helpers
# ---------------------------------------------------------------------------

@dataclass
class Proc:
    """Result of a finished (non-timed-out) subprocess."""
    returncode: int
    stdout: bytes
    stderr: bytes


def run_cmd(cmd: list[str], timeout: float) -> Optional[Proc]:
    """Run a command, capturing output. Returns None on timeout."""
    try:
        p = subprocess.run(cmd, capture_output=True, timeout=timeout)
        return Proc(p.returncode, p.stdout, p.stderr)
    except subprocess.TimeoutExpired:
        return None
    except OSError as e:
        return Proc(127, b"", str(e).encode())


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


# ---------------------------------------------------------------------------
# Classification helpers
# ---------------------------------------------------------------------------

ICE_MARKERS = ("internal compiler error", "the compiler unexpectedly panicked")


def is_ice(returncode: int, stderr: str) -> bool:
    return returncode == 101 or any(m in stderr for m in ICE_MARKERS)


def ice_signature(stderr: str) -> str:
    """Normalized panic line: strip paths, line:col numbers, hashes."""
    line = ""
    fallback = ""
    for cand in stderr.splitlines():
        if "internal compiler error" in cand:
            line = cand
            break
        if not fallback and ("panicked at" in cand or "unexpectedly panicked" in cand):
            fallback = cand
    if not line:
        line = fallback
    if not line:
        line = next((l for l in stderr.splitlines() if l.strip()), "unknown ICE")
    # Anything path-like (contains a slash) -> <path>
    line = re.sub(r"[\w~.+-]*[/\\][\w./\\+-]+", "<path>", line)
    # Long hex strings (hashes) -> <hash>
    line = re.sub(r"\b[0-9a-f]{8,}\b", "<hash>", line)
    # Remaining numbers (line:col etc.) -> <n>
    line = re.sub(r"\d+", "<n>", line)
    return line.strip()[:400]


def compile_error_signature(stderr: str) -> str:
    """First error line of a compile failure, numbers normalized."""
    for l in stderr.splitlines():
        if l.startswith("error"):
            return re.sub(r"\d+", "<n>", l).strip()[:400]
    first = next((l for l in stderr.splitlines() if l.strip()), "compile_error")
    return re.sub(r"\d+", "<n>", first).strip()[:400]


def divergence_signature(entries: list[dict[str, Any]]) -> str:
    parts = sorted({
        "timeout" if e["run_timed_out"]
        else f"exit={e['exit_code']},out={(e['output_sha'] or '')[:12]}"
        for e in entries
    })
    return " | ".join(parts)[:400]


# ---------------------------------------------------------------------------
# Per-seed worker (runs in a subprocess; returns plain data only)
# ---------------------------------------------------------------------------

@dataclass
class SeedTask:
    seed: int
    generator_cmd: str
    toolchains: list[str]            # full rustc command strings
    configs: list[tuple[str, str]]   # (name, flags)
    tmp_root: str                    # absolute path
    generate_timeout: float
    compile_timeout: float
    run_timeout: float


def _worker_init() -> None:
    # Let the main process own ctrl-C handling.
    signal.signal(signal.SIGINT, signal.SIG_IGN)


def run_seed(task: SeedTask) -> dict[str, Any]:
    """Generate, compile across the matrix, run, and classify one seed.

    Returns a plain dict; all sqlite writes happen in the main process.
    """
    seed = task.seed
    seed_dir = Path(task.tmp_root) / f"seed_{seed}"
    shutil.rmtree(seed_dir, ignore_errors=True)
    seed_dir.mkdir(parents=True)

    out: dict[str, Any] = {
        "seed": seed,
        "seed_dir": str(seed_dir),
        "results": [],
        "findings": [],
        "binary_shas": [],
        "notes": "",
    }

    def result_row(toolchain: str, config: str, status: str,
                   compile_ms: Optional[int] = None, run_ms: Optional[int] = None,
                   exit_code: Optional[int] = None, output_sha: Optional[str] = None) -> dict[str, Any]:
        return {"toolchain": toolchain, "config": config, "status": status,
                "compile_ms": compile_ms, "run_ms": run_ms,
                "exit_code": exit_code, "output_sha": output_sha}

    # 1. Generate.
    src = seed_dir / f"seed_{seed}.rs"
    gen_cmd = shlex.split(task.generator_cmd) + [
        "--seed", str(seed), "--count", "1", "--out", str(seed_dir)]
    gp = run_cmd(gen_cmd, task.generate_timeout)
    if gp is None or gp.returncode != 0 or not src.exists():
        for tc in task.toolchains:
            for cfg, _ in task.configs:
                out["results"].append(result_row(tc, cfg, "generator_error"))
        if gp is None:
            out["notes"] = "generator timeout"
        elif gp.returncode != 0:
            out["notes"] = f"generator exit {gp.returncode}: " + gp.stderr.decode("utf-8", "replace")[:300]
        else:
            out["notes"] = f"generator exited 0 but {src.name} missing"
        return out

    # 2. Compile once per matrix entry.
    multi_tc = len(task.toolchains) > 1
    entries: list[dict[str, Any]] = []
    for ti, tc in enumerate(task.toolchains):
        for cfg, flags in task.configs:
            label = f"t{ti}_{cfg}" if multi_tc else cfg
            binpath = seed_dir / (f"seed_{seed}_t{ti}_{cfg}" if multi_tc else f"seed_{seed}_{cfg}")
            cmd = shlex.split(tc) + ["--edition", "2018"] + shlex.split(flags) + [str(src), "-o", str(binpath)]
            t0 = time.monotonic()
            cp = run_cmd(cmd, task.compile_timeout)
            compile_ms = int((time.monotonic() - t0) * 1000)
            e: dict[str, Any] = {
                "label": label, "toolchain": tc, "config": cfg,
                "compile_cmd": shlex.join(cmd), "compile_ms": compile_ms,
                "run_ms": None, "exit_code": None, "output_sha": None,
                "bin": None, "bin_sha": None, "run_timed_out": False,
                "compile_stderr": "", "signature": None,
            }
            if cp is None:
                e["status"] = "compile_timeout"
            else:
                stderr = cp.stderr.decode("utf-8", "replace")
                e["compile_stderr"] = stderr
                if stderr:
                    (seed_dir / f"{label}.compile.stderr").write_text(stderr)
                if is_ice(cp.returncode, stderr):
                    e["status"] = "ice"
                    e["signature"] = ice_signature(stderr)
                elif cp.returncode != 0 or not binpath.exists():
                    e["status"] = "compile_error"
                else:
                    e["status"] = "compiled"
                    e["bin"] = str(binpath)
                    e["bin_sha"] = sha256_file(binpath)
            entries.append(e)

    # 3. Run every successfully compiled binary.
    for e in entries:
        if e["bin"] is None:
            continue
        t0 = time.monotonic()
        rp = run_cmd([e["bin"]], task.run_timeout)
        e["run_ms"] = int((time.monotonic() - t0) * 1000)
        if rp is None:
            e["status"] = "run_timeout"
            e["run_timed_out"] = True
            (seed_dir / f"{e['label']}.run.stderr").write_text("<run timed out>\n")
        else:
            e["exit_code"] = rp.returncode
            e["output_sha"] = hashlib.sha256(rp.stdout).hexdigest()
            e["status"] = "ok" if rp.returncode == 0 else "run_crash"
            (seed_dir / f"{e['label']}.stdout").write_bytes(rp.stdout)
            if rp.stderr:
                (seed_dir / f"{e['label']}.run.stderr").write_bytes(rp.stderr)

    # 4. Classify findings.
    findings: list[dict[str, str]] = []
    for e in entries:
        if e["status"] == "ice":
            findings.append({"class": "ice", "signature": e["signature"]})
    cerrs = [e for e in entries if e["status"] == "compile_error"]
    if cerrs:
        findings.append({"class": "compile_error",
                         "signature": compile_error_signature(cerrs[0]["compile_stderr"])})
    ran = [e for e in entries if e["bin"] is not None]
    if ran:
        out["binary_shas"] = [e["bin_sha"] for e in ran]
        timeouts = [e for e in ran if e["run_timed_out"]]
        if len(timeouts) == len(ran):
            findings.append({"class": "run_timeout", "signature": "run_timeout"})
        elif timeouts:
            findings.append({"class": "divergence", "signature": divergence_signature(ran)})
        else:
            observed = {(e["exit_code"], e["output_sha"]) for e in ran}
            if len(observed) > 1:
                findings.append({"class": "divergence", "signature": divergence_signature(ran)})
            elif ran[0]["exit_code"] != 0:
                findings.append({"class": "run_crash",
                                 "signature": f"exit={ran[0]['exit_code']}"})
    out["findings"] = findings

    # 5. Artifacts: meta.json for findings; binaries are always deleted.
    if findings:
        meta = {
            "seed": seed,
            "generator_cmd": shlex.join(gen_cmd),
            "findings": findings,
            "entries": [{k: e[k] for k in (
                "label", "toolchain", "config", "status", "compile_cmd",
                "compile_ms", "run_ms", "exit_code", "output_sha", "bin_sha",
                "run_timed_out")} for e in entries],
        }
        (seed_dir / "meta.json").write_text(json.dumps(meta, indent=2))

    for e in entries:
        if e["bin"]:
            Path(e["bin"]).unlink(missing_ok=True)

    for e in entries:
        out["results"].append(result_row(
            e["toolchain"], e["config"], e["status"],
            e["compile_ms"], e["run_ms"], e["exit_code"], e["output_sha"]))
    return out


# ---------------------------------------------------------------------------
# Database
# ---------------------------------------------------------------------------

def open_db(path: Path) -> sqlite3.Connection:
    conn = sqlite3.connect(path)
    conn.executescript("""
        CREATE TABLE IF NOT EXISTS campaigns(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            started_at TEXT NOT NULL,
            args_json TEXT NOT NULL,
            toolchain_versions_json TEXT NOT NULL,
            host_json TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS results(
            campaign_id INTEGER NOT NULL,
            seed INTEGER NOT NULL,
            toolchain TEXT NOT NULL,
            config TEXT NOT NULL,
            status TEXT NOT NULL,
            compile_ms INTEGER,
            run_ms INTEGER,
            exit_code INTEGER,
            output_sha TEXT,
            PRIMARY KEY(campaign_id, seed, toolchain, config));
        CREATE TABLE IF NOT EXISTS findings(
            campaign_id INTEGER NOT NULL,
            seed INTEGER NOT NULL,
            class TEXT NOT NULL,
            signature TEXT NOT NULL,
            path TEXT NOT NULL);
    """)
    return conn


# ---------------------------------------------------------------------------
# run subcommand
# ---------------------------------------------------------------------------

def parse_configs(spec: str) -> list[tuple[str, str]]:
    names = [n.strip() for n in spec.split(",") if n.strip()]
    if not names:
        raise ValueError("no configs selected")
    out = []
    for n in names:
        if n not in CONFIGS:
            raise ValueError(f"unknown config {n!r}; available: {', '.join(CONFIGS)}")
        out.append((n, CONFIGS[n]))
    return out


def toolchain_version(tc: str) -> str:
    try:
        p = subprocess.run(shlex.split(tc) + ["--version", "--verbose"],
                           capture_output=True, text=True, timeout=30)
        return (p.stdout.strip() or p.stderr.strip()) or f"<exit {p.returncode}>"
    except Exception as e:  # noqa: BLE001 - metadata only
        return f"<error: {e}>"


def fmt_counts(c: Counter) -> str:
    return "  ".join(f"{k}={v}" for k, v in sorted(c.items())) or "none"


def cmd_run(args: argparse.Namespace) -> int:
    try:
        configs = parse_configs(args.configs)
    except ValueError as e:
        print(f"error: {e}", file=sys.stderr)
        return 2
    toolchains = [t.strip() for t in args.toolchains.split(",") if t.strip()] or [args.rustc]

    work = Path(args.work_dir).resolve()
    tmp_root = work / "tmp"
    findings_root = work / "findings"
    for d in (work, tmp_root, findings_root):
        d.mkdir(parents=True, exist_ok=True)
    programs_dir = work / "programs"
    if args.keep_programs:
        programs_dir.mkdir(exist_ok=True)

    versions = {tc: toolchain_version(tc) for tc in toolchains}
    host = {"platform": platform.platform(),
            "python": platform.python_version(),
            "cpus": os.cpu_count()}

    conn = open_db(work / "results.db")
    cur = conn.execute(
        "INSERT INTO campaigns(started_at, args_json, toolchain_versions_json, host_json) "
        "VALUES(?,?,?,?)",
        (datetime.now(timezone.utc).isoformat(timespec="seconds"),
         json.dumps(vars(args), default=str), json.dumps(versions), json.dumps(host)))
    campaign_id = cur.lastrowid
    conn.commit()

    seeds = range(args.start, args.start + args.count)
    tasks = [SeedTask(seed=s, generator_cmd=args.generator, toolchains=toolchains,
                      configs=configs, tmp_root=str(tmp_root),
                      generate_timeout=args.generate_timeout,
                      compile_timeout=args.compile_timeout,
                      run_timeout=args.run_timeout)
             for s in seeds]
    total = len(tasks)
    print(f"campaign {campaign_id}: seeds {args.start}..{args.start + args.count - 1}, "
          f"{len(toolchains)} toolchain(s) x {len(configs)} config(s), jobs={args.jobs}",
          flush=True)

    status_counts: Counter = Counter()
    finding_counts: Counter = Counter()
    ice_sig_occurrences: Counter = Counter()
    ice_sig_retained: Counter = Counter()   # seeds with retained artifacts, per sig
    bin_shas: dict[int, list[str]] = {}
    gen_errors_printed = 0
    done = 0
    interrupted = False
    t_start = time.monotonic()

    executor = ProcessPoolExecutor(max_workers=args.jobs, initializer=_worker_init)
    futures = [executor.submit(run_seed, t) for t in tasks]
    try:
        for fut in as_completed(futures):
            res = fut.result()
            seed = res["seed"]
            for r in res["results"]:
                status_counts[r["status"]] += 1
                conn.execute(
                    "INSERT OR REPLACE INTO results VALUES(?,?,?,?,?,?,?,?,?)",
                    (campaign_id, seed, r["toolchain"], r["config"], r["status"],
                     r["compile_ms"], r["run_ms"], r["exit_code"], r["output_sha"]))
            if res["binary_shas"]:
                bin_shas[seed] = res["binary_shas"]
            if res["notes"] and gen_errors_printed < 5:
                print(f"  seed {seed}: {res['notes']}", flush=True)
                gen_errors_printed += 1

            seed_dir = Path(res["seed_dir"])
            findings = res["findings"]
            path_str = ""
            if findings:
                ice_sigs = {f["signature"] for f in findings if f["class"] == "ice"}
                retain = (any(f["class"] != "ice" for f in findings)
                          or any(ice_sig_retained[s] < ICE_ARTIFACT_QUOTA for s in ice_sigs))
                if retain and seed_dir.exists():
                    cls = next(c for c in CLASS_PRIORITY
                               if any(f["class"] == c for f in findings))
                    dest = findings_root / cls / f"seed_{seed}"
                    dest.parent.mkdir(parents=True, exist_ok=True)
                    if dest.exists():
                        shutil.rmtree(dest)
                    shutil.move(str(seed_dir), str(dest))
                    path_str = str(dest)
                    for s in ice_sigs:
                        ice_sig_retained[s] += 1
                for f in findings:
                    finding_counts[f["class"]] += 1
                    if f["class"] == "ice":
                        ice_sig_occurrences[f["signature"]] += 1
                    conn.execute("INSERT INTO findings VALUES(?,?,?,?,?)",
                                 (campaign_id, seed, f["class"], f["signature"], path_str))

            if args.keep_programs:
                prog = (Path(path_str) if path_str else seed_dir) / f"seed_{seed}.rs"
                if prog.exists():
                    shutil.copy2(prog, programs_dir / prog.name)
            shutil.rmtree(seed_dir, ignore_errors=True)
            conn.commit()

            done += 1
            if done % 10 == 0:
                print(f"[{done}/{total}] statuses: {fmt_counts(status_counts)} | "
                      f"findings: {fmt_counts(finding_counts)}", flush=True)
    except KeyboardInterrupt:
        interrupted = True
        print("\ninterrupted: finalizing results already collected...", flush=True)
        executor.shutdown(wait=False, cancel_futures=True)
    else:
        executor.shutdown()
    finally:
        conn.commit()

    elapsed = max(time.monotonic() - t_start, 1e-6)
    rate = done / elapsed * 3600
    print("=== summary ===")
    print(f"campaign {campaign_id}: {done}/{total} seeds in {elapsed:.1f}s "
          f"({rate:.0f} programs/hour)" + ("  [interrupted]" if interrupted else ""))
    print(f"status counts: {fmt_counts(status_counts)}")
    if finding_counts:
        print("findings:")
        for cls, n in sorted(finding_counts.items()):
            print(f"  {cls}: {n}")
        if ice_sig_occurrences:
            print("ICE signatures:")
            for sig, n in ice_sig_occurrences.most_common():
                print(f"  {n}x {sig}")
    else:
        print("findings: none")

    # Sanity guard against a matrix that does not differentiate at all.
    multi = {s: v for s, v in bin_shas.items() if len(v) >= 2}
    if len(multi) >= 20 and all(len(set(v)) == 1 for v in multi.values()):
        bar = "!" * 72
        print(bar)
        print("WARNING: for EVERY one of the "
              f"{len(multi)} seeds that compiled, all binaries are byte-identical")
        print("across configs. The compile matrix may not be differentiating at all")
        print("(flags ignored / same binary reused?). Check the rustc command lines.")
        print(bar)

    conn.close()
    return 0


# ---------------------------------------------------------------------------
# report subcommand
# ---------------------------------------------------------------------------

def cmd_report(args: argparse.Namespace) -> int:
    db_path = Path(args.work_dir) / "results.db"
    if not db_path.exists():
        print(f"no results database at {db_path}")
        return 1
    conn = sqlite3.connect(db_path)
    campaigns = conn.execute(
        "SELECT id, started_at, args_json, toolchain_versions_json "
        "FROM campaigns ORDER BY id").fetchall()
    if not campaigns:
        print("no campaigns recorded")
        return 0
    for cid, started, args_json, tv_json in campaigns:
        versions = json.loads(tv_json)
        print(f"campaign {cid}  started {started}")
        lo, hi, n = conn.execute(
            "SELECT MIN(seed), MAX(seed), COUNT(DISTINCT seed) "
            "FROM results WHERE campaign_id=?", (cid,)).fetchone()
        print(f"  seeds: {lo}..{hi} ({n} completed)" if n else "  seeds: none completed")
        tcs = [r[0] for r in conn.execute(
            "SELECT DISTINCT toolchain FROM results WHERE campaign_id=?", (cid,))]
        cfgs = [r[0] for r in conn.execute(
            "SELECT DISTINCT config FROM results WHERE campaign_id=?", (cid,))]
        print(f"  matrix: {len(tcs)} toolchain(s) x {len(cfgs)} config(s)")
        for tc in tcs:
            ver = versions.get(tc, "")
            first = ver.splitlines()[0] if ver else "unknown version"
            print(f"    toolchain: {tc}  [{first}]")
        print(f"    configs: {', '.join(cfgs)}")
        counts = conn.execute(
            "SELECT status, COUNT(*) FROM results WHERE campaign_id=? "
            "GROUP BY status ORDER BY COUNT(*) DESC", (cid,)).fetchall()
        print("  statuses: " + ("  ".join(f"{s}={c}" for s, c in counts) or "none"))
        frows = conn.execute(
            "SELECT class, signature, seed, path FROM findings "
            "WHERE campaign_id=? ORDER BY class, signature, seed", (cid,)).fetchall()
        if not frows:
            print("  findings: none")
        else:
            grouped: dict[str, dict[str, list[tuple[int, str]]]] = {}
            for cls, sig, seed, path in frows:
                grouped.setdefault(cls, {}).setdefault(sig, []).append((seed, path))
            print(f"  findings: {len(frows)}")
            for cls in sorted(grouped):
                total = sum(len(v) for v in grouped[cls].values())
                print(f"    {cls} ({total}):")
                for sig, occurrences in sorted(grouped[cls].items()):
                    print(f"      [{len(occurrences)}x] {sig or '<no signature>'}")
                    for seed, path in occurrences:
                        print(f"          seed {seed}: {path or '<artifacts not retained>'}")
        print()
    conn.close()
    return 0


# ---------------------------------------------------------------------------
# clean subcommand
# ---------------------------------------------------------------------------

def cmd_clean(args: argparse.Namespace) -> int:
    work = Path(args.work_dir)
    if work.exists():
        shutil.rmtree(work, ignore_errors=True)
        print(f"removed {work}")
    else:
        print(f"{work} does not exist; nothing to do")
    return 0


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def build_parser() -> argparse.ArgumentParser:
    ap = argparse.ArgumentParser(
        prog="fuzz.py",
        description="Parallel differential-testing driver for the rustsmith generator.")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("run", help="generate/compile/run/compare a range of seeds")
    p.add_argument("--count", type=int, required=True, help="number of seeds")
    p.add_argument("--start", type=int, default=0, help="first seed (default 0)")
    p.add_argument("--jobs", type=int, default=os.cpu_count() or 1,
                   help="parallel workers (default: cpu count)")
    p.add_argument("--generator", default=DEFAULT_GENERATOR,
                   help=f"generator command (default: {DEFAULT_GENERATOR})")
    p.add_argument("--rustc", default="rustc",
                   help="rustc command used when --toolchains is not given")
    p.add_argument("--configs", default=",".join(CONFIGS),
                   help="comma list of config names (default: all built-ins)")
    p.add_argument("--toolchains", default="",
                   help="comma list of rustc command prefixes, e.g. "
                        "'rustc,rustup run nightly rustc'")
    p.add_argument("--work-dir", default=DEFAULT_WORK_DIR)
    p.add_argument("--keep-programs", action="store_true",
                   help="keep every generated .rs under <work-dir>/programs/")
    p.add_argument("--generate-timeout", type=float, default=30.0,
                   help="generator timeout in seconds (default 30)")
    p.add_argument("--compile-timeout", type=float, default=120.0,
                   help="compile timeout in seconds (default 120)")
    p.add_argument("--run-timeout", type=float, default=10.0,
                   help="binary run timeout in seconds (default 10)")

    p = sub.add_parser("report", help="print a summary of recorded campaigns")
    p.add_argument("--work-dir", default=DEFAULT_WORK_DIR)

    p = sub.add_parser("clean", help="delete the work dir")
    p.add_argument("--work-dir", default=DEFAULT_WORK_DIR)
    return ap


def main(argv: Optional[list[str]] = None) -> int:
    args = build_parser().parse_args(argv)
    if args.cmd == "run":
        return cmd_run(args)
    if args.cmd == "report":
        return cmd_report(args)
    return cmd_clean(args)


if __name__ == "__main__":
    sys.exit(main())
