# Follow-up measurements

Defined before collecting new samples; the original data and source snapshots
remain unchanged. AI assisted with the probes and analysis.

## Large table

- Reuse the exact original baseline and patched release binaries and 200×8
  fixture. Keep CPU 4 affinity and `MALLOC_ARENA_MAX=1`.
- Increase to 24 independent A/B pairs for redraw and another 24 for vertical
  scroll; 20 warmup frames followed by 240 timed frames per process. Alternate
  A/B and B/A. Do not run builds or another benchmark while sampling.
- Report process medians, P95, paired uncertainty and faster-pair counts. Retain
  every sample. Inspect within-process first/last quarters and process-to-process
  spread to distinguish drift from outliers, without deleting slow samples.
- If uncertainty remains, report it. More sampling cannot guarantee a positive
  result and a null result is a valid outcome.

## Native application

- Build a minimal Base-only native window using the same fixture, fonts and
  800×600 viewport. Drive sustained redraw or vertical scrolling from frame
  callbacks. Use GPUI's profiler Draw and Present events; compute present FPS
  from `(count - 1) / (last present_end - first present_end)`, never inverse draw
  duration. Also report draw and present-interval P50/P95/P99 and long intervals.
- Warm up for 120 presents, then record 600 presents per run, with at least
  eight alternating A/B process pairs on the same active, unobscured window and
  display. Record refresh rate, scale, renderer and GPU. Keep the monitor setting
  constant; refresh-capped FPS can stay unchanged while CPU time improves.
- GPUI present completion measures application submission cadence, not a
  compositor's hardware scanout confirmation. State that distinction. This
  session currently cannot connect to Wayland (EPERM), has no `/dev/dri`, and
  Vulkan detects no GPU. Do not substitute synthetic FPS for unavailable data.

## Execution note, 2026-09-29

The user enabled approval-based escalation after the initial desktop failure.
Native startup then succeeded on Wayland with an RX 7900 XTX (RADV), using the
existing 3840×2160 @170 Hz display at 1.75 scale. The matching KWin configuration
has VRR policy `Never`; no display setting was changed. Native runs use normal
multicore scheduling and `MALLOC_ARENA_MAX=1`.

The first native suite completed eight medium-scroll pairs and two large-redraw
pairs before one large-redraw process lost focus (397 inactive samples). Its
original record is retained and excluded according to the predeclared focus
rule. Restart the large-redraw case for eight pairs in a fresh directory, and
run the remaining large-scroll and wrap-control cases separately. Include all
completed valid pairs from both attempts: ten large-redraw pairs and eight for
each other case. Do not select runs by measured performance. Preserve each raw
record's source path, line and original pair number in the combined evidence.
