# #389 CPU comparison

Collector: perf; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| offscreen-fill-batched / stationary / sentence / 1 | 750.14 | 552.36 | -26.37% | -26.42% to -21.70% |
| visible-stroke-batched / stationary / sentence / 1 | 2945.92 | 2758.41 | -6.37% | -8.45% to +0.68% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
