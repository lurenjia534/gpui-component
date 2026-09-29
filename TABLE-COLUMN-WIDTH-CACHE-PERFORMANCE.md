# Scroll-table column-width cache: performance and memory report

Measured 2026-09-29. Probes and analysis were prepared with AI assistance.

Publication note: the code branch was subsequently synchronized to upstream `4f861e8a`; all 1,247 Base library tests, Base formatting and lib/tests Clippy passed. The measured source remains unchanged and performance was not rerun on the new parent. See [submission validation](submission-validation/checks.md) and [provenance](PROVENANCE.json).

## Conclusion and acceptance judgment

**On this machine, avoiding repeated measurement in scroll-layout tables produces a modest whole-draw CPU benefit, with an acceptable memory cost for ordinary tables. The initial measurement still scans all cells.**

| Question | Measured evidence | Assessment |
| --- | --- | --- |
| Does the benefit reach the complete draw? | Vertical page scroll with a 48×6 table: 1.625 → 1.541 ms/frame, saving 0.083 ms (5.1%); 12 small tables: 2.441 → 2.304 ms, saving 0.137 ms (5.6%) | More than a column-measurement microbenchmark; headless CPU draw time, not application FPS |
| Does native application FPS improve? | RX 7900 XTX, 170 Hz native window, 200×8 page scroll: 84.525 → 92.797 FPS (+9.8%), 95% change interval [+3.263, +18.331] FPS, higher in 7/8 pairs | Supports a benefit for this machine's large scroll fixture; medium is capped near 170 FPS and large static redraw remains uncertain; application present cadence, not confirmed scanout |
| Does the initial draw improve? | All pooled cold-font intervals include zero; first draw of a newly parsed document with warm fonts: 2.601 → 2.601 ms | No demonstrated initial-draw improvement; does not fix initial blocking on huge input |
| Is ordinary-table memory acceptable? | About 528 B per four-column table at scale; 1,000 tables ≈0.50 MiB; 20,000 ≈10.07 MiB | Small cost for ordinary documents; visible cost with many live tables |
| Does it accumulate with frames? | Ten further measurement rounds show no growth proportional to table count; 1,000 invalidation replacements release previous caches; clone sharing and final drop pass | Supports growth with live objects, not historical frames; no global memory limit |
| Does this guarantee maintainer acceptance? | Small table saves only 17 µs; native FPS now improves for large scrolling; dynamic font installation within a window remains an invalidation concern | Supports a documented tradeoff, not a guaranteed merge |

My assessment: **the ordinary-table memory cost is acceptable and supports a focused repeated-render optimization. Native large-table scrolling now has evidence of an FPS benefit on this machine, but a universal FPS improvement or first-render freeze fix remains unsupported.** The headless medium scroll saving of 0.083 ms is about 0.5% of a 16.67 ms frame budget at 60 Hz; it does not reduce that entire budget by 5.1%. Correctness must hold independently.

## 1. Complete CPU draw time (initial eight process pairs)

Timed region: `Window::draw(cx).clear(cx)`, including production `TextView` rendering, layout, prepaint, scene painting and element-arena cleanup. The probe uses the native Linux text system with GPUI's headless `TestPlatform`. **GPU submission, presentation, vsync, real input dispatch and Markdown parsing are excluded.**

All fixtures except the control enable table `overflow-x: scroll`. The scroll operation changes the outer page's vertical offset; it does not simulate horizontal wheel events or measure native end-to-end scrolling.

| Fixture | Operation | Baseline µs/frame | Patched µs/frame | Saved µs/frame | Reduction | 95% interval for saving, µs | Faster pairs |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Small 8×4 | Redraw | 283.4 | 266.1 | +17.3 | 6.1% | [+13.2, +20.6] | 7/8 |
| Medium 48×6 | Redraw | 1,579.9 | 1,494.1 | +85.9 | 5.4% | [+67.4, +105.8] | 8/8 |
| Medium 48×6 | Vertical page scroll | 1,624.6 | 1,541.2 | +83.4 | 5.1% | [+55.2, +92.4] | 7/8 |
| Large 200×8 | Redraw | 12,772.3 | 12,688.0 | +84.3 | 0.7% | [-1,776.5, +2,222.8] | 3/8 |
| Large 200×8 | Vertical page scroll | 12,887.8 | 11,637.8 | +1,250.0 | 9.7% | [+50.2, +2,478.9] | 8/8 |
| Long text 8×4 | Redraw | 897.3 | 864.1 | +33.2 | 3.7% | [+22.9, +45.7] | 8/8 |
| Long text 8×4 | Vertical page scroll | 918.8 | 890.0 | +28.8 | 3.1% | [+24.7, +34.8] | 8/8 |
| 12 tables of 8×4 | Redraw | 2,389.8 | 2,273.1 | +116.7 | 4.9% | [+99.1, +158.6] | 8/8 |
| 12 tables of 8×4 | Vertical page scroll | 2,441.0 | 2,303.6 | +137.4 | 5.6% | [+110.8, +174.4] | 8/8 |
| Medium 48×6 | Container resize | 1,687.0 | 1,615.8 | +71.2 | 4.2% | [+50.6, +84.2] | 8/8 |
| Medium 48×6 | Alternate font sizes | 4,701.1 | 4,624.9 | +76.2 | 1.6% | [-22.3, +163.5] | 6/8 |
| Medium 48×6 | First draw of newly parsed document | 2,600.5 | 2,601.2 | -0.7 | ≈0% | [-135.8, +101.0] | 4/8 |
| Wrap layout 48×6 (control) | Redraw | 1,982.7 | 1,962.4 | +20.2 | 1.0% | [+5.0, +25.1] | 8/8 |

Positive savings mean faster patched draws. Each process contributes its median of 80 timed frames; the table reports the median of eight process medians. The interval is a paired percentile bootstrap over eight process pairs (10,000 resamples, seed 534). It is a descriptive small-sample estimate without multiple-comparison correction. The 80 frames are not treated as 80 independent processes. Faster-pair counts and the difference of two medians are distinct statistics.

- Small, medium, long-text and multi-table redraw/scroll reductions are about 3%–6%, or 17–137 µs per frame.
- **The initial 200×8 static-redraw batch is inconclusive** (follow-up in 1a): the interval crosses zero and only 3/8 pairs improve; process medians vary roughly from 9.7 to 15.5 ms. Large scrolling improves in 8/8 pairs but its saving interval is broad, 0.05–2.48 ms. The 1.25 ms saving is this sample's point estimate.
- Font-size invalidation and first draw of newly parsed documents show neither a clear improvement nor a detected stable regression. This is not a strict equivalence result.
- The wrap control has no column-cache hit path, yet differs by about 1%. That is not a cache benefit; process variation and binary layout are possible influences. Gains of a few percent warrant cautious extrapolation.
- A/B compares **the complete patch**, including entering the scroll branch earlier and avoiding the unnecessary `text_len` scan. It does not separate that change from cache savings.

## 1a. Larger-table follow-up requested by the user

For the previously inconclusive 200×8 fixture, collect **24 additional independent pairs each** for static redraw and vertical scroll, with 20 warmup and **240 measured frames per process**: **96 processes and 23,040 timed frames**. Original frozen binary hashes were verified. CPU 4, performance governor and allocator settings are unchanged; ordering alternates A/B and B/A, with no builds or other benchmarks running concurrently.

| Operation (200×8) | Baseline ms/frame | Patched ms/frame | Saved ms/frame | Reduction | 95% saving interval, ms | Faster pairs | Baseline → patched P95 ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Repeated static redraw | 10.311 | 9.453 | 0.858 | 8.3% | [0.541, 1.135] | 24/24 | 12.692 → 11.753 |
| Vertical page scroll | 10.097 | 9.520 | 0.577 | 5.7% | [0.406, 0.835] | 23/24 | 12.335 → 11.854 |

**This larger sample supports a static-redraw benefit:** about 0.858 ms/frame (8.3%), faster in 24/24 pairs, with an interval excluding zero. P95 also falls from 12.692 to 11.753 ms. Vertical scroll saves about 0.577 ms/frame (5.7%), faster in 23/24 pairs.

Variation checks: static-redraw process medians range from 9.47–11.30 ms for baseline and 8.79–10.36 ms for patched. First/last-quarter medians are 10.19/10.31 ms and 9.44/9.37 ms, respectively. These do not show a large warmup drift explaining the previous batch's spread. Absolute times differ between batches; **the specific cause of the earlier variation has not been identified**. Scheduling, frequency and other environmental effects are not ruled out. The original eight-pair results remain above; different-length batches are not pooled, and the new scroll batch's one unfavorable pair is retained.

Static redraw here continuously requests draws of unchanged content; **it is not idle-application FPS**. These remain headless CPU measurements; native-window testing is described below.

Evidence: [follow-up plan](table-column-width-cache-evidence/FOLLOWUP-PLAN.md), [all new samples](table-column-width-cache-evidence/followup-large/large.jsonl), [statistics and per-process diagnostics](table-column-width-cache-evidence/followup-large/summary.json), [binary/data verification](table-column-width-cache-evidence/followup-large/manifest.json).

## 1b. Native application present FPS: hardware GPU measurements completed

After the user enabled approval-based escalation, native windows ran successfully. The earlier Wayland `EPERM` / `NoCompositor` failures describe the previous permission state and are resolved. These samples use an actual GPUI native window and RX 7900 XTX renderer. The application is a standalone Base fixture with production `TextView`, rather than the full Story Gallery. Frame callbacks drive scrolling; real wheel-input latency is not measured.

| Configuration | Recorded setting |
| --- | --- |
| Desktop and GPU | Linux Wayland; AMD Radeon RX 7900 XTX, RADV Mesa 26.2.1-arch1.1; hardware rendering in every valid process |
| Display | DP-3, 3840×2160 @170 Hz, scale 1.75; matching KWin configuration has `vrrPolicy: Never`; display settings unchanged |
| Native window | 800×600 logical pixels; DejaVu Sans / Mono, 14 px; normal multicore scheduling, no CPU pinning |
| Sampling | 120 warmup and 600 measured presents per process; eight pairs each for medium scroll, large scroll and wrap control, ten for large static redraw; alternating A/B and B/A |
| Valid samples | 68 processes, 40,800 presents and 40,732 adjacent intervals; focus, hardware rendering, binary hashes and final A/B geometry verified |

### FPS and per-frame CPU cost

FPS is `(present count − 1) / (last present_end − first present_end)`, not inverse draw duration. It measures **native application submission cadence**, not compositor-confirmed physical scanout. FPS is calculated per process, then summarized by the median. Intervals use 10,000 paired process bootstrap resamples (seed 534), without multiple-comparison correction.

| Scenario | Baseline FPS | Patched FPS | Change, FPS | 95% change interval | Higher-FPS pairs |
| --- | ---: | ---: | ---: | ---: | ---: |
| 48×6 vertical page scroll | 169.986 | 169.993 | +0.007 | [-0.049, +0.045] | 3/8 |
| 200×8 static redraw | 87.146 | 88.058 | +0.912 | [-10.962, +19.994] | 7/10 |
| 200×8 vertical page scroll | 84.525 | 92.797 | +8.272 | [+3.263, +18.331] | 7/8 |
| 48×6 wrap control | 169.977 | 170.000 | +0.023 | [-0.016, +0.233] | 5/8 |

| Scenario | Baseline → patched draw ms | Saved ms | Reduction | 95% saving interval, ms | Faster pairs |
| --- | ---: | ---: | ---: | ---: | ---: |
| 48×6 vertical page scroll | 1.577 → 1.535 | +0.042 | 2.7% | [-0.008, +0.078] | 7/8 |
| 200×8 static redraw | 9.661 → 9.296 | +0.365 | 3.8% | [-2.673, +4.222] | 7/10 |
| 200×8 vertical page scroll | 9.754 → 8.836 | +0.918 | 9.4% | [+0.533, +4.149] | 7/8 |
| 48×6 wrap control | 2.032 → 2.043 | -0.011 | -0.5% | [-0.020, +0.031] | 3/8 |

Native draw timing comes from GPUI profiler and excludes arena cleanup performed after draw by the headless probe. Scaling also differs, so absolute CPU timings from the two probes should not be equated.

### Frame intervals and tail timings

| Scenario | Draw P95 / P99 ms (baseline → patched) | Present interval P50 / P95 / P99 ms (baseline → patched) | Long intervals (baseline → patched) | Submit P50 ms (baseline → patched) |
| --- | --- | --- | ---: | ---: |
| 48×6 vertical page scroll | 2.109 → 2.259 / 2.729 → 2.764 | 5.875 → 5.877 / 6.823 → 6.905 / 7.215 → 7.338 | 0.00% → 0.00% | 0.395 → 0.396 |
| 200×8 static redraw | 12.135 → 13.407 / 15.314 → 14.457 | 10.936 → 10.691 / 15.612 → 16.274 / 18.258 → 18.622 | 100.00% → 94.99% | 0.476 → 0.469 |
| 200×8 vertical page scroll | 12.201 → 11.011 / 14.571 → 13.841 | 11.284 → 10.306 / 16.105 → 14.662 / 18.582 → 17.176 | 100.00% → 97.33% | 0.477 → 0.470 |
| 48×6 wrap control | 2.359 → 2.362 / 3.103 → 3.301 | 5.874 → 5.873 / 6.851 → 6.878 / 7.263 → 7.327 | 0.00% → 0.00% | 0.388 → 0.389 |

Quantiles are calculated within each process, then summarized by their median across processes. One 170 Hz period is 5.882 ms; a long interval exceeds 1.5 periods, or 8.824 ms. This is **not a confirmed physical dropped-frame rate**. Groups near 170 FPS are refresh-capped; differences of hundredths of an FPS do not imply a perceptible benefit. Static redraw continuously requests draws of unchanged content and is not idle-application FPS.

### Interpretation and retained samples

- **The 200×8 page-scroll fixture supports a native benefit:** 84.525 → 92.797 FPS (+9.8%), change interval [+3.263, +18.331] FPS; draw saves 0.918 ms/frame (9.4%), interval [0.533, 4.149] ms, faster in 7/8 pairs. Present interval P95/P99 changes from 16.105/18.582 to 14.662/17.176 ms. Tail point estimates improve too, without separate significance tests for these quantiles.
- **Native 200×8 static redraw remains inconclusive:** 87.146 → 88.058 FPS, change interval [-10.962, +19.994] FPS; the draw-saving interval also crosses zero. Process FPS ranges are 57.07–90.33 for baseline and 65.66–96.33 for patched, showing substantial system/process variation whose specific cause is unidentified. Draw P95 and present-interval P95/P99 point estimates worsen, so the median cannot establish generally smoother rendering. The earlier 24-pair headless static-redraw CPU result stands separately and cannot replace this native result.
- **Medium scroll and the wrap control both remain near 170 FPS**, without evidence of an FPS improvement. Medium draw saves an estimated 0.042 ms but its interval crosses zero. Wrap draw is an estimated 0.5% slower, also with an interval crossing zero: neither a stable regression nor strict equivalence is established, and it is not a cache benefit.
- A maintainer-facing claim can therefore be **about +9.8% present FPS and 0.918 ms less CPU draw per frame in this Linux native large-scroll fixture**, accompanied by the capped and inconclusive cases. It is not a guarantee for the full Story Gallery, other hardware, or macOS/Windows.

After eight medium pairs and two large-redraw pairs, one large process recorded 397 inactive samples and was excluded under the predeclared focus rule; its original data remain intact. Eight additional large-redraw pairs were collected and all ten valid pairs are included. No samples were removed for being slower. The short startup smoke check is excluded from formal statistics.

Evidence: [combined valid samples](table-column-width-cache-evidence/native-gpu-results/native.jsonl), [full statistics](table-column-width-cache-evidence/native-gpu-results/summary.json), [input mapping and exclusions](table-column-width-cache-evidence/native-gpu-results/manifest.json), [GPU/display environment](table-column-width-cache-evidence/native-gpu-environment.json), and [native probe](table-column-width-cache-evidence/native_probe.rs). Original batches and reproduction commands are in the [README](table-column-width-cache-evidence/README.md#native-follow-up).

## 2. Initial draw, reported separately

Each fresh process records the document's first draw. Font/glyph caches are also cold; the approximately 65–99 ms below **are not column-measurement timings**. Initial fixtures are identical across the later operation groups, so all first-draw samples are pooled by fixture while preserving each `(mode, run)` A/B pair. Per-operation results remain in `summary.json`, including unfavorable samples.

| Fixture | Process pairs | Baseline ms | Patched ms | Time change | 95% interval for saving, ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| Small 8×4 | 8 | 66.489 | 65.618 | -1.31% | [-0.206, +2.717] |
| Medium 48×6 | 40 | 73.892 | 74.564 | +0.91% | [-1.930, +0.379] |
| Large 200×8 | 16 | 98.928 | 97.190 | -1.76% | [-0.430, +3.318] |
| Long text 8×4 | 16 | 78.173 | 78.629 | +0.58% | [-1.610, +1.842] |
| 12 tables of 8×4 | 16 | 74.399 | 74.754 | +0.48% | [-3.012, +0.791] |
| Wrap layout 48×6 (control) | 8 | 73.946 | 73.849 | -0.13% | [-1.525, +0.589] |

All pooled intervals include zero. The medium fixture's subsequent-scroll subgroup has a first-draw point estimate about 5.4% slower, which does not consistently recur in its other groups. Pooling all 40 identical initial-fixture pairs gives +0.91%, with an interval crossing zero. Selecting one subgroup would not justify an improvement or regression claim.

With warm fonts, repeatedly mounting newly parsed documents gives a first-draw time of **2,600.5 → 2,601.2 µs**, with a saving interval of **[-135.8, +101.0] µs**. Parsing completes before mounting; this is not `set_text`-to-screen latency.

Cache misses, reparsed tables and typography-key changes still scan and measure all ordinary cells. Custom cells are remeasured every time, and all table cells still need layout. This patch does not fully address initial UI blocking on extremely long input.

## 3. Memory: allocations, RSS and lifetime

After warming native fonts and shaped-line caches with the same text, the probe creates and retains fresh tables with one ordinary-text row each. These do not inherit populated column caches. There are eight process pairs per scenario, using the same `MALLOC_ARENA_MAX=1` setting.

Extra live heap is the patched-minus-baseline difference of `(after measurement − before construction)`, including the larger `Table` and allocator size classes. It uses glibc `mallinfo2.uordblks + hblkhd`, **including mmap allocations**. Arena-only accounting would miss large arrays. RSS comes from `/proc/self/status` and is reported separately.

| Retained tables | Columns | Extra live heap KiB | Extra B/table | Extra RSS KiB | Live heap difference after drop, B |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 10 | 4 | 6.13 | 628.00 | 4 | 1,336 |
| 100 | 4 | 51.54 | 527.76 | 36 | 1,696 |
| 1,000 | 4 | 515.65 | 528.02 | 560 | 1,736 |
| 20,000 | 1 | 10,311.91 | 527.97 | 10,332 | 1,376 |
| 20,000 | 4 | 10,312.00 | 527.97 | 10,280 | 1,688 |
| 1,000 | 16 | 562.41 | 575.90 | 548 | 2,728 |

**About 528 B/table is a local allocation result for ordinary four-column tables, not a universal constant.** Fixed overhead affects the ten-table sample; 628 B/table is not the cost at scale. Sixteen columns cost about 576 B/table. One and four columns have similar costs, consistent with allocator size rounding.

### Structures and growth

| Item on this x86_64 build | Size or behavior |
| --- | --- |
| `Table` | 88 → 104 B, +16 B even for unmeasured or wrap-layout tables |
| `TableColumnWidths` | 400 B, including the key and both Vec headers |
| Arc counters | 16 B |
| Ordinary widths | `4 × widths.capacity()` B; measured capacity = 4 for four columns |
| Custom-cell coordinates | `16 × custom_cells.capacity()` B, including spare capacity; memory fixtures above contain no custom cells |
| Direct four-column structures | 432 B cache +16 B Table growth =448 B before allocator rounding and indirect key allocations/retained references |
| Clone | Shares the existing Arc; after invalidation, live clones may independently retain different cache versions |
| Overall growth | With live tables, columns and custom-cell coordinates; no copy of ordinary cell text; no global quota or eviction policy |

### Repetition and release

| Check | Result | Supported conclusion |
| --- | --- | --- |
| Ten more rounds over each table batch | Baseline live heap increases by 368–432 B/process, patched by 400–432 B; both include fixed measurement-record overhead | No observed per-round growth proportional to table count |
| 1,000 typography-key invalidations of one Table | A Weak reference verifies the previous cache has no strong references on every replacement | Old cache versions are released when no external clone retains them |
| Clone/drop | Strong count goes to 2 on clone, returns to 1 after clone drop; Weak cannot upgrade after Table drop | Existing cache sharing and final release behave as expected |
| Drop 20,000 four-column tables | Extra live heap falls from ≈10.07 MiB to 1,688 B; extra RSS remains ≈9,972 KiB | Objects are freed, but glibc may retain pages; immediate RSS recovery is not guaranteed |

The post-drop live-heap difference also includes JSON records retained by the probe; the residual is not a byte-for-byte cache-leak measurement.

The cost is **accountable and acceptable within these ordinary-table workloads**, not automatically capped. Many custom cells, complex font-feature keys and long-lived document clones are outside the ordinary-table average's guarantee. No long-running real-application RSS churn test was performed.

## 4. Reproduction and evidence integrity

| Item | Configuration or result |
| --- | --- |
| Baseline | `11b04d9bf09cec68c98ce97d759afc6776b1cda0` |
| Patched input | Frozen `patched-node.rs`, SHA-256 `c30a9d84ddcb09a02cd21e035da5a105e918820a8ba9153018ff1fd7faa16a3d` |
| Patch SHA-256 | `09a13f940f9a38b9c79ac299b6a32c8004b0cce74b7f57c6073f4df44276f4e8` |
| CPU/memory probe platform | Linux x86_64, AMD Ryzen 9 7950X, pinned to logical CPU 4; native FPS multicore/GPU settings are in 1b |
| Build | Rust 1.98.0 / LLVM 22.1.8; GPUI `gpui-pre` 0.3.7; Cargo release library tests; identical lockfile |
| Fonts | Native Linux CosmicTextSystem; explicit DejaVu Sans / DejaVu Sans Mono; local fallback for CJK, without tracing each glyph's actual font |
| Window | Headless GPUI TestApp; 800×600 container; resize alternates 640/960, font changes alternate 14/18 px |
| Fixtures | Small 916 B; medium 8,158 B; large 42,346 B; long 42,388 B; 12 small tables 11,010 B; row counts exclude headers |
| Sampling | 13 scenarios ×8 pairs ×2 variants =208 processes; 20 warmup +80 timed frames/process =16,640 timed frames; alternating A/B and B/A |
| Memory | 6 scenarios ×8 pairs ×2 variants =96 processes, plus one lifetime-check process |
| Timing guard | Settle an empty window first; parse detached, then mount and draw within one App update; assert exactly one additional root render per timed draw, preventing TestApp automatic pre-draws from warming initial samples |
| Geometry | All A/B initial document bounds and page scroll_max match exactly; not a pixel comparison or per-frame comparison of every cell |
| Regression checks | Five `table_column` tests pass in the measured patched release binary: ordinary widths, clone, typography, reparsing, inline code and growing/shrinking custom widths |
| Product changes | This work adds evidence and reports while preserving the user's node.rs patch; no product timers or benchmark feature |

Start with [README](table-column-width-cache-evidence/README.md), [plan](table-column-width-cache-evidence/PLAN.md), [exact measured probe](table-column-width-cache-evidence/frame_probe.rs) and [analysis](table-column-width-cache-evidence/analyze.py). Raw data: [frames](table-column-width-cache-evidence/frames.jsonl), [memory](table-column-width-cache-evidence/memory.jsonl), [lifecycle](table-column-width-cache-evidence/lifecycle.jsonl); aggregate [summary](table-column-width-cache-evidence/summary.json). Input and binary hashes: [source-metadata](table-column-width-cache-evidence/source-metadata.json), [measurement-manifest](table-column-width-cache-evidence/measurement-manifest.json). Regression output: [validation-tests](table-column-width-cache-evidence/validation-tests.log). The probe preserves the source exactly as measured.

Tests ran in an isolated source archive, without modifying Base tests in the active checkout. Complete-draw heap/RSS snapshots are also retained, but their net changes include other GPUI caches and are not presented as the column cache's own allocation cost. Linux native-window/GPU present FPS measurement is complete (1b); macOS/Windows, the full Story Gallery and physical scanout FPS remain unmeasured.

## 5. Maintainer-facing tradeoff

Suggested claim: “Avoid remeasuring ordinary-cell column widths on repeated renders of horizontal-scroll-layout tables. A Linux Wayland / RX 7900 XTX native 200×8 page-scroll fixture improves from 84.5 to 92.8 present FPS and saves about 0.918 ms CPU draw per frame (eight process pairs). The separate memory probe measures about 528 B extra per ordinary four-column table. Medium is capped at 170 Hz, native large static redraw remains uncertain, and initial measurement still scans all cells.”

A maintainer previously rejected another cache because the original hotspot was around 50 µs, invalidation added complexity and some workloads regressed. This complete-draw and lifetime evidence is stronger, but absolute small-table gains may still fall below their acceptance threshold. [Historical discussion #2462](https://github.com/longbridge/gpui-kit/pull/2462#issuecomment-4705001336)

**Remaining correctness boundary:** the key checks window text-system identity, TextStyle, rem size, mono family and inline-code style, but has no generation for font installation/replacement within the same window. This work neither proves that boundary safe nor changes it. Performance and memory evidence cannot replace confirmation of that invalidation design.

## Appendix: previous isolated-function measurements

The earlier default test-profile measurements used `WideMonoTextSystem` and only timed column measurement. Their speedup factors cannot be applied to this native-font release whole-draw benchmark. They lack this round's complete reproduction materials and are not the main acceptance evidence.

| Fixture | Metric | Baseline | Patched | Earlier observation |
| --- | --- | ---: | ---: | --- |
| 96×4, 1,024 characters/cell | First measurement | 685.5 µs | 731.0 µs | +6.6%, sample ranges overlap |
| Same | Next 12 measurements combined | 5,355 µs | 18 µs | Function only, about 297.5× |
| Same | Peak RSS | 31,368 KiB | 32,146 KiB | +778 KiB, not exact cache size |
| 2×2, 120,000 characters/cell | First measurement | 3,400.5 µs | 3,500.5 µs | +2.9%, sample ranges overlap |
| Same | Next 8 measurements combined | 1,220.5 µs | 20.5 µs | Function only, about 59.5× |
| Same | Peak RSS | 35,170 KiB | 35,960 KiB | +790 KiB, not exact cache size |
| 20,000 single-column tables | RSS increase during measurement | 2,788 KiB | 13,456 KiB | Difference 10,668 KiB, about 546 B/table |

Correction to the previous interpretation: the final row starts after table construction, before measurement. Its **546 B/table cannot be said to include the additional 16 B Table field**, and RSS does not precisely decompose cache allocations. The new primary memory evidence uses glibc live allocations including table construction.

The earlier report recorded 1,245 Base library tests, Base lib/tests Clippy and formatting as passing. Those complete suites were not rerun in this round. This round adds the five regression tests, complete-draw/memory/lifetime measurements, Linux native GPU present FPS, reproduction-input checks and `git diff --check`. The earlier criterion 0.5/0.8 conflict in `Clippy --all-targets` does not affect this measurement approach.
