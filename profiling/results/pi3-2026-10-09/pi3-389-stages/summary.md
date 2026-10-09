# #389 CPU comparison

Collector: none; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| glyph-fill / moving / sentence / 1 | 1300.98 | 1240.95 | -3.48% | -4.61% to -1.35% |
| glyph-fill / stationary / sentence / 1 | 1026.17 | 929.98 | -9.57% | -10.06% to +9.21% |
| glyph-stroke / moving / sentence / 1 | 917.06 | 970.36 | +7.76% | -7.95% to +8.64% |
| glyph-stroke / stationary / sentence / 1 | 593.70 | 554.23 | -6.65% | -10.55% to -2.44% |
| gradient-fill / moving / sentence / 1 | 1294.71 | 1310.18 | +1.63% | -1.82% to +6.57% |
| gradient-fill / stationary / sentence / 1 | 962.95 | 945.08 | -1.86% | -10.58% to +7.00% |
| gradient-stroke / moving / sentence / 1 | 925.18 | 936.97 | +2.92% | -10.94% to +4.87% |
| gradient-stroke / stationary / sentence / 1 | 585.13 | 576.71 | -1.40% | -1.43% to +3.12% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
