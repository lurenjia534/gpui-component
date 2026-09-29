# Table column-width cache: brief report

Date: 2026-09-29 · [Full data and reproduction materials](TABLE-COLUMN-WIDTH-CACHE-PERFORMANCE.md)

## My assessment

**The benefit justifies the cost for large tables that are repeatedly drawn or scrolled. The value is limited for small tables or tables shown only once.** Ordinary-table memory cost is reasonable, but font-change invalidation needs to be resolved before submission.

## Benefits and costs

| Item | Measurement | Interpretation |
| --- | --- | --- |
| 200×8 large-table scroll | Native present FPS **84.53 → 92.80 (+9.8%)**; **0.918 ms less CPU draw per frame**; 7/8 pairs improve | Evidence of a benefit; the 95% interval for the FPS increase is +3.26 to +18.33 |
| 200×8 large-table static redraw | Native FPS 87.15 → 88.06, with an interval crossing zero; a separate 24-pair headless CPU follow-up saves 0.858 ms/frame | The headless follow-up supports CPU savings; native FPS remains inconclusive |
| 48×6 table scroll | Both variants run near 170 FPS | Refresh-capped, without evidence of an FPS increase |
| Initial draw | No clear improvement; a full scan is still required | Does not resolve initial blocking on huge tables |
| Ordinary-table memory | About **528 B extra per four-column table**; 1,000 tables ≈0.50 MiB; 20,000 ≈10.07 MiB | Measured by a separate memory probe; not the exact cost of the eight-column fixture above |
| Additional work per frame | Cache-key comparisons, locking and copying column widths | Reported net savings already include these costs |

**Benefits and costs are not strictly proportional; more reuse makes the tradeoff more favorable.** For ordinary text tables, cache storage grows mainly with column count and live table count. Avoided repeated measurement depends on cell count, text volume and redraw count. Existing tests did not observe accumulation with historical frame count.

Native measurements used Linux Wayland, an RX 7900 XTX and a 170 Hz display, with 68 valid processes and 40,800 presents. The application is a standalone native `TextView` fixture. FPS comes from actual application submission timestamps; the full Story Gallery and physical scanout FPS were not measured.

## Two unresolved issues

### 1. Dynamic font changes may not invalidate cached widths

**This is a correctness risk and the higher priority.** Loading or replacing fonts within the same window may change actual text widths while the text-system object, font name and style remain unchanged. The cache key has no font-generation field and may therefore reuse stale widths.

This boundary has neither been established as safe nor fixed. Before submission, confirm whether the framework's font-update mechanism invalidates the cache. If it does not, add invalidation handling and focused verification.

### 2. No global memory limit; complex workloads remain insufficiently tested

**This is a cost boundary and a verification gap.** The 528 B figure applies only to the measured ordinary four-column tables. Many custom cells require additional stored coordinates, and long-lived clones may retain different cache versions.

Existing tests support release of old caches and no continued growth proportional to table count during repeated measurement. However, long-running native-application memory stress testing has not been performed. The allocator may retain RSS after objects are released, so immediate reduction in process memory cannot be promised.

## Is further work worthwhile?

**Yes: repeated-scroll benefits have evidence, and ordinary-table memory cost is acceptable.** Address font invalidation first and state the scope of the memory findings. Lead the submission with avoiding repeated measurement of scroll-layout tables, retaining the unchanged initial-draw, refresh-capped and inconclusive static-redraw results.
