# #389 CPU comparison

Collector: none; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| offscreen-fill-batched / stationary / sentence / 1 | 711.76 | 584.07 | -17.94% | -20.09% to -17.87% |
| visible-fill-grid / stationary / sentence / 1 | 5651.02 | 5517.65 | -2.80% | -3.05% to -1.44% |
| visible-stroke-batched / stationary / sentence / 1 | 2777.10 | 2595.35 | -6.22% | -7.90% to -3.96% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
