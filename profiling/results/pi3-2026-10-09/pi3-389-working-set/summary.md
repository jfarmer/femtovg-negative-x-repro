# #389 CPU comparison

Collector: none; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| glyph-fill / moving / diverse / 1 | 4442.00 | 4917.34 | +10.78% | +10.51% to +11.00% |
| glyph-fill / moving / diverse / 8 | 38475.87 | 38283.95 | -0.44% | -0.58% to -0.29% |
| glyph-fill / stationary / diverse / 1 | 3284.36 | 2929.05 | -10.52% | -14.05% to -10.41% |
| glyph-fill / stationary / diverse / 8 | 38312.28 | 38704.60 | +0.90% | +0.45% to +1.33% |
| gradient-fill / moving / diverse / 1 | 4480.25 | 4927.45 | +9.10% | +5.81% to +9.59% |
| gradient-fill / moving / diverse / 8 | 39071.74 | 38831.25 | -0.62% | -1.25% to -0.06% |
| gradient-fill / stationary / diverse / 1 | 3137.94 | 2952.31 | -5.66% | -7.57% to -1.50% |
| gradient-fill / stationary / diverse / 8 | 39142.57 | 39116.38 | -0.50% | -0.98% to +0.56% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
