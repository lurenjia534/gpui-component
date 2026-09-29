# Scroll-table column-width cache evidence

Supporting material for **text: Avoid remeasuring scroll-table column widths**.
This independent evidence branch follows merged PRs
[#3273](https://github.com/longbridge/gpui-kit/pull/3273) and
[#3247](https://github.com/longbridge/gpui-kit/pull/3247). The code PR contains
only `crates/base/src/text/node.rs`, including its regression tests.

## Start here

- [Brief assessment: benefits, costs and unresolved issues](BRIEF-REPORT.md).
- [Complete performance and memory report](TABLE-COLUMN-WIDTH-CACHE-PERFORMANCE.md).
- [Measurement and submission provenance](PROVENANCE.json).
- [Reproduction instructions](table-column-width-cache-evidence/README.md).
- [All-file checksums](MANIFEST.sha256).

## Main results

| Scenario | Baseline | Patched | Assessment |
| --- | ---: | ---: | --- |
| Native 200×8 vertical page scroll | 84.525 present FPS | 92.797 present FPS | +9.8%; 95% change interval [+3.263, +18.331] FPS; 7/8 pairs higher |
| CPU draw for the same native scroll | 9.754 ms/frame | 8.836 ms/frame | Saves 0.918 ms/frame (9.4%); 95% saving interval [0.533, 4.149] ms |
| Native 200×8 static redraw | 87.146 present FPS | 88.058 present FPS | Inconclusive: interval crosses zero |
| Native 48×6 scroll / wrap control | Approximately 170 FPS | Approximately 170 FPS | Display-rate capped; no demonstrated FPS gain |
| Separate ordinary four-column memory probe | No column-width cache | Approximately 528 B extra per live table | About 0.50 MiB for 1,000 tables; not the eight-column fixture's exact cost |
| Initial draw | Full scan | Full scan | No demonstrated first-draw improvement |

Native results use Linux Wayland, Ryzen 9 7950X, RX 7900 XTX (RADV Mesa
26.2.1), a 3840×2160 @170 Hz display at scale 1.75, and an 800×600 logical-pixel
window. There are 68 valid processes and 40,800 measured presents, after 120
warmup presents per process. FPS uses actual GPUI platform-submission timestamps,
not inverse CPU draw time or compositor-confirmed scanout. This is a standalone
native Base/TextView application. The report retains unfavorable cases, earlier
headless results and the focus-invalid run.

## Baseline and submission validation

- Frozen measurement baseline:
  [`11b04d9bf09cec68c98ce97d759afc6776b1cda0`](https://github.com/longbridge/gpui-kit/commit/11b04d9bf09cec68c98ce97d759afc6776b1cda0).
- Submission parent after synchronizing upstream:
  [`4f861e8a0654064fa53255cf8d9597bb9d5ef826`](https://github.com/longbridge/gpui-kit/commit/4f861e8a0654064fa53255cf8d9597bb9d5ef826).
- Code commit:
  [`de586ce32f2e8bc6d3047897bf234ba66584544d`](https://github.com/lurenjia534/gpui-component/commit/de586ce32f2e8bc6d3047897bf234ba66584544d).

The intervening three upstream commits do not change `node.rs` or the lockfile.
The submitted `node.rs` is byte-identical to the measured patched snapshot.
Performance was measured on the frozen baseline and was not rerun after syncing.
Validation rerun on the submission base:

- [Base library tests](submission-validation/base-tests.log): **1,247 passed, 0 failed**.
- [Base lib/tests Clippy](submission-validation/base-clippy.log): passed with warnings denied.
- [Base formatting and the code diff whitespace check](submission-validation/checks.md): passed.

The five focused column-width tests also pass. They cover plain widths, clone
sharing, window/text-style invalidation, reparsing, inline code and custom-cell
growth/shrinkage. The evidence harness remains outside the product patch.

## Open review issue and limits

**The PR is a draft because dynamic-font invalidation is unresolved.**
The cache checks window text-system identity, text style, rem size, mono family
and inline-code style, but not the font generation changed by `TextSystem::add_fonts`.
Installing fonts within the same window may therefore leave cached widths stale.
That boundary needs a correctness solution or evidence before merge.

Memory has no global quota. Storage follows live tables, columns and custom-cell
coordinates; retained clones may own different cache versions. Replacement/drop
tests support cache release, but the allocator can retain RSS. Long-running native
memory stress, full Story Gallery performance and macOS/Windows were not tested.
No font-system API change, memory-budget feature or first-layout budget is bundled
into this PR.

## Archive and reproduction

Clone the fork with its normal branch history, then create an evidence worktree:

```sh
git clone https://github.com/lurenjia534/gpui-component.git
cd gpui-component
git worktree add ../column-width-evidence origin/evidence/table-column-width-cache-20260929
cd ../column-width-evidence
sha256sum -c MANIFEST.sha256
```

Follow the [reproduction guide](table-column-width-cache-evidence/README.md) for
building frozen variants or recomputing summaries. Do not use a shallow clone
that omits the frozen baseline. Source snapshots, mixed English/CJK fixture text
and raw measurements remain unchanged. Narrative documents are English only.
Compiled binaries, bytecode and machine-local work selectors are omitted; build
metadata retains binary hashes. AI assisted with implementation, validation,
benchmarking, analysis and publication preparation.
