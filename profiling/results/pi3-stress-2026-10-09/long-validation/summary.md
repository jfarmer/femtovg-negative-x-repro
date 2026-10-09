# #389 CPU comparison

**Collection validation only.** Artifact archiving/download overlapped early measurement. Retained as collection validation only; use the independent long-clean repeat.
Do not use timing or hardware-counter comparisons from this dataset as performance evidence.

Collector: none; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| cache-glyph-fill-2048-64-50-grouped-triangle-gradient-true-389-moving / moving / sentence / 1 | 98103.73 | 96819.02 | -1.12% | -1.33% to -0.82% |
| mixed-fill-1024-16-50-shuffled-triangle-solid-true-389-stationary / stationary / sentence / 1 | 17595.00 | 17417.99 | -1.00% | -1.41% to -0.50% |
| tiny-path-fill-512-16-50-grouped-rectangle-solid-true-389-stationary / stationary / sentence / 1 | 4446.46 | 4623.45 | +2.60% | +1.29% to +3.28% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
