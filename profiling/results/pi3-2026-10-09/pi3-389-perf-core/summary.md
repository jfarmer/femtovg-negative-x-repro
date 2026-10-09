# #389 CPU comparison

Collector: perf; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| glyph-fill / moving / sentence / 1 | 1277.09 | 1265.71 | -1.09% | -1.70% to -0.89% |
| glyph-fill / stationary / sentence / 1 | 1006.04 | 992.96 | -1.04% | -1.43% to -0.64% |
| glyph-stroke / moving / sentence / 1 | 949.83 | 961.94 | -0.94% | -2.06% to +0.40% |
| glyph-stroke / stationary / sentence / 1 | 587.91 | 582.22 | -1.33% | -2.14% to -0.74% |
| gradient-fill / moving / sentence / 1 | 1287.72 | 1281.53 | -0.61% | -1.23% to -0.35% |
| gradient-fill / stationary / sentence / 1 | 1002.76 | 984.47 | -0.78% | -1.29% to -0.17% |
| gradient-stroke / moving / sentence / 1 | 963.40 | 971.63 | -1.52% | -1.95% to +1.16% |
| gradient-stroke / stationary / sentence / 1 | 606.41 | 595.04 | -2.15% | -2.33% to -1.33% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
