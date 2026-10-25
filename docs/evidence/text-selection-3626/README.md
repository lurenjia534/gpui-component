# Issue 3626 validation evidence

The fix separates text-selection registration from frame publication and keeps unrelated GPUI Fast views retained. This evidence accompanies code commit `9b42fb299a50610d83c5fb484cf4db390871ac28` against baseline `7b14e03f2ed4b46fc4149a8a523c7f91bcad6560`. It is stored on a separate fork branch; none of these files are part of the code PR.

- [Test results and full-suite failure comparison](3626-test-results.en.md)
- [Performance methodology, raw samples and before/after GPU captures](performance/index.en.md)
- [Line-by-line review and actual desktop-window acceptance screenshots](native-review/index.en.md)
- [中文测试结果](3626-test-results.md) · [中文性能对照](performance/index.md) · [中文审查和桌面截图](native-review/index.md)

The 164-TextView workload measures median CPU input/draw/deferred processing at 51.94 ms before and 2.58 ms after. Median unrelated view renders per pointer step fall from 324 to zero. These are Linux dev-build CPU measurements, not display-presented frame latency or FPS. Each version/size has 180 samples across three trials. One pointer step can run multiple draws.

The before/after performance PNGs are real GPU offscreen captures and are byte-identical. Images under `native-review` are separate Spectacle captures of real desktop application windows. Physical touchscreen input was not verified: touch-selection UI was exercised with the demo Long press button and native pointer dragging. An EIS touch long press did not produce a confirmed selection. The first XWayland wheel attempt did not visibly scroll; the subsequent native Wayland scrolling check did, with unchanged clipboard text.

The complete Base Fast suite has 35 failures before and 23 after; all 23 after-fix failure names also occur in the baseline run. No new failing names were observed. Full-suite success is not claimed. macOS and Windows were not run. No extra production fixes were made during review.

## Reproduce the bounded regression

Run in a clone of the fork containing this evidence and the fix commit:

```sh
git fetch origin fix/3626-text-selection-frame-publication
python3 docs/evidence/text-selection-3626/reproduce.py --output /tmp/gpui-3626-review
# Optional real-font/GPU measurements and captures, with a new output directory:
python3 docs/evidence/text-selection-3626/reproduce.py --output /tmp/gpui-3626-measured --measure
```

The script preserves clean production sources in two detached worktrees, transplants only the common retained-view regression into the baseline, expects its failure before the fix and success after it, and optionally runs the archived performance fixture. It keeps separate build directories to avoid shared Cargo artifact confusion. Worktrees and build outputs remain available for inspection. Native build dependencies and a real headless GPU renderer are required for `--measure`.

The reproduction script was syntax-checked during publication; its complete rebuild was not rerun. The archived test runs and captured performance/native results are the validation evidence. Local absolute paths in the reports describe the original measurement checkout; they are not prerequisites for the portable reproduction script. The native acceptance fixture is preserved as originally exercised, including its local output path.

AI assisted with patch analysis, code review, bounded regression and performance validation, native-window acceptance, bilingual evidence and PR preparation. The production fix and added regressions originated in the supplied patch. Human maintainer review remains required.
