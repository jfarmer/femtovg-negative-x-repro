# #389 CPU comparison

Collector: none; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| cache-glyph-fill-2048-64-50-grouped-triangle-gradient-true-389-moving / moving / sentence / 1 | 96305.47 | 95894.15 | -0.43% | -0.52% to -0.03% |
| mixed-fill-1024-16-50-shuffled-triangle-solid-true-389-stationary / stationary / sentence / 1 | 17671.12 | 17524.77 | +0.74% | -3.31% to +0.78% |
| mixed-stroke-1024-16-10-shuffled-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 8208.42 | 8168.50 | -0.33% | -1.33% to -0.16% |
| tiny-path-fill-512-16-50-grouped-rectangle-solid-true-389-stationary / stationary / sentence / 1 | 4511.34 | 4532.76 | -1.14% | -3.04% to +4.24% |
| tiny-path-stroke-64-16-50-grouped-rectangle-solid-false-389-stationary / stationary / sentence / 1 | 280.26 | 278.81 | +0.71% | -2.38% to +1.79% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
