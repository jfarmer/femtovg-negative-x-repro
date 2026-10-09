# #389 CPU comparison

Collector: perf; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| cache-glyph-fill-2048-64-50-grouped-triangle-gradient-true-389-moving / moving / sentence / 1 | 97809.88 | 97088.23 | -1.05% | -1.27% to -0.74% |
| mixed-fill-1024-16-50-shuffled-triangle-solid-true-389-stationary / stationary / sentence / 1 | 18169.39 | 17829.75 | -0.60% | -2.23% to +0.59% |
| mixed-stroke-1024-16-10-shuffled-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 8334.67 | 8224.51 | -1.43% | -1.65% to -1.32% |
| tiny-path-fill-512-16-50-grouped-rectangle-solid-true-389-stationary / stationary / sentence / 1 | 4654.57 | 4579.21 | -2.09% | -4.95% to +4.47% |
| tiny-path-stroke-64-16-50-grouped-rectangle-solid-false-389-stationary / stationary / sentence / 1 | 277.32 | 282.26 | +1.97% | -2.60% to +3.04% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
