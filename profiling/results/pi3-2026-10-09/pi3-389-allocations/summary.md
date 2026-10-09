# #389 CPU comparison

Collector: none; instrumentation: allocations.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| atlas-fill / moving / sentence / 1 | 58.89 | 59.06 | +0.29% | +0.06% to +0.44% |
| atlas-fill / stationary / sentence / 1 | 57.17 | 57.74 | +0.60% | +0.47% to +66.68% |
| dashed-stroke / moving / sentence / 1 | 8320.53 | 7870.29 | -5.89% | -6.13% to -0.30% |
| dashed-stroke / stationary / sentence / 1 | 8002.27 | 8274.39 | +3.24% | +1.34% to +4.38% |
| glyph-fill / moving / sentence / 1 | 1241.44 | 1233.73 | -0.46% | -0.62% to -0.37% |
| glyph-fill / stationary / sentence / 1 | 969.09 | 1252.42 | +29.24% | +14.02% to +33.81% |
| glyph-stroke / moving / sentence / 1 | 932.03 | 1229.59 | +31.93% | +15.44% to +33.68% |
| glyph-stroke / stationary / sentence / 1 | 578.71 | 569.52 | -1.59% | -11.88% to -1.37% |
| gradient-fill / moving / sentence / 1 | 1257.30 | 1248.94 | -0.72% | -0.92% to +11.63% |
| gradient-fill / stationary / sentence / 1 | 986.71 | 975.38 | -0.72% | -1.19% to -0.66% |
| gradient-stroke / moving / sentence / 1 | 945.88 | 933.02 | -1.59% | -9.55% to -1.40% |
| gradient-stroke / stationary / sentence / 1 | 666.37 | 576.69 | -13.46% | -17.73% to -7.43% |
| image-fill / moving / sentence / 1 | 1339.05 | 1249.65 | -6.68% | -17.35% to -3.45% |
| image-fill / stationary / sentence / 1 | 982.04 | 970.97 | -1.13% | -4.76% to -0.66% |
| path-fill / moving / sentence / 1 | 32.18 | 31.94 | -0.98% | -2.23% to +1.32% |
| path-fill / stationary / sentence / 1 | 20.56 | 20.66 | +0.63% | +0.57% to +1.08% |
| path-stroke / moving / sentence / 1 | 20.46 | 20.48 | +0.13% | -0.13% to +0.70% |
| path-stroke / stationary / sentence / 1 | 8.85 | 8.73 | -1.45% | -9.56% to +2.27% |
| shadow-fill / moving / sentence / 1 | 2681.50 | 2573.79 | -4.13% | -6.27% to +4.39% |
| shadow-fill / stationary / sentence / 1 | 2591.44 | 2648.43 | +2.20% | -4.92% to +7.51% |
| transformed-fill / moving / sentence / 1 | 1249.77 | 1241.23 | -0.68% | -0.73% to -0.54% |
| transformed-fill / stationary / sentence / 1 | 983.41 | 978.44 | -0.17% | -15.12% to +0.10% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
