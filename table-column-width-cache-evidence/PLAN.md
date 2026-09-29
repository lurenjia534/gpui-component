# Measurement plan

Frozen before reading the new timing results. AI assisted in preparing the probes.

- Compare exact HEAD against the working-tree `node.rs` patch, in the same isolated checkout and Cargo release profile, using the same lockfile and Linux platform text system. No runtime feature toggles choose algorithms. Two executable files are retained.
- Time complete `Window::draw` plus arena cleanup, with production `TextView` parsing and rendering. Parsing settles before timing: replacement mounts a freshly parsed TextView state, then times its first draw, excluding parse latency. GPU submission, presentation, vsync, and real input dispatch are outside this measurement.
- Small/medium/large tables contain 8×4, 48×6 and 200×8 data cells; headers are additional. Cells contain distinct English, CJK, bold and inline code. Long cells repeat mixed text; the multi-table case contains 12 small tables. An unchanged wrap-mode case is a control.
- Compare redraw, vertical scroll, container resizing, typography changes and document replacement. Each process records its cold first draw, 20 warm-up frames, and 80 timed frames. Alternate A/B and B/A across eight process pairs per scenario, pinning to one CPU. Report paired differences and uncertainty rather than selecting the fastest runs.
- Memory: record process RSS and glibc allocated heap before/after rendering, and after removing the document. Scale retained tables from 10 through 20,000; vary column counts; compare 10 repeated measurements. Separately account for exact cache payload, shared ownership on clone, 1,000 replacements after typography invalidation, and final drop.

Decision rubric

- Repeated draw improvements must be stable across paired runs. Both absolute microseconds and percentage matter; stress-only gains do not establish benefits for ordinary tables.
- Cold/invalidation results are reported even if they regress. Any reproducible regression above 5% requires an explicit tradeoff assessment, not omission.
- Additional live memory should scale with live tables, column count and custom-cell coordinates, not elapsed frames. A replaced cache must be released unless a live clone still owns it.
