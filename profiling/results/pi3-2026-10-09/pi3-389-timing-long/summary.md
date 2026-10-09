# #389 CPU comparison

Collector: none; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| glyph-fill / moving / sentence / 1 | 1264.22 | 1254.27 | -0.95% | -1.50% to -0.33% |
| glyph-fill / stationary / sentence / 1 | 985.19 | 978.00 | -0.84% | -1.13% to -0.23% |
| glyph-stroke / moving / sentence / 1 | 954.63 | 935.21 | -1.41% | -2.68% to -0.65% |
| glyph-stroke / stationary / sentence / 1 | 585.89 | 575.70 | -2.20% | -2.37% to -1.71% |
| gradient-fill / moving / sentence / 1 | 1276.33 | 1264.15 | -0.87% | -1.56% to -0.60% |
| gradient-fill / stationary / sentence / 1 | 1008.84 | 987.68 | -2.10% | -2.40% to -0.97% |
| gradient-stroke / moving / sentence / 1 | 963.87 | 956.38 | -0.62% | -2.32% to -0.04% |
| gradient-stroke / stationary / sentence / 1 | 598.29 | 588.76 | -1.53% | -1.77% to -1.14% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
