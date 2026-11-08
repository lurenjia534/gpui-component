Paragraph highlight merging: issue #3695

Scope
-----
Frozen baseline: 8da4e005258ac0aac9a6a332cce746eeaafa08a8
Submitted code: eef63049efd728fe8d4734102ff11e7b823bd0a8
Issue: https://github.com/longbridge/gpui-kit/issues/3695

This evidence covers the repeated paragraph highlight merging problem only.
The code PR changes two files and adds no public API. Four paragraph builders
use a node-local merge and append for disjoint ranges. Caller-provided marks
extending into an earlier range retain the original whole-list merge path.
Per-node mark sorting and the compatibility fallback are not claimed linear.

Measurement
-----------
Linux x86_64, AMD Ryzen 9 7950X, rustc 1.99.1; gpui-base 0.9.0,
gpui-pre 0.3.11, gpui-fast 0.1.13. The benchmark uses default GPUI.
Only gpui-base is overridden to opt-level=3 in the repository dev/test profile;
debug assertions remain enabled. This is not a full release build.

Three original methods are extracted verbatim from before-node.rs and renamed
with a legacy_ prefix in the same test binary as the submitted implementation.
The Rust used in the measurement is archived in benchmark-module.rs. The
portable Python wrappers differ from the executed wrappers only in accepting
a repository path instead of a machine-specific absolute path; their syntax
and --help were checked, and the embedded Rust benchmark was verified identical.

Each case has calibration/warm-up, then nine samples with alternating old/new
order. Each sample averages independently calibrated iterations targeting
approximately 15 ms, with at least one call. Results are the median of nine
sample means; min/max and iteration counts are in benchmark-results.json.
The test-only highlight counter is disabled while timing. Input creation,
Markdown parsing, and cold-cache invalidation are excluded. Returned-result
destruction is included. Scheduling is unpinned; samples are from one process.

Chunk: "Some prose with `inline code` and **bold text** here. "
All fixtures compare old/new highlights, rendered text, and links. Extra cases
cover overlapping UTF-8 marks, empty marks, and marks extending across nodes.

InlineFlow item construction (not frame time):

Chunks  Paragraphs   Before ms   After ms   Speedup
   500           1  147.149935   0.409026   359.76x
  1000           1  608.372667   0.837949   726.03x
  2000           1 2369.340005   1.610897  1470.82x
   500          25    6.604241   0.402009    16.43x

For 500 chunks in one paragraph, measurement highlights fall from 150.752636
to 0.381508 ms; cold plain-render construction falls from 162.142981 to
0.459253 ms. The warm plain-render cache control is effectively unchanged:
0.045544 vs 0.045107 ms. Full data includes 250/500/1000/2000 chunks and
all three builders plus the warm-cache control.

These timings exclude shaping, line wrapping, GPU rendering, and presentation;
they must not be interpreted as end-to-end frame speedups. No macOS/Windows
performance or native Story Gallery run is claimed.

Validation
----------
The final source files match the measured after-*.rs snapshots byte for byte.

cargo test -p gpui-base --features test-support --lib --locked text:: -- --test-threads=1
  428 passed, 0 failed (text-tests.log).

cargo test -p gpui-base --features gpui-fast,test-support --lib --locked text:: -- --test-threads=1
  424 passed, 5 failed after the patch (text-tests-fast.log).
  422 passed, the same 5 failed on the exact unpatched latest baseline
  (text-tests-fast-baseline.log; comparison in fast-baseline-comparison.log).
  Both newly added regressions pass on both backends.

Unchanged GPUI Fast failures:
- text::state::tests::reveal_range::a_reveal_that_cannot_be_shown_gives_up
- text::text_view::tests::long_press_release_keeps_handles_which_drag_the_selection
- text::text_view::tests::long_press_selects_word_then_drag_extends_selection
- text::text_view::tests::ordered_markdown_list_start_reaches_layout_marker
- text::text_view::tests::table_row_backgrounds_follow_the_frame_corner_radius

Rolling back only the four production call sites makes the cost regression
fail: 6,240 highlight inputs instead of 192 for 64 nodes. The overlap/UTF-8
regression still passes (baseline-regression.log). After restoring the patch
and test counter, the two focused regressions pass again
(final-regression-tests.log).

cargo clippy -p gpui-base --features test-support --lib --tests --locked -- -D warnings
  Passed (clippy.log).

rustfmt --edition 2024 --check crates/base/src/text/inline.rs crates/base/src/text/node.rs
  Passed (fmt-targeted.log).

cargo fmt --check
  Fails on two existing differences in unchanged text/inline_flow.rs
  (fmt-workspace.log). The code PR does not modify that file.

git diff --check
  Passed. Temporary benchmark code was removed from the code PR.

Reproduction
------------
Use two separate checkouts, keeping the evidence checkout separate from the
code checkout. Run this from a directory with enough disk space for GPUI:

  git clone https://github.com/lurenjia534/gpui-component gpui-kit-3695
  git -C gpui-kit-3695 checkout eef63049efd728fe8d4734102ff11e7b823bd0a8
  git -C gpui-kit-3695 fetch origin evidence/paragraph-highlights-3695
  git -C gpui-kit-3695 worktree add ../gpui-kit-3695-evidence origin/evidence/paragraph-highlights-3695
  python3 gpui-kit-3695-evidence/docs/evidence/paragraph-highlights-3695/benchmark.py gpui-kit-3695

The script temporarily edits the two code files and restores them in finally.
Do not run concurrent edits or formatters in that code checkout. It writes its
new benchmark log into the evidence directory; copy the evidence directory to
another location first if you want to preserve the archived files unchanged.
The two check-*-baseline.py scripts similarly accept the code checkout path.
SHA256SUMS records every archived file except itself.

AI assistance
-------------
Codex applied the supplied implementation and regression tests, adapted their
formatting, reviewed the diff, performed regression and benchmark validation,
and prepared the PR and this evidence package.
