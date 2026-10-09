# #389 CPU comparison

Collector: none; instrumentation: plain.

Elapsed times cover measured frames only. Process resource usage covers the whole child, including startup and warmup.
Stage percentiles include per-frame clock overhead; allocation builds include atomic accounting overhead.
Paired deltas are within-round fixed/master ratios. IQR describes observed variation, not a confidence interval.

| Workload | Base us/frame | Fixed us/frame | Median paired change | Change IQR |
| --- | ---: | ---: | ---: | ---: |
| cache-glyph-fill-2048-1-50-grouped-triangle-gradient-true-389-moving / moving / sentence / 1 | 51662.81 | 52080.43 | +1.94% | -0.93% to +2.23% |
| cache-glyph-fill-2048-1-50-grouped-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 52474.49 | 50724.29 | -3.34% | -5.84% to -1.54% |
| cache-glyph-fill-2048-1-50-shuffled-triangle-gradient-true-389-moving / moving / sentence / 1 | 53662.01 | 52179.35 | -3.52% | -5.19% to -0.13% |
| cache-glyph-fill-2048-1-50-shuffled-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 52413.38 | 48261.28 | -7.92% | -8.65% to -4.85% |
| cache-glyph-fill-2048-1024-50-grouped-triangle-gradient-true-389-moving / moving / sentence / 1 | 129089.96 | 124843.36 | -3.29% | -3.46% to -3.24% |
| cache-glyph-fill-2048-1024-50-grouped-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 102731.90 | 102847.92 | -0.74% | -0.93% to -0.32% |
| cache-glyph-fill-2048-1024-50-shuffled-triangle-gradient-true-389-moving / moving / sentence / 1 | 127278.99 | 129669.92 | +1.53% | +0.32% to +1.91% |
| cache-glyph-fill-2048-1024-50-shuffled-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 103040.14 | 101777.25 | -1.03% | -1.13% to -0.72% |
| cache-glyph-fill-2048-16-50-grouped-triangle-gradient-true-389-moving / moving / sentence / 1 | 85970.56 | 85226.67 | -0.87% | -1.08% to -0.65% |
| cache-glyph-fill-2048-16-50-grouped-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 86390.64 | 85084.24 | -1.43% | -3.16% to -1.38% |
| cache-glyph-fill-2048-16-50-shuffled-triangle-gradient-true-389-moving / moving / sentence / 1 | 88352.63 | 85325.58 | -3.18% | -3.62% to -2.67% |
| cache-glyph-fill-2048-16-50-shuffled-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 85773.73 | 84631.52 | -1.30% | -1.38% to -0.20% |
| cache-glyph-fill-2048-256-50-grouped-triangle-gradient-true-389-moving / moving / sentence / 1 | 105650.47 | 104832.98 | -1.08% | -1.23% to -0.93% |
| cache-glyph-fill-2048-256-50-grouped-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 100347.84 | 99453.01 | -1.01% | -1.20% to -0.84% |
| cache-glyph-fill-2048-256-50-shuffled-triangle-gradient-true-389-moving / moving / sentence / 1 | 107556.35 | 104901.66 | -1.25% | -1.97% to -0.25% |
| cache-glyph-fill-2048-256-50-shuffled-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 100083.18 | 99137.98 | -0.87% | -1.20% to -0.22% |
| cache-glyph-fill-2048-4-50-grouped-triangle-gradient-true-389-moving / moving / sentence / 1 | 70380.33 | 68950.85 | -2.08% | -2.45% to -1.85% |
| cache-glyph-fill-2048-4-50-grouped-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 68675.01 | 67587.33 | -1.58% | -2.42% to -1.49% |
| cache-glyph-fill-2048-4-50-shuffled-triangle-gradient-true-389-moving / moving / sentence / 1 | 69800.10 | 68295.83 | -2.62% | -2.65% to -1.69% |
| cache-glyph-fill-2048-4-50-shuffled-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 69850.86 | 67681.41 | -2.91% | -4.16% to -2.79% |
| cache-glyph-fill-2048-64-50-grouped-triangle-gradient-true-389-moving / moving / sentence / 1 | 96977.19 | 96165.50 | -1.72% | -2.57% to +0.12% |
| cache-glyph-fill-2048-64-50-grouped-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 96063.81 | 94620.72 | -1.45% | -2.51% to -1.39% |
| cache-glyph-fill-2048-64-50-shuffled-triangle-gradient-true-389-moving / moving / sentence / 1 | 94363.60 | 96218.80 | +2.00% | +0.47% to +2.20% |
| cache-glyph-fill-2048-64-50-shuffled-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 93435.37 | 92755.19 | -0.73% | -0.85% to +0.37% |
| mixed-fill-1024-16-10-grouped-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 11796.47 | 11613.23 | -1.89% | -2.07% to -1.59% |
| mixed-fill-1024-16-10-grouped-triangle-solid-true-389-stationary / stationary / sentence / 1 | 11474.22 | 11266.44 | -1.81% | -1.97% to +2.15% |
| mixed-fill-1024-16-10-shuffled-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 11502.47 | 11415.54 | -1.16% | -1.42% to -0.82% |
| mixed-fill-1024-16-10-shuffled-triangle-solid-true-389-stationary / stationary / sentence / 1 | 11984.33 | 11434.32 | -4.16% | -4.37% to -3.31% |
| mixed-fill-1024-16-50-grouped-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 18174.79 | 17468.89 | -3.80% | -5.43% to -3.24% |
| mixed-fill-1024-16-50-grouped-triangle-solid-true-389-stationary / stationary / sentence / 1 | 17171.55 | 17899.42 | +4.24% | +0.03% to +5.89% |
| mixed-fill-1024-16-50-shuffled-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 18387.25 | 17921.99 | -2.53% | -3.33% to -2.24% |
| mixed-fill-1024-16-50-shuffled-triangle-solid-true-389-stationary / stationary / sentence / 1 | 18777.79 | 17809.25 | -3.78% | -6.36% to -2.66% |
| mixed-fill-1024-16-90-grouped-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 24016.16 | 23234.41 | -2.79% | -3.42% to -2.43% |
| mixed-fill-1024-16-90-grouped-triangle-solid-true-389-stationary / stationary / sentence / 1 | 23925.20 | 23002.33 | -1.93% | -5.59% to -1.71% |
| mixed-fill-1024-16-90-shuffled-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 23777.33 | 23500.32 | -2.06% | -2.38% to -1.31% |
| mixed-fill-1024-16-90-shuffled-triangle-solid-true-389-stationary / stationary / sentence / 1 | 23323.74 | 23000.74 | -0.65% | -1.50% to +0.38% |
| mixed-stroke-1024-16-10-grouped-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 7780.41 | 8261.91 | +6.19% | -1.20% to +7.01% |
| mixed-stroke-1024-16-10-grouped-triangle-solid-true-389-stationary / stationary / sentence / 1 | 7853.47 | 7760.24 | -1.61% | -3.91% to -0.36% |
| mixed-stroke-1024-16-10-shuffled-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 8498.05 | 8429.52 | -0.63% | -4.93% to +3.29% |
| mixed-stroke-1024-16-10-shuffled-triangle-solid-true-389-stationary / stationary / sentence / 1 | 7448.72 | 7411.42 | -0.50% | -4.34% to +4.22% |
| mixed-stroke-1024-16-50-grouped-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 13964.67 | 13680.94 | -1.68% | -5.90% to -1.67% |
| mixed-stroke-1024-16-50-grouped-triangle-solid-true-389-stationary / stationary / sentence / 1 | 13250.72 | 12886.99 | -2.74% | -3.26% to -2.74% |
| mixed-stroke-1024-16-50-shuffled-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 14124.16 | 12818.71 | -2.61% | -6.29% to -2.05% |
| mixed-stroke-1024-16-50-shuffled-triangle-solid-true-389-stationary / stationary / sentence / 1 | 13739.17 | 13510.47 | -2.30% | -2.39% to -1.98% |
| mixed-stroke-1024-16-90-grouped-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 18244.88 | 17968.21 | -1.95% | -4.22% to -0.29% |
| mixed-stroke-1024-16-90-grouped-triangle-solid-true-389-stationary / stationary / sentence / 1 | 18552.42 | 18025.03 | -2.61% | -3.29% to -2.52% |
| mixed-stroke-1024-16-90-shuffled-triangle-gradient-true-389-stationary / stationary / sentence / 1 | 19142.85 | 18608.85 | -3.05% | -3.74% to -1.62% |
| mixed-stroke-1024-16-90-shuffled-triangle-solid-true-389-stationary / stationary / sentence / 1 | 18677.59 | 18219.15 | -2.01% | -2.27% to -0.75% |
| tiny-path-fill-4096-16-50-grouped-rectangle-solid-false-389-stationary / stationary / sentence / 1 | 42236.22 | 39087.20 | -3.14% | -5.30% to -1.89% |
| tiny-path-fill-4096-16-50-grouped-rectangle-solid-true-389-stationary / stationary / sentence / 1 | 45251.49 | 42674.66 | -6.45% | -7.35% to -3.83% |
| tiny-path-fill-4096-16-50-grouped-triangle-solid-false-389-stationary / stationary / sentence / 1 | 36277.59 | 35591.80 | -1.89% | -2.43% to -1.49% |
| tiny-path-fill-4096-16-50-grouped-triangle-solid-true-389-stationary / stationary / sentence / 1 | 41784.80 | 38507.46 | -7.85% | -8.24% to -6.78% |
| tiny-path-fill-512-16-50-grouped-rectangle-solid-false-389-stationary / stationary / sentence / 1 | 4167.63 | 3800.57 | -6.58% | -7.80% to -4.82% |
| tiny-path-fill-512-16-50-grouped-rectangle-solid-true-389-stationary / stationary / sentence / 1 | 4328.59 | 4535.33 | +4.78% | -2.23% to +5.75% |
| tiny-path-fill-512-16-50-grouped-triangle-solid-false-389-stationary / stationary / sentence / 1 | 3506.96 | 3175.68 | -9.85% | -10.84% to -4.35% |
| tiny-path-fill-512-16-50-grouped-triangle-solid-true-389-stationary / stationary / sentence / 1 | 4039.06 | 3752.93 | -7.08% | -8.54% to -2.78% |
| tiny-path-fill-64-16-50-grouped-rectangle-solid-false-389-stationary / stationary / sentence / 1 | 482.15 | 521.32 | +8.53% | -0.04% to +8.66% |
| tiny-path-fill-64-16-50-grouped-rectangle-solid-true-389-stationary / stationary / sentence / 1 | 512.03 | 521.20 | -1.71% | -4.06% to +0.57% |
| tiny-path-fill-64-16-50-grouped-triangle-solid-false-389-stationary / stationary / sentence / 1 | 420.78 | 419.59 | +1.63% | -2.77% to +3.72% |
| tiny-path-fill-64-16-50-grouped-triangle-solid-true-389-stationary / stationary / sentence / 1 | 487.60 | 458.50 | -0.84% | -3.40% to -0.78% |
| tiny-path-stroke-4096-16-50-grouped-line-solid-false-389-stationary / stationary / sentence / 1 | 25448.88 | 25910.54 | +1.81% | +0.51% to +4.70% |
| tiny-path-stroke-4096-16-50-grouped-line-solid-true-389-stationary / stationary / sentence / 1 | 25850.53 | 25675.18 | -0.91% | -2.24% to -0.79% |
| tiny-path-stroke-4096-16-50-grouped-rectangle-solid-false-389-stationary / stationary / sentence / 1 | 27420.55 | 26176.80 | -4.78% | -4.86% to -1.75% |
| tiny-path-stroke-4096-16-50-grouped-rectangle-solid-true-389-stationary / stationary / sentence / 1 | 26754.18 | 27594.45 | +3.18% | -1.27% to +4.23% |
| tiny-path-stroke-4096-16-50-grouped-triangle-solid-false-389-stationary / stationary / sentence / 1 | 26278.68 | 26428.49 | +0.68% | -0.04% to +0.72% |
| tiny-path-stroke-4096-16-50-grouped-triangle-solid-true-389-stationary / stationary / sentence / 1 | 25320.04 | 26414.22 | -0.41% | -1.08% to +3.43% |
| tiny-path-stroke-512-16-50-grouped-line-solid-false-389-stationary / stationary / sentence / 1 | 2256.94 | 2206.92 | -1.79% | -2.01% to -0.89% |
| tiny-path-stroke-512-16-50-grouped-line-solid-true-389-stationary / stationary / sentence / 1 | 2264.46 | 2123.34 | -6.91% | -7.98% to +1.54% |
| tiny-path-stroke-512-16-50-grouped-rectangle-solid-false-389-stationary / stationary / sentence / 1 | 2312.70 | 2560.51 | +10.73% | +4.85% to +10.88% |
| tiny-path-stroke-512-16-50-grouped-rectangle-solid-true-389-stationary / stationary / sentence / 1 | 2494.04 | 2515.73 | +0.45% | -3.36% to +3.96% |
| tiny-path-stroke-512-16-50-grouped-triangle-solid-false-389-stationary / stationary / sentence / 1 | 2361.83 | 2184.07 | -7.74% | -7.92% to -4.14% |
| tiny-path-stroke-512-16-50-grouped-triangle-solid-true-389-stationary / stationary / sentence / 1 | 2278.16 | 2215.63 | -3.29% | -4.04% to -0.49% |
| tiny-path-stroke-64-16-50-grouped-line-solid-false-389-stationary / stationary / sentence / 1 | 263.75 | 247.49 | -6.86% | -7.46% to -0.16% |
| tiny-path-stroke-64-16-50-grouped-line-solid-true-389-stationary / stationary / sentence / 1 | 255.72 | 250.49 | -2.04% | -4.91% to +4.13% |
| tiny-path-stroke-64-16-50-grouped-rectangle-solid-false-389-stationary / stationary / sentence / 1 | 266.34 | 285.71 | +5.80% | +4.11% to +6.63% |
| tiny-path-stroke-64-16-50-grouped-rectangle-solid-true-389-stationary / stationary / sentence / 1 | 273.80 | 282.11 | -1.63% | -2.73% to +2.04% |
| tiny-path-stroke-64-16-50-grouped-triangle-solid-false-389-stationary / stationary / sentence / 1 | 275.78 | 274.88 | -0.56% | -3.80% to +1.00% |
| tiny-path-stroke-64-16-50-grouped-triangle-solid-true-389-stationary / stationary / sentence / 1 | 271.63 | 278.62 | +3.09% | -1.91% to +6.28% |

Full stage, allocation, memory, and counter statistics are in `summary.json`. Raw samples and commands are in `runs.jsonl` and `experiment.json`.
