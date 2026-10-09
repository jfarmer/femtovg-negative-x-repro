# #389 CPU comparison

Collector: none; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| cache-one-moving / moving / sentence / 1 | 11984.79 | 11758.86 | -2.52% | -4.47% to +3.33% |
| cache-one-stationary / stationary / sentence / 1 | 12644.57 | 11662.48 | -7.77% | -11.38% to -3.48% |
| mixed-fill-false / stationary / sentence / 1 | 12307.53 | 10423.34 | -15.31% | -15.43% to -0.86% |
| mixed-fill-true / stationary / sentence / 1 | 10959.12 | 12152.96 | +11.56% | -1.67% to +12.14% |
| mixed-stroke-false / stationary / sentence / 1 | 8849.35 | 8642.84 | -2.33% | -7.00% to -0.23% |
| mixed-stroke-true / stationary / sentence / 1 | 8734.83 | 8649.35 | -2.18% | -6.89% to +2.84% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
