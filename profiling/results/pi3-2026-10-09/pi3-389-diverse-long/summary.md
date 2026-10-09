# #389 CPU comparison

Collector: none; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| glyph-fill / moving / diverse / 1 | 4635.91 | 4614.15 | -0.25% | -0.67% to -0.10% |
| glyph-fill / stationary / diverse / 1 | 3066.81 | 3035.76 | -0.49% | -1.07% to -0.33% |
| gradient-fill / moving / diverse / 1 | 4688.40 | 4618.15 | -1.64% | -1.85% to -1.03% |
| gradient-fill / stationary / diverse / 1 | 3112.15 | 3069.40 | -1.34% | -1.56% to -1.14% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
