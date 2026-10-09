# #389 CPU comparison

Collector: none; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| cache-glyph-fill-2048-64-50-grouped-triangle-gradient-true-389-moving / moving / sentence / 1 | 96540.54 | 96777.11 | -0.34% | -1.38% to +1.13% |
| cache-glyph-fill-2048-64-50-shuffled-triangle-gradient-true-389-moving / moving / sentence / 1 | 94616.92 | 93413.35 | -1.16% | -2.02% to -0.69% |
| mixed-fill-1024-16-50-grouped-triangle-solid-true-389-stationary / stationary / sentence / 1 | 17442.49 | 16855.44 | -4.19% | -4.49% to -2.32% |
| mixed-fill-1024-16-50-shuffled-triangle-solid-true-389-stationary / stationary / sentence / 1 | 17756.78 | 17596.20 | -0.07% | -3.25% to +0.92% |
| mixed-stroke-1024-16-10-grouped-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 8079.35 | 7922.32 | -1.94% | -3.52% to +1.66% |
| mixed-stroke-1024-16-10-shuffled-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 8200.94 | 8146.18 | -0.69% | -1.56% to -0.55% |
| tiny-path-fill-512-16-50-grouped-rectangle-solid-true-389-stationary / stationary / sentence / 1 | 4461.62 | 4518.34 | +1.56% | -1.27% to +1.95% |
| tiny-path-fill-64-16-50-grouped-rectangle-solid-false-389-stationary / stationary / sentence / 1 | 520.56 | 481.42 | -6.64% | -10.61% to +8.70% |
| tiny-path-stroke-512-16-50-grouped-rectangle-solid-false-389-stationary / stationary / sentence / 1 | 2437.95 | 2426.24 | +0.04% | -1.16% to +0.40% |
| tiny-path-stroke-64-16-50-grouped-rectangle-solid-false-389-stationary / stationary / sentence / 1 | 279.50 | 278.56 | +1.19% | -2.96% to +2.34% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
