# #389 CPU comparison

Collector: none; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| cache-glyph-fill-2048-64-50-grouped-triangle-gradient-true-389-moving / moving / sentence / 1 | 97623.45 | 96921.22 | -0.56% | -0.93% to -0.45% |
| mixed-fill-1024-16-50-shuffled-triangle-solid-true-389-stationary / stationary / sentence / 1 | 17558.63 | 17411.22 | -0.91% | -1.22% to -0.62% |
| tiny-path-fill-512-16-50-grouped-rectangle-solid-true-389-stationary / stationary / sentence / 1 | 4482.17 | 4479.09 | -0.91% | -1.32% to +0.76% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
