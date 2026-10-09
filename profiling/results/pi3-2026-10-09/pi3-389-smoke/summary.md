# #389 CPU comparison

**Collection validation only.** Short collection-validation smoke; longer timing datasets follow.
Do not use timing or hardware-counter comparisons from this dataset as performance evidence.

Collector: none; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| atlas-fill / moving / sentence / 1 | 61.14 | 60.11 | -1.70% | -1.89% to -1.50% |
| atlas-fill / stationary / sentence / 1 | 58.41 | 59.27 | +1.47% | +0.89% to +2.05% |
| dashed-stroke / moving / sentence / 1 | 7762.34 | 7798.31 | +0.92% | -4.37% to +6.21% |
| dashed-stroke / stationary / sentence / 1 | 7834.96 | 7324.90 | -6.20% | -8.87% to -3.54% |
| glyph-fill / moving / sentence / 1 | 1207.69 | 1201.33 | -0.53% | -0.98% to -0.07% |
| glyph-fill / stationary / sentence / 1 | 1000.22 | 915.55 | -7.85% | -11.54% to -4.16% |
| glyph-stroke / moving / sentence / 1 | 1069.66 | 1084.79 | +1.07% | -0.07% to +2.22% |
| glyph-stroke / stationary / sentence / 1 | 565.63 | 558.42 | -1.27% | -1.55% to -0.99% |
| gradient-fill / moving / sentence / 1 | 1233.62 | 1532.08 | +24.23% | +11.36% to +37.10% |
| gradient-fill / stationary / sentence / 1 | 950.32 | 1021.39 | +7.52% | +2.93% to +12.11% |
| gradient-stroke / moving / sentence / 1 | 1072.70 | 904.88 | -13.93% | -20.00% to -7.86% |
| gradient-stroke / stationary / sentence / 1 | 575.26 | 572.00 | -0.57% | -0.59% to -0.54% |
| image-fill / moving / sentence / 1 | 1337.45 | 1223.91 | -7.94% | -11.41% to -4.47% |
| image-fill / stationary / sentence / 1 | 952.83 | 937.92 | -1.56% | -1.63% to -1.50% |
| path-fill / moving / sentence / 1 | 31.13 | 31.98 | +2.75% | +1.41% to +4.09% |
| path-fill / stationary / sentence / 1 | 20.90 | 21.24 | +1.67% | +1.44% to +1.90% |
| path-stroke / moving / sentence / 1 | 22.73 | 21.09 | -6.83% | -9.66% to -4.00% |
| path-stroke / stationary / sentence / 1 | 10.04 | 9.98 | -0.65% | -0.95% to -0.35% |
| shadow-fill / moving / sentence / 1 | 2528.35 | 2502.19 | -1.03% | -1.17% to -0.90% |
| shadow-fill / stationary / sentence / 1 | 2546.73 | 2982.50 | +17.23% | +7.21% to +27.24% |
| transformed-fill / moving / sentence / 1 | 1352.78 | 1206.26 | -9.92% | -14.48% to -5.35% |
| transformed-fill / stationary / sentence / 1 | 949.79 | 945.36 | -0.47% | -0.68% to -0.25% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
