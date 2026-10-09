# #389 CPU comparison

Collector: perf; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| glyph-fill / moving / sentence / 1 | 1272.72 | 1262.79 | -0.77% | -1.11% to -0.69% |
| glyph-fill / stationary / sentence / 1 | 998.15 | 981.52 | -0.49% | -3.05% to +0.68% |
| glyph-stroke / moving / sentence / 1 | 979.34 | 955.27 | -1.54% | -2.74% to -1.43% |
| glyph-stroke / stationary / sentence / 1 | 592.68 | 580.62 | -2.03% | -2.52% to -1.70% |
| gradient-fill / moving / sentence / 1 | 1288.53 | 1282.10 | -0.58% | -1.03% to -0.34% |
| gradient-fill / stationary / sentence / 1 | 1027.68 | 1012.66 | -1.40% | -1.91% to -1.23% |
| gradient-stroke / moving / sentence / 1 | 988.01 | 963.21 | -1.69% | -2.18% to -0.84% |
| gradient-stroke / stationary / sentence / 1 | 605.89 | 595.92 | -1.71% | -1.75% to -1.65% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
