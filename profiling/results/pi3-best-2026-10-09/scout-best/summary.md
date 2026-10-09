# #389 CPU comparison

Collector: none; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| mostly-offscreen-fill-batched / stationary / sentence / 1 | 1112.40 | 937.04 | -14.86% | -17.00% to -13.31% |
| mostly-offscreen-fill-grid / stationary / sentence / 1 | 2538.19 | 2555.10 | +1.05% | -3.72% to +1.54% |
| offscreen-fill-batched / stationary / sentence / 1 | 679.75 | 552.74 | -18.69% | -24.15% to -18.01% |
| rotated-labels-fill / stationary / sentence / 1 | 19648.67 | 21532.72 | +9.59% | +3.77% to +11.91% |
| rotated-labels-stroke / stationary / sentence / 1 | 16655.95 | 16169.62 | -0.47% | -1.69% to +1.99% |
| small-labels-atlas-control / stationary / sentence / 1 | 483.39 | 528.12 | +9.38% | +4.47% to +12.25% |
| visible-fill-batched / stationary / sentence / 1 | 4368.00 | 4406.84 | +1.24% | -8.14% to +5.73% |
| visible-fill-batched-aa / stationary / sentence / 1 | 4492.48 | 4349.87 | +0.37% | -6.97% to +3.68% |
| visible-fill-grid / stationary / sentence / 1 | 5325.31 | 5230.07 | -2.31% | -2.87% to +4.04% |
| visible-fill-unbatched / stationary / sentence / 1 | 10613.24 | 10556.55 | +0.41% | -6.29% to +7.29% |
| visible-stroke-batched / stationary / sentence / 1 | 2916.21 | 2489.77 | -12.48% | -15.50% to -10.55% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
