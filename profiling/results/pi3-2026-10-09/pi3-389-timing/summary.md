# #389 CPU comparison

Collector: none; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| atlas-fill / moving / sentence / 1 | 56.79 | 56.86 | +0.10% | -0.69% to +0.45% |
| atlas-fill / stationary / sentence / 1 | 56.67 | 56.77 | +0.04% | -0.06% to +0.56% |
| dashed-stroke / moving / sentence / 1 | 7676.29 | 7590.67 | -1.19% | -1.36% to -0.45% |
| dashed-stroke / stationary / sentence / 1 | 7673.60 | 7608.71 | -0.91% | -1.46% to -0.17% |
| glyph-fill / moving / sentence / 1 | 1305.93 | 1208.18 | -7.75% | -7.92% to -1.48% |
| glyph-fill / stationary / sentence / 1 | 941.52 | 992.92 | +2.29% | -6.15% to +9.09% |
| glyph-stroke / moving / sentence / 1 | 1002.42 | 906.65 | -9.50% | -11.08% to -7.51% |
| glyph-stroke / stationary / sentence / 1 | 565.34 | 615.67 | +9.66% | -3.17% to +12.27% |
| gradient-fill / moving / sentence / 1 | 1254.23 | 1312.66 | +0.90% | -2.87% to +5.47% |
| gradient-fill / stationary / sentence / 1 | 991.39 | 956.14 | -4.19% | -8.87% to +1.33% |
| gradient-stroke / moving / sentence / 1 | 930.75 | 979.04 | +5.19% | -8.80% to +8.85% |
| gradient-stroke / stationary / sentence / 1 | 575.20 | 566.40 | -0.96% | -2.12% to +4.38% |
| image-fill / moving / sentence / 1 | 1260.67 | 1303.17 | -1.58% | -4.60% to +5.84% |
| image-fill / stationary / sentence / 1 | 1021.41 | 955.03 | -5.00% | -10.10% to -0.51% |
| path-fill / moving / sentence / 1 | 29.73 | 30.04 | +0.39% | +0.22% to +0.89% |
| path-fill / stationary / sentence / 1 | 20.01 | 20.03 | +0.69% | +0.58% to +1.66% |
| path-stroke / moving / sentence / 1 | 19.89 | 19.88 | -0.51% | -0.77% to +0.27% |
| path-stroke / stationary / sentence / 1 | 9.01 | 8.93 | +0.38% | -0.80% to +0.68% |
| shadow-fill / moving / sentence / 1 | 2617.24 | 2588.42 | -1.12% | -1.50% to -0.88% |
| shadow-fill / stationary / sentence / 1 | 2612.71 | 2587.03 | -0.89% | -1.50% to -0.58% |
| transformed-fill / moving / sentence / 1 | 1222.75 | 1277.19 | +0.63% | -2.60% to +6.13% |
| transformed-fill / stationary / sentence / 1 | 953.53 | 1007.71 | +5.68% | -10.08% to +9.92% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
