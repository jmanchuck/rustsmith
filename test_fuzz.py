#!/usr/bin/env python3
"""Unit tests for fuzz.py using fake generator/rustc executables.

Run with:  python3 test_fuzz.py
"""

from __future__ import annotations

import contextlib
import io
import sqlite3
import tempfile
import unittest
from pathlib import Path

import fuzz


def write_script(dirpath: Path, name: str, body: str) -> str:
    """Write an executable python script and return its path."""
    p = Path(dirpath) / name
    p.write_text("#!/usr/bin/env python3\n" + body)
    p.chmod(0o755)
    return str(p)


def gen_body(program: str) -> str:
    """Fake generator: writes DIR/seed_N.rs containing `program`."""
    return (
        "import argparse, pathlib\n"
        "p = argparse.ArgumentParser()\n"
        "p.add_argument('--seed', type=int, required=True)\n"
        "p.add_argument('--count', type=int, default=1)\n"
        "p.add_argument('--out', required=True)\n"
        "a = p.parse_args()\n"
        f"pathlib.Path(a.out, f'seed_{{a.seed}}.rs').write_text({program!r})\n"
    )


# Common prelude for fake rustcs: handles --version, extracts the -o value,
# the opt-level from `-C opt-level=X` pairs, the source file, and the seed.
FAKE_RUSTC_PRELUDE = """\
import sys, os, pathlib, re
args = sys.argv[1:]
if "--version" in args:
    print("fake-rustc 1.0.0 (deadbeef 2026-01-01)")
    sys.exit(0)
out = args[args.index("-o") + 1]
opt = "0"
for i, a in enumerate(args):
    if a == "-C" and i + 1 < len(args) and args[i + 1].startswith("opt-level="):
        opt = args[i + 1].split("=", 1)[1]
src = next(a for a in args if a.endswith(".rs"))
seed = int(re.search(r"seed_(\\d+)", pathlib.Path(src).name).group(1))
def emit(body):
    p = pathlib.Path(out)
    p.write_text("#!/usr/bin/env python3\\n" + body)
    p.chmod(0o755)
"""

BODY_DIVERGE = FAKE_RUSTC_PRELUDE + """\
emit(f"print('output for opt {opt}')")
sys.exit(0)
"""

# Emits an ICE whose file:line:col varies with the seed, so two seeds only
# share a signature if normalization strips the numbers.
BODY_ICE = FAKE_RUSTC_PRELUDE + """\
sys.stderr.write(f"error: internal compiler error: entered unreachable code"
                 f" at compiler/rustc_middle/src/x.rs:{100 + seed}:{seed + 1}\\n")
sys.stderr.write("thread 'rustc' panicked at compiler/rustc_middle/src/x.rs\\n")
sys.exit(101)
"""

BODY_MIX = FAKE_RUSTC_PRELUDE + """\
if opt == "0":
    sys.stderr.write("error[E0308]: mismatched types\\n")
    sys.exit(1)
sys.stderr.write("error: internal compiler error: boom\\n")
sys.exit(101)
"""

BODY_SLEEPY = FAKE_RUSTC_PRELUDE + """\
emit("import time\\ntime.sleep(10)")
sys.exit(0)
"""

BODY_OK = FAKE_RUSTC_PRELUDE + """\
emit("print('hello')")
sys.exit(0)
"""


def run_fuzz(argv: list[str]) -> tuple[int, str]:
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        rc = fuzz.main(argv)
    return rc, buf.getvalue()


class FuzzTestCase(unittest.TestCase):
    def setUp(self) -> None:
        tmp = tempfile.TemporaryDirectory(prefix="fuzztest_")
        self.addCleanup(tmp.cleanup)
        self.dir = Path(tmp.name)
        self.work = str(self.dir / "work")
        self.gen = write_script(self.dir, "fake_gen.py", gen_body("fn main(){}\n"))

    # -- helpers -----------------------------------------------------------

    def run_pipeline(self, rustc_body: str, configs: str, count: int = 1,
                     extra: tuple[str, ...] = ()) -> str:
        rustc = write_script(self.dir, "fake_rustc.py", rustc_body)
        rc, out = run_fuzz(["run", "--count", str(count), "--jobs", "2",
                            "--generator", self.gen, "--rustc", rustc,
                            "--configs", configs, "--work-dir", self.work,
                            *extra])
        self.assertEqual(rc, 0, out)
        return out

    def query(self, sql: str) -> list[tuple]:
        conn = sqlite3.connect(Path(self.work) / "results.db")
        try:
            return conn.execute(sql).fetchall()
        finally:
            conn.close()

    def findings(self) -> list[tuple]:
        return self.query("SELECT class, signature, seed, path FROM findings")

    # -- tests ---------------------------------------------------------------

    def test_divergence_detected(self) -> None:
        """Binaries whose output depends on opt-level -> divergence finding."""
        self.run_pipeline(BODY_DIVERGE, "O0,O2")
        rows = self.findings()
        self.assertEqual(len(rows), 1)
        cls, _sig, seed, path = rows[0]
        self.assertEqual(cls, "divergence")
        self.assertEqual(seed, 0)
        # Artifacts saved: program, meta.json, per-config stdout.
        d = Path(path)
        self.assertTrue(d.is_dir(), path)
        self.assertTrue((d / "seed_0.rs").exists())
        self.assertTrue((d / "meta.json").exists())
        self.assertTrue((d / "O0.stdout").exists())
        self.assertTrue((d / "O2.stdout").exists())
        self.assertNotEqual((d / "O0.stdout").read_bytes(),
                            (d / "O2.stdout").read_bytes())
        # Both entries ran fine individually; divergence is seed-level.
        statuses = {r[0] for r in self.query("SELECT status FROM results")}
        self.assertEqual(statuses, {"ok"})
        shas = {r[0] for r in self.query("SELECT output_sha FROM results")}
        self.assertEqual(len(shas), 2)

    def test_ice_classified_and_deduped(self) -> None:
        """Two seeds, same ICE modulo line numbers -> one signature, 2 occurrences."""
        self.run_pipeline(BODY_ICE, "O0", count=2)
        rows = self.findings()
        self.assertEqual(len(rows), 2)
        self.assertEqual({r[0] for r in rows}, {"ice"})
        sigs = {r[1] for r in rows}
        self.assertEqual(len(sigs), 1, f"signatures not deduped: {sigs}")
        sig = next(iter(sigs))
        self.assertIn("internal compiler error", sig)
        # Numbers and paths must be normalized out of the signature.
        self.assertNotRegex(sig, r"\d")
        occurrences = self.query(
            "SELECT signature, COUNT(*) FROM findings GROUP BY signature")
        self.assertEqual(occurrences[0][1], 2)
        statuses = {r[0] for r in self.query("SELECT status FROM results")}
        self.assertEqual(statuses, {"ice"})

    def test_compile_error_vs_ice(self) -> None:
        """Exit 1 with a plain error -> compile_error; exit 101 + ICE text -> ice."""
        self.run_pipeline(BODY_MIX, "O0,O2")
        statuses = dict(self.query("SELECT config, status FROM results"))
        self.assertEqual(statuses, {"O0": "compile_error", "O2": "ice"})
        classes = {r[0] for r in self.findings()}
        self.assertEqual(classes, {"compile_error", "ice"})

    def test_run_timeout(self) -> None:
        """Binary sleeps past the (overridden) run timeout in ALL configs."""
        self.run_pipeline(BODY_SLEEPY, "O0,O2", extra=("--run-timeout", "1"))
        rows = self.findings()
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0][0], "run_timeout")
        statuses = {r[0] for r in self.query("SELECT status FROM results")}
        self.assertEqual(statuses, {"run_timeout"})

    def test_all_agree_happy_path(self) -> None:
        """Identical behavior everywhere -> no findings, all ok."""
        self.run_pipeline(BODY_OK, "O0,O2,O3", count=2)
        self.assertEqual(self.findings(), [])
        rows = self.query("SELECT status, exit_code FROM results")
        self.assertEqual(len(rows), 6)  # 2 seeds x 3 configs
        self.assertEqual({r[0] for r in rows}, {"ok"})
        self.assertEqual({r[1] for r in rows}, {0})

    def test_integration_real_rustc(self) -> None:
        """Real rustc on a tiny valid program: O0 and O2 compile, run, agree."""
        gen = write_script(self.dir, "real_gen.py",
                           gen_body('fn main(){println!("42");}\n'))
        rc, out = run_fuzz(["run", "--count", "1", "--jobs", "2",
                            "--generator", gen, "--rustc", "rustc",
                            "--configs", "O0,O2", "--work-dir", self.work])
        self.assertEqual(rc, 0, out)
        rows = self.query("SELECT config, status, exit_code, output_sha FROM results")
        self.assertEqual(len(rows), 2)
        self.assertEqual({r[1] for r in rows}, {"ok"})
        self.assertEqual({r[2] for r in rows}, {0})
        self.assertEqual(len({r[3] for r in rows}), 1)  # identical stdout
        self.assertEqual(self.findings(), [])
        # report subcommand reads the DB back.
        rc, report = run_fuzz(["report", "--work-dir", self.work])
        self.assertEqual(rc, 0)
        self.assertIn("campaign 1", report)
        self.assertIn("findings: none", report)
        self.assertIn("ok=2", report)


if __name__ == "__main__":
    unittest.main(verbosity=2)
