# #389 CPU comparison

Collector: perf; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| cache-glyph-fill-2048-64-50-grouped-triangle-gradient-true-389-moving / moving / sentence / 1 | 98891.35 | 96675.03 | -1.09% | -1.98% to -1.04% |
| mixed-fill-1024-16-50-shuffled-triangle-solid-true-389-stationary / stationary / sentence / 1 | 17637.02 | 17721.22 | +0.05% | -0.26% to +0.79% |
| mixed-stroke-1024-16-10-shuffled-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 8255.23 | 8204.56 | -0.97% | -2.01% to +0.00% |
| tiny-path-fill-512-16-50-grouped-rectangle-solid-true-389-stationary / stationary / sentence / 1 | 4597.47 | 4573.44 | -0.98% | -1.79% to -0.58% |
| tiny-path-stroke-64-16-50-grouped-rectangle-solid-false-389-stationary / stationary / sentence / 1 | 277.66 | 282.78 | +1.11% | +0.41% to +2.47% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
