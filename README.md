# Markdown source mapping verification

Supporting evidence for `text: Avoid quadratic Markdown source mapping`.

- Baseline: [`452e72f24a183174551fdec3db570997b1e1fc8f`](https://github.com/lurenjia534/gpui-component/commit/452e72f24a183174551fdec3db570997b1e1fc8f).
- Fix: [`4c58296ff116cabe5ac6a0ccd6be2b74536cc4f7`](https://github.com/lurenjia534/gpui-component/commit/4c58296ff116cabe5ac6a0ccd6be2b74536cc4f7).
- The fix changes one source file and adds five regression tests. This independent evidence branch contains the supporting artifacts, following the structure used in merged [PR #3247](https://github.com/longbridge/gpui-kit/pull/3247).

## Results

An ordinary paragraph made from repeated `a ` pairs triggers eager source alignment without selection. At 256 KiB, complete Markdown parsing fell from **13,062.822 ms to 32.487 ms**; alignment alone fell from **12,984.815 ms to 2.206 ms**.

The final Base suite reports **1,157 passed, 0 failed**. All 23 performance cases preserve the old helper's mapping results; 2,018,720 Rust differential cases and the public TextView background path also pass. Clippy and Base formatting pass. Full-workspace formatting fails only in the unchanged `crates/component/src/form/tests.rs`.

Linux x86_64 / Ryzen 9 7950X / Rust 1.98.0. Only gpui-base uses `opt-level=3`, with dependencies using the repository dev profile. Each measured operation has one warm-up and three samples; tables use independent medians. Result verification and returned-result destruction are excluded. These are parser/helper measurements, not GUI frame latency or a full release build.

## Evidence

- [Detailed report and before/after tables](markdown-source-mapping-verification.md).
- [Actual execution history](markdown-source-mapping-verification/PROCESS.md) and [commit/validation provenance](markdown-source-mapping-verification/PROVENANCE.json).
- [Raw timing samples and returned vector capacities](markdown-source-mapping-verification/results.json).
- [Final library tests](markdown-source-mapping-verification/logs/patched-base-tests.log), [differential/public-path validation](markdown-source-mapping-verification/logs/rust-validation.log), and [reproducer verification](markdown-source-mapping-verification/logs/reproducer-check.log).
- [Benchmark transcript](markdown-source-mapping-verification/logs/benchmark.log) and [earlier completed rows before the output-directory move](markdown-source-mapping-verification/logs/benchmark-interrupted.log).
- [Original patch](markdown-source-mapping-verification/original.patch), [formatted applied patch](markdown-source-mapping-verification/applied-and-formatted.patch), [old source](markdown-source-mapping-verification/markdown-before.rs), and [tested new source](markdown-source-mapping-verification/markdown-after.rs).
- [Temporary Rust instrumentation](markdown-source-mapping-verification/benchmark.rs), [environment](markdown-source-mapping-verification/environment.json), and [all-file checksums](MANIFEST.sha256).

## Reproduce

In a disposable checkout of the fork:

```sh
git switch --detach 4c58296ff116cabe5ac6a0ccd6be2b74536cc4f7
git worktree add --detach target/markdown-source-mapping-evidence origin/evidence/markdown-source-mapping-20260927
python3 target/markdown-source-mapping-evidence/markdown-source-mapping-verification/rerun.py --repo .
python3 target/markdown-source-mapping-evidence/markdown-source-mapping-verification/rerun.py --repo . --benchmark
```

The reproducer checks the exact tested source, temporarily installs the Rust module, and restores the source afterwards. Every run writes fresh logs/results under `target/markdown-source-mapping-reproduction/run-*/`, leaving this archive unchanged. It stops rather than overwriting a product file changed during a run. Benchmark reproduction runs all 23 cases and takes several minutes on the measured machine.

To verify archive integrity, run `sha256sum -c MANIFEST.sha256` from this evidence worktree.

## Limits

This fixes source alignment. Complete multiline-code parsing still grows superlinearly in the measured sizes: a 256 KiB code body takes 2,640.968 ms after the fix, while its alignment takes 1.988 ms. The remaining parser cost was not changed or attributed to a specific component. Native-window frame latency, real executor starvation, and macOS/Windows behavior were not measured.
