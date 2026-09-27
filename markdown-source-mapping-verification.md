# Markdown source mapping fix verification

Date: 2026-09-27. All 23 performance cases are complete.

The patch passed actual Rust compilation, Markdown regression tests, the complete `gpui-base` library suite, and Clippy. Source alignment for repeated-space input now grows approximately linearly rather than quadratically. The returned vector for ordinary text also no longer retains capacity proportional to the character count.

Baseline commit: `452e72f24a183174551fdec3db570997b1e1fc8f`. The applied patch was the supplied `markdown-source-mapping.patch`. The product diff changes only `crates/base/src/text/format/markdown.rs`, including the implementation and five new regression tests. Three line-wrapping differences in the patch were adjusted with rustfmt. Public APIs, Markdown size limits, and TextView scheduling are unchanged.

## Measurement method

- Environment: Linux x86_64, AMD Ryzen 9 7950X, rustc/cargo 1.98.0.
- Measurements run in a real `gpui-base` test binary. The old parser was extracted byte-for-byte from the source before applying the patch, with its original test module removed, and included in temporary instrumentation. The new parser calls the modified production functions directly. Both versions use the same dependencies and configuration.
- Performance measurements set `profile.dev.package.gpui-base.opt-level=3`; all other dependencies use the repository dev configuration. This is not a full release build.
- Each operation has one warm-up and three measured calls. Tables report the median. Input construction, result verification, and destruction of returned results are excluded. Searches, decoding, segment generation, and the old helper's final compaction are included.
- "Source alignment" measures only `aligned_source_segments`. "Complete parsing" measures the real `format::markdown::parse`, including mdast parsing and document construction, without calling a selection API. These timings do not represent GUI frame latency.
- Every performance case also asserts identical old/new helper mappings. Cases with complete parsing compare the old/new `ParsedDocument` values as well.

## Repeated-space paragraphs: before and after

Input is `"a ".repeat(n / 2)`, with no line endings. The Markdown parser removes the final trailing space, so the rendered length is 1 byte shorter than the input size in the table.

| Input (KiB) | Alignment before (ms) | Alignment after (ms) | Complete parsing before (ms) | Complete parsing after (ms) | Complete parsing speedup |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 32 | 203.292 | 0.276 | 206.347 | 3.725 | 55.4x |
| 64 | 811.222 | 0.551 | 814.399 | 7.398 | 110.1x |
| 96 | 1,808.482 | 0.830 | 1,824.782 | 11.102 | 164.4x |
| 128 | 3,235.884 | 1.136 | 3,248.564 | 16.151 | 201.1x |
| 256 | 12,984.815 | 2.206 | 13,062.822 | 32.487 | 402.1x |

At 32 → 64 → 128 KiB, alignment before the fix takes approximately 203 → 811 → 3,236 ms: doubling the input roughly quadruples the time. After the fix, the corresponding times are approximately 0.276 → 0.551 → 1.136 ms, close to doubling.

## Other input shapes

| Case | Raw bytes | Alignment before (ms) | Alignment after (ms) | Speedup |
| --- | ---: | ---: | ---: | ---: |
| Contiguous horizontal whitespace | 65,536 | 1,623.541 | 0.563 | 2882.9x |
| Multiline code | 65,536 | 319.346 | 0.497 | 642.0x |
| Unmapped characters | 65,535 | 57.568 | 0.445 | 129.4x |
| Entity candidate after mapping gaps | 65,536 | 1,084.990 | 0.497 | 2182.5x |

Contiguous horizontal whitespace is `a + spaces + b`. The multiline-code helper receives repeated `x\n` as raw input; the complete parser additionally receives code fences. The unmapped-character and entity-candidate cases construct helper inputs directly to cover failed searches and caching. They are synthetic inputs, and are not claimed to arise naturally as a single mdast text node.

| Complete parsing case | Code body or text (KiB) | Total Markdown bytes | Before (ms) | After (ms) | Speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| Contiguous horizontal whitespace | 64 | 65,536 | 1,614.180 | 4.416 | 365.5x |
| Multiline code | 64 | 65,543 | 485.803 | 169.341 | 2.9x |
| Multiline code | 256 | 262,151 | 7,622.079 | 2,640.968 | 2.9x |

Complete multiline-code parsing still grows noticeably superlinearly. The helper timings above are approximately linear, but complete parsing includes other processing. Those remaining costs were not further isolated or modified in this work.

The 23 performance cases cover repeated spaces at 32/64/96/128/256 KiB, contiguous horizontal whitespace at 8/16/32/64 KiB, multiline code at 8/16/32/64/128/256 KiB, and four sizes each for unmapped characters and entity candidates. Actual raw/rendered/source byte counts and all three samples for every case are preserved in `results.json`.

## Vector capacity

For a 96 KiB repeated-space paragraph, both helpers return one segment, but `Vec::capacity()` falls from **98,303** to **4**. Each `SourceSegment` is 32 bytes on this machine, so the returned vector's element storage falls from **3,145,696 bytes (approximately 3 MiB)** to **128 bytes**.

The old helper first generates a segment for every character, then reserves the original segment count for its compacted result. Merging segments during generation removes both the intermediate vector of per-character segments and the excessive returned capacity. This measures actual returned capacity, not process RSS or peak heap usage.

## Correctness and static checks

| Check | Result |
| --- | --- |
| Baseline Markdown tests | 52 passed, 0 failed |
| Patched Markdown tests | 57 passed, 0 failed; all five new regression tests pass |
| Complete patched `gpui-base` library suite | 1,157 passed, 0 failed |
| Actual Rust helper differential validation | 2,018,720 cases pass, covering Unicode, whitespace, CR/LF, escapes, valid/invalid entities, gaps, and nonzero source offsets |
| Public TextView background parsing path | Passed; initialization, background completion, and `set_text` replacement for approximately 96 KiB paragraphs/code blocks complete without a selection call |
| Performance cases | 23 passed; every case also verifies identical old/new mappings |
| `cargo clippy --locked -p gpui-base --all-targets -- -D warnings` | Passed |
| rustfmt for the changed file | Passed |
| `cargo fmt -p gpui-base -- --check` | Passed |
| `cargo fmt --all -- --check` | Failed: pre-existing formatting differences in the unchanged `crates/component/src/form/tests.rs`; see the log |
| `git diff --check` | Passed |

Public-path validation runs in `gpui::TestAppContext`, using the real TextView state and the test executor. Native-window drawing, real-executor starvation across multiple documents, and cross-platform behavior were not measured. Common helper paths are O(N+M); the gap-index path has an upper bound of O((N+M) log(N+1)). These results do not establish linear complexity for the entire Markdown parser on all inputs or a fixed resource budget for arbitrarily large documents.

## Evidence and reproduction

The final report is in the project root. Raw logs, source snapshots, and reproduction scripts are in the project's `markdown-source-mapping-verification/` directory.

- [Baseline Markdown tests](markdown-source-mapping-verification/logs/baseline-markdown-tests.log)
- [Patched Markdown tests](markdown-source-mapping-verification/logs/patched-markdown-tests.log)
- [Complete library suite](markdown-source-mapping-verification/logs/patched-base-tests.log)
- [Rust differential and public-path validation](markdown-source-mapping-verification/logs/rust-validation.log)
- [Raw benchmark output](markdown-source-mapping-verification/logs/benchmark.log)
- [Completed samples before the interrupted run](markdown-source-mapping-verification/logs/benchmark-interrupted.log)
- [Structured performance results](markdown-source-mapping-verification/results.json)
- [Clippy](markdown-source-mapping-verification/logs/clippy.log) · [Base formatting](markdown-source-mapping-verification/logs/fmt-base.log) · [Final workspace formatting](markdown-source-mapping-verification/logs/fmt-final.log)
- [Applied and formatted patch](markdown-source-mapping-verification/applied-and-formatted.patch) · [Diff check](markdown-source-mapping-verification/logs/diff-check.log)
- [Temporary validation source](markdown-source-mapping-verification/benchmark.rs) · [Environment](markdown-source-mapping-verification/environment.json) · [Input and validation-code SHA-256](markdown-source-mapping-verification/sha256.json)

One performance run was explicitly stopped to move its output directory. The four completed cases were retained, and the remaining cases ran after the move.

Reproduce from the repository root:

```sh
cargo test --locked -p gpui-base --lib text::format::markdown::tests
cargo test --locked -p gpui-base --lib
cargo fmt -p gpui-base -- --check
cargo clippy --locked -p gpui-base --all-targets -- -D warnings
git diff --check

# Temporarily install the validation module, then restore the product file.
python3 markdown-source-mapping-verification/rerun.py
# Each run uses a fresh output directory and leaves archived results unchanged.
python3 markdown-source-mapping-verification/rerun.py --benchmark
```

The temporary validation module has been removed from the final product source. The remaining source changes are the patch and rustfmt line wrapping.

Fresh reproduction logs and performance results are written under the project's `target/markdown-source-mapping-reproduction/run-*/` directory. The fix commit and delivery provenance are recorded in `PROVENANCE.json` in the verification directory.
